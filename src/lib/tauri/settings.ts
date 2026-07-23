import { invoke } from '@tauri-apps/api/core';

export interface ModelNote {
	alias: string;
	description: string;
}

export interface Settings {
	lastModel: string | null;
	thinkingDefault: boolean;
	webSearchDefault: boolean;
	/** Keyed by exact Ollama model name — purely local organization, see `ModelNote`. */
	modelNotes: Record<string, ModelNote>;
	/** Keyed by exact Ollama model name — absent entry means "use Ollama's own default". */
	numCtxOverrides: Record<string, number>;
}

/**
 * Keychain calls go through the OS Security framework, which can block waiting on a system
 * permission prompt (common with unsigned/ad-hoc-signed dev builds — the prompt can appear
 * behind the window, or re-trigger on every rebuild since the ad-hoc signature changes each
 * time). Without a timeout, a stuck prompt leaves the UI silently waiting forever.
 */
function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
	return new Promise<T>((resolve, reject) => {
		const timeoutHandle = setTimeout(() => {
			reject(
				new Error(
					'Keychain access timed out. macOS may be waiting on a permission prompt — check for a hidden "wants to access your keychain" dialog (it can appear behind the window) and click Allow, then try again.'
				)
			);
		}, ms);
		promise.then(
			(value) => {
				clearTimeout(timeoutHandle);
				resolve(value);
			},
			(error) => {
				clearTimeout(timeoutHandle);
				reject(error);
			}
		);
	});
}

const KEYCHAIN_TIMEOUT_MS = 8000;

export async function getSettings(): Promise<Settings> {
	return invoke<Settings>('get_settings');
}

export async function setSettings(settings: Settings): Promise<void> {
	await invoke('set_settings', { settings });
}

export async function saveWebSearchApiKey(key: string): Promise<void> {
	await withTimeout(invoke('save_web_search_api_key', { key }), KEYCHAIN_TIMEOUT_MS);
}

export async function hasWebSearchApiKey(): Promise<boolean> {
	return withTimeout(invoke<boolean>('has_web_search_api_key'), KEYCHAIN_TIMEOUT_MS);
}
