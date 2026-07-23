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

export async function cancelGeneration(): Promise<void> {
	await invoke('cancel_generation');
}

export async function generateEdit(
	model: string,
	markdown: string,
	instruction: string,
	images: string[],
	thinking: boolean,
	webSearch: boolean,
	onChunk: (chunk: string) => void,
	onThinking: (chunk: string) => void
): Promise<string> {
	const unlistenChunk = await listen<string>('generation:chunk', (event) =>
		onChunk(event.payload)
	);
	const unlistenThinking = await listen<string>('generation:thinking', (event) =>
		onThinking(event.payload)
	);
	try {
		return await invoke<string>('generate_edit', {
			model,
			markdown,
			instruction,
			images,
			thinking,
			webSearch
		});
	} finally {
		unlistenChunk();
		unlistenThinking();
	}
}
