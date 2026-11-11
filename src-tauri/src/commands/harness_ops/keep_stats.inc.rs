// included by gateway.rs — keep_stats
/// True when `content` is a short keep/discard gap percentile ask.
pub fn looks_like_keep_percentile_request(content: &str) -> bool {
    parse_keep_percentile(content).is_some()
}

/// Digits for `/keep-p47`, `p4 keep gap`, `47th percentile keep gap`, …
/// Digit-safe: `/keep-p4` does not match `/keep-p47`.
pub fn parse_keep_percentile(content: &str) -> Option<u8> {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return None;
    }
    if keep_percentile_excluded(&n) {
        return None;
    }
    let p = extract_keep_percentile_number(&n)?;
    if (1..=99).contains(&p) {
        Some(p)
    } else {
        None
    }
}

/// Used by sibling keep-stat matchers as a string-only style exclude.
pub fn mentions_keep_percentile(n: &str) -> bool {
    extract_keep_percentile_number(n).is_some()
}

/// Shared rejects for results.tsv gap-stat lanes (pace/median/range/…).
pub(super) fn keep_gap_stat_excluded(n: &str) -> bool {
    keep_percentile_excluded(n) || mentions_keep_percentile(n)
}


pub(super) fn keep_percentile_excluded(n: &str) -> bool {
    n.contains("path")
        || n.contains("where")
        || n.contains("location")
        || n.contains("folder")
        || n.contains("directory")
        || (n.contains("dir") && !n.contains("discard"))
        || n.contains("size")
        || n.contains("big")
        || n.contains("large")
        || n.contains("bytes")
        || n.contains(" mb")
        || n.contains("kb")
        || n.contains(" gi")
        || (contains_age_token(n)
            && !n.contains("average")
            && !n.contains("avg")
            && !n.contains("percentile"))
        || n.contains("how old")
        || n.contains("stale")
        || n.contains("dump")
        || n.contains("tail")
        || n.contains("read ")
        || n.contains("print ")
        || n.contains("cat ")
        || n.contains("contents")
        || n.contains("what is in")
        || n.contains("what's in")
        || n.contains("whats in")
        || n.contains("what shipped")
        || n.contains("morning surprise")
        || n.contains("any improvements")
        || n.contains("improvements from")
        || n.contains("changelog")
        || n.contains("why")
        || n.contains("fix")
        || n.contains("explain")
        || n.contains("create")
        || n.contains("delete")
        || n.contains("remove")
        || n.contains("prune")
        || n.contains("http://")
        || n.contains("https://")
        || n.contains("runs.jsonl")
        || n.contains("debug.log")
        || n.contains("loop backlog")
        || n.contains("loop_backlog")
        || n.contains("sibling")
        || n.contains("standing")
        || n.contains("how many")
        || n.contains("count")
        || n.contains("summary")
        || n.contains("hit rate")
        || n.contains("keep rate")
        || n.contains("discard rate")
        || n.contains("streak")
        || n.contains("recent keeps")
        || n.contains("recent discards")
        || n.contains("recent-keeps")
        || n.contains("recentkeeps")
        || n.contains("keep list")
        || n.contains("discard list")
        || n.contains("first keep")
        || n.contains("first discard")
        || n.contains("since last")
        || n.contains("how long since")
        || n.contains("time since")
        || n.starts_with("/since-")
        || n.starts_with("/last-")
        || n.starts_with("/first-")
        || n.starts_with("/recent-")
        || n.starts_with("/keep-rate")
        || n.starts_with("/keeprate")
        || n.starts_with("/hit-rate")
        || n.starts_with("/keep-streak")
        || n.starts_with("/longest-")
        || n.starts_with("/keep-pace")
        || n.starts_with("/keeppace")
        || n.starts_with("/discard-pace")
        || n.starts_with("/keep-median")
        || n.starts_with("/keepmedian")
        || n.starts_with("/discard-median")
        || n.starts_with("/median-")
        || n.starts_with("/keep-range")
        || n.starts_with("/keeprange")
        || n.starts_with("/discard-range")
        || n.starts_with("/keep-std")
        || n.starts_with("/keep-mad")
        || n.starts_with("/keep-cv")
        || n.starts_with("/keep-skew")
        || n.starts_with("/keep-kurtosis")
        || n.starts_with("/keep-entropy")
        || n.starts_with("/keep-mode")
        || n.starts_with("/keep-gini")
        || n.starts_with("/keep-histogram")
        || n.starts_with("/keep-iqr")
        || n.starts_with("/iqr")
        || n.contains("interquartile")
        || n.contains("histogram")
        || n.contains("entropy")
        || n.contains("kurtosis")
        || n.contains("gini")
        || (n.contains("median") && !n.contains("percentile"))
        || (n.contains("pace") && !n.contains("percentile"))
        || (n.contains("range") && !n.contains("percentile") && !n.contains("arranged"))
        || n.contains("min-max")
        || n.contains("minmax")
}

pub(super) fn slash_p(n: &str, base: &str) -> Option<u8> {
    let rest = n.strip_prefix(base)?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let after = &rest[digits.len()..];
    if after.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

pub(super) fn extract_keep_percentile_number(n: &str) -> Option<u8> {
    let mut best: Option<(u8, usize)> = None;
    let mut consider = |p: u8| {
        let len = p.to_string().len();
        if best.map(|(_, l)| len >= l).unwrap_or(true) {
            best = Some((p, len));
        }
    };

    for base in ["/keep-p", "/keepp", "/discard-p", "/discardp"] {
        if let Some(p) = slash_p(n, base) {
            consider(p);
        }
    }
    // `/p47`, `/p47-keep`, `/p47gap`, …
    if let Some(p) = slash_p(n, "/p") {
        if let Some(rest) = n.strip_prefix("/p") {
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            let after = &rest[digits.len()..];
            if after.is_empty()
                || after.starts_with('-')
                || after.starts_with("keep")
                || after.starts_with("discard")
                || after.starts_with("gap")
            {
                consider(p);
            }
        }
    }
    if let Some((p, _)) = best {
        return Some(p);
    }

    if let Some(p) = scan_ordinal(n) {
        return Some(p);
    }

    let padded = format!(" {n} ");
    if let Some(p) = scan_p_token(&padded) {
        if n.contains("keep")
            || n.contains("discard")
            || n.contains("gap")
            || n.contains("ratchet")
            || n.contains("percentile")
            || n.contains("overnight")
            || n.contains("view ")
            || n.contains("show ")
            || n.contains("see ")
            || n.contains("what is")
            || n.contains("whats ")
            || n.contains("what's ")
        {
            return Some(p);
        }
    }
    None
}

pub(super) fn scan_ordinal(n: &str) -> Option<u8> {
    let bytes = n.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let dig = std::str::from_utf8(&bytes[start..i]).ok()?;
            let p: u8 = dig.parse().ok()?;
            if !(1..=99).contains(&p) {
                continue;
            }
            let rest = n[i..].trim_start();
            let suf_ok = rest.starts_with("th")
                || rest.starts_with("st")
                || rest.starts_with("nd")
                || rest.starts_with("rd");
            if suf_ok
                && (n.contains("percentile")
                    || n.contains("keep")
                    || n.contains("discard")
                    || n.contains("gap"))
            {
                return Some(p);
            }
        } else {
            i += 1;
        }
    }
    None
}

pub(super) fn scan_p_token(padded: &str) -> Option<u8> {
    let chars: Vec<char> = padded.chars().collect();
    let mut best: Option<(u8, usize)> = None;
    let mut i = 0;
    while i + 2 < chars.len() {
        if chars[i] == ' ' && chars[i + 1] == 'p' {
            let mut j = i + 2;
            if chars.get(j) == Some(&'-') {
                j += 1;
            }
            let start = j;
            while j < chars.len() && chars[j].is_ascii_digit() {
                j += 1;
            }
            if j > start {
                let dig: String = chars[start..j].iter().collect();
                if let Ok(p) = dig.parse::<u8>() {
                    if (1..=99).contains(&p) {
                        let next_ok = matches!(chars.get(j), None | Some(' ') | Some('-'));
                        if next_ok {
                            let len = dig.len();
                            if best.map(|(_, l)| len >= l).unwrap_or(true) {
                                best = Some((p, len));
                            }
                        }
                    }
                }
            }
        }
        if chars[i] == '-' && chars.get(i + 1) == Some(&'p') {
            let mut j = i + 2;
            let start = j;
            while j < chars.len() && chars[j].is_ascii_digit() {
                j += 1;
            }
            if j > start {
                let dig: String = chars[start..j].iter().collect();
                if let Ok(p) = dig.parse::<u8>() {
                    if (1..=99).contains(&p) {
                        let len = dig.len();
                        if best.map(|(_, l)| len >= l).unwrap_or(true) {
                            best = Some((p, len));
                        }
                    }
                }
            }
        }
        i += 1;
    }
    best.map(|(p, _)| p)
}


/// Format nearest-rank keep/discard gap percentile from results.tsv.
pub fn format_keep_percentile_gateway(p: u8, content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => format!("Keep p{p}"),
        ResultsTsvLastWant::Discard => format!("Discard p{p}"),
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_percentile(p, want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((p_night, gaps_night, p_all, gaps_all)) => {
            let night_part = match p_night {
                Some(secs) => {
                    let d = duration_label_secs(secs);
                    format!("tonight p{p} **{d}** between {unit} ({gaps_night} gaps since 20:00)")
                }
                None => format!("tonight need ≥2 {unit} since 20:00 to measure p{p}"),
            };
            let all_part = match p_all {
                Some(secs) => {
                    let d = duration_label_secs(secs);
                    format!("all-time p{p} **{d}** ({gaps_all} gaps)")
                }
                None => format!("all-time need ≥2 {unit}"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · p{p} only · does not dump rows · `/keep-pace` for average gap · `/keep-median` for median gap · `/keep-range` for min–max gap · `/keep-p{{N}}` for other percentiles · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}

pub(super) fn count_results_tsv_percentile(
    p: u8,
    want: ResultsTsvLastWant,
) -> Result<(Option<u64>, u64, Option<u64>, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    let frac = (p as f64) / 100.0;
    let percentile_gap_secs = |ts: &[chrono::DateTime<chrono::Utc>]| -> (Option<u64>, u64) {
        if ts.len() < 2 {
            return (None, 0);
        }
        let mut gaps: Vec<u64> = Vec::with_capacity(ts.len() - 1);
        for w in ts.windows(2) {
            gaps.push((w[1] - w[0]).num_seconds().max(0) as u64);
        }
        let n = gaps.len() as u64;
        if n == 0 {
            return (None, 0);
        }
        gaps.sort_unstable();
        let rank = ((n as f64) * frac).ceil() as usize;
        let idx = rank.saturating_sub(1).min(gaps.len() - 1);
        (Some(gaps[idx]), n)
    };
    let (p_all, gaps_all) = percentile_gap_secs(&all_ts);
    let night_ts: Vec<_> = all_ts
        .into_iter()
        .filter(|t| *t >= window_start)
        .collect();
    let (p_night, gaps_night) = percentile_gap_secs(&night_ts);
    Ok((p_night, gaps_night, p_all, gaps_all))
}


/// Local start of the current overnight window (20:00 previous calendar day when before 20:00).
pub(super) fn overnight_window_start_local() -> chrono::DateTime<chrono::Local> {
    use chrono::Timelike;
    let now = chrono::Local::now();
    let today = now.date_naive();
    let start_date = if now.hour() >= 20 {
        today
    } else {
        today - chrono::Duration::days(1)
    };
    start_date
        .and_hms_opt(20, 0, 0)
        .and_then(|naive| naive.and_local_timezone(chrono::Local).single())
        .unwrap_or(now)
}


/// Parse keep/discard row counts from results.tsv (tab: ts, sha, outcome, description).
pub(super) fn count_results_tsv_outcomes() -> Result<(u64, u64, u64, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let mut keep_all = 0u64;
    let mut discard_all = 0u64;
    let mut keep_night = 0u64;
    let mut discard_night = 0u64;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if !is_keep && !is_discard {
            continue;
        }
        if is_keep {
            keep_all += 1;
        } else {
            discard_all += 1;
        }
        if let Some(parsed) = parse_run_ts(ts) {
            if parsed >= window_start {
                if is_keep {
                    keep_night += 1;
                } else {
                    discard_night += 1;
                }
            }
        }
    }
    Ok((keep_night, discard_night, keep_all, discard_all))
}


/// True for short keep/discard count asks (`/keeps`, `keeps tonight`, `ratchet summary`…).
/// Inventory only for count phrasing: “how many keeps”, “keep count”, “count the keeps”.
/// Verb phrases (“delete these keeps”, “clear the discards”, “export keeps”) stay with the agent.
/// Counts only — does not dump TSV rows or steal path/size/age / morning surprise / what-shipped.
pub fn looks_like_results_tsv_count_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keeps"
            | "keeps"
            | "the keeps"
            | "keep count"
            | "keeps count"
            | "how many keeps"
            | "how many keep"
            | "number of keeps"
            | "keeps tonight"
            | "keep tonight"
            | "tonight keeps"
            | "overnight keeps"
            | "overnight keep count"
            | "keeps last night"
            | "keep last night"
            | "last night keeps"
            | "keeps this night"
            | "/discards"
            | "discards"
            | "the discards"
            | "discard count"
            | "discards count"
            | "how many discards"
            | "how many discard"
            | "number of discards"
            | "discards tonight"
            | "discard tonight"
            | "tonight discards"
            | "overnight discards"
            | "discards last night"
            | "discard last night"
            | "ratchet summary"
            | "ratchet count"
            | "ratchet counts"
            | "ratchet keep count"
            | "ratchet discard count"
            | "keep discard count"
            | "keep discard counts"
            | "keep discard summary"
            | "results keep count"
            | "results discard count"
            | "results.tsv keep count"
            | "results.tsv discard count"
            | "results tsv keep count"
            | "view keeps"
            | "see keeps"
            | "show the keeps"
            | "show me the keeps"
            | "open keeps"
            | "open the keeps"
            | "list keeps"
            | "list the keeps"
            | "view discards"
            | "see discards"
            | "show the discards"
            | "show me the discards"
            | "open discards"
            | "open the discards"
            | "list discards"
            | "list the discards"
            | "view ratchet"
            | "see ratchet"
            | "open ratchet"
            | "list ratchet"
            | "view ratchet summary"
            | "see ratchet summary"
            | "open ratchet summary"
            | "list the ratchet summary"
    )
}


/// Zero-LLM keep/discard counts from results.tsv (tonight window + all-time; no row dump).
pub fn format_results_tsv_count_gateway() -> String {
    match count_results_tsv_outcomes() {
        Err(_) => {
            "**Ratchet:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
                .to_string()
        }
        Ok((keep_night, discard_night, keep_all, discard_all)) => {
            let night_total = keep_night + discard_night;
            format!(
                "**Ratchet:** tonight keep **{keep_night}** · discard **{discard_night}** ({night_total} rows since 20:00) · all-time keep **{keep_all}** · discard **{discard_all}** · counts only · does not dump rows · `/keep-rate` for hit rate · `/keep-streak` for consecutive keeps · `/longest-streak` for the record · `/recent-keeps` for a short tonight list · `/last-keep` · `/last-discard` for the newest row · `results.tsv path` for the file · `results.tsv age` for last write · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// Wanted outcome for a single-row last keep/discard glance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResultsTsvLastWant {
    Keep,
    Discard,
}


/// Last matching keep or discard row from results.tsv (append order; one row only).
pub(super) fn last_results_tsv_row(want: ResultsTsvLastWant) -> Result<(String, String, String), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut last: Option<(String, String, String)> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let sha = cols.next().unwrap_or("").trim();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let desc = cols.next().unwrap_or("").trim();
        last = Some((ts.to_string(), sha.to_string(), desc.to_string()));
    }
    last.ok_or_else(|| "empty".into())
}


/// True for short last-keep / last-discard asks (`/last-keep`, `last discard`…).
/// One newest matching row only — does not dump TSV, steal counts / path/size/age / morning surprise.
pub fn looks_like_results_tsv_last_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/last-keep"
            | "/lastkeep"
            | "/last-discard"
            | "/lastdiscard"
            | "last keep"
            | "the last keep"
            | "latest keep"
            | "the latest keep"
            | "most recent keep"
            | "the most recent keep"
            | "last keep description"
            | "latest keep description"
            | "what was the last keep"
            | "what is the last keep"
            | "whats the last keep"
            | "what's the last keep"
            | "view last keep"
            | "see last keep"
            | "show the last keep"
            | "show me the last keep"
            | "open last keep"
            | "open the last keep"
            | "list last keep"
            | "list the last keep"
            | "last discard"
            | "the last discard"
            | "latest discard"
            | "the latest discard"
            | "most recent discard"
            | "the most recent discard"
            | "last discard description"
            | "latest discard description"
            | "what was the last discard"
            | "what is the last discard"
            | "whats the last discard"
            | "what's the last discard"
            | "view last discard"
            | "see last discard"
            | "show the last discard"
            | "show me the last discard"
            | "open last discard"
            | "open the last discard"
            | "list last discard"
            | "list the last discard"
    )
}


pub(super) fn results_tsv_last_want(content: &str) -> ResultsTsvLastWant {
    let n = normalize_operator_command(content);
    if n.contains("discard") {
        ResultsTsvLastWant::Discard
    } else {
        ResultsTsvLastWant::Keep
    }
}


/// Zero-LLM newest keep or discard row from results.tsv (one description; no full dump).
pub fn format_results_tsv_last_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Last keep",
        ResultsTsvLastWant::Discard => "Last discard",
    };
    match last_results_tsv_row(want) {
        Err(_) => format!(
            "**{label}:** no matching row yet · overnight keep/discard will create it · `/keeps` for counts · `results.tsv path` for the file."
        ),
        Ok((ts, sha, desc)) => {
            let age = parse_run_ts(&ts)
                .map(|dt| {
                    let ms = dt.timestamp_millis().max(0) as u64;
                    age_from_ms(ms)
                })
                .unwrap_or_else(|| "unknown".to_string());
            let short_sha = if sha.len() > 7 { &sha[..7] } else { sha.as_str() };
            let mut desc = desc;
            if desc.chars().count() > 160 {
                desc = desc.chars().take(157).collect::<String>() + "…";
            }
            if desc.is_empty() {
                desc = "(no description)".to_string();
            }
            format!(
                "**{label}:** **{age}** ago · `{short_sha}` · {desc} · one row only · does not dump the log · `/keeps` for counts · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// Cap for `/recent-keeps` tonight list (short glance — not a full TSV dump).
pub(super) const RESULTS_TSV_RECENT_CAP: usize = 5;


/// Tonight's matching keep or discard rows from results.tsv (newest first; capped).
pub(super) fn recent_results_tsv_rows(
    want: ResultsTsvLastWant,
) -> Result<(Vec<(String, String, String)>, usize), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut rows: Vec<(String, String, String)> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let sha = cols.next().unwrap_or("").trim();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        if parsed < window_start {
            continue;
        }
        let desc = cols.next().unwrap_or("").trim();
        rows.push((ts.to_string(), sha.to_string(), desc.to_string()));
    }
    let total = rows.len();
    if total == 0 {
        return Err("empty".into());
    }
    // Newest first (append order → reverse).
    rows.reverse();
    if rows.len() > RESULTS_TSV_RECENT_CAP {
        rows.truncate(RESULTS_TSV_RECENT_CAP);
    }
    Ok((rows, total))
}


/// True for short recent-keeps / recent-discards asks (`/recent-keeps`, `recent keeps`…).
/// Tonight window only, capped list — does not dump full TSV, steal counts / last-row / path/size/age / morning surprise.
pub fn looks_like_results_tsv_recent_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/recent-keeps"
            | "/recentkeeps"
            | "/recent-discards"
            | "/recentdiscards"
            | "recent keeps"
            | "the recent keeps"
            | "recent discards"
            | "the recent discards"
            | "most recent keeps"
            | "the most recent keeps"
            | "most recent discards"
            | "the most recent discards"
            | "tonight keep list"
            | "tonight discard list"
            | "keep list tonight"
            | "discard list tonight"
            | "keeps list tonight"
            | "discards list tonight"
            | "recent keep list"
            | "recent discard list"
            | "view recent keeps"
            | "see recent keeps"
            | "show the recent keeps"
            | "show me the recent keeps"
            | "open recent keeps"
            | "open the recent keeps"
            | "list recent keeps"
            | "list the recent keeps"
            | "view recent discards"
            | "see recent discards"
            | "show the recent discards"
            | "show me the recent discards"
            | "open recent discards"
            | "open the recent discards"
            | "list recent discards"
            | "list the recent discards"
    )
}


/// Zero-LLM tonight keep/discard list from results.tsv (newest first; capped; no full dump).
pub fn format_results_tsv_recent_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Recent keeps",
        ResultsTsvLastWant::Discard => "Recent discards",
    };
    match recent_results_tsv_rows(want) {
        Err(_) => format!(
            "**{label}:** none tonight since 20:00 · `/keeps` for counts · `/last-keep` · `/last-discard` for the newest row · `results.tsv path` for the file."
        ),
        Ok((rows, total)) => {
            let mut lines: Vec<String> = Vec::with_capacity(rows.len() + 2);
            let shown = rows.len();
            lines.push(format!(
                "**{label} (tonight):** showing **{shown}** of **{total}** since 20:00"
            ));
            for (i, (ts, sha, desc)) in rows.into_iter().enumerate() {
                let age = parse_run_ts(&ts)
                    .map(|dt| {
                        let ms = dt.timestamp_millis().max(0) as u64;
                        age_from_ms(ms)
                    })
                    .unwrap_or_else(|| "unknown".to_string());
                let short_sha = if sha.len() > 7 { &sha[..7] } else { sha.as_str() };
                let mut desc = desc;
                if desc.chars().count() > 100 {
                    desc = desc.chars().take(97).collect::<String>() + "…";
                }
                if desc.is_empty() {
                    desc = "(no description)".to_string();
                }
                lines.push(format!(
                    "{}. **{age}** ago · `{short_sha}` · {desc}",
                    i + 1
                ));
            }
            lines.push(format!(
                "capped at {RESULTS_TSV_RECENT_CAP} · does not dump the full log · `/keeps` for counts · `/keep-rate` for hit rate · `/keep-streak` for consecutive keeps · `/last-keep` for the newest row · ask *morning surprise?* for ship notes."
            ));
            lines.join("\n")
        }
    }
}


/// True for short keep-rate / hit-rate asks (`/keep-rate`, `keep rate`, `ratchet hit rate`…).
/// Percentages only — does not dump TSV rows or steal counts / last-row / recent list / path/size/age / morning surprise.
pub fn looks_like_results_tsv_rate_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-rate"
            | "/keeprate"
            | "/discard-rate"
            | "/discardrate"
            | "/hit-rate"
            | "/hitrate"
            | "keep rate"
            | "the keep rate"
            | "keep percentage"
            | "the keep percentage"
            | "keep percent"
            | "the keep percent"
            | "discard rate"
            | "the discard rate"
            | "discard percentage"
            | "the discard percentage"
            | "discard percent"
            | "the discard percent"
            | "ratchet rate"
            | "the ratchet rate"
            | "ratchet hit rate"
            | "the ratchet hit rate"
            | "hit rate"
            | "the hit rate"
            | "hit rate tonight"
            | "tonight hit rate"
            | "keep hit rate"
            | "the keep hit rate"
            | "keep success rate"
            | "the keep success rate"
            | "overnight keep rate"
            | "tonight keep rate"
            | "keep rate tonight"
            | "view keep rate"
            | "see keep rate"
            | "show the keep rate"
            | "show me the keep rate"
            | "open keep rate"
            | "open the keep rate"
            | "list keep rate"
            | "list the keep rate"
            | "view discard rate"
            | "see discard rate"
            | "show the discard rate"
            | "show me the discard rate"
            | "open discard rate"
            | "open the discard rate"
            | "list discard rate"
            | "list the discard rate"
            | "view hit rate"
            | "see hit rate"
            | "show the hit rate"
            | "show me the hit rate"
            | "open hit rate"
            | "open the hit rate"
            | "list hit rate"
            | "list the hit rate"
            | "view ratchet rate"
            | "see ratchet rate"
            | "open ratchet rate"
            | "list ratchet rate"
            | "what is the keep rate"
            | "whats the keep rate"
            | "what's the keep rate"
            | "what is the hit rate"
            | "whats the hit rate"
            | "what's the hit rate"
    )
}


/// Zero-LLM keep/discard hit rate from results.tsv (tonight + all-time percentages; no row dump).
pub fn format_results_tsv_rate_gateway() -> String {
    match count_results_tsv_outcomes() {
        Err(_) => {
            "**Keep rate:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
                .to_string()
        }
        Ok((keep_night, discard_night, keep_all, discard_all)) => {
            let night_total = keep_night + discard_night;
            let all_total = keep_all + discard_all;
            let night_pct = format_keep_rate_pct(keep_night, night_total);
            let all_pct = format_keep_rate_pct(keep_all, all_total);
            let discard_night_pct = format_keep_rate_pct(discard_night, night_total);
            format!(
                "**Keep rate:** tonight **{night_pct}** keep ({keep_night}/{night_total} since 20:00) · discard **{discard_night_pct}** · all-time **{all_pct}** keep ({keep_all}/{all_total}) · rate only · does not dump rows · `/keeps` for counts · `/keep-streak` for consecutive keeps · `/longest-streak` for the record · `/recent-keeps` for a short tonight list · `/last-keep` for the newest row · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// Consecutive keep/discard streak from the end of results.tsv.
/// Returns `(keep_streak, discard_streak, keep_streak_tonight)`.
pub(super) fn count_results_tsv_streaks() -> Result<(u64, u64, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let mut outcomes: Vec<(bool, bool)> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if !is_keep && !is_discard {
            continue;
        }
        let in_night = parse_run_ts(ts)
            .map(|parsed| parsed >= window_start)
            .unwrap_or(false);
        outcomes.push((is_keep, in_night));
    }
    if outcomes.is_empty() {
        return Ok((0, 0, 0));
    }
    let last_is_keep = outcomes.last().map(|(k, _)| *k).unwrap_or(false);
    let mut streak = 0u64;
    for (is_keep, _) in outcomes.iter().rev() {
        if *is_keep == last_is_keep {
            streak += 1;
        } else {
            break;
        }
    }
    let (keep_streak, discard_streak) = if last_is_keep {
        (streak, 0)
    } else {
        (0, streak)
    };
    let mut keep_tonight = 0u64;
    for (is_keep, in_night) in outcomes.iter().rev() {
        if *is_keep && *in_night {
            keep_tonight += 1;
        } else {
            break;
        }
    }
    Ok((keep_streak, discard_streak, keep_tonight))
}


/// True for short keep-streak asks (`/keep-streak`, `keep streak`, `ratchet streak`…).
/// Consecutive count only — does not dump TSV rows or steal rate / counts / last-row / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_streak_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-streak"
            | "/keepstreak"
            | "/discard-streak"
            | "/discardstreak"
            | "/streak"
            | "keep streak"
            | "the keep streak"
            | "current keep streak"
            | "the current keep streak"
            | "current streak"
            | "the current streak"
            | "discard streak"
            | "the discard streak"
            | "current discard streak"
            | "the current discard streak"
            | "ratchet streak"
            | "the ratchet streak"
            | "overnight keep streak"
            | "tonight keep streak"
            | "keep streak tonight"
            | "streak tonight"
            | "view keep streak"
            | "see keep streak"
            | "show the keep streak"
            | "show me the keep streak"
            | "open keep streak"
            | "open the keep streak"
            | "list keep streak"
            | "list the keep streak"
            | "view discard streak"
            | "see discard streak"
            | "show the discard streak"
            | "show me the discard streak"
            | "open discard streak"
            | "open the discard streak"
            | "list discard streak"
            | "list the discard streak"
            | "view streak"
            | "see streak"
            | "show the streak"
            | "show me the streak"
            | "open streak"
            | "open the streak"
            | "list streak"
            | "list the streak"
            | "what is the keep streak"
            | "whats the keep streak"
            | "what's the keep streak"
            | "what is the streak"
            | "whats the streak"
            | "what's the streak"
    )
}


/// Zero-LLM consecutive keep/discard streak from results.tsv (current + tonight; no row dump).
pub fn format_results_tsv_streak_gateway() -> String {
    match count_results_tsv_streaks() {
        Err(_) => {
            "**Keep streak:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
                .to_string()
        }
        Ok((keep_streak, discard_streak, keep_tonight)) => {
            format!(
                "**Keep streak:** current **{keep_streak}** keep(s) in a row · tonight **{keep_tonight}** since 20:00 · discard streak **{discard_streak}** · streak only · does not dump rows · `/longest-streak` for the record · `/keeps` for counts · `/keep-rate` for hit rate · `/recent-keeps` for a short tonight list · `/last-keep` for the newest row · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// Longest keep/discard streak across results.tsv.
/// Returns `(longest_keep_all, longest_discard_all, longest_keep_tonight)`.
pub(super) fn count_results_tsv_longest_streaks() -> Result<(u64, u64, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let mut longest_keep = 0u64;
    let mut longest_discard = 0u64;
    let mut cur_keep = 0u64;
    let mut cur_discard = 0u64;
    let mut longest_keep_tonight = 0u64;
    let mut cur_keep_tonight = 0u64;
    let mut saw_row = false;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if !is_keep && !is_discard {
            continue;
        }
        saw_row = true;
        let in_night = parse_run_ts(ts)
            .map(|parsed| parsed >= window_start)
            .unwrap_or(false);
        if is_keep {
            cur_keep += 1;
            cur_discard = 0;
            longest_keep = longest_keep.max(cur_keep);
            if in_night {
                cur_keep_tonight += 1;
                longest_keep_tonight = longest_keep_tonight.max(cur_keep_tonight);
            } else {
                cur_keep_tonight = 0;
            }
        } else {
            cur_discard += 1;
            cur_keep = 0;
            cur_keep_tonight = 0;
            longest_discard = longest_discard.max(cur_discard);
        }
    }
    if !saw_row {
        return Ok((0, 0, 0));
    }
    Ok((longest_keep, longest_discard, longest_keep_tonight))
}


/// True for short longest-streak asks (`/longest-streak`, `longest streak`, `best streak`…).
/// Record max only — does not dump TSV rows or steal current streak / rate / counts / last-row / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_longest_streak_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/longest-streak"
            | "/longeststreak"
            | "/longest-keep-streak"
            | "/longestkeepstreak"
            | "/best-streak"
            | "/beststreak"
            | "/record-streak"
            | "/recordstreak"
            | "/max-streak"
            | "/maxstreak"
            | "/longest-discard-streak"
            | "/longestdiscardstreak"
            | "longest streak"
            | "the longest streak"
            | "longest keep streak"
            | "the longest keep streak"
            | "longest discard streak"
            | "the longest discard streak"
            | "best streak"
            | "the best streak"
            | "best keep streak"
            | "the best keep streak"
            | "record streak"
            | "the record streak"
            | "record keep streak"
            | "the record keep streak"
            | "max streak"
            | "the max streak"
            | "max keep streak"
            | "the max keep streak"
            | "longest streak tonight"
            | "tonight longest streak"
            | "longest keep streak tonight"
            | "view longest streak"
            | "see longest streak"
            | "show the longest streak"
            | "show me the longest streak"
            | "open longest streak"
            | "open the longest streak"
            | "list longest streak"
            | "list the longest streak"
            | "view best streak"
            | "see best streak"
            | "show the best streak"
            | "show me the best streak"
            | "open best streak"
            | "open the best streak"
            | "list best streak"
            | "list the best streak"
            | "view record streak"
            | "see record streak"
            | "show the record streak"
            | "show me the record streak"
            | "open record streak"
            | "open the record streak"
            | "what is the longest streak"
            | "whats the longest streak"
            | "what's the longest streak"
            | "what is the best streak"
            | "whats the best streak"
            | "what's the best streak"
            | "what is the record streak"
            | "whats the record streak"
            | "what's the record streak"
    )
}


/// Zero-LLM longest keep/discard streak from results.tsv (all-time + tonight; no row dump).
pub fn format_results_tsv_longest_streak_gateway() -> String {
    match count_results_tsv_longest_streaks() {
        Err(_) => {
            "**Longest streak:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
                .to_string()
        }
        Ok((longest_keep, longest_discard, longest_keep_tonight)) => {
            format!(
                "**Longest streak:** all-time keep **{longest_keep}** in a row · tonight keep **{longest_keep_tonight}** since 20:00 · longest discard **{longest_discard}** · record only · does not dump rows · `/keep-streak` for the current streak · `/keeps` for counts · `/keep-rate` for hit rate · `/recent-keeps` for a short tonight list · `/last-keep` for the newest row · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// True for short since-last-keep asks (`/since-keep`, `time since last keep`…).
/// Age of newest keep/discard row only — does not dump description or steal last-row / counts / rate / streak / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_since_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/since-keep"
            | "/sincekeep"
            | "/since-last-keep"
            | "/sincelastkeep"
            | "/time-since-keep"
            | "/timesincekeep"
            | "/since-discard"
            | "/sincediscard"
            | "/since-last-discard"
            | "/sincelastdiscard"
            | "/time-since-discard"
            | "/timesincediscard"
            | "since last keep"
            | "the since last keep"
            | "time since last keep"
            | "the time since last keep"
            | "how long since last keep"
            | "how long since the last keep"
            | "how long ago was the last keep"
            | "how long ago was last keep"
            | "since keep"
            | "the since keep"
            | "time since keep"
            | "how long since keep"
            | "view since keep"
            | "see since keep"
            | "show the since keep"
            | "show me the since keep"
            | "open since keep"
            | "open the since keep"
            | "list since keep"
            | "list the since keep"
            | "view since last keep"
            | "see since last keep"
            | "show the since last keep"
            | "show me the since last keep"
            | "open since last keep"
            | "open the since last keep"
            | "list since last keep"
            | "list the since last keep"
            | "what is the time since last keep"
            | "whats the time since last keep"
            | "what's the time since last keep"
            | "since last discard"
            | "the since last discard"
            | "time since last discard"
            | "the time since last discard"
            | "how long since last discard"
            | "how long since the last discard"
            | "how long ago was the last discard"
            | "how long ago was last discard"
            | "since discard"
            | "the since discard"
            | "time since discard"
            | "how long since discard"
            | "view since discard"
            | "see since discard"
            | "show the since discard"
            | "show me the since discard"
            | "open since discard"
            | "open the since discard"
            | "list since discard"
            | "list the since discard"
            | "view since last discard"
            | "see since last discard"
            | "show the since last discard"
            | "show me the since last discard"
            | "open since last discard"
            | "open the since last discard"
            | "list since last discard"
            | "list the since last discard"
            | "what is the time since last discard"
            | "whats the time since last discard"
            | "what's the time since last discard"
    )
}


/// Zero-LLM age since newest keep or discard row (age only; no description dump).
pub fn format_results_tsv_since_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Since last keep",
        ResultsTsvLastWant::Discard => "Since last discard",
    };
    match last_results_tsv_row(want) {
        Err(_) => format!(
            "**{label}:** no matching row yet · overnight keep/discard will create it · `/last-keep` for the newest row · `/keeps` for counts · `results.tsv path` for the file."
        ),
        Ok((ts, _sha, _desc)) => {
            let age = parse_run_ts(&ts)
                .map(|dt| {
                    let ms = dt.timestamp_millis().max(0) as u64;
                    age_from_ms(ms)
                })
                .unwrap_or_else(|| "unknown".to_string());
            format!(
                "**{label}:** **{age}** ago · age only · does not dump the row · `/last-keep` for description · `/first-keep` for tonight's first · `/keep-pace` for avg gap · `/keeps` for counts · `/keep-rate` for hit rate · `/keep-streak` for consecutive keeps · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// Earliest matching keep or discard row tonight (since 20:00 local; one row only).
pub(super) fn first_results_tsv_row_tonight(
    want: ResultsTsvLastWant,
) -> Result<(String, String, String), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let sha = cols.next().unwrap_or("").trim();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        if parsed < window_start {
            continue;
        }
        let desc = cols.next().unwrap_or("").trim();
        // Append order is chronological — first in-window match is earliest tonight.
        return Ok((ts.to_string(), sha.to_string(), desc.to_string()));
    }
    Err("empty".into())
}


/// True for short first-keep / first-discard asks (`/first-keep`, `first keep tonight`…).
/// Earliest tonight row only — does not dump TSV or steal last-row / since-age / recent / counts / rate / streak / path/size/age / morning surprise.
pub fn looks_like_results_tsv_first_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/first-keep"
            | "/firstkeep"
            | "/first-discard"
            | "/firstdiscard"
            | "/earliest-keep"
            | "/earliestkeep"
            | "/earliest-discard"
            | "/earliestdiscard"
            | "/opening-keep"
            | "/openingkeep"
            | "/opening-discard"
            | "/openingdiscard"
            | "first keep"
            | "the first keep"
            | "first keep tonight"
            | "the first keep tonight"
            | "tonight first keep"
            | "first keep of tonight"
            | "first keep of the night"
            | "earliest keep"
            | "the earliest keep"
            | "earliest keep tonight"
            | "opening keep"
            | "the opening keep"
            | "opening keep tonight"
            | "what was the first keep"
            | "what is the first keep"
            | "whats the first keep"
            | "what's the first keep"
            | "when was the first keep"
            | "view first keep"
            | "see first keep"
            | "show the first keep"
            | "show me the first keep"
            | "open first keep"
            | "open the first keep"
            | "list first keep"
            | "list the first keep"
            | "view first keep tonight"
            | "see first keep tonight"
            | "show the first keep tonight"
            | "show me the first keep tonight"
            | "open first keep tonight"
            | "open the first keep tonight"
            | "list first keep tonight"
            | "list the first keep tonight"
            | "first discard"
            | "the first discard"
            | "first discard tonight"
            | "the first discard tonight"
            | "tonight first discard"
            | "first discard of tonight"
            | "first discard of the night"
            | "earliest discard"
            | "the earliest discard"
            | "earliest discard tonight"
            | "opening discard"
            | "the opening discard"
            | "opening discard tonight"
            | "what was the first discard"
            | "what is the first discard"
            | "whats the first discard"
            | "what's the first discard"
            | "when was the first discard"
            | "view first discard"
            | "see first discard"
            | "show the first discard"
            | "show me the first discard"
            | "open first discard"
            | "open the first discard"
            | "list first discard"
            | "list the first discard"
            | "view first discard tonight"
            | "see first discard tonight"
            | "show the first discard tonight"
            | "show me the first discard tonight"
            | "open first discard tonight"
            | "open the first discard tonight"
            | "list first discard tonight"
            | "list the first discard tonight"
    )
}


/// Zero-LLM earliest keep or discard row tonight from results.tsv (one description; no full dump).
pub fn format_results_tsv_first_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "First keep tonight",
        ResultsTsvLastWant::Discard => "First discard tonight",
    };
    match first_results_tsv_row_tonight(want) {
        Err(_) => format!(
            "**{label}:** none tonight since 20:00 · overnight keep/discard will create it · `/last-keep` for the newest row · `/keeps` for counts · `results.tsv path` for the file."
        ),
        Ok((ts, sha, desc)) => {
            let age = parse_run_ts(&ts)
                .map(|dt| {
                    let ms = dt.timestamp_millis().max(0) as u64;
                    age_from_ms(ms)
                })
                .unwrap_or_else(|| "unknown".to_string());
            let short_sha = if sha.len() > 7 { &sha[..7] } else { sha.as_str() };
            let mut desc = desc;
            if desc.chars().count() > 160 {
                desc = desc.chars().take(157).collect::<String>() + "…";
            }
            if desc.is_empty() {
                desc = "(no description)".to_string();
            }
            format!(
                "**{label}:** **{age}** ago · `{short_sha}` · {desc} · first tonight since 20:00 only · does not dump the log · `/last-keep` for the newest row · `/since-keep` for age since newest · `/keep-pace` for avg gap · `/keeps` for counts · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// Human label for a duration (gap / pace), not an "ago" age from now.
pub(super) fn duration_label_secs(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        if m == 0 {
            format!("{h}h")
        } else {
            format!("{h}h{m}m")
        }
    } else {
        format!("{}d", secs / 86400)
    }
}


/// Average gap between consecutive matching keep/discard rows.
/// Returns `(avg_secs_night, gaps_night, avg_secs_all, gaps_all)`.
pub(super) fn count_results_tsv_pace(
    want: ResultsTsvLastWant,
) -> Result<(Option<u64>, u64, Option<u64>, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    fn avg_gap_secs(ts: &[chrono::DateTime<chrono::Utc>]) -> (Option<u64>, u64) {
        if ts.len() < 2 {
            return (None, 0);
        }
        let mut sum = 0u64;
        let mut gaps = 0u64;
        for w in ts.windows(2) {
            let d = (w[1] - w[0]).num_seconds().max(0) as u64;
            sum = sum.saturating_add(d);
            gaps += 1;
        }
        if gaps == 0 {
            return (None, 0);
        }
        (Some(sum / gaps), gaps)
    }
    let (avg_all, gaps_all) = avg_gap_secs(&all_ts);
    let night_ts: Vec<_> = all_ts
        .into_iter()
        .filter(|t| *t >= window_start)
        .collect();
    let (avg_night, gaps_night) = avg_gap_secs(&night_ts);
    Ok((avg_night, gaps_night, avg_all, gaps_all))
}


/// True for short keep-pace asks (`/keep-pace`, `keep pace`, `time between keeps`…).
/// Average gap only — does not dump TSV or steal first / last / since / counts / rate / streak / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_pace_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-pace"
            | "/keeppace"
            | "/discard-pace"
            | "/discardpace"
            | "/keep-gap"
            | "/keepgap"
            | "/discard-gap"
            | "/discardgap"
            | "/avg-keep-gap"
            | "/avgkeepgap"
            | "/average-keep-gap"
            | "/averagekeepgap"
            | "/time-between-keeps"
            | "/timebetweenkeeps"
            | "/time-between-discards"
            | "/timebetweendiscards"
            | "keep pace"
            | "the keep pace"
            | "keep pace tonight"
            | "tonight keep pace"
            | "ratchet pace"
            | "the ratchet pace"
            | "overnight keep pace"
            | "keep gap"
            | "the keep gap"
            | "average keep gap"
            | "the average keep gap"
            | "avg keep gap"
            | "the avg keep gap"
            | "mean keep gap"
            | "the mean keep gap"
            | "average gap between keeps"
            | "avg gap between keeps"
            | "mean gap between keeps"
            | "time between keeps"
            | "the time between keeps"
            | "average time between keeps"
            | "avg time between keeps"
            | "mean time between keeps"
            | "interval between keeps"
            | "keep interval"
            | "the keep interval"
            | "average keep interval"
            | "view keep pace"
            | "see keep pace"
            | "show the keep pace"
            | "show me the keep pace"
            | "open keep pace"
            | "open the keep pace"
            | "list keep pace"
            | "list the keep pace"
            | "what is the keep pace"
            | "whats the keep pace"
            | "what's the keep pace"
            | "discard pace"
            | "the discard pace"
            | "discard pace tonight"
            | "tonight discard pace"
            | "discard gap"
            | "the discard gap"
            | "average discard gap"
            | "avg discard gap"
            | "mean discard gap"
            | "time between discards"
            | "the time between discards"
            | "average time between discards"
            | "avg time between discards"
            | "mean time between discards"
            | "interval between discards"
            | "discard interval"
            | "view discard pace"
            | "see discard pace"
            | "show the discard pace"
            | "show me the discard pace"
            | "open discard pace"
            | "open the discard pace"
            | "list discard pace"
            | "list the discard pace"
            | "what is the discard pace"
            | "whats the discard pace"
            | "what's the discard pace"
    )
}


/// Zero-LLM average gap between keep/discard rows from results.tsv (tonight + all-time; no row dump).
pub fn format_results_tsv_pace_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Keep pace",
        ResultsTsvLastWant::Discard => "Discard pace",
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_pace(want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((avg_night, gaps_night, avg_all, gaps_all)) => {
            let night_part = match avg_night {
                Some(secs) => {
                    let d = duration_label_secs(secs);
                    format!("tonight avg **{d}** between {unit} ({gaps_night} gaps since 20:00)")
                }
                None => format!("tonight need ≥2 {unit} since 20:00 to measure pace"),
            };
            let all_part = match avg_all {
                Some(secs) => {
                    let d = duration_label_secs(secs);
                    format!("all-time avg **{d}** ({gaps_all} gaps)")
                }
                None => format!("all-time need ≥2 {unit}"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · pace only · does not dump rows · `/keep-median` for median gap · `/keep-range` for min–max gap · `/keep-p90` for p90 gap · `/keep-p95` for p95 gap · `/keep-p10` for p10 gap · `/keep-p25` for p25 gap · `/keep-iqr` for IQR · `/keep-std` for std · `/keep-mad` for MAD · `/keep-cv` for CV · `/keep-skew` for skew · `/keep-kurtosis` for kurtosis · `/keep-entropy` for entropy · `/keep-mode` for mode · `/keep-gini` for Gini · `/keep-histogram` for histogram · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// Median gap between consecutive matching keep/discard rows.
/// Returns `(median_secs_night, gaps_night, median_secs_all, gaps_all)`.
pub(super) fn count_results_tsv_median(
    want: ResultsTsvLastWant,
) -> Result<(Option<u64>, u64, Option<u64>, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    fn median_gap_secs(ts: &[chrono::DateTime<chrono::Utc>]) -> (Option<u64>, u64) {
        if ts.len() < 2 {
            return (None, 0);
        }
        let mut gaps: Vec<u64> = Vec::with_capacity(ts.len() - 1);
        for w in ts.windows(2) {
            gaps.push((w[1] - w[0]).num_seconds().max(0) as u64);
        }
        let n = gaps.len() as u64;
        if n == 0 {
            return (None, 0);
        }
        gaps.sort_unstable();
        let med = if gaps.len() % 2 == 1 {
            gaps[gaps.len() / 2]
        } else {
            let a = gaps[gaps.len() / 2 - 1];
            let b = gaps[gaps.len() / 2];
            a.saturating_add(b) / 2
        };
        (Some(med), n)
    }
    let (med_all, gaps_all) = median_gap_secs(&all_ts);
    let night_ts: Vec<_> = all_ts
        .into_iter()
        .filter(|t| *t >= window_start)
        .collect();
    let (med_night, gaps_night) = median_gap_secs(&night_ts);
    Ok((med_night, gaps_night, med_all, gaps_all))
}


/// True for short keep-median asks (`/keep-median`, `median keep gap`…).
/// Median gap only — does not dump TSV or steal pace / first / last / since / counts / rate / streak / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_median_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-median"
            | "/keepmedian"
            | "/discard-median"
            | "/discardmedian"
            | "/median-keep-gap"
            | "/mediankeepgap"
            | "/median-discard-gap"
            | "/mediandiscardgap"
            | "/median-gap"
            | "/mediangap"
            | "/median-keep"
            | "/mediankeep"
            | "/median-discard"
            | "/mediandiscard"
            | "keep median"
            | "the keep median"
            | "keep median tonight"
            | "tonight keep median"
            | "median keep"
            | "the median keep"
            | "median keep gap"
            | "the median keep gap"
            | "median gap between keeps"
            | "median time between keeps"
            | "median keep interval"
            | "the median keep interval"
            | "ratchet median"
            | "the ratchet median"
            | "overnight keep median"
            | "view keep median"
            | "see keep median"
            | "show the keep median"
            | "show me the keep median"
            | "open keep median"
            | "open the keep median"
            | "list keep median"
            | "list the keep median"
            | "what is the keep median"
            | "whats the keep median"
            | "what's the keep median"
            | "what is the median keep gap"
            | "whats the median keep gap"
            | "what's the median keep gap"
            | "discard median"
            | "the discard median"
            | "discard median tonight"
            | "tonight discard median"
            | "median discard"
            | "the median discard"
            | "median discard gap"
            | "the median discard gap"
            | "median gap between discards"
            | "median time between discards"
            | "median discard interval"
            | "view discard median"
            | "see discard median"
            | "show the discard median"
            | "show me the discard median"
            | "open discard median"
            | "open the discard median"
            | "list discard median"
            | "list the discard median"
            | "what is the discard median"
            | "whats the discard median"
            | "what's the discard median"
    )
}


/// Zero-LLM median gap between keep/discard rows from results.tsv (tonight + all-time; no row dump).
pub fn format_results_tsv_median_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Keep median",
        ResultsTsvLastWant::Discard => "Discard median",
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_median(want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((med_night, gaps_night, med_all, gaps_all)) => {
            let night_part = match med_night {
                Some(secs) => {
                    let d = duration_label_secs(secs);
                    format!("tonight median **{d}** between {unit} ({gaps_night} gaps since 20:00)")
                }
                None => format!("tonight need ≥2 {unit} since 20:00 to measure median"),
            };
            let all_part = match med_all {
                Some(secs) => {
                    let d = duration_label_secs(secs);
                    format!("all-time median **{d}** ({gaps_all} gaps)")
                }
                None => format!("all-time need ≥2 {unit}"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · median only · does not dump rows · `/keep-pace` for average gap · `/keep-range` for min–max gap · `/keep-p90` for p90 gap · `/keep-p95` for p95 gap · `/keep-p10` for p10 gap · `/keep-p25` for p25 gap · `/keep-iqr` for IQR · `/keep-std` for std · `/keep-mad` for MAD · `/keep-cv` for CV · `/keep-skew` for skew · `/keep-kurtosis` for kurtosis · `/keep-entropy` for entropy · `/keep-mode` for mode · `/keep-gini` for Gini · `/keep-histogram` for histogram · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}



/// Min/max gap between consecutive matching keep/discard rows.
/// Returns `(min_night, max_night, gaps_night, min_all, max_all, gaps_all)`.
pub(super) fn count_results_tsv_range(
    want: ResultsTsvLastWant,
) -> Result<(Option<u64>, Option<u64>, u64, Option<u64>, Option<u64>, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    fn range_gap_secs(ts: &[chrono::DateTime<chrono::Utc>]) -> (Option<u64>, Option<u64>, u64) {
        if ts.len() < 2 {
            return (None, None, 0);
        }
        let mut min_g: Option<u64> = None;
        let mut max_g: Option<u64> = None;
        let mut n = 0u64;
        for w in ts.windows(2) {
            let g = (w[1] - w[0]).num_seconds().max(0) as u64;
            min_g = Some(min_g.map_or(g, |m| m.min(g)));
            max_g = Some(max_g.map_or(g, |m| m.max(g)));
            n += 1;
        }
        (min_g, max_g, n)
    }
    let (min_all, max_all, gaps_all) = range_gap_secs(&all_ts);
    let night_ts: Vec<_> = all_ts
        .into_iter()
        .filter(|t| *t >= window_start)
        .collect();
    let (min_night, max_night, gaps_night) = range_gap_secs(&night_ts);
    Ok((min_night, max_night, gaps_night, min_all, max_all, gaps_all))
}


/// True for short keep-range asks (`/keep-range`, `keep gap range`, `min max keep gap`…).
/// Min–max gap only — does not dump TSV or steal median / pace / first / last / since / counts / rate / streak / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_range_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-range"
            | "/keeprange"
            | "/discard-range"
            | "/discardrange"
            | "/gap-range"
            | "/gaprange"
            | "/keep-gap-range"
            | "/keepgaprange"
            | "/discard-gap-range"
            | "/discardgaprange"
            | "/keep-spread"
            | "/keepspread"
            | "/discard-spread"
            | "/discardspread"
            | "/min-max-keep-gap"
            | "/minmaxkeepgap"
            | "/min-max-gap"
            | "/minmaxgap"
            | "keep range"
            | "the keep range"
            | "keep range tonight"
            | "tonight keep range"
            | "keep gap range"
            | "the keep gap range"
            | "gap range"
            | "the gap range"
            | "range of keep gaps"
            | "range of gaps between keeps"
            | "keep gap spread"
            | "the keep gap spread"
            | "gap spread"
            | "the gap spread"
            | "min max keep gap"
            | "the min max keep gap"
            | "min-max keep gap"
            | "minmax keep gap"
            | "shortest and longest keep gap"
            | "longest and shortest keep gap"
            | "shortest and longest gap"
            | "ratchet range"
            | "the ratchet range"
            | "overnight keep range"
            | "view keep range"
            | "see keep range"
            | "show the keep range"
            | "show me the keep range"
            | "open keep range"
            | "open the keep range"
            | "list keep range"
            | "list the keep range"
            | "what is the keep range"
            | "whats the keep range"
            | "what's the keep range"
            | "what is the keep gap range"
            | "whats the keep gap range"
            | "what's the keep gap range"
            | "discard range"
            | "the discard range"
            | "discard range tonight"
            | "tonight discard range"
            | "discard gap range"
            | "the discard gap range"
            | "range of discard gaps"
            | "discard gap spread"
            | "min max discard gap"
            | "min-max discard gap"
            | "view discard range"
            | "see discard range"
            | "show the discard range"
            | "show me the discard range"
            | "open discard range"
            | "open the discard range"
            | "list discard range"
            | "list the discard range"
            | "what is the discard range"
            | "whats the discard range"
            | "what's the discard range"
    )
}


/// Zero-LLM min–max gap between keep/discard rows from results.tsv (tonight + all-time; no row dump).
pub fn format_results_tsv_range_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Keep range",
        ResultsTsvLastWant::Discard => "Discard range",
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_range(want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((min_night, max_night, gaps_night, min_all, max_all, gaps_all)) => {
            let night_part = match (min_night, max_night) {
                (Some(lo), Some(hi)) => {
                    let dlo = duration_label_secs(lo);
                    let dhi = duration_label_secs(hi);
                    format!(
                        "tonight **{dlo}–{dhi}** between {unit} ({gaps_night} gaps since 20:00)"
                    )
                }
                _ => format!("tonight need ≥2 {unit} since 20:00 to measure range"),
            };
            let all_part = match (min_all, max_all) {
                (Some(lo), Some(hi)) => {
                    let dlo = duration_label_secs(lo);
                    let dhi = duration_label_secs(hi);
                    format!("all-time **{dlo}–{dhi}** ({gaps_all} gaps)")
                }
                _ => format!("all-time need ≥2 {unit}"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · range only · does not dump rows · `/keep-pace` for average gap · `/keep-median` for median gap · `/keep-p90` for p90 gap · `/keep-p95` for p95 gap · `/keep-p10` for p10 gap · `/keep-p25` for p25 gap · `/keep-iqr` for IQR · `/keep-std` for std · `/keep-mad` for MAD · `/keep-cv` for CV · `/keep-skew` for skew · `/keep-kurtosis` for kurtosis · `/keep-entropy` for entropy · `/keep-mode` for mode · `/keep-gini` for Gini · `/keep-histogram` for histogram · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// IQR (Q3−Q1) gap between consecutive matching keep/discard rows (nearest-rank quartiles).
/// Returns `(iqr_night, q1_night, q3_night, gaps_night, iqr_all, q1_all, q3_all, gaps_all)`.
pub(super) fn count_results_tsv_iqr(
    want: ResultsTsvLastWant,
) -> Result<
    (
        Option<u64>,
        Option<u64>,
        Option<u64>,
        u64,
        Option<u64>,
        Option<u64>,
        Option<u64>,
        u64,
    ),
    String,
> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    fn iqr_gap_secs(
        ts: &[chrono::DateTime<chrono::Utc>],
    ) -> (Option<u64>, Option<u64>, Option<u64>, u64) {
        if ts.len() < 2 {
            return (None, None, None, 0);
        }
        let mut gaps: Vec<u64> = Vec::with_capacity(ts.len() - 1);
        for w in ts.windows(2) {
            gaps.push((w[1] - w[0]).num_seconds().max(0) as u64);
        }
        let n = gaps.len() as u64;
        if n == 0 {
            return (None, None, None, 0);
        }
        gaps.sort_unstable();
        // Nearest-rank: ceil(p * N), 1-indexed → 0-based index.
        let rank = |p: f64| -> usize {
            let r = ((n as f64) * p).ceil() as usize;
            r.saturating_sub(1).min(gaps.len() - 1)
        };
        let q1 = gaps[rank(0.25)];
        let q3 = gaps[rank(0.75)];
        let iqr = q3.saturating_sub(q1);
        (Some(iqr), Some(q1), Some(q3), n)
    }
    let (iqr_all, q1_all, q3_all, gaps_all) = iqr_gap_secs(&all_ts);
    let night_ts: Vec<_> = all_ts
        .into_iter()
        .filter(|t| *t >= window_start)
        .collect();
    let (iqr_night, q1_night, q3_night, gaps_night) = iqr_gap_secs(&night_ts);
    Ok((
        iqr_night, q1_night, q3_night, gaps_night, iqr_all, q1_all, q3_all, gaps_all,
    ))
}


/// True for short keep-iqr asks (`/keep-iqr`, `keep iqr`, `interquartile keep gap`…).
/// IQR only — does not dump TSV or steal p90 / range / median / pace / first / last / since / counts / rate / streak / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_iqr_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-iqr"
            | "/keepiqr"
            | "/discard-iqr"
            | "/discardiqr"
            | "/iqr-keep-gap"
            | "/iqrkeepgap"
            | "/iqr-discard-gap"
            | "/iqrdiscardgap"
            | "/iqr-gap"
            | "/iqrgap"
            | "/iqr-keep"
            | "/iqrkeep"
            | "/iqr-discard"
            | "/iqrdiscard"
            | "/keep-interquartile"
            | "/keepinterquartile"
            | "/discard-interquartile"
            | "/discardinterquartile"
            | "keep iqr"
            | "the keep iqr"
            | "keep iqr tonight"
            | "tonight keep iqr"
            | "iqr keep"
            | "the iqr keep"
            | "iqr keep gap"
            | "the iqr keep gap"
            | "iqr gap between keeps"
            | "iqr time between keeps"
            | "interquartile keep gap"
            | "the interquartile keep gap"
            | "interquartile range keep gap"
            | "interquartile gap between keeps"
            | "keep gap iqr"
            | "ratchet iqr"
            | "the ratchet iqr"
            | "overnight keep iqr"
            | "view keep iqr"
            | "see keep iqr"
            | "show the keep iqr"
            | "show me the keep iqr"
            | "open keep iqr"
            | "open the keep iqr"
            | "list keep iqr"
            | "list the keep iqr"
            | "what is the keep iqr"
            | "whats the keep iqr"
            | "what's the keep iqr"
            | "what is the iqr keep gap"
            | "whats the iqr keep gap"
            | "what's the iqr keep gap"
            | "discard iqr"
            | "the discard iqr"
            | "discard iqr tonight"
            | "tonight discard iqr"
            | "iqr discard"
            | "the iqr discard"
            | "iqr discard gap"
            | "the iqr discard gap"
            | "iqr gap between discards"
            | "interquartile discard gap"
            | "view discard iqr"
            | "see discard iqr"
            | "show the discard iqr"
            | "show me the discard iqr"
            | "open discard iqr"
            | "open the discard iqr"
            | "list discard iqr"
            | "list the discard iqr"
            | "what is the discard iqr"
            | "whats the discard iqr"
            | "what's the discard iqr"
    )
}


/// Zero-LLM IQR gap between keep/discard rows from results.tsv (tonight + all-time; no row dump).
pub fn format_results_tsv_iqr_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Keep iqr",
        ResultsTsvLastWant::Discard => "Discard iqr",
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_iqr(want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((iqr_night, q1_night, q3_night, gaps_night, iqr_all, q1_all, q3_all, gaps_all)) => {
            let night_part = match (iqr_night, q1_night, q3_night) {
                (Some(iqr), Some(q1), Some(q3)) => {
                    let di = duration_label_secs(iqr);
                    let d1 = duration_label_secs(q1);
                    let d3 = duration_label_secs(q3);
                    format!(
                        "tonight IQR **{di}** (Q1 {d1} · Q3 {d3} · {gaps_night} gaps between {unit} since 20:00)"
                    )
                }
                _ => format!("tonight need ≥2 {unit} since 20:00 to measure IQR"),
            };
            let all_part = match (iqr_all, q1_all, q3_all) {
                (Some(iqr), Some(q1), Some(q3)) => {
                    let di = duration_label_secs(iqr);
                    let d1 = duration_label_secs(q1);
                    let d3 = duration_label_secs(q3);
                    format!("all-time IQR **{di}** (Q1 {d1} · Q3 {d3} · {gaps_all} gaps)")
                }
                _ => format!("all-time need ≥2 {unit}"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · IQR only · does not dump rows · `/keep-pace` for average gap · `/keep-median` for median gap · `/keep-range` for min–max gap · `/keep-p90` for p90 gap · `/keep-p95` for p95 gap · `/keep-p10` for p10 gap · `/keep-p25` for p25 gap · `/keep-iqr` for IQR · `/keep-std` for std · `/keep-mad` for MAD · `/keep-cv` for CV · `/keep-skew` for skew · `/keep-kurtosis` for kurtosis · `/keep-entropy` for entropy · `/keep-mode` for mode · `/keep-gini` for Gini · `/keep-histogram` for histogram · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}



/// Sample standard deviation of gaps between consecutive matching keep/discard rows.
/// Returns `(std_night, mean_night, gaps_night, std_all, mean_all, gaps_all)`.
pub(super) fn count_results_tsv_std(
    want: ResultsTsvLastWant,
) -> Result<(Option<u64>, Option<u64>, u64, Option<u64>, Option<u64>, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    fn std_gap_secs(
        ts: &[chrono::DateTime<chrono::Utc>],
    ) -> (Option<u64>, Option<u64>, u64) {
        // Need ≥2 gaps (≥3 rows) for sample std (n−1).
        if ts.len() < 3 {
            let gaps = ts.len().saturating_sub(1) as u64;
            return (None, None, gaps);
        }
        let mut gaps: Vec<f64> = Vec::with_capacity(ts.len() - 1);
        for w in ts.windows(2) {
            gaps.push((w[1] - w[0]).num_seconds().max(0) as f64);
        }
        let n = gaps.len() as f64;
        if n < 2.0 {
            return (None, None, gaps.len() as u64);
        }
        let mean = gaps.iter().sum::<f64>() / n;
        let var = gaps.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std = var.sqrt().round().max(0.0) as u64;
        let mean_u = mean.round().max(0.0) as u64;
        (Some(std), Some(mean_u), gaps.len() as u64)
    }
    let (std_all, mean_all, gaps_all) = std_gap_secs(&all_ts);
    let night_ts: Vec<_> = all_ts
        .into_iter()
        .filter(|t| *t >= window_start)
        .collect();
    let (std_night, mean_night, gaps_night) = std_gap_secs(&night_ts);
    Ok((std_night, mean_night, gaps_night, std_all, mean_all, gaps_all))
}


/// True for short keep-std asks (`/keep-std`, `keep std`, `standard deviation keep gap`…).
/// Std only — does not dump TSV or steal iqr / p90 / range / median / pace / first / last / since / counts / rate / streak / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_std_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-std"
            | "/keepstd"
            | "/discard-std"
            | "/discardstd"
            | "/std-keep-gap"
            | "/stdkeepgap"
            | "/std-discard-gap"
            | "/stddiscardgap"
            | "/std-gap"
            | "/stdgap"
            | "/std-keep"
            | "/stdkeep"
            | "/std-discard"
            | "/stddiscard"
            | "/keep-stddev"
            | "/keepstddev"
            | "/discard-stddev"
            | "/discardstddev"
            | "/keep-standard-deviation"
            | "/keepstandarddeviation"
            | "/discard-standard-deviation"
            | "/discardstandarddeviation"
            | "keep std"
            | "the keep std"
            | "keep std tonight"
            | "tonight keep std"
            | "std keep"
            | "the std keep"
            | "std keep gap"
            | "the std keep gap"
            | "std gap between keeps"
            | "std time between keeps"
            | "stddev keep gap"
            | "the stddev keep gap"
            | "std deviation keep gap"
            | "standard deviation keep gap"
            | "the standard deviation keep gap"
            | "keep gap std"
            | "keep gap stddev"
            | "ratchet std"
            | "the ratchet std"
            | "ratchet stddev"
            | "overnight keep std"
            | "view keep std"
            | "see keep std"
            | "show the keep std"
            | "show me the keep std"
            | "open keep std"
            | "open the keep std"
            | "list keep std"
            | "list the keep std"
            | "what is the keep std"
            | "whats the keep std"
            | "what's the keep std"
            | "what is the std keep gap"
            | "whats the std keep gap"
            | "what's the std keep gap"
            | "discard std"
            | "the discard std"
            | "discard std tonight"
            | "tonight discard std"
            | "std discard"
            | "the std discard"
            | "std discard gap"
            | "the std discard gap"
            | "std gap between discards"
            | "stddev discard gap"
            | "standard deviation discard gap"
            | "view discard std"
            | "see discard std"
            | "show the discard std"
            | "show me the discard std"
            | "open discard std"
            | "open the discard std"
            | "list discard std"
            | "list the discard std"
            | "what is the discard std"
            | "whats the discard std"
            | "what's the discard std"
    )
}


/// Zero-LLM sample-std gap between keep/discard rows from results.tsv (tonight + all-time; no row dump).
pub fn format_results_tsv_std_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Keep std",
        ResultsTsvLastWant::Discard => "Discard std",
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_std(want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((std_night, mean_night, gaps_night, std_all, mean_all, gaps_all)) => {
            let night_part = match (std_night, mean_night) {
                (Some(std), Some(mean)) => {
                    let ds = duration_label_secs(std);
                    let dm = duration_label_secs(mean);
                    format!(
                        "tonight std **{ds}** (mean {dm} · {gaps_night} gaps between {unit} since 20:00)"
                    )
                }
                _ => format!("tonight need ≥3 {unit} (2 gaps) since 20:00 to measure std"),
            };
            let all_part = match (std_all, mean_all) {
                (Some(std), Some(mean)) => {
                    let ds = duration_label_secs(std);
                    let dm = duration_label_secs(mean);
                    format!("all-time std **{ds}** (mean {dm} · {gaps_all} gaps)")
                }
                _ => format!("all-time need ≥3 {unit} (2 gaps)"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · std only · does not dump rows · `/keep-pace` for average gap · `/keep-median` for median gap · `/keep-range` for min–max gap · `/keep-p90` for p90 gap · `/keep-p95` for p95 gap · `/keep-p10` for p10 gap · `/keep-p25` for p25 gap · `/keep-iqr` for IQR · `/keep-std` for std · `/keep-mad` for MAD · `/keep-cv` for CV · `/keep-skew` for skew · `/keep-kurtosis` for kurtosis · `/keep-entropy` for entropy · `/keep-mode` for mode · `/keep-gini` for Gini · `/keep-histogram` for histogram · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// MAD (median absolute deviation) gap between consecutive matching keep/discard rows.
/// Returns `(mad_night, med_night, gaps_night, mad_all, med_all, gaps_all)`.
pub(super) fn count_results_tsv_mad(
    want: ResultsTsvLastWant,
) -> Result<(Option<u64>, Option<u64>, u64, Option<u64>, Option<u64>, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    fn median_u64(vals: &mut [u64]) -> Option<u64> {
        if vals.is_empty() {
            return None;
        }
        vals.sort_unstable();
        let n = vals.len();
        Some(if n % 2 == 1 {
            vals[n / 2]
        } else {
            let a = vals[n / 2 - 1];
            let b = vals[n / 2];
            a.saturating_add(b) / 2
        })
    }
    fn mad_gap_secs(
        ts: &[chrono::DateTime<chrono::Utc>],
    ) -> (Option<u64>, Option<u64>, u64) {
        // Need ≥2 gaps (≥3 rows) for a meaningful MAD (median of |gap − median|).
        if ts.len() < 3 {
            let gaps = ts.len().saturating_sub(1) as u64;
            return (None, None, gaps);
        }
        let mut gaps: Vec<u64> = Vec::with_capacity(ts.len() - 1);
        for w in ts.windows(2) {
            gaps.push((w[1] - w[0]).num_seconds().max(0) as u64);
        }
        let n = gaps.len() as u64;
        if n < 2 {
            return (None, None, n);
        }
        let mut gaps_for_med = gaps.clone();
        let Some(med) = median_u64(&mut gaps_for_med) else {
            return (None, None, n);
        };
        let mut absdevs: Vec<u64> = gaps
            .iter()
            .map(|g| if *g >= med { *g - med } else { med - *g })
            .collect();
        let Some(mad) = median_u64(&mut absdevs) else {
            return (None, None, n);
        };
        (Some(mad), Some(med), n)
    }
    let (mad_all, med_all, gaps_all) = mad_gap_secs(&all_ts);
    let night_ts: Vec<_> = all_ts
        .into_iter()
        .filter(|t| *t >= window_start)
        .collect();
    let (mad_night, med_night, gaps_night) = mad_gap_secs(&night_ts);
    Ok((mad_night, med_night, gaps_night, mad_all, med_all, gaps_all))
}


/// True for short keep-mad asks (`/keep-mad`, `keep mad`, `median absolute deviation keep gap`…).
/// MAD only — does not dump TSV or steal std / iqr / p90 / range / median / pace / first / last / since / counts / rate / streak / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_mad_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-mad"
            | "/keepmad"
            | "/discard-mad"
            | "/discardmad"
            | "/mad-keep-gap"
            | "/madkeepgap"
            | "/mad-discard-gap"
            | "/maddiscardgap"
            | "/mad-gap"
            | "/madgap"
            | "/mad-keep"
            | "/madkeep"
            | "/mad-discard"
            | "/maddiscard"
            | "/keep-median-absolute-deviation"
            | "/keepmedianabsolutedeviation"
            | "/discard-median-absolute-deviation"
            | "/discardmedianabsolutedeviation"
            | "keep mad"
            | "the keep mad"
            | "keep mad tonight"
            | "tonight keep mad"
            | "mad keep"
            | "the mad keep"
            | "mad keep gap"
            | "the mad keep gap"
            | "mad gap between keeps"
            | "mad time between keeps"
            | "median absolute deviation keep gap"
            | "the median absolute deviation keep gap"
            | "median absolute keep gap"
            | "keep gap mad"
            | "ratchet mad"
            | "the ratchet mad"
            | "overnight keep mad"
            | "view keep mad"
            | "see keep mad"
            | "show the keep mad"
            | "show me the keep mad"
            | "open keep mad"
            | "open the keep mad"
            | "list keep mad"
            | "list the keep mad"
            | "what is the keep mad"
            | "whats the keep mad"
            | "what's the keep mad"
            | "what is the mad keep gap"
            | "whats the mad keep gap"
            | "what's the mad keep gap"
            | "discard mad"
            | "the discard mad"
            | "discard mad tonight"
            | "tonight discard mad"
            | "mad discard"
            | "the mad discard"
            | "mad discard gap"
            | "the mad discard gap"
            | "mad gap between discards"
            | "median absolute deviation discard gap"
            | "view discard mad"
            | "see discard mad"
            | "show the discard mad"
            | "show me the discard mad"
            | "open discard mad"
            | "open the discard mad"
            | "list discard mad"
            | "list the discard mad"
            | "what is the discard mad"
            | "whats the discard mad"
            | "what's the discard mad"
    )
}


/// Zero-LLM MAD gap between keep/discard rows from results.tsv (tonight + all-time; no row dump).
pub fn format_results_tsv_mad_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Keep mad",
        ResultsTsvLastWant::Discard => "Discard mad",
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_mad(want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((mad_night, med_night, gaps_night, mad_all, med_all, gaps_all)) => {
            let night_part = match (mad_night, med_night) {
                (Some(mad), Some(med)) => {
                    let dm = duration_label_secs(mad);
                    let dmed = duration_label_secs(med);
                    format!(
                        "tonight MAD **{dm}** (median {dmed} · {gaps_night} gaps between {unit} since 20:00)"
                    )
                }
                _ => format!("tonight need ≥3 {unit} (2 gaps) since 20:00 to measure MAD"),
            };
            let all_part = match (mad_all, med_all) {
                (Some(mad), Some(med)) => {
                    let dm = duration_label_secs(mad);
                    let dmed = duration_label_secs(med);
                    format!("all-time MAD **{dm}** (median {dmed} · {gaps_all} gaps)")
                }
                _ => format!("all-time need ≥3 {unit} (2 gaps)"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · MAD only · does not dump rows · `/keep-pace` for average gap · `/keep-median` for median gap · `/keep-range` for min–max gap · `/keep-p90` for p90 gap · `/keep-p95` for p95 gap · `/keep-p10` for p10 gap · `/keep-p25` for p25 gap · `/keep-iqr` for IQR · `/keep-std` for std · `/keep-mad` for MAD · `/keep-cv` for CV · `/keep-skew` for skew · `/keep-kurtosis` for kurtosis · `/keep-entropy` for entropy · `/keep-mode` for mode · `/keep-gini` for Gini · `/keep-histogram` for histogram · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// CV (coefficient of variation = sample std / mean) of gaps between consecutive matching keep/discard rows.
/// Returns `(cv_pct_night, std_night, mean_night, gaps_night, cv_pct_all, std_all, mean_all, gaps_all)`.
/// `cv_pct` is rounded percent (0–…); None when <2 gaps or mean≈0.
pub(super) fn count_results_tsv_cv(
    want: ResultsTsvLastWant,
) -> Result<
    (
        Option<u64>,
        Option<u64>,
        Option<u64>,
        u64,
        Option<u64>,
        Option<u64>,
        Option<u64>,
        u64,
    ),
    String,
> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    fn cv_gap_secs(
        ts: &[chrono::DateTime<chrono::Utc>],
    ) -> (Option<u64>, Option<u64>, Option<u64>, u64) {
        // Need ≥2 gaps (≥3 rows) for sample std / mean.
        if ts.len() < 3 {
            let gaps = ts.len().saturating_sub(1) as u64;
            return (None, None, None, gaps);
        }
        let mut gaps: Vec<f64> = Vec::with_capacity(ts.len() - 1);
        for w in ts.windows(2) {
            gaps.push((w[1] - w[0]).num_seconds().max(0) as f64);
        }
        let n = gaps.len() as f64;
        if n < 2.0 {
            return (None, None, None, gaps.len() as u64);
        }
        let mean = gaps.iter().sum::<f64>() / n;
        if mean <= 0.0 {
            return (None, None, None, gaps.len() as u64);
        }
        let var = gaps.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std = var.sqrt();
        let cv_pct = ((std / mean) * 100.0).round().max(0.0) as u64;
        let std_u = std.round().max(0.0) as u64;
        let mean_u = mean.round().max(0.0) as u64;
        (Some(cv_pct), Some(std_u), Some(mean_u), gaps.len() as u64)
    }
    let (cv_all, std_all, mean_all, gaps_all) = cv_gap_secs(&all_ts);
    let night_ts: Vec<_> = all_ts
        .into_iter()
        .filter(|t| *t >= window_start)
        .collect();
    let (cv_night, std_night, mean_night, gaps_night) = cv_gap_secs(&night_ts);
    Ok((
        cv_night, std_night, mean_night, gaps_night, cv_all, std_all, mean_all, gaps_all,
    ))
}


/// True for short keep-cv asks (`/keep-cv`, `keep cv`, `coefficient of variation keep gap`…).
/// CV only — does not dump TSV or steal mad / std / iqr / p90 / range / median / pace / first / last / since / counts / rate / streak / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_cv_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-cv"
            | "/keepcv"
            | "/discard-cv"
            | "/discardcv"
            | "/cv-keep-gap"
            | "/cvkeepgap"
            | "/cv-discard-gap"
            | "/cvdiscardgap"
            | "/cv-gap"
            | "/cvgap"
            | "/cv-keep"
            | "/cvkeep"
            | "/cv-discard"
            | "/cvdiscard"
            | "/keep-coefficient-of-variation"
            | "/keepcoefficientofvariation"
            | "/discard-coefficient-of-variation"
            | "/discardcoefficientofvariation"
            | "keep cv"
            | "the keep cv"
            | "keep cv tonight"
            | "tonight keep cv"
            | "cv keep"
            | "the cv keep"
            | "cv keep gap"
            | "the cv keep gap"
            | "cv gap between keeps"
            | "cv time between keeps"
            | "coefficient of variation keep gap"
            | "the coefficient of variation keep gap"
            | "coeff of variation keep gap"
            | "keep gap cv"
            | "keep gap coefficient of variation"
            | "ratchet cv"
            | "the ratchet cv"
            | "overnight keep cv"
            | "view keep cv"
            | "see keep cv"
            | "show the keep cv"
            | "show me the keep cv"
            | "open keep cv"
            | "open the keep cv"
            | "list keep cv"
            | "list the keep cv"
            | "what is the keep cv"
            | "whats the keep cv"
            | "what's the keep cv"
            | "what is the cv keep gap"
            | "whats the cv keep gap"
            | "what's the cv keep gap"
            | "discard cv"
            | "the discard cv"
            | "discard cv tonight"
            | "tonight discard cv"
            | "cv discard"
            | "the cv discard"
            | "cv discard gap"
            | "the cv discard gap"
            | "cv gap between discards"
            | "coefficient of variation discard gap"
            | "view discard cv"
            | "see discard cv"
            | "show the discard cv"
            | "show me the discard cv"
            | "open discard cv"
            | "open the discard cv"
            | "list discard cv"
            | "list the discard cv"
            | "what is the discard cv"
            | "whats the discard cv"
            | "what's the discard cv"
    )
}


/// Zero-LLM CV (std/mean %) gap between keep/discard rows from results.tsv (tonight + all-time; no row dump).
pub fn format_results_tsv_cv_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Keep cv",
        ResultsTsvLastWant::Discard => "Discard cv",
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_cv(want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((cv_night, std_night, mean_night, gaps_night, cv_all, std_all, mean_all, gaps_all)) => {
            let night_part = match (cv_night, std_night, mean_night) {
                (Some(cv), Some(std), Some(mean)) => {
                    let ds = duration_label_secs(std);
                    let dm = duration_label_secs(mean);
                    format!(
                        "tonight CV **{cv}%** (std {ds} · mean {dm} · {gaps_night} gaps between {unit} since 20:00)"
                    )
                }
                _ => format!("tonight need ≥3 {unit} (2 gaps) since 20:00 to measure CV"),
            };
            let all_part = match (cv_all, std_all, mean_all) {
                (Some(cv), Some(std), Some(mean)) => {
                    let ds = duration_label_secs(std);
                    let dm = duration_label_secs(mean);
                    format!("all-time CV **{cv}%** (std {ds} · mean {dm} · {gaps_all} gaps)")
                }
                _ => format!("all-time need ≥3 {unit} (2 gaps)"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · CV only · does not dump rows · `/keep-pace` for average gap · `/keep-median` for median gap · `/keep-range` for min–max gap · `/keep-p90` for p90 gap · `/keep-p95` for p95 gap · `/keep-p10` for p10 gap · `/keep-p25` for p25 gap · `/keep-iqr` for IQR · `/keep-std` for std · `/keep-mad` for MAD · `/keep-cv` for CV · `/keep-skew` for skew · `/keep-kurtosis` for kurtosis · `/keep-entropy` for entropy · `/keep-mode` for mode · `/keep-gini` for Gini · `/keep-histogram` for histogram · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}



/// Skewness (Fisher–Pearson adjusted G1) of gaps between consecutive matching keep/discard rows.
/// Returns `(skew_x100_night, gaps_night, skew_x100_all, gaps_all)`.
/// `skew_x100` is rounded hundredths (42 = +0.42); None when <3 gaps or std≈0.
pub(super) fn count_results_tsv_skew(
    want: ResultsTsvLastWant,
) -> Result<(Option<i64>, u64, Option<i64>, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    fn skew_gap_secs(ts: &[chrono::DateTime<chrono::Utc>]) -> (Option<i64>, u64) {
        // Need ≥3 gaps (≥4 rows) for sample skewness G1.
        if ts.len() < 4 {
            let gaps = ts.len().saturating_sub(1) as u64;
            return (None, gaps);
        }
        let mut gaps: Vec<f64> = Vec::with_capacity(ts.len() - 1);
        for w in ts.windows(2) {
            gaps.push((w[1] - w[0]).num_seconds().max(0) as f64);
        }
        let n = gaps.len() as f64;
        if n < 3.0 {
            return (None, gaps.len() as u64);
        }
        let mean = gaps.iter().sum::<f64>() / n;
        let var = gaps.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std = var.sqrt();
        if std <= 0.0 {
            return (None, gaps.len() as u64);
        }
        let m3 = gaps
            .iter()
            .map(|x| ((x - mean) / std).powi(3))
            .sum::<f64>();
        // Fisher–Pearson adjusted moment coefficient G1.
        let g1 = (n / ((n - 1.0) * (n - 2.0))) * m3;
        let skew_x100 = (g1 * 100.0).round() as i64;
        (Some(skew_x100), gaps.len() as u64)
    }
    let (skew_all, gaps_all) = skew_gap_secs(&all_ts);
    let night_ts: Vec<_> = all_ts
        .into_iter()
        .filter(|t| *t >= window_start)
        .collect();
    let (skew_night, gaps_night) = skew_gap_secs(&night_ts);
    Ok((skew_night, gaps_night, skew_all, gaps_all))
}


/// True for short keep-skew asks (`/keep-skew`, `keep skew`, `skewness keep gap`…).
/// Skew only — does not dump TSV or steal cv / mad / std / iqr / p90 / range / median / pace / first / last / since / counts / rate / streak / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_skew_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-skew"
            | "/keepskew"
            | "/discard-skew"
            | "/discardskew"
            | "/skew-keep-gap"
            | "/skewkeepgap"
            | "/skew-discard-gap"
            | "/skewdiscardgap"
            | "/skew-gap"
            | "/skewgap"
            | "/skew-keep"
            | "/skewkeep"
            | "/skew-discard"
            | "/skewdiscard"
            | "/keep-skewness"
            | "/keepskewness"
            | "/discard-skewness"
            | "/discardskewness"
            | "keep skew"
            | "the keep skew"
            | "keep skew tonight"
            | "tonight keep skew"
            | "skew keep"
            | "the skew keep"
            | "skew keep gap"
            | "the skew keep gap"
            | "skew gap between keeps"
            | "skew time between keeps"
            | "skewness keep gap"
            | "the skewness keep gap"
            | "keep gap skew"
            | "keep gap skewness"
            | "ratchet skew"
            | "the ratchet skew"
            | "overnight keep skew"
            | "view keep skew"
            | "see keep skew"
            | "show the keep skew"
            | "show me the keep skew"
            | "open keep skew"
            | "open the keep skew"
            | "list keep skew"
            | "list the keep skew"
            | "what is the keep skew"
            | "whats the keep skew"
            | "what's the keep skew"
            | "what is the skew keep gap"
            | "whats the skew keep gap"
            | "what's the skew keep gap"
            | "discard skew"
            | "the discard skew"
            | "discard skew tonight"
            | "tonight discard skew"
            | "skew discard"
            | "the skew discard"
            | "skew discard gap"
            | "the skew discard gap"
            | "skew gap between discards"
            | "skewness discard gap"
            | "view discard skew"
            | "see discard skew"
            | "show the discard skew"
            | "show me the discard skew"
            | "open discard skew"
            | "open the discard skew"
            | "list discard skew"
            | "list the discard skew"
            | "what is the discard skew"
            | "whats the discard skew"
            | "what's the discard skew"
    )
}


/// Zero-LLM skewness (Fisher G1) gap between keep/discard rows from results.tsv (tonight + all-time; no row dump).
pub fn format_results_tsv_skew_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Keep skew",
        ResultsTsvLastWant::Discard => "Discard skew",
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_skew(want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((skew_night, gaps_night, skew_all, gaps_all)) => {
            let night_part = match skew_night {
                Some(s) => {
                    let fs = format_skew_x100(s);
                    format!(
                        "tonight skew **{fs}** ({gaps_night} gaps between {unit} since 20:00)"
                    )
                }
                None => format!("tonight need ≥4 {unit} (3 gaps) since 20:00 to measure skew"),
            };
            let all_part = match skew_all {
                Some(s) => {
                    let fs = format_skew_x100(s);
                    format!("all-time skew **{fs}** ({gaps_all} gaps)")
                }
                None => format!("all-time need ≥4 {unit} (3 gaps)"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · skew only · does not dump rows · `/keep-pace` for average gap · `/keep-median` for median gap · `/keep-range` for min–max gap · `/keep-p90` for p90 gap · `/keep-p95` for p95 gap · `/keep-p10` for p10 gap · `/keep-p25` for p25 gap · `/keep-iqr` for IQR · `/keep-std` for std · `/keep-mad` for MAD · `/keep-cv` for CV · `/keep-skew` for skew · `/keep-kurtosis` for kurtosis · `/keep-entropy` for entropy · `/keep-mode` for mode · `/keep-gini` for Gini · `/keep-histogram` for histogram · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// Excess kurtosis (Fisher G2) of gaps between consecutive matching keep/discard rows.
/// Returns `(kurt_x100_night, gaps_night, kurt_x100_all, gaps_all)`.
/// `kurt_x100` is rounded hundredths (42 = +0.42); None when <4 gaps or std≈0.
pub(super) fn count_results_tsv_kurtosis(
    want: ResultsTsvLastWant,
) -> Result<(Option<i64>, u64, Option<i64>, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    fn kurt_gap_secs(ts: &[chrono::DateTime<chrono::Utc>]) -> (Option<i64>, u64) {
        // Need ≥4 gaps (≥5 rows) for sample excess kurtosis G2.
        if ts.len() < 5 {
            let gaps = ts.len().saturating_sub(1) as u64;
            return (None, gaps);
        }
        let mut gaps: Vec<f64> = Vec::with_capacity(ts.len() - 1);
        for w in ts.windows(2) {
            gaps.push((w[1] - w[0]).num_seconds().max(0) as f64);
        }
        let n = gaps.len() as f64;
        if n < 4.0 {
            return (None, gaps.len() as u64);
        }
        let mean = gaps.iter().sum::<f64>() / n;
        let var = gaps.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std = var.sqrt();
        if std <= 0.0 {
            return (None, gaps.len() as u64);
        }
        let m4 = gaps
            .iter()
            .map(|x| ((x - mean) / std).powi(4))
            .sum::<f64>();
        // Fisher excess kurtosis G2 (bias-corrected).
        let g2 = (n * (n + 1.0) / ((n - 1.0) * (n - 2.0) * (n - 3.0))) * m4
            - (3.0 * (n - 1.0).powi(2) / ((n - 2.0) * (n - 3.0)));
        let kurt_x100 = (g2 * 100.0).round() as i64;
        (Some(kurt_x100), gaps.len() as u64)
    }
    let (kurt_all, gaps_all) = kurt_gap_secs(&all_ts);
    let night_ts: Vec<_> = all_ts
        .into_iter()
        .filter(|t| *t >= window_start)
        .collect();
    let (kurt_night, gaps_night) = kurt_gap_secs(&night_ts);
    Ok((kurt_night, gaps_night, kurt_all, gaps_all))
}


/// True for short keep-kurtosis asks (`/keep-kurtosis`, `keep kurtosis`, `kurtosis keep gap`…).
/// Kurtosis only — does not dump TSV or steal skew / cv / mad / std / iqr / p90 / range / median / pace / first / last / since / counts / rate / streak / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_kurtosis_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-kurtosis"
            | "/keepkurtosis"
            | "/discard-kurtosis"
            | "/discardkurtosis"
            | "/kurtosis-keep-gap"
            | "/kurtosiskeepgap"
            | "/kurtosis-discard-gap"
            | "/kurtosisdiscardgap"
            | "/kurtosis-gap"
            | "/kurtosisgap"
            | "/kurtosis-keep"
            | "/kurtosiskeep"
            | "/kurtosis-discard"
            | "/kurtosisdiscard"
            | "/keep-kurt"
            | "/keepkurt"
            | "/discard-kurt"
            | "/discardkurt"
            | "/kurt-keep-gap"
            | "/kurtkeepgap"
            | "/kurt-discard-gap"
            | "/kurtdiscardgap"
            | "/kurt-gap"
            | "/kurtgap"
            | "/excess-kurtosis"
            | "/excesskurtosis"
            | "/keep-excess-kurtosis"
            | "/keepexcesskurtosis"
            | "keep kurtosis"
            | "the keep kurtosis"
            | "keep kurtosis tonight"
            | "tonight keep kurtosis"
            | "kurtosis keep"
            | "the kurtosis keep"
            | "kurtosis keep gap"
            | "the kurtosis keep gap"
            | "kurtosis gap between keeps"
            | "kurtosis time between keeps"
            | "excess kurtosis keep gap"
            | "the excess kurtosis keep gap"
            | "keep gap kurtosis"
            | "keep gap excess kurtosis"
            | "ratchet kurtosis"
            | "the ratchet kurtosis"
            | "overnight keep kurtosis"
            | "view keep kurtosis"
            | "see keep kurtosis"
            | "show the keep kurtosis"
            | "show me the keep kurtosis"
            | "open keep kurtosis"
            | "open the keep kurtosis"
            | "list keep kurtosis"
            | "list the keep kurtosis"
            | "what is the keep kurtosis"
            | "whats the keep kurtosis"
            | "what's the keep kurtosis"
            | "what is the kurtosis keep gap"
            | "whats the kurtosis keep gap"
            | "what's the kurtosis keep gap"
            | "keep kurt"
            | "the keep kurt"
            | "kurt keep gap"
            | "kurt gap between keeps"
            | "view keep kurt"
            | "open keep kurt"
            | "discard kurtosis"
            | "the discard kurtosis"
            | "discard kurtosis tonight"
            | "tonight discard kurtosis"
            | "kurtosis discard"
            | "the kurtosis discard"
            | "kurtosis discard gap"
            | "the kurtosis discard gap"
            | "kurtosis gap between discards"
            | "excess kurtosis discard gap"
            | "view discard kurtosis"
            | "see discard kurtosis"
            | "show the discard kurtosis"
            | "show me the discard kurtosis"
            | "open discard kurtosis"
            | "open the discard kurtosis"
            | "list discard kurtosis"
            | "list the discard kurtosis"
            | "what is the discard kurtosis"
            | "whats the discard kurtosis"
            | "what's the discard kurtosis"
            | "discard kurt"
            | "kurt discard gap"
            | "view discard kurt"
            | "open discard kurt"
    )
}


/// Zero-LLM excess kurtosis (Fisher G2) gap between keep/discard rows from results.tsv (tonight + all-time; no row dump).
pub fn format_results_tsv_kurtosis_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Keep kurtosis",
        ResultsTsvLastWant::Discard => "Discard kurtosis",
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_kurtosis(want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((kurt_night, gaps_night, kurt_all, gaps_all)) => {
            let night_part = match kurt_night {
                Some(k) => {
                    let fk = format_kurt_x100(k);
                    format!(
                        "tonight kurtosis **{fk}** ({gaps_night} gaps between {unit} since 20:00)"
                    )
                }
                None => format!("tonight need ≥5 {unit} (4 gaps) since 20:00 to measure kurtosis"),
            };
            let all_part = match kurt_all {
                Some(k) => {
                    let fk = format_kurt_x100(k);
                    format!("all-time kurtosis **{fk}** ({gaps_all} gaps)")
                }
                None => format!("all-time need ≥5 {unit} (4 gaps)"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · kurtosis only · does not dump rows · `/keep-pace` for average gap · `/keep-median` for median gap · `/keep-range` for min–max gap · `/keep-p90` for p90 gap · `/keep-p95` for p95 gap · `/keep-p10` for p10 gap · `/keep-p25` for p25 gap · `/keep-iqr` for IQR · `/keep-std` for std · `/keep-mad` for MAD · `/keep-cv` for CV · `/keep-skew` for skew · `/keep-kurtosis` for kurtosis · `/keep-entropy` for entropy · `/keep-mode` for mode · `/keep-gini` for Gini · `/keep-histogram` for histogram · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// Modal (most frequent) minute-bucketed gap between consecutive matching keep/discard rows.
/// Returns `(mode_secs_night, gaps_night, hits_night, mode_secs_all, gaps_all, hits_all)`.
/// `mode_secs` is the most common whole-minute gap as seconds; ties pick the smaller minute. None when <1 gap.
/// Modal gap (bits) of minute-bucketed gaps between consecutive matching keep/discard rows.
/// Returns `(entropy_x100_night, gaps_night, buckets_night, entropy_x100_all, gaps_all, buckets_all)`.
/// `entropy_x100` is rounded hundredths of bits (185 = 1.85 bits); None when <2 gaps.
pub(super) fn count_results_tsv_mode(
    want: ResultsTsvLastWant,
) -> Result<(Option<u64>, u64, u64, Option<u64>, u64, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    fn mode_gap_secs(ts: &[chrono::DateTime<chrono::Utc>]) -> (Option<u64>, u64, u64) {
        // Need ≥1 gap (≥2 rows). Mode = most frequent minute bucket; ties → smaller minutes.
        if ts.len() < 2 {
            return (None, 0, 0);
        }
        let mut gaps: Vec<i64> = Vec::with_capacity(ts.len() - 1);
        for w in ts.windows(2) {
            let secs = (w[1] - w[0]).num_seconds().max(0);
            gaps.push(secs / 60);
        }
        let n = gaps.len() as u64;
        if n == 0 {
            return (None, 0, 0);
        }
        let mut counts: std::collections::BTreeMap<i64, u64> = std::collections::BTreeMap::new();
        for g in &gaps {
            *counts.entry(*g).or_insert(0) += 1;
        }
        let mut best_mins = 0_i64;
        let mut best_count = 0_u64;
        for (mins, c) in counts.iter() {
            if *c > best_count || (*c == best_count && *mins < best_mins) {
                best_mins = *mins;
                best_count = *c;
            }
        }
        // Report mode as whole minutes → seconds for duration_label_secs.
        let mode_secs = (best_mins.max(0) as u64).saturating_mul(60);
        (Some(mode_secs), n, best_count)
    }
    let (mode_all, gaps_all, hits_all) = mode_gap_secs(&all_ts);
    let night_ts: Vec<_> = all_ts
        .into_iter()
        .filter(|t| *t >= window_start)
        .collect();
    let (mode_night, gaps_night, hits_night) = mode_gap_secs(&night_ts);
    Ok((
        mode_night,
        gaps_night,
        hits_night,
        mode_all,
        gaps_all,
        hits_all,
    ))
}


/// True for short keep-mode asks (`/keep-mode`, `keep mode`, `mode keep gap`, `modal keep gap`…).
/// Mode (most frequent minute-bucketed gap) only — does not dump TSV or steal entropy / kurtosis / skew / cv / mad / std / iqr / p90 / range / median / pace / first / last / since / counts / rate / streak / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_mode_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-mode"
            | "/keepmode"
            | "/discard-mode"
            | "/discardmode"
            | "/mode-keep-gap"
            | "/modekeepgap"
            | "/mode-discard-gap"
            | "/modediscardgap"
            | "/mode-gap"
            | "/modegap"
            | "/mode-keep"
            | "/modekeep"
            | "/mode-discard"
            | "/modediscard"
            | "/modal-gap"
            | "/modalgap"
            | "/keep-modal-gap"
            | "/keepmodalgap"
            | "/modal-keep-gap"
            | "/modalkeepgap"
            | "keep mode"
            | "the keep mode"
            | "keep mode tonight"
            | "tonight keep mode"
            | "mode keep"
            | "the mode keep"
            | "mode keep gap"
            | "the mode keep gap"
            | "mode gap between keeps"
            | "mode time between keeps"
            | "modal keep gap"
            | "the modal keep gap"
            | "modal gap between keeps"
            | "keep gap mode"
            | "keep gap modal"
            | "ratchet mode"
            | "the ratchet mode"
            | "overnight keep mode"
            | "view keep mode"
            | "see keep mode"
            | "show the keep mode"
            | "show me the keep mode"
            | "open keep mode"
            | "open the keep mode"
            | "list keep mode"
            | "list the keep mode"
            | "what is the keep mode"
            | "whats the keep mode"
            | "what's the keep mode"
            | "what is the mode keep gap"
            | "whats the mode keep gap"
            | "what's the mode keep gap"
            | "discard mode"
            | "the discard mode"
            | "discard mode tonight"
            | "tonight discard mode"
            | "mode discard"
            | "the mode discard"
            | "mode discard gap"
            | "the mode discard gap"
            | "mode gap between discards"
            | "modal discard gap"
            | "view discard mode"
            | "see discard mode"
            | "show the discard mode"
            | "show me the discard mode"
            | "open discard mode"
            | "open the discard mode"
            | "list discard mode"
            | "list the discard mode"
            | "what is the discard mode"
            | "whats the discard mode"
            | "what's the discard mode"
    )
}



/// Zero-LLM modal (most frequent minute-bucketed) keep/discard gap from results.tsv (tonight + all-time; no row dump).
pub fn format_results_tsv_mode_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Keep mode",
        ResultsTsvLastWant::Discard => "Discard mode",
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_mode(want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((mode_night, gaps_night, hits_night, mode_all, gaps_all, hits_all)) => {
            let night_part = match mode_night {
                Some(secs) => {
                    let d = duration_label_secs(secs);
                    format!(
                        "tonight mode **{d}** (×{hits_night} of {gaps_night} gaps between {unit} since 20:00)"
                    )
                }
                None => format!("tonight need ≥2 {unit} since 20:00 to measure mode"),
            };
            let all_part = match mode_all {
                Some(secs) => {
                    let d = duration_label_secs(secs);
                    format!("all-time mode **{d}** (×{hits_all} of {gaps_all} gaps)")
                }
                None => format!("all-time need ≥2 {unit}"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · mode only · does not dump rows · `/keep-pace` for average gap · `/keep-median` for median gap · `/keep-range` for min–max gap · `/keep-p90` for p90 gap · `/keep-p95` for p95 gap · `/keep-p10` for p10 gap · `/keep-p25` for p25 gap · `/keep-iqr` for IQR · `/keep-std` for std · `/keep-mad` for MAD · `/keep-cv` for CV · `/keep-skew` for skew · `/keep-kurtosis` for kurtosis · `/keep-entropy` for entropy · `/keep-mode` for mode · `/keep-gini` for Gini · `/keep-histogram` for histogram · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// Gini coefficient (×1000) of gap seconds between consecutive matching keep/discard rows.
/// Returns `(gini_x1000_night, gaps_night, gini_x1000_all, gaps_all)`.
/// `gini_x1000` is thousandths (350 = 0.350); None when <2 gaps. 0 = equal gaps, 1000 ≈ max inequality.
pub(super) fn count_results_tsv_gini(
    want: ResultsTsvLastWant,
) -> Result<(Option<i64>, u64, Option<i64>, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    fn gini_gap_secs(ts: &[chrono::DateTime<chrono::Utc>]) -> (Option<i64>, u64) {
        // Need ≥2 gaps (≥3 rows) for a meaningful inequality measure.
        if ts.len() < 3 {
            let gaps = ts.len().saturating_sub(1) as u64;
            return (None, gaps);
        }
        let mut gaps: Vec<f64> = Vec::with_capacity(ts.len() - 1);
        for w in ts.windows(2) {
            let secs = (w[1] - w[0]).num_seconds().max(0) as f64;
            gaps.push(secs);
        }
        let n = gaps.len() as f64;
        if n < 2.0 {
            return (None, gaps.len() as u64);
        }
        gaps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let sum: f64 = gaps.iter().sum();
        if sum <= 0.0 {
            return (Some(0), gaps.len() as u64);
        }
        let mut weighted = 0.0_f64;
        for (i, g) in gaps.iter().enumerate() {
            weighted += (i as f64 + 1.0) * g;
        }
        let gini = ((2.0 * weighted) / (n * sum) - (n + 1.0) / n).clamp(0.0, 1.0);
        let gini_x1000 = (gini * 1000.0).round() as i64;
        (Some(gini_x1000), gaps.len() as u64)
    }
    let (gini_all, gaps_all) = gini_gap_secs(&all_ts);
    let night_ts: Vec<_> = all_ts
        .into_iter()
        .filter(|t| *t >= window_start)
        .collect();
    let (gini_night, gaps_night) = gini_gap_secs(&night_ts);
    Ok((gini_night, gaps_night, gini_all, gaps_all))
}


/// True for short keep-gini asks (`/keep-gini`, `keep gini`, `gini keep gap`, `gini coefficient keep gap`…).
/// Gini only — does not dump TSV or steal mode / entropy / kurtosis / skew / cv / mad / std / iqr / p90 / range / median / pace / first / last / since / counts / rate / streak / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_gini_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-gini"
            | "/keepgini"
            | "/discard-gini"
            | "/discardgini"
            | "/gini-keep-gap"
            | "/ginikeepgap"
            | "/gini-discard-gap"
            | "/ginidiscardgap"
            | "/gini-gap"
            | "/ginigap"
            | "/gini-keep"
            | "/ginikeep"
            | "/gini-discard"
            | "/ginidiscard"
            | "/gini-coefficient"
            | "/ginicoefficient"
            | "/keep-gini-gap"
            | "/keepginigap"
            | "/gini-coefficient-keep-gap"
            | "/ginicoefficientkeepgap"
            | "keep gini"
            | "the keep gini"
            | "keep gini tonight"
            | "tonight keep gini"
            | "gini keep"
            | "the gini keep"
            | "gini keep gap"
            | "the gini keep gap"
            | "gini gap between keeps"
            | "gini time between keeps"
            | "gini coefficient keep gap"
            | "the gini coefficient keep gap"
            | "keep gap gini"
            | "keep gap gini coefficient"
            | "ratchet gini"
            | "the ratchet gini"
            | "overnight keep gini"
            | "view keep gini"
            | "see keep gini"
            | "show the keep gini"
            | "show me the keep gini"
            | "open keep gini"
            | "open the keep gini"
            | "list keep gini"
            | "list the keep gini"
            | "what is the keep gini"
            | "whats the keep gini"
            | "what's the keep gini"
            | "what is the gini keep gap"
            | "whats the gini keep gap"
            | "what's the gini keep gap"
            | "discard gini"
            | "the discard gini"
            | "discard gini tonight"
            | "tonight discard gini"
            | "gini discard"
            | "the gini discard"
            | "gini discard gap"
            | "the gini discard gap"
            | "gini gap between discards"
            | "gini coefficient discard gap"
            | "view discard gini"
            | "see discard gini"
            | "show the discard gini"
            | "show me the discard gini"
            | "open discard gini"
            | "open the discard gini"
            | "list discard gini"
            | "list the discard gini"
            | "what is the discard gini"
            | "whats the discard gini"
            | "what's the discard gini"
            | "gini"
            | "the gini"
    )
}


/// Zero-LLM Gini coefficient of keep/discard gaps from results.tsv (tonight + all-time; no row dump).
pub fn format_results_tsv_gini_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Keep gini",
        ResultsTsvLastWant::Discard => "Discard gini",
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_gini(want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((gini_night, gaps_night, gini_all, gaps_all)) => {
            let night_part = match gini_night {
                Some(g) => {
                    let fg = format_gini_x1000(g);
                    format!(
                        "tonight Gini **{fg}** ({gaps_night} gaps between {unit} since 20:00)"
                    )
                }
                None => format!("tonight need ≥3 {unit} (2 gaps) since 20:00 to measure Gini"),
            };
            let all_part = match gini_all {
                Some(g) => {
                    let fg = format_gini_x1000(g);
                    format!("all-time Gini **{fg}** ({gaps_all} gaps)")
                }
                None => format!("all-time need ≥3 {unit} (2 gaps)"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · Gini only · does not dump rows · `/keep-pace` for average gap · `/keep-median` for median gap · `/keep-range` for min–max gap · `/keep-p90` for p90 gap · `/keep-p95` for p95 gap · `/keep-p10` for p10 gap · `/keep-p25` for p25 gap · `/keep-iqr` for IQR · `/keep-std` for std · `/keep-mad` for MAD · `/keep-cv` for CV · `/keep-skew` for skew · `/keep-kurtosis` for kurtosis · `/keep-entropy` for entropy · `/keep-mode` for mode · `/keep-gini` for Gini · `/keep-histogram` for histogram · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}


/// Fixed minute buckets for keep-gap histogram (compact operator glance).
pub(super) fn keep_gap_hist_bucket(mins: i64) -> &'static str {
    if mins < 5 {
        "<5m"
    } else if mins < 10 {
        "5–10m"
    } else if mins < 20 {
        "10–20m"
    } else if mins < 30 {
        "20–30m"
    } else if mins < 60 {
        "30–60m"
    } else {
        "≥60m"
    }
}


/// Minute-bucket histogram of gaps between consecutive matching keep/discard rows.
/// Returns `(hist_night, gaps_night, hist_all, gaps_all)`.
pub(super) fn count_results_tsv_histogram(
    want: ResultsTsvLastWant,
) -> Result<(Option<String>, u64, Option<String>, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    fn hist_gap_secs(ts: &[chrono::DateTime<chrono::Utc>]) -> (Option<String>, u64) {
        if ts.len() < 2 {
            return (None, 0);
        }
        let mut counts: std::collections::HashMap<&'static str, u64> =
            std::collections::HashMap::new();
        for w in ts.windows(2) {
            let secs = (w[1] - w[0]).num_seconds().max(0);
            let mins = secs / 60;
            let b = keep_gap_hist_bucket(mins);
            *counts.entry(b).or_insert(0) += 1;
        }
        let gaps = ts.len().saturating_sub(1) as u64;
        let mut parts: Vec<String> = Vec::new();
        for &label in KEEP_GAP_HIST_ORDER {
            if let Some(&c) = counts.get(label) {
                if c > 0 {
                    parts.push(format!("{label}×{c}"));
                }
            }
        }
        if parts.is_empty() {
            (None, gaps)
        } else {
            (Some(parts.join(" · ")), gaps)
        }
    }
    let night_ts: Vec<_> = all_ts
        .iter()
        .copied()
        .filter(|t| *t >= window_start)
        .collect();
    let (hist_night, gaps_night) = hist_gap_secs(&night_ts);
    let (hist_all, gaps_all) = hist_gap_secs(&all_ts);
    Ok((hist_night, gaps_night, hist_all, gaps_all))
}



/// True for short keep-histogram asks (`/keep-histogram`, `keep histogram`, `gap histogram`…).
/// Histogram only — does not dump TSV or steal gini / mode / entropy / other gap stats / path/size/age / morning surprise.
pub fn looks_like_results_tsv_histogram_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-histogram"
            | "/keephistogram"
            | "/discard-histogram"
            | "/discardhistogram"
            | "/keep-hist"
            | "/keephist"
            | "/discard-hist"
            | "/discardhist"
            | "/histogram"
            | "/hist"
            | "/keep-gap-histogram"
            | "/keepgaphistogram"
            | "/gap-histogram"
            | "/gaphistogram"
            | "/keep-histogram-gap"
            | "/keephistogramgap"
            | "keep histogram"
            | "the keep histogram"
            | "keep histogram tonight"
            | "tonight keep histogram"
            | "gap histogram"
            | "the gap histogram"
            | "keep gap histogram"
            | "the keep gap histogram"
            | "histogram keep gap"
            | "histogram gap between keeps"
            | "hist keep gap"
            | "keep hist"
            | "the keep hist"
            | "hist gap between keeps"
            | "ratchet histogram"
            | "the ratchet histogram"
            | "overnight keep histogram"
            | "view keep histogram"
            | "see keep histogram"
            | "show the keep histogram"
            | "show me the keep histogram"
            | "open keep histogram"
            | "open the keep histogram"
            | "list keep histogram"
            | "list the keep histogram"
            | "what is the keep histogram"
            | "whats the keep histogram"
            | "what's the keep histogram"
            | "view keep hist"
            | "see keep hist"
            | "show me the keep hist"
            | "open keep hist"
            | "open the keep hist"
            | "discard histogram"
            | "the discard histogram"
            | "discard histogram tonight"
            | "tonight discard histogram"
            | "discard gap histogram"
            | "histogram discard gap"
            | "discard hist"
            | "the discard hist"
            | "view discard histogram"
            | "see discard histogram"
            | "show the discard histogram"
            | "show me the discard histogram"
            | "open discard histogram"
            | "open the discard histogram"
            | "list discard histogram"
            | "list the discard histogram"
            | "what is the discard histogram"
            | "whats the discard histogram"
            | "what's the discard histogram"
            | "histogram"
            | "the histogram"
            | "hist"
            | "the hist"
    )
}


/// Zero-LLM minute-bucket histogram of keep/discard gaps from results.tsv (tonight + all-time; no row dump).
pub fn format_results_tsv_histogram_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Keep histogram",
        ResultsTsvLastWant::Discard => "Discard histogram",
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_histogram(want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((hist_night, gaps_night, hist_all, gaps_all)) => {
            let night_part = match hist_night {
                Some(ref h) => {
                    format!(
                        "tonight **{h}** ({gaps_night} gaps between {unit} since 20:00)"
                    )
                }
                None => format!("tonight need ≥2 {unit} since 20:00 to measure histogram"),
            };
            let all_part = match hist_all {
                Some(ref h) => format!("all-time **{h}** ({gaps_all} gaps)"),
                None => format!("all-time need ≥2 {unit}"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · histogram only · does not dump rows · `/keep-pace` for average gap · `/keep-median` for median gap · `/keep-range` for min–max gap · `/keep-p90` for p90 gap · `/keep-p95` for p95 gap · `/keep-p10` for p10 gap · `/keep-p25` for p25 gap · `/keep-iqr` for IQR · `/keep-std` for std · `/keep-mad` for MAD · `/keep-cv` for CV · `/keep-skew` for skew · `/keep-kurtosis` for kurtosis · `/keep-entropy` for entropy · `/keep-mode` for mode · `/keep-gini` for Gini · `/keep-histogram` for histogram · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}








/// Shannon entropy (bits) of minute-bucketed gaps between consecutive matching keep/discard rows.
/// Returns `(entropy_x100_night, gaps_night, buckets_night, entropy_x100_all, gaps_all, buckets_all)`.
/// `entropy_x100` is rounded hundredths of bits (185 = 1.85 bits); None when <2 gaps.
pub(super) fn count_results_tsv_entropy(
    want: ResultsTsvLastWant,
) -> Result<(Option<i64>, u64, u64, Option<i64>, u64, u64), String> {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return Err("missing".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let window_start = overnight_window_start_local().with_timezone(&chrono::Utc);
    let want_keep = matches!(want, ResultsTsvLastWant::Keep);
    let mut all_ts: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let Some(ts) = cols.next() else {
            continue;
        };
        let _sha = cols.next();
        let Some(outcome) = cols.next() else {
            continue;
        };
        let outcome = outcome.trim().to_ascii_lowercase();
        let is_keep = outcome == "keep";
        let is_discard = outcome == "discard";
        if want_keep && !is_keep {
            continue;
        }
        if !want_keep && !is_discard {
            continue;
        }
        let Some(parsed) = parse_run_ts(ts) else {
            continue;
        };
        all_ts.push(parsed);
    }
    fn entropy_gap_secs(ts: &[chrono::DateTime<chrono::Utc>]) -> (Option<i64>, u64, u64) {
        // Need ≥2 gaps (≥3 rows) for Shannon entropy of gap buckets.
        if ts.len() < 3 {
            let gaps = ts.len().saturating_sub(1) as u64;
            return (None, gaps, 0);
        }
        let mut gaps: Vec<i64> = Vec::with_capacity(ts.len() - 1);
        for w in ts.windows(2) {
            // Minute buckets keep second-level noise from inflating entropy.
            let secs = (w[1] - w[0]).num_seconds().max(0);
            gaps.push(secs / 60);
        }
        let n = gaps.len() as f64;
        if n < 2.0 {
            return (None, gaps.len() as u64, 0);
        }
        let mut counts: std::collections::BTreeMap<i64, u64> = std::collections::BTreeMap::new();
        for g in &gaps {
            *counts.entry(*g).or_insert(0) += 1;
        }
        let buckets = counts.len() as u64;
        let mut h = 0.0_f64;
        for c in counts.values() {
            let p = (*c as f64) / n;
            if p > 0.0 {
                h -= p * p.log2();
            }
        }
        let entropy_x100 = (h * 100.0).round() as i64;
        (Some(entropy_x100), gaps.len() as u64, buckets)
    }
    let (ent_all, gaps_all, buckets_all) = entropy_gap_secs(&all_ts);
    let night_ts: Vec<_> = all_ts
        .into_iter()
        .filter(|t| *t >= window_start)
        .collect();
    let (ent_night, gaps_night, buckets_night) = entropy_gap_secs(&night_ts);
    Ok((
        ent_night,
        gaps_night,
        buckets_night,
        ent_all,
        gaps_all,
        buckets_all,
    ))
}


/// True for short keep-entropy asks (`/keep-entropy`, `keep entropy`, `entropy keep gap`…).
/// Entropy only — does not dump TSV or steal kurtosis / skew / cv / mad / std / iqr / p90 / range / median / pace / first / last / since / counts / rate / streak / recent / path/size/age / morning surprise.
pub fn looks_like_results_tsv_entropy_request(content: &str) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return false;
    }
    if keep_gap_stat_excluded(&n) {
        return false;
    }
    matches!(
        n.as_str(),
        "/keep-entropy"
            | "/keepentropy"
            | "/discard-entropy"
            | "/discardentropy"
            | "/entropy-keep-gap"
            | "/entropykeepgap"
            | "/entropy-discard-gap"
            | "/entropydiscardgap"
            | "/entropy-gap"
            | "/entropygap"
            | "/entropy-keep"
            | "/entropykeep"
            | "/entropy-discard"
            | "/entropydiscard"
            | "/shannon-entropy"
            | "/shannonentropy"
            | "/keep-shannon-entropy"
            | "/keepshannonentropy"
            | "keep entropy"
            | "the keep entropy"
            | "keep entropy tonight"
            | "tonight keep entropy"
            | "entropy keep"
            | "the entropy keep"
            | "entropy keep gap"
            | "the entropy keep gap"
            | "entropy gap between keeps"
            | "entropy time between keeps"
            | "shannon entropy keep gap"
            | "the shannon entropy keep gap"
            | "keep gap entropy"
            | "keep gap shannon entropy"
            | "ratchet entropy"
            | "the ratchet entropy"
            | "overnight keep entropy"
            | "view keep entropy"
            | "see keep entropy"
            | "show the keep entropy"
            | "show me the keep entropy"
            | "open keep entropy"
            | "open the keep entropy"
            | "list keep entropy"
            | "list the keep entropy"
            | "what is the keep entropy"
            | "whats the keep entropy"
            | "what's the keep entropy"
            | "what is the entropy keep gap"
            | "whats the entropy keep gap"
            | "what's the entropy keep gap"
            | "discard entropy"
            | "the discard entropy"
            | "discard entropy tonight"
            | "tonight discard entropy"
            | "entropy discard"
            | "the entropy discard"
            | "entropy discard gap"
            | "the entropy discard gap"
            | "entropy gap between discards"
            | "shannon entropy discard gap"
            | "view discard entropy"
            | "see discard entropy"
            | "show the discard entropy"
            | "show me the discard entropy"
            | "open discard entropy"
            | "open the discard entropy"
            | "list discard entropy"
            | "list the discard entropy"
            | "what is the discard entropy"
            | "whats the discard entropy"
            | "what's the discard entropy"
    )
}


/// Zero-LLM Shannon entropy (bits) of minute-bucketed keep/discard gaps from results.tsv (tonight + all-time; no row dump).
pub fn format_results_tsv_entropy_gateway(content: &str) -> String {
    let want = results_tsv_last_want(content);
    let label = match want {
        ResultsTsvLastWant::Keep => "Keep entropy",
        ResultsTsvLastWant::Discard => "Discard entropy",
    };
    let unit = match want {
        ResultsTsvLastWant::Keep => "keeps",
        ResultsTsvLastWant::Discard => "discards",
    };
    match count_results_tsv_entropy(want) {
        Err(_) => format!(
            "**{label}:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file."
        ),
        Ok((ent_night, gaps_night, buckets_night, ent_all, gaps_all, buckets_all)) => {
            let night_part = match ent_night {
                Some(e) => {
                    let fe = format_entropy_x100(e);
                    format!(
                        "tonight entropy **{fe} bits** ({gaps_night} gaps · {buckets_night} minute-buckets between {unit} since 20:00)"
                    )
                }
                None => format!("tonight need ≥3 {unit} (2 gaps) since 20:00 to measure entropy"),
            };
            let all_part = match ent_all {
                Some(e) => {
                    let fe = format_entropy_x100(e);
                    format!(
                        "all-time entropy **{fe} bits** ({gaps_all} gaps · {buckets_all} minute-buckets)"
                    )
                }
                None => format!("all-time need ≥3 {unit} (2 gaps)"),
            };
            format!(
                "**{label}:** {night_part} · {all_part} · entropy only · does not dump rows · `/keep-pace` for average gap · `/keep-median` for median gap · `/keep-range` for min–max gap · `/keep-p90` for p90 gap · `/keep-p95` for p95 gap · `/keep-p10` for p10 gap · `/keep-p25` for p25 gap · `/keep-iqr` for IQR · `/keep-std` for std · `/keep-mad` for MAD · `/keep-cv` for CV · `/keep-skew` for skew · `/keep-kurtosis` for kurtosis · `/keep-entropy` for entropy · `/keep-mode` for mode · `/keep-gini` for Gini · `/keep-histogram` for histogram · `/keeps` for counts · `/keep-rate` for hit rate · `/since-keep` for age since newest · `/first-keep` for tonight's first · `/last-keep` for the newest row · `/recent-keeps` for a short tonight list · `results.tsv path` for the file · ask *morning surprise?* for ship notes."
            )
        }
    }
}

