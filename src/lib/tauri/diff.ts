import { invoke } from '@tauri-apps/api/core';

export interface WordSpan {
	text: string;
	changed: boolean;
}

export type DiffLine =
	| { kind: 'unchanged'; text: string }
	| { kind: 'removed'; text: string }
	| { kind: 'added'; text: string }
	| { kind: 'changed'; old: WordSpan[]; new: WordSpan[] };

export async function diffDocuments(oldText: string, newText: string): Promise<DiffLine[]> {
	return invoke<DiffLine[]>('diff_documents', { old: oldText, new: newText });
}
