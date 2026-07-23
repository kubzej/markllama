import { generateChatTurn, cancelGeneration } from '$lib/tauri/ollama';
import { diffDocuments, type DiffLine } from '$lib/tauri/diff';
import { saveChat } from '$lib/tauri/chats';
import type { ImageAttachment } from '$lib/images';
import { projectState } from './project.svelte';
import { chatsState } from './chats.svelte';
import { attachedFilesBlock, filesToSendPerTurn, filesToSendForNewTurn } from './attachedFiles';

export type TurnMode = 'chat' | 'write';
export type TurnStatus =
	| 'generating'
	| 'done'
	| 'reviewing'
	| 'applied'
	| 'discarded'
	| 'error'
	| 'cancelled';

export interface AttachedFile {
	path: string;
	content: string;
}

export interface ConversationTurn {
	id: string;
	mode: TurnMode;
	model: string;
	instruction: string;
	images: ImageAttachment[];
	/** Every file considered "in scope" for this turn (the active document, auto-included, plus
	 *  any extra files explicitly attached) — always the full content as of this turn, even for a
	 *  file that ended up *not* being resent to Ollama because it was unchanged since the last
	 *  turn that sent it. See the context-policy comment below for why that distinction matters. */
	attachedFiles: AttachedFile[];
	thinkingText: string;
	/**
	 * Running character count of the streamed answer — for a 'chat' turn this is just a liveness
	 * signal (the real text lands in `responseText`); for a 'write' turn the raw text isn't shown
	 * at all (it becomes the diff), so this is the only proof of life while generating.
	 */
	answerLength: number;
	status: TurnStatus;
	errorMessage: string | null;
	/** 'chat' mode only — the assistant's plain-text reply, rendered as a bubble. */
	responseText: string | null;
	/** 'write' mode only — which file this turn targets. */
	targetFile: string | null;
	diff: DiffLine[] | null;
	pendingText: string | null;
}

/**
 * # Context policy — read this before touching `buildHistory`/`filesToSendForNewTurn`/`runChat`/
 * `runWrite`
 *
 * `buildHistory()` turns this store's own turns into real request history for project-scoped
 * chats. What must never happen:
 *   - A turn from *this* chat leaking into a *different* chat's request. `reset()`/`loadTurns()`
 *     fully replace `turns` — there is no shared mutable state that could let a stale turn
 *     survive a chat switch.
 *   - Images are never replayed in history — only the *current* turn's own images are ever sent.
 *
 * A file's content (the active document, always in scope, plus anything explicitly attached) is
 * only ever transmitted once per distinct version: both `filesToSendForNewTurn()` and
 * `filesToSendPerTurn()` walk turns in order remembering the most recently sent content per path,
 * so a file unchanged since the last time it was sent gets skipped rather than resent. The single
 * most recent turn that actually changed a given path keeps its content uncompacted in
 * `buildHistory()`'s replayed message — that's the model's only remaining way to see that
 * content, so it must never be stripped out while it's still the current version of that file. If
 * a future change needs more than this, ask first rather than slipping it in while adding an
 * unrelated feature.
 */
function wasCancelled(turn: ConversationTurn): boolean {
	return turn.status === 'cancelled';
}

const SAVE_DEBOUNCE_MS = 500;

function createConversationState() {
	let turns = $state<ConversationTurn[]>([]);
	let chatCreatedAt = $state<string | null>(null);
	let saveHandle: ReturnType<typeof setTimeout> | undefined;
	/** The real prompt-token count Ollama reported for the most recently completed turn — lets
	 *  the UI show an actual number once available, alongside its pre-send character estimate. */
	let lastPromptTokenCount = $state<number | null>(null);

	const activeTurn = $derived(turns.length > 0 ? turns[turns.length - 1] : null);
	const isBusy = $derived(
		activeTurn?.status === 'generating' || activeTurn?.status === 'reviewing'
	);
	const isGenerating = $derived(activeTurn?.status === 'generating');

	/**
	 * Turns this chat's own past turns into request history — compacted, not replayed verbatim.
	 * A "chat" turn keeps its instruction + the assistant's actual reply. A "write" turn is
	 * compacted to a short marker (the document itself already reflects what changed — resending
	 * the full old document text into every future request would be pure waste). Failed/cancelled
	 * turns are skipped; they never produced anything worth the model seeing again.
	 */
	function buildHistory(): { role: 'user' | 'assistant'; content: string }[] {
		const messages: { role: 'user' | 'assistant'; content: string }[] = [];
		const sendPlan = filesToSendPerTurn(turns);
		for (const turn of turns) {
			if (turn.status === 'error' || turn.status === 'cancelled') continue;
			const changed = sendPlan.get(turn.id) ?? [];
			const unchanged = turn.attachedFiles.filter((file) => !changed.includes(file));
			const unchangedNote =
				unchanged.length > 0
					? `\n[${unchanged.map((file) => file.path).join(', ')} — unchanged, already shown above]`
					: '';
			const content = attachedFilesBlock(changed) + turn.instruction + unchangedNote;
			messages.push({ role: 'user', content });
			if (turn.mode === 'chat' && turn.responseText) {
				messages.push({ role: 'assistant', content: turn.responseText });
			} else if (turn.mode === 'write') {
				if (turn.status === 'applied') {
					messages.push({
						role: 'assistant',
						content: `[Wrote changes to ${turn.targetFile}]`
					});
				} else if (turn.status === 'discarded') {
					messages.push({
						role: 'assistant',
						content: `[Proposed changes to ${turn.targetFile} — discarded]`
					});
				}
			}
		}
		return messages;
	}

	function scheduleSave() {
		clearTimeout(saveHandle);
		saveHandle = setTimeout(flushSave, SAVE_DEBOUNCE_MS);
	}

	async function flushSave() {
		clearTimeout(saveHandle);
		const root = projectState.rootPath;
		if (!root || turns.length === 0) return;

		let chatId = chatsState.activeChatId;
		if (!chatId) {
			chatId = crypto.randomUUID();
			chatsState.setActiveChatId(chatId);
		}
		if (!chatCreatedAt) chatCreatedAt = new Date().toISOString();
		const title = turns[0].instruction.slice(0, 60) || 'New chat';

		try {
			await saveChat(root, {
				id: chatId,
				title,
				createdAt: chatCreatedAt,
				updatedAt: new Date().toISOString(),
				turns
			});
			void chatsState.refresh();
		} catch (err) {
			console.error('Failed to save chat', err);
		}
	}

	async function runChat(
		model: string,
		instruction: string,
		images: ImageAttachment[],
		candidateFiles: AttachedFile[],
		numCtx: number | null,
		thinking: boolean,
		webSearch: boolean
	): Promise<void> {
		const history = buildHistory();
		const filesToSend = filesToSendForNewTurn(turns, candidateFiles);
		turns.push({
			id: crypto.randomUUID(),
			mode: 'chat',
			model,
			instruction,
			images,
			attachedFiles: candidateFiles,
			thinkingText: '',
			answerLength: 0,
			status: 'generating',
			errorMessage: null,
			responseText: null,
			targetFile: null,
			diff: null,
			pendingText: null
		});
		// Re-read through the reactive array rather than keeping the pre-push literal — Svelte 5
		// proxies nested state on access through the array, and mutating the literal directly
		// silently detaches it from reactivity.
		const turn = turns[turns.length - 1];

		try {
			const finalText = await generateChatTurn({
				generationId: turn.id,
				model,
				mode: 'chat',
				history,
				targetDocument: null,
				attachedFiles: filesToSend,
				instruction,
				images: images.map((image) => image.base64),
				numCtx,
				thinking,
				webSearch,
				onChunk: (chunk) => {
					turn.answerLength += chunk.length;
				},
				onThinking: (chunk) => {
					turn.thinkingText += chunk;
				},
				onPromptEvalCount: (count) => {
					lastPromptTokenCount = count;
				}
			});
			if (wasCancelled(turn)) return;
			turn.responseText = finalText;
			turn.status = 'done';
		} catch (err) {
			if (wasCancelled(turn)) return;
			turn.status = 'error';
			turn.errorMessage = err instanceof Error ? err.message : String(err);
		} finally {
			scheduleSave();
		}
	}

	async function runWrite(
		model: string,
		targetFile: string,
		targetContent: string,
		instruction: string,
		images: ImageAttachment[],
		candidateFiles: AttachedFile[],
		numCtx: number | null,
		thinking: boolean,
		webSearch: boolean
	): Promise<void> {
		const history = buildHistory();
		const filesToSend = filesToSendForNewTurn(turns, candidateFiles);
		turns.push({
			id: crypto.randomUUID(),
			mode: 'write',
			model,
			instruction,
			images,
			attachedFiles: candidateFiles,
			thinkingText: '',
			answerLength: 0,
			status: 'generating',
			errorMessage: null,
			responseText: null,
			targetFile,
			diff: null,
			pendingText: null
		});
		const turn = turns[turns.length - 1];

		try {
			const finalText = await generateChatTurn({
				generationId: turn.id,
				model,
				mode: 'write',
				history,
				targetDocument: targetContent,
				attachedFiles: filesToSend,
				instruction,
				images: images.map((image) => image.base64),
				numCtx,
				thinking,
				webSearch,
				onChunk: (chunk) => {
					turn.answerLength += chunk.length;
				},
				onThinking: (chunk) => {
					turn.thinkingText += chunk;
				},
				onPromptEvalCount: (count) => {
					lastPromptTokenCount = count;
				}
			});
			if (wasCancelled(turn)) return;
			turn.diff = await diffDocuments(targetContent, finalText);
			if (wasCancelled(turn)) return;
			turn.pendingText = finalText;
			turn.status = 'reviewing';
		} catch (err) {
			if (wasCancelled(turn)) return;
			turn.status = 'error';
			turn.errorMessage = err instanceof Error ? err.message : String(err);
		} finally {
			scheduleSave();
		}
	}

	/**
	 * Stops the active generation immediately so the user can act right away — doesn't wait for
	 * Rust/Ollama to actually wind down. The real cancel signal (which closes the HTTP connection
	 * so Ollama stops generating on its end too) is fired in the background.
	 */
	function cancelActive() {
		if (!activeTurn || activeTurn.status !== 'generating') return;
		activeTurn.status = 'cancelled';
		void cancelGeneration();
	}

	function applyActive(apply: (text: string) => void) {
		if (!activeTurn || activeTurn.status !== 'reviewing' || activeTurn.pendingText == null) return;
		apply(activeTurn.pendingText);
		activeTurn.status = 'applied';
		scheduleSave();
	}

	function discardActive() {
		if (!activeTurn || activeTurn.status !== 'reviewing') return;
		activeTurn.status = 'discarded';
		scheduleSave();
	}

	function reset() {
		clearTimeout(saveHandle);
		turns = [];
		chatCreatedAt = null;
	}

	function loadTurns(loadedTurns: ConversationTurn[], createdAt: string) {
		clearTimeout(saveHandle);
		turns = loadedTurns;
		chatCreatedAt = createdAt;
	}

	return {
		get turns() {
			return turns;
		},
		get activeTurn() {
			return activeTurn;
		},
		get isBusy() {
			return isBusy;
		},
		get isGenerating() {
			return isGenerating;
		},
		get lastPromptTokenCount() {
			return lastPromptTokenCount;
		},
		runChat,
		runWrite,
		cancelActive,
		applyActive,
		discardActive,
		reset,
		loadTurns
	};
}

export const conversationState = createConversationState();
