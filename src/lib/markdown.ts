import { marked } from 'marked';
import DOMPurify from 'dompurify';

marked.setOptions({ breaks: true, gfm: true });

/**
 * Renders markdown to sanitized HTML for the read-only preview. Sanitizing matters even though
 * this is "your own" file — a downloaded/shared `.md` could carry embedded script/HTML, and
 * there's no reason a preview pane needs to execute it.
 */
export function renderMarkdown(source: string): string {
	const html = marked.parse(source, { async: false });
	return DOMPurify.sanitize(html);
}
