import { invoke } from '@tauri-apps/api/core';

export type ProjectNode =
	| { kind: 'file'; name: string; path: string }
	| { kind: 'dir'; name: string; path: string; children: ProjectNode[] };

export async function scanProject(root: string): Promise<ProjectNode> {
	return invoke<ProjectNode>('scan_project', { root });
}
