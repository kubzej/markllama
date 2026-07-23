function createDocumentState() {
	let path = $state<string | null>(null);
	let content = $state('');
	let savedContent = $state('');
	const dirty = $derived(content !== savedContent);

	return {
		get path() {
			return path;
		},
		get content() {
			return content;
		},
		set content(value: string) {
			content = value;
		},
		get dirty() {
			return dirty;
		},
		get filename() {
			return path ? path.split('/').pop() ?? path : 'Untitled';
		},
		load(newPath: string, text: string) {
			path = newPath;
			content = text;
			savedContent = text;
		},
		markSaved(newPath?: string) {
			if (newPath) path = newPath;
			savedContent = content;
		},
		reset() {
			path = null;
			content = '';
			savedContent = '';
		}
	};
}

export const documentState = createDocumentState();
