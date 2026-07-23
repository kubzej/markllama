<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import Toolbar from '$lib/components/Toolbar.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Editor from '$lib/components/Editor.svelte';
	import ChatLog from '$lib/components/ChatLog.svelte';
	import ChangesPanel from '$lib/components/ChangesPanel.svelte';
	import FileSwitchConfirmDialog from '$lib/components/FileSwitchConfirmDialog.svelte';
	import ModelInfoDialog from '$lib/components/ModelInfoDialog.svelte';
	import { documentState } from '$lib/stores/document.svelte';
	import { sessionState } from '$lib/stores/session.svelte';
	import { projectState } from '$lib/stores/project.svelte';
	import { uiState } from '$lib/stores/ui.svelte';

	$effect(() => {
		const title = documentState.dirty ? `${documentState.filename} — Edited` : documentState.filename;
		getCurrentWindow().setTitle(`Markllama — ${title}`);
	});
</script>

<div class="flex h-screen flex-col bg-neutral-100 dark:bg-neutral-950">
	<Toolbar />
	{#if sessionState.status === 'disconnected'}
		<div
			class="shrink-0 border-b border-amber-200/70 bg-amber-50 px-3 py-1.5 text-xs text-amber-800 dark:border-amber-900/40 dark:bg-amber-950/40 dark:text-amber-300"
		>
			Ollama isn't running. Start it with <code class="font-mono">ollama serve</code> (or open the
			Ollama app) to enable generation — you can still edit the document manually.
		</div>
	{/if}
	{#if projectState.error && !projectState.isOpen}
		<div
			class="flex shrink-0 items-center justify-between gap-2 border-b border-red-200/70 bg-red-50 px-3 py-1.5 text-xs text-red-800 dark:border-red-900/40 dark:bg-red-950/40 dark:text-red-300"
		>
			<span>Couldn't open that folder: {projectState.error}</span>
			<button
				aria-label="Dismiss"
				onclick={() => (projectState.error = null)}
				class="shrink-0 rounded p-0.5 hover:bg-red-900/10 dark:hover:bg-white/10"
			>
				<svg viewBox="0 0 24 24" class="size-3.5" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
					<path d="M18 6 6 18M6 6l12 12" />
				</svg>
			</button>
		</div>
	{/if}
	<div class="flex flex-1 gap-3 overflow-hidden p-3">
		{#if projectState.isOpen && uiState.sidebarOpen}
			<Sidebar />
		{/if}
		<main class="flex-1 overflow-hidden">
			<Editor />
		</main>
		<div class="flex flex-1 flex-col gap-3">
			<ChatLog />
			<ChangesPanel />
		</div>
	</div>
</div>

<FileSwitchConfirmDialog />
<ModelInfoDialog />
