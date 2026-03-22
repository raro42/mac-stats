//! Strip HTML noise (scripts, styles, nav boilerplate) from fetched pages before
//! sending content to the LLM. Produces a compact text representation that
//! preserves semantic structure (headings, links, lists) while eliminating
//! tags that waste context tokens.

use scraper::{ElementRef, Html, Node};

/// Tags whose entire subtree is dropped (content is never useful for the LLM).
const SKIP_TAGS: &[&str] = &[
    "script", "style", "head", "meta", "link", "noscript", "svg", "iframe", "object", "embed",
];

/// Block-level tags that get a newline before them in the output.
const BLOCK_TAGS: &[&str] = &[
    "p",
    "div",
    "section",
    "article",
    "main",
    "header",
    "footer",
    "nav",
    "aside",
    "blockquote",
    "pre",
    "figure",
    "figcaption",
    "details",
    "summary",
    "table",
    "tr",
    "br",
    "hr",
];

/// Strip HTML noise and return clean, structured text suitable for an LLM.
///
/// The output preserves:
/// - Headings as `# Heading` / `## Heading` etc.
/// - Links as `[text](href)` (absolute hrefs only, relative are text-only)
/// - List items as `- text`
/// - Table rows as pipe-separated values
/// - Block element boundaries as newlines
///
/// Returns the cleaned text. Callers should check for empty output (page was
/// all scripts/frames with no readable content).
pub fn clean_html(raw_html: &str) -> String {
    let document = Html::parse_document(raw_html);
    let mut out = String::with_capacity(raw_html.len() / 4);
    let root = document.root_element();
    walk_node(&root, &mut out);
    collapse_whitespace(&out)
}

fn walk_node(element: &ElementRef, out: &mut String) {
    let tag = element.value().name().to_lowercase();

    if SKIP_TAGS.contains(&tag.as_str()) {
        return;
    }

    let is_block = BLOCK_TAGS.contains(&tag.as_str());
    let is_heading = matches!(tag.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6");
    let is_link = tag == "a";
    let is_li = tag == "li";
    let is_td = tag == "td" || tag == "th";
    let is_tr = tag == "tr";
    let is_br = tag == "br";
    let is_hr = tag == "hr";

    if is_br {
        out.push('\n');
        return;
    }
    if is_hr {
        out.push_str("\n---\n");
        return;
    }

    if is_block || is_heading {
        ensure_newline(out);
    }

    if is_heading {
        let level = tag.chars().nth(1).and_then(|c| c.to_digit(10)).unwrap_or(1);
        for _ in 0..level {
            out.push('#');
        }
        out.push(' ');
    }

    if is_li {
        ensure_newline(out);
        out.push_str("- ");
    }

    if is_td && !out.ends_with('|') && !out.ends_with('\n') && !out.is_empty() {
        out.push_str(" | ");
    }

    if is_link {
        let href = element.value().attr("href").unwrap_or("");
        let text = collect_text(element);
        let text = text.trim();
        if !text.is_empty() {
            if href.starts_with("http://") || href.starts_with("https://") {
                out.push_str(&format!("[{}]({})", text, href));
            } else {
                out.push_str(text);
            }
        }
        return;
    }

    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                let t = text.text.as_ref();
                if !t.trim().is_empty() {
                    out.push_str(t);
                }
            }
            Node::Element(_) => {
                if let Some(child_el) = ElementRef::wrap(child) {
                    walk_node(&child_el, out);
                }
            }
            _ => {}
        }
    }

    if is_tr {
        out.push('\n');
    }

    if is_block || is_heading {
        out.push('\n');
    }
}

/// Collect all text content from an element (for link text etc.).
fn collect_text(element: &ElementRef) -> String {
    element.text().collect::<String>()
}

/// Push a newline only if the output doesn't already end with one.
fn ensure_newline(out: &mut String) {
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
}

/// Collapse runs of blank lines (3+ newlines) into double newlines, and
/// runs of spaces/tabs into single spaces within each line.
fn collapse_whitespace(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut consecutive_newlines = 0u32;

    for line in text.split('\n') {
        let normalized: String = line
            .chars()
            .map(|c| match c {
                // HTML / copied text often inserts break or bidi format controls that
                // `split_whitespace()` leaves inside one token (e.g. U+034F, U+061C, LRM/RLM,
                // U+2066–U+2069). ZWSP, ZWNJ, ZWJ, WJ, BOM-in-text, SHY, and NBSP get the same
                // treatment so FETCH_URL bodies tokenize cleanly for the LLM.
                '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}' | '\u{00AD}'
                | '\u{00A0}' | '\u{034F}' | '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{2066}'
                | '\u{2067}' | '\u{2068}' | '\u{2069}' => ' ',
                _ => c,
            })
            .collect();
        let trimmed = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
        if trimmed.is_empty() {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                result.push('\n');
            }
        } else {
            if consecutive_newlines > 0 && !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            consecutive_newlines = 0;
            result.push_str(&trimmed);
            result.push('\n');
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_and_style() {
        let html = r#"<html><head><style>body{color:red}</style><script>alert(1)</script></head>
            <body><p>Hello world</p><script>var x=1;</script></body></html>"#;
        let cleaned = clean_html(html);
        assert!(!cleaned.contains("alert"));
        assert!(!cleaned.contains("color:red"));
        assert!(!cleaned.contains("var x"));
        assert!(cleaned.contains("Hello world"));
    }

    #[test]
    fn preserves_headings() {
        let html = "<html><body><h1>Title</h1><h2>Subtitle</h2><p>Text</p></body></html>";
        let cleaned = clean_html(html);
        assert!(cleaned.contains("# Title"));
        assert!(cleaned.contains("## Subtitle"));
        assert!(cleaned.contains("Text"));
    }

    #[test]
    fn preserves_links_with_href() {
        let html = r#"<html><body><a href="https://example.com">Click here</a></body></html>"#;
        let cleaned = clean_html(html);
        assert!(cleaned.contains("[Click here](https://example.com)"));
    }

    #[test]
    fn relative_links_text_only() {
        let html = r#"<html><body><a href="/page">Page</a></body></html>"#;
        let cleaned = clean_html(html);
        assert!(cleaned.contains("Page"));
        assert!(!cleaned.contains("[Page]"));
    }

    #[test]
    fn preserves_list_items() {
        let html = "<html><body><ul><li>One</li><li>Two</li><li>Three</li></ul></body></html>";
        let cleaned = clean_html(html);
        assert!(cleaned.contains("- One"));
        assert!(cleaned.contains("- Two"));
        assert!(cleaned.contains("- Three"));
    }

    #[test]
    fn strips_svg_entirely() {
        let html =
            r#"<html><body><p>Before</p><svg><path d="M0 0"/></svg><p>After</p></body></html>"#;
        let cleaned = clean_html(html);
        assert!(!cleaned.contains("path"));
        assert!(cleaned.contains("Before"));
        assert!(cleaned.contains("After"));
    }

    #[test]
    fn strips_noscript_and_iframe() {
        let html = r#"<html><body><noscript>Enable JS</noscript><iframe src="ad.html"></iframe><p>Content</p></body></html>"#;
        let cleaned = clean_html(html);
        assert!(!cleaned.contains("Enable JS"));
        assert!(!cleaned.contains("ad.html"));
        assert!(cleaned.contains("Content"));
    }

    #[test]
    fn empty_page_returns_empty() {
        let html =
            "<html><head><script>all js</script></head><body><script>more</script></body></html>";
        let cleaned = clean_html(html);
        assert!(cleaned.trim().is_empty());
    }

    #[test]
    fn zero_width_space_separates_words() {
        // U+200B is not Unicode whitespace in Rust; pages use it as an invisible
        // break opportunity between words — treat it like a space for LLM text.
        let html = "<html><body><p>hello\u{200B}world</p></body></html>";
        let cleaned = clean_html(html);
        assert!(
            cleaned.contains("hello world"),
            "expected ZWSP normalized before collapse, got {:?}",
            cleaned
        );
        assert!(!cleaned.contains('\u{200B}'));
    }

    #[test]
    fn soft_hyphen_separates_words() {
        // U+00AD (SHY) is common from HTML `&shy;`; Rust does not treat it as whitespace,
        // so without normalization "hello\u{00AD}world" stays one token.
        let html = "<html><body><p>hello\u{00AD}world</p></body></html>";
        let cleaned = clean_html(html);
        assert!(
            cleaned.contains("hello world"),
            "expected soft hyphen normalized before collapse, got {:?}",
            cleaned
        );
        assert!(!cleaned.contains('\u{00AD}'));
    }

    #[test]
    fn nbsp_separates_words() {
        // U+00A0 (NO-BREAK SPACE) from `&nbsp;` is not Unicode whitespace for
        // `split_whitespace()`, so "hello\u{00A0}world" would stay one token.
        let html = "<html><body><p>hello\u{00A0}world</p></body></html>";
        let cleaned = clean_html(html);
        assert!(
            cleaned.contains("hello world"),
            "expected NBSP normalized before collapse, got {:?}",
            cleaned
        );
        assert!(!cleaned.contains('\u{00A0}'));
    }

    #[test]
    fn bidi_and_grapheme_joiner_separate_words() {
        // CGJ, ALM, LRM/RLM, directional isolates: `split_whitespace()` keeps them inside a token.
        for sep in [
            '\u{034F}',
            '\u{061C}',
            '\u{200E}',
            '\u{200F}',
            '\u{2066}',
            '\u{2067}',
            '\u{2068}',
            '\u{2069}',
        ] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {sep:?} normalized before collapse, got {:?}",
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains {sep:?}"
            );
        }
    }

    #[test]
    fn zero_width_joiners_separate_words() {
        // U+200C ZWNJ, U+200D ZWJ, U+2060 WORD JOINER: format controls that Rust does not
        // treat as whitespace; they can appear between letters in copied HTML or emoji text.
        for sep in ['\u{200C}', '\u{200D}', '\u{2060}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {sep:?} normalized before collapse, got {:?}",
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains {sep:?}"
            );
        }
    }

    #[test]
    fn collapses_excessive_whitespace() {
        let html = "<html><body><p>A</p>\n\n\n\n\n<p>B</p></body></html>";
        let cleaned = clean_html(html);
        let newline_runs: Vec<&str> = cleaned.split("\n\n\n").collect();
        assert_eq!(
            newline_runs.len(),
            1,
            "Should not have 3+ consecutive newlines"
        );
    }

    #[test]
    fn table_rows_pipe_separated() {
        let html = "<html><body><table><tr><th>Name</th><th>Age</th></tr><tr><td>Alice</td><td>30</td></tr></table></body></html>";
        let cleaned = clean_html(html);
        assert!(cleaned.contains("Name"));
        assert!(cleaned.contains("Age"));
        assert!(cleaned.contains("Alice"));
        assert!(cleaned.contains("30"));
    }

    #[test]
    fn real_world_html_compression() {
        let html = r#"<!DOCTYPE html><html><head>
            <meta charset="utf-8"><title>Test Page</title>
            <style>.nav{display:flex}.hero{background:url(img.jpg)}</style>
            <script src="app.js"></script>
            <script>window.analytics={track:function(){}}</script>
            <link rel="stylesheet" href="style.css">
            </head><body>
            <nav><a href="/">Home</a><a href="/about">About</a></nav>
            <main>
              <h1>Welcome to Our Site</h1>
              <p>This is the main content that the user cares about.</p>
              <p>It has <a href="https://example.com/link">a link</a> in it.</p>
              <ul><li>Feature one</li><li>Feature two</li></ul>
            </main>
            <script>!function(){console.log("analytics")}();</script>
            <footer><p>Copyright 2024</p></footer>
            </body></html>"#;
        let cleaned = clean_html(html);
        let ratio = cleaned.len() as f64 / html.len() as f64;
        assert!(
            ratio < 0.5,
            "Cleaned should be <50% of original, got {:.0}%",
            ratio * 100.0
        );
        assert!(cleaned.contains("# Welcome to Our Site"));
        assert!(cleaned.contains("main content"));
        assert!(cleaned.contains("[a link](https://example.com/link)"));
        assert!(cleaned.contains("- Feature one"));
        assert!(!cleaned.contains("analytics"));
        assert!(!cleaned.contains("display:flex"));
    }
}
