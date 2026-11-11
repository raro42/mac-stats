//! Shared operator-command normalization.

/// Whole-token `age` only — not substrings of average / usage / agent / …
#[inline]
pub fn contains_age_token(n: &str) -> bool {
    n.split(|c: char| !c.is_ascii_alphanumeric())
        .any(|t| t == "age")
}


pub fn normalize_operator_command(content: &str) -> String {
    let n = content
        .trim()
        .trim_start_matches('@')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let n = n
        .trim_start_matches("werner")
        .trim_start_matches(',')
        .trim()
        .trim_start_matches("please")
        .trim()
        .trim_start_matches("can you")
        .trim()
        .trim_start_matches("could you")
        .trim()
        .trim_start_matches("show me")
        .trim()
        .trim_start_matches("show")
        .trim()
        .trim_end_matches('?')
        .trim()
        .to_string();
    n
}

