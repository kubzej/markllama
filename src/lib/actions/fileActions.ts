import { documentState } from '$lib/stores/document.svelte';
import { conversationState } from '$lib/stores/conversation.svelte';
import { projectState } from '$lib/stores/project.svelte';
import { uiState } from '$lib/stores/ui.svelte';
import {
	pickAndOpenDocument,
	pickSavePath,
	pickFolder,
	readDocument,
	writeDocument
} from '$lib/tauri/fs';
import { scanProject } from '$lib/tauri/project';

/**
 * Shared guard for every action about to replace the active document's content. Returns
 * 'cancel' if the switch must not proceed. If the user chooses to save and the save doesn't
 * actually clear the dirty flag (e.g. a Save As dialog was cancelled, or the write failed),
 * this is treated as a cancel rather than silently discarding the content anyway.
 */
async function confirmDiscardIfDirty(): Promise<'proceed' | 'cancel'> {
	if (!documentState.dirty) return 'proceed';
	const choice = await uiState.requestFileSwitchConfirm();
	if (choice === 'cancel') return 'cancel';
	if (choice === 'save') {
		try {
			await saveDocument();
		} catch {
			return 'cancel';
		}
		if (documentState.dirty) return 'cancel';
	}
	return 'proceed';
}

export async function openDocument(): Promise<void> {
	if ((await confirmDiscardIfDirty()) === 'cancel') return;
	const opened = await pickAndOpenDocument();
	if (!opened) return;
	projectState.close();
	documentState.load(opened.path, opened.content);
	conversationState.reset();
}

export async function openFolder(): Promise<void> {
	if ((await confirmDiscardIfDirty()) === 'cancel') return;
	const dir = await pickFolder();
	if (!dir) return;
	projectState.loading = true;
	try {
		const tree = await scanProject(dir);
		projectState.open(dir, tree);
		documentState.reset();
		conversationState.reset();
	} catch (err) {
		projectState.error = err instanceof Error ? err.message : String(err);
	} finally {
		projectState.loading = false;
	}
}

export async function switchActiveFile(path: string): Promise<void> {
	if (path === documentState.path) return;
	if ((await confirmDiscardIfDirty()) === 'cancel') return;
	const content = await readDocument(path);
	documentState.load(path, content);
	conversationState.reset();
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
