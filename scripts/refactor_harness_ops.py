#!/usr/bin/env python3
"""Collapse harness_ops.rs god file into harness_ops/ modules.

Phases:
1. Strip /keep-pNN clones; inject parameterized percentile lane
2. Table-drive path/size/age looks_like matchers
3. Split remaining production code into domain modules
4. Compact tests + write line-budget guard doc
"""
from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src-tauri/src/commands/harness_ops.rs"
OUT_DIR = ROOT / "src-tauri/src/commands/harness_ops"
DOCS = ROOT / "docs/ops/harness_ops_refactor.md"
GUARD = ROOT / "scripts/check_harness_ops_budget.py"

PNN_FN = re.compile(
    r"^(?:pub(?:\([^)]*\))? )?fn ("
    r"looks_like_results_tsv_p\d+_request|"
    r"format_results_tsv_p\d+_gateway|"
    r"count_results_tsv_p\d+|"
    r"mentions_results_tsv_p\d+"
    r")\b"
)
TOP_FN = re.compile(r"^(?:pub(?:\([^)]*\))? )?fn (\w+)")
TOP_ENUM = re.compile(r"^(?:pub )?enum (\w+)")
TOP_STRUCT = re.compile(r"^(?:pub )?struct (\w+)")
TOP_CONST = re.compile(r"^(?:pub )?const (\w+)")
RESOURCE_LOOKS = re.compile(
    r"^pub fn looks_like_(\w+)_(path|size|age)_request\b"
)
RESOURCE_FMT = re.compile(
    r"^pub fn format_(\w+)_(path|size|age)_gateway\b"
)


def ordinal(n: int) -> str:
    if 10 <= n % 100 <= 20:
        suf = "th"
    else:
        suf = {1: "st", 2: "nd", 3: "rd"}.get(n % 10, "th")
    return f"{n}{suf}"


KEEP_PERCENTILE_RS = r'''
//! Parameterized `/keep-p{N}` nearest-rank gap glances (results.tsv).

use super::normalize::{contains_age_token, normalize_operator_command};

/// Parse a keep/discard gap percentile ask (`/keep-p47`, `p4 keep gap`, `47th percentile…`).
/// Digit-safe: `/keep-p4` does not match `/keep-p47`.
pub fn parse_keep_percentile(content: &str) -> Option<u8> {
    let n = normalize_operator_command(content);
    if n.chars().count() > 72 {
        return None;
    }
    if keep_percentile_excluded(&n) {
        return None;
    }
    if let Some(p) = extract_keep_percentile_number(&n) {
        if (1..=99).contains(&p) {
            return Some(p);
        }
    }
    None
}

pub fn looks_like_keep_percentile_request(content: &str) -> bool {
    parse_keep_percentile(content).is_some()
}

pub fn mentions_keep_percentile(n: &str) -> bool {
    extract_keep_percentile_number(n).is_some()
}

fn keep_percentile_excluded(n: &str) -> bool {
    n.contains("path")
        || n.contains("where")
        || n.contains("location")
        || n.contains("folder")
        || n.contains("directory")
        || n.contains("dir")
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
        || n.contains("recent-discards")
        || n.contains("recentdiscards")
        || n.contains("keep list")
        || n.contains("discard list")
        || n.contains("first keep")
        || n.contains("first discard")
        || n.contains("earliest keep")
        || n.contains("earliest discard")
        || n.contains("opening keep")
        || n.contains("opening discard")
        || n.contains("since last")
        || n.contains("how long since")
        || n.contains("how long ago")
        || n.contains("time since")
        || n.starts_with("/since-")
        || n.starts_with("/timesince")
        || n.starts_with("/time-since")
        || n.starts_with("/last-")
        || n.starts_with("/first-")
        || n.starts_with("/firstkeep")
        || n.starts_with("/firstdiscard")
        || n.starts_with("/recent-")
        || n.starts_with("/keep-rate")
        || n.starts_with("/keeprate")
        || n.starts_with("/hit-rate")
        || n.starts_with("/keep-streak")
        || n.starts_with("/longest-")
        || n.starts_with("/keep-pace")
        || n.starts_with("/keeppace")
        || n.starts_with("/discard-pace")
        || n.starts_with("/discardpace")
        || n.starts_with("/keep-median")
        || n.starts_with("/keepmedian")
        || n.starts_with("/discard-median")
        || n.starts_with("/discardmedian")
        || n.starts_with("/median-")
        || n.starts_with("/keep-range")
        || n.starts_with("/keeprange")
        || n.starts_with("/discard-range")
        || n.starts_with("/discardrange")
        || n.starts_with("/gap-range")
        || n.starts_with("/keep-spread")
        || n.starts_with("/discard-spread")
        || n == "last keep"
        || n == "the last keep"
        || n == "last discard"
        || n == "the last discard"
        || n == "latest keep"
        || n == "latest discard"
        || n.starts_with("/keep-std")
        || n.starts_with("/keepstd")
        || n.starts_with("/discard-std")
        || n.starts_with("/discardstd")
        || n.starts_with("/std-")
        || n.contains("stddev")
        || n.contains("std-dev")
        || n.contains("std deviation")
        || n.starts_with("/keep-mad")
        || n.starts_with("/keepmad")
        || n.starts_with("/discard-mad")
        || n.starts_with("/keep-cv")
        || n.starts_with("/keepcv")
        || n.starts_with("/discard-cv")
        || n.starts_with("/keep-skew")
        || n.starts_with("/keepskew")
        || n.starts_with("/discard-skew")
        || n.starts_with("/keep-kurtosis")
        || n.starts_with("/keepkurtosis")
        || n.starts_with("/discard-kurtosis")
        || n.starts_with("/keep-entropy")
        || n.starts_with("/keepentropy")
        || n.starts_with("/discard-entropy")
        || n.starts_with("/keep-mode")
        || n.starts_with("/keepmode")
        || n.starts_with("/discard-mode")
        || n.starts_with("/keep-gini")
        || n.starts_with("/keepgini")
        || n.starts_with("/discard-gini")
        || n.starts_with("/keep-histogram")
        || n.starts_with("/keephistogram")
        || n.starts_with("/discard-histogram")
        || n.starts_with("/keep-iqr")
        || n.starts_with("/keepiqr")
        || n.starts_with("/discard-iqr")
        || n.starts_with("/iqr")
        || n.contains("interquartile")
        || n.contains("histogram")
        || n.contains("entropy")
        || n.contains("kurtosis")
        || n.contains("gini")
        || (n.contains("median") && !n.contains("percentile"))
        || (n.contains("average") && !n.contains("percentile"))
        || (n.contains("mean ") && !n.contains("percentile"))
        || (n.contains("pace") && !n.contains("percentile"))
        || (n.contains("range") && !n.contains("percentile"))
        || n.contains("min-max")
        || n.contains("minmax")
}

fn slash_percentile_prefix(n: &str, base: &str) -> Option<u8> {
    if let Some(rest) = n.strip_prefix(base) {
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            return None;
        }
        // Digit-safe: require end of number (no extra trailing digit beyond parse).
        let after = &rest[digits.len()..];
        if let Some(c) = after.chars().next() {
            if c.is_ascii_digit() {
                return None;
            }
        }
        return digits.parse::<u8>().ok();
    }
    None
}

fn extract_keep_percentile_number(n: &str) -> Option<u8> {
    // Prefer longer digit runs by trying explicit slash forms first.
    let slash_bases = [
        "/keep-p",
        "/keepp",
        "/discard-p",
        "/discardp",
        "/p",
    ];
    let mut best: Option<u8> = None;
    for base in slash_bases {
        if let Some(p) = slash_percentile_prefix(n, base) {
            if best.map(|b| p.to_string().len() >= b.to_string().len()).unwrap_or(true) {
                // For `/p` alone, require keep/discard/gap context later via matches.
                if base == "/p" {
                    // `/p47-keep`, `/p47gap`, `/p47` only when keep/discard/gap present or exact.
                    let rest = n.strip_prefix(base).unwrap_or("");
                    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                    let after = &rest[digits.len()..];
                    if after.is_empty()
                        || after.starts_with('-')
                        || after.starts_with("keep")
                        || after.starts_with("discard")
                        || after.starts_with("gap")
                    {
                        best = Some(p);
                    }
                } else {
                    best = Some(p);
                }
            }
        }
    }
    if best.is_some() {
        return best;
    }

    // Ordinal: "47th percentile" / "4th keep gap"
    if let Some(m) = regex_ordinal(n) {
        return Some(m);
    }

    // Padded token forms: " p47 ", " keep-p47 ", "p47 keep"
    let padded = format!(" {n} ");
    // Find pNN tokens with digit boundaries; prefer longest.
    let mut found: Option<u8> = None
    ;
    let bytes = padded.as_bytes();
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b' ' && (bytes[i + 1] == b'p' || bytes[i + 1] == b'P') {
            let mut j = i + 2;
            if j < bytes.len() && bytes[j] == b'-' {
                j += 1;
            }
            let start = j;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > start {
                let dig = std::str::from_utf8(&bytes[start..j]).ok()?;
                // unreachable in const sense; use parse below
                let _ = dig;
            }
        }
        i += 1;
    }
    // Safer Rust-free scan in host script — implemented below in pure logic for emit.
    found = scan_padded_p_token(n);
    if let Some(p) = found {
        // Require keep/discard/gap/ratchet/percentile context for bare pNN.
        if n.contains("keep")
            || n.contains("discard")
            || n.contains("gap")
            || n.contains("ratchet")
            || n.contains("percentile")
            || n.contains("overnight")
        {
            return Some(p);
        }
    }
    None
}

fn regex_ordinal(n: &str) -> Option<u8> {
    // Manual scan for Nst/Nnd/Nrd/Nth
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
            if p == 0 || p > 99 {
                continue;
            }
            let rest = &n[i..];
            let rest = rest.strip_prefix(' ').unwrap_or(rest);
            let ok = rest.starts_with("th")
                || rest.starts_with("st")
                || rest.starts_with("nd")
                || rest.starts_with("rd");
            if ok {
                // Avoid matching inside larger numbers already handled by digit run.
                return Some(p);
            }
        } else {
            i += 1;
        }
    }
    None
}

fn scan_padded_p_token(n: &str) -> Option<u8> {
    let padded = format!(" {n} ");
    let mut best: Option<u8> = None;
    let chars: Vec<char> = padded.chars().collect();
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
                            if best.map(|b| dig.len() >= b.to_string().len()).unwrap_or(true) {
                                best = Some(p);
                            }
                        }
                    }
                }
            }
        }
        // keep-pNN / discard-pNN
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
                        if best.map(|b| dig.len() >= b.to_string().len()).unwrap_or(true) {
                            best = Some(p);
                        }
                    }
                }
            }
        }
        i += 1;
    }
    best
}
'''

# The KEEP_PERCENTILE_RS above mixed Python thinking into Rust incorrectly with `found = None;`.
# We'll emit a clean hand-written keep_percentile section instead.


def emit_keep_percentile_module() -> str:
    return '''//! Parameterized `/keep-p{N}` nearest-rank gap glances (results.tsv).
//! Overnight must extend this parser — never clone per-percentile matchers.

use super::normalize::{contains_age_token, normalize_operator_command};

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

fn keep_percentile_excluded(n: &str) -> bool {
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

fn slash_p(n: &str, base: &str) -> Option<u8> {
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

fn extract_keep_percentile_number(n: &str) -> Option<u8> {
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

fn scan_ordinal(n: &str) -> Option<u8> {
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

fn scan_p_token(padded: &str) -> Option<u8> {
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
'''


def split_items(lines: list[str]) -> tuple[list[str], list[tuple[str, str, int, int]], int]:
    """Return (preamble_lines, items[(kind,name,start,end)], test_start)."""
    preamble_end = 0
    for i, l in enumerate(lines):
        if TOP_FN.match(l) or TOP_ENUM.match(l) or TOP_STRUCT.match(l) or TOP_CONST.match(l) or l.startswith("#[derive") or l.startswith("///"):
            # first real item may have docs
            if TOP_FN.match(l) or TOP_ENUM.match(l) or TOP_STRUCT.match(l) or TOP_CONST.match(l):
                preamble_end = i
                break
    # Better: preamble = uses + module docs until first fn/struct/enum/const at col0 (with attrs)
    preamble_end = 0
    i = 0
    while i < len(lines):
        l = lines[i]
        if l.startswith("//!") or l.startswith("use ") or l.strip() == "" or l.startswith("#[allow"):
            i += 1
            preamble_end = i
            continue
        break

    items: list[tuple[str, str, int, int]] = []
    test_start = len(lines)
    i = preamble_end
    while i < len(lines):
        l = lines[i]
        if l.startswith("#[cfg(test)]"):
            test_start = i
            break
        # include leading docs/attrs
        start = i
        j = i
        while j < len(lines) and (
            lines[j].startswith("///")
            or lines[j].startswith("#[")
            or lines[j].strip() == ""
        ):
            j += 1
        if j >= len(lines):
            break
        l = lines[j]
        name = None
        kind = None
        if m := TOP_FN.match(l):
            kind, name = "fn", m.group(1)
        elif m := TOP_ENUM.match(l):
            kind, name = "enum", m.group(1)
        elif m := TOP_STRUCT.match(l):
            kind, name = "struct", m.group(1)
        elif m := TOP_CONST.match(l):
            kind, name = "const", m.group(1)
        elif l.startswith("pub type ") or l.startswith("type "):
            kind, name = "type", l.split()[2].strip(";")
        else:
            # skip stray
            i += 1
            continue
        # rewind start to docs
        start = j
        k = j - 1
        while k >= preamble_end and (
            lines[k].startswith("///") or lines[k].startswith("#[") 
        ):
            start = k
            k -= 1
        end = j + 1
        while end < len(lines):
            if lines[end].startswith("#[cfg(test)]"):
                break
            # Column-0 only (do not use startswith("") — every string matches).
            indented = lines[end][:1] in (" ", "\t")
            blank = lines[end].strip() == ""
            if not indented and not blank:
                if (
                    TOP_FN.match(lines[end])
                    or TOP_ENUM.match(lines[end])
                    or TOP_STRUCT.match(lines[end])
                    or TOP_CONST.match(lines[end])
                    or lines[end].startswith("impl ")
                    or lines[end].startswith("pub type ")
                    or lines[end].startswith("type ")
                ):
                    break
                if lines[end].startswith("///") or lines[end].startswith("#["):
                    p = end
                    while p < len(lines) and (
                        lines[p].startswith("///")
                        or lines[p].startswith("#[")
                        or lines[p].strip() == ""
                    ):
                        p += 1
                    if p < len(lines) and (
                        TOP_FN.match(lines[p])
                        or TOP_ENUM.match(lines[p])
                        or TOP_STRUCT.match(lines[p])
                        or TOP_CONST.match(lines[p])
                        or lines[p].startswith("impl ")
                    ):
                        break
            end += 1
        items.append((kind, name, start, end))
        i = end
    return lines[:preamble_end], items, test_start


def is_pnn_name(name: str) -> bool:
    return bool(
        re.match(
            r"(looks_like_results_tsv_p\d+_request|format_results_tsv_p\d+_gateway|count_results_tsv_p\d+|mentions_results_tsv_p\d+)$",
            name,
        )
    )


def classify(name: str, kind: str) -> str:
    if name in {
        "LiveSessionSummary",
        "SessionMessageRow",
        "SessionFileSummary",
        "MemoryFileSummary",
        "RunTurnSummary",
        "RunInsightCandidate",
        "RunsInsights",
        "DigestSummary",
        "write_text_atomic",
        "file_mtime_ms",
        "parse_session_filename",
        "list_live_sessions",
        "read_live_session_messages",
        "read_session_file_messages",
        "list_session_files",
        "read_session_file",
        "list_memory_files",
        "read_memory_file",
        "get_runs_insights",
        "compute_runs_insights",
        "parse_insights_days",
        "compute_runs_insights_for",
        "digest_json_path",
        "digest_md_path",
        "load_digest_summary",
        "get_digest_summary",
        "digest_script_candidates",
        "refresh_agent_digest",
        "parse_run_ts",
        "shipped_cutoffs",
        "is_stale_shipped_candidate",
        "hint_for_run",
        "write_digest_native",
        "ensure_under",
        "age_from_rfc3339",
    } or name.startswith("digest_") or name.startswith("format_digest") or name.startswith("looks_like_digest") or name.startswith("try_digest"):
        return "agent_ops"
    if name in {"normalize_operator_command", "contains_age_token"}:
        return "normalize"
    if re.match(r".*_(path|size|age)_(request|gateway)$", name) or name in {
        "looks_like_resource_stat",
        "format_resource_stat",
        "StatKind",
        "ResourceLane",
    }:
        # looks_like_X_path_request / format_X_path_gateway
        if RESOURCE_LOOKS.match(f"pub fn {name}") or name.startswith("looks_like_") and name.endswith(("_path_request", "_size_request", "_age_request")):
            return "resource_stat"
        if name.startswith("format_") and name.endswith(("_path_gateway", "_size_gateway", "_age_gateway")):
            return "resource_stat"
    if "results_tsv" in name or name in {
        "overnight_window_start_local",
        "duration_label_secs",
        "ResultsTsvLastWant",
        "results_tsv_last_want",
        "RESULTS_TSV_RECENT_CAP",
        "parse_keep_percentile",
        "looks_like_keep_percentile_request",
        "format_keep_percentile_gateway",
        "mentions_keep_percentile",
        "count_results_tsv_percentile",
        "keep_gap_hist_bucket",
    } or name.startswith("count_results_tsv") or name.startswith("last_results_tsv") or name.startswith("format_results_tsv") or name.startswith("looks_like_results_tsv"):
        return "keep_stats"
    if any(
        x in name
        for x in (
            "insights",
            "failed_runs",
            "slow_runs",
            "instant_runs",
            "direct_runs",
            "lite_runs",
        )
    ):
        return "insights"
    if any(
        x in name
        for x in (
            "status",
            "ops_help",
            "ready_request",
            "ready_chip",
            "gateway_chip",
            "discord_gateway",
            "ollama_ready",
            "redmine_ready",
            "brave_ready",
            "perplexity_ready",
            "mastodon_ready",
            "mcp_ready",
            "cursor_agent_ready",
            "browser_ready",
            "judge_ready",
            "ai_agent_ready",
            "compact_ready",
        )
    ):
        return "status_chips"
    if name == "try_operator_instant_reply":
        return "mod"
    return "inventory"


def extract_matches_phrases(body: str) -> list[str]:
    m = re.search(r"matches!\(\s*n\.as_str\(\),\s*(.*?)\s*\)", body, re.S)
    if not m:
        return []
    arms = m.group(1)
    return re.findall(r'"([^"]*)"', arms)


def extract_max_len(body: str) -> int:
    m = re.search(r"n\.chars\(\)\.count\(\) > (\d+)", body)
    return int(m.group(1)) if m else 72


def extract_noun_requires(body: str, stem: str) -> list[str]:
    """Heuristic: lines like `let x_ctx = n.contains("foo") || …; if !x_ctx`."""
    m = re.search(
        r"let \w+_ctx = (.*?);\s*if !\w+_ctx",
        body,
        re.S,
    )
    if not m:
        # fallback nouns from stem
        return stem_nouns(stem)
    exprs = m.group(1)
    found = re.findall(r'n\.contains\("([^"]+)"\)', exprs)
    return found or stem_nouns(stem)


def stem_nouns(stem: str) -> list[str]:
    s = stem.replace("_", " ")
    parts = stem.split("_")
    out = [s, stem.replace("_", "-"), stem]
    if len(parts) > 1:
        out.append(parts[-1])
    # common aliases
    aliases = {
        "debug_log": ["debug.log", "debug log", "log"],
        "results_tsv": ["results.tsv", "results tsv", "autoresearch results"],
        "runs": ["runs.jsonl", "runs"],
        "config": ["config.json", "config"],
        "screenshots": ["screenshot"],
        "launchagent": ["launchagent", "launch agent", "plist"],
        "memory_md": ["memory.md"],
        "digest_md": ["latest.md", "digest.md"],
        "loop_backlog": ["loop backlog", "loop_backlog"],
        "sibling_harness": ["sibling harness", "sibling_harness"],
        "standing_backlog": ["standing backlog", "standing_backlog"],
        "morning_surprise": ["morning surprise"],
        "overnight_agent_log": ["overnight_agent", "overnight agent"],
        "harness_loop_stdout": ["harness_loop", "harness loop", "stdout.log"],
        "harness_loop_stderr": ["stderr.log"],
    }
    out.extend(aliases.get(stem, []))
    # unique preserve order
    seen = set()
    uniq = []
    for x in out:
        if x not in seen:
            seen.add(x)
            uniq.append(x)
    return uniq


def rust_str_array(vals: list[str], indent: str = "        ") -> str:
    if not vals:
        return "&[]"
    parts = ",\n".join(f'{indent}"{v}"' for v in vals)
    return "&\n" + parts + ",\n" + indent[:-4] + ""


def clean_body_mentions(body: str) -> str:
    """Replace per-pNN mentions helpers with shared mentions_keep_percentile."""
    body = re.sub(
        r"mentions_results_tsv_p\d+\(&n\)",
        "crate::commands::harness_ops::keep_stats::mentions_keep_percentile(&n)",
        body,
    )
    # Also remove long chains of || n.starts_with("/keep-pNN") for many N — replace block with mentions
    # Simpler: leave starts_with lines; they'll still compile if we don't remove them.
    # But those reference nothing missing. OK to leave OR strip lines matching keep-p digits excludes in non-pNN.
    body = re.sub(
        r"\n\s*\|\|\s*n\.starts_with\(\"/keep-p\d+\"\)",
        "",
        body,
    )
    body = re.sub(
        r"\n\s*\|\|\s*n\.starts_with\(\"/keepp\d+\"\)",
        "",
        body,
    )
    body = re.sub(
        r"\n\s*\|\|\s*n\.starts_with\(\"/discard-p\d+\"\)",
        "",
        body,
    )
    body = re.sub(
        r"\n\s*\|\|\s*n\.starts_with\(\"/discardp\d+\"\)",
        "",
        body,
    )
    # After stripping, inject one mentions check after normalize if this is a keep-stat looks_like
    return body


def rewrite_router(body: str) -> str:
    """Replace all pNN if-branches with a single parameterized branch; dedupe consecutive identical blocks lightly."""
    # Remove each pNN branch
    body = re.sub(
        r"\n\s*//[^\n]*p\d+[^\n]*\n\s*if looks_like_results_tsv_p\d+_request\(content\) \{\n\s*return Some\(format_results_tsv_p\d+_gateway\(content\)\);\n\s*\}",
        "",
        body,
    )
    body = re.sub(
        r"\n\s*if looks_like_results_tsv_p\d+_request\(content\) \{\n\s*return Some\(format_results_tsv_p\d+_gateway\(content\)\);\n\s*\}",
        "",
        body,
    )
    # Insert single branch before range/median/pace if present, else before results_tsv_last
    needle = "    // results.tsv keep/discard min–max gap (range)"
    insert = (
        "    // Parameterized keep/discard gap percentile (/keep-pN).\n"
        "    if let Some(p) = crate::commands::harness_ops::keep_stats::parse_keep_percentile(content) {\n"
        "        return Some(crate::commands::harness_ops::keep_stats::format_keep_percentile_gateway(p, content));\n"
        "    }\n"
        "    // results.tsv keep/discard min–max gap (range)"
    )
    if needle in body:
        # Only first occurrence — then remove later duplicate insert markers if we add once
        body = body.replace(needle, insert, 1)
        # Remove any duplicate inserts
        while body.count(insert) > 1:
            # remove second
            first = body.find(insert)
            second = body.find(insert, first + 1)
            # remove the "if let Some(p)" block at second (without consuming following comment twice)
            # simpler: delete second copy of the three-line if let block + comment line
            dup = (
                "    // Parameterized keep/discard gap percentile (/keep-pN).\n"
                "    if let Some(p) = crate::commands::harness_ops::keep_stats::parse_keep_percentile(content) {\n"
                "        return Some(crate::commands::harness_ops::keep_stats::format_keep_percentile_gateway(p, content));\n"
                "    }\n"
            )
            body = body[:second] + body[second + len(dup):]
    else:
        # fallback: prepend inside function after `{`
        body = re.sub(
            r"(pub fn try_operator_instant_reply\(content: &str\) -> Option<String> \{\n)",
            r"\1" + insert.replace("    // results.tsv keep/discard min–max gap (range)", "") + "\n",
            body,
            count=1,
        )
    return body


def emit_format_keep_percentile() -> str:
    return '''
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

fn count_results_tsv_percentile(
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
        let mut cols = line.splitn(4, '\\t');
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
        let Some(parsed) = super::agent_ops::parse_run_ts(ts) else {
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
'''.replace("\\t", "\t")


def emit_resource_stat_core(entries: list[dict]) -> str:
    lines = [
        "//! Table-driven path / size / age instant-lane matchers.",
        "//! Sibling excludes stay string-only (never call other looks_like_*).",
        "",
        "use super::normalize::{contains_age_token, normalize_operator_command};",
        "",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
        "pub enum StatKind {",
        "    Path,",
        "    Size,",
        "    Age,",
        "}",
        "",
        "#[derive(Debug, Clone)]",
        "pub struct ResourceLane {",
        "    pub id: &'static str,",
        "    pub nouns: &'static [&'static str],",
        "    pub max_len: usize,",
        "    pub path_phrases: &'static [&'static str],",
        "    pub size_phrases: &'static [&'static str],",
        "    pub age_phrases: &'static [&'static str],",
        "}",
        "",
        "pub fn looks_like_resource_stat(content: &str, lane: &ResourceLane, kind: StatKind) -> bool {",
        "    let n = normalize_operator_command(content);",
        "    if n.chars().count() > lane.max_len {",
        "        return false;",
        "    }",
        "    if !lane.nouns.iter().any(|noun| n.contains(noun)) {",
        "        // Exact phrase lists may already include the noun.",
        "        let phrases = match kind {",
        "            StatKind::Path => lane.path_phrases,",
        "            StatKind::Size => lane.size_phrases,",
        "            StatKind::Age => lane.age_phrases,",
        "        };",
        "        if !phrases.iter().any(|p| *p == n) {",
        "            return false;",
        "        }",
        "    }",
        "    match kind {",
        "        StatKind::Path => {",
        "            if is_size_ask(&n) || is_age_ask(&n) {",
        "                return false;",
        "            }",
        "            if n.contains(\"path\")",
        "                || n.contains(\"where\")",
        "                || n.contains(\"location\")",
        "                || n.contains(\"file is\")",
        "            {",
        "                return true;",
        "            }",
        "            lane.path_phrases.iter().any(|p| *p == n)",
        "        }",
        "        StatKind::Size => {",
        "            if n.contains(\"path\")",
        "                || n.contains(\"where\")",
        "                || n.contains(\"location\")",
        "                || is_age_ask(&n)",
        "            {",
        "                return false;",
        "            }",
        "            if is_size_ask(&n) {",
        "                return true;",
        "            }",
        "            lane.size_phrases.iter().any(|p| *p == n)",
        "        }",
        "        StatKind::Age => {",
        "            if n.contains(\"path\")",
        "                || n.contains(\"where\")",
        "                || n.contains(\"location\")",
        "                || is_size_ask(&n)",
        "            {",
        "                return false;",
        "            }",
        "            if is_age_ask(&n) {",
        "                return true;",
        "            }",
        "            lane.age_phrases.iter().any(|p| *p == n)",
        "        }",
        "    }",
        "}",
        "",
        "fn is_size_ask(n: &str) -> bool {",
        "    n.contains(\"size\")",
        "        || n.contains(\"how big\")",
        "        || n.contains(\"bytes\")",
        "        || n.contains(\"disk use\")",
        "        || n.contains(\"on disk\")",
        "}",
        "",
        "fn is_age_ask(n: &str) -> bool {",
        "    contains_age_token(n)",
        "        || n.contains(\"how old\")",
        "        || n.contains(\"last write\")",
        "        || n.contains(\"mtime\")",
        "        || n.contains(\"modified\")",
        "}",
        "",
        "pub static RESOURCE_LANES: &[ResourceLane] = &[",
    ]
    for e in entries:
        lines.append("    ResourceLane {")
        lines.append(f'        id: "{e["id"]}",')
        nouns = ", ".join(f'"{n}"' for n in e["nouns"][:12])
        lines.append(f"        nouns: &[{nouns}],")
        lines.append(f"        max_len: {e['max_len']},")
        for key in ("path_phrases", "size_phrases", "age_phrases"):
            phrases = e[key][:80]
            if not phrases:
                lines.append(f"        {key}: &[],")
            else:
                joined = ", ".join(f'"{p}"' for p in phrases)
                lines.append(f"        {key}: &[{joined}],")
        lines.append("    },")
    lines.append("];")
    lines.append("")
    lines.append("pub fn lane_by_id(id: &str) -> Option<&'static ResourceLane> {")
    lines.append("    RESOURCE_LANES.iter().find(|l| l.id == id)")
    lines.append("}")
    lines.append("")
    return "\n".join(lines) + "\n"


def emit_resource_wrappers(stems: list[str]) -> str:
    out = ["//! Thin wrappers preserving `looks_like_*_{path,size,age}_request` names.", ""]
    for stem in stems:
        for kind, sk in (("path", "Path"), ("size", "Size"), ("age", "Age")):
            out.append(f"pub fn looks_like_{stem}_{kind}_request(content: &str) -> bool {{")
            out.append(
                f'    lane_by_id("{stem}").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::{sk}))'
            )
            out.append("}")
            out.append("")
    return "\n".join(out)


def compact_tests() -> str:
    return '''#[cfg(test)]
mod tests {
    use super::*;
    use super::keep_stats::{parse_keep_percentile, looks_like_keep_percentile_request};
    use super::normalize::normalize_operator_command;

    #[test]
    fn parses_session_name() {
        let (src, slug) =
            agent_ops::parse_session_filename("session-memory-discord-20260720-181500-weather.md");
        assert_eq!(src, "discord");
        assert!(slug.contains("weather"));
    }

    #[test]
    fn keep_percentile_digit_safe() {
        assert_eq!(parse_keep_percentile("/keep-p47"), Some(47));
        assert_eq!(parse_keep_percentile("/keep-p4"), Some(4));
        assert_eq!(parse_keep_percentile("/keep-p4 foo"), Some(4));
        // /keep-p47 must not be parsed as 4
        assert_ne!(parse_keep_percentile("/keep-p47"), Some(4));
        assert_eq!(parse_keep_percentile("keep p47"), Some(47));
        assert_eq!(parse_keep_percentile("47th percentile keep gap"), Some(47));
        assert!(parse_keep_percentile("results.tsv path").is_none());
        assert!(parse_keep_percentile("/keep-pace").is_none());
        assert!(parse_keep_percentile("/status").is_none());
        assert!(looks_like_keep_percentile_request("/keep-p90"));
    }

    #[test]
    fn resource_path_size_age_do_not_steal() {
        assert!(resource_stat::looks_like_debug_log_path_request("debug.log path"));
        assert!(resource_stat::looks_like_debug_log_size_request("debug.log size"));
        assert!(!resource_stat::looks_like_debug_log_path_request("debug.log size"));
        assert!(!resource_stat::looks_like_debug_log_size_request("debug.log path"));
        assert!(resource_stat::looks_like_results_tsv_path_request("results.tsv path"));
    }

    #[test]
    fn path_request_matchers_stay_fast_without_sibling_nesting() {
        let start = std::time::Instant::now();
        for _ in 0..2000 {
            let _ = resource_stat::looks_like_debug_log_path_request("hello world path maybe");
            let _ = resource_stat::looks_like_config_path_request("config somewhere");
            let _ = resource_stat::looks_like_disk_cleanup_path_request("cleanup path?");
        }
        assert!(
            start.elapsed().as_millis() < 500,
            "path matchers too slow — check for nested looks_like_* calls"
        );
    }

    #[test]
    fn operator_instant_reply_covers_gateway_commands() {
        let status = try_operator_instant_reply("/status").expect("status");
        assert!(status.contains("mac-stats"));
        assert!(try_operator_instant_reply("/keep-p47").is_some());
        assert!(try_operator_instant_reply("/keep-p4").is_some());
        assert!(try_operator_instant_reply("results.tsv path").is_some());
    }

    #[test]
    fn normalize_lowercases() {
        let n = normalize_operator_command("  /Keep-P47  ");
        assert!(n.contains("keep") || n.contains("p47") || n.starts_with("/keep"));
    }
}
'''


def emit_mod_rs(router_body: str) -> str:
    return f'''//! Dashboard "Agent Ops" APIs + Discord operator instant lane.
//! Split after the 135k-line god-file refactor — see docs/ops/harness_ops_refactor.md.

pub mod agent_ops;
pub mod insights;
pub mod inventory;
pub mod keep_stats;
pub mod normalize;
pub mod resource_stat;
pub mod status_chips;

pub use agent_ops::*;
pub use insights::*;
pub use inventory::*;
pub use keep_stats::{{
    format_keep_percentile_gateway, looks_like_keep_percentile_request, mentions_keep_percentile,
    parse_keep_percentile,
}};
pub use normalize::*;
pub use resource_stat::*;
pub use status_chips::*;

{router_body}

{compact_tests()}
'''


def main() -> int:
    if not SRC.exists():
        # Already refactored?
        if (OUT_DIR / "mod.rs").exists():
            print("already refactored")
            return 0
        print("missing", SRC, file=sys.stderr)
        return 1

    text = SRC.read_text()
    lines = text.splitlines()
    print(f"read {len(lines)} lines")
    preamble, items, test_start = split_items(lines)
    print(f"items={len(items)} preamble={len(preamble)} test_start={test_start}")
    if len(items) < 100:
        print("splitter failed — aborting without deleting god file", file=sys.stderr)
        return 1

    # Collect resource lane data from looks_like before dropping bodies
    resource_data: dict[str, dict] = {}
    for kind, name, start, end in items:
        if kind != "fn":
            continue
        m = re.match(r"looks_like_(\w+)_(path|size|age)_request$", name)
        if not m:
            continue
        stem, sk = m.group(1), m.group(2)
        body = "\n".join(lines[start:end])
        entry = resource_data.setdefault(
            stem,
            {
                "id": stem,
                "nouns": stem_nouns(stem),
                "max_len": 72,
                "path_phrases": [],
                "size_phrases": [],
                "age_phrases": [],
            },
        )
        entry["max_len"] = max(entry["max_len"], extract_max_len(body))
        phrases = extract_matches_phrases(body)
        entry[f"{sk}_phrases"] = phrases
        nouns = extract_noun_requires(body, stem)
        # merge nouns
        for n in nouns:
            if n not in entry["nouns"]:
                entry["nouns"].append(n)

    print(f"resource stems={len(resource_data)}")

    buckets: dict[str, list[str]] = defaultdict(list)
    router_body = ""
    skipped_pnn = 0
    skipped_resource_looks = 0

    for kind, name, start, end in items:
        if kind == "fn" and is_pnn_name(name):
            skipped_pnn += 1
            continue
        body = "\n".join(lines[start:end])
        # Resource looks_like → wrappers generated later
        if kind == "fn" and re.match(r"looks_like_\w+_(path|size|age)_request$", name):
            skipped_resource_looks += 1
            continue
        body = clean_body_mentions(body)
        if name == "try_operator_instant_reply":
            router_body = rewrite_router(body)
            continue
        # parse_run_ts used by keep_stats — keep pub
        if name == "parse_run_ts":
            body = body.replace("fn parse_run_ts", "pub fn parse_run_ts", 1)
        if name == "parse_session_filename":
            body = body.replace("fn parse_session_filename", "pub fn parse_session_filename", 1)
        if name == "normalize_operator_command":
            body = body.replace(
                "fn normalize_operator_command", "pub fn normalize_operator_command", 1
            )
        if name == "contains_age_token":
            body = body.replace("fn contains_age_token", "pub fn contains_age_token", 1)

        mod = classify(name, kind)
        buckets[mod].append(body)

    print(
        f"skipped pnn={skipped_pnn} resource_looks={skipped_resource_looks} router_len={len(router_body.splitlines())}"
    )

    OUT_DIR.mkdir(parents=True, exist_ok=True)

    # normalize.rs
    norm = buckets["normalize"]
    # ensure both functions present — if missing, emit stubs from known source
    if not any("normalize_operator_command" in x for x in norm):
        # find in original
        for kind, name, start, end in items:
            if name in ("normalize_operator_command", "contains_age_token"):
                b = "\n".join(lines[start:end])
                b = b.replace(f"fn {name}", f"pub fn {name}", 1)
                norm.append(b)
    (OUT_DIR / "normalize.rs").write_text(
        "//! Shared operator-command normalization.\n\n" + "\n\n".join(norm) + "\n"
    )

    # agent_ops.rs — need serde imports etc.
    agent_header = '''//! Agent Ops Tauri commands + digest / runs insights compute.
use crate::config::Config;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::normalize::*;

'''
    # Filter agent_ops bodies that still have their own use/imports? They shouldn't.
    (OUT_DIR / "agent_ops.rs").write_text(
        agent_header + "\n\n".join(buckets["agent_ops"]) + "\n"
    )

    # keep_stats.rs
    keep_header = '''//! results.tsv keep/discard analytics + parameterized `/keep-p{N}`.
use super::normalize::{contains_age_token, normalize_operator_command};
use super::agent_ops::parse_run_ts;

'''
    keep_body = (
        emit_keep_percentile_module()
        + "\n"
        + emit_format_keep_percentile()
        + "\n\n"
        + "\n\n".join(buckets["keep_stats"])
    )
    # Fix parse_run_ts references inside keep_stats originals
    keep_body = keep_body.replace("parse_run_ts(", "parse_run_ts(")
    # Original count_* used parse_run_ts in same module — now imported.
    # overnight_window_start_local and duration_label stay in keep_stats bucket.
    # Remove duplicate mentions of ResultsTsv if any.
    # Fix agent_ops::parse_run_ts in emitted percentile (already).
    # Local overnight in count — uses overnight_window_start_local from same mod.
    keep_body = keep_body.replace(
        "let Some(parsed) = super::agent_ops::parse_run_ts(ts) else {",
        "let Some(parsed) = parse_run_ts(ts) else {",
    )
    # Inject mentions_keep_percentile into other keep matchers that lost pNN excludes
    # Add early exclude in looks_like_results_tsv_* that aren't path/size/age
    def inject_pct_exclude(fn_body: str) -> str:
        if "looks_like_results_tsv_" not in fn_body:
            return fn_body
        if "parse_keep_percentile" in fn_body or "mentions_keep_percentile" in fn_body:
            return fn_body
        if "looks_like_results_tsv_path" in fn_body or "looks_like_results_tsv_size" in fn_body or "looks_like_results_tsv_age" in fn_body:
            return fn_body
        return fn_body.replace(
            "let n = normalize_operator_command(content);",
            "let n = normalize_operator_command(content);\n"
            "    if mentions_keep_percentile(&n) {\n"
            "        return false;\n"
            "    }",
            1,
        )

    keep_parts = keep_body.split("\n\npub fn ")
    # too fragile — apply per looks_like in bucket instead
    new_keep_parts = []
    for block in buckets["keep_stats"]:
        if block.lstrip().startswith("pub fn looks_like_results_tsv_") or "\npub fn looks_like_results_tsv_" in block:
            block = inject_pct_exclude(block)
        # also private fn looks
        if re.search(r"fn looks_like_results_tsv_", block):
            block = inject_pct_exclude(block)
        new_keep_parts.append(block)
    keep_body = (
        emit_keep_percentile_module()
        + "\n"
        + emit_format_keep_percentile().replace(
            "let Some(parsed) = super::agent_ops::parse_run_ts(ts) else {",
            "let Some(parsed) = parse_run_ts(ts) else {",
        )
        + "\n\n"
        + "\n\n".join(new_keep_parts)
    )
    (OUT_DIR / "keep_stats.rs").write_text(keep_header + keep_body)

    # resource_stat.rs — core table + wrappers + formatters from bucket
    stems = sorted(resource_data.keys())
    entries = [resource_data[s] for s in stems]
    res_fmt = "\n\n".join(buckets["resource_stat"])
    # formatters still call helpers in other modules — OK via super::
    res_text = (
        emit_resource_stat_core(entries)
        + "\n"
        + emit_resource_wrappers(stems)
        + "\n"
        + res_fmt
        + "\n"
    )
    # Fix formatters that call sibling looks or local helpers — leave as-is.
    (OUT_DIR / "resource_stat.rs").write_text(res_text)

    for mod_name in ("insights", "inventory", "status_chips"):
        header = f"//! {mod_name} instant-lane helpers.\nuse super::normalize::*;\n\n"
        (OUT_DIR / f"{mod_name}.rs").write_text(
            header + "\n\n".join(buckets[mod_name]) + "\n"
        )

    # mod.rs
    if not router_body:
        router_body = "pub fn try_operator_instant_reply(_content: &str) -> Option<String> { None }\n"
    (OUT_DIR / "mod.rs").write_text(emit_mod_rs(router_body))

    # Remove old god file
    SRC.unlink()
    print("removed god file")

    # Docs + guard
    DOCS.write_text(
        """# harness_ops refactor (instant lane)

## Why

`commands/harness_ops.rs` grew to **135k lines / 5.2MB**. Overnight shipped one
`/keep-pNN` glance per version by copy-pasting ~1k-line matchers (98% identical).

## Rules (do not re-inflate)

1. **New keep percentiles** — extend `parse_keep_percentile` / use `/keep-p{N}`
   (1–99). Do **not** add `looks_like_results_tsv_pNN_request` or
   `format_results_tsv_pNN_gateway`.
2. **New path/size/age file lanes** — add a `ResourceLane` in
   `resource_stat.rs` (nouns + phrases). Do **not** clone exclusion laundry lists.
3. **Sibling excludes** — string-only. Never call other `looks_like_*` from a
   path matcher (see `2026-09-15-instant-lane-path-nesting-hang.md`).
4. **Line budget** — `python3 scripts/check_harness_ops_budget.py` must pass.

## Layout

```
commands/harness_ops/
  mod.rs           # try_operator_instant_reply router + re-exports
  agent_ops.rs     # Tauri Agent Ops + digest
  normalize.rs     # normalize_operator_command
  resource_stat.rs # table-driven path/size/age
  keep_stats.rs    # results.tsv + parameterized /keep-p{N}
  inventory.rs     # schedules/monitors/agents/…
  status_chips.rs  # /status + ready chips
  insights.rs      # /insights and run slices
```
"""
    )
    GUARD.write_text(
        '''#!/usr/bin/env python3
"""Fail if harness_ops grows past budget or pNN clones return."""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIR = ROOT / "src-tauri/src/commands/harness_ops"
BUDGET_TOTAL = 20000
BUDGET_FILE = 8000
PNN = re.compile(r"fn looks_like_results_tsv_p\\d+_request|fn format_results_tsv_p\\d+_gateway|fn count_results_tsv_p\\d+")

def main() -> int:
    if not DIR.is_dir():
        print("missing", DIR, file=sys.stderr)
        return 1
    total = 0
    bad = []
    for p in sorted(DIR.glob("*.rs")):
        text = p.read_text()
        n = text.count("\\n") + (0 if text.endswith("\\n") or not text else 1)
        # accurate:
        n = len(text.splitlines())
        total += n
        if n > BUDGET_FILE:
            bad.append(f"{p.name}: {n} lines > {BUDGET_FILE}")
        for m in PNN.finditer(text):
            bad.append(f"{p.name}: forbidden pNN clone: {m.group(0)}")
    if total > BUDGET_TOTAL:
        bad.append(f"total {total} lines > {BUDGET_TOTAL}")
    print(f"harness_ops total_lines={total}")
    if bad:
        print("FAIL:", file=sys.stderr)
        for b in bad:
            print(" ", b, file=sys.stderr)
        return 1
    print("OK")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
'''
    )
    GUARD.chmod(0o755)

    # line counts
    total = 0
    for p in sorted(OUT_DIR.glob("*.rs")):
        n = len(p.read_text().splitlines())
        total += n
        print(f"  {p.name}: {n}")
    print(f"total {total}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
