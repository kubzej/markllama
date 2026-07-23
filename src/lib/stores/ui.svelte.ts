export type FileSwitchChoice = 'save' | 'discard' | 'cancel';

function createUiState() {
	let settingsOpen = $state(false);
	let sidebarOpen = $state(true);
	let resolvePending = $state<((choice: FileSwitchChoice) => void) | null>(null);
	let modelInfoTarget = $state<string | null>(null);

	function requestFileSwitchConfirm(): Promise<FileSwitchChoice> {
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
