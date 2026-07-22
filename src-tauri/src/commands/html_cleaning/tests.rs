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
        // U+00A0 (NO-BREAK SPACE) from `&nbsp;` is Unicode Zs; Rust `split_whitespace` already splits
        // on it. `collapse_whitespace` still maps it to ASCII U+0020 so cleaned FETCH_URL text is
        // plain-space–only and never retains U+00A0.
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
    fn ideographic_space_zs_u3000_separates_words_plain_ascii_between_tokens() {
        // IDEOGRAPHIC SPACE (U+3000, `Zs`) is Rust whitespace; it is not on the CJK punctuation `match`
        // arm (see `collapse_whitespace` comment; FEAT-D273). Between Latin tokens, `split_whitespace`
        // still splits and `join(" ")` yields ASCII spaces only—unlike NBSP, there is no explicit `=> ' '`
        // mapping because the separator never survives as U+3000 between words after collapse.
        let html = "<html><body><p>hello\u{3000}world</p></body></html>";
        let cleaned = clean_html(html);
        assert!(
            cleaned.contains("hello world"),
            "expected ideographic space to separate tokens, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains('\u{3000}'),
            "cleaned output should not retain U+3000 between tokens"
        );
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
        // NKo U+07F6 (SYMBOL OO DENNEN; So), U+07F7–U+07F9 (SYMBOL GBAKURUNEN, COMMA, EXCLAMATION MARK; Po),
        // U+07FE–U+07FF (DOROME / TAMAN SIGNS; Sc)—not Rust whitespace.
        // Samaritan U+0830–U+083E (punctuation marks through ANNAAU; Po)—not Rust whitespace.
        // U+083F unassigned. U+07FA LAJANYALAN (Lm) omitted.
        for cp in [0x07F6u32, 0x07F7, 0x07F8, 0x07F9, 0x07FE, 0x07FF]
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
    fn nko_lajanyalan_tone_apostrophe_and_dantayalan_stay_unmapped() {
        // U+07FA LAJANYALAN (Lm), U+07F4–U+07F5 tone apostrophe (Lm), U+07FD DANTAYALAN (Mn), and
        // combining marks U+07F0–U+07F3 must not split Latin tokens.
        for cp in [
            0x07F0u32, 0x07F1, 0x07F2, 0x07F3, 0x07F4, 0x07F5, 0x07FA, 0x07FD,
        ] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn adlam_initial_punctuation_and_medefaidrin_sentence_marks_separate_words() {
        // Adlam U+1E95E–U+1E95F (INITIAL EXCLAMATION / QUESTION; Po)—not Rust whitespace.
        // Medefaidrin U+16E97–U+16E9A (COMMA, FULL STOP, SYMBOL AIVA, EXCLAMATION OH; Po)—not Rust
        // whitespace (FEAT-D204 extends FEAT-D92 contiguous run).
        for cp in (0x16E97u32..=0x16E9A).chain(0x1E95Eu32..=0x1E95F) {
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
    fn medefaidrin_letter_ll_u16e6d_stays_unmapped() {
        // U+16E6D MEDEFAIDRIN SMALL LETTER A (Ll) must not split tokens like U+16E97–U+16E9A `Po`.
        let cp = 0x16E6D_u32;
        let sep = char::from_u32(cp).unwrap();
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+{:04X} must stay unmapped, got {:?}",
            cp,
            cleaned
        );
        assert!(
            cleaned.contains(sep),
            "U+{:04X} should pass through, got {:?}",
            cp,
            cleaned
        );
        assert_eq!(
            cleaned.split_whitespace().count(),
            1,
            "U+{:04X} must not become a word break",
            cp
        );
    }

    #[test]
    fn wancho_comma_full_stop_and_nyiakeng_sentence_punctuation_separate_words() {
        // Wancho U+1E2FF NGUN SIGN (Sc)—not Rust whitespace (UnicodeData; historical FEAT-D91 “comma /
        // full stop Po” for U+1E2FE–U+1E2FF superseded: digits occupy U+1E2F0–U+1E2F9, gap U+1E2FA–U+1E2FE
        // unassigned, FEAT-D214).
        // Nyiakeng Puachue Hmong U+16FE2–U+16FE3 (EXCLAMATION MARK, QUESTION MARK; Po)—not Rust whitespace.
        // Nyiakeng Puachue Hmong U+1E14F CIRCLED CA (So)—not Rust whitespace; extends FEAT-D91 arm.
        for cp in [0x16FE2u32, 0x16FE3]
            .into_iter()
            .chain(std::iter::once(0x1E14F))
            .chain(std::iter::once(0x1E2FF))
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
    fn wancho_digit_nd_and_unassigned_gap_before_ngun_stay_unmapped() {
        // U+1E2F0 WANCHO DIGIT ZERO (Nd) and unassigned U+1E2FA (`Cn` before NGUN SIGN) must not split tokens
        // like U+1E2FF WANCHO NGUN SIGN (`Sc`; FEAT-D214).
        for cp in [0x1E2F0u32, 0x1E2FA] {
            let sep = char::from_u32(cp).expect("valid test scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} must stay unmapped, got {:?}",
                cp,
                cleaned
            );
            assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn nyiakeng_puachue_hmong_digit_nd_and_category_lm_stay_unmapped() {
        // U+1E140 DIGIT ZERO (Nd) and U+1E13D SYLLABLE LENGTHENER (Lm) must not split tokens like U+1E14F `So`.
        for cp in [0x1E140u32, 0x1E13D] {
            let sep = char::from_u32(cp).expect("valid test scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} must stay unmapped, got {:?}",
                cp,
                cleaned
            );
            assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
        }
    }

    #[test]
    fn latin1_currency_signs_u00a2_through_u00a5_separate_words() {
        // U+00A2 CENT through U+00A5 YEN SIGN (all Sc); not Rust whitespace—Western price copy or
        // Unicode-sample HTML can glue Latin tokens without ASCII space (FEAT-D223).
        for sep in '\u{00A2}'..='\u{00A5}' {
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
    fn latin1_degree_sign_so_u00b0_and_math_sm_u00ac_u00b1_u00d7_u00f7_separate_words() {
        // U+00B0 DEGREE SIGN (So); U+00AC NOT SIGN, U+00B1 PLUS-MINUS, U+00D7 MULTIPLICATION, U+00F7 DIVISION
        // (all Sm); not Rust whitespace—scientific or pasted math HTML can glue Latin tokens without ASCII space
        // (FEAT-D225; outside U+2200–U+22FF Mathematical Operators arm).
        for sep in ['\u{00B0}', '\u{00AC}', '\u{00B1}', '\u{00D7}', '\u{00F7}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {sep:?} normalized before collapse, got {:?}",
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
    fn spacing_modifier_arrowheads_sk_u02c2_through_u02c5_and_tacks_u02d2_through_u02d5_separate_words(
    ) {
        // U+02C2–U+02C5: modifier-letter arrowheads (`Sk`); U+02D2–U+02D5: centred half-rings and tacks (`Sk`); not Rust
        // whitespace—IPA or Unicode-sample HTML can glue Latin tokens without ASCII space (FEAT-D239 arm narrowed in FEAT-D240:
        // U+02C6/U+02C7 are `Lm`, not `Sk`).
        let seps: Vec<char> = ('\u{02C2}'..='\u{02C5}')
            .chain('\u{02D2}'..='\u{02D5}')
            .collect();
        assert_eq!(seps.len(), 4 + 4);
        for sep in seps {
            let s = format!("foo{sep}bar");
            let t = collapse_whitespace(&s);
            let w: Vec<&str> = t.split_whitespace().collect();
            assert_eq!(
                w,
                vec!["foo", "bar"],
                "U+{:04X} Sk should separate words, got {:?}",
                sep as u32,
                w
            );
        }
    }

    #[test]
    fn spacing_modifier_circumflex_and_caron_lm_u02c6_u02c7_stay_unmapped() {
        // U+02C6 MODIFIER LETTER CIRCUMFLEX ACCENT and U+02C7 CARON are `Lm` in UnicodeData—IPA superscript diacritics;
        // must not split like arrowhead `Sk` U+02C2–U+02C5 (FEAT-D240; corrects historical FEAT-D239 miscategorization).
        for sep in ['\u{02C6}', '\u{02C7}'] {
            let s = format!("foo{sep}bar");
            let t = collapse_whitespace(&s);
            let w: Vec<&str> = t.split_whitespace().collect();
            assert_eq!(
                w,
                vec![s.as_str()],
                "U+{:04X} Lm should not split tokens, got {:?}",
                sep as u32,
                w
            );
        }
    }

    #[test]
    fn spacing_modifier_stress_marks_lm_u02c8_and_u02d0_stay_unmapped() {
        // U+02C8 VERTICAL LINE and U+02D0 TRIANGULAR COLON are `Lm` between unmapped Lm U+02C6/U+02C7 and mapped Sk U+02D2—stress / length,
        // not word separators like arrowheads or tacks.
        for sep in ['\u{02C8}', '\u{02D0}'] {
            let s = format!("foo{sep}bar");
            let t = collapse_whitespace(&s);
            let w: Vec<&str> = t.split_whitespace().collect();
            assert_eq!(
                w,
                vec![s.as_str()],
                "U+{:04X} Lm should not split tokens, got {:?}",
                sep as u32,
                w
            );
        }
    }

    #[test]
    fn modifier_letter_plus_and_minus_sk_u02d6_u02d7_separate_words() {
        // U+02D6 MODIFIER LETTER PLUS SIGN and U+02D7 MODIFIER LETTER MINUS SIGN (`Sk`); not Rust whitespace—IPA or
        // linguistic HTML can glue Latin tokens without ASCII space (FEAT-D237; spirit of U+00B1).
        for sep in ['\u{02D6}', '\u{02D7}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {sep:?} normalized before collapse, got {:?}",
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
    fn spacing_modifier_tone_contour_sk_u02e5_through_u02eb_u02ed_u02ef_through_u02ff_separate_words(
    ) {
        // U+02E5–U+02EB, U+02ED, U+02EF–U+02FF: tone bars, departing marks, unaspirated, arrowheads, shelves (`Sk`);
        // not Rust whitespace.
        let seps: Vec<char> = ('\u{02E5}'..='\u{02EB}')
            .chain(std::iter::once('\u{02ED}'))
            .chain('\u{02EF}'..='\u{02FF}')
            .collect();
        assert_eq!(seps.len(), 7 + 1 + 17);
        for sep in seps {
            let s = format!("foo{sep}bar");
            let t = collapse_whitespace(&s);
            let w: Vec<&str> = t.split_whitespace().collect();
            assert_eq!(
                w,
                vec!["foo", "bar"],
                "U+{:04X} Sk should separate words, got {:?}",
                sep as u32,
                w
            );
        }
    }

    #[test]
    fn spacing_modifier_voicing_lm_u02ec_and_double_apostrophe_lm_u02ee_stay_unmapped() {
        // U+02EC VOICING and U+02EE DOUBLE APOSTROPHE are `Lm`—must not split like neighboring Sk tone contours.
        for sep in ['\u{02EC}', '\u{02EE}'] {
            let s = format!("foo{sep}bar");
            let t = collapse_whitespace(&s);
            let w: Vec<&str> = t.split_whitespace().collect();
            assert_eq!(
                w,
                vec![s.as_str()],
                "U+{:04X} Lm should not split tokens, got {:?}",
                sep as u32,
                w
            );
        }
    }

    #[test]
    fn spacing_modifier_superscript_letter_lm_u02e1_stays_unmapped() {
        // U+02E1 MODIFIER LETTER SMALL L sits between unmapped Sk hooks (U+02DF) and mapped tone bars (U+02E5)—`Lm`.
        let sep = '\u{02E1}';
        let s = format!("foo{sep}bar");
        let t = collapse_whitespace(&s);
        let w: Vec<&str> = t.split_whitespace().collect();
        assert_eq!(
            w,
            vec![s.as_str()],
            "U+02E1 Lm should not split tokens, got {:?}",
            w
        );
    }

    #[test]
    fn modifier_tone_letters_sk_u_a700_through_u_a716_separate_words() {
        // Modifier Tone Letters: contiguous `Sk` U+A700–U+A716 (Chinese tone marks through extra-low left-stem tone bar);
        // not Rust whitespace (FEAT-D243).
        for sep in '\u{A700}'..='\u{A716}' {
            let s = format!("foo{sep}bar");
            let t = collapse_whitespace(&s);
            let w: Vec<&str> = t.split_whitespace().collect();
            assert_eq!(
                w,
                vec!["foo", "bar"],
                "U+{:04X} Sk should separate words, got {:?}",
                sep as u32,
                w
            );
        }
    }

    #[test]
    fn modifier_tone_letters_lm_u_a717_through_u_a71f_stay_unmapped() {
        // U+A717–U+A71F are `Lm` (dot vertical bar through low inverted exclamation)—spot-check ends and middle.
        for sep in ['\u{A717}', '\u{A71A}', '\u{A71F}'] {
            let s = format!("foo{sep}bar");
            let t = collapse_whitespace(&s);
            let w: Vec<&str> = t.split_whitespace().collect();
            assert_eq!(
                w,
                vec![s.as_str()],
                "U+{:04X} Lm should not split tokens, got {:?}",
                sep as u32,
                w
            );
        }
    }

    #[test]
    fn latin_extended_d_modifier_letters_sk_a720_a721_a789_a78a_separate_words() {
        // Latin Extended-D: sole `Sk` scalars U+A720/U+A721 (stress / high-low tone) and U+A789/U+A78A (colon / short equals);
        // not Rust whitespace (FEAT-D241).
        for sep in ['\u{A720}', '\u{A721}', '\u{A789}', '\u{A78A}'] {
            let s = format!("foo{sep}bar");
            let t = collapse_whitespace(&s);
            let w: Vec<&str> = t.split_whitespace().collect();
            assert_eq!(
                w,
                vec!["foo", "bar"],
                "U+{:04X} Sk should separate words, got {:?}",
                sep as u32,
                w
            );
        }
    }

    #[test]
    fn latin_extended_d_modifier_letter_lm_a788_and_letter_ll_a723_stay_unmapped() {
        // U+A788 MODIFIER LETTER LOW CIRCUMFLEX ACCENT (`Lm`); U+A723 LATIN SMALL LETTER EGYPTOLOGICAL ALEF (`Ll`)—spot-check.
        for sep in ['\u{A788}', '\u{A723}'] {
            let s = format!("foo{sep}bar");
            let t = collapse_whitespace(&s);
            let w: Vec<&str> = t.split_whitespace().collect();
            assert_eq!(
                w,
                vec![s.as_str()],
                "U+{:04X} should not split tokens, got {:?}",
                sep as u32,
                w
            );
        }
    }

    #[test]
    fn latin_extended_e_modifier_letters_sk_ab5b_ab6a_ab6b_separate_words() {
        // Latin Extended-E: sole `Sk` scalars U+AB5B (modifier breve with inverted breve), U+AB6A (left tack), U+AB6B (right tack);
        // not Rust whitespace (FEAT-D242).
        for sep in ['\u{AB5B}', '\u{AB6A}', '\u{AB6B}'] {
            let s = format!("foo{sep}bar");
            let t = collapse_whitespace(&s);
            let w: Vec<&str> = t.split_whitespace().collect();
            assert_eq!(
                w,
                vec!["foo", "bar"],
                "U+{:04X} Sk should separate words, got {:?}",
                sep as u32,
                w
            );
        }
    }

    #[test]
    fn latin_extended_e_letter_ll_ab5a_and_modifier_lm_ab5c_stay_unmapped() {
        // U+AB5A LATIN SMALL LETTER Y WITH SHORT RIGHT LEG (`Ll`); U+AB5C MODIFIER LETTER SMALL HENG (`Lm`)—spot-check.
        for sep in ['\u{AB5A}', '\u{AB5C}'] {
            let s = format!("foo{sep}bar");
            let t = collapse_whitespace(&s);
            let w: Vec<&str> = t.split_whitespace().collect();
            assert_eq!(
                w,
                vec![s.as_str()],
                "U+{:04X} should not split tokens, got {:?}",
                sep as u32,
                w
            );
        }
    }

    #[test]
    fn greek_extended_spacing_sk_polytonic_separate_words() {
        // Greek Extended: fifteen assigned spacing `Sk` polytonic marks; not Rust whitespace (FEAT-D244; extends FEAT-D235).
        let seps: Vec<char> = std::iter::once('\u{1FBD}')
            .chain(std::iter::once('\u{1FBF}'))
            .chain('\u{1FC0}'..='\u{1FC1}')
            .chain('\u{1FCD}'..='\u{1FCF}')
            .chain('\u{1FDD}'..='\u{1FDF}')
            .chain('\u{1FED}'..='\u{1FEF}')
            .chain('\u{1FFD}'..='\u{1FFE}')
            .collect();
        assert_eq!(seps.len(), 15);
        for sep in seps {
            let s = format!("foo{sep}bar");
            let t = collapse_whitespace(&s);
            let w: Vec<&str> = t.split_whitespace().collect();
            assert_eq!(
                w,
                vec!["foo", "bar"],
                "U+{:04X} Sk should separate words, got {:?}",
                sep as u32,
                w
            );
        }
    }

    #[test]
    fn greek_extended_letter_alpha_with_perispomeni_ll_u1fb6_stays_unmapped() {
        // U+1FB6 GREEK SMALL LETTER ALPHA WITH PERISPOMENI (`Ll`)—precomposed syllable, not a spacing `Sk` separator.
        let sep = '\u{1FB6}';
        let s = format!("foo{sep}bar");
        let t = collapse_whitespace(&s);
        let w: Vec<&str> = t.split_whitespace().collect();
        assert_eq!(
            w,
            vec![s.as_str()],
            "U+1FB6 Ll should not split tokens, got {:?}",
            w
        );
    }

    #[test]
    fn modifier_letter_breve_sk_u02d8_stays_unmapped() {
        // U+02D8 BREVE (`Sk`) is a legacy spacing accent—must not split like modifier letter plus/minus U+02D6/U+02D7.
        let sep = '\u{02D8}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+02D8 Sk should not split tokens like U+02D6/U+02D7, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
    }

    #[test]
    fn latin1_vulgar_fractions_no_u00bc_u00bd_u00be_separate_words() {
        // U+00BC VULGAR FRACTION ONE QUARTER, U+00BD ONE HALF, U+00BE THREE QUARTERS (all No); not Rust
        // whitespace—extends Number Forms spirit (FEAT-D146) into Latin-1 Supplement (FEAT-D229).
        for sep in ['\u{00BC}', '\u{00BD}', '\u{00BE}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {:?} to separate words, got {:?}",
                sep,
                cleaned
            );
        }
    }

    #[test]
    fn latin1_superscript_digits_no_and_masculine_ordinal_ll_stay_unmapped() {
        // U+00B2, U+00B3, U+00B9 (No) and U+00BA MASCULINE ORDINAL INDICATOR (Ll) sit near mapped Latin-1
        // scalars; numeric / ordinal semantics—must not split like U+00B0 / U+00B1 (FEAT-D225). Vulgar
        // fractions U+00BC–U+00BE map as word separators (FEAT-D229).
        for cp in [0x00B2u32, 0x00B3, 0x00B9, 0x00BA] {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} must stay unmapped, got {:?}",
                cp,
                cleaned
            );
            assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
        }
    }

    #[test]
    fn latin1_copyright_and_registered_signs_so_u00a9_u00ae_separate_words() {
        // U+00A9 COPYRIGHT SIGN and U+00AE REGISTERED SIGN (both So); not Rust whitespace—legal or
        // trademark HTML can glue Latin tokens without ASCII space (FEAT-D224).
        for sep in ['\u{00A9}', '\u{00AE}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {sep:?} normalized before collapse, got {:?}",
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
    fn latin1_feminine_ordinal_ll_u00aa_stays_unmapped() {
        // U+00AA FEMININE ORDINAL INDICATOR (Ll) sits between mapped Latin-1 scalars; letter-like,
        // word-internal risk—must not get the So treatment of U+00A9 / U+00AE (FEAT-D224).
        let sep = '\u{00AA}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+00AA must stay unmapped, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
    }

    #[test]
    fn latin1_broken_bar_sm_u00a6_separate_words() {
        // U+00A6 BROKEN BAR (So in UnicodeData; legacy docs call it a bar / pipe substitute); not Rust whitespace—legacy HTML
        // (FEAT-D226; ISO 8859-1 pipe substitute, distinct from ASCII U+007C).
        let sep = '\u{00A6}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains("hello world"),
            "expected {sep:?} normalized before collapse, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains(sep),
            "cleaned output still contains {:?}",
            sep
        );
    }

    #[test]
    fn latin1_diaeresis_sk_u00a8_stays_unmapped() {
        // U+00A8 DIAERESIS (Sk) is not contiguous `Sc` with U+00A2–U+00A5; spacing-mark semantics—stay
        // unmapped (FEAT-D223 spirit; FEAT-D226 maps U+00A6 only).
        let sep = '\u{00A8}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+00A8 must stay unmapped, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
    }

    #[test]
    fn latin1_macron_acute_and_cedilla_sk_u00af_u00b4_u00b8_separate_words() {
        // U+00AF MACRON, U+00B4 ACUTE ACCENT, U+00B8 CEDILLA (Sk); not Rust whitespace—legacy Latin-1 or
        // dictionary typography can glue Latin tokens without ASCII space (FEAT-D245). U+00A8 DIAERESIS (`Sk`)
        // stays unmapped—`latin1_diaeresis_sk_u00a8_stays_unmapped`.
        for sep in ['\u{00AF}', '\u{00B4}', '\u{00B8}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {sep:?} normalized before collapse, got {:?}",
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
    fn basic_latin_spacing_circumflex_and_grave_sk_u005e_u0060_separate_words() {
        // U+005E CIRCUMFLEX ACCENT and U+0060 GRAVE ACCENT (both Sk); not Rust whitespace—ASCII or transliterated HTML can
        // glue Latin tokens without ASCII space (FEAT-D247).
        for sep in ['\u{005E}', '\u{0060}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected {sep:?} normalized before collapse, got {:?}",
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
    fn basic_latin_low_line_pc_u005f_stays_unmapped() {
        // U+005F LOW LINE (`Pc`); underscore semantics—must not split like U+005E / U+0060 (FEAT-D247).
        let sep = '\u{005F}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+005F must stay unmapped, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
    }

    #[test]
    fn fullwidth_quotation_mark_po_ff02_separate_words() {
        // U+FF02 FULLWIDTH QUOTATION MARK (`Po`); not Rust whitespace—CJK or mixed-layout HTML can glue
        // Latin tokens without ASCII space (fills gap between U+FF01 and U+FF03 on the fullwidth arm).
        let sep = '\u{FF02}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains("hello world"),
            "expected {sep:?} normalized before collapse, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains(sep),
            "cleaned output still contains {:?}",
            sep
        );
    }

    #[test]
    fn fullwidth_apostrophe_po_ff07_stays_unmapped() {
        // U+FF07 FULLWIDTH APOSTROPHE (`Po`) stays unmapped—typographic apostrophe / word-internal risk (FEAT-D125).
        let sep = '\u{FF07}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+FF07 must stay unmapped, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
    }

    #[test]
    fn fullwidth_circumflex_sk_ff3e_separate_words() {
        // U+FF3E FULLWIDTH CIRCUMFLEX ACCENT (`Sk`); not Rust whitespace—JIS / fullwidth typography can glue
        // Latin tokens without ASCII space (extends FEAT-D247 U+005E).
        let sep = '\u{FF3E}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains("hello world"),
            "expected {sep:?} normalized before collapse, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains(sep),
            "cleaned output still contains {:?}",
            sep
        );
    }

    #[test]
    fn fullwidth_grave_sk_ff40_stays_unmapped() {
        // U+FF40 FULLWIDTH GRAVE ACCENT (`Sk`) stays unmapped—modifier-like, word-internal risk (FEAT-D125).
        let sep = '\u{FF40}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+FF40 must stay unmapped, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
    }

    #[test]
    fn c0_controls_u0001_through_u0008_u000e_through_u001f_and_del_separate_words() {
        // U+0001–U+0008, U+000E–U+001F, U+007F (Cc); not Unicode White_Space—binary-pasted or legacy
        // control bytes in FETCH_URL bodies should still tokenize (FEAT-D228). U+0000 is dropped by
        // the HTML parser in text nodes; U+0009–U+000D are White_Space and omitted from the match arm.
        for cp in (0x1u32..=0x8)
            .chain(0xE..=0x1F)
            .chain(std::iter::once(0x7F))
        {
            let sep = char::from_u32(cp).unwrap();
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized, got {:?}",
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
    fn c1_controls_u0080_through_u009f_separate_words() {
        // U+0080–U+009F (Cc); not Rust whitespace—C1 noise in FETCH_URL bodies should still
        // tokenize (FEAT-D227). Spot-check NEL (U+0085) and a high C1 (U+009F).
        for cp in 0x80u32..=0x9F {
            let sep = char::from_u32(cp).unwrap();
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "expected U+{:04X} normalized, got {:?}",
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
    fn currency_symbols_u20a0_through_u20c1_separate_words() {
        // Currency Symbols U+20A0–U+20C1 (all Sc); not Rust whitespace—price or Unicode-sample HTML
        // can glue Latin tokens without ASCII space. U+20C0 SOM SIGN (Unicode 14), U+20C1 SAUDI RIYAL
        // SIGN (Unicode 17) extend FEAT-D141 contiguous block tail.
        for sep in '\u{20A0}'..='\u{20C1}' {
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
    fn currency_symbols_unassigned_tail_u20c2_through_u20cf_stay_unmapped() {
        // Currency Symbols block tail U+20C2–U+20CF: UnicodeData unassigned (`Cn`)—must not widen
        // the contiguous `Sc` arm past U+20C1 (FEAT-D222).
        for cp in 0x20C2u32..=0x20CF {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} is unassigned and must not split words, got {:?}",
                cp,
                cleaned
            );
        }
    }

    #[test]
    fn number_forms_fractions_romans_and_turned_digits_separate_words() {
        // U+2150–U+2182, U+2185–U+218B: vulgar fractions, Roman numerals, late forms, vulgar fraction
        // zero thirds (U+2189, `No`), turned digits U+218A–U+218B (`So` in UnicodeData; FEAT-D265—FEAT-D263
        // mis-stated them as `No`/`Nl`). Other mapped scalars here are `No`/`Nl`. Not Rust whitespace.
        // U+2183 (Lu) / U+2184 (Ll) omitted in implementation.
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
    fn letterlike_symbol_subranges_separate_words() {
        // Letterlike Symbols: So/Sm only (FEAT-D147; U+2103..=U+2106 and U+2108..=U+2109; U+2107 EULER CONSTANT is `Lu`,
        // FEAT-D252); U+2139 INFORMATION SOURCE `So` with U+213A–U+213B, FEAT-D251; U+2140–U+2144 are all `Sm` (FEAT-D261;
        // `collapse_whitespace` narrative once called this run `So` by mistake); U+213F is `Lu` (FEAT-D246). Tail U+214A–U+214D
        // (`So`/`Sm`/`So`/`So`) and U+214F (`So`) are documented on the `collapse_whitespace` block comment (FEAT-D262)—not Rust whitespace.
        let runs: &[(u32, u32)] = &[
            (0x2100, 0x2101),
            (0x2103, 0x2106),
            (0x2108, 0x2109),
            (0x2114, 0x2114),
            (0x2116, 0x2118),
            (0x211E, 0x2123),
            (0x2125, 0x2125),
            (0x2127, 0x2127),
            (0x2129, 0x2129),
            (0x212E, 0x212E),
            (0x2139, 0x213B),
            (0x2140, 0x2144),
            (0x214A, 0x214D),
            (0x214F, 0x214F),
        ];
        for &(a, b) in runs {
            for cp in a..=b {
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
    }

    #[test]
    fn letterlike_mathematical_letters_stay_unmapped() {
        // Lu/Ll/Lo in Letterlike Symbols—must not split Latin tokens like mapped So/Sm.
        for cp in [
            0x2102u32, // DOUBLE-STRUCK CAPITAL C (Lu)
            0x2107,    // EULER CONSTANT (Lu; FEAT-D252)
            0x210E,    // PLANCK CONSTANT (Ll)
            0x2115,    // DOUBLE-STRUCK CAPITAL N (Lu)
            0x2126,    // OHM SIGN (Lu)
            0x2135,    // ALEF SYMBOL (Lo)
            0x213C,    // DOUBLE-STRUCK SMALL PI (Ll)
            0x213E,    // DOUBLE-STRUCK CAPITAL GAMMA (Lu)
            0x213F,    // DOUBLE-STRUCK CAPITAL PI (Lu)
            0x2146,    // DOUBLE-STRUCK ITALIC SMALL D (Ll)
            0x214E,    // TURNED SMALL F (Ll; FEAT-D262—mapped tail is U+214A–U+214D / U+214F only)
        ] {
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
        // (full stop, hyphen; Po/Pd)—not Rust whitespace. U+058D–U+058F: eternity signs (`So`) and dram (`Sc`)
        // extend FEAT-D72; not Rust whitespace. U+055A apostrophe and U+055B emphasis
        // stay unmapped (word-internal risk, like U+2019).
        for sep in [
            '\u{055C}', '\u{055D}', '\u{055E}', '\u{055F}', '\u{0589}', '\u{058A}', '\u{058D}',
            '\u{058E}', '\u{058F}',
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
    fn armenian_modifier_letter_half_ring_lm_u0559_stays_unmapped() {
        // U+0559 ARMENIAN MODIFIER LETTER LEFT HALF RING (`Lm`)—modifier-like, word-internal risk; not mapped like Po sentence marks.
        let sep = '\u{0559}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+0559 Lm should not split tokens like eternity/dram signs, got {:?}",
            cleaned
        );
        assert!(
            cleaned.contains(sep),
            "expected raw U+0559 preserved in output"
        );
    }

    #[test]
    fn latin1_greek_and_arabic_script_punctuation_separate_words() {
        // U+00A1 / U+00BF (inverted ! / ?), U+00AB / U+00BB (guillemets), U+037E (Greek question
        // mark), U+0384 / U+0385 (Greek spacing tonos / dialytika tonos, Sk), U+03D6 (Greek pi symbol, Sm),
        // U+03F6 (Greek reversed lunate epsilon symbol, Sm), U+060C / U+060E / U+060F / U+061B / U+061D / U+061F / U+06D4 (Arabic comma, poetic verse / misra So, semicolon, end of text mark, question, full stop),
        // U+0888 (Arabic raised round dot, Sk), U+066A–U+066D (Arabic percent, decimal sep, thousands sep, five pointed star; Po / Pi / Pf / So); not Rust
        // whitespace—mixed European, Greek, or Arabic/Latin HTML can glue tokens without ASCII space.
        for sep in [
            '\u{00A1}', '\u{00BF}', '\u{00AB}', '\u{00BB}', '\u{037E}', '\u{0384}', '\u{0385}',
            '\u{03D6}', '\u{03F6}', '\u{060C}', '\u{060E}', '\u{060F}', '\u{061B}', '\u{061D}',
            '\u{061F}', '\u{0888}', '\u{06D4}', '\u{066A}', '\u{066B}', '\u{066C}', '\u{066D}',
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
    fn arabic_small_farsi_yeh_lm_u08c9_stays_unmapped() {
        // U+08C9 ARABIC SMALL FARSI YEH (`Lm`)—modifier-like, word-internal risk; not mapped like U+0888 (`Sk`).
        let sep = '\u{08C9}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+08C9 Lm should not split tokens like U+0888, got {:?}",
            cleaned
        );
        assert!(
            cleaned.contains(sep),
            "expected raw U+08C9 preserved in output"
        );
    }

    #[test]
    fn arabic_indic_cube_root_sm_u0606_and_sallallahou_mn_u0610_stay_unmapped() {
        // U+0606–U+0608 (Sm) and U+0610+ (Mn) sit between mapped Arabic supplementary punctuation; must not split like U+060E/U+060F.
        for sep in ['\u{0606}', '\u{0610}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} must stay unmapped, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
        }
    }

    #[test]
    fn greek_lower_numeral_sign_sk_u0375_stays_unmapped() {
        // U+0375 GREEK LOWER NUMERAL SIGN (`Sk`) marks thousands in Greek numeral notation—must not
        // split like spacing tonos U+0384 / dialytika tonos U+0385 (FEAT-D235).
        let sep = '\u{0375}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+0375 Sk should not split tokens like U+0384/U+0385, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
    }

    #[test]
    fn greek_lunate_epsilon_letter_ll_u03f5_stays_unmapped() {
        // U+03F5 (Ll) is a Greek letter symbol—must not split like the adjacent Sm U+03F6.
        let sep = '\u{03F5}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "Greek lunate epsilon U+03F5 (Ll) must stay unmapped, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
    }

    #[test]
    fn greek_phi_symbol_lu_u03d5_stays_unmapped() {
        // U+03D5 (Lu) is a Greek letter symbol—must not split like the adjacent Sm U+03D6.
        let sep = '\u{03D5}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "Greek phi symbol U+03D5 (Lu) must stay unmapped, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
    }

    #[test]
    fn cyrillic_thousands_sign_so_u0482_separate_words() {
        // U+0482 CYRILLIC THOUSANDS SIGN (`So`): not Rust whitespace; extends FEAT-D178 Slavonic/Cyrillic coverage.
        let sep = '\u{0482}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains("hello world"),
            "expected U+0482 normalized before collapse, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains(sep),
            "cleaned output still contains U+0482"
        );
    }

    #[test]
    fn cyrillic_combining_titlo_mn_u0483_and_capital_letter_lo_u0410_stay_unmapped() {
        // U+0483 COMBINING CYRILLIC TITLO (Mn) and U+0410 CYRILLIC CAPITAL LETTER A (Lo)—must not split like U+0482 So.
        for cp in [0x0483_u32, 0x0410] {
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
        // U+1800–U+1805 (Mongolian/Manchu sentence punctuation, Po), U+1806 MONGOLIAN TODO SOFT HYPHEN (Pd;
        // Unicode name contains "TODO"), U+1807–U+180A (Sibe boundary / Manchu stops / NIRUGU, Po): not Rust
        // whitespace, so
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
    fn katakana_hiragana_voiced_semi_voiced_sound_marks_sk_separate_words() {
        // U+309B / U+309C: Katakana-Hiragana voiced / semi-voiced sound marks (`Sk`); not Rust whitespace.
        for sep in ['\u{309B}', '\u{309C}'] {
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
    fn hiragana_letter_lo_u3042_stays_unmapped() {
        // U+3042 HIRAGANA LETTER A (`Lo`) must not be treated like spacing `Sk` U+309B/U+309C.
        let sep = '\u{3042}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains(sep),
            "expected hiragana letter to remain in output, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains("hello world"),
            "hiragana letter must not become ASCII space between tokens, got {:?}",
            cleaned
        );
    }

    #[test]
    fn supplemental_punctuation_u2e00_through_u2e5d_separate_words() {
        // U+2E00–U+2E5D except U+2E2F: assigned Supplemental Punctuation (Po/Pd/Pi/Pf/Ps/Pe/So); not
        // Rust whitespace—critical edition / transcription HTML can glue Latin tokens for
        // `split_whitespace()` without normalization. U+2E2F VERTICAL TILDE (`Lm`) is excluded—see
        // `supplemental_punctuation_vertical_tilde_lm_u2e2f_stays_unmapped`.
        for cp in (0x2E00u32..=0x2E2E).chain(0x2E30u32..=0x2E5D) {
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
    fn supplemental_punctuation_vertical_tilde_lm_u2e2f_stays_unmapped() {
        // U+2E2F VERTICAL TILDE (`Lm`) must not be treated like Supplemental Punctuation `Po` neighbors.
        let sep = '\u{2E2F}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains(sep),
            "expected vertical tilde to remain in output, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains("hello world"),
            "Lm vertical tilde must not become ASCII space between tokens, got {:?}",
            cleaned
        );
    }

    #[test]
    fn supplemental_punctuation_reserved_tail_u2e5e_through_u2e7f_stay_unmapped() {
        // Unicode 17: Supplemental Punctuation ends at U+2E7F; scalars U+2E5E–U+2E7F are unassigned (`Cn`).
        // They must not be folded into a widened `U+2E30..=U+2E7F` word-separator arm—future standard versions
        // could assign them with categories that need separate handling.
        for cp in 0x2E5Eu32..=0x2E7F {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(sep),
                "reserved U+{:04X} must pass through clean_html, got {:?}",
                cp,
                cleaned
            );
            assert!(
                !cleaned.contains("hello world"),
                "reserved U+{:04X} must not become ASCII space between tokens",
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
        // U+10100 / U+10101 / U+10102 (Aegean word separator line/dot/check mark, Po); U+10137–U+1013F (Aegean
        // weight/measure `So`); and U+1091F (Phoenician word separator, Po) are not Rust whitespace.
        for sep in (0x10137u32..=0x1013F)
            .chain([0x10100, 0x10101, 0x10102, 0x1091F])
            .map(|cp| char::from_u32(cp).expect("valid test scalar"))
        {
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
    fn aegean_number_no_and_unassigned_gap_stay_unmapped() {
        // Aegean NUMBER ONE (U+10107, No) and unassigned U+10134 (Cn) must not split tokens like the adjacent `So` arm.
        for cp in [0x10107u32, 0x10134] {
            let sep = char::from_u32(cp).expect("valid test scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} must stay unmapped, got {:?}",
                cp,
                cleaned
            );
            assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
        }
    }

    #[test]
    fn hanifi_rohingya_unassigned_gap_u10d28_through_u10d2f_stays_unmapped() {
        // UnicodeData: U+10D27 SIGN TASSI (`Mn`) then U+10D30 DIGIT ZERO (`Nd`); U+10D28–U+10D2F are unassigned (`Cn`).
        for cp in 0x10D28u32..=0x10D2F {
            let sep = char::from_u32(cp).expect("valid test scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} Cn must stay unmapped, got {:?}",
                cp,
                cleaned
            );
            assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
        }
    }

    #[test]
    fn hanifi_rohingya_combining_signs_mn_u10d24_through_u10d27_stay_unmapped() {
        for cp in [0x10D24u32, 0x10D25, 0x10D26, 0x10D27] {
            let sep = char::from_u32(cp).expect("valid test scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} Mn must stay unmapped, got {:?}",
                cp,
                cleaned
            );
            assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
        }
    }

    #[test]
    fn hanifi_rohingya_digit_nd_u10d30_stays_unmapped() {
        let sep = '\u{10D30}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "Hanifi digit Nd must stay unmapped, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
    }

    #[test]
    fn warang_citi_small_letters_u118c8_through_u118cf_stay_unmapped() {
        // U+118C8–U+118CF are Warang Citi SMALL LETTER E..SMALL LETTER UC (`Ll`); must not be normalized to spaces
        // like true sentence punctuation (FEAT-D212).
        for cp in 0x118C8u32..=0x118CF {
            let sep = char::from_u32(cp).expect("valid test scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} Ll must stay unmapped, got {:?}",
                cp,
                cleaned
            );
            assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
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
    fn vithkuqi_capital_letters_and_gap_u1057b_stay_unmapped() {
        // UnicodeData: U+10570–U+1057A and U+1057C–U+1057F are Vithkuqi capital letters (`Lu`); U+1057B is unassigned (`Cn`).
        // Mapping them to ASCII space splits script text (historical FEAT-D117 mismatch, FEAT-D220).
        for sep in ['\u{10570}', '\u{1057B}', '\u{1057C}', '\u{1057F}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} must stay unmapped, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
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
        let mut seps: Vec<char> = vec![
            char::from_u32(0x10ED0).unwrap(),
            char::from_u32(0x1E5FF).unwrap(),
        ];
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
    fn nabataean_hatran_old_north_arabian_word_separators_separate_words() {
        // Nabataean U+1089F WORD SEPARATOR, Hatran U+108FB SECTION MARK, Old North Arabian U+10A9D WORD DIVIDER (all Po).
        // U+10A9E / U+10A9F (Old North Arabian numbers, Nl) stay unmapped. Elymaic U+10FF5 is `Lo` (FEAT-D213)—see
        // `elymaic_letter_taw_lo_u10ff5_stays_unmapped`.
        for sep in ['\u{1089F}', '\u{108FB}', '\u{10A9D}'] {
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
    fn elymaic_letter_taw_lo_u10ff5_stays_unmapped() {
        // U+10FF5 is ELYMAIC LETTER TAW (`Lo`); mapping it to ASCII space splits Elymaic words incorrectly
        // (historical FEAT-D110 mismatch, same spirit as FEAT-D212 Warang Citi).
        let sep = '\u{10FF5}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+10FF5 Lo must stay unmapped, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
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
        // Lydian U+1093F; Old South Arabian U+10A7F; Manichaean SIGN UD U+10AC8 (So); Manichaean U+10AF0 PUNCTUATION STAR
        // through U+10AF6 PUNCTUATION LINE FILLER (all Po).
        for cp in std::iter::once(0x1039Fu32)
            .chain(std::iter::once(0x103D0))
            .chain(std::iter::once(0x1056F))
            .chain(std::iter::once(0x10857))
            .chain(std::iter::once(0x1093F))
            .chain(std::iter::once(0x10A7F))
            .chain(std::iter::once(0x10AC8))
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
    fn manichaean_abbreviation_marks_and_numbers_stay_unmapped() {
        // Manichaean ABBREVIATION MARK ABOVE / BELOW (U+10AE5–U+10AE6, Mn) and numbers ONE through ONE HUNDRED
        // (U+10AEB–U+10AEF, No)—word-internal / numeric risk; distinct from SIGN UD U+10AC8 and Po U+10AF0+.
        for cp in (0x10AE5u32..=0x10AE6).chain(0x10AEB..=0x10AEF) {
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
            '\u{2010}', '\u{2011}', '\u{2012}', '\u{2013}', '\u{2014}', '\u{2015}',
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
    fn superscripts_and_subscripts_math_delimiters_separate_words() {
        // U+207A–U+207C super plus/minus/equals (Sm), U+207D/U+207E super parens (Ps/Pe); U+208A–
        // U+208C sub plus/minus/equals (Sm), U+208D/U+208E sub parens (Ps/Pe)—not Rust whitespace.
        for cp in (0x207Au32..=0x207E).chain(0x208A..=0x208E) {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "U+{:04X} should separate words, got {:?}",
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
    fn superscripts_subscripts_digit_numerics_and_letter_marks_stay_unmapped() {
        // U+2070 / U+2079 (No), U+2071 / U+207F (Lm), U+2080 / U+2089 (No), U+208F (Cn) must not widen the math-delimiter arms.
        for cp in [0x2070u32, 0x2071, 0x2079, 0x207F, 0x2080, 0x2089, 0x208F] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn cjk_symbols_brackets_ditto_wave_vertical_repeat_masu_half_fill_separate_words() {
        // U+3003 DITTO MARK (`Po`) / U+3004 JAPANESE INDUSTRIAL STANDARD SYMBOL (`So`) / U+3006 IDEOGRAPHIC CLOSING MARK (`Lo`
        // in UnicodeData; FEAT-D257 wrongly said `Po`—still mapped as a non–Rust-whitespace separator);
        // U+3007 IDEOGRAPHIC NUMBER ZERO (`Nl`) stays unmapped with IDEOGRAPHIC ITERATION MARK U+3005 /
        // VERTICAL IDEOGRAPHIC ITERATION MARK U+303B (both `Lm`; see filter below);
        // U+3008–U+3011 / U+3014–U+301B: bracket `Ps`/`Pe` pairs per UnicodeData; U+3012 POSTAL MARK / U+3013 GETA MARK (`So`);
        // U+301C WAVE DASH (`Pd`); U+301D / U+301E–U+301F (UnicodeData: REVERSED DOUBLE PRIME QUOTATION MARK `Ps`, DOUBLE PRIME QUOTATION MARK / LOW DOUBLE PRIME QUOTATION MARK `Pe`); U+3020 POSTAL MARK FACE (`So`); U+3031–U+3035 (`Lm`); U+3036 CIRCLED POSTAL MARK (`So`); U+303C MASU MARK (`So`); U+303E IDEOGRAPHIC VARIATION INDICATOR / U+303F IDEOGRAPHIC HALF FILL SPACE (`So`): not Rust whitespace
        // (see `collapse_whitespace` comment; FEAT-D271 / FEAT-D272 official scalar names). Mixed
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
        // U+3001 IDEOGRAPHIC COMMA / U+3002 IDEOGRAPHIC FULL STOP (`Po` in UnicodeData; colloquial
        // “ideographic comma / full stop”), U+FF0C/FF1A/FF1B/FF01/FF02/FF1F (fullwidth
        // comma, colon, semicolon, exclamation, quotation, question), U+FE10–U+FE19 (Vertical Forms
        // compatibility punctuation): Po/Ps/Pe/Pc—not Rust whitespace. Mixed CJK / Latin HTML
        // or vertical-layout compatibility text can sit between Latin tokens without ASCII space.
        for sep in [
            '\u{3001}', '\u{3002}', '\u{FF0C}', '\u{FF1A}', '\u{FF1B}', '\u{FF01}', '\u{FF02}',
            '\u{FF1F}', '\u{FE10}', '\u{FE11}', '\u{FE12}', '\u{FE13}', '\u{FE14}', '\u{FE15}',
            '\u{FE16}', '\u{FE17}', '\u{FE18}', '\u{FE19}',
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
    fn cjk_compatibility_forms_fe30_through_fe4f_separate_words() {
        // U+FE30–U+FE4F CJK Compatibility Forms: vertical presentation punctuation and spacing
        // (Po / Pd / Ps / Pe / Pc)—not Rust whitespace. U+FE10–U+FE19 remains a separate arm.
        for cp in 0xFE30u32..=0xFE4F {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains("hello world"),
                "U+{:04X} should separate words, got {:?}",
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
    fn combining_half_marks_fe20_through_fe2f_stay_unmapped() {
        // U+FE20–U+FE2F Combining Half Marks (Mn)—must not map to ASCII space (word-internal risk).
        for cp in [0xFE20u32, 0xFE26, 0xFE2F] {
            let sep = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} combining half mark must not split tokens: {:?}",
                cp,
                cleaned
            );
        }
    }

    #[test]
    fn fullwidth_delimiters_operators_and_brackets_separate_words() {
        // U+FF02 QUOTATION MARK, U+FF03 NUMBER SIGN, U+FF04 DOLLAR SIGN, U+FF05 PERCENT, U+FF06
        // AMPERSAND, U+FF08/U+FF09 PARENS, U+FF0A ASTERISK, U+FF0B PLUS, U+FF0D HYPHEN-MINUS,
        // U+FF0E FULL STOP,
        // U+FF1C–U+FF1E LESS/EQUALS/GREATER, U+FF20 COMMERCIAL AT, U+FF3B–U+FF3D BRACKETS/SOLIDUS,
        // U+FF5B–U+FF5D CURLY BRACES, U+FF5C VERTICAL LINE—not Rust whitespace; fullwidth
        // typography HTML can glue Latin tokens without ASCII space. U+FF07 apostrophe, U+FF5E tilde,
        // U+FF40 grave intentionally omitted (see `collapse_whitespace` comment).
        for sep in [
            '\u{FF02}', '\u{FF03}', '\u{FF04}', '\u{FF05}', '\u{FF06}', '\u{FF08}', '\u{FF09}',
            '\u{FF0A}', '\u{FF0B}', '\u{FF0D}', '\u{FF0E}', '\u{FF1C}', '\u{FF1D}', '\u{FF1E}',
            '\u{FF20}', '\u{FF3B}', '\u{FF3C}', '\u{FF3D}', '\u{FF5B}', '\u{FF5C}', '\u{FF5D}',
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
            '\u{FF5F}', '\u{FF60}', '\u{FF61}', '\u{FF62}', '\u{FF63}', '\u{FF64}', '\u{FFE0}',
            '\u{FFE1}', '\u{FFE2}', '\u{FFE4}', '\u{FFE5}', '\u{FFE6}',
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
            '\u{FFE8}', '\u{FFE9}', '\u{FFEA}', '\u{FFEB}', '\u{FFEC}', '\u{FFED}', '\u{FFEE}',
            '\u{FFEF}',
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
    fn devanagari_abbreviation_sign_u0970_and_mandaic_punctuation_u085e_separate_words() {
        // U+0970 DEVANAGARI ABBREVIATION SIGN and U+085E MANDAIC PUNCTUATION (both Po); not Rust whitespace.
        for sep in ['\u{0970}', '\u{085E}'] {
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
    fn devanagari_high_spacing_dot_u0971_and_mandaic_combining_marks_stay_unmapped() {
        // U+0971 (Lm), Devanagari digit zero (U+0966, Nd), and Mandaic combining marks (Mn) must not widen the
        // Devanagari / Mandaic Po arms.
        for cp in [0x0966u32, 0x0971, 0x0859, 0x085A, 0x085B] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn vedic_extensions_nihshvasa_u1cd3_separate_words() {
        // U+1CD3 VEDIC SIGN NIHSHVASA (Po); not Rust whitespace.
        let sep = '\u{1CD3}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains("hello world"),
            "expected U+1CD3 normalized before collapse, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains(sep),
            "cleaned output still contains U+1CD3"
        );
    }

    #[test]
    fn ethiopic_supplement_tonal_marks_so_u1390_through_u1399_separate_words() {
        // Ethiopic Supplement U+1390–U+1399 ETHIOPIC TONAL MARK YIZET through KURT (all So); not Rust whitespace.
        for cp in 0x1390u32..=0x1399 {
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
    fn ethiopic_supplement_syllables_lo_and_unassigned_tail_stay_unmapped() {
        // Ethiopic Supplement syllables (Lo) and unassigned tail must not split Latin tokens.
        for cp in [0x1380u32, 0x138F, 0x139A, 0x139F] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn vedic_tone_marks_mn_u1cd0_u1cd1_u1cd4_stay_unmapped() {
        // Vedic Extensions combining tone marks (Mn); must not widen the Po arm to the full block.
        for cp in [0x1CD0u32, 0x1CD1, 0x1CD4] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
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
    fn ogham_feather_marks_and_duployan_o_cross_and_chinook_full_stop_separate_words() {
        // Ogham U+169B FEATHER MARK (Ps), U+169C REVERSED FEATHER MARK (Pe). Duployan U+1BC9C SIGN O WITH CROSS
        // (So) and U+1BC9F CHINOOK FULL STOP (Po). None are Rust whitespace.
        for sep in ['\u{169B}', '\u{169C}', '\u{1BC9C}', '\u{1BC9F}'] {
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
    fn duployan_unassigned_gap_u1bc9a_u1bc9b_stays_unmapped() {
        // UnicodeData Cn before U+1BC9C; must not widen the So arm to a contiguous range.
        for cp in [0x1BC9Au32, 0x1BC9B] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn tifinagh_separator_and_arabic_supplementary_punctuation_separate_words() {
        // Tifinagh U+2D70 SEPARATOR MARK (Po). Arabic U+0609 / U+060A per mille / per ten thousand (Po),
        // U+060B AFGHANI SIGN (Sc), U+060D DATE SEPARATOR (Po), U+060E POETIC VERSE SIGN / U+060F SIGN MISRA (So), U+061D END OF TEXT MARK (Po), U+061E TRIPLE DOT PUNCTUATION (Po),
        // U+0888 ARABIC RAISED ROUND DOT (Sk), U+066D FIVE POINTED STAR (Po). None are Rust whitespace.
        for sep in [
            '\u{2D70}', '\u{0609}', '\u{060A}', '\u{060B}', '\u{060D}', '\u{060E}', '\u{060F}',
            '\u{061D}', '\u{061E}', '\u{0888}', '\u{066D}',
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
    fn arabic_presentation_forms_ornate_parens_rial_and_bismillah_ligature_separate_words() {
        // Arabic Presentation Forms-A: ornate parentheses U+FD3E / U+FD3F (Pe/Ps), RIAL SIGN U+FDFC (Sc),
        // BISMILLAH ligature U+FDFD (So). None are Rust whitespace.
        for sep in ['\u{FD3E}', '\u{FD3F}', '\u{FDFC}', '\u{FDFD}'] {
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
    fn arabic_presentation_forms_honorific_so_ligatures_fbc3_fbd2_separate_words() {
        // U+FBC3–U+FBD2: honorific / salutation ligatures (all So) before U+FD3E; not Rust whitespace.
        for sep in '\u{FBC3}'..='\u{FBD2}' {
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
    fn arabic_presentation_forms_honorific_so_ligatures_fd40_fd4f_fdcf_fdfe_fdff_separate_words() {
        // U+FD40–U+FD4F, U+FDCF, U+FDFE, U+FDFF: honorific / salutation ligatures (all So); not Rust whitespace.
        let mut seps: Vec<char> = ('\u{FD40}'..='\u{FD4F}').collect();
        seps.push('\u{FDCF}');
        seps.push('\u{FDFE}');
        seps.push('\u{FDFF}');
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
    fn arabic_mathematical_alphabetic_operator_sm_separate_words() {
        // U+1EEF0 ARABIC MATHEMATICAL OPERATOR MEEM WITH HAH WITH TATWEEL, U+1EEF1 ARABIC MATHEMATICAL OPERATOR HAH
        // WITH DAL (Sm); not Rust whitespace.
        for sep in ['\u{1EEF0}', '\u{1EEF1}'] {
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
    fn ottoman_siyaq_marratan_so_separate_words() {
        // U+1ED2E OTTOMAN SIYAQ MARRATAN (So); not Rust whitespace.
        let sep = '\u{1ED2E}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains("hello world"),
            "expected U+1ED2E normalized before collapse, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains(sep),
            "cleaned output still contains U+1ED2E"
        );
    }

    #[test]
    fn ottoman_siyaq_number_forms_stay_unmapped() {
        // Block U+1ED01–U+1ED3D: all assigned scalars except U+1ED2E are No—must not map contiguously.
        for cp in [0x1ED01u32, 0x1ED2D, 0x1ED2F, 0x1ED3D] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn arabic_mathematical_alphabetic_letters_and_block_gaps_stay_unmapped() {
        // Block U+1EE00–U+1EEFF: mathematical Arabic letters (Lo) and large Cn gaps—must not map contiguously.
        for cp in [0x1EE01u32, 0x1EED0, 0x1EEEF] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn arabic_presentation_forms_quranic_symbol_dots_stay_unmapped() {
        // U+FBB2–U+FBC1 ARABIC SYMBOL DOT / TWO DOTS / … / RING / SMALL TAH (Sk)—annotation-like; must not widen
        // to the whole Arabic Presentation Forms-A block (mostly Lo).
        for cp in 0xFBB2u32..=0xFBC1 {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn arabic_presentation_forms_sk_fbc2_and_lo_fbd3_stay_unmapped() {
        // U+FBC2 ARABIC SYMBOL WASLA ABOVE (Sk)—annotation-like; U+FBD3 ARABIC LETTER NG ISOLATED FORM (Lo)—must not map
        // with the honorific So arm U+FBC3..=U+FBD2 (FEAT-D216).
        for cp in [0xFBC2u32, 0xFBD3] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn arabic_presentation_forms_koranic_stop_lo_and_triple_ligatures_stay_unmapped() {
        // U+FDF0–U+FDFB Koranic stop / honorific fragments (Lo); U+FD50+ triple ligatures (Lo)—must not map as the So
        // honorific arm U+FD40..=U+FD4F.
        for cp in [0xFDF0u32, 0xFDF2, 0xFD50] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
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
    fn bengali_currency_sc_u09f2_u09f3_u09fb_separate_words() {
        // U+09F2 BENGALI RUPEE MARK, U+09F3 BENGALI RUPEE SIGN, U+09FB BENGALI GANDA MARK (all Sc); not Rust whitespace.
        for sep in ['\u{09F2}', '\u{09F3}', '\u{09FB}'] {
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
    fn bengali_currency_numerator_no_and_khanda_ta_stay_unmapped() {
        // U+09F4 BENGALI CURRENCY NUMERATOR ONE (No) and U+09FC BENGALI LETTER KHANDA TA (Lo) must not widen the Bengali Sc arms.
        for cp in [0x09F4u32, 0x09FC] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn gujarati_rupee_sign_sc_u0af1_separate_words() {
        // U+0AF1 GUJARATI RUPEE SIGN (Sc); not Rust whitespace. Extends FEAT-D118 Gujarati ABBREVIATION SIGN (Po).
        let sep = '\u{0AF1}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains("hello world"),
            "expected U+0AF1 normalized before collapse, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains(sep),
            "cleaned output still contains U+0AF1"
        );
    }

    #[test]
    fn gujarati_letter_lo_and_digit_nd_stay_unmapped() {
        // U+0A85 GUJARATI LETTER A (Lo) and U+0AE6 GUJARATI DIGIT ZERO (Nd) must not widen the Gujarati Po/Sc arms.
        for cp in [0x0A85u32, 0x0AE6] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn telugu_sign_tuumu_so_u0c7f_separate_words() {
        // U+0C7F TELUGU SIGN TUUMU (So); not Rust whitespace. Extends FEAT-D118 Telugu SIGN SIDDHAM (Po).
        let sep = '\u{0C7F}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains("hello world"),
            "expected U+0C7F normalized before collapse, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains(sep),
            "cleaned output still contains U+0C7F"
        );
    }

    #[test]
    fn telugu_letter_lo_and_digit_nd_stay_unmapped() {
        // U+0C05 TELUGU LETTER A (Lo) and U+0C66 TELUGU DIGIT ZERO (Nd) must not widen the Telugu Po/So arms.
        for cp in [0x0C05u32, 0x0C66] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
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
    fn thai_baht_currency_sc_u0e3f_separate_words() {
        // U+0E3F THAI CURRENCY SYMBOL BAHT (Sc); not Rust whitespace. Extends FEAT-D75 Thai sentence Po marks.
        let sep = '\u{0E3F}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains("hello world"),
            "expected U+0E3F normalized before collapse, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains(sep),
            "cleaned output still contains U+0E3F"
        );
    }

    #[test]
    fn lao_yamak_po_u0edf_separate_words() {
        // U+0EDF LAO YAMAK (Po); not Rust whitespace. Extends FEAT-D75 Lao ellipsis U+0EAF.
        let sep = '\u{0EDF}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains("hello world"),
            "expected U+0EDF normalized before collapse, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains(sep),
            "cleaned output still contains U+0EDF"
        );
    }

    #[test]
    fn lao_letter_khmu_go_u0ede_and_lao_digit_nd_stay_unmapped() {
        // U+0EDE LAO LETTER KHMU GO (Lo) and U+0ED0 LAO DIGIT ZERO (Nd) must not widen the Lao Po arms.
        for cp in [0x0EDEu32, 0x0ED0] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn thai_letter_lo_and_digit_nd_stay_unmapped() {
        // U+0E01 THAI CHARACTER KO KAI (Lo) and U+0E50 THAI DIGIT ZERO (Nd) must not widen the Thai Po/Sc arms.
        for cp in [0x0E01u32, 0x0E50] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn thai_lao_myanmar_sentence_punctuation_separate_words() {
        // Thai: U+0E2F PAIYANNOI, U+0E4F FONGMAN, U+0E5A ANGKHANKHU, U+0E5B KHOMUT (Po). Lao: U+0EAF ELLIPSIS
        // (Po). Myanmar: U+104A LITTLE SECTION, U+104B SECTION (Po); U+104C–U+104F locative / exclamation /
        // completed / aforementioned (So); U+109E–U+109F SHAN ONE / SHAN EXCLAMATION (So). None are Rust whitespace.
        for cp in [0x0E2Fu32, 0x0E4F, 0x0E5A, 0x0E5B, 0x0EAF]
            .into_iter()
            .chain(0x104A..=0x104F)
            .chain(0x109E..=0x109F)
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
    fn myanmar_letter_ka_lo_u1000_stays_unmapped() {
        // U+1000 MYANMAR LETTER KA (`Lo`) must not split like the block-tail Shan `So` symbols U+109E–U+109F.
        let sep = '\u{1000}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "Myanmar letter ka U+1000 (Lo) must stay unmapped, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
    }

    #[test]
    fn myanmar_shan_digit_nd_u1090_stays_unmapped() {
        // U+1090 MYANMAR SHAN DIGIT ZERO (`Nd`) must not split like U+109E–U+109F (`So`).
        let sep = '\u{1090}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "Myanmar Shan digit U+1090 (Nd) must stay unmapped, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
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
        // U+05BE (Hebrew maqaf, Pd), U+05C0 (Hebrew paseq, Po), U+05C6 (Hebrew nun hafukha, Po),
        // U+0F0B (Tibetan tsheg, Po), U+1362 (Ethiopic full stop, Po; contiguous Ethiopic clause punctuation is U+1360–U+1368)—none are Rust whitespace; RTL or Ethiopic-layout HTML
        // can glue Latin tokens for `split_whitespace()` without ASCII space.
        for sep in ['\u{05BE}', '\u{05C0}', '\u{05C6}', '\u{0F0B}', '\u{1362}'] {
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
    fn hebrew_alternative_plus_sign_sm_fb29_separate_words() {
        // U+FB29 HEBREW LETTER ALTERNATIVE PLUS SIGN (`Sm`); not Rust whitespace (FEAT-D232).
        let sep = '\u{FB29}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains("hello world"),
            "expected U+FB29 normalized before collapse, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains(sep),
            "cleaned output still contains U+FB29, got {:?}",
            cleaned
        );
    }

    #[test]
    fn hebrew_geresh_gershayim_po_u05f3_u05f4_stay_unmapped() {
        // U+05F3 GERESH and U+05F4 GERSHAYIM (Po) are omitted from `collapse_whitespace()`—apostrophe- /
        // abbreviation-like, word-internal risk (same spirit as U+2019).
        for sep in ['\u{05F3}', '\u{05F4}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} must stay unmapped, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                cleaned.contains(sep),
                "expected U+{:04X} to remain in output, got {:?}",
                sep as u32,
                cleaned
            );
        }
    }

    #[test]
    fn hebrew_letter_aleph_lo_u05d0_stay_unmapped() {
        let sep = '\u{05D0}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "Hebrew letter U+05D0 (Lo) must stay unmapped, got {:?}",
            cleaned
        );
        assert!(cleaned.contains(sep), "expected {:?} in {:?}", sep, cleaned);
    }

    #[test]
    fn khmer_sentence_signs_and_javanese_pada_separate_words() {
        // Khmer U+17D4 SIGN KHAN through U+17DA SIGN KOOMUUT (Po). Javanese U+A9C1 LEFT RERENGGAN through U+A9CF SECTION SIGN (Po)
        // plus U+A9DE PADA TIRTA TUMETES and U+A9DF PADA ISEN-ISEN (Po; after digits U+A9D0–U+A9D9, FEAT-D215).
        // U+A9C0 PANGKON (Cf) omitted. None of the mapped code points are Rust whitespace.
        let mut seps: Vec<char> = (0x17D4..=0x17DA).filter_map(char::from_u32).collect();
        seps.extend('\u{A9C1}'..='\u{A9CF}');
        seps.push('\u{A9DE}');
        seps.push('\u{A9DF}');
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
    fn javanese_digit_nd_u_a9d0_stays_unmapped() {
        // U+A9D0 JAVANESE DIGIT ZERO (Nd) must not be swept into a widened Javanese pada arm (U+A9DE–U+A9DF map separately).
        let sep = '\u{A9D0}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains(sep),
            "Javanese digit U+A9D0 must pass through collapse_whitespace, got {:?}",
            cleaned
        );
        assert_eq!(
            cleaned.split_whitespace().count(),
            1,
            "U+A9D0 must not become a word break"
        );
    }

    #[test]
    fn khmer_riel_sign_sc_u17db_separate_words() {
        // U+17DB KHMER CURRENCY SYMBOL RIEL (Sc); not Rust whitespace. Extends FEAT-D78 Khmer sentence Po U+17D4–U+17DA.
        let sep = '\u{17DB}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains("hello world"),
            "expected U+17DB normalized before collapse, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains(sep),
            "cleaned output still contains U+17DB"
        );
    }

    #[test]
    fn khmer_sign_avakrahasanya_lo_and_atthacan_mn_stay_unmapped() {
        // U+17DC KHMER SIGN AVAKRAHASANYA (Lo) and U+17DD KHMER SIGN ATTHACAN (Mn) must not widen the Khmer Po/Sc arms.
        for cp in [0x17DCu32, 0x17DD] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn khmer_symbols_so_u19e0_through_u19ff_separate_words() {
        // Unicode block U+19E0–U+19FF: all 32 assigned scalars are So (PATHAMASAT, KOET/ROC lunar abbreviations, etc.);
        // not Rust whitespace. Distinct from Khmer script U+1780+ and currency U+17DB (FEAT-D196).
        for cp in 0x19E0u32..=0x19FF {
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
    fn khmer_digit_nd_and_lek_attak_no_stay_unmapped() {
        // U+17E0 KHMER DIGIT ZERO (Nd) and U+17F0 KHMER SYMBOL LEK ATTAK SON (No) must not widen the Khmer Po/Sc/So arms.
        for cp in [0x17E0u32, 0x17F0] {
            let c = char::from_u32(cp).expect("valid scalar");
            let html = format!("<html><body><p>hello{c}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                cleaned.contains(c),
                "U+{:04X} should pass through collapse_whitespace, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn balinese_inverted_carik_sentence_po_musical_so_and_lantang_separate_words() {
        // Po U+1B4E–U+1B4F, U+1B5A–U+1B60, U+1B7D–U+1B7F; So U+1B61–U+1B6A, U+1B74–U+1B7C. Not Rust whitespace.
        let seps: Vec<char> = ('\u{1B4E}'..='\u{1B4F}')
            .chain('\u{1B5A}'..='\u{1B60}')
            .chain('\u{1B61}'..='\u{1B6A}')
            .chain('\u{1B74}'..='\u{1B7F}')
            .collect();
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
    fn balinese_digits_and_combining_musical_marks_stay_unmapped() {
        // Nd U+1B50–U+1B59; Mn U+1B6B–U+1B73 (combining tegeh through gong).
        for cp in (0x1B50_u32..=0x1B59).chain(0x1B6B..=0x1B73) {
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
    fn philippine_single_double_punctuation_separate_words() {
        // U+1735 PHILIPPINE SINGLE PUNCTUATION, U+1736 PHILIPPINE DOUBLE PUNCTUATION (Po). None are Rust whitespace.
        for sep in '\u{1735}'..='\u{1736}' {
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
    fn hanunoo_sign_pamudpod_mn_u1734_stays_unmapped() {
        // U+1734 HANUNOO SIGN PAMUDPOD (`Mn`); not a Philippine sentence Po—must not be folded to ASCII space.
        let sep = '\u{1734}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            cleaned.contains(sep),
            "expected U+1734 to remain in cleaned output, got {:?}",
            cleaned
        );
        assert!(
            !cleaned.contains("hello world"),
            "combining mark U+1734 must not insert a word boundary between Latin tokens"
        );
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
        // U+1400 CANADIAN SYLLABICS HYPHEN (Pd). U+166E CANADIAN SYLLABICS FULL STOP (Po). Bamum U+A6F2 NJAEMLI through
        // U+A6F7 QUESTION MARK (Po). Mro U+16A6E DANDA, U+16A6F DOUBLE DANDA (Po). New Tai Lue U+19DE SIGN LAE, U+19DF
        // SIGN LAEV (So). None are Rust whitespace.
        let mut seps: Vec<char> = vec!['\u{1400}', '\u{166E}'];
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
        // Meetei Mayek Extensions U+AAF0 CHEIKHAN, U+AAF1 AHANG KHUDAM (Po). None are Rust whitespace.
        // U+AAF2 MEETEI MAYEK ANJI is `Lo`—see `meetei_mayek_anji_lo_u_aaf2_stays_unmapped`.
        for sep in [
            '\u{0DF4}', '\u{1940}', '\u{1944}', '\u{1945}', '\u{AAF0}', '\u{AAF1}',
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
    fn meetei_mayek_heavy_half_tone_lm_aaf3_aaf4_stay_unmapped() {
        // U+AAF3 SYLLABLE HEAVY TONE and U+AAF4 SYLLABLE HALF TONE (`Lm`) must not split tokens like U+AAF0–U+AAF1 `Po`.
        for cp in [0xAAF3u32, 0xAAF4] {
            let sep = char::from_u32(cp).unwrap();
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} must stay unmapped, got {:?}",
                cp,
                cleaned
            );
            assert!(
                cleaned.contains(sep),
                "U+{:04X} should pass through, got {:?}",
                cp,
                cleaned
            );
            assert_eq!(
                cleaned.split_whitespace().count(),
                1,
                "U+{:04X} must not become a word break",
                cp
            );
        }
    }

    #[test]
    fn meetei_mayek_anji_lo_u_aaf2_stays_unmapped() {
        // UnicodeData: U+AAF2 MEETEI MAYEK ANJI (`Lo`), not `Po`; mapping it split script text (historical FEAT-D218 mismatch, FEAT-D221).
        let sep = '\u{AAF2}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+AAF2 must stay unmapped, got {:?}",
            cleaned
        );
        assert!(
            cleaned.contains(sep),
            "U+AAF2 should pass through, got {:?}",
            cleaned
        );
        assert_eq!(
            cleaned.split_whitespace().count(),
            1,
            "U+AAF2 must not become a word break"
        );
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
    fn ethiopic_section_wordspace_and_sentence_punctuation_u1360_through_u1368_separate_words() {
        // U+1360 SECTION MARK through U+1368 PARAGRAPH SEPARATOR (all Po)—not Rust whitespace; extends FEAT-D48
        // wordspace (U+1361) / full stop (U+1362). U+1369+ Ethiopic digit numerics (`No`) stay unmapped.
        for cp in 0x1360u32..=0x1368 {
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
    fn ethiopic_digit_one_no_u1369_and_number_ten_no_u1372_stay_unmapped() {
        // U+1369 ETHIOPIC DIGIT ONE and U+1372 ETHIOPIC NUMBER TEN (`No`)—numeric / word-internal risk; must not split Latin tokens.
        for sep in ['\u{1369}', '\u{1372}'] {
            let html = format!("<html><body><p>hello{sep}world</p></body></html>");
            let cleaned = clean_html(&html);
            assert!(
                !cleaned.contains("hello world"),
                "U+{:04X} No should not split tokens like Ethiopic Po U+1360–U+1368, got {:?}",
                sep as u32,
                cleaned
            );
            assert!(
                cleaned.contains(sep),
                "expected raw U+{:04X} preserved in output",
                sep as u32
            );
        }
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
    fn mahjong_domino_and_playing_cards_separate_words() {
        // U+1F000–U+1F02F Mahjong, U+1F030–U+1F093 Domino, U+1F0A0–U+1F0FF Playing Cards (all So); not Rust whitespace.
        // U+1F094–U+1F09F reserved—excluded.
        let cps = (0x1F000u32..=0x1F02F)
            .chain(0x1F030..=0x1F093)
            .chain(0x1F0A0..=0x1F0FF);
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
    fn mahjong_domino_playing_cards_reserved_gap_stays_unmapped() {
        // U+1F094–U+1F09F between Domino and Playing Cards blocks are reserved.
        for cp in [0x1F094u32, 0x1F098, 0x1F09F] {
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
    fn enclosed_alphanumeric_supplement_assigned_separate_words() {
        // U+1F100–U+1F1AD and U+1F1E6–U+1F1FF: enclosed Latin/digits and regional indicators (No / So); not Rust
        // whitespace. U+1F1AE–U+1F1E5 unassigned—excluded from implementation.
        let cps = (0x1F100u32..=0x1F1AD).chain(0x1F1E6..=0x1F1FF);
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
    fn enclosed_alphanumeric_supplement_unassigned_gap_stays_unmapped() {
        // U+1F1AE–U+1F1E5 are unassigned; must not be forced to ASCII space like assigned supplement scalars.
        for cp in [0x1F1AEu32, 0x1F1B0, 0x1F1E5] {
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
    fn cjk_radicals_supplement_so_assigned_subranges_separate_words() {
        // CJK Radicals Supplement: So U+2E80–U+2E99, U+2E9B–U+2EF3; not Rust whitespace.
        let cps = (0x2E80u32..=0x2E99).chain(0x2E9B..=0x2EF3);
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
    fn cjk_radicals_supplement_unassigned_gaps_stay_unmapped() {
        // Cn: U+2E9A, U+2EF4–U+2EFF—not included in supplement So arms.
        for cp in [0x2E9Au32, 0x2EF4, 0x2EFF] {
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
    fn kangxi_radicals_so_u2f00_through_u2fd5_separate_words() {
        // Kangxi Radicals: all assigned scalars U+2F00–U+2FD5 are So; not Rust whitespace.
        for cp in 0x2F00u32..=0x2FD5 {
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
    fn kangxi_radicals_unassigned_tail_and_inter_block_gap_stay_unmapped() {
        // Cn: U+2FD6–U+2FDF (tail of Kangxi block), U+2FE0–U+2FEF (gap before IDC U+2FF0+).
        for cp in [0x2FD6u32, 0x2FDF, 0x2FE0, 0x2FEF] {
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
    fn kangxi_radicals_neighbor_cjk_ext_a_stay_unmapped() {
        // CJK Unified Ideographs Extension A U+3400 (`Lo`)—word-internal risk; not a Kangxi radical.
        let cp = 0x3400u32;
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

    #[test]
    fn kanbun_so_marks_and_cjk_strokes_separate_words() {
        // Kanbun: So U+3190–U+3191, U+3196–U+319F; CJK Strokes U+31C0–U+31E5 (all So as assigned in Unicode 17). Not Rust whitespace.
        let cps = (0x3190u32..=0x3191)
            .chain(0x3196..=0x319F)
            .chain(0x31C0..=0x31E5);
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
    fn kanbun_no_numerics_bopomofo_ext_and_strokes_gap_stay_unmapped() {
        // Kanbun No U+3192–U+3195; Bopomofo Extended Lo U+31A0–U+31BF; CJK Strokes inner gap U+31E6–U+31EE (`Cn`); Katakana Phonetic Ext Lo U+31F0.
        for cp in [0x3192u32, 0x3195, 0x31A0, 0x31BF, 0x31E6, 0x31EE, 0x31F0] {
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
    fn cjk_strokes_inner_gap_u31e6_through_u31ee_stay_unmapped() {
        // Unicode 17 assigns CJK Strokes U+31C0–U+31E5; U+31E6–U+31EE remain unassigned (`Cn`) and must not split Latin tokens.
        for cp in 0x31E6u32..=0x31EE {
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
    fn enclosed_cjk_letters_and_months_so_assigned_subranges_separate_words() {
        // U+3200–U+32FF block: all assigned So subranges; not Rust whitespace. No numerics and unassigned U+321F excluded in
        // implementation—see `enclosed_cjk_letters_and_months_no_numerics_and_gap_stay_unmapped`.
        let cps = (0x3200u32..=0x321E)
            .chain(0x322A..=0x3247)
            .chain(std::iter::once(0x3250))
            .chain(0x3260..=0x327F)
            .chain(0x328A..=0x32B0)
            .chain(0x32C0..=0x32FF);
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
    fn enclosed_cjk_letters_and_months_no_numerics_and_gap_stay_unmapped() {
        // No: U+3220–U+3229, U+3248–U+324F, U+3251–U+325F, U+3280–U+3289, U+32B1–U+32BF. Cn: U+321F.
        for cp in [
            0x321Fu32, 0x3220, 0x3229, 0x3248, 0x324F, 0x3251, 0x325F, 0x3280, 0x3289, 0x32B1,
            0x32BF,
        ] {
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
    fn cjk_compatibility_u3300_through_u33ff_all_so_separate_words() {
        // U+3300–U+33FF: UnicodeData assigns every scalar as So; not Rust whitespace.
        for cp in 0x3300u32..=0x33FF {
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
    fn cjk_compatibility_neighbor_cjk_ext_a_stay_unmapped() {
        // CJK Unified Ideographs Extension A starts at U+3400 (Lo)—must not split Latin tokens like squared katakana So.
        let sep = char::from_u32(0x3400).expect("valid scalar");
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+3400 should not map to ASCII space between Latin tokens, got {:?}",
            cleaned
        );
    }

    #[test]
    fn enclosed_ideographic_supplement_assigned_separate_words() {
        // U+1F200–U+1F202, U+1F210–U+1F23B, U+1F240–U+1F248, U+1F250–U+1F251, U+1F260–U+1F265: squared / bracketed /
        // circled / rounded CJK symbols (all So); not Rust whitespace. Inter-range gaps unassigned—excluded.
        let cps = (0x1F200u32..=0x1F202)
            .chain(0x1F210..=0x1F23B)
            .chain(0x1F240..=0x1F248)
            .chain(0x1F250..=0x1F251)
            .chain(0x1F260..=0x1F265);
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
    fn enclosed_ideographic_supplement_unassigned_gaps_stay_unmapped() {
        // U+1F203–U+1F20F, U+1F23C–U+1F23F, U+1F249–U+1F24F, U+1F252–U+1F25F, U+1F266–U+1F2FF unassigned.
        for cp in [
            0x1F203u32, 0x1F20A, 0x1F23C, 0x1F249, 0x1F252, 0x1F266, 0x1F2FF,
        ] {
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
    fn miscellaneous_symbols_pictographs_and_emoticons_u1f300_through_u1f67f_separate_words() {
        // U+1F300–U+1F5FF Miscellaneous Symbols and Pictographs, U+1F600–U+1F64F Emoticons, U+1F650–U+1F67F
        // ornamental dingbats (mostly So); not Rust whitespace.
        for cp in 0x1F300u32..=0x1F67F {
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
    fn transport_and_map_symbols_assigned_subranges_separate_words() {
        // U+1F680–U+1F6D7, U+1F6E0–U+1F6EC, U+1F6F0–U+1F6FC (mostly So); not Rust whitespace.
        let cps = (0x1F680u32..=0x1F6D7)
            .chain(0x1F6E0..=0x1F6EC)
            .chain(0x1F6F0..=0x1F6FC);
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
    fn transport_and_map_symbols_unassigned_gaps_stay_unmapped() {
        // U+1F6D8–U+1F6DF, U+1F6ED–U+1F6EF, U+1F6FD–U+1F6FF unassigned (Unicode 16).
        for cp in [0x1F6D8u32, 0x1F6DF, 0x1F6ED, 0x1F6EF, 0x1F6FD, 0x1F6FF] {
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
    fn alchemical_symbols_u1f700_through_u1f773_separate_words() {
        // U+1F700–U+1F773 (all So); not Rust whitespace.
        for cp in 0x1F700u32..=0x1F773 {
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
    fn alchemical_symbols_unassigned_tail_u1f774_through_u1f77f_stays_unmapped() {
        // U+1F774–U+1F77F unassigned (Unicode 16).
        for cp in [0x1F774u32, 0x1F77A, 0x1F77F] {
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
    fn geometric_shapes_extended_assigned_subranges_separate_words() {
        // U+1F780–U+1F7D8 and U+1F7E0–U+1F7EB (mostly So); not Rust whitespace.
        let cps = (0x1F780u32..=0x1F7D8).chain(0x1F7E0..=0x1F7EB);
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
    fn geometric_shapes_extended_unassigned_gaps_stay_unmapped() {
        // U+1F7D9–U+1F7DF and U+1F7EC–U+1F7FF unassigned.
        for cp in [0x1F7D9u32, 0x1F7DF, 0x1F7EC, 0x1F7FF] {
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
    fn supplemental_arrows_c_assigned_subranges_separate_words() {
        // Unicode 16 assigned subranges within Supplemental Arrows-C (mostly Sm); not Rust whitespace.
        let cps = (0x1F800u32..=0x1F80B)
            .chain(0x1F810..=0x1F847)
            .chain(0x1F850..=0x1F859)
            .chain(0x1F860..=0x1F887)
            .chain(0x1F890..=0x1F8AD)
            .chain(0x1F8B0..=0x1F8B1);
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
    fn supplemental_arrows_c_unassigned_gaps_stay_unmapped() {
        // Unassigned gaps within U+1F800–U+1F8FF (Unicode 16).
        for cp in [
            0x1F80C_u32,
            0x1F848,
            0x1F85A,
            0x1F888,
            0x1F8AE,
            0x1F8B2,
            0x1F8FF,
        ] {
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
    fn supplemental_symbols_and_pictographs_u1f900_through_u1f9ff_separate_words() {
        // Supplemental Symbols and Pictographs: contiguous U+1F900–U+1F9FF (mostly So / emoji presentation);
        // not Rust whitespace.
        for cp in 0x1F900u32..=0x1F9FF {
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
    fn chess_symbols_assigned_subranges_separate_words() {
        // Unicode Chess Symbols: U+1FA00–U+1FA57 and U+1FA60–U+1FA6D (mostly So); not Rust whitespace.
        let cps = (0x1FA00u32..=0x1FA57).chain(0x1FA60..=0x1FA6D);
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
    fn chess_symbols_unassigned_gaps_stay_unmapped() {
        for cp in [0x1FA58_u32, 0x1FA5F, 0x1FA6E, 0x1FA6F] {
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
    fn symbols_and_pictographs_extended_a_assigned_subranges_separate_words() {
        // Symbols and Pictographs Extended-A: assigned subranges (mostly So / emoji presentation); not Rust whitespace.
        let cps = (0x1FA70u32..=0x1FA7C)
            .chain(0x1FA80..=0x1FA8A)
            .chain(0x1FA8E..=0x1FA8F)
            .chain(0x1FA90..=0x1FAC6)
            .chain(std::iter::once(0x1FAC8))
            .chain(0x1FACD..=0x1FACF)
            .chain(0x1FAD0..=0x1FADC)
            .chain(std::iter::once(0x1FADF))
            .chain(0x1FAE0..=0x1FAEA)
            .chain(std::iter::once(0x1FAEF))
            .chain(0x1FAF0..=0x1FAF8);
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
    fn symbols_and_pictographs_extended_a_unassigned_gaps_stay_unmapped() {
        for cp in [
            0x1FA7D_u32,
            0x1FA8B,
            0x1FAC7,
            0x1FAC9,
            0x1FADD,
            0x1FAEB,
            0x1FAF9,
        ] {
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
    fn symbols_for_legacy_computing_assigned_so_subranges_separate_words() {
        // Symbols for Legacy Computing: So U+1FB00–U+1FB92 and U+1FB94–U+1FBEF; not Rust whitespace.
        let cps = (0x1FB00u32..=0x1FB92).chain(0x1FB94..=0x1FBEF);
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
    fn symbols_for_legacy_computing_unassigned_gap_nd_and_tail_stay_unmapped() {
        for cp in [0x1FB93_u32, 0x1FBF0, 0x1FBF5, 0x1FBF9, 0x1FBFA, 0x1FBFF] {
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
    fn symbols_for_legacy_computing_supplement_assigned_subranges_separate_words() {
        // Symbols for Legacy Computing Supplement: So U+1CC00–U+1CCFC, U+1CD00–U+1CEB3, U+1CEBA–U+1CEBF; not Rust whitespace.
        let cps = (0x1CC00u32..=0x1CCFC)
            .chain(0x1CD00..=0x1CEB3)
            .chain(0x1CEBA..=0x1CEBF);
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
    fn symbols_for_legacy_computing_supplement_unassigned_gaps_stay_unmapped() {
        for cp in [0x1CCFD_u32, 0x1CCFE, 0x1CCFF, 0x1CEB5, 0x1CEB8] {
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
    fn miscellaneous_symbols_supplement_assigned_subranges_separate_words() {
        // Miscellaneous Symbols Supplement: So U+1CEC0–U+1CED0, U+1CEE0–U+1CEEF, U+1CEF0; not Rust whitespace.
        let cps = (0x1CEC0u32..=0x1CED0)
            .chain(0x1CEE0..=0x1CEEF)
            .chain(std::iter::once(0x1CEF0));
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
    fn miscellaneous_symbols_supplement_unassigned_gaps_stay_unmapped() {
        for cp in [0x1CED1_u32, 0x1CEDF, 0x1CEF1, 0x1CEFF] {
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
    fn byzantine_musical_symbols_assigned_subrange_separate_words() {
        // Byzantine Musical Symbols: So U+1D000–U+1D0F5; not Rust whitespace.
        for cp in 0x1D000u32..=0x1D0F5 {
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
    fn byzantine_musical_symbols_unassigned_tail_stays_unmapped() {
        for cp in [0x1D0F6_u32, 0x1D0FA, 0x1D0FF] {
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
    fn western_musical_symbols_assigned_subranges_separate_words() {
        // Western Musical Symbols: UnicodeData-assigned U+1D100–U+1D126, U+1D129–U+1D172, U+1D17B–U+1D1EA; not Rust whitespace.
        for cp in (0x1D100u32..=0x1D126)
            .chain(0x1D129..=0x1D172)
            .chain(0x1D17B..=0x1D1EA)
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
    fn western_musical_symbols_unassigned_gaps_stay_unmapped() {
        for cp in [0x1D127_u32, 0x1D128, 0x1D1EB, 0x1D1F0, 0x1D1FF] {
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
    fn ancient_greek_musical_notation_assigned_subrange_separate_words() {
        // Ancient Greek Musical Notation: UnicodeData U+1D200–U+1D245 (So + Mn combining); not Rust whitespace.
        for cp in 0x1D200u32..=0x1D245 {
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
    fn ancient_greek_musical_notation_unassigned_tail_stays_unmapped() {
        for cp in [0x1D246_u32, 0x1D24A, 0x1D24F] {
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
    fn tai_xuan_jing_symbols_assigned_subrange_separate_words() {
        // Tai Xuan Jing Symbols: UnicodeData U+1D300–U+1D356 (all So); not Rust whitespace.
        for cp in 0x1D300u32..=0x1D356 {
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
    fn tai_xuan_jing_unassigned_gaps_and_numeric_neighbors_stay_unmapped() {
        // Unassigned before block; Mayan numerals (No); gap after So block; Counting Rod digits (No).
        for cp in [0x1D280_u32, 0x1D2E0, 0x1D357, 0x1D360, 0x1D378] {
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
    fn mathematical_alphanumeric_nabla_partial_differential_sm_separate_words() {
        // UnicodeData Sm only: styled ∇ / ∂ in Mathematical Alphanumeric Symbols; not Rust whitespace.
        for cp in [
            0x1D6C1_u32,
            0x1D6DB,
            0x1D6FB,
            0x1D715,
            0x1D735,
            0x1D74F,
            0x1D76F,
            0x1D789,
            0x1D7A9,
            0x1D7C3,
        ] {
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
    fn mathematical_alphanumeric_letters_digits_and_block_gaps_stay_unmapped() {
        // Lu/Ll/Nd and a Cn gap in U+1D400–U+1D7FF must not become word separators.
        for cp in [0x1D400_u32, 0x1D455, 0x1D7F6] {
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
    fn sutton_signwriting_so_assigned_subranges_separate_words() {
        // Sutton SignWriting: UnicodeData So U+1D800–U+1D9FF, U+1DA37–U+1DA3A, U+1DA6D–U+1DA74, U+1DA76–U+1DA83,
        // U+1DA85–U+1DA86; not Rust whitespace.
        let ranges = [
            0x1D800u32..=0x1D9FF,
            0x1DA37..=0x1DA3A,
            0x1DA6D..=0x1DA74,
            0x1DA76..=0x1DA83,
            0x1DA85..=0x1DA86,
        ];
        for r in ranges {
            for cp in r {
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
    }

    #[test]
    fn yijing_hexagram_symbols_u4dc0_through_u4dff_separate_words() {
        // Yijing Hexagram Symbols: UnicodeData So U+4DC0–U+4DFF (64 hexagrams); not Rust whitespace.
        for cp in 0x4DC0_u32..=0x4DFF {
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
    fn yi_radicals_u_a490_through_u_a4c6_all_so_separate_words() {
        // Yi Radicals: UnicodeData So U+A490–U+A4C6 (55 assigned scalars); not Rust whitespace.
        for cp in 0xA490_u32..=0xA4C6 {
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
    fn yi_radicals_unassigned_tail_and_neighbor_lisu_stay_unmapped() {
        // Tail U+A4C7–U+A4CF (Cn); Lisu U+A4D0 LETTER BA (Lo)—must not split Latin tokens.
        for cp in [0xA4C7_u32, 0xA4CF, 0xA4D0] {
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
    fn slavonic_asterisk_kavyka_syloti_poetry_north_indic_marks_separate_words() {
        // Cyrillic Extended-B Po U+A673 / U+A67E; Syloti Nagri So U+A828–U+A82B; North Indic So/Sc U+A836–U+A839.
        let cps = [0xA673_u32, 0xA67E]
            .into_iter()
            .chain(0xA828..=0xA82B)
            .chain(0xA836..=0xA839);
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
    fn slavonic_kavyka_syloti_north_indic_fractions_and_letter_neighbors_stay_unmapped() {
        // Cyrillic Ext-B Me/Mn/Lm; Syloti Lo/Mc; North Indic fraction numerics (No)—must not split Latin tokens.
        for cp in [0xA672_u32, 0xA674, 0xA67F, 0xA800, 0xA827, 0xA830, 0xA835] {
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
    fn yijing_neighbors_cjk_ext_a_and_unified_stay_unmapped() {
        // CJK Extension A ends at U+4DBF (Lo); CJK Unified starts at U+4E00 (Lo)—must not split Latin tokens.
        for cp in [0x4DBF_u32, 0x4E00] {
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
    fn ancient_symbols_assigned_so_and_phaistos_disc_logograms_separate_words() {
        // Ancient Symbols: UnicodeData So U+10190–U+1019C and U+101A0; Phaistos Disc: So U+101D0–U+101FC.
        let cps = (0x10190..=0x1019C)
            .chain(std::iter::once(0x101A0))
            .chain(0x101D0..=0x101FC);
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
    fn ancient_symbols_block_gaps_phaistos_mn_and_tail_unassigned_stay_unmapped() {
        // Ancient Symbols unassigned U+1019D–U+1019F / U+101A1; Phaistos Mn U+101FD; tail U+101FE–U+101FF (Cn).
        for cp in [
            0x1019D_u32,
            0x1019E,
            0x1019F,
            0x101A1,
            0x101FD,
            0x101FE,
            0x101FF,
        ] {
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
    fn ancient_greek_numbers_metrological_so_subranges_separate_words() {
        // Ancient Greek Numbers: UnicodeData So U+10179–U+10189 and U+1018C–U+1018E; not Rust whitespace.
        let cps = (0x10179_u32..=0x10189).chain(0x1018C..=0x1018E);
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
    fn ancient_greek_numbers_nl_no_and_unassigned_gap_stay_unmapped() {
        // Nl acrophonic U+10140; No fractions U+10175–U+10178; No zero / quarter U+1018A–U+1018B; unassigned U+1018F.
        for cp in [0x10140_u32, 0x10175, 0x10178, 0x1018A, 0x1018B, 0x1018F] {
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
    fn znamenny_neume_so_assigned_subrange_separate_words() {
        // Znamenny Musical Notation: UnicodeData So U+1CF50–U+1CFC3 (neumes); not Rust whitespace.
        for cp in 0x1CF50_u32..=0x1CFC3 {
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
    fn znamenny_combining_mn_and_block_gaps_stay_unmapped() {
        // Mn combining marks, unassigned gaps U+1CF2E–U+1CF2F / U+1CF47–U+1CF4F, and tail U+1CFC4–U+1CFCF must not split Latin tokens.
        for cp in [
            0x1CF00_u32,
            0x1CF2D,
            0x1CF2E,
            0x1CF2F,
            0x1CF30,
            0x1CF46,
            0x1CF47,
            0x1CF4F,
            0x1CFC4,
            0x1CFCF,
        ] {
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
    fn sutton_signwriting_mn_unassigned_gap_stay_unmapped() {
        // Mn combining marks, unassigned gap U+1DA8C–U+1DA9A, and block tail Mn after Po must not split Latin tokens.
        for cp in [
            0x1DA00_u32,
            0x1DA36,
            0x1DA3B,
            0x1DA6C,
            0x1DA75,
            0x1DA84,
            0x1DA8C,
            0x1DA90,
            0x1DA9B,
            0x1DAA1,
            0x1DAAF,
        ] {
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
        // (Po) and U+1BC9C SIGN O WITH CROSS (So) are covered by
        // `ogham_feather_marks_and_duployan_o_cross_and_chinook_full_stop_separate_words`.
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
        // Ahom U+1173C–U+1173E (Po) and U+1173F SYMBOL VI (So); Dogra U+1183B; Dives Akuru U+11944–U+11946;
        // Nandinagari U+119E2; Bhaiksuki U+11C41–U+11C45 (all Po).
        for cp in (0x1173Cu32..=0x1173F)
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
    fn ahom_letter_ka_lo_u11700_stays_unmapped() {
        // U+11700 AHOM LETTER KA (`Lo`)—word-internal risk; must not split Latin tokens like Ahom Po/So U+1173C–U+1173F.
        let sep = '\u{11700}';
        let html = format!("<html><body><p>hello{sep}world</p></body></html>");
        let cleaned = clean_html(&html);
        assert!(
            !cleaned.contains("hello world"),
            "U+11700 Lo should not split tokens like Ahom U+1173C–U+1173F, got {:?}",
            cleaned
        );
        assert!(
            cleaned.contains(sep),
            "expected raw U+11700 preserved in output"
        );
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
    fn tamil_supplement_metrological_and_currency_so_sc_separate_words() {
        // Tamil Supplement U+11FD5 TAMIL SIGN NEL through U+11FF1 TAMIL SIGN VAKAIYARAA (all So/Sc).
        for cp in 0x11FD5..=0x11FF1 {
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
    fn tamil_supplement_fraction_numerics_and_unassigned_gap_stay_unmapped() {
        // No U+11FD0–U+11FD4; unassigned tail U+11FF2–U+11FFE (Cn)—must not split Latin tokens.
        for cp in (0x11FD0..=0x11FD4).chain(0x11FF2..=0x11FFE) {
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
        // U+2FF0–U+2FFF and Unicode 17 U+31EF (IDEOGRAPHIC DESCRIPTION CHARACTER SUBTRACTION): So, not Rust whitespace; rare
        // mixed or pedagogical HTML can place them between Latin letters without an ASCII space.
        for cp in (0x2FF0u32..=0x2FFF).chain(std::iter::once(0x31EF)) {
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
