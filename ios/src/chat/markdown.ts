// Markdown de las respuestas del modelo. La salida del modelo no es de fiar: se
// sanitiza con DOMPurify y una lista blanca corta (sin imágenes, estilos ni scripts)
// antes de insertarla, y nunca se ejecuta nada de lo que devuelve.
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

/** Los enlaces de las respuestas no navegan: la app no tiene navegador. */
export function disableLinks(container: HTMLElement): void {
  container.addEventListener("click", (event) => {
    if ((event.target as HTMLElement).closest("a")) event.preventDefault();
  });
}
