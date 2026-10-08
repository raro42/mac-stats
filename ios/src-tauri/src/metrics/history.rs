//! In-memory history for the charts: 300 one-second samples («5 min» view)
//! and 60 one-minute buckets («1 h» view).
//!
//! Rewrite of `src-tauri/src/metrics/history.rs` from the Mac app, which assumed
//! one sample per second and returned duplicate points where its tiers
//! overlapped. Here the tiers are independent and gaps (the app was in the
//! background) are marked with a `gap` point so the chart breaks the line.

use serde::Serialize;
use std::collections::VecDeque;

use super::apple::Thermal;

pub const SECONDS_CAP: usize = 300;
pub const MINUTES_CAP: usize = 60;
const MINUTE_MS: i64 = 60_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    /// Milliseconds since the Unix epoch.
    pub ts: i64,
    pub cpu: Option<f32>,
    /// System RAM used, in %.
    pub ram: Option<f32>,
    /// App memory, in MB.
    pub app_mb: Option<f32>,
    /// iOS thermal state; in the «1 h» view, the worst state of the minute.
    pub thermal: Option<Thermal>,
    /// `true` marks a break: there is no data between the previous point and the next.
    pub gap: bool,
}

impl Point {
    pub fn gap(ts: i64) -> Self {
        Point { ts, gap: true, ..Point::default() }
    }
}

#[derive(Default)]
struct Mean {
    sum: f64,
    n: u32,
}

impl Mean {
    fn add(&mut self, v: Option<f32>) {
        if let Some(v) = v {
            self.sum += v as f64;
            self.n += 1;
        }
    }

    fn value(&self) -> Option<f32> {
        (self.n > 0).then(|| (self.sum / self.n as f64) as f32)
    }
}

/// Current minute, not closed yet.
struct MinuteAccum {
    minute: i64,
    cpu: Mean,
    ram: Mean,
    app_mb: Mean,
    thermal: Option<Thermal>,
}

impl MinuteAccum {
    fn new(minute: i64) -> Self {
        MinuteAccum { minute, cpu: Mean::default(), ram: Mean::default(), app_mb: Mean::default(), thermal: None }
    }

    fn point(&self) -> Point {
        Point {
            ts: self.minute * MINUTE_MS,
            cpu: self.cpu.value(),
            ram: self.ram.value(),
            app_mb: self.app_mb.value(),
            thermal: self.thermal,
            gap: false,
        }
    }
}

#[derive(Default)]
pub struct History {
    seconds: VecDeque<Point>,
    minutes: VecDeque<Point>,
    current: Option<MinuteAccum>,
}

fn push_capped(buf: &mut VecDeque<Point>, p: Point, cap: usize) {
    if buf.len() == cap {
        buf.pop_front();
    }
    buf.push_back(p);
}

impl History {
    pub fn push(&mut self, p: Point) {
        push_capped(&mut self.seconds, p, SECONDS_CAP);

        let minute = p.ts.div_euclid(MINUTE_MS);
        match self.current.as_ref().map(|acc| acc.minute) {
            Some(current) if current == minute => {}
            Some(current) if minute < current => return, // clock went backwards: ignored
            _ => self.close_minute(minute),
        }
        let acc = self.current.get_or_insert_with(|| MinuteAccum::new(minute));
        acc.cpu.add(p.cpu);
        acc.ram.add(p.ram);
        acc.app_mb.add(p.app_mb);
        acc.thermal = Thermal::worst(acc.thermal, p.thermal);
    }

    /// Marks a break in the seconds view. The minutes view detects missing
    /// minutes on its own when it closes the next one.
    pub fn push_gap(&mut self, ts: i64) {
        if self.seconds.back().is_some_and(|p| !p.gap) {
            push_capped(&mut self.seconds, Point::gap(ts), SECONDS_CAP);
        }
    }

    /// Closes the current minute and opens `next`; if minutes are missing in between,
    /// it leaves a gap point.
    fn close_minute(&mut self, next: i64) {
        if let Some(acc) = self.current.take() {
            push_capped(&mut self.minutes, acc.point(), MINUTES_CAP);
            if next > acc.minute + 1 {
                push_capped(&mut self.minutes, Point::gap((acc.minute + 1) * MINUTE_MS), MINUTES_CAP);
            }
        }
        self.current = Some(MinuteAccum::new(next));
    }

    pub fn seconds(&self) -> Vec<Point> {
        self.seconds.iter().copied().collect()
    }

    /// Closed minutes plus the current one (so the «1 h» view stays live).
    pub fn minutes(&self) -> Vec<Point> {
        let mut out: Vec<Point> = self.minutes.iter().copied().collect();
        if let Some(acc) = &self.current {
            out.push(acc.point());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(ts: i64, cpu: f32) -> Point {
        Point { ts, cpu: Some(cpu), ram: Some(50.0), app_mb: Some(80.0), thermal: Some(Thermal::Nominal), gap: false }
    }

    #[test]
    fn seconds_tier_is_capped() {
        let mut h = History::default();
        for i in 0..(SECONDS_CAP as i64 + 10) {
            h.push(sample(i * 1_000, 1.0));
        }
        let s = h.seconds();
        assert_eq!(s.len(), SECONDS_CAP);
        assert_eq!(s[0].ts, 10_000);
    }

    #[test]
    fn minute_buckets_average_and_close_on_rollover() {
        let mut h = History::default();
        h.push(sample(0, 10.0));
        h.push(sample(30_000, 30.0));
        assert_eq!(h.minutes().len(), 1, "only the current minute");
        assert_eq!(h.minutes()[0].cpu, Some(20.0));

        h.push(sample(60_000, 50.0));
        let m = h.minutes();
        assert_eq!(m.len(), 2);
        assert_eq!(
            m[0],
            Point { ts: 0, cpu: Some(20.0), ram: Some(50.0), app_mb: Some(80.0), thermal: Some(Thermal::Nominal), gap: false }
        );
        assert_eq!(m[1].ts, 60_000);
        assert_eq!(m[1].cpu, Some(50.0));
    }

    #[test]
    fn missing_values_do_not_drag_the_average() {
        let mut h = History::default();
        h.push(sample(0, 40.0));
        h.push(Point { ts: 1_000, cpu: None, ..sample(1_000, 0.0) });
        assert_eq!(h.minutes()[0].cpu, Some(40.0));
    }

    #[test]
    fn skipped_minutes_leave_a_gap_point() {
        let mut h = History::default();
        h.push(sample(0, 10.0));
        h.push(sample(5 * MINUTE_MS, 20.0));
        let m = h.minutes();
        assert_eq!(m.len(), 3);
        assert!(!m[0].gap);
        assert!(m[1].gap);
        assert_eq!(m[1].ts, MINUTE_MS);
        assert_eq!(m[2].ts, 5 * MINUTE_MS);
    }

    #[test]
    fn gaps_in_seconds_are_not_duplicated() {
        let mut h = History::default();
        h.push(sample(0, 1.0));
        h.push_gap(500);
        h.push_gap(600);
        let s = h.seconds();
        assert_eq!(s.len(), 2);
        assert!(s[1].gap);
    }

    #[test]
    fn minutes_tier_is_capped() {
        let mut h = History::default();
        for m in 0..(MINUTES_CAP as i64 + 5) {
            h.push(sample(m * MINUTE_MS, 1.0));
        }
        // 60 closed + the current minute
        assert_eq!(h.minutes().len(), MINUTES_CAP + 1);
    }

    #[test]
    fn minute_buckets_keep_the_worst_thermal_state() {
        let mut h = History::default();
        let at = |ts: i64, t: Thermal| Point { thermal: Some(t), ..sample(ts, 1.0) };
        h.push(at(0, Thermal::Nominal));
        h.push(at(10_000, Thermal::Serious));
        h.push(at(20_000, Thermal::Fair));
        h.push(Point { thermal: None, ..sample(30_000, 1.0) });
        assert_eq!(h.minutes()[0].thermal, Some(Thermal::Serious));
        // Unknown never hides a known state.
        h.push(at(40_000, Thermal::Unknown));
        assert_eq!(h.minutes()[0].thermal, Some(Thermal::Serious));
        // The seconds view keeps each sample as is.
        assert_eq!(h.seconds()[2].thermal, Some(Thermal::Fair));
    }

    #[test]
    fn clock_going_backwards_is_ignored_for_minutes() {
        let mut h = History::default();
        h.push(sample(2 * MINUTE_MS, 10.0));
        h.push(sample(MINUTE_MS, 90.0));
        assert_eq!(h.minutes().len(), 1);
        assert_eq!(h.minutes()[0].cpu, Some(10.0));
    }
}
