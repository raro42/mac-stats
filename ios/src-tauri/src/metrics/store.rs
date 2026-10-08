//! History saved on the iPhone: one JSON line per minute (CPU and RAM means, worst
//! thermal state, whether it was a background minute), one file per UTC day in
//! `app_data_dir/history/YYYY-MM-DD.jsonl`, kept for 30 days.
//!
//! Minutes are appended when they close. During a short background wake-up the partial
//! minute is also appended every few samples, so it survives if iOS terminates the app
//! afterwards; reading keeps the last record of each minute.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::apple::Thermal;
use super::history::Point;

pub const KEEP_DAYS: i64 = 30;
const DAY_MS: i64 = 86_400_000;

/// One saved minute.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Record {
    /// Start of the minute, ms since the Unix epoch.
    pub ts: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ram: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thermal: Option<Thermal>,
    #[serde(default)]
    pub bg: bool,
    /// Samples behind the means.
    pub n: u32,
}

impl Record {
    pub fn from_point(p: &Point) -> Self {
        Record { ts: p.ts, cpu: p.cpu, ram: p.ram, thermal: p.thermal, bg: p.bg, n: p.n }
    }

    pub fn to_point(self) -> Point {
        Point { ts: self.ts, cpu: self.cpu, ram: self.ram, thermal: self.thermal, bg: self.bg, n: self.n, ..Point::default() }
    }
}

/// Civil date (UTC) of a day number since 1970-01-01 (Howard Hinnant's algorithm).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(month <= 2), month, day)
}

/// Day number since 1970-01-01 of a civil date (inverse of `civil_from_days`).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn day_of(ts: i64) -> i64 {
    ts.div_euclid(DAY_MS)
}

fn file_name(day: i64) -> String {
    let (y, m, d) = civil_from_days(day);
    format!("{y:04}-{m:02}-{d:02}.jsonl")
}

/// Day number of a `YYYY-MM-DD.jsonl` file name.
fn day_from_name(name: &str) -> Option<i64> {
    let date = name.strip_suffix(".jsonl")?;
    let mut parts = date.split('-').map(str::parse::<i64>);
    let (y, m, d) = (parts.next()?.ok()?, parts.next()?.ok()?, parts.next()?.ok()?);
    let day = days_from_civil(y, u32::try_from(m).ok()?, u32::try_from(d).ok()?);
    (file_name(day) == name).then_some(day)
}

pub struct HistoryStore {
    dir: PathBuf,
}

impl HistoryStore {
    pub fn new(dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&dir);
        HistoryStore { dir }
    }

    fn path(&self, day: i64) -> PathBuf {
        self.dir.join(file_name(day))
    }

    pub fn append(&self, record: &Record) -> io::Result<()> {
        let mut line = serde_json::to_vec(record)?;
        line.push(b'\n');
        fs::OpenOptions::new().create(true).append(true).open(self.path(day_of(record.ts)))?.write_all(&line)
    }

    /// Records of one day file, one per minute (the last one written wins).
    fn read_day(path: &Path) -> BTreeMap<i64, Record> {
        let mut out = BTreeMap::new();
        if let Ok(text) = fs::read_to_string(path) {
            for record in text.lines().filter_map(|l| serde_json::from_str::<Record>(l).ok()) {
                out.insert(record.ts, record);
            }
        }
        out
    }

    /// Minutes with `from <= ts < to`, oldest first.
    pub fn range(&self, from: i64, to: i64) -> Vec<Record> {
        (day_of(from)..=day_of(to))
            .flat_map(|day| Self::read_day(&self.path(day)).into_values())
            .filter(|r| r.ts >= from && r.ts < to)
            .collect()
    }

    /// Deletes days older than `KEEP_DAYS` and rewrites past days without the duplicate
    /// partial records left by background wake-ups. Run at startup.
    pub fn maintain(&self, now: i64) {
        let today = day_of(now);
        let Ok(entries) = fs::read_dir(&self.dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(day) = path.file_name().and_then(|n| n.to_str()).and_then(day_from_name) else {
                continue; // not one of our day files
            };
            if today - day >= KEEP_DAYS {
                let _ = fs::remove_file(&path);
            } else if day < today {
                let records = Self::read_day(&path);
                let lines: String = records.values().filter_map(|r| serde_json::to_string(r).ok()).map(|l| l + "\n").collect();
                let tmp = path.with_extension("jsonl.tmp");
                if fs::write(&tmp, lines).is_ok() {
                    let _ = fs::rename(&tmp, &path);
                }
            }
        }
    }

    /// Deletes every saved minute.
    pub fn clear(&self) -> io::Result<()> {
        for entry in fs::read_dir(&self.dir)?.flatten() {
            fs::remove_file(entry.path())?;
        }
        Ok(())
    }
}

/// Groups minutes into buckets of `bucket` ms for the long views: CPU and RAM means
/// weighted by sample count, the worst thermal state, and `bg` when every minute was a
/// background one. Missing buckets are simply absent (the charts break the line there).
pub fn downsample(records: &[Record], bucket: i64) -> Vec<Point> {
    let mut groups: BTreeMap<i64, Vec<&Record>> = BTreeMap::new();
    for r in records {
        groups.entry(r.ts - r.ts.rem_euclid(bucket)).or_default().push(r);
    }
    groups
        .into_iter()
        .map(|(ts, rs)| {
            let mean = |get: fn(&Record) -> Option<f32>| {
                let (sum, weight) = rs
                    .iter()
                    .filter_map(|r| get(r).map(|v| (v as f64 * r.n.max(1) as f64, r.n.max(1) as f64)))
                    .fold((0.0, 0.0), |(s, w), (v, n)| (s + v, w + n));
                (weight > 0.0).then(|| (sum / weight) as f32)
            };
            Point {
                ts,
                cpu: mean(|r| r.cpu),
                ram: mean(|r| r.ram),
                thermal: rs.iter().fold(None, |acc, r| Thermal::worst(acc, r.thermal)),
                bg: rs.iter().all(|r| r.bg),
                n: rs.iter().map(|r| r.n).sum(),
                ..Point::default()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: i64 = 60_000;

    fn temp_store(name: &str) -> HistoryStore {
        let dir = std::env::temp_dir().join(format!("ios-stats-history-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        HistoryStore::new(dir)
    }

    fn rec(ts: i64, cpu: f32, n: u32) -> Record {
        Record { ts, cpu: Some(cpu), ram: Some(50.0), thermal: Some(Thermal::Nominal), bg: false, n }
    }

    #[test]
    fn day_file_names_are_utc_dates() {
        assert_eq!(file_name(0), "1970-01-01.jsonl");
        assert_eq!(file_name(day_of(1_791_417_600_000)), "2026-10-08.jsonl");
        assert_eq!(file_name(day_of(951_782_400_000)), "2000-02-29.jsonl");
    }

    #[test]
    fn file_names_round_trip() {
        for day in [0, 11_016, 20_369, 20_734] {
            assert_eq!(day_from_name(&file_name(day)), Some(day));
        }
        assert_eq!(day_from_name("notes.txt"), None);
        assert_eq!(day_from_name("2026-02-30.jsonl"), None);
    }

    #[test]
    fn partial_then_final_record_of_a_minute_reads_once() {
        let store = temp_store("dedupe");
        store.append(&rec(0, 10.0, 5)).unwrap();
        store.append(&rec(0, 12.0, 60)).unwrap();
        store.append(&rec(MIN, 20.0, 60)).unwrap();
        let out = store.range(0, 2 * MIN);
        assert_eq!(out.iter().map(|r| (r.ts, r.cpu, r.n)).collect::<Vec<_>>(), [(0, Some(12.0), 60), (MIN, Some(20.0), 60)]);
        assert!(store.range(MIN + 1, 3 * MIN).is_empty());
    }

    #[test]
    fn ranges_span_day_files() {
        let store = temp_store("days");
        let midnight = 20 * DAY_MS;
        store.append(&rec(midnight - MIN, 1.0, 60)).unwrap();
        store.append(&rec(midnight, 2.0, 60)).unwrap();
        let out = store.range(midnight - DAY_MS, midnight + DAY_MS);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn maintenance_drops_old_days_and_compacts_past_ones() {
        let store = temp_store("maintain");
        let now = 100 * DAY_MS + 5 * MIN;
        let old = now - KEEP_DAYS * DAY_MS;
        let yesterday = now - DAY_MS;
        store.append(&rec(old, 1.0, 60)).unwrap();
        store.append(&rec(yesterday, 2.0, 3)).unwrap();
        store.append(&rec(yesterday, 2.5, 60)).unwrap();
        store.append(&rec(now, 3.0, 3)).unwrap();
        store.append(&rec(now, 3.5, 6)).unwrap();
        store.maintain(now);
        assert!(!store.path(day_of(old)).exists());
        let lines = |ts: i64| fs::read_to_string(store.path(day_of(ts))).unwrap().lines().count();
        assert_eq!(lines(yesterday), 1, "past day compacted");
        assert_eq!(lines(now), 2, "today is left alone");
        assert_eq!(store.range(yesterday, yesterday + 1)[0].cpu, Some(2.5));
    }

    #[test]
    fn clear_removes_everything() {
        let store = temp_store("clear");
        store.append(&rec(0, 1.0, 1)).unwrap();
        store.clear().unwrap();
        assert!(store.range(0, DAY_MS).is_empty());
    }

    #[test]
    fn downsampling_weights_means_and_keeps_the_worst_state() {
        let mut a = rec(0, 10.0, 60);
        let mut b = rec(MIN, 40.0, 20);
        b.thermal = Some(Thermal::Serious);
        a.bg = true;
        b.bg = true;
        let mut c = rec(10 * MIN, 50.0, 60);
        c.bg = true;
        let mut d = rec(11 * MIN, 50.0, 60);
        d.bg = false;
        let out = downsample(&[a, b, c, d], 5 * MIN);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].ts, 0);
        assert_eq!(out[0].cpu, Some(17.5)); // (10*60 + 40*20) / 80
        assert_eq!(out[0].thermal, Some(Thermal::Serious));
        assert!(out[0].bg);
        assert_eq!(out[1].ts, 10 * MIN);
        assert!(!out[1].bg, "a bucket with foreground minutes is not a background bucket");
    }
}
