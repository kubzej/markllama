import { invoke } from '@tauri-apps/api/core';
import type { ImageAttachment } from '$lib/images';
import type { DiffLine } from './diff';

export type ChatTurnMode = 'chat' | 'write';
export type ChatTurnStatus =
	| 'generating'
	| 'done'
	| 'reviewing'
	| 'applied'
	| 'discarded'
	| 'error'
	| 'cancelled';

export interface ChatAttachedFile {
	path: string;
	content: string;
}

export interface ChatTurn {
	id: string;
	mode: ChatTurnMode;
	model: string;
	instruction: string;
	images: ImageAttachment[];
	attachedFiles: ChatAttachedFile[];
	thinkingText: string;
	answerLength: number;
	status: ChatTurnStatus;
	errorMessage: string | null;
	responseText: string | null;
	targetFile: string | null;
	diff: DiffLine[] | null;
	pendingText: string | null;
}

export interface Chat {
	id: string;
	title: string;
	createdAt: string;
	updatedAt: string;
	turns: ChatTurn[];
}

export interface ChatSummary {
	id: string;
	title: string;
	updatedAt: string;
}

export async function listChats(projectRoot: string): Promise<ChatSummary[]> {
	return invoke<ChatSummary[]>('list_chats', { projectRoot });
}

export async function loadChat(projectRoot: string, chatId: string): Promise<Chat> {
	return invoke<Chat>('load_chat', { projectRoot, chatId });
}

export async function saveChat(projectRoot: string, chat: Chat): Promise<void> {
	await invoke('save_chat', { projectRoot, chat });
}

export async function deleteChat(projectRoot: string, chatId: string): Promise<void> {
	await invoke('delete_chat', { projectRoot, chatId });
}

export async function getLastActiveChatId(projectRoot: string): Promise<string | null> {
	return invoke<string | null>('get_last_active_chat_id', { projectRoot });
}

export async function setLastActiveChatId(
	projectRoot: string,
	chatId: string | null
): Promise<void> {
	await invoke('set_last_active_chat_id', { projectRoot, chatId });
}
