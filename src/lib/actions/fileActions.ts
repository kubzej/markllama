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
 * Shared guard for every action about to replace the active document's content. Covers both
 * unsaved document edits AND a completed-but-unreviewed AI suggestion (`status === 'reviewing'`)
 * — discarding a diff the user hasn't looked at yet is just as much lost work as discarding
 * typed edits, even though the document's own content hasn't changed yet at that point. Returns
 * 'cancel' if the switch must not proceed. If the user chooses to save and the save doesn't
 * actually clear the dirty flag (e.g. a Save As dialog was cancelled, or the write failed),
 * this is treated as a cancel rather than silently discarding the content anyway.
 */
async function confirmDiscardIfDirty(): Promise<'proceed' | 'cancel'> {
	const pendingReview = conversationState.activeTurn?.status === 'reviewing';
	if (!documentState.dirty && !pendingReview) return 'proceed';

	const choice = await uiState.requestFileSwitchConfirm();
	if (choice === 'cancel') return 'cancel';
	if (choice === 'save' && documentState.dirty) {
		try {
			await saveDocument();
		} catch {
			return 'cancel';
		}
		if (documentState.dirty) return 'cancel';
	}
	return 'proceed';
}

/**
 * A generation still actively streaming (not yet reached 'reviewing') is speculative work, not
 * a result the user has seen and could lose — same as clicking Stop, so this cancels it outright
 * rather than prompting. Must run only after the caller has already decided to proceed with the
 * switch (never before `confirmDiscardIfDirty` — if the user cancels the switch, whatever's
 * running should keep running).
 */
function cancelActiveGenerationIfAny(): void {
	if (conversationState.isGenerating) conversationState.cancelActive();
}

export async function openDocument(): Promise<void> {
	if ((await confirmDiscardIfDirty()) === 'cancel') return;
	cancelActiveGenerationIfAny();
	const opened = await pickAndOpenDocument();
	if (!opened) return;
	projectState.close();
	documentState.load(opened.path, opened.content);
	conversationState.reset();
}

export async function openFolder(): Promise<void> {
	if ((await confirmDiscardIfDirty()) === 'cancel') return;
	cancelActiveGenerationIfAny();
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
	cancelActiveGenerationIfAny();
	const content = await readDocument(path);
	documentState.load(path, content);
	conversationState.reset();
}

/**
 * Guards against a double-click (or ⌘S fired while a Toolbar click is already in flight) both
 * opening their own native Save dialog for a brand-new, path-less document — without this, two
 * concurrent calls both read `documentState.path` as `null` and each awaits its own
 * `pickSavePath()`.
 */
let saveInFlight = false;

export async function saveDocument(): Promise<void> {
	if (saveInFlight) return;
	saveInFlight = true;
	try {
		const path = documentState.path ?? (await pickSavePath());
		if (!path) return;
		await writeDocument(path, documentState.content);
		documentState.markSaved(path);
	} finally {
		saveInFlight = false;
	}
}

export async function saveDocumentAs(): Promise<void> {
	if (saveInFlight) return;
	saveInFlight = true;
	try {
		const path = await pickSavePath();
		if (!path) return;
		await writeDocument(path, documentState.content);
		documentState.markSaved(path);
	} finally {
		saveInFlight = false;
	}
}
