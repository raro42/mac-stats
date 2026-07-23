//! Shared search result shaping for web search tools (Perplexity, Brave).
//!
//! Provides: snippet truncation, domain deduplication, result cap, and optional
//! head+tail truncation of the final formatted blob for the model.

use std::collections::HashMap;

/// One search result item suitable for shaping and formatting.
#[derive(Debug, Clone)]
pub struct ShapableSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub date: Option<String>,
}

/// Normalize URL to domain for deduplication (strip www).
fn normalized_domain(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| {
            u.host_str()
                .map(|s| s.trim_start_matches("www.").to_string())
        })
        .unwrap_or_default()
}

/// Shape results: truncate snippets, limit per domain, cap total count.
/// Returns results in order; drops excess by domain and then by count.
pub fn shape_search_results(
    results: Vec<ShapableSearchResult>,
    snippet_max_chars: usize,
    max_results: usize,
    max_per_domain: usize,
) -> Vec<ShapableSearchResult> {
    let mut per_domain: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::with_capacity(results.len().min(max_results));
    for mut r in results {
        if out.len() >= max_results {
            break;
        }
        let domain = normalized_domain(&r.url);
        let count = per_domain.get(&domain).copied().unwrap_or(0);
        if count >= max_per_domain {
            continue;
        }
        if r.snippet.chars().count() > snippet_max_chars {
            r.snippet = format!(
                "{}…",
                r.snippet
                    .chars()
                    .take(snippet_max_chars)
                    .collect::<String>()
            );
        }
        per_domain.insert(domain, count + 1);
        out.push(r);
    }
    out
}

/// Make long search snippets readable: real newlines, sentence breaks, no run-on walls.
pub fn normalize_snippet_layout(snippet: &str) -> String {
    let mut s = snippet.replace("\\n", "\n").replace("\\r", "");
    s = s.replace('\r', "\n");
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    let mut newline_run = 0u8;
    for ch in s.chars() {
        if ch == '\n' {
            if newline_run < 2 {
                out.push('\n');
                newline_run += 1;
            }
            prev_space = false;
            continue;
        }
        newline_run = 0;
        if ch.is_whitespace() {
            if !prev_space && !out.ends_with('\n') {
                out.push(' ');
                prev_space = true;
            }
            continue;
        }
        prev_space = false;
        out.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            let line_len = out.rsplit('\n').next().map(|l| l.chars().count()).unwrap_or(0);
            if line_len >= 90 {
                out.push('\n');
                newline_run = 1;
                prev_space = false;
            }
        }
    }
    let trimmed = out.trim().to_string();
    if trimmed.is_empty() {
        return String::new();
    }
    trimmed
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| format!("> {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format shaped results as markdown. If the resulting string exceeds max_chars,
/// apply head+tail truncation (keep start and end, insert "... [truncated] ..." in the middle).
pub fn format_search_results_blob(
    results: &[ShapableSearchResult],
    heading: &str,
    max_chars: usize,
) -> String {
    let mut blob = String::new();
    if !heading.is_empty() {
        blob.push_str(heading);
        blob.push_str("\n\n");
    }
    for (i, r) in results.iter().enumerate() {
        let date_line = r
            .date
            .as_deref()
            .map(|d| format!("- **Date:** {}\n", d.trim()))
            .unwrap_or_default();
        let domain = normalized_domain(&r.url);
        let domain_line = if domain.is_empty() {
            String::new()
        } else {
            format!("- **Source:** {}\n", domain)
        };
        let snippet = normalize_snippet_layout(&r.snippet);
        if i > 0 {
            blob.push_str("---\n\n");
        }
        blob.push_str(&format!(
            "### {}. {}\n{}{}- **URL:** {}\n\n{}\n\n",
            i + 1,
            r.title.trim(),
            domain_line,
            date_line,
            r.url,
            snippet
        ));
    }
    if blob.chars().count() <= max_chars {
        return blob;
    }
    truncate_head_tail(&blob, max_chars)
}

/// Truncate string to max_chars, keeping a head and tail with "... [truncated] ..." in the middle.
fn truncate_head_tail(text: &str, max_chars: usize) -> String {
    const MIDDLE: &str = "\n\n... [truncated] ...\n\n";
    let mid_len = MIDDLE.chars().count();
    if max_chars <= mid_len {
        return text.chars().take(max_chars).collect::<String>();
    }
    let half = (max_chars - mid_len) / 2;
    let head: String = text.chars().take(half).collect();
    let tail_start = text.chars().count().saturating_sub(half);
    let tail: String = text.chars().skip(tail_start).collect();
    format!("{}{}{}", head, MIDDLE, tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_truncates_snippet_and_caps() {
        let results = vec![
            ShapableSearchResult {
                title: "A".to_string(),
                url: "https://example.com/1".to_string(),
                snippet: "x".repeat(400),
                date: None,
            },
            ShapableSearchResult {
                title: "B".to_string(),
                url: "https://example.com/2".to_string(),
                snippet: "short".to_string(),
                date: None,
            },
        ];
        let shaped = shape_search_results(results, 50, 10, 2);
        assert_eq!(shaped.len(), 2);
        assert!(shaped[0].snippet.chars().count() <= 51);
        assert_eq!(shaped[1].snippet, "short");
    }

    #[test]
    fn shape_limits_per_domain() {
        let results = vec![
            ShapableSearchResult {
                title: "1".to_string(),
                url: "https://a.com/1".to_string(),
                snippet: "s".to_string(),
                date: None,
            },
            ShapableSearchResult {
                title: "2".to_string(),
                url: "https://a.com/2".to_string(),
                snippet: "s".to_string(),
                date: None,
            },
            ShapableSearchResult {
                title: "3".to_string(),
                url: "https://a.com/3".to_string(),
                snippet: "s".to_string(),
                date: None,
            },
        ];
        let shaped = shape_search_results(results, 500, 10, 2);
        assert_eq!(shaped.len(), 2);
    }

    #[test]
    fn truncate_head_tail_under_limit_unchanged() {
        let s = "short";
        let out = format_search_results_blob(
            &[ShapableSearchResult {
                title: "T".to_string(),
                url: "https://x.com".to_string(),
                snippet: s.to_string(),
                date: None,
            }],
            "## Results",
            2000,
        );
        assert!(out.contains("> short") || out.contains("short"));
        assert!(!out.contains("[truncated]"));
        assert!(out.contains("**Source:** x.com"));
    }

    #[test]
    fn truncate_head_tail_over_limit_has_middle() {
        let results = (0..5)
            .map(|i| ShapableSearchResult {
                title: format!("Title {}", i),
                url: format!("https://x.com/{}", i),
                snippet: "snippet ".repeat(200),
                date: None,
            })
            .collect::<Vec<_>>();
        let out = format_search_results_blob(&results, "## Results", 500);
        assert!(out.contains("... [truncated] ..."));
        assert!(out.chars().count() <= 520);
    }
}
