import {
	listChats,
	loadChat,
	deleteChat,
	getLastActiveChatId,
	setLastActiveChatId,
	type ChatSummary
} from '$lib/tauri/chats';
import { projectState } from './project.svelte';
import { documentState } from './document.svelte';
import { conversationState } from './conversation.svelte';

/**
 * Chat management for the currently open workspace: a project folder when one is open, otherwise
 * the one open Markdown file. A chat's actual message content lives in `conversationState`; this
 * store only tracks which chat is active and the lightweight summaries needed to render a switcher
 * list.
 */
function createChatsState() {
	let chats = $state<ChatSummary[]>([]);
	let activeChatId = $state<string | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);

	const scopeKey = $derived(projectState.rootPath ?? documentState.path);

	async function refresh() {
		const key = scopeKey;
		if (!key) {
			chats = [];
			return;
		}
		loading = true;
		try {
			chats = await listChats(key);
			error = null;
		} catch (err) {
			error = err instanceof Error ? err.message : String(err);
		} finally {
			loading = false;
		}
	}

	async function switchChat(id: string): Promise<void> {
		const key = scopeKey;
		if (!key) return;
		const chat = await loadChat(key, id);
		activeChatId = chat.id;
		conversationState.loadTurns(chat.turns, chat.createdAt);
		await setLastActiveChatId(key, id);
	}

	function newChat(): void {
		activeChatId = null;
		conversationState.reset();
		const key = scopeKey;
		if (key) void setLastActiveChatId(key, null);
	}

	async function removeChat(id: string): Promise<void> {
		const key = scopeKey;
		if (!key) return;
		await deleteChat(key, id);
		if (activeChatId === id) {
			newChat();
		}
		await refresh();
	}

	/** Called right after a project or file finishes opening — resumes whichever chat was active
	 *  last time this scope was open, if any. */
	async function restoreLastActive(): Promise<void> {
		const key = scopeKey;
		if (!key) return;
		try {
			const lastId = await getLastActiveChatId(key);
			if (lastId) await switchChat(lastId);
		} catch (err) {
			error = err instanceof Error ? err.message : String(err);
		}
	}

	/** Called when the current chat scope is closed — nothing about it should linger. */
	function clear(): void {
		chats = [];
		activeChatId = null;
		error = null;
	}

	return {
		get chats() {
			return chats;
		},
		get activeChatId() {
			return activeChatId;
		},
		get loading() {
			return loading;
		},
		get error() {
			return error;
		},
		get scopeKey() {
			return scopeKey;
		},
		refresh,
		switchChat,
		newChat,
		removeChat,
		restoreLastActive,
		clear,
		setActiveChatId(id: string | null) {
			activeChatId = id;
		}
	};
}

export const chatsState = createChatsState();
