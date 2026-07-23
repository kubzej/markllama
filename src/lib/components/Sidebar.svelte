<script lang="ts">
	import { projectState } from '$lib/stores/project.svelte';
	import { uiState } from '$lib/stores/ui.svelte';
	import { scanProject } from '$lib/tauri/project';
	import FileTreeNode from './FileTreeNode.svelte';

	const rootName = $derived(projectState.rootPath?.split('/').pop() ?? projectState.rootPath ?? '');

	async function refresh() {
		if (!projectState.rootPath) return;
		projectState.loading = true;
		try {
			const tree = await scanProject(projectState.rootPath);
			projectState.open(projectState.rootPath, tree);
		} catch (err) {
			projectState.error = err instanceof Error ? err.message : String(err);
		} finally {
			projectState.loading = false;
		}
	}
</script>

<aside
	class="flex w-64 shrink-0 flex-col overflow-hidden rounded-2xl bg-white shadow-sm ring-1 ring-neutral-200/70 dark:bg-neutral-900 dark:ring-white/[0.06]"
>
	<div
		class="flex items-center justify-between gap-1 border-b border-neutral-200/70 px-3 py-2.5 dark:border-white/[0.06]"
	>
		<span class="min-w-0 flex-1 truncate text-xs font-medium text-neutral-500 dark:text-neutral-400">
			{rootName}
		</span>
		<button
			title="Refresh"
			aria-label="Refresh project file tree"
			onclick={refresh}
			class="shrink-0 rounded-md p-1 text-neutral-400 transition-colors duration-150 hover:bg-neutral-900/5 hover:text-neutral-600 dark:hover:bg-white/[0.06] dark:hover:text-neutral-300"
		>
			<svg viewBox="0 0 24 24" class="size-3.5" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<path d="M20 11a8 8 0 1 0-2.3 5.7M20 5v6h-6" />
			</svg>
		</button>
		<button
			title="Hide sidebar"
			aria-label="Hide sidebar"
			onclick={() => (uiState.sidebarOpen = false)}
			class="shrink-0 rounded-md p-1 text-neutral-400 transition-colors duration-150 hover:bg-neutral-900/5 hover:text-neutral-600 dark:hover:bg-white/[0.06] dark:hover:text-neutral-300"
		>
			<svg viewBox="0 0 24 24" class="size-3.5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
				<rect x="3" y="4" width="18" height="16" rx="2" />
				<path d="M9 4v16" />
			</svg>
		</button>
	</div>

	<div class="flex-1 overflow-y-auto p-1.5">
		{#if projectState.loading}
			<p class="px-2 py-1.5 text-xs text-neutral-400 dark:text-neutral-500">Scanning…</p>
		{:else if projectState.error}
			<p class="px-2 py-1.5 text-xs text-red-600 dark:text-red-400">{projectState.error}</p>
		{:else if projectState.tree && projectState.tree.kind === 'dir' && projectState.tree.children.length === 0}
			<p class="px-2 py-1.5 text-xs text-neutral-400 dark:text-neutral-500">
				No Markdown files in this folder.
			</p>
		{:else if projectState.tree && projectState.tree.kind === 'dir'}
			{#each projectState.tree.children as child (child.path)}
				<FileTreeNode node={child} />
			{/each}
		{/if}
	</div>
</aside>
