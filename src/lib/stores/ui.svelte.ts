function createUiState() {
	let settingsOpen = $state(false);

	return {
		get settingsOpen() {
			return settingsOpen;
		},
		set settingsOpen(value: boolean) {
			settingsOpen = value;
		}
	};
}

export const uiState = createUiState();
