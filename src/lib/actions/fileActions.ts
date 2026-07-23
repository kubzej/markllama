import { documentState } from '$lib/stores/document.svelte';
import { conversationState } from '$lib/stores/conversation.svelte';
import { pickAndOpenDocument, pickSavePath, writeDocument } from '$lib/tauri/fs';

export async function openDocument(): Promise<void> {
	const opened = await pickAndOpenDocument();
	if (opened) {
		documentState.load(opened.path, opened.content);
		conversationState.reset();
	}
}

export async function saveDocument(): Promise<void> {
	const path = documentState.path ?? (await pickSavePath());
	if (!path) return;
	await writeDocument(path, documentState.content);
	documentState.markSaved(path);
}

export async function saveDocumentAs(): Promise<void> {
	const path = await pickSavePath();
	if (!path) return;
	await writeDocument(path, documentState.content);
	documentState.markSaved(path);
}
