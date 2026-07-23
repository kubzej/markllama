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
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/25 p-4 backdrop-blur-sm">
		<div
			class="app-surface w-full max-w-sm overflow-hidden rounded-xl"
			role="dialog"
			aria-modal="true"
			aria-labelledby="file-switch-title"
		>
			<div class="app-panel-header flex items-center justify-between px-4 py-3">
				<h2 id="file-switch-title" class="text-sm font-semibold text-[var(--text-primary)]">
					{documentState.dirty ? 'Unsaved changes' : 'Unreviewed suggestion'}
				</h2>
				<button
					type="button"
					aria-label="Close"
					onclick={() => uiState.resolveFileSwitchConfirm('cancel')}
					class="shrink-0 rounded-md p-1 text-[var(--text-muted)] transition-colors duration-150 hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]"
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
				<p class="text-sm text-[var(--text-secondary)]">{message}</p>
			</div>
			<div class="app-panel-footer flex justify-end gap-2 px-4 py-3">
				<button
					class="rounded-lg px-2.5 py-1.5 text-sm text-[var(--text-secondary)] transition-colors duration-150 hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]"
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
