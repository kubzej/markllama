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
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/25 p-4 backdrop-blur-sm">
		<div
			class="app-surface flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl"
			role="dialog"
			aria-modal="true"
			aria-labelledby="settings-modal-title"
		>
			<div class="app-panel-header flex items-center justify-between px-5 py-4">
				<h2 id="settings-modal-title" class="text-sm font-semibold text-[var(--text-primary)]">
					Settings
				</h2>
				<button
					type="button"
					aria-label="Close settings"
					onclick={() => (open = false)}
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

			<div class="flex-1 space-y-4 overflow-y-auto px-5 py-4">
				<section class="rounded-lg bg-[var(--surface-muted)] p-4">
					<label
						for="ollama-api-key"
						class="mb-1.5 block text-xs font-medium text-[var(--text-secondary)]"
					>
						Ollama Web Search API key
					</label>
					<input
						id="ollama-api-key"
						type="password"
						class="soft-input mb-1.5 w-full rounded-md px-2.5 py-1.5 text-sm outline-none"
						placeholder={hasKey
							? 'Configured — enter a new key to replace it'
							: 'Paste your API key'}
						bind:value={apiKeyInput}
					/>
					<p class="text-xs text-[var(--text-muted)]">
						Stored in the macOS Keychain. Inference stays local — this key is only used for the
						optional Web Search toggle.
						<span class="font-medium"
							>{hasKey ? 'A key is currently configured.' : 'No key configured.'}</span
						>
					</p>

					{#if saveState === 'error'}
						<p class="mt-2 text-xs text-red-600 dark:text-red-400">{errorMessage}</p>
					{/if}

					<div class="mt-3 flex items-center justify-between">
						<button
							class="rounded-lg px-2.5 py-1.5 text-sm text-[var(--text-secondary)] transition-colors duration-150 hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)] disabled:opacity-40"
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

				<section class="rounded-lg bg-[var(--surface-muted)] p-4">
					<h3 class="mb-0.5 text-xs font-medium text-[var(--text-secondary)]">Model notes</h3>
					<p class="mb-3 text-xs text-[var(--text-muted)]">
						Give each model an alias, a short description, and an optional context window override.
						The context window controls how much of the conversation the model can see at once;
						higher uses more memory and is slower. Leave it empty to use Ollama's default. Saved
						automatically as you type.
					</p>
					{#if sessionState.models.length === 0}
						<p class="text-xs text-[var(--text-muted)]">
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

			<div class="app-panel-footer flex justify-end px-5 py-3">
				<button
					class="rounded-lg px-3 py-1.5 text-sm text-[var(--text-secondary)] transition-colors duration-150 hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]"
					onclick={() => (open = false)}
				>
					Close
				</button>
			</div>
		</div>
	</div>
{/if}
