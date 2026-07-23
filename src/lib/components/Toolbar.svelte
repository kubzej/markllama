<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';
	import { documentState } from '$lib/stores/document.svelte';
	import { sessionState } from '$lib/stores/session.svelte';
	import { projectState } from '$lib/stores/project.svelte';
	import { uiState } from '$lib/stores/ui.svelte';
	import { openDocument, openFolder, saveDocument, saveDocumentAs } from '$lib/actions/fileActions';
	import Logo from './Logo.svelte';
	import OllamaStatus from './OllamaStatus.svelte';
	import ModelSelector from './ModelSelector.svelte';
	import InstructionSelector from './InstructionSelector.svelte';
	import ThinkingToggle from './ThinkingToggle.svelte';
	import WebSearchToggle from './WebSearchToggle.svelte';
	import SettingsModal from './SettingsModal.svelte';

	const OLLAMA_POLL_INTERVAL_MS = 5000;
	let pollHandle: ReturnType<typeof setInterval>;
	let unlistenMenu: Array<() => void> = [];

	let openMenuOpen = $state(false);
	let openMenuRootEl: HTMLDivElement | undefined = $state(undefined);

	function handleWindowClick(event: MouseEvent) {
		if (openMenuOpen && openMenuRootEl && !openMenuRootEl.contains(event.target as Node)) {
			openMenuOpen = false;
		}
	}

	function handleWindowKeydown(event: KeyboardEvent) {
		if (openMenuOpen && event.key === 'Escape') openMenuOpen = false;
	}

	onMount(() => {
		sessionState.loadPreferences().then(() => sessionState.refresh());
		pollHandle = setInterval(() => sessionState.refresh(), OLLAMA_POLL_INTERVAL_MS);

		Promise.all([
			listen('menu:open', () => openDocument()),
			listen('menu:open-folder', () => openFolder()),
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

<svelte:window onclick={handleWindowClick} onkeydown={handleWindowKeydown} />

<header
	class="relative z-40 flex h-12 shrink-0 items-center gap-4 bg-[var(--chrome-bg)] px-3.5 text-sm shadow-[var(--shadow-toolbar)] backdrop-blur-xl"
	data-tauri-drag-region
>
	<div class="flex shrink-0 items-center gap-1 pl-24">
		<Logo class="mr-1 size-4.5 shrink-0 rounded-[5px]" />
		<div class="relative" bind:this={openMenuRootEl}>
			<button
				title="Open…"
				aria-haspopup="menu"
				aria-expanded={openMenuOpen}
				onclick={() => (openMenuOpen = !openMenuOpen)}
				class="flex shrink-0 items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-[var(--text-secondary)] transition-colors duration-150 hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]"
			>
				<svg
					viewBox="0 0 24 24"
					class="size-4 shrink-0"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M3 7a1 1 0 0 1 1-1h5l2 2h9a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1Z" />
				</svg>
				<span class="whitespace-nowrap">Open</span>
				<svg
					viewBox="0 0 24 24"
					class="size-3 shrink-0"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M6 9l6 6 6-6" />
				</svg>
			</button>
			{#if openMenuOpen}
				<div class="app-popover absolute top-full left-0 z-50 mt-1 w-44 rounded-xl p-1">
					<button
						onclick={() => {
							openMenuOpen = false;
							openDocument();
						}}
						class="flex w-full items-center rounded-lg px-2.5 py-1.5 text-left text-sm text-[var(--text-secondary)] hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]"
					>
						Open File…
					</button>
					<button
						onclick={() => {
							openMenuOpen = false;
							openFolder();
						}}
						class="flex w-full items-center rounded-lg px-2.5 py-1.5 text-left text-sm text-[var(--text-secondary)] hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]"
					>
						Open Folder…
					</button>
				</div>
			{/if}
		</div>
		<button
			title="Save (⌘S)"
			class="flex shrink-0 items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-[var(--text-secondary)] transition-colors duration-150 hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]"
			onclick={() => saveDocument()}
		>
			<svg
				viewBox="0 0 24 24"
				class="size-4 shrink-0"
				fill="none"
				stroke="currentColor"
				stroke-width="1.8"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M5 4h11l3 3v13a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1Z" />
				<path d="M8 4v5h7V4" />
				<rect x="8" y="14" width="8" height="6" />
			</svg>
			<span class="whitespace-nowrap">Save</span>
		</button>
		{#if projectState.isOpen}
			<button
				title="Toggle sidebar"
				aria-label="Toggle sidebar"
				onclick={() => (uiState.sidebarOpen = !uiState.sidebarOpen)}
				class="flex shrink-0 items-center rounded-md p-1 text-[var(--text-muted)] transition-colors duration-150 hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]"
			>
				<svg
					viewBox="0 0 24 24"
					class="size-3.5 shrink-0"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<rect x="3" y="4" width="18" height="16" rx="2" />
					<path d="M9 4v16" />
				</svg>
			</button>
		{/if}
	</div>

	<div class="flex min-w-0 flex-1 items-center">
		<div class="toolbar-collapsible-label flex min-w-0 items-center gap-2">
			<span class="min-w-0 truncate font-medium text-[var(--text-primary)]">
				{documentState.filename}
			</span>
			{#if documentState.dirty}
				<span
					class="inline-block size-1.5 shrink-0 rounded-full bg-accent"
					aria-label="Unsaved changes"
				></span>
			{/if}
		</div>
	</div>

	<div class="flex min-w-0 shrink-0 items-center gap-1.5 whitespace-nowrap">
		<OllamaStatus />
		<ModelSelector />
		<InstructionSelector />
		<ThinkingToggle />
		<WebSearchToggle />
	</div>

	<div class="flex shrink-0 items-center">
		<button
			class="toolbar-tight-button flex shrink-0 items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-[var(--text-secondary)] transition-colors duration-150 hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]"
			onclick={() => (uiState.settingsOpen = true)}
		>
			<svg
				viewBox="0 0 24 24"
				class="size-4 shrink-0"
				fill="none"
				stroke="currentColor"
				stroke-width="1.8"
				stroke-linecap="round"
			>
				<line x1="4" y1="7" x2="20" y2="7" />
				<circle cx="15" cy="7" r="2.1" fill="currentColor" stroke="none" />
				<line x1="4" y1="17" x2="20" y2="17" />
				<circle cx="9" cy="17" r="2.1" fill="currentColor" stroke="none" />
			</svg>
			<span class="toolbar-collapsible-label whitespace-nowrap">Settings</span>
		</button>
	</div>
</header>

<SettingsModal bind:open={uiState.settingsOpen} />
