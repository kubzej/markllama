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

	const activeSegmentClass = 'bg-white text-neutral-900 shadow-sm dark:bg-neutral-700 dark:text-white';
	const idleSegmentClass =
		'text-neutral-500 hover:text-neutral-700 dark:text-neutral-400 dark:hover:text-neutral-200';
</script>

<div
	class="flex h-full flex-col overflow-hidden rounded-2xl bg-white shadow-sm ring-1 ring-neutral-200/70 dark:bg-neutral-900 dark:ring-white/[0.06]"
>
	<div
		class="flex items-center justify-end border-b border-neutral-200/70 px-2 py-1.5 dark:border-white/[0.06]"
	>
		<div class="flex items-center gap-0.5 rounded-lg bg-neutral-100 p-0.5 dark:bg-white/5">
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
				class="flex h-full items-center justify-center p-6 text-center text-sm text-neutral-400 dark:text-neutral-500"
			>
				Select a Markdown file from the sidebar to start editing.
			</div>
		{:else if mode === 'raw'}
			<div bind:this={container} class="h-full w-full"></div>
		{:else}
			<div class="markdown-preview p-6 text-sm text-neutral-800 dark:text-neutral-200">
				<!-- eslint-disable-next-line svelte/no-at-html-tags -- previewHtml is DOMPurify-sanitized in $lib/markdown.ts, never raw source -->
				{@html previewHtml}
			</div>
		{/if}
	</div>
</div>

<style>
	.markdown-preview :global(h1) {
		font-size: 1.6em;
		font-weight: 700;
		margin: 0.9em 0 0.4em;
		line-height: 1.3;
	}
	.markdown-preview :global(h2) {
		font-size: 1.3em;
		font-weight: 700;
		margin: 0.9em 0 0.4em;
		line-height: 1.3;
	}
	.markdown-preview :global(h3) {
		font-size: 1.1em;
		font-weight: 600;
		margin: 0.8em 0 0.4em;
	}
	.markdown-preview :global(h1:first-child),
	.markdown-preview :global(h2:first-child),
	.markdown-preview :global(h3:first-child) {
		margin-top: 0;
	}
	.markdown-preview :global(p) {
		margin: 0.6em 0;
		line-height: 1.65;
	}
	.markdown-preview :global(ul) {
		margin: 0.6em 0;
		padding-left: 1.4em;
		list-style: disc;
	}
	.markdown-preview :global(ol) {
		margin: 0.6em 0;
		padding-left: 1.4em;
		list-style: decimal;
	}
	.markdown-preview :global(li) {
		margin: 0.25em 0;
		line-height: 1.6;
	}
	.markdown-preview :global(a) {
		color: var(--color-accent);
		text-decoration: underline;
	}
	.markdown-preview :global(strong) {
		font-weight: 600;
	}
	.markdown-preview :global(code) {
		font-family: var(--font-mono);
		background: var(--editor-selection);
		padding: 0.15em 0.4em;
		border-radius: 4px;
		font-size: 0.9em;
	}
	.markdown-preview :global(pre) {
		background: var(--editor-selection);
		padding: 0.8em 1em;
		border-radius: 10px;
		overflow-x: auto;
		margin: 0.8em 0;
	}
	.markdown-preview :global(pre code) {
		background: none;
		padding: 0;
	}
	.markdown-preview :global(blockquote) {
		border-left: 3px solid var(--color-accent);
		padding-left: 1em;
		margin: 0.8em 0;
		opacity: 0.85;
	}
	.markdown-preview :global(hr) {
		border: none;
		border-top: 1px solid currentColor;
		opacity: 0.15;
		margin: 1.4em 0;
	}
	.markdown-preview :global(table) {
		border-collapse: collapse;
		margin: 0.8em 0;
		font-size: 0.95em;
	}
	.markdown-preview :global(th),
	.markdown-preview :global(td) {
		border: 1px solid var(--editor-selection);
		padding: 0.4em 0.8em;
		text-align: left;
	}
	.markdown-preview :global(img) {
		max-width: 100%;
		border-radius: 8px;
	}
</style>
