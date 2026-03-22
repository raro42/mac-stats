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
                // decimal / thousands / five-pointed-star (U+066A–U+066D, Po) are not Rust whitespace; RTL or
                // bilingual numeric HTML can glue Latin tokens for `split_whitespace()`. Arabic-indic per mille /
                // per ten thousand (U+0609, U+060A, Po), Afghani sign (U+060B, Sc), date separator (U+060D, Po),
                // and triple-dot punctuation (U+061E, Po) are not Rust whitespace either; RTL financial or
                // editorial HTML can glue Latin tokens without ASCII space.
                // Armenian exclamation / comma / question / abbreviation mark (U+055C–U+055F, Po) and
                // full stop / hyphen (U+0589–U+058A, Po/Pd) are not Rust whitespace; bilingual or
                // Unicode-sample HTML can glue Latin tokens. U+055A (ARMENIAN APOSTROPHE) and U+055B
                // (EMPHASIS MARK, Po) are omitted—apostrophe- or stress-like marks can sit word-internally
                // in Armenian (same spirit as omitting U+2019 for Latin contractions).
                // Devanagari danda / double danda (U+0964, U+0965, Po) are not Rust whitespace; mixed
                // Latin–Devanagari or Unicode-sample HTML can place them between Latin tokens without ASCII space.
                // Gurmukhi abbreviation sign (U+0A76, Po), Gujarati abbreviation sign (U+0AF0, Po), Telugu sign
                // siddham (U+0C77, Po), and Kannada sign siddham (U+0C84, Po) are not Rust whitespace; South Indic
                // scholarly or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Bengali iss-har (U+09FA, So) and abbreviation sign (U+09FD, Po), Oriya isshar (U+0B70, So), Tamil
                // day through number signs (U+0BF3–U+0BFA, So/Sc), and Malayalam sign para / date mark (U+0D4F,
                // U+0D79, So) are not Rust whitespace; Eastern / Tamil / Malayalam mixed-script or Unicode-sample HTML
                // can glue Latin tokens without ASCII space.
                // Thai PAIYANNOI / FONGMAN / ANGKHANKHU / KHOMUT (U+0E2F, U+0E4F, U+0E5A, U+0E5B, Po), Lao
                // ELLIPSIS (U+0EAF, Po), Myanmar LITTLE SECTION / SECTION (U+104A–U+104B, Po), and Myanmar symbols
                // LOCATIVE / EXCLAMATION / COMPLETED / AFOREMENTIONED (U+104C–U+104F, So) are not Rust whitespace;
                // Southeast Asian–Latin bilingual or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Thai MAIYAMOK (U+0E46, Lm) stays unmapped—modifier-like, can repeat word-internally.
                // Arabic number signs / ayah markers (U+0600–U+0605, U+06DD, U+08E2), Arabic
                // Extended-A currency format marks (U+0890–U+0891, pound/piastre mark above), Syriac
                // end of paragraph (U+0700, Bk) and sentence punctuation (U+0701–U+070D, Po), and
                // Syriac abbreviation mark (U+070F) are not Rust whitespace (070F is Cf); RTL scholarly
                // or financial HTML can place them between scripts without a real space. NKo GBAKURUNEN
                // symbol / comma / exclamation (U+07F7–U+07F9, Po) are not Rust whitespace; U+07FA
                // LAJANYALAN (Lm) omitted—modifier-like, word-internal risk. Samaritan punctuation
                // marks (U+0830–U+083E, Po) are not Rust whitespace; Hebrew/Samaritan scholarly or
                // Unicode-sample HTML can glue Latin tokens without ASCII space. Nyiakeng Puachue
                // Hmong exclamation / question (U+16FE2, U+16FE3, Po) are not Rust whitespace;
                // U+16FE0–U+16FE1 (Lo/Lm) and U+16FE4 LOGOGRAM NYIAM (Lo) stay unmapped. Wancho
                // comma / full stop (U+1E2FE–U+1E2FF, Po) are not Rust whitespace; Northeast-Indian
                // script or Unicode-sample HTML can glue Latin tokens without ASCII space. Adlam
                // initial exclamation / question (U+1E95E, U+1E95F, Po) are not Rust whitespace;
                // Adlam–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Medefaidrin comma / full stop / exclamation oh (U+16E97, U+16E98, U+16E9A, Po) are
                // not Rust whitespace; U+16E99 SYMBOL AIVA (So) stays unmapped. Mongolian
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
                // Ideographic comma / full stop (U+3001, U+3002, Po) and
                // fullwidth ASCII-like punctuation (U+FF0C comma, U+FF1A colon, U+FF1B semicolon,
                // U+FF01 exclamation, U+FF1F question; all Po) are not Rust whitespace; CJK or
                // mixed-layout HTML often places them between Latin tokens without ASCII space.
                // Other Halfwidth and Fullwidth Forms delimiters—U+FF03 number sign, U+FF04 dollar,
                // U+FF05 percent, U+FF06 ampersand, U+FF08/U+FF09 parens, U+FF0A asterisk, U+FF0B plus,
                // U+FF0D hyphen-minus, U+FF0E full stop, U+FF1C–U+FF1E relations, U+FF20 commercial at,
                // U+FF3B–U+FF3D brackets and reverse solidus, U+FF5B–U+FF5D braces, U+FF5C vertical line
                // (Po/Ps/Pe/Sm/Pd/Sc as assigned)—are not Rust whitespace either. U+FF07 FULLWIDTH
                // APOSTROPHE (Po) stays unmapped—typographic apostrophe risk. U+FF5E FULLWIDTH TILDE (Sm)
                // stays unmapped—Japanese range notation (e.g. 3～5). U+FF40 FULLWIDTH GRAVE (Po) stays
                // unmapped—modifier-like, word-internal risk. Fullwidth white parentheses (U+FF5F, U+FF60,
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
                // U+1945, Po), Meetei Mayek cheikhei / ahang khuda (U+AAF0, U+AAF1, Po), and Meetei Mayek cheikhei (U+ABEB, Po) are not
                // Rust whitespace; mixed-script or Unicode-sample HTML can glue Latin tokens without ASCII space. Ethiopic full stop
                // (U+1362, Po) is not Rust whitespace; mixed-script HTML can glue Latin tokens. Khmer signs khan through koomuut
                // (U+17D4–U+17DA, Po) are not Rust whitespace;
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
                // Philippine single / double / triple punctuation (U+1734–U+1736, Po; Hanunoo / Buhid / Tagbanwa) are
                // not Rust whitespace; Unicode-sample or Philippine-script HTML can glue Latin tokens without ASCII space.
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
                // Canadian Syllabics full stop (U+166E, Po) is not Rust whitespace; U+166D CHI SIGN (So) stays unmapped—
                // syllabics-internal risk. Bamum sentence punctuation (U+A6F2–U+A6F7, Po), Mro danda / double danda
                // (U+16A6E–U+16A6F, Po), and New Tai Lue signs lae / laev (U+19DE–U+19DF, So) are not Rust whitespace;
                // Aboriginal / Bamum / Mro / New Tai Lue–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Buginese pallawa and end-of-section (U+1A1E, U+1A1F, Po) are not Rust whitespace; Buginese–Latin or
                // Unicode-sample HTML can glue Latin tokens without ASCII space. Tai Tham signs wiang through reversed
                // rotated rana (U+1AA0–U+1AA6, Po) and kaan through caang (U+1AA8–U+1AAD, Po) are not Rust whitespace;
                // Lanna–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space. U+1AA7 MAI YAMOK (Lm) is
                // omitted—modifier-like, word-internal risk (same spirit as Thai U+0E46).
                // Currency Symbols U+20A0–U+20BF (all Sc) are not Rust whitespace; multilingual
                // price copy or Unicode-sample HTML can place euro, rupee, bitcoin, etc. between
                // Latin tokens without ASCII space. U+20C0–U+20CF are unassigned and stay unmapped.
                // Number Forms U+2150–U+2182 and U+2185–U+218B (vulgar fractions, Roman
                // numerals, turned digit two/three; all No / Nl / So) are not Rust whitespace;
                // typography or Unicode-sample HTML can place ⅓, Ⅷ, vulgar fraction zero thirds
                // (U+2189), turned digits, etc. between Latin tokens
                // without ASCII space. U+2183 ROMAN NUMERAL REVERSED ONE HUNDRED (Lu) and U+2184
                // LATIN SMALL LETTER REVERSED C (Ll) stay unmapped—letter-like, word-internal risk.
                // U+218C–U+218F are unassigned and stay unmapped.
                // Arrows U+2190–U+21FF (mostly Sm) are not Rust whitespace; MathML, diagram, or
                // Unicode-sample HTML can place arrows between Latin tokens without ASCII space.
                // Number Forms fractions / Roman numerals (U+2150–U+2182, U+2185–U+218B; includes U+2189) sit
                // after Currency Symbols and before Arrows; U+2183/U+2184 letter-like scalars stay unmapped.
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
                // or Unicode-sample HTML can glue Latin tokens for `split_whitespace()`. Ogham feather
                // mark and reversed feather mark (U+169B, U+169C, Ps/Pe) are not Rust whitespace;
                // epigraphic Ogham–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Duployan CHINOOK FULL STOP (U+1BC9F, Po) is visible sentence punctuation, not Rust
                // whitespace; shorthand HTML can glue Latin tokens (U+1BC9D–U+1BC9E Mn and U+1BCA0–U+1BCA3
                // Cf are mapped separately). Tifinagh separator mark (U+2D70, Po) is not Rust whitespace;
                // Berber / Unicode-sample HTML can glue Latin tokens without ASCII space. Aegean word
                // separator line / dot / check mark (U+10100–U+10102, Po) and Phoenician word separator (U+1091F,
                // Po) are not Rust whitespace; scholarly or mixed-script HTML can place them between
                // Latin tokens without ASCII space. Ugaritic word divider (U+1039F), Old Persian word
                // divider (U+103D0), Caucasian Albanian citation mark (U+1056F), Imperial Aramaic section
                // sign (U+10857), Lydian triangular mark (U+1093F), Old South Arabian numeric indicator
                // (U+10A7F), and Manichaean punctuation star through line filler (U+10AF0–U+10AF6, Po)
                // are not Rust whitespace; epigraphic or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Palmyrene left- and right-pointing fleurons (U+10877, U+10878, So) are not Rust whitespace;
                // epigraphic Palmyrene–Latin or Unicode-sample HTML can place them between tokens without ASCII space.
                // Pahawh Hmong clause and sentence signs (U+16B37–U+16B3B, U+16B44, Po) are not Rust whitespace;
                // U+16B30–U+16B36 (Mn), U+16B40–U+16B43 (Lm), and U+16B3C–U+16B3F / U+16B45 (So) stay unmapped.
                // Nabataean word separator (U+1089F, Po) and Hatran section mark (U+108FB, Po) are not Rust
                // whitespace; epigraphic or Unicode-sample HTML can place them between Latin tokens without ASCII space.
                // Old North Arabian word divider (U+10A9D, Po) is not Rust whitespace; U+10A9E / U+10A9F (Nl) stay
                // unmapped. Elymaic section mark (U+10FF5, Po) is not Rust whitespace; Unicode-sample HTML can glue
                // Latin tokens without ASCII space.
                // Hanifi Rohingya signs tana / penda / dotted variants / jaha (U+10D29–U+10D2D, Po) are not Rust
                // whitespace; U+10D2E SIGN VIRAMA (Mn) stays unmapped—word-internal risk.
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
                // Vithkuqi comma through question mark (U+1057B–U+1057F, Po) are not Rust whitespace; Albanian Vithkuqi
                // or Unicode-sample HTML can place them between Latin tokens without ASCII space. U+10570–U+1057A (Lo)
                // stay unmapped—letters, word-internal risk.
                // Warang Citi danda through section mark (U+118C8–U+118CF, Po) are not Rust whitespace; Austroasiatic
                // or Unicode-sample HTML can glue Latin tokens without ASCII space. U+118D0 NUMBER ZERO (Nl) starts
                // the numeric subrange and is not included.
                // Devanagari Extended-A head marks and bhale signs (U+11B00–U+11B09, Po) are not Rust whitespace;
                // editorial Devanagari–Latin or Unicode-sample HTML can glue Latin tokens without ASCII space.
                // Sunuwar sign pvo (U+11BE1, Po) is not Rust whitespace; Nepal–Unicode-sample HTML can glue Latin
                // tokens without ASCII space.
                // Bassa Vah full stop (U+16AF5, Po) is not Rust whitespace; Liberian / Unicode-sample HTML can glue
                // Latin tokens without ASCII space.
                // Kirat Rai sign yupi, danda, double danda (U+16D6D–U+16D6F, Po) are not Rust whitespace;
                // U+16D6B SIGN VIRAMA and U+16D6C SIGN SAAT (Lm) stay unmapped—modifier-like, word-internal risk.
                // Ethiopic wordspace (U+1361, Po) is not Rust whitespace. Braille Patterns (U+2800–U+28FF;
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
                // Duployan thick letter selector / double mark (U+1BC9D–U+1BC9E, Mn), CHINOOK FULL
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
                // U+2FFB, So) are not Rust whitespace; Han-component notation or pasted CJK-scholarly
                // HTML can glue Latin tokens. Chakma section mark and sentence punctuation (U+11140–
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
                // virama risks. Ahom small section / section / rulai (U+1173C–U+1173E, Po) are not
                // Rust whitespace. Dogra abbreviation sign (U+1183B, Po) is not Rust whitespace.
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
                // Tamil Supplement END OF TEXT (U+11FFF, Po) is not Rust whitespace; mixed Tamil–Latin
                // or Unicode-sample HTML can glue Latin tokens without ASCII space.
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
                | '\u{07F7}'..='\u{07F9}'
                | '\u{0830}'..='\u{083E}'
                | '\u{16FE2}'
                | '\u{16FE3}'
                | '\u{1E2FE}'..='\u{1E2FF}'
                | '\u{16E97}'
                | '\u{16E98}'
                | '\u{16E9A}'
                | '\u{1E95E}'..='\u{1E95F}'
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
                | '\u{20A0}'..='\u{20BF}'
                | '\u{2150}'..='\u{2182}'
                | '\u{2185}'..='\u{218B}'
                | '\u{2190}'..='\u{21FF}'
                | '\u{2200}'..='\u{22FF}'
                | '\u{2300}'..='\u{23FF}'
                | '\u{2400}'..='\u{243F}'
                | '\u{037E}'
                | '\u{0609}'
                | '\u{060A}'
                | '\u{060B}'
                | '\u{060C}'
                | '\u{060D}'
                | '\u{061B}'
                | '\u{061E}'
                | '\u{061F}'
                | '\u{06D4}'
                | '\u{066A}'..='\u{066D}'
                | '\u{055C}'..='\u{055F}'
                | '\u{0589}'
                | '\u{058A}'
                | '\u{0964}'
                | '\u{0965}'
                | '\u{0A76}'
                | '\u{0AF0}'
                | '\u{0C77}'
                | '\u{0C84}'
                | '\u{09FA}'
                | '\u{09FD}'
                | '\u{0B70}'
                | '\u{0BF3}'..='\u{0BFA}'
                | '\u{0D4F}'
                | '\u{0D79}'
                | '\u{0E2F}'
                | '\u{0E4F}'
                | '\u{0E5A}'
                | '\u{0E5B}'
                | '\u{0EAF}'
                | '\u{104A}'..='\u{104F}'
                | '\u{1734}'..='\u{1736}'
                | '\u{1A1E}'..='\u{1A1F}'
                | '\u{1AA0}'..='\u{1AA6}'
                | '\u{1AA8}'..='\u{1AAD}'
                | '\u{1CC0}'..='\u{1CC7}'
                | '\u{A4FE}'..='\u{A4FF}'
                | '\u{A60C}'..='\u{A60F}'
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
                | '\u{1362}'
                | '\u{2010}'..='\u{2015}'
                | '\u{2016}'..='\u{2018}'
                | '\u{201A}'..='\u{201F}'
                | '\u{2020}'..='\u{2023}'
                | '\u{2024}'..='\u{2027}'
                | '\u{2030}'..='\u{203B}'
                | '\u{203C}'..='\u{205E}'
                | '\u{2E00}'..='\u{2E5D}'
                | '\u{169B}'
                | '\u{169C}'
                | '\u{16EB}'..='\u{16ED}'
                | '\u{10079}'..='\u{1007C}'
                | '\u{10100}'..='\u{10102}'
                | '\u{1039F}'
                | '\u{103D0}'
                | '\u{1056F}'
                | '\u{1057B}'..='\u{1057F}'
                | '\u{10857}'
                | '\u{10877}'
                | '\u{10878}'
                | '\u{1089F}'
                | '\u{108FB}'
                | '\u{1091F}'
                | '\u{1093F}'
                | '\u{10A7F}'
                | '\u{10A9D}'
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
                | '\u{10FF5}'
                | '\u{10D29}'..='\u{10D2D}'
                | '\u{10D6E}'
                | '\u{10D8E}'
                | '\u{10D8F}'
                | '\u{10EAD}'
                | '\u{118C8}'..='\u{118CF}'
                | '\u{11B00}'..='\u{11B09}'
                | '\u{11BE1}'
                | '\u{16AF5}'
                | '\u{16D6D}'..='\u{16D6F}'
                | '\u{1361}'
                | '\u{2440}'..='\u{245F}'
                | '\u{2460}'..='\u{24FF}'
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
                | '\u{1173C}'..='\u{1173E}'
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
                | '\u{11FFF}'
                | '\u{12470}'..='\u{12474}'
                | '\u{12FF1}'..='\u{12FF2}'
                | '\u{13430}'..='\u{13455}'
                | '\u{2FF0}'..='\u{2FFB}'
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
    fn interlinear_annotation_object_replacement_and_replacement_char_separate_words() {
        // U+FFF9–U+FFFB (interlinear annotation anchor/separator/terminator), U+FFFC (object
        // replacement), and U+FFFD (replacement character, So) are not Rust whitespace; they can
        // appear in copied or transcoded HTML.
        for sep in ['\u{FFF9}', '\u{FFFA}', '\u{FFFB}', '\u{FFFC}', '\u{FFFD}'] {
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
    fn syriac_paragraph_end_and_sentence_punctuation_separate_words() {
        // U+0700 (END OF PARAGRAPH, Bk) and U+0701–U+070D (supralinear/sublinear stops, colons,
        // dots, Harklean marks; Po)—not Rust whitespace; Syriac–Latin or Unicode-sample HTML can glue
        // Latin tokens for `split_whitespace()`. U+070E unassigned; U+070F tested in
        // `arabic_and_syriac_edition_format_separate_words`.
        for cp in 0x0700u32..=0x070D {
            let sep = char::from_u32(cp).unwrap();
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
    fn nko_sentence_punctuation_and_samaritan_punctuation_separate_words() {
        // NKo U+07F7–U+07F9 (SYMBOL GBAKURUNEN, COMMA, EXCLAMATION MARK; Po)—not Rust whitespace.
        // Samaritan U+0830–U+083E (punctuation marks through ANNAAU; Po)—not Rust whitespace.
        // U+083F unassigned. U+07FA LAJANYALAN (Lm) omitted.
        for cp in [0x07F7u32, 0x07F8, 0x07F9]
            .into_iter()
            .chain(0x0830u32..=0x083E)
        {
            let sep = char::from_u32(cp).unwrap();
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
    fn adlam_initial_punctuation_and_medefaidrin_sentence_marks_separate_words() {
        // Adlam U+1E95E–U+1E95F (INITIAL EXCLAMATION / QUESTION; Po)—not Rust whitespace.
        // Medefaidrin U+16E97–U+16E98, U+16E9A (COMMA, FULL STOP, EXCLAMATION OH; Po)—not Rust
        // whitespace. U+16E99 SYMBOL AIVA (So) omitted.
        for cp in [0x16E97u32, 0x16E98, 0x16E9A]
            .into_iter()
            .chain(0x1E95Eu32..=0x1E95F)
        {
            let sep = char::from_u32(cp).unwrap();
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
    fn wancho_comma_full_stop_and_nyiakeng_sentence_punctuation_separate_words() {
        // Wancho U+1E2FE–U+1E2FF (COMMA, FULL STOP; Po)—not Rust whitespace.
        // Nyiakeng Puachue Hmong U+16FE2–U+16FE3 (EXCLAMATION MARK, QUESTION MARK; Po)—not Rust whitespace.
        for cp in [0x16FE2u32, 0x16FE3].into_iter().chain(0x1E2FEu32..=0x1E2FF) {
            let sep = char::from_u32(cp).unwrap();
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
    fn currency_symbols_u20a0_through_u20bf_separate_words() {
        // Currency Symbols U+20A0–U+20BF (all Sc); not Rust whitespace—price or Unicode-sample HTML
        // can glue Latin tokens without ASCII space.
        for sep in '\u{20A0}'..='\u{20BF}' {
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
    fn number_forms_fractions_romans_and_turned_digits_separate_words() {
        // U+2150–U+2182, U+2185–U+218B: vulgar fractions, Roman numerals, late forms, vulgar fraction
        // zero thirds (U+2189), turned digits (No / Nl / So); not Rust whitespace. U+2183 (Lu) / U+2184 (Ll) omitted in implementation.
        let cps = (0x2150u32..=0x2182).chain(0x2185..=0x218B);
        for cp in cps {
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
    fn number_forms_reversed_roman_c_and_reversed_c_letter_stay_unmapped() {
        // U+2183 / U+2184 are Lu/Ll—letter-like; must not force a word break like vulgar fractions.
        for cp in [0x2183u32, 0x2184] {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} should not map to ASCII space between Latin tokens, got {:?}",
                cp,
                cleaned
            );
        }
    }

    #[test]
    fn arrows_u2190_through_u21ff_separate_words() {
        // Arrows U+2190–U+21FF (mostly Sm); not Rust whitespace—MathML / diagram / Unicode-sample HTML
        // can glue Latin tokens without ASCII space.
        for cp in 0x2190u32..=0x21FF {
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
    fn mathematical_operators_u2200_through_u22ff_separate_words() {
        // Mathematical Operators U+2200–U+22FF (mostly Sm); not Rust whitespace—MathML or Unicode-sample
        // HTML can glue Latin tokens without ASCII space.
        for cp in 0x2200u32..=0x22FF {
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
    fn miscellaneous_technical_u2300_through_u23ff_separate_words() {
        // Miscellaneous Technical U+2300–U+23FF (mostly So / Sm / Po); not Rust whitespace—technical,
        // APL, or Unicode-sample HTML can glue Latin tokens without ASCII space.
        for cp in 0x2300u32..=0x23FF {
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
    fn control_pictures_u2400_through_u243f_separate_words() {
        // Control Pictures U+2400–U+243F (all So); not Rust whitespace—terminal dumps, legacy copy, or Unicode-sample HTML
        // can glue Latin tokens without ASCII space. U+2440+ OCR block is covered by `ocr_symbols_separate_words`.
        for cp in 0x2400u32..=0x243F {
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
        // U+066A–U+066D (Arabic percent, decimal sep, thousands sep, five pointed star): Po / Pi / Pf; not Rust
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
            '\u{066D}',
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
        // U+10100 / U+10101 / U+10102 (Aegean word separator line/dot/check mark, Po) and U+1091F (Phoenician word
        // separator, Po) are not Rust whitespace.
        for sep in ['\u{10100}', '\u{10101}', '\u{10102}', '\u{1091F}'] {
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
    fn hanifi_rohingya_sentence_marks_and_warang_citi_punctuation_separate_words() {
        // Hanifi Rohingya U+10D29–U+10D2D (tana / penda / dotted / jaha, all Po). U+10D2E VIRAMA (Mn) omitted.
        // Warang Citi U+118C8–U+118CF (danda through section mark, all Po).
        for sep in (0x10D29u32..=0x10D2D).chain(0x118C8..=0x118CF) {
            let sep = char::from_u32(sep).expect("valid test scalar");
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
    fn garay_hyphen_plus_and_minus_separate_words() {
        // Garay U+10D6E HYPHEN (Pd). Garay U+10D8E PLUS SIGN / U+10D8F MINUS SIGN (Sm).
        for sep in [0x10D6Eu32, 0x10D8E, 0x10D8F] {
            let sep = char::from_u32(sep).expect("valid test scalar");
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
    fn vithkuqi_sentence_punctuation_separate_words() {
        // Vithkuqi U+1057B COMMA through U+1057F QUESTION MARK (all Po).
        for sep in '\u{1057B}'..='\u{1057F}' {
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
    fn yezidi_hyphenation_and_indic_siyaq_placeholder_rupee_separate_words() {
        // Yezidi U+10EAD HYPHENATION MARK (Pd). Indic Siyaq U+1ECAC PLACEHOLDER (So), U+1ECB0 RUPEE MARK (Sc).
        for sep in ['\u{10EAD}', '\u{1ECAC}', '\u{1ECB0}'] {
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
    fn arabic_biblical_end_of_verse_signwriting_and_ol_onal_punctuation_separate_words() {
        // Arabic Extended-C U+10ED0 END OF VERSE (Po). SignWriting U+1DA87–U+1DA8B (Po). Ol Onal U+1E5FF (Po).
        let mut seps: Vec<char> = vec![char::from_u32(0x10ED0).unwrap(), char::from_u32(0x1E5FF).unwrap()];
        seps.extend('\u{1DA87}'..='\u{1DA8B}');
        for sep in seps {
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
    fn devanagari_extended_a_and_bassa_vah_sentence_punctuation_separate_words() {
        // Devanagari Extended-A U+11B00 HEAD MARK through U+11B09 SIGN MINDU (all Po). Bassa Vah U+16AF5 FULL STOP (Po).
        for sep in (0x11B00u32..=0x11B09).chain(std::iter::once(0x16AF5)) {
            let sep = char::from_u32(sep).expect("valid test scalar");
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
    fn sunuwar_pvo_and_kirat_rai_sentence_punctuation_separate_words() {
        // Sunuwar U+11BE1 SIGN PVO (Po). Kirat Rai U+16D6D SIGN YUPI, U+16D6E DANDA, U+16D6F DOUBLE DANDA (all Po).
        // U+16D6B SIGN VIRAMA / U+16D6C SIGN SAAT (Lm) omitted.
        for sep in std::iter::once(0x11BE1u32).chain(0x16D6D..=0x16D6F) {
            let sep = char::from_u32(sep).expect("valid test scalar");
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
    fn nabataean_hatran_old_north_arabian_elymaic_word_separators_separate_words() {
        // Nabataean U+1089F WORD SEPARATOR, Hatran U+108FB SECTION MARK, Old North Arabian U+10A9D WORD DIVIDER,
        // Elymaic U+10FF5 SECTION MARK (all Po). U+10A9E / U+10A9F (Old North Arabian numbers, Nl) stay unmapped.
        for sep in ['\u{1089F}', '\u{108FB}', '\u{10A9D}', '\u{10FF5}'] {
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
    fn palmyrene_fleurons_and_pahawh_hmong_clause_signs_separate_words() {
        // Palmyrene U+10877 / U+10878 (left/right-pointing fleuron, So). Pahawh Hmong U+16B37–U+16B3B and
        // U+16B44 (clause/sentence signs, Po). None are Rust whitespace.
        for sep in [
            '\u{10877}',
            '\u{10878}',
            '\u{16B37}',
            '\u{16B38}',
            '\u{16B39}',
            '\u{16B3A}',
            '\u{16B3B}',
            '\u{16B44}',
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
    fn ancient_word_dividers_and_manichaean_punctuation_separate_words() {
        // Ugaritic U+1039F; Old Persian U+103D0; Caucasian Albanian U+1056F; Imperial Aramaic U+10857;
        // Lydian U+1093F; Old South Arabian U+10A7F; Manichaean U+10AF0 PUNCTUATION STAR through
        // U+10AF6 PUNCTUATION LINE FILLER (all Po).
        for cp in std::iter::once(0x1039Fu32)
            .chain(std::iter::once(0x103D0))
            .chain(std::iter::once(0x1056F))
            .chain(std::iter::once(0x10857))
            .chain(std::iter::once(0x1093F))
            .chain(std::iter::once(0x10A7F))
            .chain(0x10AF0..=0x10AF6)
        {
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
    fn fullwidth_delimiters_operators_and_brackets_separate_words() {
        // U+FF03 NUMBER SIGN, U+FF04 DOLLAR SIGN, U+FF05 PERCENT, U+FF06 AMPERSAND, U+FF08/U+FF09
        // PARENS, U+FF0A ASTERISK, U+FF0B PLUS, U+FF0D HYPHEN-MINUS, U+FF0E FULL STOP,
        // U+FF1C–U+FF1E LESS/EQUALS/GREATER, U+FF20 COMMERCIAL AT, U+FF3B–U+FF3D BRACKETS/SOLIDUS,
        // U+FF5B–U+FF5D CURLY BRACES, U+FF5C VERTICAL LINE—not Rust whitespace; fullwidth
        // typography HTML can glue Latin tokens without ASCII space. U+FF07 apostrophe, U+FF5E tilde,
        // U+FF40 grave intentionally omitted (see `collapse_whitespace` comment).
        for sep in [
            '\u{FF03}',
            '\u{FF04}',
            '\u{FF05}',
            '\u{FF06}',
            '\u{FF08}',
            '\u{FF09}',
            '\u{FF0A}',
            '\u{FF0B}',
            '\u{FF0D}',
            '\u{FF0E}',
            '\u{FF1C}',
            '\u{FF1D}',
            '\u{FF1E}',
            '\u{FF20}',
            '\u{FF3B}',
            '\u{FF3C}',
            '\u{FF3D}',
            '\u{FF5B}',
            '\u{FF5C}',
            '\u{FF5D}',
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
    fn fullwidth_white_parens_halfwidth_cjk_punct_and_fullwidth_currency_separate_words() {
        // U+FF5F/U+FF60 WHITE PARENTHESIS (Ps/Pe), U+FF61 HALFWIDTH IDEOGRAPHIC FULL STOP,
        // U+FF62/U+FF63 CORNER BRACKETS (Ps/Pe), U+FF64 HALFWIDTH IDEOGRAPHIC COMMA (Po)—not Rust
        // whitespace; U+FF65 KATAKANA MIDDLE DOT stays on the middle-dot arm. U+FFE0–U+FFE2 cent /
        // pound / not (Sc/Sm), U+FFE4 broken bar (So), U+FFE5–U+FFE6 yen / won (Sc)—not Rust
        // whitespace. U+FFE3 FULLWIDTH MACRON (Sk) intentionally omitted (overline / word-internal risk).
        for sep in [
            '\u{FF5F}',
            '\u{FF60}',
            '\u{FF61}',
            '\u{FF62}',
            '\u{FF63}',
            '\u{FF64}',
            '\u{FFE0}',
            '\u{FFE1}',
            '\u{FFE2}',
            '\u{FFE4}',
            '\u{FFE5}',
            '\u{FFE6}',
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
    fn halfwidth_forms_light_vertical_arrows_square_circle_small_square_separate_words() {
        // U+FFE8 HALFWIDTH FORMS LIGHT VERTICAL (So), U+FFE9–U+FFEC halfwidth arrows (Sm), U+FFED /
        // U+FFEE halfwidth black square / white circle (So), U+FFEF halfwidth black small square (So)—
        // not Rust whitespace; U+FFE3 FULLWIDTH MACRON (Sk) stays unmapped (between U+FFE2 and U+FFE4
        // in the block, not in this arm). Completes Halfwidth and Fullwidth Forms through U+FFEF.
        for sep in [
            '\u{FFE8}', '\u{FFE9}', '\u{FFEA}', '\u{FFEB}', '\u{FFEC}', '\u{FFED}', '\u{FFEE}', '\u{FFEF}',
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
    fn phags_pa_saurashtra_devanagari_ext_myanmar_aiton_meetei_cheikhei_separate_words() {
        // Phags-pa U+A874–U+A877 (single/double head mark, shad, double shad; Po). Saurashtra U+A8CE DANDA, U+A8CF DOUBLE DANDA (Po).
        // Devanagari Extended U+A8F8 PUSHPIKA, U+A8F9 GAP FILLER, U+A8FA CARET, U+A8FC SIGN SIDDHAM (Po). Myanmar Extended-A U+AA77–U+AA79
        // Aiton exclamation / one / two (So). Meetei Mayek U+ABEB CHEIKHEI (Po). U+A8FB HEADSTROKE (Lo) omitted. None are Rust whitespace.
        let mut seps: Vec<char> = ('\u{A874}'..='\u{A877}').collect();
        seps.extend('\u{A8CE}'..='\u{A8CF}');
        seps.extend('\u{A8F8}'..='\u{A8FA}');
        seps.push('\u{A8FC}');
        seps.extend('\u{AA77}'..='\u{AA79}');
        seps.push('\u{ABEB}');
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
    fn ogham_feather_marks_and_duployan_chinook_full_stop_separate_words() {
        // Ogham U+169B FEATHER MARK (Ps), U+169C REVERSED FEATHER MARK (Pe). Duployan U+1BC9F CHINOOK FULL
        // STOP (Po). None are Rust whitespace.
        for sep in ['\u{169B}', '\u{169C}', '\u{1BC9F}'] {
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
    fn tifinagh_separator_and_arabic_supplementary_punctuation_separate_words() {
        // Tifinagh U+2D70 SEPARATOR MARK (Po). Arabic U+0609 / U+060A per mille / per ten thousand (Po),
        // U+060B AFGHANI SIGN (Sc), U+060D DATE SEPARATOR (Po), U+061E TRIPLE DOT PUNCTUATION (Po),
        // U+066D FIVE POINTED STAR (Po). None are Rust whitespace.
        for sep in [
            '\u{2D70}',
            '\u{0609}',
            '\u{060A}',
            '\u{060B}',
            '\u{060D}',
            '\u{061E}',
            '\u{066D}',
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
    fn coptic_full_stop_morphological_divider_and_linear_b_word_separators_separate_words() {
        // Coptic symbols U+2CE5–U+2CEA (So); Old Nubian U+2CF9–U+2CFC (Po); full stop / morphological divider U+2CFE–U+2CFF
        // (Po). Linear B U+10079–U+1007C word separator dot / small / medium / large (Po). None are Rust whitespace.
        let mut seps: Vec<char> = ('\u{2CE5}'..='\u{2CEA}').collect();
        seps.extend('\u{2CF9}'..='\u{2CFC}');
        seps.extend('\u{2CFE}'..='\u{2CFF}');
        seps.extend('\u{10079}'..='\u{1007C}');
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
    fn coptic_fraction_one_half_does_not_split_words() {
        // U+2CFD COPTIC FRACTION ONE HALF (No)—numeric; must not be treated as a word separator.
        let sep = '\u{2CFD}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+2CFD should not normalize to space, got {:?}",
            cleaned
        );
        assert!(
            cleaned.contains(sep),
            "expected U+2CFD preserved in output, got {:?}",
            cleaned
        );
    }

    #[test]
    fn bengali_oriya_tamil_malayalam_issher_abbrev_and_financial_signs_separate_words() {
        // Bengali U+09FA ISSHAR (So), U+09FD ABBREVIATION SIGN (Po); Oriya U+0B70 ISSHAR (So); Tamil U+0BF3–U+0BFA
        // (day/month/year/debit/credit/as above/rupee/number; So/Sc); Malayalam U+0D4F SIGN PARA, U+0D79 DATE MARK
        // (So). None are Rust whitespace.
        for cp in [0x09FAu32, 0x09FD, 0x0B70, 0x0D4F, 0x0D79]
            .into_iter()
            .chain(0x0BF3..=0x0BFA)
        {
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
    fn gurmukhi_gujarati_telugu_kannada_abbreviation_siddham_separate_words() {
        // Gurmukhi U+0A76 ABBREVIATION SIGN, Gujarati U+0AF0 ABBREVIATION SIGN, Telugu U+0C77 SIGN SIDDHAM,
        // Kannada U+0C84 SIGN SIDDHAM (all Po); not Rust whitespace.
        for sep in ['\u{0A76}', '\u{0AF0}', '\u{0C77}', '\u{0C84}'] {
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
        // (Po). Myanmar: U+104A LITTLE SECTION, U+104B SECTION (Po); U+104C–U+104F locative / exclamation /
        // completed / aforementioned (So). None are Rust whitespace.
        for cp in [0x0E2Fu32, 0x0E4F, 0x0E5A, 0x0E5B, 0x0EAF].into_iter().chain(0x104A..=0x104F) {
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
    fn philippine_single_double_triple_punctuation_separate_words() {
        // U+1734 PHILIPPINE SINGLE PUNCTUATION, U+1735 DOUBLE, U+1736 TRIPLE (Po). None are Rust whitespace.
        for sep in '\u{1734}'..='\u{1736}' {
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
    fn sundanese_supplement_bindu_punctuation_separate_words() {
        // U+1CC0–U+1CC7 SUNDANESE PUNCTUATION BINDU SURYA through BINDU PAMENENG (Po). None are Rust whitespace.
        for sep in '\u{1CC0}'..='\u{1CC7}' {
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
    fn canadian_syllabics_full_stop_bamum_mro_and_new_tai_lue_signs_separate_words() {
        // U+166E CANADIAN SYLLABICS FULL STOP (Po). Bamum U+A6F2 NJAEMLI through U+A6F7 QUESTION MARK (Po).
        // Mro U+16A6E DANDA, U+16A6F DOUBLE DANDA (Po). New Tai Lue U+19DE SIGN LAE, U+19DF SIGN LAEV (So). None are Rust whitespace.
        let mut seps: Vec<char> = vec!['\u{166E}'];
        seps.extend('\u{A6F2}'..='\u{A6F7}');
        seps.extend('\u{16A6E}'..='\u{16A6F}');
        seps.extend('\u{19DE}'..='\u{19DF}');
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
    fn lisu_comma_full_stop_and_vai_sentence_punctuation_separate_words() {
        // Lisu U+A4FE PUNCTUATION COMMA, U+A4FF PUNCTUATION FULL STOP (Po). Vai U+A60C COMMA through U+A60F EXCLAMATION MARK (Po).
        let mut seps: Vec<char> = ('\u{A4FE}'..='\u{A4FF}').collect();
        seps.extend('\u{A60C}'..='\u{A60F}');
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
    fn kayah_li_cwi_shya_and_rejang_section_mark_separate_words() {
        // Kayah Li U+A92E SIGN CWI, U+A92F SIGN SHYA (Po). Rejang U+A95F SECTION MARK (Po). None are Rust whitespace.
        for sep in ['\u{A92E}', '\u{A92F}', '\u{A95F}'] {
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
    fn buginese_pallawa_end_section_and_tai_tham_signs_separate_words() {
        // Buginese U+1A1E PALLAWA, U+1A1F END OF SECTION (Po). Tai Tham U+1AA0 WIANG through U+1AA6 REVERSED ROTATED RANA,
        // U+1AA8 KAAN through U+1AAD CAANG (Po). U+1AA7 MAI YAMOK (Lm) omitted. None of the mapped code points are Rust whitespace.
        let mut seps: Vec<char> = ('\u{1A1E}'..='\u{1A1F}').collect();
        seps.extend('\u{1AA0}'..='\u{1AA6}');
        seps.extend('\u{1AA8}'..='\u{1AAD}');
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
    fn ocr_symbols_separate_words() {
        // U+2440–U+245F: Optical Character Recognition block (hook, chair, fork, belt buckle, bow tie,
        // branch, amount of check, dash, etc., through OCR DOUBLE BACKSLASH; all So) are not Rust
        // whitespace; OCR’d page HTML can glue Latin tokens without ASCII space.
        for cp in 0x2440u32..=0x245F {
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
    fn enclosed_alphanumerics_separate_words() {
        // U+2460–U+24FF: circled / parenthesized digits, digit full stops, enclosed Latin, etc. (No / So);
        // not Rust whitespace; Unicode lists or samples can glue Latin tokens without ASCII space.
        for cp in 0x2460u32..=0x24FF {
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
    fn box_drawing_block_elements_geometric_shapes_separate_words() {
        // U+2500–U+257F Box Drawing, U+2580–U+259F Block Elements, U+25A0–U+25FF Geometric Shapes (mostly So);
        // not Rust whitespace; table / diagram HTML can glue Latin tokens without ASCII space.
        for cp in 0x2500u32..=0x25FF {
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
    fn miscellaneous_symbols_separate_words() {
        // U+2600–U+26FF Miscellaneous Symbols (mostly So / Sm / Po); not Rust whitespace; weather / chess / dingbat
        // or Unicode-sample HTML can glue Latin tokens without ASCII space.
        for cp in 0x2600u32..=0x26FF {
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
    fn dingbats_separate_words() {
        // U+2700–U+27BF Dingbats (mostly So / Sm / Po); not Rust whitespace; Zapf-style or Unicode-sample HTML can
        // glue Latin tokens without ASCII space.
        for cp in 0x2700u32..=0x27BF {
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
    fn miscellaneous_mathematical_symbols_a_and_supplemental_arrows_a_separate_words() {
        // U+27C0–U+27EF Miscellaneous Mathematical Symbols-A + U+27F0–U+27FF Supplemental Arrows-A (mostly Sm); not Rust
        // whitespace; MathML / LaTeX-style or Unicode-sample HTML can glue Latin tokens without ASCII space.
        for cp in 0x27C0u32..=0x27FF {
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
    fn braille_patterns_separate_words() {
        // U+2800–U+28FF Braille Patterns (all So); not Rust whitespace; U+2800 blank and raised-dot
        // cells in accessibility or Unicode-sample HTML can glue Latin tokens without ASCII space.
        for cp in 0x2800u32..=0x28FF {
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
    fn supplemental_arrows_b_and_miscellaneous_mathematical_symbols_b_separate_words() {
        // U+2900–U+297F Supplemental Arrows-B + U+2980–U+29FF Miscellaneous Mathematical Symbols-B
        // (mostly Sm); not Rust whitespace; MathML / Unicode-sample HTML can glue Latin tokens without ASCII space.
        for cp in 0x2900u32..=0x29FF {
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
    fn supplemental_mathematical_operators_separate_words() {
        // U+2A00–U+2AFF Supplemental Mathematical Operators (mostly Sm); not Rust whitespace;
        // MathML / Unicode-sample HTML can glue Latin tokens without ASCII space.
        for cp in 0x2A00u32..=0x2AFF {
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
    fn miscellaneous_symbols_and_arrows_separate_words() {
        // U+2B00–U+2BFF Miscellaneous Symbols and Arrows (mostly So / Sm / Po); not Rust whitespace;
        // diagram or Unicode-sample HTML can glue Latin tokens without ASCII space.
        for cp in 0x2B00u32..=0x2BFF {
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
        // shorthand format overlap / step (Cf). None are Rust whitespace. U+1BC9F CHINOOK FULL STOP
        // (Po) is covered by `ogham_feather_marks_and_duployan_chinook_full_stop_separate_words`.
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
    fn brahmi_danda_through_lotus_and_kaithi_sentence_punctuation_separate_words() {
        // Brahmi: U+11047 DANDA through U+1104D PUNCTUATION LOTUS (Po). Kaithi: U+110BB–U+110BC,
        // U+110BE–U+110C1 (Po); U+110BD NUMBER SIGN (Cf) is covered by `kaithi_number_format_signs_separate_words`.
        for cp in (0x11047u32..=0x1104D)
            .chain(0x110BB..=0x110BC)
            .chain(0x110BE..=0x110C1)
        {
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
    fn chakma_sharada_and_khojki_sentence_punctuation_separate_words() {
        // Chakma: U+11140 SECTION MARK through U+11143 QUESTION MARK (Po). Sharada: U+111C5 DANDA
        // through U+111C8 SEPARATOR, U+111CD SUTRA MARK, U+111DB SIGN SIDDHAM, U+111DD CONTINUATION
        // SIGN, U+111DE / U+111DF SECTION MARK-1 / -2 (all Po). Khojki: U+11238 DANDA through
        // U+1123D ABBREVIATION SIGN (Po). None are Rust whitespace.
        for cp in (0x11140u32..=0x11143)
            .chain(0x111C5..=0x111C8)
            .chain(std::iter::once(0x111CD))
            .chain(std::iter::once(0x111DB))
            .chain(0x111DD..=0x111DF)
            .chain(0x11238..=0x1123D)
        {
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
    fn siddham_sentence_punctuation_separate_words() {
        // Siddham U+115C1 SIGN SIDDHAM through U+115D7 SECTION MARK WITH CIRCLES AND FOUR
        // ENCLOSURES (all Po). U+115C0 SIGN NUKTA (Mn) omitted.
        for cp in 0x115C1u32..=0x115D7 {
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
    fn mongolian_supplement_birga_ornaments_separate_words() {
        // Mongolian Supplement U+11660 BIRGA WITH ORNAMENT through U+1166C TURNED SWIRL BIRGA WITH
        // DOUBLE ORNAMENT (all Po). Distinct from Basic Mongolian U+1800–U+180E arm.
        for cp in 0x11660u32..=0x1166C {
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
    fn takri_sentence_punctuation_separate_words() {
        // Takri U+116B9 ABBREVIATION SIGN through U+116BC SECTION MARK (all Po). U+116B8 LETTER SSA
        // (Lo) omitted.
        for cp in 0x116B9u32..=0x116BC {
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
    fn ahom_dogra_dives_akuru_nandinagari_bhaiksuki_sentence_punctuation_separate_words() {
        // Ahom U+1173C–U+1173E; Dogra U+1183B; Dives Akuru U+11944–U+11946; Nandinagari U+119E2;
        // Bhaiksuki U+11C41–U+11C45 (all Po).
        for cp in (0x1173Cu32..=0x1173E)
            .chain(std::iter::once(0x1183B))
            .chain(0x11944..=0x11946)
            .chain(std::iter::once(0x119E2))
            .chain(0x11C41..=0x11C45)
        {
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
    fn zanabazar_square_and_soyombo_sentence_punctuation_separate_words() {
        // Zanabazar Square U+11A3F–U+11A46 (head marks, tsheg, shad); Soyombo U+11A9A–U+11A9C,
        // U+11A9E–U+11AA2 (tsheg, shad, head marks, terminal marks)—all Po; U+11A9D MARK PLUTA (Lo) omitted.
        for cp in (0x11A3Fu32..=0x11A46)
            .chain(0x11A9A..=0x11A9C)
            .chain(0x11A9E..=0x11AA2)
        {
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
    fn marchen_and_makasar_sentence_punctuation_separate_words() {
        // Marchen U+11C70 HEAD MARK, U+11C71 MARK SHAD; Makasar U+11EF7 PASSIMBANG, U+11EF8 END OF
        // SECTION (all Po).
        for cp in (0x11C70u32..=0x11C71).chain(0x11EF7..=0x11EF8) {
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
    fn kawi_sentence_punctuation_and_tamil_end_of_text_separate_words() {
        // Kawi U+11F43 DANDA through U+11F4F CLOSING SPIRAL (all Po); Tamil Supplement U+11FFF END OF
        // TEXT (Po).
        for cp in (0x11F43u32..=0x11F4F).chain(std::iter::once(0x11FFF)) {
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
    fn old_uyghur_punctuation_separate_words() {
        // Old Uyghur U+10F86 PUNCTUATION BAR through U+10F89 PUNCTUATION FOUR DOTS (all Po).
        for cp in 0x10F86..=0x10F89 {
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
    fn psalter_pahlavi_and_sogdian_punctuation_separate_words() {
        // Psalter Pahlavi U+10B99 SECTION MARK through U+10B9C FOUR DOTS WITH DOT (all Po); Sogdian
        // U+10F55 TWO VERTICAL BARS through U+10F59 HALF CIRCLE WITH DOT (all Po).
        for cp in (0x10B99u32..=0x10B9C).chain(0x10F55..=0x10F59) {
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
    fn kharoshthi_and_avestan_punctuation_separate_words() {
        // Kharoshthi U+10A50 PUNCTUATION DOT through U+10A58 PUNCTUATION LINES (all Po); Avestan
        // U+10B39 ABBREVIATION MARK through U+10B3F LARGE ONE RING OVER TWO RINGS PUNCTUATION (all Po).
        for cp in (0x10A50u32..=0x10A58).chain(0x10B39..=0x10B3F) {
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
    fn cuneiform_punctuation_and_cypro_minoan_po_signs_separate_words() {
        // Cuneiform U+12470 OLD ASSYRIAN WORD DIVIDER through U+12474 DIAGONAL QUADCOLON (all Po);
        // Cypro-Minoan U+12FF1 CM301 and U+12FF2 CM302 (Po).
        for cp in (0x12470u32..=0x12474).chain(0x12FF1..=0x12FF2) {
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
    fn mahajani_multani_tulu_tigalari_newa_tirhuta_modi_sentence_punctuation_separate_words() {
        // Mahajani U+11174–U+11175; Multani U+112A9; Tulu-Tigalari U+113D4–U+113D5, U+113D7–U+113D8;
        // Newa U+1144B–U+1144F, U+1145A, U+1145B, U+1145D; Tirhuta U+114C6; Modi U+11641–U+11643 (all Po).
        for cp in (0x11174u32..=0x11175)
            .chain(std::iter::once(0x112A9))
            .chain(0x113D4..=0x113D5)
            .chain(0x113D7..=0x113D8)
            .chain(0x1144B..=0x1144F)
            .chain([0x1145Au32, 0x1145B, 0x1145D])
            .chain(std::iter::once(0x114C6))
            .chain(0x11641..=0x11643)
        {
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
