<script lang="ts">
	import { documentState } from '$lib/stores/document.svelte';
	import { sessionState } from '$lib/stores/session.svelte';
	import { conversationState } from '$lib/stores/conversation.svelte';
	import { projectState } from '$lib/stores/project.svelte';
	import { pickImages, readImageBase64 } from '$lib/tauri/fs';
	import { mimeTypeForPath, toDataUrl, type ImageAttachment } from '$lib/images';
	import ChatTurn from './ChatTurn.svelte';

	let instruction = $state('');
	let attachedImages = $state<ImageAttachment[]>([]);
	let textareaEl: HTMLTextAreaElement;
	let scrollEl: HTMLDivElement;

	const noFileSelected = $derived(projectState.isOpen && !documentState.path);
	const attachDisabled = $derived(
		!sessionState.modelSupportsVision || sessionState.status !== 'connected' || noFileSelected
	);

	const sendDisabled = $derived(
		sessionState.status !== 'connected' ||
			!sessionState.selectedModel ||
			conversationState.isBusy ||
			noFileSelected
	);

	function resizeTextarea() {
		if (!textareaEl) return;
		textareaEl.style.height = 'auto';
		textareaEl.style.height = `${Math.min(textareaEl.scrollHeight, 160)}px`;
	}

	async function handleGenerate() {
		const model = sessionState.selectedModel;
		const text = instruction.trim();
		if (!model || !text) return;
		const images = attachedImages;
		const numCtx = sessionState.getNumCtxOverride(model);
		instruction = '';
		attachedImages = [];
		resizeTextarea();
		await conversationState.run(
			model,
			documentState.content,
			text,
			images,
			numCtx,
			sessionState.thinkingEnabled,
			sessionState.webSearchEnabled
		);
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

	$effect(() => {
		if (!sessionState.modelSupportsVision && attachedImages.length > 0) attachedImages = [];
	});

	function handleButtonClick() {
		if (conversationState.isGenerating) {
			conversationState.cancelActive();
		} else {
			handleGenerate();
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		// Enter sends; Shift+Enter (or Cmd/Ctrl+Enter) inserts a newline instead — standard chat
		// input convention.
		if (event.key === 'Enter' && !event.shiftKey && !event.metaKey && !event.ctrlKey) {
			event.preventDefault();
			handleGenerate();
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

<div
	class="flex min-h-0 flex-[3] flex-col overflow-hidden rounded-2xl bg-white shadow-sm ring-1 ring-neutral-200/70 dark:bg-neutral-900 dark:ring-white/[0.06]"
>
	<div
		class="border-b border-neutral-200/70 px-3.5 py-2.5 text-xs font-medium text-neutral-500 dark:border-white/[0.06] dark:text-neutral-400"
	>
		Chat
	</div>

	<div bind:this={scrollEl} class="flex-1 overflow-y-auto px-3.5 py-3">
		{#if conversationState.turns.length === 0}
			<p class="text-sm text-neutral-400 dark:text-neutral-500">
				{noFileSelected
					? 'Select a Markdown file from the sidebar to start chatting.'
					: 'Describe a change below to get started.'}
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

	<div class="flex items-end gap-2 border-t border-neutral-200/70 p-2.5 dark:border-white/[0.06]">
		<button
			type="button"
			title={sessionState.modelSupportsVision
				? 'Attach an image'
				: 'This model does not support vision'}
			aria-label="Attach an image"
			disabled={attachDisabled}
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
			disabled={sessionState.status !== 'connected' || noFileSelected}
			rows="1"
			placeholder="Describe the change you want…"
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
			{conversationState.isGenerating ? 'Stop' : 'Send'}
		</button>
	</div>
</div>
