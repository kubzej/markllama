<script lang="ts">
	import { documentState } from '$lib/stores/document.svelte';
	import { sessionState } from '$lib/stores/session.svelte';
	import { conversationState } from '$lib/stores/conversation.svelte';
	import ChatTurn from './ChatTurn.svelte';

	let instruction = $state('');
	let textareaEl: HTMLTextAreaElement;
	let scrollEl: HTMLDivElement;

	const sendDisabled = $derived(
		sessionState.status !== 'connected' || !sessionState.selectedModel || conversationState.isBusy
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
		instruction = '';
		resizeTextarea();
		await conversationState.run(
			model,
			documentState.content,
			text,
			sessionState.thinkingEnabled,
			sessionState.webSearchEnabled
		);
	}

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
				Describe a change below to get started.
			</p>
		{/if}
		<div class="space-y-3">
			{#each conversationState.turns as turn (turn.id)}
				<ChatTurn {turn} />
			{/each}
		</div>
	</div>

	<div class="flex items-end gap-2 border-t border-neutral-200/70 p-2.5 dark:border-white/[0.06]">
		<textarea
			bind:this={textareaEl}
			bind:value={instruction}
			oninput={resizeTextarea}
			onkeydown={handleKeydown}
			disabled={sessionState.status !== 'connected'}
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
