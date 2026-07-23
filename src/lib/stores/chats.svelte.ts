import {
	listChats,
	loadChat,
	deleteChat,
	getLastActiveChatId,
	setLastActiveChatId,
	type ChatSummary
} from '$lib/tauri/chats';
import { projectState } from './project.svelte';
import { conversationState } from './conversation.svelte';

/**
 * Chat management for the currently open project — list/switch/new/delete. A chat's actual
 * message content lives in `conversationState`; this store only tracks which chat is active and
 * the lightweight summaries needed to render a switcher list. Scoped entirely to project mode —
 * single-file mode has no persisted chat at all (see `conversation.svelte.ts`'s callers).
 */
function createChatsState() {
	let chats = $state<ChatSummary[]>([]);
	let activeChatId = $state<string | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);

	async function refresh() {
		const root = projectState.rootPath;
		if (!root) {
			chats = [];
			return;
		}
		loading = true;
		try {
			chats = await listChats(root);
			error = null;
		} catch (err) {
			error = err instanceof Error ? err.message : String(err);
		} finally {
			loading = false;
		}
	}

	async function switchChat(id: string): Promise<void> {
		const root = projectState.rootPath;
		if (!root) return;
		const chat = await loadChat(root, id);
		activeChatId = chat.id;
		conversationState.loadTurns(chat.turns, chat.createdAt);
		await setLastActiveChatId(root, id);
	}

	function newChat(): void {
		activeChatId = null;
		conversationState.reset();
		const root = projectState.rootPath;
		if (root) void setLastActiveChatId(root, null);
	}

	async function removeChat(id: string): Promise<void> {
		const root = projectState.rootPath;
		if (!root) return;
		await deleteChat(root, id);
		if (activeChatId === id) {
			newChat();
		}
		await refresh();
	}

	/** Called once, right after a project finishes opening — resumes whichever chat was active
	 *  last time this project was open, if any. */
	async function restoreLastActive(): Promise<void> {
		const root = projectState.rootPath;
		if (!root) return;
		try {
			const lastId = await getLastActiveChatId(root);
			if (lastId) await switchChat(lastId);
		} catch (err) {
			error = err instanceof Error ? err.message : String(err);
		}
	}

	/** Called when a project is closed — nothing about a closed project's chats should linger. */
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
