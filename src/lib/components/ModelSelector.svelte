<script lang="ts">
	import { sessionState } from '$lib/stores/session.svelte';
	import { uiState } from '$lib/stores/ui.svelte';
	import type { OllamaModel } from '$lib/tauri/ollama';

	let open = $state(false);
	let rootEl: HTMLDivElement | undefined = $state(undefined);

	function displayName(name: string): string {
		const note = sessionState.getModelNote(name);
		return note.alias.trim() || name;
	}

	function hasNote(name: string): boolean {
		const note = sessionState.getModelNote(name);
		return Boolean(note.alias.trim() || note.description.trim());
	}

	// With many installed models, the ones you've bothered to label are the ones you actually
	// know and reach for — sort those first so you're not hunting through unlabeled models.
	const sortedModels = $derived(
		[...sessionState.models].sort((a: OllamaModel, b: OllamaModel) => {
			const aNoted = hasNote(a.name) ? 0 : 1;
			const bNoted = hasNote(b.name) ? 0 : 1;
			return aNoted - bNoted;
		})
	);

	function handleWindowClick(event: MouseEvent) {
		if (open && rootEl && !rootEl.contains(event.target as Node)) {
			open = false;
		}
	}

	function handleWindowKeydown(event: KeyboardEvent) {
		if (open && event.key === 'Escape') open = false;
	}

	function selectModel(name: string) {
		sessionState.selectedModel = name;
		open = false;
	}
</script>

{#snippet modelRow(model: OllamaModel)}
	{@const note = sessionState.getModelNote(model.name)}
	{@const selected = model.name === sessionState.selectedModel}
	{@const numCtx = sessionState.getNumCtxOverride(model.name)}
	<div
		class="flex w-full items-start gap-0.5 rounded-lg {selected
			? 'bg-accent/10 ring-1 ring-inset ring-accent/25 dark:bg-accent/15'
			: ''}"
	>
		<button
			type="button"
			onclick={() => selectModel(model.name)}
			class="flex min-w-0 flex-1 items-start gap-2 rounded-lg px-2.5 py-1.5 text-left transition-colors duration-150 hover:bg-neutral-100 dark:hover:bg-white/[0.08]"
		>
			<span class="block min-w-0 flex-1">
				<span class="flex items-baseline gap-1.5">
					<span
						class="truncate text-sm font-medium {selected
							? 'text-neutral-900 dark:text-white'
							: 'text-neutral-800 dark:text-neutral-100'}"
					>
						{note.alias.trim() || model.name}
					</span>
					{#if note.alias.trim()}
						<span class="truncate font-mono text-[11px] text-neutral-400 dark:text-neutral-500"
							>{model.name}</span
						>
					{/if}
					{#if numCtx}
						<span
							title="Custom context window: {numCtx.toLocaleString()} tokens"
							class="shrink-0 rounded-full bg-neutral-100 px-1.5 py-0.5 text-[10px] font-medium text-neutral-500 dark:bg-white/10 dark:text-neutral-400"
						>
							{numCtx >= 1000 ? `${Math.round(numCtx / 1000)}K` : numCtx} ctx
						</span>
					{/if}
				</span>
				{#if note.description.trim()}
					<span class="line-clamp-2 mt-0.5 block whitespace-normal text-xs text-neutral-500 dark:text-neutral-400"
						>{note.description}</span
					>
				{/if}
			</span>
			{#if selected}
				<svg
					viewBox="0 0 24 24"
					class="mt-0.5 size-3.5 shrink-0 text-accent"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M20 6 9 17l-5-5" />
				</svg>
			{/if}
		</button>
		<button
			type="button"
			title="Model info"
			aria-label="Model info for {model.name}"
			onclick={() => uiState.openModelInfo(model.name)}
			class="shrink-0 self-start rounded-md p-1.5 text-neutral-400 transition-colors duration-150 hover:bg-neutral-900/5 hover:text-neutral-600 dark:hover:bg-white/[0.06] dark:hover:text-neutral-300"
		>
			<svg
				viewBox="0 0 24 24"
				class="size-3.5"
				fill="none"
				stroke="currentColor"
				stroke-width="1.8"
			>
				<circle cx="12" cy="12" r="9" />
				<path d="M12 11v5" stroke-linecap="round" />
				<circle cx="12" cy="8" r="0.9" fill="currentColor" stroke="none" />
			</svg>
		</button>
	</div>
{/snippet}

<svelte:window onclick={handleWindowClick} onkeydown={handleWindowKeydown} />

<div class="relative" bind:this={rootEl}>
	<button
		type="button"
		disabled={sessionState.models.length === 0}
		onclick={() => (open = !open)}
		aria-haspopup="listbox"
		aria-expanded={open}
		class="flex max-w-48 items-center gap-1 rounded-lg px-2.5 py-1.5 text-neutral-600 transition-colors duration-150 hover:bg-neutral-900/5 disabled:cursor-not-allowed disabled:opacity-40 dark:text-neutral-400 dark:hover:bg-white/[0.06]"
	>
		<span class="truncate">
			{sessionState.models.length === 0
				? 'No models found'
				: sessionState.selectedModel
					? displayName(sessionState.selectedModel)
					: 'Select a model'}
		</span>
		<svg
			viewBox="0 0 24 24"
			class="size-3.5 shrink-0"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M6 9l6 6 6-6" />
		</svg>
	</button>

	{#if open}
		<div
			class="absolute top-full left-0 z-50 mt-1 max-h-80 w-80 overflow-y-auto rounded-xl bg-white p-1 shadow-lg ring-1 ring-neutral-200/70 dark:bg-neutral-800 dark:ring-white/10"
		>
			{#each sortedModels as model (model.name)}
				{@render modelRow(model)}
			{/each}
		</div>
	{/if}
</div>
