<script lang="ts">
	import type { ConversationTurn } from '$lib/stores/conversation.svelte';
	import { sessionState } from '$lib/stores/session.svelte';
	import { toDataUrl } from '$lib/images';

	let { turn }: { turn: ConversationTurn } = $props();

	let thinkingExpanded = $state(true);

	const modelLabel = $derived(sessionState.getModelNote(turn.model).alias.trim() || turn.model);

	// 'generating' covers three visually different moments — distinguish them so an idle
	// "Thinking…" label (with thinking disabled) never gets confused with actual reasoning, and
	// so there's always some visible sign of progress even with no thinking trace at all.
	const statusLabel = $derived.by(() => {
		if (turn.status !== 'generating') {
			return (
				{
					reviewing: 'Ready to review',
					applied: 'Applied',
					discarded: 'Discarded',
					error: 'Failed',
					cancelled: 'Stopped'
				} as const
			)[turn.status];
		}
		if (turn.answerLength > 0) return 'Writing…';
		if (turn.thinkingText.length > 0) return 'Thinking…';
		return 'Generating…';
	});

	const statusDotClass = $derived(
		turn.status === 'generating'
			? 'bg-accent animate-pulse'
			: turn.status === 'reviewing'
				? 'bg-amber-500'
				: turn.status === 'applied'
					? 'bg-emerald-500'
					: turn.status === 'error'
						? 'bg-red-500'
						: 'bg-neutral-300 dark:bg-neutral-600'
	);
</script>

{#if turn.images.length > 0}
	<div class="flex justify-end gap-1.5">
		{#each turn.images as image, index (index)}
			<img
				src={toDataUrl(image)}
				alt="Attached"
				class="size-14 rounded-lg object-cover ring-1 ring-neutral-200/70 dark:ring-white/10"
			/>
		{/each}
	</div>
{/if}
<div class="flex justify-end">
	<div class="max-w-[85%] rounded-2xl bg-accent px-3 py-2 text-sm text-white">
		{turn.instruction}
	</div>
</div>
<div class="flex flex-col gap-1.5">
	<div class="flex items-center gap-1.5 text-xs font-medium text-neutral-400 dark:text-neutral-500">
		<span class="truncate text-neutral-500 dark:text-neutral-400">{modelLabel}</span>
		<span class="text-neutral-300 dark:text-neutral-600">·</span>
		<span class={`size-1.5 rounded-full ${statusDotClass}`}></span>
		{statusLabel}
		{#if turn.status === 'generating' && turn.answerLength > 0}
			<span class="text-neutral-300 dark:text-neutral-600">· {turn.answerLength} chars</span>
		{/if}
	</div>

	{#if turn.thinkingText}
		<div class="max-w-[85%]">
			<button
				onclick={() => (thinkingExpanded = !thinkingExpanded)}
				class="flex items-center gap-1 text-xs text-neutral-400 hover:text-neutral-600 dark:text-neutral-500 dark:hover:text-neutral-300"
			>
				<svg
					viewBox="0 0 24 24"
					class={`size-3 transition-transform duration-150 ${thinkingExpanded ? 'rotate-90' : ''}`}
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M9 6l6 6-6 6" />
				</svg>
				Thinking
			</button>
			{#if thinkingExpanded}
				<p
					class="mt-1 rounded-2xl bg-neutral-100 px-3 py-2 text-sm whitespace-pre-wrap text-neutral-500 italic dark:bg-white/5 dark:text-neutral-400"
				>
					{turn.thinkingText}
				</p>
			{/if}
		</div>
	{/if}

	{#if turn.status === 'error'}
		<p
			class="max-w-[85%] rounded-2xl bg-red-50 px-3 py-2 text-sm text-red-600 dark:bg-red-950/30 dark:text-red-400"
		>
			{turn.errorMessage}
		</p>
	{/if}
</div>
