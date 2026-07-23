import { detectOllama, listOllamaModels, supportsThinking, type OllamaModel } from '$lib/tauri/ollama';
import { getSettings, setSettings, hasWebSearchApiKey, type ModelNote } from '$lib/tauri/settings';

export type OllamaStatus = 'checking' | 'connected' | 'disconnected';
export type { ModelNote };

function createSessionState() {
	let status = $state<OllamaStatus>('checking');
	let models = $state<OllamaModel[]>([]);
	let selectedModel = $state<string | null>(null);
	let modelSupportsThinking = $state(false);
	let thinkingEnabled = $state(false);
	let hasApiKey = $state(false);
	let webSearchEnabled = $state(false);
	/** Keyed by exact Ollama model name — local-only alias/description, never sent to Ollama. */
	let modelNotes = $state<Record<string, ModelNote>>({});
	let preferencesLoaded = $state(false);

	async function refreshApiKeyStatus() {
		try {
			hasApiKey = await hasWebSearchApiKey();
		} catch {
			hasApiKey = false;
		}
		if (!hasApiKey) webSearchEnabled = false;
	}

	function persistPreferences() {
		if (!preferencesLoaded) return;
		void setSettings({
			lastModel: selectedModel,
			thinkingDefault: thinkingEnabled,
			webSearchDefault: webSearchEnabled,
			modelNotes
		});
	}

	async function loadPreferences() {
		const saved = await getSettings();
		selectedModel = saved.lastModel;
		thinkingEnabled = saved.thinkingDefault;
		webSearchEnabled = saved.webSearchDefault;
		modelNotes = saved.modelNotes ?? {};
		preferencesLoaded = true;
	}

	function getModelNote(name: string): ModelNote {
		return modelNotes[name] ?? { alias: '', description: '' };
	}

	function setModelNote(name: string, note: ModelNote) {
		if (!note.alias.trim() && !note.description.trim()) {
			const rest = { ...modelNotes };
			delete rest[name];
			modelNotes = rest;
		} else {
			modelNotes = { ...modelNotes, [name]: note };
		}
		persistPreferences();
	}

	/**
	 * Drops notes for models that no longer exist in Ollama (e.g. `ollama rm`'d) so deleting a
	 * model also cleans up its alias/description rather than leaving it orphaned in settings.json
	 * forever. Only called with a freshly, successfully fetched model list — never on a failed
	 * fetch or a disconnected Ollama, since that would otherwise wipe every note just because the
	 * service was briefly unreachable.
	 */
	function pruneOrphanedNotes(currentModels: OllamaModel[]) {
		const validNames = new Set(currentModels.map((model) => model.name));
		const hasOrphan = Object.keys(modelNotes).some((name) => !validNames.has(name));
		if (!hasOrphan) return;
		const pruned: Record<string, ModelNote> = {};
		for (const [name, note] of Object.entries(modelNotes)) {
			if (validNames.has(name)) pruned[name] = note;
		}
		modelNotes = pruned;
		persistPreferences();
	}

	async function refreshThinkingSupport() {
		if (!selectedModel) {
			modelSupportsThinking = false;
			thinkingEnabled = false;
			return;
		}
		try {
			modelSupportsThinking = await supportsThinking(selectedModel);
		} catch {
			modelSupportsThinking = false;
		}
		if (!modelSupportsThinking) thinkingEnabled = false;
	}

	async function refresh() {
		const connected = await detectOllama();
		status = connected ? 'connected' : 'disconnected';

		if (!connected) {
			models = [];
			return;
		}

		try {
			models = await listOllamaModels();
			pruneOrphanedNotes(models);
		} catch {
			models = [];
		}

		if (!selectedModel || !models.some((model) => model.name === selectedModel)) {
			selectedModel = models[0]?.name ?? null;
			persistPreferences();
		}

		await refreshThinkingSupport();
		await refreshApiKeyStatus();
	}

	return {
		get status() {
			return status;
		},
		get models() {
			return models;
		},
		get selectedModel() {
			return selectedModel;
		},
		set selectedModel(name: string | null) {
			selectedModel = name;
			void refreshThinkingSupport();
			persistPreferences();
		},
		get modelSupportsThinking() {
			return modelSupportsThinking;
		},
		get thinkingEnabled() {
			return thinkingEnabled;
		},
		set thinkingEnabled(value: boolean) {
			thinkingEnabled = modelSupportsThinking ? value : false;
			persistPreferences();
		},
		get hasApiKey() {
			return hasApiKey;
		},
		get webSearchEnabled() {
			return webSearchEnabled;
		},
		set webSearchEnabled(value: boolean) {
			webSearchEnabled = hasApiKey ? value : false;
			persistPreferences();
		},
		get modelNotes() {
			return modelNotes;
		},
		getModelNote,
		setModelNote,
		loadPreferences,
		refresh,
		refreshApiKeyStatus
	};
}

export const sessionState = createSessionState();
