<script lang="ts">
	import { uiState } from '$lib/stores/ui.svelte';
	import { documentState } from '$lib/stores/document.svelte';
	import { conversationState } from '$lib/stores/conversation.svelte';

	const pendingReview = $derived(conversationState.activeTurn?.status === 'reviewing');

	const message = $derived(
		documentState.dirty && pendingReview
			? `"${documentState.filename}" has unsaved changes, and there's an AI suggestion you haven't reviewed yet. Save before continuing?`
			: documentState.dirty
				? `"${documentState.filename}" has unsaved changes. Save before continuing?`
				: `There's an AI suggestion for "${documentState.filename}" you haven't reviewed yet. Discard it?`
	);

	function handleWindowKeydown(event: KeyboardEvent) {
		if (uiState.fileSwitchConfirmPending && event.key === 'Escape') {
			uiState.resolveFileSwitchConfirm('cancel');
		}
	}
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#if uiState.fileSwitchConfirmPending}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/20 p-4">
		<div
			class="w-full max-w-sm rounded-xl border border-neutral-200 bg-white shadow-xl dark:border-neutral-800 dark:bg-neutral-900"
			role="dialog"
			aria-modal="true"
			aria-labelledby="file-switch-title"
		>
			<div
				class="flex items-center justify-between border-b border-neutral-200 px-4 py-3 dark:border-neutral-800"
			>
				<h2
					id="file-switch-title"
					class="text-sm font-semibold text-neutral-800 dark:text-neutral-200"
				>
					{documentState.dirty ? 'Unsaved changes' : 'Unreviewed suggestion'}
				</h2>
				<button
					type="button"
					aria-label="Close"
					onclick={() => uiState.resolveFileSwitchConfirm('cancel')}
					class="shrink-0 rounded-md p-1 text-neutral-400 hover:bg-neutral-100 hover:text-neutral-600 dark:hover:bg-neutral-800 dark:hover:text-neutral-300"
				>
					<svg
						viewBox="0 0 24 24"
						class="size-4"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
					>
						<path d="M18 6 6 18M6 6l12 12" />
					</svg>
				</button>
			</div>
			<div class="px-4 py-3">
				<p class="text-sm text-neutral-500 dark:text-neutral-400">{message}</p>
			</div>
			<div
				class="flex justify-end gap-2 border-t border-neutral-200 px-4 py-3 dark:border-neutral-800"
			>
				<button
					class="rounded-lg px-2.5 py-1.5 text-sm text-neutral-600 transition-colors duration-150 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-800"
					onclick={() => uiState.resolveFileSwitchConfirm('cancel')}
				>
					Cancel
				</button>
				<button
					class="rounded-lg px-2.5 py-1.5 text-sm text-red-600 transition-colors duration-150 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950/30"
					onclick={() => uiState.resolveFileSwitchConfirm('discard')}
				>
					Discard
				</button>
				{#if documentState.dirty}
					<button
						class="rounded-lg bg-accent px-2.5 py-1.5 text-sm font-medium text-white transition-colors duration-150 hover:bg-accent-dark"
						onclick={() => uiState.resolveFileSwitchConfirm('save')}
					>
						Save
					</button>
				{/if}
			</div>
		</div>
	</div>
{/if}
