import MarkdownIt from "markdown-it";
import { createHighlighter, type Highlighter } from "shiki";

// Languages we eagerly load for fenced-code highlighting in writeups.
const LANGS = [
  "c",
  "cpp",
  "python",
  "rust",
  "javascript",
  "typescript",
  "bash",
  "shellscript",
  "json",
  "yaml",
  "diff",
  "asm",
  "makefile",
  "go",
  "java",
];

const THEME = "github-dark";

let highlighter: Highlighter | null = null;

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

/** Load the Shiki highlighter once. Safe to call repeatedly. */
export async function initHighlighter(): Promise<void> {
  if (highlighter) return;
  highlighter = await createHighlighter({ themes: [THEME], langs: LANGS });
}

const md: MarkdownIt = new MarkdownIt({
  html: false,
  linkify: true,
  typographer: false,
  highlight: (code, lang) => {
    if (highlighter) {
      const loaded = highlighter.getLoadedLanguages();
      const language = lang && loaded.includes(lang) ? lang : "text";
      try {
        return highlighter.codeToHtml(code, { lang: language, theme: THEME });
      } catch {
        /* fall through to plain rendering */
      }
    }
    return `<pre class="shiki-fallback"><code>${escapeHtml(code)}</code></pre>`;
  },
});

export function renderMarkdown(src: string): string {
  return md.render(src);
}
