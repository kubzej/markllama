<script lang="ts">
	import { untrack } from 'svelte';
	import { sessionState } from '$lib/stores/session.svelte';
	import { uiState } from '$lib/stores/ui.svelte';
	import { getModelInfo } from '$lib/tauri/ollama';

	let { modelName }: { modelName: string } = $props();

	// Deliberately read once — this seeds the editable local fields from whatever was saved
	// before; each model gets its own component instance (keyed #each in SettingsModal), so
	// `modelName` never actually changes under a live instance.
	const existing = untrack(() => sessionState.getModelNote(modelName));
	let alias = $state(existing.alias);
	let description = $state(existing.description);
	let numCtx = $state<number | undefined>(
		untrack(() => sessionState.getNumCtxOverride(modelName)) ?? undefined
	);
	let maxContext = $state<number | null>(null);
	let infoUnavailable = $state(false);
	let justSaved = $state(false);
	let saveError = $state<string | null>(null);
	let debounceHandle: ReturnType<typeof setTimeout> | undefined;
	let flashHandle: ReturnType<typeof setTimeout> | undefined;

	const CONTEXT_PRESETS = [8192, 16384, 32768, 65536, 131072];

	const contextPresets = $derived.by(() => {
		const max = maxContext;
		const values = max ? CONTEXT_PRESETS.filter((value) => value <= max) : CONTEXT_PRESETS;
		const shouldIncludeMax = max && !values.includes(max);
		return [
			{ label: 'Default', value: null },
			...values.map((value) => ({ label: formatContextValue(value), value })),
			...(shouldIncludeMax ? [{ label: 'Max', value: max }] : [])
		];
	});

	// Fetched lazily for the `max` bound on the num_ctx input. A failure here isn't fatal (the
	// input just stays unbounded), but it's surfaced with a small hint rather than swallowed
	// entirely — silently showing nothing looks identical to "this model has no known max",
	// which isn't true and was confusing.
	untrack(() => getModelInfo(modelName))
		.then((info) => {
			maxContext = info.contextLength;
		})
		.catch(() => {
			infoUnavailable = true;
		});

	function clampNumCtx(value: number): number {
		const rounded = Math.max(1, Math.round(value));
		return maxContext ? Math.min(rounded, maxContext) : rounded;
	}

	function formatContextValue(value: number): string {
		return value % 1024 === 0 ? `${Math.round(value / 1024)}K` : value.toLocaleString();
	}

	// Debounced (not just onblur) so an edit still persists even if the modal is closed via
	// Escape — a focused input removed from the DOM never fires `blur`, so relying on that
	// alone silently drops in-progress text. The pending setTimeout survives unmount and still
	// calls into the store, which does not depend on this component's lifecycle.
	async function flush() {
		clearTimeout(debounceHandle);
		const clamped = numCtx == null ? null : clampNumCtx(numCtx);
		if (clamped !== numCtx) numCtx = clamped ?? undefined;
		try {
			await Promise.all([
				sessionState.setModelNote(modelName, { alias, description }),
				sessionState.setNumCtxOverride(modelName, clamped)
			]);
			saveError = null;
			justSaved = true;
			clearTimeout(flashHandle);
			flashHandle = setTimeout(() => (justSaved = false), 1200);
		} catch (err) {
			justSaved = false;
			saveError = err instanceof Error ? err.message : String(err);
		}
	}

	function scheduleSave() {
		clearTimeout(debounceHandle);
		debounceHandle = setTimeout(flush, 500);
	}

	function applyContextPreset(value: number | null) {
		numCtx = value ?? undefined;
		void flush();
	}
</script>

<div class="rounded-lg bg-[var(--surface-bg)] p-3 ring-1 ring-[var(--surface-ring)]">
	<div class="mb-2 flex items-center justify-between gap-2">
		<div class="flex min-w-0 items-center gap-1">
			<p class="truncate font-mono text-xs text-[var(--text-secondary)]">{modelName}</p>
			<button
				type="button"
				title="Model info"
				aria-label="Model info for {modelName}"
				onclick={() => uiState.openModelInfo(modelName)}
				class="shrink-0 rounded-md p-1 text-[var(--text-muted)] transition-colors duration-150 hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]"
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
		{#if justSaved}
			<span class="shrink-0 text-[11px] text-emerald-600 dark:text-emerald-400">Saved</span>
		{:else if saveError}
			<span class="shrink-0 text-[11px] text-red-600 dark:text-red-400" title={saveError}
				>Save failed</span
			>
		{/if}
	</div>
	<input
		bind:value={alias}
		oninput={scheduleSave}
		onblur={flush}
		placeholder="Alias (optional)"
		class="soft-input mb-2 w-full rounded-md px-2.5 py-1.5 text-sm outline-none"
	/>
	<textarea
		bind:value={description}
		oninput={scheduleSave}
		onblur={flush}
		rows="2"
		placeholder="What's it good for? (optional)"
		class="soft-input mb-2 w-full resize-none rounded-md px-2.5 py-1.5 text-sm outline-none"
	></textarea>
	<div class="mb-1 flex items-center justify-between text-[11px] text-[var(--text-muted)]">
		<span>Context window (num_ctx)</span>
		{#if maxContext}
			<span>max {maxContext.toLocaleString()}</span>
		{:else if infoUnavailable}
			<span title="Could not fetch this model's max context length from Ollama">max unknown</span>
		{/if}
	</div>
	<div class="mb-2 flex flex-wrap gap-1">
		{#each contextPresets as preset (preset.label)}
			<button
				type="button"
				onclick={() => applyContextPreset(preset.value)}
				class="rounded-md px-2 py-1 text-[11px] font-medium ring-1 transition-colors duration-150 {numCtx ===
				(preset.value ?? undefined)
					? 'bg-accent/10 text-accent ring-accent/25'
					: 'text-[var(--text-secondary)] ring-[var(--surface-ring)] hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]'}"
			>
				{preset.label}
			</button>
		{/each}
	</div>
	<input
		type="number"
		min="1"
		max={maxContext ?? undefined}
		step="1024"
		bind:value={numCtx}
		oninput={scheduleSave}
		onblur={flush}
		placeholder="Default"
		class="soft-input mb-1 w-full rounded-md px-2.5 py-1.5 text-sm outline-none"
	/>
</div>
