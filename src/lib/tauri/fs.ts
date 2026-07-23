import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';

const MARKDOWN_FILTER = { name: 'Markdown', extensions: ['md', 'markdown'] };

export interface OpenedDocument {
	path: string;
	content: string;
}

export async function pickAndOpenDocument(): Promise<OpenedDocument | null> {
	const selected = await open({ multiple: false, filters: [MARKDOWN_FILTER] });
	if (!selected || Array.isArray(selected)) return null;
	const content = await invoke<string>('read_document', { path: selected });
	return { path: selected, content };
}

export async function writeDocument(path: string, content: string): Promise<void> {
	await invoke('write_document', { path, content });
}

/**
 * The native save dialog's extension filter is only a suggestion — if the user types a filename
 * without an extension (or edits the default away), macOS can hand back a path with no
 * extension at all. Markdown files should always end up as `.md`, so it's enforced here rather
 * than trusting the dialog.
 */
function ensureMarkdownExtension(path: string): string {
	return /\.(md|markdown)$/i.test(path) ? path : `${path}.md`;
}

export async function pickSavePath(): Promise<string | null> {
	const selected = await save({ filters: [MARKDOWN_FILTER], defaultPath: 'untitled.md' });
	return selected ? ensureMarkdownExtension(selected) : null;
}
