// Markdown for the model's replies. Model output is untrusted: it is sanitized with
// DOMPurify and a short allowlist (no images, styles or scripts) before it is inserted,
// and nothing it returns is ever executed.
import DOMPurify from "dompurify";
import { marked } from "marked";

const ALLOWED_TAGS = [
  "p", "br", "strong", "em", "b", "i", "del", "code", "pre", "blockquote",
  "ul", "ol", "li", "h1", "h2", "h3", "h4", "hr", "a",
  "table", "thead", "tbody", "tr", "th", "td",
];

export function renderMarkdown(text: string): string {
  const html = marked.parse(text, { async: false, gfm: true, breaks: true });
  return DOMPurify.sanitize(html, { ALLOWED_TAGS, ALLOWED_ATTR: ["href"], ALLOW_DATA_ATTR: false });
}

/** Links in replies do not navigate: the app has no browser. */
export function disableLinks(container: HTMLElement): void {
  container.addEventListener("click", (event) => {
    if ((event.target as HTMLElement).closest("a")) event.preventDefault();
  });
}
