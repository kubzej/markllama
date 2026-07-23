<script lang="ts">
	import { saveWebSearchApiKey } from '$lib/tauri/settings';
	import { sessionState } from '$lib/stores/session.svelte';
	import ModelNoteEditor from './ModelNoteEditor.svelte';

	let { open = $bindable(false) } = $props();

	let apiKeyInput = $state('');
	let saveState = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');
	let errorMessage = $state('');
	const hasKey = $derived(sessionState.hasApiKey);

	async function handleSave() {
		saveState = 'saving';
		errorMessage = '';
		try {
			await saveWebSearchApiKey(apiKeyInput);
			apiKeyInput = '';
			await sessionState.refreshApiKeyStatus();
			saveState = 'saved';
			setTimeout(() => (saveState = 'idle'), 1500);
		} catch (err) {
			errorMessage = err instanceof Error ? err.message : String(err);
			saveState = 'error';
		}
	}

	function handleClear() {
		apiKeyInput = '';
		void handleSave();
	}

	function handleWindowKeydown(event: KeyboardEvent) {
		if (open && event.key === 'Escape') open = false;
	}
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#if open}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/20 p-4">
		<div
			class="flex max-h-[85vh] w-full max-w-2xl flex-col rounded-xl border border-neutral-200 bg-white shadow-xl dark:border-neutral-800 dark:bg-neutral-900"
			role="dialog"
			aria-modal="true"
			aria-labelledby="settings-modal-title"
		>
			<div
				class="flex items-center justify-between border-b border-neutral-200 px-5 py-4 dark:border-neutral-800"
			>
				<h2
					id="settings-modal-title"
					class="text-sm font-semibold text-neutral-800 dark:text-neutral-200"
				>
					Settings
				</h2>
				<button
					type="button"
					aria-label="Close settings"
					onclick={() => (open = false)}
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

			<div class="flex-1 space-y-4 overflow-y-auto px-5 py-4">
				<section class="rounded-lg bg-neutral-50 p-4 dark:bg-neutral-800/40">
					<label
						for="ollama-api-key"
						class="mb-1.5 block text-xs font-medium text-neutral-500 dark:text-neutral-400"
					>
						Ollama Web Search API key
					</label>
					<input
						id="ollama-api-key"
						type="password"
						class="mb-1.5 w-full rounded-md border border-neutral-300 bg-white px-2.5 py-1.5 text-sm text-neutral-900 outline-none focus:border-accent dark:border-neutral-700 dark:bg-neutral-950 dark:text-neutral-100"
						placeholder={hasKey ? 'Configured — enter a new key to replace it' : 'Paste your API key'}
						bind:value={apiKeyInput}
					/>
					<p class="text-xs text-neutral-400 dark:text-neutral-500">
						Stored in the macOS Keychain. Inference stays local — this key is only used for the
						optional Web Search toggle.
						<span class="font-medium">{hasKey ? 'A key is currently configured.' : 'No key configured.'}</span>
					</p>

					{#if saveState === 'error'}
						<p class="mt-2 text-xs text-red-600 dark:text-red-400">{errorMessage}</p>
					{/if}

					<div class="mt-3 flex items-center justify-between">
						<button
							class="rounded-lg px-2.5 py-1.5 text-sm text-neutral-500 transition-colors duration-150 hover:bg-neutral-100 disabled:opacity-40 dark:text-neutral-400 dark:hover:bg-neutral-800"
							onclick={handleClear}
							disabled={!hasKey || saveState === 'saving'}
						>
							Delete key
						</button>
						<button
							class="rounded-lg bg-accent px-2.5 py-1.5 text-sm font-medium text-white transition-colors duration-150 hover:bg-accent-dark disabled:opacity-40"
							onclick={handleSave}
							disabled={!apiKeyInput.trim() || saveState === 'saving'}
						>
							{saveState === 'saved'
								? 'Saved'
								: saveState === 'saving'
									? 'Saving…'
									: saveState === 'error'
										? 'Retry'
										: 'Save key'}
						</button>
					</div>
				</section>

				<section class="rounded-lg bg-neutral-50 p-4 dark:bg-neutral-800/40">
					<h3 class="mb-0.5 text-xs font-medium text-neutral-500 dark:text-neutral-400">
						Model notes
					</h3>
					<p class="mb-3 text-xs text-neutral-400 dark:text-neutral-500">
						Give each model an alias and a short description of what it's good for — shown next to
						the model name when you pick one. Saved automatically as you type.
					</p>
					{#if sessionState.models.length === 0}
						<p class="text-xs text-neutral-400 dark:text-neutral-500">
							No models detected yet — they'll show up here once Ollama is connected.
						</p>
					{:else}
						<div class="max-h-80 space-y-2 overflow-y-auto pr-1">
							{#each sessionState.models as model (model.name)}
								<ModelNoteEditor modelName={model.name} />
							{/each}
						</div>
					{/if}
				</section>
			</div>

			<div class="flex justify-end border-t border-neutral-200 px-5 py-3 dark:border-neutral-800">
				<button
					class="rounded-lg px-3 py-1.5 text-sm text-neutral-600 transition-colors duration-150 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-800"
					onclick={() => (open = false)}
				>
					Close
				</button>
			</div>
		</div>
	</div>
{/if}
