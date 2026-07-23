<script lang="ts">
	import { documentState } from '$lib/stores/document.svelte';
	import { conversationState } from '$lib/stores/conversation.svelte';
	import { saveDocument } from '$lib/actions/fileActions';
	import DiffView from './DiffView.svelte';

	const activeTurn = $derived(conversationState.activeTurn);

	let applying = $state(false);
	let applyError = $state<string | null>(null);

	// Mirrors ChatTurn's own generating-status label so the Changes panel doesn't look dead while
	// a generation is actually in progress right next to it.
	const generatingLabel = $derived.by(() => {
		if (activeTurn?.status !== 'generating') return null;
		if (activeTurn.answerLength > 0) return 'Writing…';
		if (activeTurn.thinkingText.length > 0) return 'Thinking…';
		return 'Generating…';
	});

	/**
	 * Applying an AI suggestion is conceptually "accept and save this" — saving to disk
	 * automatically afterwards means the only way content actually lands in the file is still
	 * through this explicit confirmation (never a hidden background write), it just no longer
	 * requires a separate manual Save click for the common case. If the write fails (no
	 * writable path resolved, disk error, etc.), the change is still applied in memory — just
	 * left dirty, same as any other edit that hasn't been saved yet — and the failure is
	 * surfaced rather than silently swallowed.
	 */
	async function handleApply() {
		conversationState.applyActive((text) => {
			documentState.content = text;
		});
		applyError = null;
		applying = true;
		try {
			await saveDocument();
		} catch (err) {
			applyError = err instanceof Error ? err.message : String(err);
		} finally {
			applying = false;
		}
	}

	$effect(() => {
		// Reset transient per-turn state whenever the active turn changes out from under it (a
		// new turn started, etc.) so a previous apply's save error never lingers on screen for
		// the wrong turn.
		void activeTurn?.id;
		applyError = null;
	});
</script>

<aside class="app-surface flex min-h-0 flex-[2] flex-col overflow-hidden rounded-2xl">
	<div class="app-panel-header px-3.5 py-2.5 text-xs font-medium">Changes</div>

	<div class="flex-1 overflow-auto p-3 text-sm">
		{#if applying}
			<p class="text-sm text-[var(--text-muted)]">Saving…</p>
		{:else if applyError}
			<p class="text-sm text-red-600 dark:text-red-400">
				Applied, but saving to disk failed: {applyError} The change is still in the editor — use Save
				to try again.
			</p>
		{:else if activeTurn?.status === 'reviewing' && activeTurn.diff}
			<DiffView diff={activeTurn.diff} />
		{:else if generatingLabel}
			<div class="flex items-center gap-1.5 text-sm text-[var(--text-muted)]">
				<span class="size-1.5 animate-pulse rounded-full bg-accent"></span>
				{generatingLabel}
			</div>
		{:else}
			<p class="text-sm text-[var(--text-muted)]">No changes to review yet.</p>
		{/if}
	</div>

	{#if activeTurn?.status === 'reviewing'}
		<div class="app-panel-footer flex items-center justify-end gap-2 px-3.5 py-2.5">
			<button
				class="rounded-lg px-2.5 py-1.5 text-sm text-[var(--text-secondary)] transition-colors duration-150 hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]"
				onclick={() => conversationState.discardActive()}
			>
				Discard
			</button>
			<button
				class="rounded-lg bg-accent px-3 py-1.5 text-sm font-medium text-white transition-colors duration-150 hover:bg-accent-dark"
				onclick={handleApply}
			>
				Apply
			</button>
		</div>
	{/if}
</aside>
