export interface ImageAttachment {
	base64: string;
	mimeType: string;
}

const MIME_BY_EXTENSION: Record<string, string> = {
	png: 'image/png',
	jpg: 'image/jpeg',
	jpeg: 'image/jpeg',
	gif: 'image/gif',
	webp: 'image/webp'
};

export function mimeTypeForPath(path: string): string {
	const ext = path.split('.').pop()?.toLowerCase() ?? '';
	return MIME_BY_EXTENSION[ext] ?? 'image/png';
}

export function toDataUrl(image: ImageAttachment): string {
	return `data:${image.mimeType};base64,${image.base64}`;
}
