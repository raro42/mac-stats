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
    /// Taken while the app was not in the foreground (a background refresh wake-up).
    /// For minutes and longer buckets: every sample in it was.
    pub bg: bool,
    /// Samples behind this point (1 for the seconds view).
    #[serde(skip)]
    pub n: u32,
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
    samples: u32,
    background: u32,
}

impl MinuteAccum {
    fn new(minute: i64) -> Self {
        MinuteAccum {
            minute,
            cpu: Mean::default(),
            ram: Mean::default(),
            app_mb: Mean::default(),
            thermal: None,
            samples: 0,
            background: 0,
        }
    }

    fn point(&self) -> Point {
        Point {
            ts: self.minute * MINUTE_MS,
            cpu: self.cpu.value(),
            ram: self.ram.value(),
            app_mb: self.app_mb.value(),
            thermal: self.thermal,
            bg: self.samples > 0 && self.background == self.samples,
            n: self.samples,
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
    /// Adds a one-second sample. Returns the minute it closed, if any, so the caller can
    /// save it (`History` itself does no I/O).
    pub fn push(&mut self, p: Point) -> Option<Point> {
        push_capped(&mut self.seconds, p, SECONDS_CAP);

        let minute = p.ts.div_euclid(MINUTE_MS);
        let closed = match self.current.as_ref().map(|acc| acc.minute) {
            Some(current) if current == minute => None,
            Some(current) if minute < current => return None, // clock went backwards: ignored
            _ => self.close_minute(minute),
        };
        let acc = self.current.get_or_insert_with(|| MinuteAccum::new(minute));
        acc.cpu.add(p.cpu);
        acc.ram.add(p.ram);
        acc.app_mb.add(p.app_mb);
        acc.thermal = Thermal::worst(acc.thermal, p.thermal);
        acc.samples += 1;
        if p.bg {
            acc.background += 1;
        }
        closed
    }

    /// The minute being filled, as a point (to save it early during a short background
    /// wake-up).
    pub fn current_minute(&self) -> Option<Point> {
        self.current.as_ref().map(MinuteAccum::point)
    }

    /// Minutes read from disk at startup, oldest first (only ones before the current
    /// minute), so the «1 h» view survives a restart.
    pub fn seed_minutes(&mut self, points: impl IntoIterator<Item = Point>) {
        for p in points {
            push_capped(&mut self.minutes, p, MINUTES_CAP);
        }
    }

    pub fn clear(&mut self) {
        *self = History::default();
    }

    /// Marks a break in the seconds view. The minutes view detects missing
    /// minutes on its own when it closes the next one.
    pub fn push_gap(&mut self, ts: i64) {
        if self.seconds.back().is_some_and(|p| !p.gap) {
            push_capped(&mut self.seconds, Point::gap(ts), SECONDS_CAP);
        }
    }

    /// Closes the current minute and opens `next`; if minutes are missing in between,
    /// it leaves a gap point. Returns the closed minute.
    fn close_minute(&mut self, next: i64) -> Option<Point> {
        let closed = self.current.take().map(|acc| {
            let point = acc.point();
            push_capped(&mut self.minutes, point, MINUTES_CAP);
            if next > acc.minute + 1 {
                push_capped(&mut self.minutes, Point::gap((acc.minute + 1) * MINUTE_MS), MINUTES_CAP);
            }
            point
        });
        self.current = Some(MinuteAccum::new(next));
        closed
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
        Point { ts, cpu: Some(cpu), ram: Some(50.0), app_mb: Some(80.0), thermal: Some(Thermal::Nominal), bg: false, n: 1, gap: false }
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
            Point { ts: 0, cpu: Some(20.0), ram: Some(50.0), app_mb: Some(80.0), thermal: Some(Thermal::Nominal), bg: false, n: 2, gap: false }
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
    fn push_returns_the_closed_minute_and_marks_background_minutes() {
        let mut h = History::default();
        let bg = |ts: i64| Point { bg: true, ..sample(ts, 5.0) };
        assert_eq!(h.push(bg(0)), None);
        assert_eq!(h.push(bg(10_000)), None);
        assert_eq!(h.current_minute().map(|p| (p.n, p.bg)), Some((2, true)));
        let closed = h.push(sample(MINUTE_MS, 7.0)).expect("closed minute");
        assert_eq!((closed.ts, closed.n, closed.bg), (0, 2, true));
        // A minute with any foreground sample is not a background minute.
        h.push(bg(MINUTE_MS + 1_000));
        assert_eq!(h.current_minute().map(|p| p.bg), Some(false));
    }

    #[test]
    fn seeded_minutes_show_in_the_hour_view() {
        let mut h = History::default();
        h.seed_minutes([Point { n: 60, ..sample(0, 10.0) }, Point { n: 60, ..sample(MINUTE_MS, 20.0) }]);
        h.push(sample(2 * MINUTE_MS, 30.0));
        let cpu: Vec<_> = h.minutes().iter().map(|p| p.cpu).collect();
        assert_eq!(cpu, [Some(10.0), Some(20.0), Some(30.0)]);
        h.clear();
        assert!(h.minutes().is_empty() && h.seconds().is_empty());
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
