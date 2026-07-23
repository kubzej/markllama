import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface OllamaModel {
	name: string;
}

export async function detectOllama(): Promise<boolean> {
	return invoke<boolean>('ollama_detect');
}

export async function listOllamaModels(): Promise<OllamaModel[]> {
	return invoke<OllamaModel[]>('ollama_list_models');
}

export async function supportsThinking(model: string): Promise<boolean> {
	return invoke<boolean>('ollama_supports_thinking', { model });
}

export async function supportsVision(model: string): Promise<boolean> {
	return invoke<boolean>('ollama_supports_vision', { model });
}

export interface ModelParameter {
	key: string;
	value: string;
}

export interface ModelInfo {
	architecture: string;
	parameterSize: string;
	quantization: string;
	contextLength: number | null;
	capabilities: string[];
	parameters: ModelParameter[];
}

export async function getModelInfo(model: string): Promise<ModelInfo> {
	return invoke<ModelInfo>('ollama_get_model_info', { model });
}

export async function cancelGeneration(): Promise<void> {
	await invoke('cancel_generation');
}

interface GenerationEventPayload {
	id: string;
	chunk: string;
}

interface PromptEvalEventPayload {
	id: string;
	count: number;
}

/** One message in `history` — already compacted by the caller (`conversation.svelte.ts`); this
 *  layer just relays it through unchanged. */
export interface ChatMessageInput {
	role: 'user' | 'assistant';
	content: string;
	images?: string[];
}

export interface AttachedFileInput {
	path: string;
	content: string;
}

export interface GenerateChatTurnParams {
	generationId: string;
	model: string;
	/** `'chat'` — plain conversational reply, no document involved. `'write'` — diffed against
	 *  `targetDocument`, then reviewed/applied. */
	mode: 'chat' | 'write';
	history: ChatMessageInput[];
	targetDocument: string | null;
	attachedFiles: AttachedFileInput[];
	instruction: string;
	images: string[];
	numCtx: number | null;
	thinking: boolean;
	webSearch: boolean;
	onChunk: (chunk: string) => void;
	onThinking: (chunk: string) => void;
	/** Fired once, when Ollama reports the real prompt-token count for this request — lets the
	 *  UI replace its pre-send character-based estimate with an actual number. */
	onPromptEvalCount?: (count: number) => void;
}

/**
 * `generationId` is echoed back on every emitted event so this call's listeners can ignore
 * events that belong to a *different* generation — e.g. one abandoned by a file switch whose
 * HTTP stream hadn't fully closed yet when a new generation started. Without this, chunks from
 * an abandoned generation could bleed into whichever turn is now active.
 */
export async function generateChatTurn(params: GenerateChatTurnParams): Promise<string> {
	const {
		generationId,
		model,
		mode,
		history,
		targetDocument,
		attachedFiles,
		instruction,
		images,
		numCtx,
		thinking,
		webSearch,
		onChunk,
		onThinking,
		onPromptEvalCount
	} = params;

	const unlistenChunk = await listen<GenerationEventPayload>('generation:chunk', (event) => {
		if (event.payload.id === generationId) onChunk(event.payload.chunk);
	});
	const unlistenThinking = await listen<GenerationEventPayload>(
		'generation:thinking',
		(event) => {
			if (event.payload.id === generationId) onThinking(event.payload.chunk);
		}
	);
	const unlistenPromptEval = await listen<PromptEvalEventPayload>(
		'generation:promptEvalCount',
		(event) => {
			if (event.payload.id === generationId) onPromptEvalCount?.(event.payload.count);
		}
	);

	try {
		return await invoke<string>('generate_chat_turn', {
			request: {
				generationId,
				model,
				mode,
				history,
				targetDocument,
				attachedFiles,
				instruction,
				images,
				numCtx,
				thinking,
				webSearch
			}
		});
	} finally {
		unlistenChunk();
		unlistenThinking();
		unlistenPromptEval();
	}
}
