import { generateEdit, cancelGeneration } from '$lib/tauri/ollama';
import { diffDocuments, type DiffLine } from '$lib/tauri/diff';
import type { ImageAttachment } from '$lib/images';

export type TurnStatus =
	| 'generating'
	| 'reviewing'
	| 'applied'
	| 'discarded'
	| 'error'
	| 'cancelled';

export interface ConversationTurn {
	id: string;
	model: string;
	instruction: string;
	/** Display-only, exactly like `instruction` — never read back into a request. */
	images: ImageAttachment[];
	thinkingText: string;
	/**
	 * Running character count of the streamed answer. The raw answer text itself isn't shown in
	 * chat (it becomes the diff below), but the count is — it's the only proof of life while
	 * `status` is `'generating'` and there's no thinking trace (thinking off, or not yet
	 * started), so the UI doesn't look frozen.
	 */
	answerLength: number;
	status: TurnStatus;
	errorMessage: string | null;
	diff: DiffLine[] | null;
	pendingText: string | null;
}

/**
 * CORE INVARIANT — this store is purely a local, in-memory, UI-only chat log for the user's own
 * reference (never persisted to disk, always empty on a fresh app launch, cleared when a new
 * file is opened). `turns` here must NEVER be sent to Ollama, in whole or in part.
 *
 * `run()` below only ever passes `instruction` and the CURRENT `markdown` (read fresh from
 * `documentState.content` at call-time by the caller) into `generateEdit` — never `turns` itself,
 * never a previous turn's `instruction`/`thinkingText`/`pendingText`. Every request Ollama
 * receives is system prompt + current document + current instruction, nothing else. This is the
 * whole point of the app (keep the local model's context small and cheap) — if a change to this
 * file ever threads `turns`/history into the `run()` call, that is a regression, not a feature,
 * unless the user has explicitly asked for that architecture change.
 */
/**
 * Deliberately indirect (vs. inlining `turn.status === 'cancelled'`) — TypeScript's narrowing
 * otherwise assumes `status` can't have changed across the `await`s in `run()` below just
 * because a prior check in the same function ruled it out, even though `cancelActive()` mutates
 * the very same object from outside this function while those awaits are pending.
 */
function wasCancelled(turn: ConversationTurn): boolean {
	return turn.status === 'cancelled';
}

function createConversationState() {
	let turns = $state<ConversationTurn[]>([]);

	const activeTurn = $derived(turns.length > 0 ? turns[turns.length - 1] : null);
	const isBusy = $derived(activeTurn?.status === 'generating' || activeTurn?.status === 'reviewing');
	const isGenerating = $derived(activeTurn?.status === 'generating');

	async function run(
		model: string,
		markdown: string,
		instruction: string,
		images: ImageAttachment[],
		thinking: boolean,
		webSearch: boolean
	): Promise<void> {
		turns.push({
			id: crypto.randomUUID(),
			model,
			instruction,
			images,
			thinkingText: '',
			answerLength: 0,
			status: 'generating',
			errorMessage: null,
			diff: null,
			pendingText: null
		});
		// Re-read the reference through the reactive array rather than mutating the plain object
		// literal above directly — Svelte 5 proxies nested state on access through the array, and
		// mutating the pre-push literal instead silently detaches it from reactivity (the data
		// changes, but nothing re-renders). This bit us once already; don't reintroduce it.
		const turn = turns[turns.length - 1];

		try {
			const finalText = await generateEdit(
				model,
				markdown,
				instruction,
				images.map((image) => image.base64),
				thinking,
				webSearch,
				(chunk) => {
					// Raw streamed answer text isn't shown in chat (it becomes the diff below) —
					// only its length is, as a liveness signal.
					turn.answerLength += chunk.length;
				},
				(chunk) => {
					turn.thinkingText += chunk;
				}
			);
			// A cancel in flight already set this turn's status directly (see `cancelActive`) —
			// don't let a response that arrives just after that clobber it back to 'reviewing'.
			if (wasCancelled(turn)) return;
			turn.diff = await diffDocuments(markdown, finalText);
			if (wasCancelled(turn)) return;
			turn.pendingText = finalText;
			turn.status = 'reviewing';
		} catch (err) {
			if (wasCancelled(turn)) return;
			turn.status = 'error';
			turn.errorMessage = err instanceof Error ? err.message : String(err);
		}
	}

	/**
	 * Stops the active generation immediately so the user can send the next message right away —
	 * doesn't wait for Rust/Ollama to actually wind down. The real cancel signal (which closes the
	 * HTTP connection so Ollama stops generating on its end too) is fired in the background.
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
	}

	function discardActive() {
		if (!activeTurn || activeTurn.status !== 'reviewing') return;
		activeTurn.status = 'discarded';
	}

	function reset() {
		turns = [];
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
		run,
		cancelActive,
		applyActive,
		discardActive,
		reset
	};
}

export const conversationState = createConversationState();
