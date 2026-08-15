//! Per-process GPU usage via AGXAccelerator IORegistry (Apple Silicon).
//!
//! Each Metal/GL client is an `AGXDeviceUserClient` under the accelerator with
//! `IOUserClientCreator` (`pid N, name`) and `AppUsage` (`accumulatedGPUTime` ns).
//! Sample twice and divide delta by wall time → estimated GPU % (best-effort).
//! No sudo. Same family of data Activity Monitor uses.

use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use std::time::Instant;

use crate::debug3;

#[derive(Clone, Debug)]
struct GpuSample {
    /// Sum of accumulatedGPUTime (ns) across that PID’s user clients.
    accumulated_ns: u64,
}

static PREV_SAMPLES: Mutex<Option<(HashMap<u32, GpuSample>, Instant)>> = Mutex::new(None);
static CACHED_PCT: Mutex<Option<(HashMap<u32, f32>, Instant)>> = Mutex::new(None);

/// Latest per-PID estimated GPU utilization % (0–100+; may exceed 100 if multi-queue).
pub fn gpu_usage_by_pid() -> HashMap<u32, f32> {
    // How old is the previous raw sample?
    let prev_age_ms = PREV_SAMPLES
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|(_, at)| at.elapsed().as_millis()))
        .unwrap_or(u128::MAX);

    // Rate-limit ioreg (~20ms each). Allow a quick second sample for deltas,
    // then hold ~3s so process-list + details refresh do not stack dumps.
    let have_prev = prev_age_ms != u128::MAX;
    let need_second = have_prev && prev_age_ms >= 50 && prev_age_ms < 2000;
    let ioreg_fresh_enough = (prev_age_ms < 3000 && !need_second) || prev_age_ms < 50;

    if ioreg_fresh_enough {
        if let Ok(cache) = CACHED_PCT.lock() {
            if let Some((map, _)) = cache.as_ref() {
                if !map.is_empty() || prev_age_ms < 50 {
                    return map.clone();
                }
            }
        }
    }

    let raw = sample_accumulated_ns_by_pid();
    let now = Instant::now();
    let mut pct: HashMap<u32, f32> = HashMap::new();

    if let Ok(mut prev_guard) = PREV_SAMPLES.lock() {
        if let Some((prev_map, prev_at)) = prev_guard.as_ref() {
            let elapsed_ns = prev_at.elapsed().as_nanos() as f64;
            if elapsed_ns > 50_000_000.0 {
                for (pid, cur_ns) in &raw {
                    if let Some(prev) = prev_map.get(pid) {
                        if *cur_ns >= prev.accumulated_ns {
                            let delta = (*cur_ns - prev.accumulated_ns) as f64;
                            let mut p = (delta / elapsed_ns) * 100.0;
                            if p.is_finite() {
                                if p < 0.0 {
                                    p = 0.0;
                                }
                                if p > 999.0 {
                                    p = 999.0;
                                }
                                if p >= 0.05 {
                                    pct.insert(*pid, p as f32);
                                }
                            }
                        }
                    }
                }
            }
        }

        let stamped: HashMap<u32, GpuSample> = raw
            .into_iter()
            .map(|(pid, ns)| (pid, GpuSample { accumulated_ns: ns }))
            .collect();
        *prev_guard = Some((stamped, now));
    }

    // Keep last non-empty % if this pass is still warming (first sample → empty pct).
    if pct.is_empty() {
        if let Ok(cache) = CACHED_PCT.lock() {
            if let Some((map, at)) = cache.as_ref() {
                if !map.is_empty() && at.elapsed().as_secs() < 5 {
                    return map.clone();
                }
            }
        }
    }

    if let Ok(mut cache) = CACHED_PCT.lock() {
        // Do not overwrite a good cache with empty first-sample noise.
        let overwrite = !pct.is_empty()
            || cache
                .as_ref()
                .map(|(m, _)| m.is_empty())
                .unwrap_or(true);
        if overwrite {
            *cache = Some((pct.clone(), now));
        }
    }
    debug3!(
        "GPU processes: {} PIDs with measurable use (raw clients sampled)",
        pct.len()
    );
    pct
}

fn sample_accumulated_ns_by_pid() -> HashMap<u32, u64> {
    // `-l` lists properties on children (AGXDeviceUserClient). Without it, some
    // macOS builds omit AppUsage / IOUserClientCreator on the text dump.
    let output = Command::new("/usr/sbin/ioreg")
        .args(["-r", "-l", "-w", "0", "-c", "AGXAccelerator"])
        .stderr(std::process::Stdio::null())
        .output();

    let Ok(output) = output else {
        return HashMap::new();
    };
    if !output.status.success() {
        return HashMap::new();
    }
    parse_ioreg_agx_clients(&String::from_utf8_lossy(&output.stdout))
}

/// Parse `ioreg -rl -c AGXAccelerator` text for creator PID + AppUsage GPU times.
pub(crate) fn parse_ioreg_agx_clients(stdout: &str) -> HashMap<u32, u64> {
    let mut by_pid: HashMap<u32, u64> = HashMap::new();
    let mut pending_usage: Option<u64> = None;

    for line in stdout.lines() {
        let t = line.trim();
        if t.contains("AppUsage") {
            pending_usage = Some(sum_accumulated_gpu_time(t));
            continue;
        }
        if let Some(pid) = parse_creator_pid(t) {
            let add = pending_usage.take().unwrap_or(0);
            let entry = by_pid.entry(pid).or_insert(0);
            *entry = entry.saturating_add(add);
        }
    }
    by_pid
}

fn sum_accumulated_gpu_time(line: &str) -> u64 {
    let mut total = 0u64;
    let key = "accumulatedGPUTime";
    let mut rest = line;
    while let Some(idx) = rest.find(key) {
        rest = &rest[idx + key.len()..];
        let rest = rest.trim_start_matches(|c: char| c == '"' || c == '=' || c.is_whitespace());
        let num: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(v) = num.parse::<u64>() {
            total = total.saturating_add(v);
        }
    }
    total
}

fn parse_creator_pid(line: &str) -> Option<u32> {
    // "IOUserClientCreator" = "pid 532, WindowServer"
    if !line.contains("IOUserClientCreator") {
        return None;
    }
    let after = line.split("pid ").nth(1)?;
    let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_creator_and_usage() {
        let sample = r#"
  +-o AGXDeviceUserClient
  |     "AppUsage" = ({"API"="Metal","accumulatedGPUTime"=1000},{"API"="Metal","accumulatedGPUTime"=500})
  |     "IOUserClientCreator" = "pid 532, WindowServer"
  +-o AGXDeviceUserClient
  |     "AppUsage" = ()
  |     "IOUserClientCreator" = "pid 999, IdleClient"
"#;
        let m = parse_ioreg_agx_clients(sample);
        assert_eq!(m.get(&532), Some(&1500));
        assert_eq!(m.get(&999), Some(&0));
    }

    #[test]
    fn sum_usage_line() {
        assert_eq!(
            sum_accumulated_gpu_time(
                r#"| "AppUsage" = ({"accumulatedGPUTime"=10},{"accumulatedGPUTime"=20})"#
            ),
            30
        );
    }

    #[test]
    fn parses_live_shaped_multiline_block() {
        // Creator after AppUsage (real ioreg order).
        let sample = r#"
  +-o AGXDeviceUserClient  <class AGXDeviceUserClient>
  |   {
  |     "AppUsage" = ({"API"="Metal","lastSubmittedTime"=1,"accumulatedGPUTime"=5000000000})
  |     "IOUserClientCreator" = "pid 4242, Chrome Helper"
  |   }
"#;
        let m = parse_ioreg_agx_clients(sample);
        assert_eq!(m.get(&4242), Some(&5_000_000_000));
    }
}
