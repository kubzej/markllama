<script lang="ts">
	import { sessionState } from '$lib/stores/session.svelte';

	const label = {
		checking: 'Checking…',
		connected: 'Ollama',
		disconnected: 'Ollama not running'
	} as const;

	const dotClass = {
		checking: 'bg-neutral-400 dark:bg-neutral-500',
		connected: 'bg-emerald-500',
		disconnected: 'bg-red-500'
	} as const;
</script>

<span
	class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-[var(--text-secondary)]"
	title={label[sessionState.status]}
>
	<span class="relative flex size-1.5">
		{#if sessionState.status === 'connected'}
			<span
				class="absolute inline-flex size-full animate-ping rounded-full bg-emerald-500 opacity-75"
			></span>
		{/if}
		<span class={`relative inline-flex size-1.5 rounded-full ${dotClass[sessionState.status]}`}
		></span>
	</span>
	{label[sessionState.status]}
</span>
