<script lang="ts">
	import { uiState } from '$lib/stores/ui.svelte';
	import { documentState } from '$lib/stores/document.svelte';

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
			<div class="px-5 py-4">
				<h2
					id="file-switch-title"
					class="text-sm font-semibold text-neutral-800 dark:text-neutral-200"
				>
					Unsaved changes
				</h2>
				<p class="mt-1.5 text-sm text-neutral-500 dark:text-neutral-400">
					"{documentState.filename}" has unsaved changes. Save before continuing?
				</p>
			</div>
			<div
				class="flex justify-end gap-2 border-t border-neutral-200 px-5 py-3 dark:border-neutral-800"
			>
				<button
					class="rounded-md px-2.5 py-1.5 text-sm text-neutral-600 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-800"
					onclick={() => uiState.resolveFileSwitchConfirm('cancel')}
				>
					Cancel
				</button>
				<button
					class="rounded-md px-2.5 py-1.5 text-sm text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950/30"
					onclick={() => uiState.resolveFileSwitchConfirm('discard')}
				>
					Discard
				</button>
				<button
					class="rounded-md bg-accent px-2.5 py-1.5 text-sm font-medium text-white hover:bg-accent-dark"
					onclick={() => uiState.resolveFileSwitchConfirm('save')}
				>
					Save
				</button>
			</div>
		</div>
	</div>
{/if}
