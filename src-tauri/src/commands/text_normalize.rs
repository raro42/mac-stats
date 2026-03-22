//! Unicode homoglyph normalization for **untrusted** web text (defence in depth).
//!
//! Fullwidth ASCII (Unicode Halfwidth and Fullwidth Forms) is mapped to plain ASCII.
//! Confusable “angle” brackets (guillemets, mathematical angle brackets, CJK quotation
//! marks, fullwidth &lt; &gt;, etc.) are mapped to ASCII `<` and `>` so delimiter-style
//! spoofing is easier to detect downstream. This is **not** a substitute for random
//! boundary markers: page content remains attacker-controlled.

use crate::config::Config;

/// Map a single scalar to its normalized form; returns `None` if unchanged.
#[inline]
fn map_homoglyph(c: char) -> Option<char> {
    let cp = c as u32;
    // Ideographic space → ASCII space (common in CJK pages; can glue spoofed tokens).
    if cp == 0x3000 {
        return Some(' ');
    }
    // Halfwidth and Fullwidth Forms: U+FF01..=U+FF5E → U+0021..=U+007E
    if (0xFF01..=0xFF5E).contains(&cp) {
        return char::from_u32(cp - 0xFEE0);
    }
    // Angle-bracket and guillemet confusables → '<' or '>'
    match cp {
        0x00AB | 0x2039 | 0x2329 | 0x27E8 | 0x27EA | 0x27EC | 0x27EE | 0x3008 | 0x300A
        | 0xFE64 | 0xFF1C => Some('<'),
        0x00BB | 0x203A | 0x232A | 0x27E9 | 0x27EB | 0x27ED | 0x27EF | 0x3009 | 0x300B
        | 0xFE65 | 0xFF1E => Some('>'),
        _ => None,
    }
}

/// OpenClaw-style homoglyph normalization for untrusted text.
/// Returns `(output, count)` where `count` is how many Unicode scalars were replaced.
pub fn normalize_untrusted_homoglyphs(input: &str) -> (String, u32) {
    let mut out = String::with_capacity(input.len());
    let mut n = 0u32;
    for c in input.chars() {
        if let Some(m) = map_homoglyph(c) {
            out.push(m);
            n += 1;
        } else {
            out.push(c);
        }
    }
    (out, n)
}

/// If enabled in config / env, normalize and log at debug when anything changed.
pub fn apply_untrusted_homoglyph_normalization(text: String) -> String {
    if !Config::normalize_untrusted_homoglyphs_enabled() {
        return text;
    }
    let (out, n) = normalize_untrusted_homoglyphs(&text);
    if n > 0 {
        tracing::debug!(
            "Untrusted text: homoglyph normalization rewrote {} Unicode scalar(s)",
            n
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fullwidth_fetch_url_token_normalizes() {
        let s = "ＦＥＴＣＨ＿ＵＲＬ: https://example.com";
        let (out, n) = normalize_untrusted_homoglyphs(s);
        assert!(n > 0);
        assert!(
            out.contains("FETCH_URL:") || out.contains("FETCH_URL"),
            "got {:?}",
            out
        );
    }

    #[test]
    fn unicode_angle_brackets_normalize() {
        let s = "a\u{3008}tag\u{3009}b";
        let (out, _) = normalize_untrusted_homoglyphs(s);
        assert_eq!(out, "a<tag>b");
    }

    #[test]
    fn guillemets_normalize() {
        let s = "\u{00AB}FAKE\u{00BB}";
        let (out, _) = normalize_untrusted_homoglyphs(s);
        assert_eq!(out, "<FAKE>");
    }

    #[test]
    fn ascii_unchanged() {
        let s = "normal FETCH_URL: https://x.test";
        let (out, n) = normalize_untrusted_homoglyphs(s);
        assert_eq!(n, 0);
        assert_eq!(out, s);
    }
}
