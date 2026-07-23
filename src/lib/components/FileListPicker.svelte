<script lang="ts">
	import { projectState } from '$lib/stores/project.svelte';
	import type { ProjectNode } from '$lib/tauri/project';

	let {
		open = $bindable(false),
		onSelect,
		excludePaths = [],
		emptyLabel = 'No Markdown files in this project.'
	}: {
		open: boolean;
		onSelect: (file: { path: string; name: string }) => void;
		/** Paths to leave out of the list entirely — e.g. the file already auto-included as the
		 *  active document, which would be redundant to offer for explicit attachment too. */
		excludePaths?: string[];
		emptyLabel?: string;
	} = $props();

	function flatten(node: ProjectNode | null): { path: string; name: string }[] {
		if (!node) return [];
		if (node.kind === 'file') return [{ path: node.path, name: node.name }];
		return node.children.flatMap(flatten);
	}

	const files = $derived(flatten(projectState.tree).filter((file) => !excludePaths.includes(file.path)));

	function handleWindowKeydown(event: KeyboardEvent) {
		if (open && event.key === 'Escape') open = false;
	}
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#if open}
	<div
		class="absolute top-full left-0 z-50 mt-1 max-h-64 w-64 overflow-y-auto rounded-xl bg-white p-1 shadow-lg ring-1 ring-neutral-200/70 dark:bg-neutral-800 dark:ring-white/10"
	>
		{#if files.length === 0}
			<p class="px-2.5 py-1.5 text-xs text-neutral-400 dark:text-neutral-500">{emptyLabel}</p>
		{:else}
			{#each files as file (file.path)}
				<button
					type="button"
					onclick={() => {
						onSelect(file);
						open = false;
					}}
					class="flex w-full items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-left text-sm text-neutral-700 transition-colors duration-150 hover:bg-neutral-100 dark:text-neutral-200 dark:hover:bg-white/5"
				>
					<svg
						viewBox="0 0 24 24"
						class="size-3.5 shrink-0 text-neutral-400 dark:text-neutral-500"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path d="M6 3h8l4 4v14a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z" />
						<path d="M14 3v4h4" />
					</svg>
					<span class="min-w-0 flex-1 truncate">{file.name}</span>
				</button>
			{/each}
		{/if}
	</div>
{/if}
