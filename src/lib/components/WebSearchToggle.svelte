<script lang="ts">
	import { sessionState } from '$lib/stores/session.svelte';

	const activeClass = 'bg-accent text-white ring-accent hover:bg-accent-dark';
	const idleClass =
		'bg-transparent text-[var(--text-secondary)] ring-[var(--surface-ring)] hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]';
</script>

<button
	type="button"
	role="switch"
	aria-checked={sessionState.webSearchEnabled}
	disabled={!sessionState.hasApiKey}
	title={sessionState.hasApiKey
		? 'Let Ollama search the web for reference'
		: 'Add an Ollama Web Search API key in Settings to enable this'}
	class="toolbar-tight-button flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 ring-1 transition-colors duration-150 disabled:cursor-not-allowed disabled:opacity-40 {sessionState.webSearchEnabled
		? activeClass
		: idleClass}"
	onclick={() => (sessionState.webSearchEnabled = !sessionState.webSearchEnabled)}
>
	<svg
		viewBox="0 0 24 24"
		class="size-4"
		fill="none"
		stroke="currentColor"
		stroke-width="1.8"
		stroke-linecap="round"
	>
		<circle cx="12" cy="12" r="9" />
		<ellipse cx="12" cy="12" rx="4" ry="9" />
		<line x1="3" y1="12" x2="21" y2="12" />
	</svg>
	<span class="toolbar-collapsible-label">Web Search</span>
</button>
