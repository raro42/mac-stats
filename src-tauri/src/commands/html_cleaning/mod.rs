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
                // markers, object replacement (U+FFF9–U+FFFC), replacement character (U+FFFD, So),
                // and deprecated Unicode language
                // tag characters (U+E0000–U+E007F) get the same treatment so FETCH_URL bodies
                // tokenize cleanly. NBSP (U+00A0, Zs) is also mapped to ASCII U+0020 so cleaned text
                // uses plain spaces even though Rust `split_whitespace` already treats it as a separator.
                // C1 control characters (U+0080–U+009F, Cc) are not Rust
                // whitespace; garbled 8-bit, Windows-1252 mis-decodes, or binary-pasted HTML can
                // inject them into text nodes and glue tokens for `split_whitespace()`.
                // C0 ASCII / ISO control characters U+0001–U+0008, U+000E–U+001F, and U+007F DELETE (Cc)
                // are not Unicode White_Space—Rust `char::is_whitespace` is false—so legacy device controls
                // or binary-pasted HTML can glue Latin tokens the same way C1 does. U+0000 NULL is omitted:
                // HTML5 tree construction drops NUL from text nodes before we see them (so `clean_html` cannot
                // exercise NUL end-to-end). TAB (U+0009), LF / VT / FF / CR (U+000A–U+000D) stay unmapped
                // here—they are White_Space and already tokenize with `split_whitespace()`.
                // Basic Latin SPACING CIRCUMFLEX ACCENT (U+005E) and SPACING GRAVE ACCENT (U+0060, both Sk) are not Rust
                // whitespace; ASCII markup, transliteration, or legacy typography in HTML can glue Latin tokens without ASCII
                // space (distinct from MODIFIER LETTER CIRCUMFLEX U+02C6 / CARON U+02C7, which are `Lm` and stay unmapped).
                // LOW LINE U+005F (`Pc`, underscore) stays unmapped—identifier / word-internal risk.
                // Khmer inherent vowels (U+17B4, U+17B5) are Cf and not Rust
                // whitespace, so Khmer-layout HTML can otherwise glue adjacent Latin tokens.
                // Latin-1 inverted exclamation / question (U+00A1, U+00BF) and double angle quotes
                // (U+00AB, U+00BB, Pi/Pf) are not Rust whitespace; Spanish / French typography in
                // HTML can place them between Latin tokens without ASCII space. Cent / pound /
                // generic currency / yen signs (U+00A2–U+00A5, all Sc) are not Rust whitespace;
                // legacy Western price copy or entity-encoded HTML can glue Latin tokens without ASCII
                // space (distinct from block Currency Symbols U+20A0+). BROKEN BAR (U+00A6, So) is not Rust
                // whitespace; legacy ISO 8859-1 / entity-encoded HTML often uses it as a tall pipe between
                // tokens (distinct from ASCII VERTICAL LINE U+007C). U+00A8 DIAERESIS (Sk) stays unmapped—
                // spacing-mark / combining-like risk. MACRON (U+00AF), ACUTE ACCENT (U+00B4), and CEDILLA
                // (U+00B8, all Sk) are not Rust whitespace; legacy ISO 8859-1 or linguistics typography in HTML
                // can sit them between Latin tokens without ASCII space (FEAT-D245). COPYRIGHT SIGN
                // (U+00A9, So) and REGISTERED SIGN (U+00AE, So) are not Rust whitespace; legal or trademark HTML
                // often places them between Latin tokens without ASCII space. DEGREE SIGN (U+00B0, So) is not Rust
                // whitespace; weather or scientific HTML can glue Latin tokens without ASCII space. Latin-1 math
                // symbols NOT SIGN (U+00AC), PLUS-MINUS SIGN (U+00B1), MULTIPLICATION SIGN (U+00D7), and DIVISION
                // SIGN (U+00F7) are Sm and not Rust whitespace; they sit outside the U+2200–U+22FF Mathematical
                // Operators arm. Vulgar fractions one quarter / one half / three quarters (U+00BC, U+00BD, U+00BE, No)
                // are not Rust whitespace; recipe, price, or legacy Latin-1 HTML can place them between Latin tokens
                // without ASCII space (same spirit as Number Forms U+2150+, FEAT-D146). Superscript digit numerics
                // (U+00B2, U+00B3, U+00B9, No) and MASCULINE ORDINAL INDICATOR (U+00BA, Ll) stay unmapped—numeric /
                // word-internal risk. Spacing Modifier Letters modifier letter plus / minus (U+02D6, U+02D7, Sk) are not
                // Rust whitespace; IPA, phonetic, or Unicode-sample HTML can sit them between Latin tokens without ASCII
                // space (same spirit as Latin-1 PLUS-MINUS U+00B1, Sm). Modifier-letter arrowheads U+02C2 LEFT through
                // U+02C5 DOWN ARROWHEAD (`Sk`), plus centred half-rings U+02D2 / U+02D3 and up/down tack U+02D4 / U+02D5 (`Sk`),
                // are not Rust whitespace; IPA or Unicode-sample HTML can sit them between Latin tokens without ASCII space.
                // U+02C6 MODIFIER LETTER CIRCUMFLEX ACCENT and U+02C7 CARON are `Lm` in UnicodeData (not `Sk`)—they stay
                // unmapped like stress / length marks. U+02C8 VERTICAL LINE through U+02D1 HALF TRIANGULAR COLON (`Lm`) stay
                // unmapped—word-internal risk. Neighboring Sk scalars U+02D8 BREVE through U+02DD
                // DOUBLE ACUTE ACCENT, U+02DE RHOTIC HOOK, and U+02DF CROSS ACCENT stay unmapped—legacy spacing-accent /
                // hook / cross-accent word-internal risk. Contiguous tone-bar / tone-mark / contour Sk subranges U+02E5
                // EXTRA-HIGH TONE BAR through U+02EB YANG DEPARTING TONE MARK, U+02ED UNASPIRATED, and U+02EF LOW DOWN
                // ARROWHEAD through U+02FF LOW LEFT ARROW are not Rust whitespace; IPA / Chin Wu / Unicode-sample HTML
                // can sit them between Latin tokens without ASCII space. U+02EC MODIFIER LETTER VOICING (Lm) and U+02EE
                // MODIFIER LETTER DOUBLE APOSTROPHE (Lm) stay unmapped—letter- / apostrophe-like, word-internal risk.
                // Superscript letters U+02E0 SMALL GAMMA through U+02E4 REVERSED GLOTTAL STOP (Lm) stay unmapped.
                // Modifier Tone Letters (Unicode block U+A700–U+A71F): seventeen contiguous spacing `Sk` scalars U+A700
                // MODIFIER LETTER CHINESE TONE YIN PING through U+A716 MODIFIER LETTER EXTRA-LOW LEFT-STEM TONE BAR—are
                // not Rust whitespace; Chinese tone / IPA-style tone-bar notation or Unicode-sample HTML can sit them between
                // Latin tokens without ASCII space. Tail U+A717 MODIFIER LETTER DOT VERTICAL BAR through U+A71F MODIFIER LETTER
                // LOW INVERTED EXCLAMATION MARK are `Lm`—stay unmapped (word-internal risk).
                // Latin Extended-D (Unicode block U+A720–U+A7FF): four spacing `Sk` scalars—MODIFIER LETTER STRESS AND HIGH
                // TONE (U+A720), STRESS AND LOW TONE (U+A721), COLON (U+A789), SHORT EQUALS SIGN (U+A78A)—are not Rust
                // whitespace; phonetic or Unicode-sample HTML can sit them between Latin tokens without ASCII space. All
                // other assigned code points in the block are letters (Lu/Ll) or modifier letters (Lm)—excluded (word-internal risk).
                // Latin Extended-E (Unicode block U+AB30–U+AB6F): three spacing `Sk` scalars—MODIFIER BREVE WITH INVERTED BREVE
                // (U+AB5B), MODIFIER LETTER LEFT TACK (U+AB6A), MODIFIER LETTER RIGHT TACK (U+AB6B)—are not Rust whitespace;
                // phonetic or Unicode-sample HTML can sit them between Latin tokens without ASCII space. Neighbors such as
                // U+AB5A LATIN SMALL LETTER Y WITH SHORT RIGHT LEG (`Ll`) and U+AB5C MODIFIER LETTER SMALL HENG (`Lm`) stay
                // unmapped—word-internal risk. Tail U+AB6C–U+AB6F are unassigned (`Cn`).
                // Section sign (U+00A7)
                // and pilcrow (U+00B6, Po) are not Rust whitespace either; legal or editorial HTML
                // often uses them between Latin tokens without ASCII space. Greek question mark
                // (U+037E, Po; erotimatiko) is not Rust whitespace and is distinct from Greek ano
                // teleia (U+0387) mapped with middle-dot punctuation. Greek spacing tonos (U+0384, Sk)
                // and spacing dialytika tonos (U+0385, Sk) are not Rust whitespace; polytonic Greek or
                // mixed-script HTML can sit them between Latin tokens without ASCII space. Greek Extended
                // (Unicode block U+1F00–U+1FFF): fifteen spacing polytonic `Sk` scalars—U+1FBD KORONIS,
                // U+1FBF PSILI, U+1FC0 PERISPOMENI, U+1FC1 DIALYTIKA AND PERISPOMENI, U+1FCD–U+1FCF (psili
                // with varia/oxia/perispomeni), U+1FDD–U+1FDF (dasia with varia/oxia/perispomeni),
                // U+1FED–U+1FEF (dialytika with varia/oxia, VARIA), U+1FFD OXIA, U+1FFE DASIA—are not Rust
                // whitespace; polytonic Greek or Unicode-sample HTML can sit them between Latin tokens without
                // ASCII space (extends FEAT-D235). Precomposed Greek letters with diacritics (`Ll`/`Lu`/`Lt`),
                // U+1FBE PROSGEGRAMMENI (`Ll`), and other block scalars stay unmapped—word-internal risk.
                // U+0374 GREEK NUMERAL SIGN (Lm) and U+0375 GREEK LOWER NUMERAL SIGN (Sk) stay unmapped—Greek
                // thousands / numeral notation, word-internal risk. Greek pi symbol (U+03D6, Sm) and
                // reversed lunate epsilon symbol (U+03F6, Sm) are not Rust whitespace; MathML or Greek
                // math typography can sit them between Latin tokens without ASCII space (outside the
                // U+2200–U+22FF Mathematical Operators arm). U+03D5 GREEK PHI SYMBOL (Lu) and U+03F5
                // GREEK LUNATE EPSILON SYMBOL (Ll) stay unmapped—letter-like, word-internal risk. Arabic comma / semicolon / end of text mark (U+061D, Po) /
                // question / full stop (U+060C, U+061B, U+061F, U+06D4, Po) and Arabic percent /
                // decimal / thousands / five-pointed-star (U+066A–U+066D, Po) are not Rust whitespace; RTL or
                // bilingual numeric HTML can glue Latin tokens for `split_whitespace()`. Arabic-indic per mille /
                // per ten thousand (U+0609, U+060A, Po), Afghani sign (U+060B, Sc), date separator (U+060D, Po),
                // poetic verse sign / misra (U+060E, U+060F, So), and triple-dot punctuation (U+061E, Po) are not Rust
                // whitespace either; RTL financial, Quranic-notation, or editorial HTML can glue Latin tokens without ASCII
                // space. Arabic-indic cube / fourth root and ray (U+0606–U+0608, Sm) stay unmapped (math-internal risk);
                // honorific and combining marks U+0610+ (Mn) stay unmapped. Arabic Extended-B raised round dot
                // (U+0888, Sk) is not Rust whitespace; Arabic typography or Unicode-sample HTML can sit it between
                // Latin tokens without ASCII space. U+08C9 ARABIC SMALL FARSI YEH (`Lm`) stays unmapped—modifier-like,
                // word-internal risk.
                // Arabic Presentation Forms-A: honorific / salutation ligatures (U+FBC3–U+FBD2, So) before the ornate-
                // paren scalars; not Rust whitespace. Ornate left / right parentheses (U+FD3E, U+FD3F, Pe/Ps), RIAL
                // SIGN (U+FDFC, Sc), BISMILLAH ligature (U+FDFD, So), and further honorific ligatures (U+FD40–U+FD4F,
                // U+FDCF, U+FDFE, U+FDFF, So) are not Rust whitespace; Persian / Arabic typography or Unicode-sample HTML
                // can place them between Latin tokens without ASCII space. Letter ligatures U+FD50+ (Lo), Koranic stop
                // signs U+FDF0–U+FDFB (Lo), and Quranic annotation dot symbols U+FBB2–U+FBC1 (Sk) plus U+FBC2 WASLA
                // ABOVE (Sk) stay unmapped—word-internal or modifier-like risk (same spirit as omitting many combining marks).
                // Armenian exclamation / comma / question / abbreviation mark (U+055C–U+055F, Po) and
                // full stop / hyphen (U+0589–U+058A, Po/Pd) are not Rust whitespace; bilingual or
                // Unicode-sample HTML can glue Latin tokens. U+055A (ARMENIAN APOSTROPHE) and U+055B
                // (EMPHASIS MARK, Po) are omitted—apostrophe- or stress-like marks can sit word-internally
                // in Armenian (same spirit as omitting U+2019 for Latin contractions).
                // Armenian right- / left-facing eternity signs (U+058D, U+058E, So) and dram sign (U+058F, Sc) are not
                // Rust whitespace; Armenian–Latin or Unicode-sample HTML can place them between Latin tokens without ASCII
                // space. Unassigned U+058B–U+058C (Cn) are excluded—only U+058D..=U+058F are mapped.
                // Devanagari danda / double danda (U+0964, U+0965, Po) and abbreviation sign (U+0970, Po) are not Rust
                // whitespace; mixed Latin–Devanagari or Unicode-sample HTML can place them between Latin tokens without ASCII
                // space. Devanagari digits U+0966–U+096F (Nd) and high spacing dot (U+0971, Lm) stay unmapped—numeric /
                // modifier-like, word-internal risk.
                // Vedic Extensions U+1CD3 VEDIC SIGN NIHSHVASA (Po) is not Rust whitespace; transliterated Vedic
                // or Unicode-sample HTML can glue Latin tokens without ASCII space. Vedic tone marks U+1CD0–U+1CD2,
                // U+1CD4–U+1CDF (Mn) stay unmapped—combining / word-internal risk.
                // Gurmukhi abbreviation sign (U+0A76, Po), Gujarati abbreviation sign (U+0AF0, Po), Gujarati rupee
                // sign (U+0AF1, Sc), Telugu sign siddham (U+0C77, Po), SIGN TUUMU (U+0C7F, So), and Kannada sign
                // siddham (U+0C84, Po) are not Rust whitespace; South Indic scholarly or Unicode-sample HTML can glue
                // Latin tokens without ASCII space. Telugu digits U+0C66–U+0C6F (Nd) and letters (Lo) stay unmapped—word-internal risk.
                // Bengali rupee mark / rupee sign / ganda mark (U+09F2, U+09F3, U+09FB, Sc) are not Rust whitespace;
                // South Asian price copy or Unicode-sample HTML can place them between Latin tokens without ASCII space.
                // Bengali currency numerators U+09F4–U+09F9 (No) and letter khanda ta (U+09FC, Lo) stay unmapped—numeric /
                // letter-internal risk. Iss-har (U+09FA, So) and abbreviation sign (U+09FD, Po), Oriya isshar (U+0B70, So), Tamil
                // day through number signs (U+0BF3–U+0BFA, So/Sc), and Malayalam sign para / date mark (U+0D4F,
                // U+0D79, So) are not Rust whitespace; Eastern / Tamil / Malayalam mixed-script or Unicode-sample HTML
                // can glue Latin tokens without ASCII space.
                // Thai BAHT SIGN (U+0E3F, Sc), PAIYANNOI / FONGMAN / ANGKHANKHU / KHOMUT (U+0E2F, U+0E4F, U+0E5A, U+0E5B, Po), Lao
                // ELLIPSIS (U+0EAF, Po) and LAO YAMAK (U+0EDF, Po), Myanmar LITTLE SECTION / SECTION (U+104A–U+104B, Po), and Myanmar symbols
                // LOCATIVE / EXCLAMATION / COMPLETED / AFOREMENTIONED (U+104C–U+104F, So), plus block-tail SHAN ONE / SHAN EXCLAMATION
                // (U+109E–U+109F, So), are not Rust whitespace;
                // Southeast Asian–Latin bilingual or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Lao LETTER KHMU GO (U+0EDE, Lo) and Lao digits U+0ED0–U+0ED9 (Nd) stay unmapped—letter- / numeric-internal risk.
                // Thai MAIYAMOK (U+0E46, Lm) stays unmapped—modifier-like, can repeat word-internally.
                // Arabic number signs / ayah markers (U+0600–U+0605, U+06DD, U+08E2), Arabic
                // Extended-A currency format marks (U+0890–U+0891, pound/piastre mark above), Syriac
                // end of paragraph (U+0700, Bk) and sentence punctuation (U+0701–U+070D, Po), and
                // Syriac abbreviation mark (U+070F) are not Rust whitespace (070F is Cf); RTL scholarly
                // or financial HTML can place them between scripts without a real space. NKo SYMBOL OO
                // DENNEN (U+07F6, So), GBAKURUNEN symbol / comma / exclamation (U+07F7–U+07F9, Po), and
                // DOROME / TAMAN SIGNS (U+07FE–U+07FF, Sc) are not Rust whitespace; U+07FA LAJANYALAN (Lm),
                // tone apostrophes U+07F4–U+07F5 (Lm), combining marks U+07F0–U+07F3 / U+07FD (Mn), and
                // unassigned U+07FB–U+07FC omitted. Samaritan punctuation
                // marks (U+0830–U+083E, Po) are not Rust whitespace; Hebrew/Samaritan scholarly or
                // Unicode-sample HTML can glue Latin tokens without ASCII space. Mandaic PUNCTUATION (U+085E, Po) is not Rust
                // whitespace; Mandaic–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space. Mandaic
                // affrication / vocalization / gemination marks (U+0859–U+085B, Mn) stay unmapped—combining /
                // word-internal risk. Syriac Supplement letters U+0860–U+086A (Lo) stay unmapped.
                // Nyiakeng Puachue
                // Hmong exclamation / question (U+16FE2, U+16FE3, Po) are not Rust whitespace;
                // U+16FE0–U+16FE1 (Lo/Lm) and U+16FE4 LOGOGRAM NYIAM (Lo) stay unmapped. Nyiakeng
                // Puachue Hmong block tail CIRCLED CA (U+1E14F, So) is not Rust whitespace; Unicode-
                // sample or Hmong–Latin HTML can glue Latin tokens without ASCII space. Digits
                // U+1E140–U+1E149 (Nd) and category signs / syllable lengthener Lm U+1E137–U+1E13D stay
                // unmapped—numeric / word-internal risk. Wancho NGUN SIGN (U+1E2FF, Sc) is not Rust
                // whitespace; Northeast-Indian script or Unicode-sample HTML can glue Latin tokens without ASCII
                // space. Wancho digits U+1E2F0–U+1E2F9 (Nd) and gap U+1E2FA–U+1E2FE (unassigned `Cn`) stay
                // unmapped—numeric / non-character risk (FEAT-D214; historical FEAT-D91 Wancho “comma / full stop Po”
                // arm). Adlam
                // initial exclamation / question (U+1E95E, U+1E95F, Po) are not Rust whitespace;
                // Adlam–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Medefaidrin comma / full stop / symbol aiva / exclamation oh (U+16E97–U+16E9A, Po) are
                // not Rust whitespace; Medefaidrin–Latin or Unicode-sample HTML can glue Latin tokens
                // without ASCII space. Medefaidrin letters (Ll/Lu) stay unmapped—word-internal risk. Mongolian
                // U+1800–U+180E (BIRGA through vowel separator) are Po/Pd/Mn/Cf and not Rust
                // whitespace—sentence punctuation (U+1800–U+1805, U+1807–U+180A), MONGOLIAN TODO SOFT HYPHEN
                // (U+1806, Pd; Unicode name contains the substring "TODO", not a source-code TODO), free variation
                // selectors (U+180B–U+180D, Mn), vowel separator
                // (U+180E, Cf)—so mixed or pasted Mongolian/Manchu HTML can glue Latin tokens.
                // Middle dot (U+00B7, Po), Greek ano teleia (U+0387, Po), Katakana middle dot
                // (U+30FB, Po), and halfwidth Katakana middle dot (U+FF65, Po) are not Rust
                // whitespace; European / Greek / Japanese typography often uses them as word
                // separators, so pasted HTML can glue Latin tokens for `split_whitespace()`.
                // Katakana-Hiragana voiced sound mark (U+309B, Sk) and semi-voiced sound mark (U+309C, Sk)
                // are not Rust whitespace; romaji–kana or Unicode-sample HTML can place ﾞ/ﾟ between Latin
                // tokens without ASCII space. Hiragana / Katakana letters (Lo) stay unmapped.
                // Dagger / double dagger / bullet / triangular bullet (U+2020–U+2023, Po) sit
                // between the curly-quote arm and dot-leader arm; U+2019 between them is omitted
                // (apostrophe). None are Rust whitespace; footnote-style or list HTML can glue
                // Latin tokens without ASCII space. Mathematical Operators U+2200–U+22FF (mostly Sm;
                // some Mn / Me) are not Rust whitespace; MathML can place ∀, ∈, integrals, bullet
                // operator U+2219, dot operator U+22C5, division slash U+2215, etc. between Latin
                // tokens without ASCII space (one contiguous range arm). Miscellaneous Technical
                // U+2300–U+23FF (mostly So / Sm / Po) are not Rust whitespace; electrotechnical symbols,
                // APL operators, dice faces, or Unicode-sample HTML can sit between Latin tokens without
                // ASCII space (contiguous range arm). Control Pictures U+2400–U+243F (all So) are not Rust
                // whitespace; terminal / legacy or Unicode-sample HTML can place control-picture glyphs between
                // Latin tokens without ASCII space (contiguous range arm). U+2440+ Optical Character Recognition
                // is a separate arm (FEAT-D130). Fraction slash (U+2044, Sm)
                // and fullwidth solidus (U+FF0F, Po) are not Rust whitespace either; math fractions,
                // MathML, or CJK fullwidth paths can use them between Latin tokens without ASCII space.
                // IDEOGRAPHIC SPACE (U+3000, Zs) is Unicode separator space and Rust `char::is_whitespace`—it
                // is not listed on this match arm; the per-line `split_whitespace()` pass already splits on it
                // (same spirit as TAB / LF–CR staying unmapped at the top of this function). NBSP (U+00A0, Zs)
                // is still mapped to ASCII U+0020 so cleaned FETCH_URL text is plain-space–only even when a
                // scalar would otherwise survive inside a segment; U+3000 between Latin tokens still yields
                // ASCII spaces after split+join. IDEOGRAPHIC COMMA (U+3001, Po) and IDEOGRAPHIC FULL STOP (U+3002, Po; colloquially
                // “ideographic comma / full stop” for both) and
                // fullwidth ASCII-like punctuation (U+FF0C comma, U+FF1A colon, U+FF1B semicolon,
                // U+FF01 exclamation, U+FF02 quotation, U+FF1F question; all Po) are not Rust whitespace; CJK or
                // mixed-layout HTML often places them between Latin tokens without ASCII space.
                // Other Halfwidth and Fullwidth Forms delimiters—U+FF03 number sign, U+FF04 dollar,
                // U+FF05 percent, U+FF06 ampersand, U+FF08/U+FF09 parens, U+FF0A asterisk, U+FF0B plus,
                // U+FF0D hyphen-minus, U+FF0E full stop, U+FF1C–U+FF1E relations, U+FF20 commercial at,
                // U+FF3B–U+FF3D brackets and reverse solidus, U+FF5B–U+FF5D braces, U+FF5C vertical line
                // (Po/Ps/Pe/Sm/Pd/Sc as assigned)—are not Rust whitespace either. U+FF07 FULLWIDTH
                // APOSTROPHE (Po) stays unmapped—typographic apostrophe risk. U+FF5E FULLWIDTH TILDE (Sm)
                // stays unmapped—Japanese range notation (e.g. 3～5). U+FF3E FULLWIDTH CIRCUMFLEX ACCENT (Sk)
                // is not Rust whitespace—fullwidth typography can glue Latin tokens like ASCII U+005E (FEAT-D247);
                // maps to ASCII space (FEAT-D249). U+FF40 FULLWIDTH GRAVE ACCENT (Sk) stays unmapped—modifier-like,
                // word-internal risk (FEAT-D125). Fullwidth white parentheses (U+FF5F, U+FF60,
                // Ps/Pe), halfwidth ideographic full stop / corner brackets / ideographic comma
                // (U+FF61–U+FF64, Po/Ps/Pe; U+FF65 middle dot is on the middle-dot arm), and fullwidth
                // cent / pound / not sign / broken bar / yen / won (U+FFE0–U+FFE2, U+FFE4–U+FFE6,
                // Sc/Sm/So) are not Rust whitespace either. U+FFE3 FULLWIDTH MACRON (Sk) stays
                // unmapped—overline-like, word-internal risk. Halfwidth forms light vertical (U+FFE8,
                // So), halfwidth arrows (U+FFE9–U+FFEC, Sm), and halfwidth black square / white circle
                // (U+FFED–U+FFEE, So), and halfwidth black small square (U+FFEF, So) are not Rust
                // whitespace either; JIS / compat HTML can glue Latin tokens without ASCII space.
                // U+FFFD REPLACEMENT CHARACTER (So, same block) is not Rust whitespace either;
                // transcoding or mojibake HTML can insert it between Latin tokens without ASCII space.
                // CJK Symbols and Punctuation: DITTO MARK (U+3003, Po), JAPANESE INDUSTRIAL STANDARD SYMBOL
                // (U+3004, So; colloquially “JIS symbol”),
                // IDEOGRAPHIC CLOSING MARK (U+3006, Lo in UnicodeData—not `Po`; FEAT-D257 misread the
                // category), CJK brackets and postal/geta marks (official `UnicodeData.txt` names):
                // LEFT ANGLE BRACKET (U+3008, Ps) / RIGHT ANGLE BRACKET (U+3009, Pe) / LEFT DOUBLE ANGLE
                // BRACKET (U+300A, Ps) / RIGHT DOUBLE ANGLE BRACKET (U+300B, Pe) / LEFT CORNER BRACKET
                // (U+300C, Ps) / RIGHT CORNER BRACKET (U+300D, Pe) / LEFT WHITE CORNER BRACKET (U+300E, Ps) /
                // RIGHT WHITE CORNER BRACKET (U+300F, Pe) / LEFT BLACK LENTICULAR BRACKET (U+3010, Ps) /
                // RIGHT BLACK LENTICULAR BRACKET (U+3011, Pe), POSTAL MARK (U+3012, So), GETA MARK (U+3013, So),
                // LEFT TORTOISE SHELL BRACKET (U+3014, Ps) / RIGHT TORTOISE SHELL BRACKET (U+3015, Pe) /
                // LEFT WHITE LENTICULAR BRACKET (U+3016, Ps) / RIGHT WHITE LENTICULAR BRACKET (U+3017, Pe) /
                // LEFT WHITE TORTOISE SHELL BRACKET (U+3018, Ps) / RIGHT WHITE TORTOISE SHELL BRACKET (U+3019, Pe) /
                // LEFT WHITE SQUARE BRACKET (U+301A, Ps) / RIGHT WHITE SQUARE BRACKET (U+301B, Pe). WAVE DASH (U+301C, Pd), reversed/double-prime quotation
                // marks—REVERSED DOUBLE PRIME QUOTATION MARK (U+301D, Ps), DOUBLE PRIME QUOTATION MARK and LOW DOUBLE PRIME QUOTATION MARK (U+301E–U+301F, Pe;
                // both `Pe` in UnicodeData, not a Ps/Pe pair by adjacent code points), POSTAL MARK FACE (U+3020, So), vertical kana repeat marks
                // (U+3031–U+3035, Lm), CIRCLED POSTAL MARK (U+3036, So; same contiguous arm), MASU MARK
                // (U+303C, So), IDEOGRAPHIC VARIATION INDICATOR /
                // IDEOGRAPHIC HALF FILL SPACE (U+303E–U+303F, So) are not Rust whitespace. Omitted on purpose:
                // IDEOGRAPHIC ITERATION MARK (U+3005, Lm) and VERTICAL IDEOGRAPHIC ITERATION MARK (U+303B, Lm),
                // IDEOGRAPHIC NUMBER ZERO (U+3007, Nl; colloquially
                // “ideographic zero”) and Hangzhou numerals
                // U+3021–U+3029 / U+3038–U+303A (Nl), and ideographic tone marks U+302A–U+302F (Mn/Mc).
                // Vertical Forms compatibility punctuation (U+FE10–U+FE19, Po/Ps/Pe/Pc) is not
                // Rust whitespace. CJK Compatibility Forms U+FE30–U+FE4F (vertical presentation for
                // two-dot leader / dashes / underscores / brackets / sesame dots / dashed–wavy overlines
                // and low lines; Po / Pd / Ps / Pe / Pc as assigned) are not Rust whitespace; vertical
                // CJK-layout or compatibility HTML can glue Latin tokens without ASCII space. Combining
                // Half Marks U+FE20–U+FE2F (Mn) stay unmapped—word-internal combining risk. Small Form Variants (U+FE50–U+FE52, U+FE54–U+FE66, U+FE68–U+FE6B;
                // Po / Pd / Ps / Pe / Sm / Sc as assigned—skips unassigned U+FE53, U+FE67, U+FE6C–U+FE6F)
                // are not Rust whitespace; compatibility typography HTML can glue Latin tokens without
                // ASCII space. WAVY DASH (U+3030, Pd), IDEOGRAPHIC TELEGRAPH LINE FEED SEPARATOR SYMBOL
                // (U+3037, So), PART ALTERNATION MARK (U+303D, Po), KATAKANA-HIRAGANA DOUBLE HYPHEN
                // (U+30A0, Pd), and FULLWIDTH LOW LINE (U+FF3F, Pc) are not Rust whitespace either;
                // mixed CJK / romanization HTML can do the same. CJK Radicals Supplement (Unicode block U+2E80–U+2EFF):
                // assigned So U+2E80–U+2E99 and U+2E9B–U+2EF3; not Rust whitespace. Unassigned U+2E9A and tail U+2EF4–U+2EFF
                // (Cn) stay unmapped. Kangxi Radicals (Unicode block U+2F00–U+2FDF): assigned So U+2F00–U+2FD5 (214 radicals);
                // not Rust whitespace. Unassigned tail U+2FD6–U+2FDF (Cn) and inter-block gap U+2FE0–U+2FEF (Cn) stay unmapped.
                // Ideographic Description Characters U+2FF0–U+2FFF (all assigned scalars So, including overlap/surround/overlay/rotation U+2FFC–U+2FFF);
                // not Rust whitespace. Unicode 17 adds IDEOGRAPHIC DESCRIPTION CHARACTER SUBTRACTION U+31EF (`So`)—mapped on the
                // same arm as U+2FF0–U+2FFF. Kanbun (Unicode block U+3190–U+319F): linking and reverse
                // marks U+3190–U+3191 (So) and top/earth/man annotation marks U+3196–U+319F (So) are not Rust whitespace;
                // classical Japanese kanbun or Unicode-sample HTML can glue Latin tokens without ASCII space. Ideographic
                // annotation digit marks U+3192–U+3195 (No) stay unmapped—numeric risk. Bopomofo Extended U+31A0–U+31BF (Lo)
                // stays unmapped. CJK Strokes U+31C0–U+31E5 (all So as assigned in Unicode 17): not Rust whitespace; stroke nomenclature or
                // font-chart HTML can sit between Latin tokens without ASCII space. Inner gap U+31E6–U+31EE unassigned (`Cn`)—excluded.
                // Katakana Phonetic Extensions U+31F0–U+31FF (Lo)—excluded. Enclosed CJK Letters and Months (Unicode block
                // U+3200–U+32FF): assigned So parenthesized / circled Hangul, parenthesized / circled CJK ideograph labels,
                // ideographic telegraph month symbols, circled katakana, squared Latin abbreviations (e.g. PTE, Hz, eV), era
                // name square, etc.; not Rust whitespace. All No scalars stay unmapped: parenthesized ideograph one–ten
                // (U+3220–U+3229), circled numbers on black square (U+3248–U+324F), circled Latin digit pairs / decades
                // (U+3251–U+325F, U+32B1–U+32BF), circled ideograph one–ten (U+3280–U+3289). Unassigned U+321F (Cn) excluded.
                // CJK Compatibility (Unicode block U+3300–U+33FF): all 256 assigned scalars are So (squared katakana words,
                // telegraph-era labels, etc.); not Rust whitespace. JIS / carrier or Unicode-sample HTML can place them between
                // Latin tokens without ASCII space. CJK Unified Ideographs Extension A U+3400+ (Lo) immediately after—excluded.
                // Hebrew maqaf (U+05BE, Pd), paseq (U+05C0, Po), sof pasuq (U+05C3, Po; sentence end like a colon), and
                // nun hafukha (U+05C6, Po) are not Rust whitespace. Alphabetic Presentation Forms U+FB29 HEBREW LETTER
                // ALTERNATIVE PLUS SIGN (`Sm`) is not Rust whitespace—Hebrew typography or Unicode-sample HTML can sit it
                // between Latin tokens without ASCII space. U+05F3 GERESH and U+05F4 GERSHAYIM (Po) stay unmapped—
                // apostrophe- / abbreviation-like, word-internal risk (same spirit as U+2019). Georgian paragraph separator (U+10FB, Po) is
                // not Rust whitespace; mixed Latin–Georgian or Unicode-sample HTML can glue tokens without ASCII space. Tibetan yig mgo
                // and shad marks (U+0F04–U+0F12, Po), gter tsheg (U+0F14, Po), corner brackets (U+0F3A–U+0F3D, Ps/Pe), paluta (U+0F85,
                // Po), and astrological / editorial marks (U+0FD0–U+0FD4, U+0FD9–U+0FDA, Po) are not Rust whitespace—only intersyllabic
                // tsheg (U+0F0B) was covered before; mixed Tibetan–Latin or Unicode-sample HTML can otherwise glue Latin tokens. U+0F13
                // (caret So) stays unmapped. Sinhala kunddaliya (U+0DF4, Po), Limbu tokma / exclamation / question (U+1940, U+1944,
                // U+1945, Po), Meetei Mayek Extensions cheikhan / ahang khuda (U+AAF0–U+AAF1, Po), and Meetei Mayek cheikhei (U+ABEB, Po) are not
                // Rust whitespace; U+AAF3–U+AAF4 (Meetei Mayek syllable heavy / half tone, Lm) and U+AAF5–U+AAF6 (Mc/Mn) stay
                // unmapped—modifier-like, word-internal risk. U+AAF2 MEETEI MAYEK ANJI (`Lo`) stays unmapped—letter-like; historical FEAT-D218 treated it as `Po`
                // (FEAT-D221). Mixed-script or Unicode-sample HTML can glue Latin tokens without ASCII space. Ethiopic section
                // mark, wordspace, full stop, comma, semicolon, colon, preface colon, question mark, and paragraph separator
                // (U+1360–U+1368, Po) are not Rust whitespace; mixed Ethiopic–Latin or Unicode-sample HTML can glue Latin tokens.
                // Ethiopic digit numerics U+1369+ (`No`) stay unmapped—numeric / word-internal risk. Khmer signs khan through koomuut
                // (U+17D4–U+17DA, Po) and KHMER CURRENCY SYMBOL RIEL (U+17DB, Sc) are not Rust whitespace;
                // Khmer Symbols PATHAMASAT through DAP-PRAM ROC (U+19E0–U+19FF, So) are not Rust whitespace;
                // Khmer digits U+17E0–U+17E9 (Nd) and LEK ATTAK numerics U+17F0–U+17F9 (No) stay unmapped—numeric-internal risk.
                // Javanese pada / section marks (U+A9C1–U+A9CF, Po) and tail marks PADA TIRTA TUMETES / PADA ISEN-ISEN
                // (U+A9DE–U+A9DF, Po) are not Rust whitespace—digit numerics U+A9D0–U+A9D9 (Nd) sit between those Po runs and stay
                // unmapped. U+A9C0 PANGKON (Cf, virama) stays unmapped as
                // word-internal risk alongside U+17B4/U+17B5 inherent vowels already mapped as format controls. Khmer SIGN
                // AVAKRAHASANYA (U+17DC, Lo) and SIGN ATTHACAN (U+17DD, Mn) stay unmapped—letter- / combining-internal risk.
                // Balinese inverted carik (U+1B4E–U+1B4F, Po), carik through pamengkeb (U+1B5A–U+1B60, Po), musical pitch
                // names dong through dang gede (U+1B61–U+1B6A, So), hand / drum notation (U+1B74–U+1B7C, So), and lantang / bawak
                // punctuation (U+1B7D–U+1B7F, Po) are not Rust whitespace; Balinese–Latin or Unicode-sample HTML can glue Latin
                // tokens without ASCII space. Digits U+1B50–U+1B59 (Nd) and combining musical marks U+1B6B–U+1B73 (Mn) stay unmapped—
                // numeric / word-internal risk (same spirit as Western Musical Symbols Mc/Mn in FEAT-D162).
                // Batak bindu symbols (U+1BFC–U+1BFF, Po), Lepcha punctuation ta-rol through tshook (U+1C3B–U+1C3F, Po), and Ol Chiki mucaad /
                // double mucaad (U+1C7E–U+1C7F, Po) are not Rust whitespace; Batak–Latin, Lepcha–Latin, Ol Chiki–Latin, or Unicode-sample HTML
                // can glue Latin tokens without ASCII space. Cham spiral and danda marks (U+AA5C–U+AA5F, Po) are not Rust whitespace;
                // Cham–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space. Tai Viet HO HOI / KOI KOI
                // (U+AADE–U+AADF, Po) are not Rust whitespace; Tai Viet–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // U+AADB KON and U+AADC NUENG are Lo; U+AADD SAM is Lm—omitted (word-internal risk, like Thai U+0E46).
                // Philippine single / double punctuation (U+1735–U+1736, Po; Buhid / Tagbanwa) are not Rust whitespace;
                // Unicode-sample or Philippine-script HTML can glue Latin tokens without ASCII space. U+1734 HANUNOO
                // SIGN PAMUDPOD is `Mn` in UnicodeData—stays unmapped (combining / word-internal risk; FEAT-D83 assumed
                // a contiguous Po triple with 1734–1736).
                // Sundanese Supplement bindu punctuation (U+1CC0–U+1CC7, Po; bindu surya through bindu pameneng) are
                // not Rust whitespace; Sundanese–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Kayah Li signs cwi and shya (U+A92E, U+A92F, Po) and Rejang section mark (U+A95F, Po) are not Rust
                // whitespace; Myanmar-extended / Sumatra-script HTML or Unicode samples can glue Latin tokens without ASCII space.
                // Phags-pa single/double head marks and shad / double shad (U+A874–U+A877, Po) are not Rust whitespace;
                // Phags-pa–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Saurashtra danda / double danda (U+A8CE, U+A8CF, Po) are not Rust whitespace; Saurashtra–Latin or
                // Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Devanagari Extended pushpika, gap filler, caret, and siddham (U+A8F8–U+A8FA, U+A8FC, Po) are not Rust
                // whitespace; editorial Devanagari–Latin HTML can glue Latin tokens without ASCII space. U+A8FB HEADSTROKE (Lo)
                // stays unmapped—letter-like, word-internal risk.
                // Myanmar Extended-A Aiton exclamation / one / two (U+AA77–U+AA79, So) are not Rust whitespace; Khamti /
                // Aiton or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Coptic symbols MI RO through SHIMA SIMA (U+2CE5–U+2CEA, So) and Old Nubian full stop / question marks /
                // verse divider (U+2CF9–U+2CFC, Po) are not Rust whitespace; Coptic–Latin or Unicode-sample HTML can glue
                // Latin tokens without ASCII space. U+2CF0–U+2CF1 combining marks (Mn), letters U+2CF2–U+2CF3 (Lu/Ll), and
                // U+2CFD FRACTION ONE HALF (No) stay unmapped—word-internal or numeric risk. Full stop and morphological
                // divider (U+2CFE, U+2CFF, Po) remain on the same arm.
                // Linear B word separator dot / small / medium / large (U+10079–U+1007C, Po) are not Rust whitespace;
                // Mycenaean transliteration or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Lisu punctuation comma / full stop (U+A4FE, U+A4FF, Po) and Vai comma / full stop / question / exclamation
                // (U+A60C–U+A60F, Po) are not Rust whitespace; Fraser- or Vai–Latin bilingual or Unicode-sample HTML can glue
                // Latin tokens without ASCII space.
                // Canadian Syllabics hyphen (U+1400, Pd) is not Rust whitespace; Aboriginal–Latin or Unicode-sample HTML
                // can glue Latin tokens without ASCII space (same spirit as Unicode dash punctuation U+2010–U+2015). Full stop
                // (U+166E, Po) is not Rust whitespace; U+166D CHI SIGN (So) stays unmapped—syllabics-internal risk. Bamum
                // sentence punctuation (U+A6F2–U+A6F7, Po), Mro danda / double danda
                // (U+16A6E–U+16A6F, Po), and New Tai Lue signs lae / laev (U+19DE–U+19DF, So) are not Rust whitespace;
                // Aboriginal / Bamum / Mro / New Tai Lue–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Buginese pallawa and end-of-section (U+1A1E, U+1A1F, Po) are not Rust whitespace; Buginese–Latin or
                // Unicode-sample HTML can glue Latin tokens without ASCII space. Tai Tham signs wiang through reversed
                // rotated rana (U+1AA0–U+1AA6, Po) and kaan through caang (U+1AA8–U+1AAD, Po) are not Rust whitespace;
                // Lanna–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space. U+1AA7 MAI YAMOK (Lm) is
                // omitted—modifier-like, word-internal risk (same spirit as Thai U+0E46).
                // Currency Symbols U+20A0–U+20C1 (all Sc) are not Rust whitespace; multilingual
                // price copy or Unicode-sample HTML can place euro, rupee, bitcoin, som (U+20C0,
                // Unicode 14), Saudi riyal (U+20C1, Unicode 17), etc. between Latin tokens without
                // ASCII space. Extends FEAT-D141 (formerly U+20A0–U+20BF only). U+20C2–U+20CF are
                // unassigned (`Cn`) and stay unmapped.
                // Letterlike Symbols U+2100–U+214F: only So/Sm subranges (account-of, degree signs,
                // degree Celsius / care / cada una / scruple / Fahrenheit (U+2103–U+2106, U+2108–U+2109, So);
                // U+2107 EULER CONSTANT is `Lu` in UnicodeData—stays unmapped (FEAT-D252; FEAT-D230 assumed `So`).
                // Numero, prescription take, trade mark, sans-serif math symbols, per sign, etc.;
                // not Rust whitespace. Lu/Ll/Lo mathematical letters (e.g. U+2102, U+2107, U+210E–U+2113,
                // U+2115, U+2119–U+2124, U+2126, U+212A–U+212D, U+212F–U+2134, U+2135–U+2138,
                // U+213C–U+213E, U+2145–U+2149, U+214E) stay unmapped—word-internal risk.
                // INFORMATION SOURCE (U+2139, `So`) is not Rust whitespace—UI / accessibility copy or
                // Unicode-sample HTML can place ⓘ between Latin tokens without ASCII space; maps with
                // turned sans-serif So U+213A–U+213B (FEAT-D251).
                // DOUBLE-STRUCK N-ARY SUMMATION (U+2140, `Sm`) maps—not Rust whitespace; extends turned
                // sans-serif `So` U+2139–U+213B into double-struck `Sm` U+2140–U+2144 (n-ary summation,
                // DOUBLE-STRUCK DIGIT TWO–FOUR, TURNED SANS-SERIF CAPITAL G; all `Sm` in UnicodeData—not `So`).
                // U+213F DOUBLE-STRUCK CAPITAL PI is `Lu` in UnicodeData—stays unmapped (FEAT-D246; historical FEAT-D231
                // assumed n-ary summation at U+213F).
                // U+214A PROPERTY LINE (`So`), U+214B TURNED AMPERSAND (`Sm`), U+214C PER SIGN (`So`),
                // U+214D AKTIESELSKAB (`So`), and U+214F SYMBOL FOR SAMARITAN SOURCE (`So`) map on dedicated
                // match arms—Letterlike gap between U+2144 and Number Forms U+2150+; not Rust whitespace (FEAT-D262).
                // U+214E TURNED SMALL F (`Ll`) stays unmapped—letter-like, word-internal risk.
                // U+20D0–U+20FF combining marks for symbols stay unmapped—combining / enclosing risk.
                // Number Forms U+2150–U+2182 and U+2185–U+218B: vulgar fractions, Roman numerals,
                // late Roman numerals, vulgar fraction zero thirds (U+2189, `No`), turned digits
                // U+218A–U+218B (TURNED DIGIT TWO/THREE; both `So` in UnicodeData—symbol digits, not `Nl`;
                // FEAT-D265 corrects FEAT-D263’s misread). Every other mapped scalar in these arms is `No` or `Nl`.
                // Not Rust whitespace;
                // typography or Unicode-sample HTML can place ⅓, Ⅷ, vulgar fraction zero thirds
                // (U+2189), turned digits, etc. between Latin tokens
                // without ASCII space. U+2183 ROMAN NUMERAL REVERSED ONE HUNDRED (Lu) and U+2184
                // LATIN SMALL LETTER REVERSED C (Ll) stay unmapped—letter-like, word-internal risk.
                // U+218C–U+218F are unassigned and stay unmapped.
                // Arrows U+2190–U+21FF (mostly Sm) are not Rust whitespace; MathML, diagram, or
                // Unicode-sample HTML can place arrows between Latin tokens without ASCII space.
                // Number Forms fractions / Roman numerals (U+2150–U+2182, U+2185–U+218B; includes U+2189 (`No`)
                // and U+218A–U+218B (`So`); remaining mapped scalars `No`/`Nl`) sit after Currency Symbols and before Arrows;
                // U+2183/U+2184 letter-like scalars stay unmapped.
                // Mathematical Operators U+2200–U+22FF, Miscellaneous Technical U+2300–U+23FF, and Control Pictures U+2400–U+243F
                // are mapped in the following contiguous range arms.
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
                // Superscripts and Subscripts: super/subscript plus / minus / equals (U+207A–U+207C,
                // U+208A–U+208C, Sm) and super/subscript parentheses (U+207D–U+207E, U+208D–U+208E,
                // Ps/Pe) are not Rust whitespace; math or Unicode-sample HTML can place them between
                // Latin tokens without ASCII space. Superscript / subscript digit numerics (U+2070–U+2079,
                // U+2080–U+2089, No), superscript letters U+2071 / U+207F (Lm), and unassigned U+208F (Cn)
                // stay unmapped—numeric / word-internal risk.
                // One dot leader / two dot leader / horizontal
                // ellipsis / hyphenation point (U+2024–U+2027, Po) are not Rust whitespace either;
                // TOC-style leaders or UI copy like "more…" can glue Latin tokens without ASCII space.
                // Supplemental Punctuation U+2E00–U+2E5D except U+2E2F (editorial / transcription brackets and
                // marks, word-separator dots, dashes, stenographic and medieval punctuation, Tironian
                // et, specialized brackets, oblique hyphen; Po / Pd / Pi / Pf / Ps / Pe / So as assigned)
                // are not Rust whitespace—scholarly HTML can sit them between Latin tokens without ASCII space.
                // U+2E2F VERTICAL TILDE is `Lm` in UnicodeData; it stays unmapped—modifier-letter / word-internal risk
                // (FEAT-D255). Earlier arms covered only U+2E31–U+2E3B; U+2E00–U+2E2E and U+2E30–U+2E5D are now two
                // match ranges around U+2E2F. Block tail U+2E5E–U+2E7F is unassigned (`Cn`) as of Unicode 17—do not widen
                // the separator arms to the block end; future assignments need explicit category review (FEAT-D256).
                // Runic single /
                // multiple / cross punctuation (U+16EB–U+16ED, Po) are not Rust whitespace; epigraphic
                // or Unicode-sample HTML can glue Latin tokens for `split_whitespace()`. Ogham feather
                // mark and reversed feather mark (U+169B, U+169C, Ps/Pe) are not Rust whitespace;
                // epigraphic Ogham–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Duployan SIGN O WITH CROSS (U+1BC9C, So) is not Rust whitespace; shorthand or Unicode-
                // sample HTML can glue Latin tokens without ASCII space. Unassigned U+1BC9A–U+1BC9B (Cn) stay
                // unmapped. CHINOOK FULL STOP (U+1BC9F, Po) is visible sentence punctuation (U+1BC9D–U+1BC9E Mn
                // and U+1BCA0–U+1BCA3 Cf are mapped separately). Tifinagh separator mark (U+2D70, Po) is not Rust whitespace;
                // Berber / Unicode-sample HTML can glue Latin tokens without ASCII space. Aegean word
                // separator line / dot / check mark (U+10100–U+10102, Po), weight / measure signs (U+10137–U+1013F,
                // So), and Phoenician word separator (U+1091F, Po) are not Rust whitespace; scholarly or Unicode-sample
                // HTML can place them between Latin tokens without ASCII space. Unassigned gap U+10134–U+10136 (Cn) and
                // Aegean number forms U+10107–U+10133 (No) stay unmapped—numeric / unassigned risk. Ancient Symbols (Unicode block U+10190–U+101CF): assigned
                // Roman metrological and ASCIA So U+10190–U+1019C and GREEK SYMBOL TAU RHO U+101A0; not Rust
                // whitespace. Unassigned gaps U+1019D–U+1019F and U+101A1–U+101CF (UnicodeData Cn) stay unmapped.
                // Phaistos Disc logograms U+101D0–U+101FC (all So) are not Rust whitespace; U+101FD COMBINING
                // OBLIQUE STROKE (Mn) and U+101FE–U+101FF (Cn) stay unmapped. Ancient Greek Numbers (Unicode block
                // U+10140–U+1018E): metrological / currency `So` U+10179–U+10189 and U+1018C–U+1018E are not Rust
                // whitespace; scholarly HTML can place them between Latin tokens without ASCII space. Acrophonic and
                // tally `Nl` U+10140–U+10174, fraction `No` U+10175–U+10178, and `No` zero / quarter U+1018A–U+1018B
                // stay unmapped—numeric risk. Unassigned U+1018F (Cn) stays unmapped. Aegean U+10100–U+10102 and
                // U+10137–U+1013F (`So`) above.
                // Ugaritic word divider (U+1039F), Old Persian word
                // divider (U+103D0), Caucasian Albanian citation mark (U+1056F), Imperial Aramaic section
                // sign (U+10857), Lydian triangular mark (U+1093F), Old South Arabian numeric indicator
                // (U+10A7F), Manichaean SIGN UD (U+10AC8, So; the lone symbol among Manichaean letters), and Manichaean
                // punctuation star through line filler (U+10AF0–U+10AF6, Po) are not Rust whitespace; epigraphic or
                // Unicode-sample HTML can glue Latin tokens without ASCII space. Manichaean abbreviation marks (U+10AE5–
                // U+10AE6, Mn) and numbers (U+10AEB–U+10AEF, No) stay unmapped—combining / numeric risk.
                // Palmyrene left- and right-pointing fleurons (U+10877, U+10878, So) are not Rust whitespace;
                // epigraphic Palmyrene–Latin or Unicode-sample HTML can place them between tokens without ASCII space.
                // Pahawh Hmong clause and sentence signs (U+16B37–U+16B3B, U+16B44, Po) are not Rust whitespace;
                // U+16B30–U+16B36 (Mn), U+16B40–U+16B43 (Lm), and U+16B3C–U+16B3F / U+16B45 (So) stay unmapped.
                // Nabataean word separator (U+1089F, Po) and Hatran section mark (U+108FB, Po) are not Rust
                // whitespace; epigraphic or Unicode-sample HTML can place them between Latin tokens without ASCII space.
                // Old North Arabian word divider (U+10A9D, Po) is not Rust whitespace; U+10A9E / U+10A9F (Nl) stay
                // unmapped. Elymaic U+10FF5 is LETTER TAW (`Lo`), not punctuation; it stays unmapped so Elymaic script
                // text is not split like Latin tokens (FEAT-D213; historical FEAT-D110 called it a section mark `Po`).
                // Hanifi Rohingya: UnicodeData leaves U+10D28–U+10D2F unassigned (`Cn`) between U+10D27 SIGN TASSI (`Mn`)
                // and digit numerics U+10D30+ (`Nd`); historical FEAT-D111 matched U+10D29–U+10D2D as sentence `Po` (stale).
                // FEAT-D254 removed that arm—reserved scalars must not be word separators. Combining signs U+10D24–U+10D27 (`Mn`)
                // and letters / marks U+10D22–U+10D23 (`Lo`) stay unmapped—word-internal risk.
                // Garay hyphen (U+10D6E, Pd) and Garay plus / minus (U+10D8E–U+10D8F, Sm) are not Rust whitespace;
                // Senegalese Garay typography or Unicode-sample HTML can place them between Latin tokens without ASCII space.
                // Arabic Biblical end of verse (U+10ED0, Po) is not Rust whitespace; scholarly Arabic Extended-C HTML
                // can place it between Latin tokens without ASCII space. U+10ED1–U+10ED8 honorific ligatures (So) stay
                // unmapped—multi-letter units, not general clause punctuation.
                // SignWriting comma through parenthesis (U+1DA87–U+1DA8B, Po) are not Rust whitespace; Sutton
                // SignWriting–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Ol Onal abbreviation sign (U+1E5FF, Po) is not Rust whitespace; Unicode-sample or mixed-script HTML
                // can glue Latin tokens without ASCII space.
                // Yezidi hyphenation mark (U+10EAD, Pd) is not Rust whitespace; Kurdish Yezidi or Unicode-sample HTML
                // can place it between Latin tokens without ASCII space.
                // Indic Siyaq placeholder (U+1ECAC, So) and rupee mark (U+1ECB0, Sc) are not Rust whitespace;
                // Indo-Persian financial notation or Unicode-sample HTML can glue Latin tokens for `split_whitespace()`.
                // Ottoman Siyaq Numbers: MARRATAN (U+1ED2E, So) is not Rust whitespace; Ottoman–Latin or Unicode-sample
                // HTML can place it between Latin tokens without ASCII space. All other assigned scalars in the block are
                // No (U+1ED01–U+1ED2D, U+1ED2F–U+1ED3D)—numeric / word-internal risk, excluded. Gap U+1ED3E+ (Cn) excluded.
                // Arabic Mathematical Alphabetic Symbols: operator MEEM WITH HAH WITH TATWEEL (U+1EEF0, Sm) and
                // HAH WITH DAL (U+1EEF1, Sm) are not Rust whitespace; MathML- or Unicode-sample HTML can place them
                // between Latin tokens without ASCII space. The rest of U+1EE00–U+1EEFF is Lo or unassigned (Cn)—excluded.
                // Vithkuqi: historical FEAT-D117 mapped U+1057B–U+1057F as sentence `Po`; current UnicodeData has an
                // unassigned gap at U+1057B (`Cn`) and U+1057C–U+1057F as capital letters (`Lu`)—same spirit as
                // FEAT-D212 / FEAT-D213; they stay unmapped. Caucasian Albanian citation mark U+1056F (`Po`) remains mapped.
                // Warang Citi U+118C8–U+118CF are SMALL LETTER E through SMALL LETTER UC (Ll) in Unicode 14+—not
                // punctuation; they stay unmapped so Warang Citi syllables are not torn apart (historical FEAT-D111 arm removed, FEAT-D212).
                // Devanagari Extended-A head marks and bhale signs (U+11B00–U+11B09, Po) are not Rust whitespace;
                // editorial Devanagari–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Sunuwar sign pvo (U+11BE1, Po) is not Rust whitespace; Nepal–Unicode-sample HTML can glue Latin
                // tokens without ASCII space.
                // Bassa Vah full stop (U+16AF5, Po) is not Rust whitespace; Liberian / Unicode-sample HTML can glue
                // Latin tokens without ASCII space.
                // Kirat Rai sign yupi, danda, double danda (U+16D6D–U+16D6F, Po) are not Rust whitespace;
                // U+16D6B SIGN VIRAMA and U+16D6C SIGN SAAT (Lm) stay unmapped—modifier-like, word-internal risk.
                // Ethiopic section mark through paragraph separator (U+1360–U+1368, Po) are not Rust whitespace; extends FEAT-D48
                // wordspace (U+1361) / full stop (U+1362) with section mark (U+1360) and clause punctuation (U+1363–U+1368). Ethiopic
                // Supplement tonal marks U+1390–U+1399 (YIZET through KURT; all So) are not Rust whitespace; Ethiopic-layout or
                // Unicode-sample HTML can glue Latin tokens without ASCII space. Ethiopic Supplement syllables
                // U+1380–U+138F (Lo) stay unmapped—word-internal risk. Unassigned tail U+139A–U+139F (Cn) excluded.
                // Braille Patterns (U+2800–U+28FF;
                // all So, including pattern blank U+2800) are not Rust whitespace; accessibility or
                // Unicode-sample HTML can place dot patterns between Latin tokens without ASCII space.
                // Supplemental Arrows-B (U+2900–U+297F; mostly Sm) and Miscellaneous Mathematical Symbols-B
                // (U+2980–U+29FF; delimiters, operators, shapes—mostly Sm) are not Rust whitespace; MathML,
                // LaTeX-to-HTML, or Unicode-sample pages can place them between Latin tokens without ASCII space.
                // Supplemental Mathematical Operators (U+2A00–U+2AFF; mostly Sm) are not Rust whitespace;
                // advanced MathML / Unicode-sample HTML can place n-ary operators, large operators, and
                // integrals between Latin tokens without ASCII space.
                // Miscellaneous Symbols and Arrows (U+2B00–U+2BFF; arrows, math symbols, game pieces—mostly
                // So / Sm / Po as assigned) are not Rust whitespace; diagram, chess, or Unicode-sample HTML
                // can place them between Latin tokens without ASCII space.
                // Optical Character Recognition hook through double backslash (U+2440–U+245F, So)
                // are not Rust whitespace; OCR zoning / anchor symbols in scanned PDF or legacy HTML exports
                // can sit between Latin tokens without ASCII space. Enclosed Alphanumerics (U+2460–U+24FF: circled
                // and parenthesized digits, digit full stops, enclosed Latin letters, etc.; No / So) are not Rust
                // whitespace; Unicode-sample or list-style HTML can place them between Latin tokens without ASCII space.
                // Enclosed Alphanumeric Supplement U+1F100–U+1F1AD and U+1F1E6–U+1F1FF (parenthesized / circled /
                // squared Latin and digits, regional indicators, etc.; No / So as assigned) are not Rust whitespace;
                // emoji-era labels or Unicode-sample HTML can place them between Latin tokens without ASCII space.
                // U+1F1AE–U+1F1E5 are unassigned and stay unmapped.
                // Mahjong Tiles U+1F000–U+1F02F, Domino Tiles U+1F030–U+1F093, and Playing Cards U+1F0A0–U+1F0FF (all
                // So as assigned) are not Rust whitespace; game or Unicode-sample HTML can place tile / suit glyphs
                // between Latin tokens without ASCII space. U+1F094–U+1F09F are reserved—excluded.
                // Enclosed Ideographic Supplement: assigned subranges U+1F200–U+1F202, U+1F210–U+1F23B, U+1F240–U+1F248,
                // U+1F250–U+1F251, U+1F260–U+1F265 (squared katakana / CJK, tortoise-shell bracketed ideographs, circled
                // advantage/accept, rounded fu/lu/shou/xi/shuangxi/cai; all So) are not Rust whitespace; Japanese mobile
                // carrier or Unicode-sample HTML can place them between Latin tokens without ASCII space. Gaps
                // U+1F203–U+1F20F, U+1F23C–U+1F23F, U+1F249–U+1F24F, U+1F252–U+1F25F, U+1F266–U+1F2FF unassigned—excluded.
                // Miscellaneous Symbols and Pictographs (U+1F300–U+1F5FF), Emoticons (U+1F600–U+1F64F), and ornamental
                // dingbat leaf pointers U+1F650–U+1F67F (mostly So / emoji presentation); not Rust whitespace. Emoji or
                // Unicode-sample HTML can place cyclones, smileys, etc. between Latin tokens without ASCII space. The
                // unassigned gap U+1F266–U+1F2FF before U+1F300 stays unmapped.
                // Transport and Map Symbols: assigned U+1F680–U+1F6D7, U+1F6E0–U+1F6EC, U+1F6F0–U+1F6FC (rockets, vehicles,
                // map pins, clocks, etc.—mostly So / emoji presentation); not Rust whitespace. Travel or emoji HTML can
                // place them between Latin tokens without ASCII space. Unassigned gaps U+1F6D8–U+1F6DF, U+1F6ED–U+1F6EF,
                // U+1F6FD–U+1F6FF stay unmapped.
                // Alchemical Symbols U+1F700–U+1F773 (all So as assigned); not Rust whitespace. Scholarly or Unicode-sample
                // HTML can place alembic, sulfur, mercury, etc. between Latin tokens without ASCII space. Tail U+1F774–U+1F77F
                // unassigned—excluded.
                // Geometric Shapes Extended: assigned U+1F780–U+1F7D8 and U+1F7E0–U+1F7EB (triangles, heavy/light geometric
                // symbols, emoji-style color squares / circles—mostly So); not Rust whitespace. Wingdings-style or Unicode-sample
                // HTML can place them between Latin tokens without ASCII space. Gaps U+1F7D9–U+1F7DF and U+1F7EC–U+1F7FF
                // unassigned—excluded.
                // Supplemental Arrows-C: assigned U+1F800–U+1F80B, U+1F810–U+1F847, U+1F850–U+1F859, U+1F860–U+1F887,
                // U+1F890–U+1F8AD, U+1F8B0–U+1F8B1 (mostly Sm); not Rust whitespace. Diagram or Unicode-sample HTML can place
                // wide / shadowed / bent arrows between Latin tokens without ASCII space. Gaps U+1F80C–U+1F80F, U+1F848–U+1F84F,
                // U+1F85A–U+1F85F, U+1F888–U+1F88F, U+1F8AE–U+1F8AF, U+1F8B2–U+1F8FF unassigned—excluded.
                // Supplemental Symbols and Pictographs U+1F900–U+1F9FF (mostly So / emoji presentation; Typikon
                // crosses, hand gestures, food, objects, etc.); not Rust whitespace. The block is fully assigned as of
                // Unicode 14.0+. Emoji or Unicode-sample HTML can place these between Latin tokens without ASCII space.
                // Chess Symbols: U+1FA00–U+1FA57 and U+1FA60–U+1FA6D (mostly So); not Rust whitespace. Gaps U+1FA58–U+1FA5F
                // and U+1FA6E–U+1FA6F unassigned—excluded. Unicode-sample or game-notation HTML can place chess / xiangqi
                // glyphs between Latin tokens without ASCII space.
                // Symbols and Pictographs Extended-A: assigned subranges U+1FA70–U+1FA7C, U+1FA80–U+1FA8A, U+1FA8E–U+1FA8F,
                // U+1FA90–U+1FAC6, U+1FAC8, U+1FACD–U+1FACF, U+1FAD0–U+1FADC, U+1FADF, U+1FAE0–U+1FAEA, U+1FAEF,
                // U+1FAF0–U+1FAF8 (mostly So / emoji presentation); not Rust whitespace. Gaps U+1FA7D–U+1FA7F, U+1FA8B–U+1FA8D,
                // U+1FAC7, U+1FAC9–U+1FACC, U+1FADD–U+1FADE, U+1FAEB–U+1FAEE, U+1FAF9–U+1FAFF unassigned—excluded.
                // Symbols for Legacy Computing: assigned So U+1FB00–U+1FB92 and U+1FB94–U+1FBEF (PETSCII / block graphics,
                // box-drawing diagonals, stick figures, etc.); not Rust whitespace. U+1FB93 unassigned—excluded. U+1FBF0–U+1FBF9
                // SEGMENTED DIGIT ZERO–NINE (Nd)—excluded (numeric / word-internal risk). U+1FBFA–U+1FBFF unassigned—excluded.
                // BMP Greek Extended spacing polytonic `Sk` U+1FBD, U+1FBF, U+1FC0–U+1FC1, U+1FCD–U+1FCF,
                // U+1FDD–U+1FDF, U+1FED–U+1FEF, U+1FFD–U+1FFE map on the dedicated arm (FEAT-D244)—not Rust
                // whitespace; unrelated to supplementary-plane Symbols for Legacy Computing U+1FB00+ (Nd/So).
                // Symbols for Legacy Computing Supplement: assigned So U+1CC00–U+1CCFC, U+1CD00–U+1CEB3, U+1CEBA–U+1CEBF (Unicode 17
                // block; game sprites, schematics, octants, outlined Latin/digits as So, etc.); not Rust whitespace. Sundanese
                // Supplement U+1CC0–U+1CC7 (Po) stays on its own arm—distinct from supplementary-plane U+1CC00+. Gaps U+1CCFD–U+1CCFF
                // and U+1CEB4–U+1CEB9 unassigned—excluded.
                // Miscellaneous Symbols Supplement (Unicode 17 block U+1CEC0–U+1CEFF): assigned So U+1CEC0–U+1CED0 (asteroid
                // symbols), U+1CEE0–U+1CEEF (geomantic figures), U+1CEF0 (medium small white circle with horizontal bar); not Rust
                // whitespace. Gaps U+1CED1–U+1CEDF and U+1CEF1–U+1CEFF unassigned—excluded.
                // Znamenny Musical Notation (Unicode block U+1CF00–U+1CFCF): assigned So neumes U+1CF50–U+1CFC3; not Rust
                // whitespace. Combining marks U+1CF00–U+1CF2D and U+1CF30–U+1CF46 (Mn), unassigned gaps U+1CF2E–U+1CF2F and
                // U+1CF47–U+1CF4F, and tail U+1CFC4–U+1CFCF (Cn)—excluded.
                // Byzantine Musical Symbols U+1D000–U+1D0F5 (all So as assigned); not Rust whitespace. Scholarly or Unicode-sample
                // HTML can place psili, neumes, kentimata, etc. between Latin tokens without ASCII space. Tail U+1D0F6–U+1D0FF
                // unassigned—excluded.
                // Western Musical Symbols (Unicode block U+1D100–U+1D1FF): assigned U+1D100–U+1D126, U+1D129–U+1D172, U+1D17B–U+1D1EA
                // (mostly So; Mc/Mn combining stems, flags, articulation, etc.); not Rust whitespace. MusicXML or Unicode-sample HTML
                // can place barlines, clefs, noteheads, mensural / Gregorian / Kievan / Iranian marks between Latin tokens without ASCII
                // space. Gaps U+1D127–U+1D128 and U+1D1EB–U+1D1FF unassigned—excluded. Format controls U+1D173–U+1D17A (Cf) stay on the
                // dedicated arm (FEAT-D51 / overlap with beam–tie–slur–phrase).
                // Ancient Greek Musical Notation (Unicode block U+1D200–U+1D24F): assigned contiguous U+1D200–U+1D245
                // (vocal / instrumental notation symbols, mostly So; U+1D242–U+1D244 combining triseme/tetraseme/pentaseme Mn);
                // not Rust whitespace. Scholarly or Unicode-sample HTML can place them between Latin tokens without ASCII space.
                // Tail U+1D246–U+1D24F unassigned—excluded. Distinct from Phonetic Extensions U+1D20–U+1D2B (Ll) and Western U+1D100+.
                // Tai Xuan Jing Symbols U+1D300–U+1D356 (all So as assigned); not Rust whitespace. I Ching / scholarly HTML
                // can place monograms / digrams / tetragrams between Latin tokens without ASCII space. Gaps U+1D246–U+1D2FF before
                // U+1D300 include unassigned scalars and Mayan numerals U+1D2E0–U+1D2F3 (No)—excluded. Gap U+1D357–U+1D35F and
                // Counting Rod Numerals U+1D360–U+1D378 (No) after U+1D356—excluded (numeric / word-internal risk, like FEAT-D158 Nd).
                // Yijing Hexagram Symbols U+4DC0–U+4DFF (all So as assigned); not Rust whitespace. I Ching / Unicode-sample HTML
                // can place the 64 hexagram glyphs between Latin tokens without ASCII space. CJK Unified Ideographs Extension A
                // U+3400–U+4DBF (Lo) immediately before this block and CJK Unified U+4E00+ after—excluded (word-internal risk).
                // Yi Radicals U+A490–U+A4C6 (all So as assigned); not Rust whitespace. Yi-script indexes or Unicode-sample HTML
                // can place radical glyphs between Latin tokens without ASCII space. Unassigned tail U+A4C7–U+A4CF (Cn) and Lisu
                // letters U+A4D0+ (Lo)—excluded.
                // Cyrillic CYRILLIC THOUSANDS SIGN (U+0482, So) is not Rust whitespace; Church Slavonic or Unicode-sample HTML
                // can place it between Latin tokens without ASCII space—distinct from Cyrillic letters (Lu/Ll) in the block.
                // Combining titlo stack U+0483–U+0489 (Mn/Me) stay unmapped—word-internal risk.
                // Cyrillic Extended-B: Slavonic asterisk U+A673 and Cyrillic kavyka U+A67E (Po); not Rust whitespace. Church
                // Slavonic or Unicode-sample HTML can place them between Latin tokens without ASCII space. Combining marks
                // U+A672 (Me), U+A674 / U+A67D (Mn), and letter modifier U+A67F (Lm)—excluded.
                // Syloti Nagri poetry marks U+A828–U+A82B (all So); not Rust whitespace. Syloti Nagri letters U+A800+ and
                // combining / Mc vowel signs before U+A828—excluded.
                // North Indic Number Forms: quarter / placeholder / quantity marks U+A836–U+A837 / U+A839 (So) and rupee mark
                // U+A838 (Sc); not Rust whitespace. Fraction numerics U+A830–U+A835 (No) and unassigned U+A83A–U+A83F (Cn)—excluded.
                // Mathematical Alphanumeric Symbols U+1D400–U+1D7FF: almost all Lu / Ll / Nd (styled math letters and digits)—excluded
                // (word-internal / numeric risk, like the rest of the block). Only ten Sm scalars—MATHEMATICAL * NABLA and * PARTIAL
                // DIFFERENTIAL in bold / italic / sans-serif faces (U+1D6C1, U+1D6DB, U+1D6FB, U+1D715, U+1D735, U+1D74F, U+1D76F,
                // U+1D789, U+1D7A9, U+1D7C3)—are not Rust whitespace; MathML or Unicode-sample HTML can place them between Latin
                // tokens without ASCII space. BMP ∇ U+2207 / ∂ U+2202 stay on the U+2200–U+22FF arm. Unassigned gaps in the block
                // (UnicodeData Cn) and all letter/digit categories stay unmapped.
                // Sutton SignWriting (Unicode block U+1D800–U+1DAAF): assigned So U+1D800–U+1D9FF, U+1DA37–U+1DA3A,
                // U+1DA6D–U+1DA74, U+1DA76–U+1DA83, U+1DA85–U+1DA86; not Rust whitespace. Combining rotation / head /
                // location marks U+1DA00–U+1DA36, U+1DA3B–U+1DA6C, U+1DA75, U+1DA84, U+1DA9B–U+1DA9F, U+1DAA1–U+1DAAF (Mn)—
                // excluded (word-internal risk). Po U+1DA87–U+1DA8B stay on the dedicated arm (FEAT-D115). Gap U+1DA8C–U+1DA9A
                // unassigned (Cn)—excluded.
                // Box Drawing (U+2500–U+257F), Block Elements (U+2580–U+259F), and Geometric Shapes (U+25A0–U+25FF;
                // mostly So) are not Rust whitespace; ASCII-art tables, UI mockups, or Unicode-sample HTML can place
                // light horizontal / shaded blocks / filled squares between Latin tokens without ASCII space.
                // Miscellaneous Symbols (U+2600–U+26FF; weather, chess, card suits, warning signs, etc.—mostly So / Sm /
                // Po as assigned) are not Rust whitespace; emoji-era or Unicode-sample HTML can place them between
                // Latin tokens without ASCII space.
                // Dingbats (U+2700–U+27BF; checkmarks, crosses, pencil, scissors, decorative bullets, etc.—mostly So /
                // Sm / Po as assigned) are not Rust whitespace; Zapf Dingbats–style or Unicode-sample HTML can place
                // them between Latin tokens without ASCII space.
                // Miscellaneous Mathematical Symbols-A (U+27C0–U+27EF; angles, triangles, set/membership notations,
                // etc.—mostly Sm) and Supplemental Arrows-A (U+27F0–U+27FF; long arrows, mostly Sm) are not Rust
                // whitespace; MathML, LaTeX-to-HTML, or Unicode-sample pages can place them between Latin tokens without
                // ASCII space.
                // Duployan SIGN O WITH CROSS (U+1BC9C, So) is not Rust whitespace; mapped like other visible So
                // shorthand symbols. Thick letter selector / double mark (U+1BC9D–U+1BC9E, Mn), CHINOOK FULL
                // STOP (U+1BC9F, Po), and shorthand format overlap / step (U+1BCA0–U+1BCA3, Cf) are not Rust
                // whitespace (Mn/Cf mapped with other controls; U+1BC9F mapped as visible Po).
                // Kaithi number signs (U+110BD, U+110CD, Cf) are not Rust whitespace; Indic numeral
                // layout HTML can place them between scripts without an ASCII space. Kaithi
                // abbreviation / enumeration and section marks through double danda (U+110BB, U+110BC,
                // U+110BE–U+110C1, Po) are mapped here as word separators; they are not Rust
                // whitespace. Egyptian
                // hieroglyph format joiners / segment markers (U+13430–U+13455, Cf) and musical
                // symbol begin/end beam–tie–slur–phrase (U+1D173–
                // U+1D17A, Cf) are not Rust whitespace; scholarly or MusicXML-derived HTML can place
                // them between scripts without an ASCII space. Brahmi number joiner (U+1107F, Mn) is
                // not Rust whitespace; Indic numeral layout can sit between scripts without ASCII
                // space. Brahmi danda / double danda and punctuation dot through lotus (U+11047–
                // U+1104D, Po) are not Rust whitespace; epigraphic or Unicode-sample HTML can glue
                // Latin tokens without ASCII space. Ideographic description characters (U+2FF0–
                // U+2FFF, So) and U+31EF IDEOGRAPHIC DESCRIPTION CHARACTER SUBTRACTION (`So`) are not Rust whitespace;
                // Han-component notation or pasted CJK-scholarly HTML can glue Latin tokens. Chakma section mark and sentence punctuation (U+11140–
                // U+11143, Po) are not Rust whitespace; U+11144–U+11147 (Lo/Mc) stay unmapped.
                // Sharada danda / double danda / abbreviation / separator / sutra mark (U+111C5–U+111C8,
                // U+111CD, Po), sign siddham (U+111DB, Po), and continuation / section marks (U+111DD–
                // U+111DF, Po) are not Rust whitespace; U+111DA EKAM and U+111DC HEADSTROKE (Lo) stay
                // unmapped. Sharada–Latin or Unicode-sample HTML can glue
                // Latin tokens without ASCII space. Khojki danda through abbreviation sign (U+11238–
                // U+1123D, Po) are not Rust whitespace; Khojki–Latin or Unicode-sample HTML can glue
                // Latin tokens without ASCII space. Mahajani abbreviation / section marks (U+11174–
                // U+11175, Po) are not Rust whitespace; U+11173 SIGN NUKTA (Mn) stays unmapped.
                // Multani section mark (U+112A9, Po) is not Rust whitespace—the block has no second
                // Po danda in the standard. Tulu-Tigalari danda / double danda and om / shrii
                // pushpika (U+113D4–U+113D5, U+113D7–U+113D8, Po) are not Rust whitespace; U+113D6
                // is unassigned. Newa danda through abbreviation (U+1144B–U+1144F, Po), double comma /
                // placeholder / insertion (U+1145A, U+1145B, U+1145D, Po) are not Rust whitespace;
                // U+1145C is unassigned. Tirhuta abbreviation sign (U+114C6, Po) is not Rust
                // whitespace; U+114C7 OM (Lo) stays unmapped. Modi danda / double danda /
                // abbreviation (U+11641–U+11643, Po) are not Rust whitespace. Khudawadi has no Po
                // sentence punctuation in the chart (virama / nukta are Mn); omitted like other
                // virama risks. Ahom small section / section / rulai (U+1173C–U+1173E, Po) and SYMBOL VI
                // (U+1173F, So) are not Rust whitespace; extends FEAT-D96 contiguous Po run with the block’s
                // sole So. Ahom letters (Lo, e.g. U+11700) stay unmapped—word-internal risk. Dogra
                // abbreviation sign (U+1183B, Po) is not Rust whitespace.
                // Dives Akuru double danda / gap filler / end of text (U+11944–U+11946, Po) are not
                // Rust whitespace. Nandinagari sign siddham (U+119E2, Po) is not Rust whitespace.
                // Bhaiksuki danda / double danda / word separator / gap fillers (U+11C41–U+11C45, Po)
                // are not Rust whitespace; Unicode-sample or mixed-script HTML can glue Latin tokens.
                // Zanabazar Square head marks, tsheg, shad, and double-lined marks (U+11A3F–U+11A46, Po)
                // are not Rust whitespace; Mongolian / Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Soyombo tsheg / shad / double shad and head / terminal marks (U+11A9A–U+11A9C, U+11A9E–U+11AA2, Po)
                // are not Rust whitespace; U+11A9D MARK PLUTA (Lo) stays unmapped—letter-like, word-internal risk.
                // Marchen head mark and shad (U+11C70–U+11C71, Po) are not Rust whitespace; Zhang-
                // Zhung / Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Makasar passimbang and end of section (U+11EF7–U+11EF8, Po) are not Rust whitespace;
                // Sulawesi-script or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Kawi danda through closing spiral (U+11F43–U+11F4F, Po) are not Rust whitespace;
                // Indonesian / Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Tamil Supplement metrological / land / currency signs (U+11FD5–U+11FF1, So/Sc) are
                // not Rust whitespace; U+11FD0–U+11FD4 fraction numerics (No) and unassigned
                // U+11FF2–U+11FFE (Cn) stay unmapped. END OF TEXT (U+11FFF, Po) is not Rust
                // whitespace; mixed Tamil–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Cuneiform punctuation Old Assyrian word divider through diagonal quadcolon (U+12470–
                // U+12474, Po) are not Rust whitespace; transliterated scholarly HTML can glue Latin
                // tokens without ASCII space.
                // Cypro-Minoan signs CM301 / CM302 (U+12FF1–U+12FF2, Po) are not Rust whitespace;
                // epigraphic or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Kharoshthi punctuation dot through lines (U+10A50–U+10A58, Po) are not Rust
                // whitespace; Gandharan / Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Avestan abbreviation mark and dot/ring punctuation (U+10B39–U+10B3F, Po) are not
                // Rust whitespace; Zoroastrian scholarly or Unicode-sample HTML can glue Latin tokens.
                // Psalter Pahlavi section marks (U+10B99–U+10B9C, Po) are not Rust whitespace;
                // manuscript or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Sogdian punctuation (U+10F55–U+10F59, Po) are not Rust whitespace; Silk Road /
                // Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Old Uyghur punctuation bar through four dots (U+10F86–U+10F89, Po) are not Rust
                // whitespace; Turfan / Unicode-sample HTML can glue Latin tokens without ASCII space.
                // U+10F82–U+10F85 combining dots (Mn) stay unmapped—word-internal risk.
                // Siddham sign siddham through section mark with circles and four enclosures
                // (U+115C1–U+115D7, Po) are not Rust whitespace; U+115C0 SIGN NUKTA (Mn) stays
                // unmapped—modifier-like, word-internal risk. Buddhist manuscript or Unicode-sample
                // HTML can glue Latin tokens without ASCII space.
                // Mongolian Supplement ornamental birga marks (U+11660–U+1166C, Po) are not Rust
                // whitespace; vertical Mongolian typography or Unicode-sample HTML can glue Latin
                // tokens without ASCII space—distinct from Basic Mongolian script U+1800–U+180E
                // already mapped in one arm.
                // Takri abbreviation sign, double danda, danda, section mark (U+116B9–U+116BC, Po)
                // are not Rust whitespace; U+116B8 LETTER SSA (Lo) stays unmapped. Himachal /
                // Unicode-sample HTML can glue Latin tokens without ASCII space.
                '\u{0600}'..='\u{0605}'
                | '\u{06DD}'
                | '\u{0700}'..='\u{070D}'
                | '\u{070F}'
                | '\u{07F6}'
                | '\u{07F7}'..='\u{07F9}'
                | '\u{07FE}'..='\u{07FF}'
                | '\u{0830}'..='\u{083E}'
                | '\u{085E}'
                | '\u{16FE2}'
                | '\u{16FE3}'
                | '\u{1E14F}'
                | '\u{1E2FF}'
                | '\u{16E97}'..='\u{16E9A}'
                | '\u{1E95E}'..='\u{1E95F}'
                | '\u{08E2}'
                | '\u{0888}'
                | '\u{0890}'..='\u{0891}'
                | '\u{200B}'
                | '\u{200C}'
                | '\u{200D}'
                | '\u{2060}'
                | '\u{FEFF}'
                | '\u{00AD}'
                | '\u{0001}'..='\u{0008}'
                | '\u{000E}'..='\u{001F}'
                | '\u{005E}'
                | '\u{0060}'
                | '\u{007F}'
                | '\u{0080}'..='\u{009F}'
                | '\u{00A0}'
                | '\u{17B4}'
                | '\u{17B5}'
                | '\u{17D4}'..='\u{17DA}'
                | '\u{17DB}'
                | '\u{19E0}'..='\u{19FF}'
                | '\u{A9C1}'..='\u{A9CF}'
                | '\u{A9DE}'
                | '\u{A9DF}'
                | '\u{1B4E}'..='\u{1B4F}'
                | '\u{1B5A}'..='\u{1B60}'
                | '\u{1B61}'..='\u{1B6A}'
                | '\u{1B74}'..='\u{1B7F}'
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
                | '\u{309B}'
                | '\u{309C}'
                | '\u{FF65}'
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
                | '\u{2E80}'..='\u{2E99}'
                | '\u{2E9B}'..='\u{2EF3}'
                | '\u{2F00}'..='\u{2FD5}'
                | '\u{3190}'..='\u{3191}'
                | '\u{3196}'..='\u{319F}'
                | '\u{31C0}'..='\u{31E5}'
                | '\u{3200}'..='\u{321E}'
                | '\u{322A}'..='\u{3247}'
                | '\u{3250}'
                | '\u{3260}'..='\u{327F}'
                | '\u{328A}'..='\u{32B0}'
                | '\u{32C0}'..='\u{32FF}'
                | '\u{3300}'..='\u{33FF}'
                | '\u{FF0C}'
                | '\u{FF1A}'
                | '\u{FF1B}'
                | '\u{FF01}'
                | '\u{FF02}'
                | '\u{FF1F}'
                | '\u{FF03}'
                | '\u{FF04}'
                | '\u{FF05}'
                | '\u{FF06}'
                | '\u{FF08}'
                | '\u{FF09}'
                | '\u{FF0A}'
                | '\u{FF0B}'
                | '\u{FF0D}'
                | '\u{FF0E}'
                | '\u{FF1C}'..='\u{FF1E}'
                | '\u{FF20}'
                | '\u{FF3B}'
                | '\u{FF3C}'
                | '\u{FF3D}'
                | '\u{FF3E}'
                | '\u{FF5B}'
                | '\u{FF5C}'
                | '\u{FF5D}'
                | '\u{FF5F}'
                | '\u{FF60}'
                | '\u{FF61}'
                | '\u{FF62}'
                | '\u{FF63}'
                | '\u{FF64}'
                | '\u{FFE0}'
                | '\u{FFE1}'
                | '\u{FFE2}'
                | '\u{FFE4}'
                | '\u{FFE5}'
                | '\u{FFE6}'
                | '\u{FFE8}'..='\u{FFEF}'
                | '\u{FE10}'..='\u{FE19}'
                | '\u{FE30}'..='\u{FE4F}'
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
                | '\u{05C6}'
                | '\u{FB29}'
                | '\u{10FB}'
                | '\u{00A1}'
                | '\u{00A2}'..='\u{00A5}'
                | '\u{00A6}'
                | '\u{00AF}'
                | '\u{00B4}'
                | '\u{00B8}'
                | '\u{00A9}'
                | '\u{00AE}'
                | '\u{00B0}'
                | '\u{00AC}'
                | '\u{00B1}'
                | '\u{02C2}'..='\u{02C5}'
                | '\u{02D2}'..='\u{02D5}'
                | '\u{02D6}'
                | '\u{02D7}'
                | '\u{02E5}'..='\u{02EB}'
                | '\u{02ED}'
                | '\u{02EF}'..='\u{02FF}'
                | '\u{A700}'..='\u{A716}'
                | '\u{A720}'
                | '\u{A721}'
                | '\u{A789}'
                | '\u{A78A}'
                | '\u{AB5B}'
                | '\u{AB6A}'
                | '\u{AB6B}'
                | '\u{1FBD}'
                | '\u{1FBF}'
                | '\u{1FC0}'..='\u{1FC1}'
                | '\u{1FCD}'..='\u{1FCF}'
                | '\u{1FDD}'..='\u{1FDF}'
                | '\u{1FED}'..='\u{1FEF}'
                | '\u{1FFD}'..='\u{1FFE}'
                | '\u{00D7}'
                | '\u{00F7}'
                | '\u{00BC}'
                | '\u{00BD}'
                | '\u{00BE}'
                | '\u{00BF}'
                | '\u{00AB}'
                | '\u{00BB}'
                | '\u{00A7}'
                | '\u{00B6}'
                | '\u{20A0}'..='\u{20C1}'
                | '\u{2100}'..='\u{2101}'
                | '\u{2103}'..='\u{2106}'
                | '\u{2108}'..='\u{2109}'
                | '\u{2114}'
                | '\u{2116}'..='\u{2118}'
                | '\u{211E}'..='\u{2123}'
                | '\u{2125}'
                | '\u{2127}'
                | '\u{2129}'
                | '\u{212E}'
                | '\u{2139}'..='\u{213B}'
                | '\u{2140}'..='\u{2144}'
                | '\u{214A}'..='\u{214D}'
                | '\u{214F}'
                | '\u{2150}'..='\u{2182}'
                | '\u{2185}'..='\u{218B}'
                | '\u{2190}'..='\u{21FF}'
                | '\u{2200}'..='\u{22FF}'
                | '\u{2300}'..='\u{23FF}'
                | '\u{2400}'..='\u{243F}'
                | '\u{037E}'
                | '\u{0384}'
                | '\u{0385}'
                | '\u{03D6}'
                | '\u{03F6}'
                | '\u{0482}'
                | '\u{0609}'
                | '\u{060A}'
                | '\u{060B}'
                | '\u{060C}'
                | '\u{060D}'
                | '\u{060E}'
                | '\u{060F}'
                | '\u{061B}'
                | '\u{061D}'
                | '\u{061E}'
                | '\u{061F}'
                | '\u{06D4}'
                | '\u{066A}'..='\u{066D}'
                | '\u{FBC3}'..='\u{FBD2}'
                | '\u{FD3E}'
                | '\u{FD3F}'
                | '\u{FD40}'..='\u{FD4F}'
                | '\u{FDCF}'
                | '\u{FDFC}'
                | '\u{FDFD}'
                | '\u{FDFE}'
                | '\u{FDFF}'
                | '\u{055C}'..='\u{055F}'
                | '\u{0589}'
                | '\u{058A}'
                | '\u{058D}'..='\u{058F}'
                | '\u{0964}'
                | '\u{0965}'
                | '\u{0970}'
                | '\u{1CD3}'
                | '\u{0A76}'
                | '\u{0AF0}'
                | '\u{0AF1}'
                | '\u{0C77}'
                | '\u{0C7F}'
                | '\u{0C84}'
                | '\u{09F2}'
                | '\u{09F3}'
                | '\u{09FA}'
                | '\u{09FB}'
                | '\u{09FD}'
                | '\u{0B70}'
                | '\u{0BF3}'..='\u{0BFA}'
                | '\u{0D4F}'
                | '\u{0D79}'
                | '\u{0E2F}'
                | '\u{0E3F}'
                | '\u{0E4F}'
                | '\u{0E5A}'
                | '\u{0E5B}'
                | '\u{0EAF}'
                | '\u{0EDF}'
                | '\u{104A}'..='\u{104F}'
                | '\u{109E}'..='\u{109F}'
                | '\u{1735}'..='\u{1736}'
                | '\u{1A1E}'..='\u{1A1F}'
                | '\u{1AA0}'..='\u{1AA6}'
                | '\u{1AA8}'..='\u{1AAD}'
                | '\u{1CC0}'..='\u{1CC7}'
                | '\u{A4FE}'..='\u{A4FF}'
                | '\u{A60C}'..='\u{A60F}'
                | '\u{1400}'
                | '\u{166E}'
                | '\u{A6F2}'..='\u{A6F7}'
                | '\u{16A6E}'..='\u{16A6F}'
                | '\u{16B37}'..='\u{16B3B}'
                | '\u{16B44}'
                | '\u{19DE}'..='\u{19DF}'
                | '\u{A92E}'
                | '\u{A92F}'
                | '\u{A95F}'
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
                | '\u{A874}'..='\u{A877}'
                | '\u{A8CE}'..='\u{A8CF}'
                | '\u{A8F8}'..='\u{A8FA}'
                | '\u{A8FC}'
                | '\u{AA77}'..='\u{AA79}'
                | '\u{ABEB}'
                | '\u{1360}'..='\u{1368}'
                | '\u{1390}'..='\u{1399}'
                | '\u{2010}'..='\u{2015}'
                | '\u{2016}'..='\u{2018}'
                | '\u{201A}'..='\u{201F}'
                | '\u{2020}'..='\u{2023}'
                | '\u{2024}'..='\u{2027}'
                | '\u{2030}'..='\u{203B}'
                | '\u{203C}'..='\u{205E}'
                | '\u{207A}'..='\u{207E}'
                | '\u{208A}'..='\u{208E}'
                | '\u{2E00}'..='\u{2E2E}'
                | '\u{2E30}'..='\u{2E5D}'
                | '\u{169B}'
                | '\u{169C}'
                | '\u{16EB}'..='\u{16ED}'
                | '\u{10079}'..='\u{1007C}'
                | '\u{10100}'..='\u{10102}'
                | '\u{10137}'..='\u{1013F}'
                | '\u{10179}'..='\u{10189}'
                | '\u{1018C}'..='\u{1018E}'
                | '\u{10190}'..='\u{1019C}'
                | '\u{101A0}'
                | '\u{101D0}'..='\u{101FC}'
                | '\u{1039F}'
                | '\u{103D0}'
                | '\u{1056F}'
                | '\u{10857}'
                | '\u{10877}'
                | '\u{10878}'
                | '\u{1089F}'
                | '\u{108FB}'
                | '\u{1091F}'
                | '\u{1093F}'
                | '\u{10A7F}'
                | '\u{10A9D}'
                | '\u{10AC8}'
                | '\u{10AF0}'..='\u{10AF6}'
                | '\u{10A50}'..='\u{10A58}'
                | '\u{10B39}'..='\u{10B3F}'
                | '\u{10B99}'..='\u{10B9C}'
                | '\u{10F55}'..='\u{10F59}'
                | '\u{10F86}'..='\u{10F89}'
                | '\u{10ED0}'
                | '\u{1DA87}'..='\u{1DA8B}'
                | '\u{1E5FF}'
                | '\u{1ECAC}'
                | '\u{1ECB0}'
                | '\u{1ED2E}'
                | '\u{1EEF0}'
                | '\u{1EEF1}'
                | '\u{10D6E}'
                | '\u{10D8E}'
                | '\u{10D8F}'
                | '\u{10EAD}'
                | '\u{11B00}'..='\u{11B09}'
                | '\u{11BE1}'
                | '\u{16AF5}'
                | '\u{16D6D}'..='\u{16D6F}'
                | '\u{2440}'..='\u{245F}'
                | '\u{2460}'..='\u{24FF}'
                | '\u{1F000}'..='\u{1F02F}'
                | '\u{1F030}'..='\u{1F093}'
                | '\u{1F0A0}'..='\u{1F0FF}'
                | '\u{1F100}'..='\u{1F1AD}'
                | '\u{1F1E6}'..='\u{1F1FF}'
                | '\u{1F200}'..='\u{1F202}'
                | '\u{1F210}'..='\u{1F23B}'
                | '\u{1F240}'..='\u{1F248}'
                | '\u{1F250}'..='\u{1F251}'
                | '\u{1F260}'..='\u{1F265}'
                | '\u{1F300}'..='\u{1F67F}'
                | '\u{1F680}'..='\u{1F6D7}'
                | '\u{1F6E0}'..='\u{1F6EC}'
                | '\u{1F6F0}'..='\u{1F6FC}'
                | '\u{1F700}'..='\u{1F773}'
                | '\u{1F780}'..='\u{1F7D8}'
                | '\u{1F7E0}'..='\u{1F7EB}'
                | '\u{1F800}'..='\u{1F80B}'
                | '\u{1F810}'..='\u{1F847}'
                | '\u{1F850}'..='\u{1F859}'
                | '\u{1F860}'..='\u{1F887}'
                | '\u{1F890}'..='\u{1F8AD}'
                | '\u{1F8B0}'..='\u{1F8B1}'
                | '\u{1F900}'..='\u{1F9FF}'
                | '\u{1FA00}'..='\u{1FA57}'
                | '\u{1FA60}'..='\u{1FA6D}'
                | '\u{1FA70}'..='\u{1FA7C}'
                | '\u{1FA80}'..='\u{1FA8A}'
                | '\u{1FA8E}'..='\u{1FA8F}'
                | '\u{1FA90}'..='\u{1FAC6}'
                | '\u{1FAC8}'
                | '\u{1FACD}'..='\u{1FACF}'
                | '\u{1FAD0}'..='\u{1FADC}'
                | '\u{1FADF}'
                | '\u{1FAE0}'..='\u{1FAEA}'
                | '\u{1FAEF}'
                | '\u{1FAF0}'..='\u{1FAF8}'
                | '\u{1FB00}'..='\u{1FB92}'
                | '\u{1FB94}'..='\u{1FBEF}'
                | '\u{1CC00}'..='\u{1CCFC}'
                | '\u{1CD00}'..='\u{1CEB3}'
                | '\u{1CEBA}'..='\u{1CEBF}'
                | '\u{1CEC0}'..='\u{1CED0}'
                | '\u{1CEE0}'..='\u{1CEEF}'
                | '\u{1CEF0}'
                | '\u{1CF50}'..='\u{1CFC3}'
                | '\u{1D000}'..='\u{1D0F5}'
                | '\u{1D100}'..='\u{1D126}'
                | '\u{1D129}'..='\u{1D172}'
                | '\u{1D17B}'..='\u{1D1EA}'
                | '\u{1D200}'..='\u{1D245}'
                | '\u{1D300}'..='\u{1D356}'
                | '\u{4DC0}'..='\u{4DFF}'
                | '\u{A490}'..='\u{A4C6}'
                | '\u{A673}'
                | '\u{A67E}'
                | '\u{A828}'..='\u{A82B}'
                | '\u{A836}'..='\u{A839}'
                | '\u{1D6C1}'
                | '\u{1D6DB}'
                | '\u{1D6FB}'
                | '\u{1D715}'
                | '\u{1D735}'
                | '\u{1D74F}'
                | '\u{1D76F}'
                | '\u{1D789}'
                | '\u{1D7A9}'
                | '\u{1D7C3}'
                | '\u{1D800}'..='\u{1D9FF}'
                | '\u{1DA37}'..='\u{1DA3A}'
                | '\u{1DA6D}'..='\u{1DA74}'
                | '\u{1DA76}'..='\u{1DA83}'
                | '\u{1DA85}'..='\u{1DA86}'
                | '\u{2500}'..='\u{25FF}'
                | '\u{2600}'..='\u{26FF}'
                | '\u{2700}'..='\u{27BF}'
                | '\u{27C0}'..='\u{27FF}'
                | '\u{2800}'..='\u{28FF}'
                | '\u{2900}'..='\u{29FF}'
                | '\u{2A00}'..='\u{2AFF}'
                | '\u{2B00}'..='\u{2BFF}'
                | '\u{2CE5}'..='\u{2CEA}'
                | '\u{2CF9}'..='\u{2CFC}'
                | '\u{2CFE}'..='\u{2CFF}'
                | '\u{2D70}'
                | '\u{1BC9C}'
                | '\u{1BC9D}'..='\u{1BC9E}'
                | '\u{1BC9F}'
                | '\u{1BCA0}'..='\u{1BCA3}'
                | '\u{1107F}'
                | '\u{11047}'..='\u{1104D}'
                | '\u{110BB}'..='\u{110BC}'
                | '\u{110BE}'..='\u{110C1}'
                | '\u{110BD}'
                | '\u{110CD}'
                | '\u{11140}'..='\u{11143}'
                | '\u{111C5}'..='\u{111C8}'
                | '\u{111CD}'
                | '\u{111DB}'
                | '\u{111DD}'..='\u{111DF}'
                | '\u{11238}'..='\u{1123D}'
                | '\u{11174}'..='\u{11175}'
                | '\u{112A9}'
                | '\u{113D4}'..='\u{113D5}'
                | '\u{113D7}'..='\u{113D8}'
                | '\u{1144B}'..='\u{1144F}'
                | '\u{1145A}'
                | '\u{1145B}'
                | '\u{1145D}'
                | '\u{114C6}'
                | '\u{115C1}'..='\u{115D7}'
                | '\u{11641}'..='\u{11643}'
                | '\u{11660}'..='\u{1166C}'
                | '\u{116B9}'..='\u{116BC}'
                | '\u{1173C}'..='\u{1173F}'
                | '\u{1183B}'
                | '\u{11944}'..='\u{11946}'
                | '\u{119E2}'
                | '\u{11A3F}'..='\u{11A46}'
                | '\u{11A9A}'..='\u{11A9C}'
                | '\u{11A9E}'..='\u{11AA2}'
                | '\u{11C41}'..='\u{11C45}'
                | '\u{11C70}'..='\u{11C71}'
                | '\u{11EF7}'..='\u{11EF8}'
                | '\u{11F43}'..='\u{11F4F}'
                | '\u{11FD5}'..='\u{11FF1}'
                | '\u{11FFF}'
                | '\u{12470}'..='\u{12474}'
                | '\u{12FF1}'..='\u{12FF2}'
                | '\u{13430}'..='\u{13455}'
                | '\u{2FF0}'..='\u{2FFF}'
                | '\u{31EF}'
                | '\u{1D173}'..='\u{1D17A}'
                | '\u{FE00}'..='\u{FE0F}'
                | '\u{E0100}'..='\u{E01EF}'
                | '\u{E0000}'..='\u{E007F}'
                | '\u{2061}'..='\u{206F}'
                | '\u{FFF9}'..='\u{FFFD}' => ' ',
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
mod tests;
