<script lang="ts">
	import { sessionState } from '$lib/stores/session.svelte';

	let open = $state(false);
	let rootEl: HTMLDivElement | undefined = $state(undefined);

	const selectedName = $derived(
		sessionState.selectedInstructionPreset
			? sessionState.selectedInstructionPreset.name.trim() || 'Untitled instruction'
			: 'No instructions'
	);

	function handleWindowClick(event: MouseEvent) {
		if (open && rootEl && !rootEl.contains(event.target as Node)) {
			open = false;
		}
	}

	function handleWindowKeydown(event: KeyboardEvent) {
		if (open && event.key === 'Escape') open = false;
	}

	function selectInstruction(id: string | null) {
		sessionState.selectedInstructionId = id;
		open = false;
	}
</script>

<svelte:window onclick={handleWindowClick} onkeydown={handleWindowKeydown} />

<div class="relative" bind:this={rootEl}>
	<button
		type="button"
		title="Instruction preset"
		aria-haspopup="listbox"
		aria-expanded={open}
		onclick={() => (open = !open)}
		class="flex max-w-40 items-center gap-1 rounded-lg px-2.5 py-1.5 text-[var(--text-secondary)] transition-colors duration-150 hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]"
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
			<path d="M8 4h8" />
			<path d="M9 20h6" />
			<path d="M12 4v16" />
			<path d="M6 8h12" />
		</svg>
		<span class="toolbar-collapsible-label truncate">{selectedName}</span>
		<svg
			viewBox="0 0 24 24"
			class="toolbar-collapsible-label size-3.5 shrink-0"
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
			role="listbox"
			class="app-popover absolute top-full right-0 z-[90] mt-1 w-64 rounded-xl p-1"
		>
			<button
				type="button"
				role="option"
				aria-selected={sessionState.selectedInstructionId === null}
				onclick={() => selectInstruction(null)}
				class="flex w-full items-center justify-between gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm transition-colors duration-150 hover:bg-[var(--control-hover)] {sessionState.selectedInstructionId ===
				null
					? 'text-[var(--text-primary)]'
					: 'text-[var(--text-secondary)]'}"
			>
				<span class="truncate">No instructions</span>
				{#if sessionState.selectedInstructionId === null}
					<svg
						viewBox="0 0 24 24"
						class="size-3.5 shrink-0 text-accent"
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
			{#if sessionState.instructionPresets.length > 0}
				<div class="my-1 h-px bg-[var(--surface-ring)]"></div>
				{#each sessionState.instructionPresets as preset (preset.id)}
					<button
						type="button"
						role="option"
						aria-selected={preset.id === sessionState.selectedInstructionId}
						title={preset.text}
						onclick={() => selectInstruction(preset.id)}
						class="flex w-full items-start justify-between gap-2 rounded-lg px-2.5 py-1.5 text-left transition-colors duration-150 hover:bg-[var(--control-hover)] {preset.id ===
						sessionState.selectedInstructionId
							? 'text-[var(--text-primary)]'
							: 'text-[var(--text-secondary)]'}"
					>
						<span class="min-w-0">
							<span class="block truncate text-sm font-medium"
								>{preset.name.trim() || 'Untitled instruction'}</span
							>
							{#if preset.text.trim()}
								<span class="instruction-selector-preview mt-0.5 text-xs text-[var(--text-muted)]"
									>{preset.text}</span
								>
							{/if}
						</span>
						{#if preset.id === sessionState.selectedInstructionId}
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
				{/each}
			{/if}
		</div>
	{/if}
</div>
