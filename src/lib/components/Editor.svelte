<script lang="ts">
	import { onDestroy } from 'svelte';
	import { EditorView, keymap, placeholder } from '@codemirror/view';
	import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
	import { EditorState } from '@codemirror/state';
	import { markdown } from '@codemirror/lang-markdown';
	import { languages } from '@codemirror/language-data';
	import { documentState } from '$lib/stores/document.svelte';
	import { projectState } from '$lib/stores/project.svelte';
	import { editorTheme } from './editorTheme';
	import { renderMarkdown } from '$lib/markdown';

	type Mode = 'raw' | 'preview';

	let container: HTMLDivElement | undefined = $state(undefined);
	let view: EditorView | undefined;
	let applyingExternalChange = false;
	let mode = $state<Mode>('raw');

	const noFileSelected = $derived(projectState.isOpen && !documentState.path);
	const previewHtml = $derived(mode === 'preview' ? renderMarkdown(documentState.content) : '');

	function createState(doc: string) {
		return EditorState.create({
			doc,
			extensions: [
				history(),
				keymap.of([...defaultKeymap, ...historyKeymap]),
				markdown({ codeLanguages: languages }),
				EditorView.lineWrapping,
				placeholder('Open a Markdown file, or just start typing…'),
				editorTheme,
				EditorView.updateListener.of((update) => {
					if (update.docChanged && !applyingExternalChange) {
						documentState.content = update.state.doc.toString();
					}
				})
			]
		});
	}

	// CodeMirror only exists in the DOM while in 'raw' mode — it's destroyed and recreated on
	// toggle rather than hidden with CSS, since CodeMirror can't measure layout while hidden.
	$effect(() => {
		if (mode === 'raw' && container && !view && !noFileSelected) {
			view = new EditorView({ state: createState(documentState.content), parent: container });
		} else if ((mode !== 'raw' || noFileSelected) && view) {
			view.destroy();
			view = undefined;
		}
	});

	onDestroy(() => view?.destroy());

	$effect(() => {
		const external = documentState.content;
		if (!view) return;
		const current = view.state.doc.toString();
		if (current !== external) {
			applyingExternalChange = true;
			view.dispatch({ changes: { from: 0, to: current.length, insert: external } });
			applyingExternalChange = false;
		}
	});

	const activeSegmentClass = 'bg-[var(--control-active-bg)] text-[var(--text-primary)] shadow-sm';
	const idleSegmentClass =
		'text-[var(--text-muted)] hover:bg-[var(--control-hover)] hover:text-[var(--text-primary)]';
</script>

<div class="app-surface flex h-full flex-col overflow-hidden rounded-2xl">
	<div class="app-panel-header flex items-center justify-end px-2 py-1.5">
		<div class="flex items-center gap-0.5 rounded-lg bg-[var(--surface-inset)] p-0.5">
			<button
				onclick={() => (mode = 'raw')}
				class="rounded-md px-2.5 py-1 text-xs font-medium transition-colors duration-150 {mode ===
				'raw'
					? activeSegmentClass
					: idleSegmentClass}"
			>
				Raw
			</button>
			<button
				onclick={() => (mode = 'preview')}
				class="rounded-md px-2.5 py-1 text-xs font-medium transition-colors duration-150 {mode ===
				'preview'
					? activeSegmentClass
					: idleSegmentClass}"
			>
				Preview
			</button>
		</div>
	</div>

	<div class="flex-1 overflow-auto">
		{#if noFileSelected}
			<div
				class="flex h-full items-center justify-center p-6 text-center text-sm text-[var(--text-muted)]"
			>
				Select a Markdown file from the sidebar to start editing.
			</div>
		{:else if mode === 'raw'}
			<div bind:this={container} class="h-full w-full"></div>
		{:else}
			<div class="markdown-preview p-6 text-sm text-[var(--text-primary)]">
				<!-- eslint-disable-next-line svelte/no-at-html-tags -- previewHtml is DOMPurify-sanitized in $lib/markdown.ts, never raw source -->
				{@html previewHtml}
			</div>
		{/if}
	</div>
</div>
