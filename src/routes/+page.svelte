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
