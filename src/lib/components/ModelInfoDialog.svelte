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
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/25 p-4 backdrop-blur-sm">
		<div
			class="app-surface w-full max-w-sm overflow-hidden rounded-xl"
			role="dialog"
			aria-modal="true"
			aria-labelledby="model-info-title"
		>
			<div class="app-panel-header flex items-center justify-between px-4 py-3">
				<h2
					id="model-info-title"
					class="truncate font-mono text-sm font-semibold text-[var(--text-primary)]"
				>
					{uiState.modelInfoTarget}
				</h2>
				<button
					type="button"
					aria-label="Close"
					onclick={() => uiState.closeModelInfo()}
					class="shrink-0 rounded-md p-1 text-[var(--text-muted)] transition-colors duration-150 hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]"
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
					<p class="text-xs text-[var(--text-muted)]">Loading…</p>
				{:else if error}
					<p class="text-xs text-red-600 dark:text-red-400">{error}</p>
				{:else if info}
					{#if info.capabilities.length > 0}
						<div class="mb-3 flex flex-wrap gap-1.5">
							{#each info.capabilities as capability (capability)}
								<span
									class="rounded-full bg-[var(--surface-inset)] px-2 py-0.5 text-[11px] font-medium text-[var(--text-secondary)]"
								>
									{capability}
								</span>
							{/each}
						</div>
					{/if}

					<dl class="mb-3 grid grid-cols-2 gap-2 text-xs">
						<div>
							<dt class="text-[var(--text-muted)]">Architecture</dt>
							<dd class="font-medium text-[var(--text-primary)]">
								{info.architecture || '—'}
							</dd>
						</div>
						<div>
							<dt class="text-[var(--text-muted)]">Parameters</dt>
							<dd class="font-medium text-[var(--text-primary)]">
								{info.parameterSize || '—'}
							</dd>
						</div>
						<div>
							<dt class="text-[var(--text-muted)]">Quantization</dt>
							<dd class="font-medium text-[var(--text-primary)]">
								{info.quantization || '—'}
							</dd>
						</div>
						<div>
							<dt class="text-[var(--text-muted)]">Context length</dt>
							<dd class="font-medium text-[var(--text-primary)]">
								{info.contextLength?.toLocaleString() ?? '—'}
							</dd>
						</div>
					</dl>

					{#if info.parameters.length > 0}
						<p class="mb-1 text-xs font-medium text-[var(--text-secondary)]">Default parameters</p>
						<div class="space-y-0.5">
							{#each info.parameters as param (param.key)}
								<div class="flex justify-between gap-2 text-xs">
									<span class="font-mono text-[var(--text-secondary)]">{param.key}</span>
									<span class="truncate text-[var(--text-primary)]">{param.value}</span>
								</div>
							{/each}
						</div>
					{/if}
				{/if}
			</div>
		</div>
	</div>
{/if}
