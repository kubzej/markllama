<script lang="ts">
	import { untrack } from 'svelte';
	import { sessionState, type InstructionPreset } from '$lib/stores/session.svelte';

	let { preset }: { preset: InstructionPreset } = $props();

	let name = $state(untrack(() => preset.name));
	let text = $state(untrack(() => preset.text));
	let justSaved = $state(false);
	let saveError = $state<string | null>(null);
	let debounceHandle: ReturnType<typeof setTimeout> | undefined;
	let flashHandle: ReturnType<typeof setTimeout> | undefined;

	async function flush() {
		clearTimeout(debounceHandle);
		try {
			await sessionState.updateInstructionPreset(preset.id, { name, text });
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

	function handleDelete() {
		void sessionState.deleteInstructionPreset(preset.id);
	}
</script>

<div class="rounded-lg bg-[var(--surface-bg)] p-3 ring-1 ring-[var(--surface-ring)]">
	<div class="mb-2 flex items-center justify-between gap-2">
		<input
			bind:value={name}
			oninput={scheduleSave}
			onblur={flush}
			placeholder="Instruction name"
			class="soft-input min-w-0 flex-1 rounded-md px-2.5 py-1.5 text-sm font-medium outline-none"
		/>
		<div class="flex shrink-0 items-center gap-1">
			{#if justSaved}
				<span class="text-[11px] text-emerald-600 dark:text-emerald-400">Saved</span>
			{:else if saveError}
				<span class="text-[11px] text-red-600 dark:text-red-400" title={saveError}>Save failed</span
				>
			{/if}
			<button
				type="button"
				title="Delete instruction"
				aria-label="Delete instruction {name || preset.name}"
				onclick={handleDelete}
				class="rounded-md p-1.5 text-[var(--text-muted)] transition-colors duration-150 hover:bg-[var(--control-hover)] hover:text-red-500"
			>
				<svg
					viewBox="0 0 24 24"
					class="size-3.5"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M4 7h16" />
					<path d="M10 11v6" />
					<path d="M14 11v6" />
					<path d="M6 7l1 14h10l1-14" />
					<path d="M9 7V4h6v3" />
				</svg>
			</button>
		</div>
	</div>
	<textarea
		bind:value={text}
		oninput={scheduleSave}
		onblur={flush}
		rows="4"
		placeholder="How should the model behave when this instruction is selected?"
		class="soft-input w-full resize-y rounded-md px-2.5 py-1.5 text-sm outline-none"></textarea>
</div>
