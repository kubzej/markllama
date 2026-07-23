import {
	detectOllama,
	listOllamaModels,
	supportsThinking,
	supportsVision,
	type OllamaModel
} from '$lib/tauri/ollama';
import {
	getSettings,
	setSettings,
	hasWebSearchApiKey,
	type ModelNote,
	type InstructionPreset
} from '$lib/tauri/settings';

export type OllamaStatus = 'checking' | 'connected' | 'disconnected';
export type { ModelNote, InstructionPreset };

function createSessionState() {
	let status = $state<OllamaStatus>('checking');
	let models = $state<OllamaModel[]>([]);
	let selectedModel = $state<string | null>(null);
	let modelSupportsThinking = $state(false);
	let modelSupportsVision = $state(false);
	let thinkingEnabled = $state(false);
	let hasApiKey = $state(false);
	let webSearchEnabled = $state(false);
	/** Keyed by exact Ollama model name — local-only alias/description, never sent to Ollama. */
	let modelNotes = $state<Record<string, ModelNote>>({});
	/** Keyed by exact Ollama model name — absent entry means "use Ollama's own default". */
	let numCtxOverrides = $state<Record<string, number>>({});
	let selectedInstructionId = $state<string | null>(null);
	let instructionPresets = $state<InstructionPreset[]>([]);
	let preferencesLoaded = $state(false);

	async function refreshApiKeyStatus() {
		try {
			hasApiKey = await hasWebSearchApiKey();
		} catch {
			hasApiKey = false;
		}
		if (!hasApiKey) webSearchEnabled = false;
	}

	/**
	 * Returns the underlying save so callers that show a "Saved" confirmation (e.g.
	 * `ModelNoteEditor`) can await it and only claim success once the write actually completed.
	 * Callers that don't need to react to failure can keep treating it as fire-and-forget by not
	 * awaiting it.
	 */
	function persistPreferences(): Promise<void> {
		if (!preferencesLoaded) return Promise.resolve();
		return setSettings({
			lastModel: selectedModel,
			thinkingDefault: thinkingEnabled,
			webSearchDefault: webSearchEnabled,
			selectedInstructionId,
			modelNotes,
			numCtxOverrides,
			instructionPresets
		});
	}

	/** For call sites that don't need to react to save failures — still logs instead of leaving
	 *  an unhandled promise rejection. */
	function persistPreferencesInBackground() {
		persistPreferences().catch((err) => {
			console.error('Failed to save preferences', err);
		});
	}

	async function loadPreferences() {
		const saved = await getSettings();
		selectedModel = saved.lastModel;
		thinkingEnabled = saved.thinkingDefault;
		webSearchEnabled = saved.webSearchDefault;
		modelNotes = saved.modelNotes ?? {};
		numCtxOverrides = saved.numCtxOverrides ?? {};
		instructionPresets = saved.instructionPresets ?? [];
		selectedInstructionId = instructionPresets.some(
			(preset) => preset.id === saved.selectedInstructionId
		)
			? saved.selectedInstructionId
			: null;
		preferencesLoaded = true;
	}

	function getModelNote(name: string): ModelNote {
		return modelNotes[name] ?? { alias: '', description: '' };
	}

	function setModelNote(name: string, note: ModelNote): Promise<void> {
		if (!note.alias.trim() && !note.description.trim()) {
			const rest = { ...modelNotes };
			delete rest[name];
			modelNotes = rest;
		} else {
			modelNotes = { ...modelNotes, [name]: note };
		}
		return persistPreferences();
	}

	function getNumCtxOverride(name: string): number | null {
		return numCtxOverrides[name] ?? null;
	}

	function setNumCtxOverride(name: string, value: number | null): Promise<void> {
		if (value == null) {
			const rest = { ...numCtxOverrides };
			delete rest[name];
			numCtxOverrides = rest;
		} else {
			numCtxOverrides = { ...numCtxOverrides, [name]: value };
		}
		return persistPreferences();
	}

	function createInstructionPreset(): Promise<void> {
		const id = crypto.randomUUID();
		instructionPresets = [
			...instructionPresets,
			{
				id,
				name: 'New instruction',
				text: ''
			}
		];
		selectedInstructionId = id;
		return persistPreferences();
	}

	function updateInstructionPreset(
		id: string,
		patch: Partial<Omit<InstructionPreset, 'id'>>
	): Promise<void> {
		instructionPresets = instructionPresets.map((preset) =>
			preset.id === id ? { ...preset, ...patch } : preset
		);
		return persistPreferences();
	}

	function deleteInstructionPreset(id: string): Promise<void> {
		instructionPresets = instructionPresets.filter((preset) => preset.id !== id);
		if (selectedInstructionId === id) selectedInstructionId = null;
		return persistPreferences();
	}

	/**
	 * Drops notes/overrides for models that no longer exist in Ollama (e.g. `ollama rm`'d) so
	 * deleting a model also cleans up its alias/description/num_ctx rather than leaving them
	 * orphaned in settings.json forever. Only called with a freshly, successfully fetched model
	 * list — never on a failed fetch or a disconnected Ollama, since that would otherwise wipe
	 * every note/override just because the service was briefly unreachable.
	 */
	function pruneOrphanedNotes(currentModels: OllamaModel[]) {
		const validNames = new Set(currentModels.map((model) => model.name));

		const hasOrphanNote = Object.keys(modelNotes).some((name) => !validNames.has(name));
		if (hasOrphanNote) {
			const pruned: Record<string, ModelNote> = {};
			for (const [name, note] of Object.entries(modelNotes)) {
				if (validNames.has(name)) pruned[name] = note;
			}
			modelNotes = pruned;
		}

		const hasOrphanOverride = Object.keys(numCtxOverrides).some((name) => !validNames.has(name));
		if (hasOrphanOverride) {
			const pruned: Record<string, number> = {};
			for (const [name, value] of Object.entries(numCtxOverrides)) {
				if (validNames.has(name)) pruned[name] = value;
			}
			numCtxOverrides = pruned;
		}

		if (hasOrphanNote || hasOrphanOverride) persistPreferencesInBackground();
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

	async function refreshVisionSupport() {
		if (!selectedModel) {
			modelSupportsVision = false;
			return;
		}
		try {
			modelSupportsVision = await supportsVision(selectedModel);
		} catch {
			modelSupportsVision = false;
		}
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
			persistPreferencesInBackground();
		}

		await refreshThinkingSupport();
		await refreshVisionSupport();
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
			void refreshVisionSupport();
			persistPreferencesInBackground();
		},
		get modelSupportsThinking() {
			return modelSupportsThinking;
		},
		get modelSupportsVision() {
			return modelSupportsVision;
		},
		get thinkingEnabled() {
			return thinkingEnabled;
		},
		set thinkingEnabled(value: boolean) {
			thinkingEnabled = modelSupportsThinking ? value : false;
			persistPreferencesInBackground();
		},
		get hasApiKey() {
			return hasApiKey;
		},
		get webSearchEnabled() {
			return webSearchEnabled;
		},
		set webSearchEnabled(value: boolean) {
			webSearchEnabled = hasApiKey ? value : false;
			persistPreferencesInBackground();
		},
		get modelNotes() {
			return modelNotes;
		},
		get instructionPresets() {
			return instructionPresets;
		},
		get selectedInstructionId() {
			return selectedInstructionId;
		},
		set selectedInstructionId(id: string | null) {
			selectedInstructionId = instructionPresets.some((preset) => preset.id === id) ? id : null;
			persistPreferencesInBackground();
		},
		get selectedInstructionPreset() {
			return instructionPresets.find((preset) => preset.id === selectedInstructionId) ?? null;
		},
		getModelNote,
		setModelNote,
		getNumCtxOverride,
		setNumCtxOverride,
		createInstructionPreset,
		updateInstructionPreset,
		deleteInstructionPreset,
		loadPreferences,
		refresh,
		refreshApiKeyStatus
	};
}

export const sessionState = createSessionState();
