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
</script>

<div class="rounded-lg border border-neutral-200 p-3 dark:border-neutral-800">
	<div class="mb-2 flex items-center justify-between gap-2">
		<div class="flex min-w-0 items-center gap-1">
			<p class="truncate font-mono text-xs text-neutral-500 dark:text-neutral-400">{modelName}</p>
			<button
				type="button"
				title="Model info"
				aria-label="Model info for {modelName}"
				onclick={() => uiState.openModelInfo(modelName)}
				class="shrink-0 rounded-md p-1 text-neutral-400 transition-colors duration-150 hover:bg-neutral-900/5 hover:text-neutral-600 dark:hover:bg-white/[0.06] dark:hover:text-neutral-300"
			>
				<svg viewBox="0 0 24 24" class="size-3.5" fill="none" stroke="currentColor" stroke-width="1.8">
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
		class="mb-2 w-full rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm text-neutral-900 outline-none focus:border-accent dark:border-neutral-700 dark:bg-neutral-950 dark:text-neutral-100"
	/>
	<textarea
		bind:value={description}
		oninput={scheduleSave}
		onblur={flush}
		rows="2"
		placeholder="What's it good for? (optional)"
		class="mb-2 w-full resize-none rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm text-neutral-900 outline-none focus:border-accent dark:border-neutral-700 dark:bg-neutral-950 dark:text-neutral-100"
	></textarea>
	<div class="mb-1 flex items-center justify-between text-[11px] text-neutral-400 dark:text-neutral-500">
		<span>Context window (num_ctx)</span>
		{#if maxContext}
			<span>max {maxContext.toLocaleString()}</span>
		{:else if infoUnavailable}
			<span title="Could not fetch this model's max context length from Ollama">max unknown</span>
		{/if}
	</div>
	<input
		type="number"
		min="1"
		max={maxContext ?? undefined}
		bind:value={numCtx}
		oninput={scheduleSave}
		onblur={flush}
		placeholder="Default"
		class="mb-1 w-full rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm text-neutral-900 outline-none focus:border-accent dark:border-neutral-700 dark:bg-neutral-950 dark:text-neutral-100"
	/>
	<p class="text-[11px] text-neutral-400 dark:text-neutral-500">
		How much of the conversation this model can "see" at once. Higher uses more memory and is
		slower; leave empty to use Ollama's own default.
	</p>
</div>
