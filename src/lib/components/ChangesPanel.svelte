<script lang="ts">
	import { documentState } from '$lib/stores/document.svelte';
	import { conversationState } from '$lib/stores/conversation.svelte';
	import DiffView from './DiffView.svelte';

	const activeTurn = $derived(conversationState.activeTurn);

	function handleApply() {
		conversationState.applyActive((text) => {
			documentState.content = text;
		});
	}

	function handleDiscard() {
		conversationState.discardActive();
	}
</script>

<aside
	class="flex min-h-0 flex-[2] flex-col overflow-hidden rounded-2xl bg-white shadow-sm ring-1 ring-neutral-200/70 dark:bg-neutral-900 dark:ring-white/[0.06]"
>
	<div
		class="border-b border-neutral-200/70 px-3.5 py-2.5 text-xs font-medium text-neutral-500 dark:border-white/[0.06] dark:text-neutral-400"
	>
		Changes
	</div>

	<div class="flex-1 overflow-auto p-3 text-sm">
		{#if activeTurn?.status === 'reviewing' && activeTurn.diff}
			<DiffView diff={activeTurn.diff} />
		{:else}
			<p class="text-sm text-neutral-400 dark:text-neutral-500">No changes to review yet.</p>
		{/if}
	</div>

	{#if activeTurn?.status === 'reviewing'}
		<div
			class="flex items-center justify-end gap-2 border-t border-neutral-200/70 px-3.5 py-2.5 dark:border-white/[0.06]"
		>
			<button
				class="rounded-lg px-2.5 py-1.5 text-sm text-neutral-600 transition-colors duration-150 hover:bg-neutral-900/5 dark:text-neutral-400 dark:hover:bg-white/[0.06]"
				onclick={handleDiscard}
			>
				Discard
			</button>
			<button
				class="rounded-lg bg-accent px-3 py-1.5 text-sm font-medium text-white transition-colors duration-150 hover:bg-accent-dark"
				onclick={handleApply}
			>
				Apply
			</button>
		</div>
	{/if}
</aside>
