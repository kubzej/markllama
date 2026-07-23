import type { AttachedFile, ConversationTurn } from './conversation.svelte';

/**
 * Kept in a plain `.ts` module rather than alongside the rest of `conversation.svelte.ts` — the
 * `Map`s below are pure local computation, never Svelte state, but the linter's
 * `svelte/prefer-svelte-reactivity` rule flags any built-in `Map` inside a `.svelte.ts` file
 * regardless of whether it's reactive, so this logic lives where that rule doesn't apply.
 */

export function attachedFilesBlock(files: AttachedFile[]): string {
	let block = '';
	for (const file of files) block += `Attached: ${file.path}\n\n${file.content}\n\n---\n\n`;
	return block;
}

/** For each turn, only the files whose content actually differs from the most recent prior turn
 *  that carried that same path — i.e. the ones worth transmitting at all. Skips error/cancelled
 *  turns, same as `buildHistory()`, since they never really joined the conversation. */
export function filesToSendPerTurn(turnsInOrder: ConversationTurn[]): Map<string, AttachedFile[]> {
	const lastSent = new Map<string, string>();
	const plan = new Map<string, AttachedFile[]>();
	for (const turn of turnsInOrder) {
		if (turn.status === 'error' || turn.status === 'cancelled') continue;
		const changed: AttachedFile[] = [];
		for (const file of turn.attachedFiles) {
			if (lastSent.get(file.path) !== file.content) changed.push(file);
			lastSent.set(file.path, file.content);
		}
		plan.set(turn.id, changed);
	}
	return plan;
}

/** Given the files in scope for a brand-new turn (not yet in `turns`), which of them actually
 *  need to be sent — i.e. which ones aren't already sitting, unchanged, in an earlier turn's
 *  still-uncompacted history message. */
export function filesToSendForNewTurn(
	existingTurns: ConversationTurn[],
	candidates: AttachedFile[]
): AttachedFile[] {
	const lastSent = new Map<string, string>();
	for (const turn of existingTurns) {
		if (turn.status === 'error' || turn.status === 'cancelled') continue;
		for (const file of turn.attachedFiles) lastSent.set(file.path, file.content);
	}
	return candidates.filter((file) => lastSent.get(file.path) !== file.content);
}
