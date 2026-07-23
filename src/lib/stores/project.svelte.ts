import { SvelteSet } from 'svelte/reactivity';
import type { ProjectNode } from '$lib/tauri/project';

function createProjectState() {
	let rootPath = $state<string | null>(null);
	let tree = $state<ProjectNode | null>(null);
	const expanded = new SvelteSet<string>();
	let loading = $state(false);
	let error = $state<string | null>(null);

	const isOpen = $derived(rootPath !== null);

	function open(root: string, rootTree: ProjectNode) {
		rootPath = root;
		tree = rootTree;
		expanded.clear();
		expanded.add(root);
		error = null;
	}

	function close() {
		rootPath = null;
		tree = null;
		expanded.clear();
		error = null;
	}

	function toggleExpanded(path: string) {
		if (expanded.has(path)) {
			expanded.delete(path);
		} else {
			expanded.add(path);
		}
	}

	function isExpanded(path: string): boolean {
		return expanded.has(path);
	}

	return {
		get rootPath() {
			return rootPath;
		},
		get tree() {
			return tree;
		},
		get isOpen() {
			return isOpen;
		},
		get loading() {
			return loading;
		},
		set loading(value: boolean) {
			loading = value;
		},
		get error() {
			return error;
		},
		set error(value: string | null) {
			error = value;
		},
		open,
		close,
		toggleExpanded,
		isExpanded
	};
}

export const projectState = createProjectState();
