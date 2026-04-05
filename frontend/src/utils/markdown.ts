import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js/lib/common';
import 'highlight.js/styles/github-dark.css';

/**
 * Custom renderer to handle code highlighting
 */
const renderer = new marked.Renderer();

renderer.code = ({ text, lang }) => {
    const language = lang && hljs.getLanguage(lang) ? lang : 'plaintext';
    const highlighted = hljs.highlight(text, { language }).value;
    return `<pre><code class="hljs ${language}">${highlighted}</code></pre>`;
};

marked.use({
    renderer,
    breaks: true,
    gfm: true
});

import { replaceEmojiNames } from './emoji';

/**
 * Renders markdown string to safe HTML.
 * @param markdown - The raw markdown string from the message
 * @param highlightMentions - Optional current username to highlight mentions
 * @returns Safe HTML string
 */
export function renderMarkdown(markdown: string, highlightMentions?: string): string {
    if (!markdown) return '';

    // Step 0: Replace inline emoji names
    const emojified = replaceEmojiNames(markdown);

    // Step 1: Parse Markdown
    const html = marked.parse(emojified) as string;

    // Step 2: Sanitize HTML
    const sanitizedHtml = DOMPurify.sanitize(html, {
        ALLOWED_TAGS: [
            'p', 'br', 'strong', 'em', 'code', 'pre', 'span', 'ul', 'ol', 'li',
            'blockquote', 'a', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'table', 'thead', 'tbody', 'tr', 'th', 'td'
        ],
        ALLOWED_ATTR: ['href', 'target', 'class', 'style', 'rel']
    });

    // Step 3: Post-process for Mentions (Interactive)
    const processedHtml = sanitizedHtml.replace(
        /@(\w+)/g,
        (_match, username) => {
            const isMe = highlightMentions && username === highlightMentions;
            const highlightClass = isMe
                ? 'bg-warning/20 text-warning font-bold px-0.5 rounded border border-warning/30'
                : 'text-brand font-semibold hover:underline cursor-pointer';
            return `<span class="mention ${highlightClass}" data-username="${username}">@${username}</span>`;
        }
    );

    return processedHtml;
}
