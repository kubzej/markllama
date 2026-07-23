import { EditorView } from '@codemirror/view';

// Colors are CSS custom properties (see layout.css) so the editor follows the
// OS light/dark setting without any JS-side theme switching.
export const editorTheme = EditorView.theme({
	'&': {
		height: '100%',
		color: 'var(--editor-fg)',
		backgroundColor: 'transparent',
		fontSize: '14px'
	},
	'.cm-content': {
		fontFamily: 'var(--font-mono)',
		caretColor: 'var(--color-accent)',
		padding: '1.25rem 1.5rem'
	},
	'.cm-scroller': {
		fontFamily: 'inherit'
	},
	'&.cm-focused': {
		outline: 'none'
	},
	'.cm-selectionBackground, ::selection': {
		backgroundColor: 'var(--editor-selection) !important'
	},
	'.cm-gutters': {
		display: 'none'
	},
	'.cm-line': {
		padding: '0'
	}
});
