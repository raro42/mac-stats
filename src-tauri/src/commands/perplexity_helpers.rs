//! Perplexity and news search helper functions.
//!
//! Extracted from `ollama.rs` for cohesion: news-query detection,
//! search-result scoring/shaping, Perplexity fallback logic,
//! tool-suffix/summary formatting, and the PERPLEXITY_SEARCH tool handler.

use std::path::PathBuf;
use tracing::info;

/// Result of the PERPLEXITY_SEARCH tool handler.
pub(crate) struct PerplexitySearchHandlerResult {
    pub text: String,
    pub new_attachment_paths: Vec<PathBuf>,
    /// `Some(true)` when the search returned only hub/landing pages (no article-like results).
    pub news_search_was_hub_only: Option<bool>,
}

/// Execute the PERPLEXITY_SEARCH tool: search, format results, optionally auto-screenshot.
pub(crate) async fn handle_perplexity_search(
    question: &str,
    search_arg: &str,
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
    request_id: &str,
) -> PerplexitySearchHandlerResult {
    let send_status = |msg: &str| {
        if let Some(tx) = status_tx {
            let _ = tx.send(msg.to_string());
        }
    };
    send_status(&format!(
        "🔎 Searching (Perplexity) for \"{}\"…",
        crate::logging::ellipse(search_arg, 35)
    ));
    info!("Discord/Ollama: PERPLEXITY_SEARCH requested: {}", search_arg);

    let q_lower = question.to_lowercase();
    let is_news = is_news_query(question);
    let snippet_max = crate::config::Config::perplexity_snippet_max_chars();
    let max_results = crate::config::Config::perplexity_max_results();

    match search_perplexity_with_news_fallback(question, search_arg, max_results, snippet_max).await
    {
        Ok((shaped_results, urls, filtered_any, refined_query_used)) => {
            let want_screenshots = (q_lower.contains("screenshot")
                || q_lower.contains("screen shot"))
                && (q_lower.contains("visit")
                    || q_lower.contains("url")
                    || q_lower.contains(" 5 ")
                    || q_lower.contains(" 3 ")
                    || q_lower.contains("send me")
                    || q_lower.contains("send the")
                    || q_lower.contains("in discord")
                    || q_lower.contains(" here "));
            let urls: Vec<String> = urls
                .into_iter()
                .filter(|url| url.starts_with("http://") || url.starts_with("https://"))
                .take(5)
                .collect();

            const MAX_PERPLEXITY_SUMMARY_CHARS: usize = 380;
            if status_tx.is_some() {
                let n = shaped_results.len();
                let titles: String = shaped_results
                    .iter()
                    .take(5)
                    .map(|r| r.title.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");
                let summary =
                    build_perplexity_verbose_summary(n, titles, MAX_PERPLEXITY_SUMMARY_CHARS);
                send_status(&summary);
            }

            let num_results = shaped_results.len();
            let search_had_article_like = is_news
                && shaped_results
                    .iter()
                    .any(|r| is_likely_article_like_result(&r.title, &r.url, &r.snippet));

            let news_search_was_hub_only = if is_news {
                if !search_had_article_like {
                    info!("Agent router: news search returned only hub/landing pages; completion verification will require article-grade evidence");
                }
                Some(!search_had_article_like)
            } else {
                None
            };

            let results: String = format_search_results_markdown(&shaped_results, is_news);
            let mut result_text = if results.is_empty() {
                "Perplexity search returned no results. Answer from general knowledge.".to_string()
            } else {
                format!(
                    "## Perplexity Search Results ({} items)\n\n{}\n\nUse these to answer the user's question. Cite source number, title or URL, and date when given.",
                    num_results, results
                )
            };
            if is_news && !results.is_empty() {
                result_text.push_str(&build_perplexity_news_tool_suffix(
                    search_had_article_like,
                    refined_query_used.as_deref(),
                    filtered_any,
                ));
            }

            let mut new_attachment_paths: Vec<PathBuf> = Vec::new();
            if want_screenshots && !urls.is_empty() {
                auto_screenshot_urls(&urls, status_tx, &mut new_attachment_paths).await;
                result_text.push_str(&format!(
                    "\n\nI navigated to and took screenshots of {} page(s). The app will attach them in Discord.",
                    urls.len()
                ));
            }

            info!(
                "Agent router [{}]: PERPLEXITY_SEARCH returned {} results, blob {} bytes",
                request_id,
                num_results,
                result_text.len()
            );
            PerplexitySearchHandlerResult {
                text: result_text,
                new_attachment_paths,
                news_search_was_hub_only,
            }
        }
        Err(e) => {
            if status_tx.is_some() {
                send_status(&format!(
                    "Perplexity search failed: {}",
                    crate::logging::ellipse(&e.to_string(), 120)
                ));
            }
            PerplexitySearchHandlerResult {
                text: format!(
                    "Perplexity search failed: {}. Answer without search results.",
                    e
                ),
                new_attachment_paths: Vec::new(),
                news_search_was_hub_only: None,
            }
        }
    }
}

/// Format shaped search results into structured markdown for the model.
fn format_search_results_markdown(
    results: &[crate::commands::perplexity::PerplexitySearchResult],
    is_news: bool,
) -> String {
    results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let date_str = r
                .date
                .as_deref()
                .or(r.last_updated.as_deref())
                .unwrap_or("")
                .trim();
            let date_line = if date_str.is_empty() {
                String::new()
            } else {
                format!("- **Date:** {}\n", date_str)
            };
            let page_type = if is_news && is_likely_article_like_result(&r.title, &r.url, &r.snippet) {
                "- **Page type:** article-like\n"
            } else if is_news {
                "- **Page type:** hub/landing page\n"
            } else {
                ""
            };
            format!(
                "### {}. {}\n- **URL:** {}\n{}{}- **Snippet:** {}",
                i + 1,
                r.title,
                r.url,
                date_line,
                page_type,
                r.snippet
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Navigate to each URL and take a screenshot, collecting attachment paths.
async fn auto_screenshot_urls(
    urls: &[String],
    status_tx: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
    attachment_paths: &mut Vec<PathBuf>,
) {
    let send_status = |msg: &str| {
        if let Some(tx) = status_tx {
            let _ = tx.send(msg.to_string());
        }
    };
    info!(
        "Agent router: auto-visit and screenshot for {} URLs (user asked for screenshots)",
        urls.len()
    );
    for (i, url) in urls.iter().enumerate() {
        send_status(&format!("🧭 Visiting {} of {}…", i + 1, urls.len()));
        let nav_result = tokio::task::spawn_blocking({
            let u = url.clone();
            move || crate::browser_agent::navigate_and_get_state(&u)
        })
        .await;
        match nav_result {
            Ok(Ok(_)) => {
                send_status(&format!("📸 Taking screenshot {} of {}…", i + 1, urls.len()));
                let shot_result =
                    tokio::task::spawn_blocking(crate::browser_agent::take_screenshot_current_page)
                        .await;
                if let Ok(Ok(path)) = shot_result {
                    attachment_paths.push(path.clone());
                    if let Some(tx) = status_tx {
                        let _ = tx.send(format!("ATTACH:{}", path.display()));
                    }
                    info!(
                        "Agent router: auto-screenshot {} saved to {:?}",
                        i + 1,
                        path
                    );
                }
            }
            Ok(Err(e)) => {
                info!(
                    "Agent router: auto-navigate {} failed: {}",
                    url,
                    crate::logging::ellipse(&e, 80)
                );
            }
            Err(e) => {
                info!("Agent router: auto-navigate task error: {}", e);
            }
        }
    }
}

pub(crate) fn is_news_query(question: &str) -> bool {
    let q = question.to_lowercase();
    q.contains("news")
        || q.contains("latest")
        || q.contains("recent")
        || q.contains("headlines")
        || q.contains("current events")
        || q.contains("today")
        || q.contains("this week")
}

pub(crate) fn normalized_search_result_domain(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| {
            u.host_str()
                .map(|s| s.trim_start_matches("www.").to_string())
        })
        .unwrap_or_default()
}

pub(crate) fn is_likely_article_like_result(title: &str, url: &str, snippet: &str) -> bool {
    score_search_result_for_news(title, url, snippet) > 0
}

pub(crate) fn score_search_result_for_news(title: &str, url: &str, snippet: &str) -> i32 {
    let title_l = title.to_lowercase();
    let url_l = url.to_lowercase();
    let snippet_l = snippet.to_lowercase();
    let domain = normalized_search_result_domain(url);
    let path = url::Url::parse(url)
        .ok()
        .map(|u| u.path().trim_matches('/').to_string())
        .unwrap_or_default();
    let path_depth = path.split('/').filter(|s| !s.is_empty()).count();

    let mut score = 0i32;

    if path_depth >= 2 {
        score += 2;
    }
    if path.contains('-') && path.len() > 18 {
        score += 2;
    }
    if snippet.lines().count() <= 3 {
        score += 1;
    }
    if [
        "reuters.com",
        "apnews.com",
        "bbc.com",
        "euronews.com",
        "catalannews.com",
    ]
    .iter()
    .any(|d| domain.ends_with(d))
    {
        score += 2;
    }

    if path.is_empty() || path == "news" || path.ends_with("/news") || path.ends_with("/news/") {
        score -= 3;
    }
    if url_l.contains("/tag/") || url_l.contains("/category/") || url_l.contains("/topics/") {
        score -= 3;
    }
    if url_l.contains("wikipedia.org/wiki/")
        || domain.ends_with("wikipedia.org")
        || domain.ends_with("spain.info")
        || url_l.contains("/destination/")
        || url_l.contains("/destinazione/")
    {
        score -= 5;
    }
    if title_l.contains("top stories")
        || title_l.contains("latest ")
        || title_l.contains("breaking ")
        || title_l.contains("scores")
        || title_l.contains("standings")
        || title_l.contains("rumors")
        || title_l.contains("official channel")
        || title_l.contains("what to see and do")
    {
        score -= 3;
    }
    if snippet_l.contains("view on x")
        || snippet_l.contains("rumor")
        || snippet_l.contains("standings")
        || snippet_l.contains("scores")
        || snippet_l.contains("trendiest")
        || snippet_l.contains("tourist")
    {
        score -= 2;
    }
    if snippet.lines().count() >= 5 {
        score -= 1;
    }
    if domain.contains("newsnow") || domain.contains("transferfeed") {
        score -= 2;
    }

    score
}

pub(crate) fn shape_perplexity_results_for_question(
    question: &str,
    results: Vec<crate::commands::perplexity::PerplexitySearchResult>,
    snippet_max: usize,
) -> (
    Vec<crate::commands::perplexity::PerplexitySearchResult>,
    Vec<String>,
    bool,
) {
    let is_news = is_news_query(question);
    if !is_news {
        let urls = results.iter().map(|r| r.url.clone()).collect();
        return (results, urls, false);
    }

    let mut ranked: Vec<_> = results
        .into_iter()
        .map(|r| {
            let score = score_search_result_for_news(&r.title, &r.url, &r.snippet);
            (score, r)
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.0.cmp(&a.0).then_with(|| {
            let ad =
                a.1.date
                    .as_deref()
                    .or(a.1.last_updated.as_deref())
                    .unwrap_or("");
            let bd =
                b.1.date
                    .as_deref()
                    .or(b.1.last_updated.as_deref())
                    .unwrap_or("");
            bd.cmp(ad)
        })
    });

    let mut kept = Vec::new();
    let mut urls = Vec::new();
    let mut per_domain: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut filtered_any = false;
    for (score, mut result) in ranked {
        let domain = normalized_search_result_domain(&result.url);
        let domain_count = per_domain.get(&domain).copied().unwrap_or(0);
        let article_like = score > 0;
        let allow = article_like || kept.len() < 3;
        if !allow || domain_count >= 2 {
            filtered_any = true;
            continue;
        }
        if result.snippet.chars().count() > snippet_max {
            result.snippet = format!(
                "{}…",
                result.snippet.chars().take(snippet_max).collect::<String>()
            );
        }
        per_domain.insert(domain, domain_count + 1);
        urls.push(result.url.clone());
        kept.push(result);
        if kept.len() >= 6 {
            break;
        }
    }

    (kept, urls, filtered_any)
}

pub(crate) async fn search_perplexity_with_news_fallback(
    question: &str,
    query: &str,
    max_results: u32,
    snippet_max: usize,
) -> Result<
    (
        Vec<crate::commands::perplexity::PerplexitySearchResult>,
        Vec<String>,
        bool,
        Option<String>,
    ),
    String,
> {
    let mut effective_query = query.trim().to_string();
    if is_news_query(question)
        && (effective_query.chars().count() < 18 || effective_query.split_whitespace().count() < 3)
    {
        effective_query = format!("{} latest news sources dates", question.trim());
    }

    let first = crate::commands::perplexity::perplexity_search(
        crate::commands::perplexity::PerplexitySearchRequest {
            query: effective_query.clone(),
            max_results: Some(max_results),
        },
    )
    .await?;

    let is_news = is_news_query(question);
    let mut used_query = None;
    let (mut shaped, mut urls, mut filtered_any) =
        shape_perplexity_results_for_question(question, first.results, snippet_max);

    let need_fallback = is_news
        && !shaped.is_empty()
        && shaped
            .iter()
            .all(|r| !is_likely_article_like_result(&r.title, &r.url, &r.snippet));

    if need_fallback {
        let fallback_query = format!("{} recent article source date", effective_query.trim());
        tracing::info!(
            "Perplexity search: first pass for news returned only hub/landing pages, retrying with refined query: {}",
            fallback_query
        );
        let second = crate::commands::perplexity::perplexity_search(
            crate::commands::perplexity::PerplexitySearchRequest {
                query: fallback_query.clone(),
                max_results: Some(max_results),
            },
        )
        .await?;
        let (fallback_shaped, fallback_urls, fallback_filtered) =
            shape_perplexity_results_for_question(question, second.results, snippet_max);
        let fallback_has_article_like = fallback_shaped
            .iter()
            .any(|r| is_likely_article_like_result(&r.title, &r.url, &r.snippet));
        if fallback_has_article_like {
            shaped = fallback_shaped;
            urls = fallback_urls;
            filtered_any = fallback_filtered;
            used_query = Some(fallback_query);
        }
    }

    Ok((shaped, urls, filtered_any, used_query))
}

/// Builds the suffix appended to PERPLEXITY_SEARCH tool result when is_news_query and results are non-empty.
/// When search_had_article_like is false, includes the "Article-grade results were not found" warning.
pub(crate) fn build_perplexity_news_tool_suffix(
    search_had_article_like: bool,
    refined_query_used: Option<&str>,
    filtered_any: bool,
) -> String {
    const HUB_ONLY_LINE: &str = "\n\n**Article-grade results were not found;** only hub/landing/tag/standings pages are listed below. Do not present these as a complete news answer; state that concrete article links were not found or run another search.";
    const NEWS_GUIDANCE: &str = "\n\nFor news requests, prefer concrete article/report results over homepages, hub pages, standings pages, rumor indexes, or official landing pages. If a result looks like a hub, use it only as fallback and say so clearly.";
    let mut s = String::new();
    if !search_had_article_like {
        s.push_str(HUB_ONLY_LINE);
    }
    s.push_str(NEWS_GUIDANCE);
    if let Some(refined) = refined_query_used {
        s.push_str(&format!(
            "\nRefined search query used to find article-like results: {}.",
            refined
        ));
    }
    if filtered_any {
        s.push_str(
            "\nFiltered to higher-signal results and limited repeated domains where possible.",
        );
    }
    s
}

/// Build the short Perplexity result summary for verbose Discord (respects max_chars).
pub(crate) fn build_perplexity_verbose_summary(
    n: usize,
    titles: String,
    max_chars: usize,
) -> String {
    if n == 0 {
        "Perplexity: 0 results.".to_string()
    } else if titles.trim().is_empty() {
        format!("Perplexity: {} result(s) received.", n)
    } else {
        let raw = format!("Perplexity: {} result(s) — {}", n, titles.trim());
        if raw.chars().count() > max_chars {
            format!(
                "{}…",
                raw.chars()
                    .take(max_chars - 1)
                    .collect::<String>()
                    .trim_end()
            )
        } else {
            raw
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perplexity_verbose_summary_zero_results() {
        let s = build_perplexity_verbose_summary(0, String::new(), 380);
        assert_eq!(s, "Perplexity: 0 results.");
    }

    #[test]
    fn perplexity_verbose_summary_with_titles() {
        let s = build_perplexity_verbose_summary(3, "El País, El Mundo, ABC".to_string(), 380);
        assert_eq!(s, "Perplexity: 3 result(s) — El País, El Mundo, ABC");
    }

    #[test]
    fn perplexity_verbose_summary_titles_empty_but_n_nonzero() {
        let s = build_perplexity_verbose_summary(5, String::new(), 380);
        assert_eq!(s, "Perplexity: 5 result(s) received.");
    }

    #[test]
    fn perplexity_verbose_summary_truncated() {
        let long = "A".repeat(400);
        let s = build_perplexity_verbose_summary(1, long.clone(), 380);
        assert!(s.chars().count() <= 380);
        assert!(s.ends_with('…'));
        assert!(s.starts_with("Perplexity: 1 result(s) — "));
    }

    #[test]
    fn perplexity_verbose_summary_just_under_limit() {
        let titles = "X, Y, Z";
        let s = build_perplexity_verbose_summary(3, titles.to_string(), 380);
        assert_eq!(s, "Perplexity: 3 result(s) — X, Y, Z");
    }

    #[test]
    fn score_search_result_for_news_prefers_article_like_pages() {
        let article_score = score_search_result_for_news(
            "Barcelona opens new civic center",
            "https://example.com/news/barcelona-opens-new-civic-center",
            "Barcelona opened a new civic center on March 6.",
        );
        let hub_score = score_search_result_for_news(
            "FC Barcelona News | Top Stories",
            "https://www.newsnow.com/us/Sports/Soccer/La+Liga/Barcelona",
            "Top stories and transfer rumors View on X",
        );
        assert!(article_score > hub_score);
    }

    #[test]
    fn shape_perplexity_results_for_question_limits_repeated_domains_for_news() {
        let results = vec![
            crate::commands::perplexity::PerplexitySearchResult {
                title: "Hub page".to_string(),
                url: "https://www.newsnow.com/us/Sports/Soccer/La+Liga/Barcelona".to_string(),
                snippet: "Top stories and rumors".to_string(),
                date: Some("2026-03-06".to_string()),
                last_updated: None,
            },
            crate::commands::perplexity::PerplexitySearchResult {
                title: "Article one".to_string(),
                url: "https://example.com/news/barcelona-culture-update".to_string(),
                snippet: "Culture update from Barcelona.".to_string(),
                date: Some("2026-03-06".to_string()),
                last_updated: None,
            },
            crate::commands::perplexity::PerplexitySearchResult {
                title: "Article two".to_string(),
                url: "https://example.com/news/barcelona-transit-update".to_string(),
                snippet: "Transit update from Barcelona.".to_string(),
                date: Some("2026-03-05".to_string()),
                last_updated: None,
            },
            crate::commands::perplexity::PerplexitySearchResult {
                title: "Article three".to_string(),
                url: "https://example.com/news/barcelona-housing-update".to_string(),
                snippet: "Housing update from Barcelona.".to_string(),
                date: Some("2026-03-04".to_string()),
                last_updated: None,
            },
        ];
        let (shaped, _, filtered_any) = shape_perplexity_results_for_question(
            "Show me recent Barcelona news with sources and dates.",
            results,
            280,
        );
        let example_count = shaped
            .iter()
            .filter(|r| r.url.contains("example.com"))
            .count();
        assert!(filtered_any);
        assert_eq!(example_count, 2);
        assert_eq!(
            shaped.first().map(|r| r.title.as_str()),
            Some("Article one")
        );
    }

    #[test]
    fn shape_perplexity_results_for_question_preserves_hub_only_fallback() {
        let results = vec![
            crate::commands::perplexity::PerplexitySearchResult {
                title: "Catalan News | News in English from Barcelona & Catalonia".to_string(),
                url: "https://www.catalannews.com".to_string(),
                snippet: "### International Women's Day in Barcelona\nMore headlines".to_string(),
                date: Some("2026-03-06".to_string()),
                last_updated: None,
            },
            crate::commands::perplexity::PerplexitySearchResult {
                title: "FC Barcelona News | Barça News & Top Stories".to_string(),
                url: "https://www.newsnow.com/us/Sports/Soccer/La+Liga/Barcelona".to_string(),
                snippet: "Top stories and transfer rumors View on X".to_string(),
                date: Some("2026-03-06".to_string()),
                last_updated: None,
            },
        ];
        let (shaped, _, _) = shape_perplexity_results_for_question(
            "Can you look on the Internet for news involving Barcelona? Mention sources and dates.",
            results,
            280,
        );
        assert_eq!(shaped.len(), 2);
        assert!(shaped
            .iter()
            .all(|r| !is_likely_article_like_result(&r.title, &r.url, &r.snippet)));
    }

    #[test]
    fn build_perplexity_news_tool_suffix_includes_hub_only_warning_when_no_article_like() {
        let suffix =
            build_perplexity_news_tool_suffix(false, None, false);
        assert!(
            suffix.contains("Article-grade results were not found"),
            "hub-only suffix must contain Article-grade warning, got: {:?}",
            suffix
        );
        assert!(suffix.contains("hub/landing/tag/standings"));
    }

    #[test]
    fn build_perplexity_news_tool_suffix_no_hub_only_warning_when_article_like() {
        let suffix =
            build_perplexity_news_tool_suffix(true, None, false);
        assert!(
            !suffix.contains("Article-grade results were not found"),
            "when search had article-like results, suffix must not contain hub-only warning"
        );
    }
}
