<script lang="ts">
	import { sessionState } from '$lib/stores/session.svelte';

	const activeClass = 'bg-accent text-white ring-accent hover:bg-accent-dark';
	const idleClass =
		'bg-transparent text-[var(--text-secondary)] ring-[var(--surface-ring)] hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]';
</script>

<button
	type="button"
	role="switch"
	aria-checked={sessionState.thinkingEnabled}
	disabled={!sessionState.modelSupportsThinking}
	aria-label={sessionState.modelSupportsThinking
		? sessionState.thinkingEnabled
			? 'Disable Ollama thinking'
			: 'Enable Ollama thinking'
		: 'Model thinking unavailable'}
	title={sessionState.modelSupportsThinking
		? sessionState.thinkingEnabled
			? 'Disable thinking so the model outputs directly'
			: 'Enable thinking and show it separately from the answer'
		: 'This model does not advertise Ollama thinking support'}
	class="toolbar-tight-button flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 ring-1 transition-colors duration-150 disabled:cursor-not-allowed disabled:opacity-40 {sessionState.thinkingEnabled
		? activeClass
		: idleClass}"
	onclick={() => (sessionState.thinkingEnabled = !sessionState.thinkingEnabled)}
>
	<svg
		viewBox="0 0 24 24"
		class="size-4"
		fill="none"
		stroke="currentColor"
		stroke-width="1.8"
		stroke-linecap="round"
		stroke-linejoin="round"
	>
		<path d="M12 2 L14.2 9.8 L22 12 L14.2 14.2 L12 22 L9.8 14.2 L2 12 L9.8 9.8 Z" />
	</svg>
	<span class="toolbar-collapsible-label">Thinking</span>
</button>
