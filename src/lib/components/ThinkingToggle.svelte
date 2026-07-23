<script lang="ts">
	import { sessionState } from '$lib/stores/session.svelte';

	const activeClass = 'bg-accent text-white ring-accent hover:bg-accent-dark';
	const idleClass =
		'bg-transparent text-neutral-600 ring-neutral-200 hover:bg-neutral-900/5 dark:text-neutral-400 dark:ring-white/10 dark:hover:bg-white/[0.06]';
</script>

<button
	type="button"
	role="switch"
	aria-checked={sessionState.thinkingEnabled}
	disabled={!sessionState.modelSupportsThinking}
	title={sessionState.modelSupportsThinking
		? 'Show the model’s thinking trace'
		: 'This model does not support thinking'}
	class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 ring-1 transition-colors duration-150 disabled:cursor-not-allowed disabled:opacity-40 {sessionState.thinkingEnabled
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
	Thinking
</button>
