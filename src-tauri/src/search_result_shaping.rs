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
/// Also turns raw Markdown pipe-tables (AEMET etc.) into bullet lists.
pub fn normalize_snippet_layout(snippet: &str) -> String {
    let mut s = snippet.replace("\\n", "\n").replace("\\r", "");
    s = s.replace('\r', "\n");
    if let Some(table) = humanize_pipe_table_snippet(&s) {
        return table
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .map(|l| {
                if l.starts_with('•') || l.starts_with('>') {
                    l.to_string()
                } else {
                    format!("> {l}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
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

/// Detect Markdown/CSV-style `|cell|cell|` blobs and turn them into bullets.
/// Returns `None` when the text does not look like a pipe table.
pub fn humanize_pipe_table_snippet(snippet: &str) -> Option<String> {
    let pipe_count = snippet.chars().filter(|c| *c == '|').count();
    if pipe_count < 4 {
        return None;
    }
    let raw_rows: Vec<&str> = if snippet.contains('\n') {
        snippet.lines().collect()
    } else {
        vec![snippet]
    };

    let mut cells: Vec<String> = Vec::new();
    for row in raw_rows {
        let t = row.trim();
        if t.is_empty() {
            continue;
        }
        // Skip markdown separator rows: |---|:---|
        let only_sep = t
            .chars()
            .all(|c| matches!(c, '|' | '-' | ':' | ' ' | '\t'));
        if only_sep {
            continue;
        }
        for cell in t.split('|') {
            let c = cell.trim();
            if c.is_empty() {
                continue;
            }
            if c.chars().all(|ch| matches!(ch, '-' | ':')) {
                continue;
            }
            cells.push(prettify_weather_table_cell(c));
        }
    }
    if cells.len() < 2 {
        return None;
    }
    // Cap so AEMET 7-day dumps don't flood the card.
    const MAX_CELLS: usize = 12;
    let shown = if cells.len() > MAX_CELLS {
        let mut v = cells[..MAX_CELLS].to_vec();
        v.push(format!("… +{} more", cells.len() - MAX_CELLS));
        v
    } else {
        cells
    };
    Some(
        shown
            .into_iter()
            .map(|c| format!("• {c}"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn prettify_weather_table_cell(cell: &str) -> String {
    // "06–12 h 22°C" → "06–12 h · 22°C"
    if let Some(idx) = cell.find('°') {
        let before = &cell[..idx];
        if let Some(sp) = before.rfind(|c: char| c.is_whitespace()) {
            let left = before[..sp].trim();
            let temp = cell[sp..].trim();
            if !left.is_empty()
                && temp.chars().any(|c| c.is_ascii_digit())
                && (left.contains('h') || left.contains('–') || left.contains('-'))
            {
                return format!("{left} · {temp}");
            }
        }
    }
    cell.to_string()
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
    fn humanize_aemet_pipe_table_to_bullets() {
        let raw = "|--|--|--|\n|06–12 h 22°C|12–18 h 20°C|18–24 h 18°C|\n|00–06 h 17°C|06–12 h 24°C|";
        let got = humanize_pipe_table_snippet(raw).expect("table");
        assert!(got.contains("• 06–12 h · 22°C"), "{got}");
        assert!(got.contains("• 12–18 h · 20°C"), "{got}");
        assert!(!got.contains("|--"), "{got}");
        let normalized = normalize_snippet_layout(raw);
        assert!(normalized.contains('•') || normalized.contains('>'), "{normalized}");
    }

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
