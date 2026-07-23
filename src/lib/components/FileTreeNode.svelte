<script lang="ts">
	import type { ProjectNode } from '$lib/tauri/project';
	import { projectState } from '$lib/stores/project.svelte';
	import { documentState } from '$lib/stores/document.svelte';
	import { switchActiveFile } from '$lib/actions/fileActions';
	import FileTreeNode from './FileTreeNode.svelte';

	let { node, depth = 0 }: { node: ProjectNode; depth?: number } = $props();

	const rowPadding = $derived(`${8 + depth * 14}px`);
</script>

{#if node.kind === 'dir'}
	{@const expanded = projectState.isExpanded(node.path)}
	<button
		type="button"
		onclick={() => projectState.toggleExpanded(node.path)}
		style={`padding-left: ${rowPadding}`}
		class="flex w-full items-center gap-1.5 rounded-md py-1 pr-2 text-left text-sm text-neutral-600 hover:bg-neutral-900/5 dark:text-neutral-300 dark:hover:bg-white/[0.06]"
	>
		<svg
			viewBox="0 0 24 24"
			class={`size-3 shrink-0 transition-transform duration-150 ${expanded ? 'rotate-90' : ''}`}
			fill="none"
			stroke="currentColor"
			stroke-width="2.5"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M9 6l6 6-6 6" />
		</svg>
		<span class="truncate">{node.name}</span>
	</button>
	{#if expanded}
		{#each node.children as child (child.path)}
			<FileTreeNode node={child} depth={depth + 1} />
		{/each}
	{/if}
{:else}
	{@const active = node.path === documentState.path}
	<button
		type="button"
		onclick={() => switchActiveFile(node.path)}
		style={`padding-left: ${rowPadding}`}
		class="flex w-full items-center gap-1.5 rounded-md py-1 pr-2 text-left text-sm hover:bg-neutral-900/5 dark:hover:bg-white/[0.06] {active
			? 'bg-accent/10 font-medium text-neutral-900 dark:bg-accent/15 dark:text-white'
			: 'text-neutral-600 dark:text-neutral-300'}"
	>
		<svg
			viewBox="0 0 24 24"
			class="size-3 shrink-0"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M6 3h8l4 4v14a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z" />
			<path d="M14 3v4h4" />
		</svg>
		<span class="truncate">{node.name}</span>
	</button>
{/if}
