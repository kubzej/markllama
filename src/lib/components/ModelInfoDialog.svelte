<script lang="ts">
	import { uiState } from '$lib/stores/ui.svelte';
	import { getModelInfo, type ModelInfo } from '$lib/tauri/ollama';

	let info = $state<ModelInfo | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);

	$effect(() => {
		const model = uiState.modelInfoTarget;
		if (!model) {
			info = null;
			error = null;
			return;
		}
		loading = true;
		error = null;
		getModelInfo(model)
			.then((result) => {
				info = result;
			})
			.catch((err) => {
				error = err instanceof Error ? err.message : String(err);
			})
			.finally(() => {
				loading = false;
			});
	});

	function handleWindowKeydown(event: KeyboardEvent) {
		if (uiState.modelInfoTarget && event.key === 'Escape') uiState.closeModelInfo();
	}
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#if uiState.modelInfoTarget}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/20 p-4">
		<div
			class="w-full max-w-sm rounded-xl border border-neutral-200 bg-white shadow-xl dark:border-neutral-800 dark:bg-neutral-900"
			role="dialog"
			aria-modal="true"
			aria-labelledby="model-info-title"
		>
			<div
				class="flex items-center justify-between border-b border-neutral-200 px-4 py-3 dark:border-neutral-800"
			>
				<h2
					id="model-info-title"
					class="truncate font-mono text-sm font-semibold text-neutral-800 dark:text-neutral-200"
				>
					{uiState.modelInfoTarget}
				</h2>
				<button
					type="button"
					aria-label="Close"
					onclick={() => uiState.closeModelInfo()}
					class="shrink-0 rounded-md p-1 text-neutral-400 transition-colors duration-150 hover:bg-neutral-100 hover:text-neutral-600 dark:hover:bg-neutral-800 dark:hover:text-neutral-300"
				>
					<svg
						viewBox="0 0 24 24"
						class="size-4"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
					>
						<path d="M18 6 6 18M6 6l12 12" />
					</svg>
				</button>
			</div>

			<div class="max-h-96 overflow-y-auto px-4 py-3">
				{#if loading}
					<p class="text-xs text-neutral-400 dark:text-neutral-500">Loading…</p>
				{:else if error}
					<p class="text-xs text-red-600 dark:text-red-400">{error}</p>
				{:else if info}
					{#if info.capabilities.length > 0}
						<div class="mb-3 flex flex-wrap gap-1.5">
							{#each info.capabilities as capability (capability)}
								<span
									class="rounded-full bg-neutral-100 px-2 py-0.5 text-[11px] font-medium text-neutral-600 dark:bg-white/5 dark:text-neutral-300"
								>
									{capability}
								</span>
							{/each}
						</div>
					{/if}

					<dl class="mb-3 grid grid-cols-2 gap-2 text-xs">
						<div>
							<dt class="text-neutral-400 dark:text-neutral-500">Architecture</dt>
							<dd class="font-medium text-neutral-700 dark:text-neutral-200">
								{info.architecture || '—'}
							</dd>
						</div>
						<div>
							<dt class="text-neutral-400 dark:text-neutral-500">Parameters</dt>
							<dd class="font-medium text-neutral-700 dark:text-neutral-200">
								{info.parameterSize || '—'}
							</dd>
						</div>
						<div>
							<dt class="text-neutral-400 dark:text-neutral-500">Quantization</dt>
							<dd class="font-medium text-neutral-700 dark:text-neutral-200">
								{info.quantization || '—'}
							</dd>
						</div>
						<div>
							<dt class="text-neutral-400 dark:text-neutral-500">Context length</dt>
							<dd class="font-medium text-neutral-700 dark:text-neutral-200">
								{info.contextLength?.toLocaleString() ?? '—'}
							</dd>
						</div>
					</dl>

					{#if info.parameters.length > 0}
						<p class="mb-1 text-xs font-medium text-neutral-500 dark:text-neutral-400">
							Default parameters
						</p>
						<div class="space-y-0.5">
							{#each info.parameters as param (param.key)}
								<div class="flex justify-between gap-2 text-xs">
									<span class="font-mono text-neutral-500 dark:text-neutral-400">{param.key}</span>
									<span class="truncate text-neutral-700 dark:text-neutral-200">{param.value}</span>
								</div>
							{/each}
						</div>
					{/if}
				{/if}
			</div>
		</div>
	</div>
{/if}
