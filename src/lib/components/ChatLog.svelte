<script lang="ts">
	import { documentState } from '$lib/stores/document.svelte';
	import { sessionState } from '$lib/stores/session.svelte';
	import { conversationState, type AttachedFile } from '$lib/stores/conversation.svelte';
	import { projectState } from '$lib/stores/project.svelte';
	import { chatsState } from '$lib/stores/chats.svelte';
	import { switchActiveFile } from '$lib/actions/fileActions';
	import { pickImages, readImageBase64, readDocument } from '$lib/tauri/fs';
	import { getModelInfo } from '$lib/tauri/ollama';
	import { mimeTypeForPath, toDataUrl, type ImageAttachment } from '$lib/images';
	import ChatTurn from './ChatTurn.svelte';
	import FileListPicker from './FileListPicker.svelte';

	interface AttachedFileChip {
		path: string;
		name: string;
		content: string;
	}

	let instruction = $state('');
	let attachedImages = $state<ImageAttachment[]>([]);
	let attachedFiles = $state<AttachedFileChip[]>([]);
	/** Project mode only — a one-shot arm set by picking a target file (`handleWriteTargetSelect`),
	 *  cleared again after the next Send. */
	let writeTarget = $state<{ path: string; name: string } | null>(null);
	/** Single-file mode only — a persistent toggle (there's only ever one possible target, the
	 *  one open document, so there's nothing to "pick" per message) defaulting to on, matching
	 *  this mode's original behavior of every Send being an edit. */
	let singleFileWriteMode = $state(true);
	let textareaEl: HTMLTextAreaElement;
	let scrollEl: HTMLDivElement;

	let chatMenuOpen = $state(false);
	let chatMenuRootEl: HTMLDivElement | undefined = $state(undefined);
	let attachFileMenuOpen = $state(false);
	let attachFileMenuRootEl: HTMLDivElement | undefined = $state(undefined);
	let writeMenuOpen = $state(false);
	let writeMenuRootEl: HTMLDivElement | undefined = $state(undefined);

	let modelMaxContext = $state<number | null>(null);

	// Neither a project nor a single document is open — there is nothing to write to and nothing
	// to discuss yet, so the whole input is disabled rather than silently generating against an
	// empty untitled document.
	const nothingOpen = $derived(!projectState.isOpen && !documentState.path);

	const attachImageDisabled = $derived(
		!sessionState.modelSupportsVision || sessionState.status !== 'connected' || nothingOpen
	);
	const sendDisabled = $derived(
		sessionState.status !== 'connected' ||
			!sessionState.selectedModel ||
			conversationState.isBusy ||
			nothingOpen
	);
	const currentChatTitle = $derived(
		chatsState.chats.find((chat) => chat.id === chatsState.activeChatId)?.title ?? 'New chat'
	);

	/** The file this send would write to, if any — `writeTarget` in project mode, or the single
	 *  open document when `singleFileWriteMode` is on in single-file mode. `null` means a plain
	 *  chat turn. */
	const effectiveWriteTarget = $derived(
		projectState.isOpen
			? writeTarget
			: singleFileWriteMode && documentState.path
				? { path: documentState.path, name: documentState.filename }
				: null
	);

	// Rough chars-per-token≈4 heuristic — there's no tokenizer available client-side. Covers prior
	// turns' text, the active document (always in scope) and any extra attached files' content,
	// and the draft being typed. Slightly over-counts in the common case where the active document
	// hasn't changed since it was last actually sent (see conversation.svelte.ts's de-dup), but
	// that's the safe direction for an estimate to be wrong in.
	const CHARS_PER_TOKEN = 4;
	const SYSTEM_PROMPT_TOKEN_ESTIMATE = 80;
	const estimatedTokens = $derived.by(() => {
		let chars = 0;
		for (const turn of conversationState.turns) {
			chars += turn.instruction.length;
			chars += turn.responseText?.length ?? 0;
		}
		chars += instruction.length;
		if (documentState.path && !effectiveWriteTarget) chars += documentState.content.length;
		for (const file of attachedFiles) chars += file.content.length;
		return SYSTEM_PROMPT_TOKEN_ESTIMATE + Math.ceil(chars / CHARS_PER_TOKEN);
	});
	const contextWindow = $derived(
		sessionState.selectedModel
			? sessionState.getNumCtxOverride(sessionState.selectedModel) ?? modelMaxContext
			: null
	);
	const contextUsageRatio = $derived(
		contextWindow ? Math.min(1, estimatedTokens / contextWindow) : null
	);

	$effect(() => {
		const model = sessionState.selectedModel;
		if (!model) {
			modelMaxContext = null;
			return;
		}
		getModelInfo(model)
			.then((info) => {
				modelMaxContext = info.contextLength;
			})
			.catch(() => {
				modelMaxContext = null;
			});
	});

	function resizeTextarea() {
		if (!textareaEl) return;
		textareaEl.style.height = 'auto';
		textareaEl.style.height = `${Math.min(textareaEl.scrollHeight, 160)}px`;
	}

	/** The active document is always in scope for a chat turn — no need to attach it manually —
	 *  plus whatever extra files were explicitly attached. `excludePath` drops the write target
	 *  from this list when one is armed, since its content is sent separately as the document
	 *  being edited, not as a reference attachment. */
	function gatherCandidateFiles(excludePath: string | null): AttachedFile[] {
		const files: AttachedFile[] = [];
		if (documentState.path && documentState.path !== excludePath) {
			files.push({ path: documentState.path, content: documentState.content });
		}
		for (const file of attachedFiles) {
			if (file.path !== excludePath) files.push({ path: file.path, content: file.content });
		}
		return files;
	}

	/** A plain chat turn, unless a write target is in effect (`effectiveWriteTarget` — an armed
	 *  pick in project mode, or the toggle in single-file mode), in which case this instruction
	 *  becomes the "how/what to write" for that document instead. Either way, nothing generates
	 *  just from arming/toggling a target — only Send does, so the user can add context (what to
	 *  write, what to pull from earlier messages) before anything is actually generated. */
	async function handleSendMessage() {
		const model = sessionState.selectedModel;
		const text = instruction.trim();
		if (!model || !text) return;

		const target = effectiveWriteTarget;
		if (target && target.path !== documentState.path) {
			await switchActiveFile(target.path);
			if (documentState.path !== target.path) return; // switch guard cancelled — keep the draft
		}

		const images = attachedImages;
		const files = gatherCandidateFiles(target?.path ?? null);
		const numCtx = sessionState.getNumCtxOverride(model);
		instruction = '';
		attachedImages = [];
		attachedFiles = [];
		writeTarget = null; // only the one-shot project-mode arm resets — singleFileWriteMode persists
		resizeTextarea();

		if (target) {
			await conversationState.runWrite(
				model,
				target.path,
				documentState.content,
				text,
				images,
				files,
				numCtx,
				sessionState.thinkingEnabled,
				sessionState.webSearchEnabled
			);
		} else {
			await conversationState.runChat(
				model,
				text,
				images,
				files,
				numCtx,
				sessionState.thinkingEnabled,
				sessionState.webSearchEnabled
			);
		}
	}

	function handleSend() {
		void handleSendMessage();
	}

	/** Picking a target only arms it — it switches to that document (same guarded flow as clicking
	 *  it in the sidebar) so it's visible for review, but generation only happens once the user
	 *  actually hits Send with their instruction; see `handleSendChat`. */
	async function handleWriteTargetSelect(file: { path: string; name: string }) {
		if (file.path !== documentState.path) {
			await switchActiveFile(file.path);
			if (documentState.path !== file.path) return; // the switch guard was cancelled
		}
		writeTarget = file;
		textareaEl?.focus();
	}

	async function addImages() {
		const paths = await pickImages();
		for (const path of paths) {
			const base64 = await readImageBase64(path);
			attachedImages = [...attachedImages, { base64, mimeType: mimeTypeForPath(path) }];
		}
	}

	function removeImage(index: number) {
		attachedImages = attachedImages.filter((_, i) => i !== index);
	}

	async function handleAttachFileSelect(file: { path: string; name: string }) {
		if (attachedFiles.some((f) => f.path === file.path)) return;
		try {
			const content = await readDocument(file.path);
			attachedFiles = [...attachedFiles, { path: file.path, name: file.name, content }];
		} catch (err) {
			console.error('Failed to read attached file', err);
		}
	}

	function removeAttachedFile(path: string) {
		attachedFiles = attachedFiles.filter((f) => f.path !== path);
	}

	$effect(() => {
		if (!sessionState.modelSupportsVision && attachedImages.length > 0) attachedImages = [];
	});

	function handleButtonClick() {
		if (conversationState.isGenerating) {
			conversationState.cancelActive();
		} else {
			handleSend();
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		// Enter sends; Shift+Enter (or Cmd/Ctrl+Enter) inserts a newline instead — standard chat
		// input convention. Enter always means "send", never "write to document" — that's always
		// an explicit separate button click.
		if (event.key === 'Enter' && !event.shiftKey && !event.metaKey && !event.ctrlKey) {
			event.preventDefault();
			handleSend();
		}
	}

	function handleWindowClick(event: MouseEvent) {
		const target = event.target as Node;
		if (chatMenuOpen && chatMenuRootEl && !chatMenuRootEl.contains(target)) chatMenuOpen = false;
		if (attachFileMenuOpen && attachFileMenuRootEl && !attachFileMenuRootEl.contains(target)) {
			attachFileMenuOpen = false;
		}
		if (writeMenuOpen && writeMenuRootEl && !writeMenuRootEl.contains(target)) {
			writeMenuOpen = false;
		}
	}

	$effect(() => {
		// Reading these properties is what makes this effect re-fire as the conversation grows
		// (a new turn, or thinking text streaming in) — it's how the log auto-scrolls live.
		const turns = conversationState.turns;
		void turns.length;
		for (const turn of turns) {
			void turn.thinkingText.length;
			void turn.answerLength;
			void turn.status;
		}
		if (scrollEl) {
			requestAnimationFrame(() => {
				scrollEl.scrollTop = scrollEl.scrollHeight;
			});
		}
	});
</script>

<svelte:window onclick={handleWindowClick} />

<div
	class="flex min-h-0 flex-[3] flex-col rounded-2xl bg-white shadow-sm ring-1 ring-neutral-200/70 dark:bg-neutral-900 dark:ring-white/[0.06]"
>
	{#if projectState.isOpen}
		<div class="relative border-b border-neutral-200/70 dark:border-white/[0.06]" bind:this={chatMenuRootEl}>
			<div class="flex items-center gap-2 px-3.5 py-2">
				<button
					type="button"
					aria-haspopup="menu"
					aria-expanded={chatMenuOpen}
					onclick={() => (chatMenuOpen = !chatMenuOpen)}
					class="flex min-w-0 flex-1 items-center gap-1 rounded-md px-1 py-1 text-left text-xs font-medium text-neutral-500 transition-colors duration-150 hover:bg-neutral-900/5 dark:text-neutral-400 dark:hover:bg-white/[0.06]"
				>
					<span class="min-w-0 flex-1 truncate">{currentChatTitle}</span>
					<svg viewBox="0 0 24 24" class="size-3 shrink-0" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="M6 9l6 6 6-6" />
					</svg>
				</button>
				{#if contextWindow}
					<div
						class="flex shrink-0 items-center gap-1.5"
						title="~{estimatedTokens.toLocaleString()} / {contextWindow.toLocaleString()} tokens (estimate){conversationState.lastPromptTokenCount ? ` · last request used ${conversationState.lastPromptTokenCount.toLocaleString()}` : ''}"
					>
						<div class="h-1 w-10 overflow-hidden rounded-full bg-neutral-100 dark:bg-white/10">
							<div
								class="h-full rounded-full {contextUsageRatio && contextUsageRatio > 0.85 ? 'bg-red-500' : 'bg-accent'}"
								style={`width: ${Math.round((contextUsageRatio ?? 0) * 100)}%`}
							></div>
						</div>
						<span class="text-[10px] text-neutral-400 dark:text-neutral-500">
							~{estimatedTokens >= 1000 ? `${Math.round(estimatedTokens / 1000)}K` : estimatedTokens}
						</span>
					</div>
				{/if}
			</div>
			{#if chatMenuOpen}
				<div
					class="absolute top-full left-0 z-50 mt-1 max-h-72 w-64 overflow-y-auto rounded-xl bg-white p-1 shadow-lg ring-1 ring-neutral-200/70 dark:bg-neutral-800 dark:ring-white/10"
				>
					<button
						type="button"
						onclick={() => {
							chatsState.newChat();
							chatMenuOpen = false;
						}}
						class="flex w-full items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-left text-sm font-medium text-accent transition-colors duration-150 hover:bg-neutral-100 dark:hover:bg-white/5"
					>
						<svg viewBox="0 0 24 24" class="size-3.5 shrink-0" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
							<path d="M12 5v14M5 12h14" />
						</svg>
						New chat
					</button>
					{#if chatsState.chats.length > 0}
						<div class="my-1 border-t border-neutral-200 dark:border-neutral-700"></div>
						{#each chatsState.chats as chat (chat.id)}
							<div
								class="flex w-full items-center gap-0.5 rounded-lg {chat.id === chatsState.activeChatId
									? 'bg-accent/10 ring-1 ring-inset ring-accent/25 dark:bg-accent/15'
									: ''}"
							>
								<button
									type="button"
									onclick={() => {
										void chatsState.switchChat(chat.id);
										chatMenuOpen = false;
									}}
									class="min-w-0 flex-1 truncate rounded-lg px-2.5 py-1.5 text-left text-sm text-neutral-700 transition-colors duration-150 hover:bg-neutral-100 dark:text-neutral-200 dark:hover:bg-white/5"
								>
									{chat.title}
								</button>
								<button
									type="button"
									title="Delete chat"
									aria-label="Delete chat {chat.title}"
									onclick={() => void chatsState.removeChat(chat.id)}
									class="shrink-0 rounded-md p-1 text-neutral-400 transition-colors duration-150 hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-950/30 dark:hover:text-red-400"
								>
									<svg viewBox="0 0 24 24" class="size-3.5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
										<path d="M4 7h16M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2m2 0-.6 12.2A2 2 0 0 1 14.4 21H9.6a2 2 0 0 1-2-1.8L7 7" />
									</svg>
								</button>
							</div>
						{/each}
					{/if}
				</div>
			{/if}
		</div>
	{:else}
		<div
			class="border-b border-neutral-200/70 px-3.5 py-2.5 text-xs font-medium text-neutral-500 dark:border-white/[0.06] dark:text-neutral-400"
		>
			Chat
		</div>
	{/if}

	<div bind:this={scrollEl} class="flex-1 overflow-y-auto px-3.5 py-3">
		{#if conversationState.turns.length === 0}
			<p class="text-sm text-neutral-400 dark:text-neutral-500">
				{nothingOpen
					? 'Open a file or folder to start.'
					: 'Ask a question, discuss the document, or write to it when you know what you want.'}
			</p>
		{/if}
		<div class="space-y-3">
			{#each conversationState.turns as turn (turn.id)}
				<ChatTurn {turn} />
			{/each}
		</div>
	</div>

	{#if attachedImages.length > 0}
		<div class="flex flex-wrap gap-1.5 border-t border-neutral-200/70 px-2.5 py-2.5 dark:border-white/[0.06]">
			{#each attachedImages as image, index (index)}
				<div class="relative">
					<img
						src={toDataUrl(image)}
						alt="Attached"
						class="size-12 rounded-md object-cover ring-1 ring-neutral-200/70 dark:ring-white/10"
					/>
					<button
						title="Remove"
						aria-label="Remove attached image"
						onclick={() => removeImage(index)}
						class="absolute -top-1.5 -right-1.5 flex size-4 items-center justify-center rounded-full bg-neutral-700 text-white hover:bg-neutral-600 dark:bg-neutral-500 dark:hover:bg-neutral-400"
					>
						<svg viewBox="0 0 24 24" class="size-2.5" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round">
							<path d="M18 6 6 18M6 6l12 12" />
						</svg>
					</button>
				</div>
			{/each}
		</div>
	{/if}

	{#if projectState.isOpen}
		<div
			class="relative flex flex-wrap items-center gap-1.5 border-t border-neutral-200/70 px-2.5 py-2.5 dark:border-white/[0.06]"
			bind:this={attachFileMenuRootEl}
		>
			{#if documentState.path && !writeTarget}
				<span
					title="Always included automatically — the currently active document"
					class="flex items-center gap-1 rounded-full bg-accent/10 px-2 py-1 text-xs font-medium text-accent dark:bg-accent/15"
				>
					<svg viewBox="0 0 24 24" class="size-3 shrink-0" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="M6 3h8l4 4v14a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z" />
						<path d="M14 3v4h4" />
					</svg>
					<span class="max-w-32 truncate">{documentState.filename}</span>
				</span>
			{/if}
			{#each attachedFiles as file (file.path)}
				<span
					class="flex items-center gap-1 rounded-full bg-neutral-100 px-2 py-1 text-xs text-neutral-600 dark:bg-white/10 dark:text-neutral-300"
				>
					<svg viewBox="0 0 24 24" class="size-3 shrink-0" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="M6 3h8l4 4v14a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z" />
						<path d="M14 3v4h4" />
					</svg>
					<span class="max-w-32 truncate">{file.name}</span>
					<button
						title="Remove"
						aria-label="Remove attached file {file.name}"
						onclick={() => removeAttachedFile(file.path)}
						class="shrink-0 text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-100"
					>
						<svg viewBox="0 0 24 24" class="size-3" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
							<path d="M18 6 6 18M6 6l12 12" />
						</svg>
					</button>
				</span>
			{/each}
			<button
				type="button"
				title="Attach another file"
				aria-label="Attach another file"
				onclick={() => (attachFileMenuOpen = !attachFileMenuOpen)}
				class="flex items-center gap-1 rounded-full px-2 py-1 text-xs text-neutral-500 ring-1 ring-neutral-200 transition-colors duration-150 hover:bg-neutral-900/5 dark:text-neutral-400 dark:ring-white/10 dark:hover:bg-white/[0.06]"
			>
				<svg viewBox="0 0 24 24" class="size-3 shrink-0" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
					<path d="M12 5v14M5 12h14" />
				</svg>
				Add file
			</button>
			<FileListPicker
				bind:open={attachFileMenuOpen}
				onSelect={handleAttachFileSelect}
				excludePaths={documentState.path ? [documentState.path] : []}
			/>
		</div>
	{/if}

	{#if projectState.isOpen}
		<div
			class="relative flex items-center gap-2 border-t border-neutral-200/70 px-2.5 py-2 dark:border-white/[0.06]"
			bind:this={writeMenuRootEl}
		>
			{#if writeTarget}
				<span
					class="flex items-center gap-1.5 rounded-lg bg-accent/10 px-2.5 py-1 text-xs font-medium text-accent ring-1 ring-inset ring-accent/25 dark:bg-accent/15"
				>
					<svg viewBox="0 0 24 24" class="size-3.5 shrink-0" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
						<path d="M12 20h9" />
						<path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
					</svg>
					Writing to {writeTarget.name}
					<button
						type="button"
						title="Cancel"
						aria-label="Cancel writing to {writeTarget.name}"
						onclick={() => (writeTarget = null)}
						class="shrink-0 text-accent/70 hover:text-accent"
					>
						<svg viewBox="0 0 24 24" class="size-3" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
							<path d="M18 6 6 18M6 6l12 12" />
						</svg>
					</button>
				</span>
			{:else}
				<button
					type="button"
					disabled={sendDisabled}
					onclick={() => (writeMenuOpen = !writeMenuOpen)}
					class="flex items-center gap-1.5 rounded-lg px-2.5 py-1 text-xs font-medium text-neutral-600 ring-1 ring-neutral-200 transition-colors duration-150 hover:bg-neutral-900/5 disabled:cursor-not-allowed disabled:opacity-40 dark:text-neutral-300 dark:ring-white/10 dark:hover:bg-white/[0.06]"
				>
					<svg viewBox="0 0 24 24" class="size-3.5 shrink-0" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
						<path d="M12 20h9" />
						<path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
					</svg>
					Write to document…
				</button>
			{/if}
			<FileListPicker bind:open={writeMenuOpen} onSelect={handleWriteTargetSelect} />
		</div>
	{/if}

	{#if !projectState.isOpen && documentState.path}
		<div class="flex items-center gap-2 border-t border-neutral-200/70 px-2.5 py-2 dark:border-white/[0.06]">
			<button
				type="button"
				role="switch"
				aria-checked={singleFileWriteMode}
				title={singleFileWriteMode
					? 'Writing mode — Send will propose changes to this document. Click to just chat instead.'
					: 'Chat mode — Send will just discuss. Click to write to this document instead.'}
				onclick={() => (singleFileWriteMode = !singleFileWriteMode)}
				class="flex items-center gap-1.5 rounded-lg px-2.5 py-1 text-xs font-medium ring-1 transition-colors duration-150 {singleFileWriteMode
					? 'bg-accent/10 text-accent ring-accent/25 dark:bg-accent/15'
					: 'text-neutral-600 ring-neutral-200 hover:bg-neutral-900/5 dark:text-neutral-300 dark:ring-white/10 dark:hover:bg-white/[0.06]'}"
			>
				<svg viewBox="0 0 24 24" class="size-3.5 shrink-0" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
					<path d="M12 20h9" />
					<path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
				</svg>
				{singleFileWriteMode ? `Writing to ${documentState.filename}` : 'Write to document'}
			</button>
		</div>
	{/if}

	<div class="flex items-end gap-2 border-t border-neutral-200/70 p-2.5 dark:border-white/[0.06]">
		<button
			type="button"
			title={sessionState.modelSupportsVision
				? 'Attach an image'
				: 'This model does not support vision'}
			aria-label="Attach an image"
			disabled={attachImageDisabled}
			onclick={addImages}
			class="flex shrink-0 items-center rounded-xl p-2 text-neutral-500 transition-colors duration-150 hover:bg-neutral-900/5 disabled:cursor-not-allowed disabled:opacity-40 dark:text-neutral-400 dark:hover:bg-white/[0.06]"
		>
			<svg viewBox="0 0 24 24" class="size-4.5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
				<rect x="3" y="4" width="18" height="16" rx="2" />
				<circle cx="9" cy="10" r="1.5" fill="currentColor" stroke="none" />
				<path d="M21 16l-5.5-5.5a2 2 0 0 0-2.8 0L4 19" />
			</svg>
		</button>
		<textarea
			bind:this={textareaEl}
			bind:value={instruction}
			oninput={resizeTextarea}
			onkeydown={handleKeydown}
			disabled={sessionState.status !== 'connected' || nothingOpen}
			rows="1"
			placeholder={effectiveWriteTarget
				? `What should change in ${effectiveWriteTarget.name}?`
				: 'Message…'}
			class="max-h-40 flex-1 resize-none overflow-hidden rounded-xl bg-transparent px-2.5 py-2 text-sm text-neutral-900 outline-none placeholder:text-neutral-400 disabled:opacity-50 dark:text-neutral-100 dark:placeholder:text-neutral-500"
		></textarea>
		<button
			title={conversationState.isGenerating ? 'Stop' : '⏎'}
			onclick={handleButtonClick}
			disabled={conversationState.isGenerating ? false : sendDisabled || !instruction.trim()}
			class="shrink-0 rounded-xl px-3.5 py-2 text-sm font-medium transition-colors duration-150 disabled:cursor-not-allowed disabled:opacity-40 {conversationState.isGenerating
				? 'bg-neutral-800 text-white hover:bg-neutral-700 dark:bg-neutral-200 dark:text-neutral-900 dark:hover:bg-neutral-300'
				: 'bg-accent text-white hover:bg-accent-dark'}"
		>
			{conversationState.isGenerating ? 'Stop' : effectiveWriteTarget ? 'Write' : 'Send'}
		</button>
	</div>
</div>
