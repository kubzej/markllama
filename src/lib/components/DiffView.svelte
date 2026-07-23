<script lang="ts">
	import type { DiffLine } from '$lib/tauri/diff';

	let { diff }: { diff: DiffLine[] } = $props();
</script>

<div class="space-y-0.5 font-mono text-xs leading-relaxed">
	{#each diff as line, i (i)}
		{#if line.kind === 'unchanged'}
			<div class="px-1.5 py-0.5 text-neutral-500 dark:text-neutral-400">
				{line.text || ' '}
			</div>
		{:else if line.kind === 'removed'}
			<div
				class="rounded-md bg-red-50 px-1.5 py-0.5 text-red-700 line-through dark:bg-red-950/40 dark:text-red-300"
			>
				{line.text || ' '}
			</div>
		{:else if line.kind === 'added'}
			<div
				class="rounded-md bg-emerald-50 px-1.5 py-0.5 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300"
			>
				{line.text || ' '}
			</div>
		{:else if line.kind === 'changed'}
			<div
				class="rounded-md bg-red-50 px-1.5 py-0.5 text-red-700 line-through dark:bg-red-950/40 dark:text-red-300"
			>
				{#each line.old as span, j (j)}<span class:font-semibold={span.changed}
						>{span.text}</span
					>{/each}
			</div>
			<div
				class="rounded-md bg-emerald-50 px-1.5 py-0.5 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300"
			>
				{#each line.new as span, j (j)}<span class:font-semibold={span.changed}
						>{span.text}</span
					>{/each}
			</div>
		{/if}
	{/each}
</div>
