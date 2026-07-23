export type FileSwitchChoice = 'save' | 'discard' | 'cancel';

function createUiState() {
	let settingsOpen = $state(false);
	let sidebarOpen = $state(true);
	let resolvePending = $state<((choice: FileSwitchChoice) => void) | null>(null);
	let modelInfoTarget = $state<string | null>(null);

	function requestFileSwitchConfirm(): Promise<FileSwitchChoice> {
		// A second confirm request arriving while one is already pending (e.g. clicking two
		// different sidebar files in quick succession) would otherwise overwrite `resolvePending`
		// and permanently strand the first caller's promise. Resolve the stale one as 'cancel'
		// first so nothing is left dangling forever — the safest default for a request that's
		// about to be superseded anyway.
		resolvePending?.('cancel');
		return new Promise((resolve) => {
			resolvePending = (choice) => {
				resolvePending = null;
				resolve(choice);
			};
		});
	}

	function resolveFileSwitchConfirm(choice: FileSwitchChoice) {
		resolvePending?.(choice);
	}

	return {
		get settingsOpen() {
			return settingsOpen;
		},
		set settingsOpen(value: boolean) {
			settingsOpen = value;
		},
		get sidebarOpen() {
			return sidebarOpen;
		},
		set sidebarOpen(value: boolean) {
			sidebarOpen = value;
		},
		get fileSwitchConfirmPending() {
			return resolvePending !== null;
		},
		requestFileSwitchConfirm,
		resolveFileSwitchConfirm,
		get modelInfoTarget() {
			return modelInfoTarget;
		},
		openModelInfo(modelName: string) {
			modelInfoTarget = modelName;
		},
		closeModelInfo() {
			modelInfoTarget = null;
		}
	};
}

export const uiState = createUiState();
