<script lang="ts">
	import { untrack } from 'svelte';
	import { sessionState } from '$lib/stores/session.svelte';

	let { modelName }: { modelName: string } = $props();

	// Deliberately read once — this seeds the editable local fields from whatever was saved
	// before; each model gets its own component instance (keyed #each in SettingsModal), so
	// `modelName` never actually changes under a live instance.
	const existing = untrack(() => sessionState.getModelNote(modelName));
	let alias = $state(existing.alias);
	let description = $state(existing.description);
	let justSaved = $state(false);
	let debounceHandle: ReturnType<typeof setTimeout> | undefined;
	let flashHandle: ReturnType<typeof setTimeout> | undefined;

	// Debounced (not just onblur) so an edit still persists even if the modal is closed via
	// Escape — a focused input removed from the DOM never fires `blur`, so relying on that
	// alone silently drops in-progress text. The pending setTimeout survives unmount and still
	// calls into the store, which does not depend on this component's lifecycle.
	function flush() {
		clearTimeout(debounceHandle);
		sessionState.setModelNote(modelName, { alias, description });
		justSaved = true;
		clearTimeout(flashHandle);
		flashHandle = setTimeout(() => (justSaved = false), 1200);
	}

	function scheduleSave() {
		clearTimeout(debounceHandle);
		debounceHandle = setTimeout(flush, 500);
	}
</script>

<div class="rounded-lg border border-neutral-200 p-3 dark:border-neutral-800">
	<div class="mb-2 flex items-center justify-between gap-2">
		<p class="truncate font-mono text-xs text-neutral-500 dark:text-neutral-400">{modelName}</p>
		{#if justSaved}
			<span class="text-[11px] text-emerald-600 dark:text-emerald-400">Saved</span>
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
		class="w-full resize-none rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm text-neutral-900 outline-none focus:border-accent dark:border-neutral-700 dark:bg-neutral-950 dark:text-neutral-100"
	></textarea>
</div>
