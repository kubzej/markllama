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

/**
 * `generationId` is echoed back on every emitted event so this call's listeners can ignore
 * events that belong to a *different* generation — e.g. one abandoned by a file switch whose
 * HTTP stream hadn't fully closed yet when a new generation started. Without this, chunks from
 * an abandoned generation could bleed into whichever turn is now active.
 */
export async function generateEdit(
	model: string,
	markdown: string,
	instruction: string,
	images: string[],
	numCtx: number | null,
	thinking: boolean,
	webSearch: boolean,
	generationId: string,
	onChunk: (chunk: string) => void,
	onThinking: (chunk: string) => void
): Promise<string> {
	const unlistenChunk = await listen<GenerationEventPayload>('generation:chunk', (event) => {
		if (event.payload.id === generationId) onChunk(event.payload.chunk);
	});
	const unlistenThinking = await listen<GenerationEventPayload>(
		'generation:thinking',
		(event) => {
			if (event.payload.id === generationId) onThinking(event.payload.chunk);
		}
	);
	try {
		return await invoke<string>('generate_edit', {
			generationId,
			model,
			markdown,
			instruction,
			images,
			numCtx,
			thinking,
			webSearch
		});
	} finally {
		unlistenChunk();
		unlistenThinking();
	}
}
