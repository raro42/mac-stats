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
/// - `<img>` elements with non-empty `alt` or `title` as inline `[Image: …]` text
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
    let is_img = tag == "img";

    if is_br {
        out.push('\n');
        return;
    }
    if is_hr {
        out.push_str("\n---\n");
        return;
    }
    if is_img {
        let alt = element.value().attr("alt").unwrap_or("").trim();
        let title = element.value().attr("title").unwrap_or("").trim();
        let desc = if !alt.is_empty() {
            alt
        } else if !title.is_empty() {
            title
        } else {
            ""
        };
        if !desc.is_empty() {
            out.push_str("[Image: ");
            out.push_str(desc);
            out.push(']');
        }
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
                // U+202A–U+202E bidi embedding/override/pop, U+2061–U+206F invisible math +
                // bidi/shaping controls). ZWSP, ZWNJ, ZWJ, WJ, BOM-in-text, SHY, NBSP,
                // Mongolian free variation selectors + vowel separator, Hangul / halfwidth
                // filler jamo, variation selectors (emoji / IVS), interlinear annotation
                // markers, object replacement (U+FFF9–U+FFFC), and deprecated Unicode language
                // tag characters (U+E0000–U+E007F) get the same treatment so FETCH_URL bodies
                // tokenize cleanly. Khmer inherent vowels (U+17B4, U+17B5) are Cf and not Rust
                // whitespace, so Khmer-layout HTML can otherwise glue adjacent Latin tokens.
                // Latin-1 inverted exclamation / question (U+00A1, U+00BF) and double angle quotes
                // (U+00AB, U+00BB, Pi/Pf) are not Rust whitespace; Spanish / French typography in
                // HTML can place them between Latin tokens without ASCII space. Section sign (U+00A7)
                // and pilcrow (U+00B6, Po) are not Rust whitespace either; legal or editorial HTML
                // often uses them between Latin tokens without ASCII space. Greek question mark
                // (U+037E, Po; erotimatiko) is not Rust whitespace and is distinct from Greek ano
                // teleia (U+0387) mapped with middle-dot punctuation. Arabic comma / semicolon /
                // question / full stop (U+060C, U+061B, U+061F, U+06D4, Po) and Arabic percent /
                // decimal / thousands separators (U+066A–U+066C, Po) are not Rust whitespace; RTL or
                // bilingual numeric HTML can glue Latin tokens for `split_whitespace()`.
                // Armenian exclamation / comma / question / abbreviation mark (U+055C–U+055F, Po) and
                // full stop / hyphen (U+0589–U+058A, Po/Pd) are not Rust whitespace; bilingual or
                // Unicode-sample HTML can glue Latin tokens. U+055A (ARMENIAN APOSTROPHE) and U+055B
                // (EMPHASIS MARK, Po) are omitted—apostrophe- or stress-like marks can sit word-internally
                // in Armenian (same spirit as omitting U+2019 for Latin contractions).
                // Devanagari danda / double danda (U+0964, U+0965, Po) are not Rust whitespace; mixed
                // Latin–Devanagari or Unicode-sample HTML can place them between Latin tokens without ASCII space.
                // Thai PAIYANNOI / FONGMAN / ANGKHANKHU / KHOMUT (U+0E2F, U+0E4F, U+0E5A, U+0E5B, Po), Lao
                // ELLIPSIS (U+0EAF, Po), and Myanmar LITTLE SECTION / SECTION (U+104A–U+104B, Po) are not Rust
                // whitespace; Southeast Asian–Latin bilingual or Unicode-sample HTML can glue Latin tokens without
                // ASCII space. Thai MAIYAMOK (U+0E46, Lm) stays unmapped—modifier-like, can repeat word-internally.
                // Arabic number signs / ayah markers (U+0600–U+0605, U+06DD, U+08E2), Arabic
                // Extended-A currency format marks (U+0890–U+0891, pound/piastre mark above), and
                // Syriac abbreviation mark (U+070F) are Cf and not Rust whitespace; RTL scholarly
                // or financial HTML can place them between scripts without a real space. Mongolian
                // U+1800–U+180E (BIRGA through vowel separator) are Po/Pd/Mn/Cf and not Rust
                // whitespace—sentence punctuation (U+1800–U+1805, U+1807–U+180A), TODO soft hyphen
                // (U+1806, Pd), free variation selectors (U+180B–U+180D, Mn), vowel separator
                // (U+180E, Cf)—so mixed or pasted Mongolian/Manchu HTML can glue Latin tokens.
                // Middle dot (U+00B7, Po), Greek ano teleia (U+0387, Po), Katakana middle dot
                // (U+30FB, Po), and halfwidth Katakana middle dot (U+FF65, Po) are not Rust
                // whitespace; European / Greek / Japanese typography often uses them as word
                // separators, so pasted HTML can glue Latin tokens for `split_whitespace()`.
                // Dagger / double dagger / bullet / triangular bullet (U+2020–U+2023, Po) sit
                // between the curly-quote arm and dot-leader arm; U+2019 between them is omitted
                // (apostrophe). None are Rust whitespace; footnote-style or list HTML can glue
                // Latin tokens without ASCII space. Bullet operator (U+2219, Sm) and dot operator
                // (U+22C5, Sm) are not Rust whitespace either; MathML-style HTML can do the same. Fraction slash (U+2044, Sm), division slash
                // (U+2215, Sm), and fullwidth solidus (U+FF0F, Po) are not Rust whitespace either;
                // math fractions, MathML, or CJK fullwidth paths can use them between Latin tokens
                // without ASCII space. Ideographic comma / full stop (U+3001, U+3002, Po) and
                // fullwidth ASCII-like punctuation (U+FF0C comma, U+FF1A colon, U+FF1B semicolon,
                // U+FF01 exclamation, U+FF1F question; all Po) are not Rust whitespace; CJK or
                // mixed-layout HTML often places them between Latin tokens without ASCII space.
                // CJK Symbols and Punctuation: ditto mark (U+3003, Po), JIS symbol (U+3004, So),
                // ideographic closing mark (U+3006, Lo), CJK brackets and postal/geta marks
                // (U+3008–U+301B, Ps/Pe/So), wave dash (U+301C, Pd), reversed/double-prime quotes
                // (U+301D–U+301F), postal mark face (U+3020, So), vertical kana repeat marks
                // (U+3031–U+3036, Lm), masu mark (U+303C, Lo), ideographic variation indicator /
                // half fill space (U+303E–U+303F, So) are not Rust whitespace. Omitted on purpose:
                // iteration marks U+3005 / U+303B (Lm), ideographic zero U+3007 and Hangzhou numerals
                // U+3021–U+3029 / U+3038–U+303A (Nl), and ideographic tone marks U+302A–U+302F (Mn/Mc).
                // Vertical Forms compatibility punctuation (U+FE10–U+FE19, Po/Ps/Pe/Pc) is not
                // Rust whitespace. Small Form Variants (U+FE50–U+FE52, U+FE54–U+FE66, U+FE68–U+FE6B;
                // Po / Pd / Ps / Pe / Sm / Sc as assigned—skips unassigned U+FE53, U+FE67, U+FE6C–U+FE6F)
                // are not Rust whitespace; compatibility typography HTML can glue Latin tokens without
                // ASCII space. Wavy dash (U+3030, Pd), ideographic telegraph line-feed separator
                // (U+3037, So), part alternation mark (U+303D, Po), Katakana-Hiragana double hyphen
                // (U+30A0, Pd), and fullwidth low line (U+FF3F, Pc) are not Rust whitespace either;
                // mixed CJK / romanization HTML can do the same. Hebrew maqaf (U+05BE, Pd), paseq (U+05C0, Po), and sof pasuq
                // (U+05C3, Po; sentence end like a colon) are not Rust whitespace. Georgian paragraph separator (U+10FB, Po) is
                // not Rust whitespace; mixed Latin–Georgian or Unicode-sample HTML can glue tokens without ASCII space. Tibetan yig mgo
                // and shad marks (U+0F04–U+0F12, Po), gter tsheg (U+0F14, Po), corner brackets (U+0F3A–U+0F3D, Ps/Pe), paluta (U+0F85,
                // Po), and astrological / editorial marks (U+0FD0–U+0FD4, U+0FD9–U+0FDA, Po) are not Rust whitespace—only intersyllabic
                // tsheg (U+0F0B) was covered before; mixed Tibetan–Latin or Unicode-sample HTML can otherwise glue Latin tokens. U+0F13
                // (caret So) stays unmapped. Sinhala kunddaliya (U+0DF4, Po), Limbu tokma / exclamation / question (U+1940, U+1944,
                // U+1945, Po), and Meetei Mayek cheikhei / ahang khuda (U+AAF0, U+AAF1, Po) are not Rust whitespace; mixed-script or
                // Unicode-sample HTML can glue Latin tokens without ASCII space. Ethiopic full stop (U+1362, Po) is not Rust whitespace;
                // mixed-script HTML can glue Latin tokens. Khmer signs khan through koomuut (U+17D4–U+17DA, Po) are not Rust whitespace;
                // Javanese pada / section marks (U+A9C1–U+A9CF, Po) are not Rust whitespace—U+A9C0 PANGKON (Cf, virama) stays unmapped as
                // word-internal risk alongside U+17B4/U+17B5 inherent vowels already mapped as format controls.
                // Balinese carik through pamengkeb (U+1B5A–U+1B60, Po) are not Rust whitespace; Balinese–Latin or Unicode-sample HTML can glue
                // Latin tokens without ASCII space.
                // Batak bindu symbols (U+1BFC–U+1BFF, Po), Lepcha punctuation ta-rol through tshook (U+1C3B–U+1C3F, Po), and Ol Chiki mucaad /
                // double mucaad (U+1C7E–U+1C7F, Po) are not Rust whitespace; Batak–Latin, Lepcha–Latin, Ol Chiki–Latin, or Unicode-sample HTML
                // can glue Latin tokens without ASCII space. Cham spiral and danda marks (U+AA5C–U+AA5F, Po) are not Rust whitespace;
                // Cham–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space. Tai Viet HO HOI / KOI KOI
                // (U+AADE–U+AADF, Po) are not Rust whitespace; Tai Viet–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // U+AADB KON and U+AADC NUENG are Lo; U+AADD SAM is Lm—omitted (word-internal risk, like Thai U+0E46).
                // Unicode dash punctuation (U+2010–U+2015, Pd)—hyphen, non-breaking hyphen, figure
                // dash, en dash, em dash, horizontal bar—are not Rust whitespace; typographic HTML or
                // pasted Office copy often uses them between Latin tokens without ASCII space.
                // Double vertical line / double low line (U+2016–U+2017, Po) and single/double
                // quotation marks U+2018, U+201A–U+201F (Pi/Pf/Ps) are not Rust whitespace; curly-quote
                // HTML can glue Latin tokens without ASCII space. U+2019 RIGHT SINGLE QUOTATION MARK is
                // omitted: it is the usual typographic apostrophe inside contractions (`don't`).
                // Per mille / per ten thousand (U+2030–U+2031, Po), prime marks (U+2032–U+2037, Po),
                // caret (U+2038, Po), single guillemets (U+2039–U+203A, Pi/Pf), and reference mark
                // (U+203B, Po) are not Rust whitespace; measurements, foot/inch notation, or European
                // typography in HTML can place them between Latin tokens without ASCII space.
                // U+203C–U+205E (double exclamation through punctuation boundary; includes fraction
                // slash U+2044, undertie, Tironian et, reversed pilcrow, four-dot punctuation, etc.;
                // Po/Pc/Ps/Pe/Sm) are not Rust whitespace—U+205F (medium mathematical space) is, so it
                // stops before U+2061. Pasted UI copy or scholarly HTML can glue Latin tokens here.
                // One dot leader / two dot leader / horizontal
                // ellipsis / hyphenation point (U+2024–U+2027, Po) are not Rust whitespace either;
                // TOC-style leaders or UI copy like "more…" can glue Latin tokens without ASCII space.
                // Supplemental Punctuation U+2E00–U+2E5D (editorial / transcription brackets and
                // marks, word-separator dots, dashes, stenographic and medieval punctuation, Tironian
                // et, specialized brackets, oblique hyphen; Po / Pd / Pi / Pf / Ps / Pe / Lm / So as
                // assigned) are not Rust whitespace—the full block can sit between Latin tokens in
                // scholarly HTML without ASCII space. Earlier arms covered only U+2E31–U+2E3B; U+2E00–
                // U+2E30 and U+2E3C–U+2E5D are now included in one range. Runic single /
                // multiple / cross punctuation (U+16EB–U+16ED, Po) are not Rust whitespace; epigraphic
                // or Unicode-sample HTML can glue Latin tokens for `split_whitespace()`. Aegean word
                // separator line / dot (U+10100–U+10101, Po) and Phoenician word separator (U+1091F,
                // Po) are not Rust whitespace; scholarly or mixed-script HTML can place them between
                // Latin tokens without ASCII space.
                // Ethiopic wordspace (U+1361, Po) and Braille pattern blank (U+2800, So) are not Rust
                // whitespace. Duployan thick letter selector / double mark (U+1BC9D–U+1BC9E, Mn) and
                // shorthand format overlap / step (U+1BCA0–U+1BCA3, Cf) are not Rust whitespace.
                // Kaithi number signs (U+110BD, U+110CD, Cf) are not Rust whitespace; Indic numeral
                // layout HTML can place them between scripts without an ASCII space. Egyptian
                // hieroglyph format joiners / segment markers (U+13430–U+13455, Cf) and musical
                // symbol begin/end beam–tie–slur–phrase (U+1D173–
                // U+1D17A, Cf) are not Rust whitespace; scholarly or MusicXML-derived HTML can place
                // them between scripts without an ASCII space. Brahmi number joiner (U+1107F, Mn) is
                // not Rust whitespace; Indic numeral layout can sit between scripts without ASCII
                // space. Ideographic description characters (U+2FF0–U+2FFB, So) are not Rust
                // whitespace; Han-component notation or pasted CJK-scholarly HTML can glue Latin
                // tokens.
                '\u{0600}'..='\u{0605}'
                | '\u{06DD}'
                | '\u{070F}'
                | '\u{08E2}'
                | '\u{0890}'..='\u{0891}'
                | '\u{200B}'
                | '\u{200C}'
                | '\u{200D}'
                | '\u{2060}'
                | '\u{FEFF}'
                | '\u{00AD}'
                | '\u{00A0}'
                | '\u{17B4}'
                | '\u{17B5}'
                | '\u{17D4}'..='\u{17DA}'
                | '\u{A9C1}'..='\u{A9CF}'
                | '\u{1B5A}'..='\u{1B60}'
                | '\u{1BFC}'..='\u{1BFF}'
                | '\u{1C3B}'..='\u{1C3F}'
                | '\u{1C7E}'..='\u{1C7F}'
                | '\u{AA5C}'..='\u{AA5F}'
                | '\u{AADE}'..='\u{AADF}'
                | '\u{034F}'
                | '\u{061C}'
                | '\u{200E}'
                | '\u{200F}'
                | '\u{202A}'..='\u{202E}'
                | '\u{1800}'..='\u{180E}'
                | '\u{115F}'
                | '\u{1160}'
                | '\u{3164}'
                | '\u{FFA0}'
                | '\u{00B7}'
                | '\u{0387}'
                | '\u{30FB}'
                | '\u{FF65}'
                | '\u{2215}'
                | '\u{FF0F}'
                | '\u{3001}'
                | '\u{3002}'
                | '\u{3003}'
                | '\u{3004}'
                | '\u{3006}'
                | '\u{3008}'..='\u{301B}'
                | '\u{301C}'
                | '\u{301D}'..='\u{301F}'
                | '\u{3020}'
                | '\u{3031}'..='\u{3036}'
                | '\u{303C}'
                | '\u{303E}'
                | '\u{303F}'
                | '\u{FF0C}'
                | '\u{FF1A}'
                | '\u{FF1B}'
                | '\u{FF01}'
                | '\u{FF1F}'
                | '\u{FE10}'..='\u{FE19}'
                | '\u{FE50}'..='\u{FE52}'
                | '\u{FE54}'..='\u{FE66}'
                | '\u{FE68}'..='\u{FE6B}'
                | '\u{3030}'
                | '\u{3037}'
                | '\u{303D}'
                | '\u{30A0}'
                | '\u{FF3F}'
                | '\u{05BE}'
                | '\u{05C0}'
                | '\u{05C3}'
                | '\u{10FB}'
                | '\u{00A1}'
                | '\u{00BF}'
                | '\u{00AB}'
                | '\u{00BB}'
                | '\u{00A7}'
                | '\u{00B6}'
                | '\u{037E}'
                | '\u{060C}'
                | '\u{061B}'
                | '\u{061F}'
                | '\u{06D4}'
                | '\u{066A}'..='\u{066C}'
                | '\u{055C}'..='\u{055F}'
                | '\u{0589}'
                | '\u{058A}'
                | '\u{0964}'
                | '\u{0965}'
                | '\u{0E2F}'
                | '\u{0E4F}'
                | '\u{0E5A}'
                | '\u{0E5B}'
                | '\u{0EAF}'
                | '\u{104A}'..='\u{104B}'
                | '\u{0F04}'..='\u{0F12}'
                | '\u{0F14}'
                | '\u{0F3A}'..='\u{0F3D}'
                | '\u{0F85}'
                | '\u{0FD0}'..='\u{0FD4}'
                | '\u{0FD9}'..='\u{0FDA}'
                | '\u{0DF4}'
                | '\u{1940}'
                | '\u{1944}'
                | '\u{1945}'
                | '\u{AAF0}'
                | '\u{AAF1}'
                | '\u{1362}'
                | '\u{2010}'..='\u{2015}'
                | '\u{2016}'..='\u{2018}'
                | '\u{201A}'..='\u{201F}'
                | '\u{2020}'..='\u{2023}'
                | '\u{2024}'..='\u{2027}'
                | '\u{2030}'..='\u{203B}'
                | '\u{203C}'..='\u{205E}'
                | '\u{2E00}'..='\u{2E5D}'
                | '\u{16EB}'..='\u{16ED}'
                | '\u{10100}'..='\u{10101}'
                | '\u{1091F}'
                | '\u{2219}'
                | '\u{22C5}'
                | '\u{1361}'
                | '\u{2800}'
                | '\u{1BC9D}'..='\u{1BC9E}'
                | '\u{1BCA0}'..='\u{1BCA3}'
                | '\u{1107F}'
                | '\u{110BD}'
                | '\u{110CD}'
                | '\u{13430}'..='\u{13455}'
                | '\u{2FF0}'..='\u{2FFB}'
                | '\u{1D173}'..='\u{1D17A}'
                | '\u{FE00}'..='\u{FE0F}'
                | '\u{E0100}'..='\u{E01EF}'
                | '\u{E0000}'..='\u{E007F}'
                | '\u{2061}'..='\u{206F}'
                | '\u{FFF9}'..='\u{FFFC}' => ' ',
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
    fn img_alt_emitted_as_readable_text() {
        let html = r#"<html><body><p>Before <img src="/x.png" alt="Company logo"> after</p></body></html>"#;
        let cleaned = clean_html(html);
        assert!(
            cleaned.contains("[Image: Company logo]"),
            "expected img alt in output, got {:?}",
            cleaned
        );
        assert!(cleaned.contains("Before"));
        assert!(cleaned.contains("after"));
    }

    #[test]
    fn img_title_used_when_alt_empty() {
        let html =
            r#"<html><body><p><img src="/c.svg" alt="" title="Revenue chart"></p></body></html>"#;
        let cleaned = clean_html(html);
        assert!(
            cleaned.contains("[Image: Revenue chart]"),
            "expected title fallback, got {:?}",
            cleaned
        );
    }

    #[test]
    fn img_alt_preferred_over_title() {
        let html =
            r#"<html><body><img src="/d.png" alt="Short" title="Longer tooltip"></body></html>"#;
        let cleaned = clean_html(html);
        assert!(
            cleaned.contains("[Image: Short]"),
            "expected alt to win over title, got {:?}",
            cleaned
        );
        assert!(!cleaned.contains("Longer tooltip"));
    }

    #[test]
    fn img_without_alt_or_title_emits_nothing() {
        let html = r#"<html><body><p>Text <img src="/z.gif"> more</p></body></html>"#;
        let cleaned = clean_html(html);
        assert!(
            !cleaned.contains("[Image:"),
            "decorative img should not add placeholder, got {:?}",
            cleaned
        );
        assert!(cleaned.contains("Text"));
        assert!(cleaned.contains("more"));
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
    fn bidi_embedding_and_override_separate_words() {
        // U+202A–U+202E: embedding, override, and pop directional formatting — Cf, not Rust
        // whitespace, common in copied RTL/LTR web text.
        for sep in ['\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}'] {
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
    fn bidi_and_grapheme_joiner_separate_words() {
        // CGJ, ALM, LRM/RLM, directional isolates: `split_whitespace()` keeps them inside a token.
        for sep in [
            '\u{034F}', '\u{061C}', '\u{200E}', '\u{200F}', '\u{2066}', '\u{2067}', '\u{2068}',
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
    fn invisible_math_and_bidi_format_separate_words() {
        // U+2061–U+206F: function application (2061), invisible times/separator/plus (2062–4),
        // unassigned slot, bidi isolates (2066–9), and symmetric-swapping / Arabic shaping /
        // digit-shape format chars — all Cf, not Rust whitespace, so they would glue tokens
        // without normalization.
        for cp in 0x2061u32..=0x206F {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                cp,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                cp
            );
        }
    }

    #[test]
    fn interlinear_annotation_and_object_replacement_separate_words() {
        // U+FFF9–U+FFFB (interlinear annotation anchor/separator/terminator) and U+FFFC
        // (object replacement) are Cf and not Rust whitespace; they can appear in copied HTML.
        for sep in ['\u{FFF9}', '\u{FFFA}', '\u{FFFB}', '\u{FFFC}'] {
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
    fn variation_selectors_separate_words() {
        // U+FE00–U+FE0F (emoji / text presentation, IVS) and U+E0100–U+E01EF (variation
        // selectors supplement) are not Rust whitespace; copied HTML or plain text can place
        // them between Latin tokens without a real space.
        for sep in ['\u{FE00}', '\u{FE0F}', '\u{E0100}', '\u{E01EF}'] {
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
    fn arabic_extended_a_currency_format_marks_separate_words() {
        // U+0890 / U+0891 (Arabic pound mark above / piastre mark above): Cf, not Rust whitespace.
        for sep in ['\u{0890}', '\u{0891}'] {
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
    fn arabic_and_syriac_edition_format_separate_words() {
        // U+0600–U+0605 (Arabic number sign / edition marks), U+06DD (end of ayah), U+08E2
        // (disputed end of ayah), U+070F (Syriac abbreviation mark): Cf, not Rust whitespace.
        for sep in [
            '\u{0600}', '\u{0601}', '\u{0602}', '\u{0603}', '\u{0604}', '\u{0605}', '\u{06DD}',
            '\u{070F}', '\u{08E2}',
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
    fn latin1_section_sign_and_pilcrow_separate_words() {
        // U+00A7 SECTION SIGN, U+00B6 PILCROW SIGN (Po); not Rust whitespace—legal or editorial HTML
        // can place them between Latin tokens without ASCII space.
        for sep in ['\u{00A7}', '\u{00B6}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {sep:?} normalized before collapse, got {:?}",
                cleaned
            );
        }
    }

    #[test]
    fn armenian_script_punctuation_separates_words() {
        // U+055C–U+055F (exclamation, comma, question, abbreviation mark; Po) and U+0589 / U+058A
        // (full stop, hyphen; Po/Pd)—not Rust whitespace. U+055A apostrophe and U+055B emphasis
        // stay unmapped (word-internal risk, like U+2019).
        for sep in ['\u{055C}', '\u{055D}', '\u{055E}', '\u{055F}', '\u{0589}', '\u{058A}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                sep as u32
            );
        }
    }

    #[test]
    fn latin1_greek_and_arabic_script_punctuation_separate_words() {
        // U+00A1 / U+00BF (inverted ! / ?), U+00AB / U+00BB (guillemets), U+037E (Greek question
        // mark), U+060C / U+061B / U+061F / U+06D4 (Arabic comma, semicolon, question, full stop),
        // U+066A–U+066C (Arabic percent, decimal sep, thousands sep): Po / Pi / Pf; not Rust
        // whitespace—mixed European, Greek, or Arabic/Latin HTML can glue tokens without ASCII space.
        for sep in [
            '\u{00A1}',
            '\u{00BF}',
            '\u{00AB}',
            '\u{00BB}',
            '\u{037E}',
            '\u{060C}',
            '\u{061B}',
            '\u{061F}',
            '\u{06D4}',
            '\u{066A}',
            '\u{066B}',
            '\u{066C}',
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
    fn khmer_inherent_vowel_format_separates_words() {
        // U+17B4/U+17B5 (Khmer vowel inherent AQ / AA) are Cf; Rust `split_whitespace()` does not
        // treat them as whitespace, so mixed Khmer/Latin or pasted layout text can glue tokens.
        for sep in ['\u{17B4}', '\u{17B5}'] {
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
    fn mongolian_punctuation_separates_words() {
        // U+1800–U+1805 (Mongolian/Manchu sentence punctuation, Po), U+1806 (TODO soft hyphen, Pd),
        // U+1807–U+180A (Sibe boundary / Manchu stops / NIRUGU, Po): not Rust whitespace, so
        // `hello᠁world`-style text stays one token without normalization.
        for sep in [
            '\u{1800}', '\u{1801}', '\u{1802}', '\u{1803}', '\u{1804}', '\u{1805}', '\u{1806}',
            '\u{1807}', '\u{1808}', '\u{1809}', '\u{180A}',
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
    fn invisible_fillers_separate_words() {
        // U+180B–U+180E Mongolian free variation selectors (Mn) and vowel separator (Cf);
        // U+115F/U+1160 Hangul choseong/jungseong fillers; U+3164 Hangul filler; U+FFA0 halfwidth
        // Hangul filler: not Unicode whitespace in Rust, so they glue tokens in `split_whitespace()`
        // without normalization. (U+1800–U+180A punctuation: `mongolian_punctuation_separates_words`.)
        for sep in [
            '\u{180B}', '\u{180C}', '\u{180D}', '\u{180E}', '\u{115F}', '\u{1160}', '\u{3164}',
            '\u{FFA0}',
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
    fn middle_dot_punctuation_separates_words() {
        // U+00B7 / U+0387 / U+30FB / U+FF65: middle-dot punctuation (Po) is not Rust whitespace.
        for sep in ['\u{00B7}', '\u{0387}', '\u{30FB}', '\u{FF65}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {:?} normalized before collapse, got {:?}",
                sep,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains {:?}",
                sep
            );
        }
    }

    #[test]
    fn supplemental_punctuation_u2e00_through_u2e5d_separate_words() {
        // U+2E00–U+2E5D: full assigned Supplemental Punctuation block (Po/Pd/Pi/Pf/Ps/Pe/Lm/So); not
        // Rust whitespace—critical edition / transcription HTML can glue Latin tokens for
        // `split_whitespace()` without normalization.
        for cp in 0x2E00u32..=0x2E5D {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                cp,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                cp
            );
        }
    }

    #[test]
    fn runic_word_punctuation_separates_words() {
        // U+16EB–U+16ED: Runic single / multiple / cross punctuation (Po); not Rust whitespace.
        for cp in 0x16EBu32..=0x16ED {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                cp,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                cp
            );
        }
    }

    #[test]
    fn aegean_and_phoenician_word_separator_marks_separate_words() {
        // U+10100 / U+10101 (Aegean word separator line/dot, Po) and U+1091F (Phoenician word
        // separator, Po) are not Rust whitespace.
        for sep in ['\u{10100}', '\u{10101}', '\u{1091F}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {:?} normalized before collapse, got {:?}",
                sep,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains {:?}",
                sep
            );
        }
    }

    #[test]
    fn bullet_and_dot_operators_separate_words() {
        // U+2022 (BULLET, Po), U+2219 (BULLET OPERATOR, Sm), U+22C5 (DOT OPERATOR, Sm): not Rust
        // whitespace; list or math-heavy HTML can glue Latin tokens for `split_whitespace()`.
        for sep in ['\u{2022}', '\u{2219}', '\u{22C5}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {:?} normalized before collapse, got {:?}",
                sep,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains {:?}",
                sep
            );
        }
    }

    #[test]
    fn general_punctuation_u2020_through_u2023_separate_words() {
        // U+2020 DAGGER, U+2021 DOUBLE DAGGER, U+2022 BULLET, U+2023 TRIANGULAR BULLET (all Po)—not
        // Rust whitespace; they fall between U+201F and U+2024, with U+2019 omitted as apostrophe.
        for sep in ['\u{2020}', '\u{2021}', '\u{2022}', '\u{2023}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                sep as u32
            );
        }
    }

    #[test]
    fn unicode_dash_punctuation_u2010_through_u2015_separate_words() {
        // U+2010 HYPHEN, U+2011 NON-BREAKING HYPHEN, U+2012 FIGURE DASH, U+2013 EN DASH, U+2014 EM
        // DASH, U+2015 HORIZONTAL BAR (all Pd)—not Rust whitespace. Em/en dashes in body copy can
        // sit between Latin tokens without ASCII space.
        for sep in [
            '\u{2010}',
            '\u{2011}',
            '\u{2012}',
            '\u{2013}',
            '\u{2014}',
            '\u{2015}',
        ] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "sep {:?} should separate words, got {:?}",
                sep,
                cleaned
            );
        }
    }

    #[test]
    fn general_punctuation_u2016_u2018_and_u201a_through_u201f_separate_words() {
        // U+2016 DOUBLE VERTICAL LINE, U+2017 DOUBLE LOW LINE, U+2018 LEFT SINGLE QUOTATION MARK,
        // U+201A–U+201F (low-9 / reversed-9 / double quotation marks)—Pi/Pf/Ps/Po; not Rust
        // whitespace. U+2019 is excluded (see `typographic_apostrophe_u2019_does_not_split_contractions`).
        for cp in (0x2016u32..=0x2018).chain(0x201Au32..=0x201F) {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "U+{:04X} should separate words, got {:?}",
                cp,
                cleaned
            );
        }
    }

    #[test]
    fn typographic_apostrophe_u2019_does_not_split_contractions() {
        // U+2019 is the usual curly apostrophe in "don't"; it must not become a word separator.
        let html = "<html><body><p>don\u{2019}t stop</p></body></html>";
        let cleaned = clean_html(html);
        assert!(
            cleaned.contains("don't stop") || cleaned.contains("don\u{2019}t stop"),
            "expected contraction preserved, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains("don t stop"),
            "U+2019 must not split contractions: {:?}",
            cleaned
        );
    }

    #[test]
    fn general_punctuation_u2030_through_u203b_separate_words() {
        // U+2030 PER MILLE through U+203B REFERENCE MARK (Po / Pi / Pf)—not Rust whitespace.
        // Primes, per-mille signs, guillemets, or reference marks in body copy can glue Latin tokens.
        for cp in 0x2030u32..=0x203B {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "U+{:04X} should separate words, got {:?}",
                cp,
                cleaned
            );
        }
    }

    #[test]
    fn general_punctuation_u203c_through_u205e_separate_words() {
        // U+203C DOUBLE EXCLAMATION through U+205E VERTICAL FOUR DOTS (Po/Pc/Ps/Pe/Sm)—not Rust
        // whitespace. U+205F MEDIUM MATHEMATICAL SPACE is Unicode whitespace (Rust splits on it).
        // Includes U+2044 FRACTION SLASH (previously a dedicated arm). Interrobang, undertie,
        // Tironian et, or four-dot punctuation in pasted HTML can glue Latin tokens.
        for cp in 0x203Cu32..=0x205E {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "U+{:04X} should separate words, got {:?}",
                cp,
                cleaned
            );
        }
    }

    #[test]
    fn cjk_symbols_brackets_ditto_wave_vertical_repeat_masu_half_fill_separate_words() {
        // U+3003 / U+3004 / U+3006; U+3008–U+301B; U+301C; U+301D–U+301F; U+3020; U+3031–U+3036;
        // U+303C; U+303E–U+303F: not Rust whitespace (see `collapse_whitespace` comment). Mixed
        // CJK-layout or romanized HTML can otherwise glue Latin tokens for `split_whitespace()`.
        for sep in (0x3003u32..=0x301B)
            .filter(|&cp| cp != 0x3005 && cp != 0x3007)
            .chain(0x301C..=0x3020)
            .chain(0x3031..=0x3036)
            .chain([0x303C, 0x303E, 0x303F])
            .filter_map(char::from_u32)
        {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                sep as u32
            );
        }
    }

    #[test]
    fn cjk_fullwidth_and_vertical_forms_punctuation_separate_words() {
        // U+3001/U+3002 (ideographic comma / full stop), U+FF0C/FF1A/FF1B/FF01/FF1F (fullwidth
        // comma, colon, semicolon, exclamation, question), U+FE10–U+FE19 (Vertical Forms
        // compatibility punctuation): Po/Ps/Pe/Pc—not Rust whitespace. Mixed CJK / Latin HTML
        // or vertical-layout compatibility text can sit between Latin tokens without ASCII space.
        for sep in [
            '\u{3001}',
            '\u{3002}',
            '\u{FF0C}',
            '\u{FF1A}',
            '\u{FF1B}',
            '\u{FF01}',
            '\u{FF1F}',
            '\u{FE10}',
            '\u{FE11}',
            '\u{FE12}',
            '\u{FE13}',
            '\u{FE14}',
            '\u{FE15}',
            '\u{FE16}',
            '\u{FE17}',
            '\u{FE18}',
            '\u{FE19}',
        ] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {:?} normalized before collapse, got {:?}",
                sep,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains {:?}",
                sep
            );
        }
    }

    #[test]
    fn small_form_variants_fe50_fe52_fe54_fe66_fe68_fe6b_separate_words() {
        // U+FE50–U+FE52, U+FE54–U+FE66, U+FE68–U+FE6B (Small Form Variants; skips unassigned
        // U+FE53, U+FE67, U+FE6C–U+FE6F)—not Rust whitespace; compatibility punctuation can glue
        // Latin tokens in pasted HTML.
        for cp in (0xFE50u32..=0xFE52)
            .chain(0xFE54..=0xFE66)
            .chain(0xFE68..=0xFE6B)
        {
            let sep = char::from_u32(cp).expect("valid BMP scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                cp,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                cp
            );
        }
    }

    #[test]
    fn cjk_compat_wavy_dash_telegraph_sep_pam_double_hyphen_fullwidth_low_line_separate_words() {
        // U+3030 WAVY DASH, U+3037 IDEOGRAPHIC TELEGRAPH LINE FEED SEPARATOR SYMBOL, U+303D PART
        // ALTERNATION MARK, U+30A0 KATAKANA-HIRAGANA DOUBLE HYPHEN, U+FF3F FULLWIDTH LOW LINE—not
        // Rust whitespace; CJK / romanization HTML can glue Latin tokens without ASCII space.
        for sep in ['\u{3030}', '\u{3037}', '\u{303D}', '\u{30A0}', '\u{FF3F}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                sep as u32
            );
        }
    }

    #[test]
    fn devanagari_danda_double_danda_separate_words() {
        // U+0964 (DEVANAGARI DANDA) and U+0965 (DEVANAGARI DOUBLE DANDA), both Po; not Rust
        // whitespace—mixed Latin–Devanagari or Unicode-sample HTML can glue tokens without ASCII space.
        for sep in ['\u{0964}', '\u{0965}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                sep as u32
            );
        }
    }

    #[test]
    fn thai_lao_myanmar_sentence_punctuation_separate_words() {
        // Thai: U+0E2F PAIYANNOI, U+0E4F FONGMAN, U+0E5A ANGKHANKHU, U+0E5B KHOMUT (Po). Lao: U+0EAF ELLIPSIS
        // (Po). Myanmar: U+104A LITTLE SECTION, U+104B SECTION (Po). None are Rust whitespace.
        for cp in [0x0E2Fu32, 0x0E4F, 0x0E5A, 0x0E5B, 0x0EAF].into_iter().chain(0x104A..=0x104B) {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                cp,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                cp
            );
        }
    }

    #[test]
    fn hebrew_sof_pasuq_georgian_paragraph_separator_separate_words() {
        // U+05C3 (Hebrew sof pasuq, Po) and U+10FB (Georgian paragraph separator, Po); not Rust
        // whitespace—RTL or mixed-script HTML can sit Latin tokens on either side without ASCII space.
        for sep in ['\u{05C3}', '\u{10FB}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                sep as u32
            );
        }
    }

    #[test]
    fn hebrew_maqaf_paseq_tibetan_tsheg_ethiopic_stop_separate_words() {
        // U+05BE (Hebrew maqaf, Pd), U+05C0 (Hebrew paseq, Po), U+0F0B (Tibetan tsheg, Po),
        // U+1362 (Ethiopic full stop, Po)—none are Rust whitespace; RTL or Ethiopic-layout HTML
        // can glue Latin tokens for `split_whitespace()` without ASCII space.
        for sep in ['\u{05BE}', '\u{05C0}', '\u{0F0B}', '\u{1362}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {:?} normalized before collapse, got {:?}",
                sep,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains {:?}",
                sep
            );
        }
    }

    #[test]
    fn khmer_sentence_signs_and_javanese_pada_separate_words() {
        // Khmer U+17D4 SIGN KHAN through U+17DA SIGN KOOMUUT (Po). Javanese U+A9C1 LEFT RERENGGAN through U+A9CF SECTION SIGN (Po).
        // U+A9C0 PANGKON (Cf) omitted. None of the mapped code points are Rust whitespace.
        let mut seps: Vec<char> = (0x17D4..=0x17DA).filter_map(char::from_u32).collect();
        seps.extend('\u{A9C1}'..='\u{A9CF}');
        for sep in seps {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                sep as u32
            );
        }
    }

    #[test]
    fn balinese_carik_through_pamengkeb_separate_words() {
        // U+1B5A CARIK SIKI through U+1B60 PAMENGKEB (Po)—sentence / section punctuation; not Rust whitespace.
        for sep in '\u{1B5A}'..='\u{1B60}' {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                sep as u32
            );
        }
    }

    #[test]
    fn batak_bindu_lepcha_punctuation_ol_chiki_mucaad_separate_words() {
        // Batak U+1BFC BINDU NA METEK through U+1BFF BINDU PANGOLAT (Po). Lepcha U+1C3B TA-ROL through U+1C3F TSHOOK (Po).
        // Ol Chiki U+1C7E MUCAAD and U+1C7F DOUBLE MUCAAD (Po). None are Rust whitespace.
        let mut seps: Vec<char> = ('\u{1BFC}'..='\u{1BFF}').collect();
        seps.extend('\u{1C3B}'..='\u{1C3F}');
        seps.extend('\u{1C7E}'..='\u{1C7F}');
        for sep in seps {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                sep as u32
            );
        }
    }

    #[test]
    fn cham_spiral_danda_through_triple_danda_separate_words() {
        // Cham U+AA5C PUNCTUATION SPIRAL through U+AA5F PUNCTUATION TRIPLE DANDA (Po). None are Rust whitespace.
        for sep in '\u{AA5C}'..='\u{AA5F}' {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                sep as u32
            );
        }
    }

    #[test]
    fn tai_viet_ho_hoi_koi_koi_separate_words() {
        // Tai Viet U+AADE SYMBOL HO HOI, U+AADF SYMBOL KOI KOI (Po). None are Rust whitespace.
        for sep in '\u{AADE}'..='\u{AADF}' {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                sep as u32
            );
        }
    }

    #[test]
    fn sinhala_limbu_meetei_sentence_punctuation_separate_words() {
        // U+0DF4 SINHALA PUNCTUATION KUNDDALIYA (Po). Limbu U+1940 SIGN TOKMA, U+1944 EXCLAMATION MARK, U+1945 QUESTION MARK (Po).
        // Meetei Mayek U+AAF0 CHEIKHEI, U+AAF1 AHANG KHUDA (Po). None are Rust whitespace.
        for sep in [
            '\u{0DF4}',
            '\u{1940}',
            '\u{1944}',
            '\u{1945}',
            '\u{AAF0}',
            '\u{AAF1}',
        ] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                sep as u32
            );
        }
    }

    #[test]
    fn tibetan_shad_head_marks_brackets_astro_separate_words() {
        // U+0F04–U+0F12 (yig mgo + tsheg + delimiter + shad stack), U+0F14, U+0F3A–U+0F3D, U+0F85,
        // U+0FD0–U+0FD4, U+0FD9–U+0FDA: Po/Ps/Pe, not Rust whitespace; U+0F13 (So) omitted.
        let mut seps: Vec<char> = (0x0F04..=0x0F12).filter_map(char::from_u32).collect();
        seps.push('\u{0F14}');
        seps.extend('\u{0F3A}'..='\u{0F3D}');
        seps.push('\u{0F85}');
        seps.extend((0x0FD0..=0x0FD4).filter_map(char::from_u32));
        seps.extend((0x0FD9..=0x0FDA).filter_map(char::from_u32));
        for sep in seps {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                sep as u32
            );
        }
    }

    #[test]
    fn fraction_division_and_fullwidth_solidus_separate_words() {
        // U+2044 (FRACTION SLASH) and U+2215 (DIVISION SLASH) are Sm; U+FF0F (FULLWIDTH SOLIDUS) is
        // Po—none are Rust whitespace; math or CJK-layout HTML can glue Latin tokens for
        // `split_whitespace()` without ASCII space.
        for sep in ['\u{2044}', '\u{2215}', '\u{FF0F}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {:?} normalized before collapse, got {:?}",
                sep,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains {:?}",
                sep
            );
        }
    }

    #[test]
    fn dot_leaders_ellipsis_and_hyphenation_point_separate_words() {
        // U+2024 (ONE DOT LEADER), U+2025 (TWO DOT LEADER), U+2026 (HORIZONTAL ELLIPSIS), U+2027
        // (HYPHENATION POINT): all Po, not Rust whitespace; TOC / UI typography can sit between
        // Latin tokens without ASCII space.
        for sep in ['\u{2024}', '\u{2025}', '\u{2026}', '\u{2027}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {:?} normalized before collapse, got {:?}",
                sep,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains {:?}",
                sep
            );
        }
    }

    #[test]
    fn ethiopic_wordspace_separates_words() {
        // U+1361 ETHIOPIC WORDSPACE is Po, not Rust whitespace; Ethiopic-layout HTML can sit between
        // Latin tokens without a real ASCII space.
        let html = "<html><body><p>hello\u{1361}world</p></body></html>";
        let cleaned = clean_html(html);
        assert!(
            cleaned.contains("hello world"),
            "expected Ethiopic wordspace normalized before collapse, got {:?}",
            cleaned
        );
        assert!(!cleaned.contains('\u{1361}'));
    }

    #[test]
    fn braille_pattern_blank_separates_words() {
        // U+2800 BRAILLE PATTERN BLANK is So, not Rust whitespace; used as a visible “empty” cell
        // and sometimes as a spacing hack in plain text.
        let html = "<html><body><p>hello\u{2800}world</p></body></html>";
        let cleaned = clean_html(html);
        assert!(
            cleaned.contains("hello world"),
            "expected Braille blank normalized before collapse, got {:?}",
            cleaned
        );
        assert!(!cleaned.contains('\u{2800}'));
    }

    #[test]
    fn egyptian_hieroglyph_format_controls_separate_words() {
        // U+13430–U+13455: Egyptian hieroglyph format controls (joiners, segment markers, extended
        // quadrat / area / insert controls) are Cf and not Rust whitespace, so mixed or
        // transliterated HTML can glue Latin tokens.
        for cp in 0x13430u32..=0x13455 {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                cp,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                cp
            );
        }
    }

    #[test]
    fn musical_symbol_format_controls_separate_words() {
        // U+1D173–U+1D17A: musical symbol begin/end beam, tie, slur, phrase (Cf) are not Rust
        // whitespace; MusicXML or similar text can insert them between letters.
        for cp in 0x1D173u32..=0x1D17A {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                cp,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                cp
            );
        }
    }

    #[test]
    fn duployan_selectors_and_shorthand_format_separate_words() {
        // U+1BC9D–U+1BC9E: Duployan thick letter selector / double mark (Mn). U+1BCA0–U+1BCA3:
        // shorthand format overlap / step (Cf). None are Rust whitespace. (U+1BC9F is Po—visible
        // punctuation; not mapped.)
        for cp in (0x1BC9Du32..=0x1BC9E).chain(0x1BCA0..=0x1BCA3) {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                cp,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                cp
            );
        }
    }

    #[test]
    fn brahmi_number_joiner_separates_words() {
        // U+1107F BRAHMI NUMBER JOINER is Mn, not Rust whitespace; Brahmi numeral HTML can glue
        // adjacent Latin tokens for `split_whitespace()`.
        let html = "<html><body><p>hello\u{1107F}world</p></body></html>";
        let cleaned = clean_html(html);
        assert!(
            cleaned.contains("hello world"),
            "expected Brahmi number joiner normalized before collapse, got {:?}",
            cleaned
        );
        assert!(!cleaned.contains('\u{1107F}'));
    }

    #[test]
    fn ideographic_description_characters_separate_words() {
        // U+2FF0–U+2FFB: ideographic description characters are So, not Rust whitespace; rare
        // mixed or pedagogical HTML can place them between Latin letters without an ASCII space.
        for cp in 0x2FF0u32..=0x2FFB {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                cp,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                cp
            );
        }
    }

    #[test]
    fn kaithi_number_format_signs_separate_words() {
        // U+110BD / U+110CD: Kaithi number sign and number sign above (Cf) are not Rust whitespace.
        for cp in [0x110BDu32, 0x110CD] {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized before collapse, got {:?}",
                cp,
                cleaned
            );
            assert!(
                !cleaned.contains(sep),
                "cleaned output still contains U+{:04X}",
                cp
            );
        }
    }

    #[test]
    fn unicode_language_tag_characters_separate_words() {
        // U+E0000–U+E007F (Tags block): deprecated language-tag / tag-id characters are Cf and
        // not Rust whitespace, so they would glue Latin tokens in `split_whitespace()`.
        for sep in ['\u{E0000}', '\u{E0001}', '\u{E0020}', '\u{E007F}'] {
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
