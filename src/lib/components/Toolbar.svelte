<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';
	import { documentState } from '$lib/stores/document.svelte';
	import { sessionState } from '$lib/stores/session.svelte';
	import { uiState } from '$lib/stores/ui.svelte';
	import { openDocument, saveDocument, saveDocumentAs } from '$lib/actions/fileActions';
	import Logo from './Logo.svelte';
	import OllamaStatus from './OllamaStatus.svelte';
	import ModelSelector from './ModelSelector.svelte';
	import ThinkingToggle from './ThinkingToggle.svelte';
	import WebSearchToggle from './WebSearchToggle.svelte';
	import SettingsModal from './SettingsModal.svelte';

	const OLLAMA_POLL_INTERVAL_MS = 5000;
	let pollHandle: ReturnType<typeof setInterval>;
	let unlistenMenu: Array<() => void> = [];

	onMount(() => {
		sessionState.loadPreferences().then(() => sessionState.refresh());
		pollHandle = setInterval(() => sessionState.refresh(), OLLAMA_POLL_INTERVAL_MS);

		Promise.all([
			listen('menu:open', () => openDocument()),
			listen('menu:save', () => saveDocument()),
			listen('menu:save-as', () => saveDocumentAs()),
			listen('menu:settings', () => (uiState.settingsOpen = true))
		]).then((unlisteners) => {
			unlistenMenu = unlisteners;
		});
	});

	onDestroy(() => {
		clearInterval(pollHandle);
		unlistenMenu.forEach((unlisten) => unlisten());
	});
</script>

<header
	class="flex h-12 shrink-0 items-center gap-5 border-b border-neutral-200/70 bg-neutral-50/80 px-3.5 text-sm backdrop-blur-xl dark:border-white/[0.06] dark:bg-neutral-950/70"
	data-tauri-drag-region
>
	<div class="flex shrink-0 items-center gap-1 pl-24">
		<button
			title="Open… (⌘O)"
			class="flex shrink-0 items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-neutral-600 transition-colors duration-150 hover:bg-neutral-900/5 dark:text-neutral-400 dark:hover:bg-white/[0.06]"
			onclick={() => openDocument()}
		>
			<svg viewBox="0 0 24 24" class="size-4 shrink-0" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
				<path d="M3 7a1 1 0 0 1 1-1h5l2 2h9a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1Z" />
			</svg>
			<span class="whitespace-nowrap">Open</span>
		</button>
		<button
			title="Save (⌘S)"
			class="flex shrink-0 items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-neutral-600 transition-colors duration-150 hover:bg-neutral-900/5 dark:text-neutral-400 dark:hover:bg-white/[0.06]"
			onclick={() => saveDocument()}
		>
			<svg viewBox="0 0 24 24" class="size-4 shrink-0" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
				<path d="M5 4h11l3 3v13a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1Z" />
				<path d="M8 4v5h7V4" />
				<rect x="8" y="14" width="8" height="6" />
			</svg>
			<span class="whitespace-nowrap">Save</span>
		</button>
	</div>

	<div class="flex min-w-0 flex-1 items-center gap-2">
		<Logo class="size-4.5 shrink-0 rounded-[5px]" />
		<span class="min-w-0 flex-1 truncate font-medium text-neutral-700 dark:text-neutral-300">
			{documentState.filename}
		</span>
		{#if documentState.dirty}
			<span
				class="inline-block size-1.5 shrink-0 rounded-full bg-accent"
				aria-label="Unsaved changes"
			></span>
		{/if}
	</div>

	<div class="flex shrink-0 items-center gap-2 whitespace-nowrap">
		<OllamaStatus />
		<ModelSelector />
		<span class="mx-1 h-5 w-px shrink-0 bg-neutral-200 dark:bg-white/10"></span>
		<ThinkingToggle />
		<WebSearchToggle />
	</div>

	<div class="flex shrink-0 items-center">
		<button
			class="flex shrink-0 items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-neutral-600 transition-colors duration-150 hover:bg-neutral-900/5 dark:text-neutral-400 dark:hover:bg-white/[0.06]"
			onclick={() => (uiState.settingsOpen = true)}
		>
			<svg viewBox="0 0 24 24" class="size-4 shrink-0" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
				<line x1="4" y1="7" x2="20" y2="7" />
				<circle cx="15" cy="7" r="2.1" fill="currentColor" stroke="none" />
				<line x1="4" y1="17" x2="20" y2="17" />
				<circle cx="9" cy="17" r="2.1" fill="currentColor" stroke="none" />
			</svg>
			<span class="whitespace-nowrap">Settings</span>
		</button>
	</div>
</header>

<SettingsModal bind:open={uiState.settingsOpen} />
