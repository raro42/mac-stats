//! Generic per-service circuit breaker (closed → open → half-open).
//!
//! Used for Ollama (`/api/chat`, `/api/tags`) and Discord outbound HTTP sends. Logs transitions
//! under `mac_stats::circuit` for `~/.mac-stats/debug.log`.

use std::time::{Duration, Instant};

use crate::{mac_stats_debug, mac_stats_info, mac_stats_warn};

/// Emit at most one "Circuit opened" WARN per this interval; further reopenings stay DEBUG
/// (Ollama-down half-open retries every ~30s must not flood `debug.log`).
const OPEN_WARN_INTERVAL: Duration = Duration::from_secs(5 * 60);

fn should_warn_circuit_open(last_warn: Option<Instant>, now: Instant, interval: Duration) -> bool {
    last_warn
        .map(|t| now.duration_since(t) >= interval)
        .unwrap_or(true)
}

/// Circuit breaker state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Tracks consecutive failures and gates requests when the service is known unhealthy.
#[derive(Debug)]
pub struct CircuitBreaker {
    service_label: &'static str,
    state: CircuitState,
    consecutive_failures: u32,
    failure_threshold: u32,
    reset_interval: Duration,
    last_transition: Instant,
    /// In `HalfOpen`, whether the single probe slot is still available.
    half_open_probe_pending: bool,
    /// Last time we emitted a "Circuit opened" WARN for this breaker.
    last_open_warn: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(
        service_label: &'static str,
        failure_threshold: u32,
        reset_interval: Duration,
    ) -> Self {
        Self {
            service_label,
            state: CircuitState::Closed,
            consecutive_failures: 0,
            failure_threshold,
            reset_interval,
            last_transition: Instant::now(),
            half_open_probe_pending: false,
            last_open_warn: None,
        }
    }

    pub fn new_ollama() -> Self {
        Self::new("Ollama", 3, Duration::from_secs(30))
    }

    pub fn new_discord_sends() -> Self {
        Self::new("Discord API", 3, Duration::from_secs(30))
    }

    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// True when the circuit is fully open (not half-open probe).
    pub fn is_open_blocking(&self) -> bool {
        matches!(self.state, CircuitState::Open)
    }

    /// Returns `Ok(())` if an HTTP attempt may proceed; `Err` is user-facing when blocked.
    pub fn allow_request(&mut self) -> Result<(), String> {
        if self.state == CircuitState::Open {
            if self.last_transition.elapsed() >= self.reset_interval {
                self.state = CircuitState::HalfOpen;
                self.last_transition = Instant::now();
                self.half_open_probe_pending = true;
            } else {
                let remaining = self
                    .reset_interval
                    .saturating_sub(self.last_transition.elapsed());
                let secs = remaining.as_secs().max(1);
                return Err(format!(
                    "{} is temporarily unavailable (circuit open, will retry in {}s)",
                    self.service_label, secs
                ));
            }
        }

        if self.state == CircuitState::HalfOpen {
            if self.half_open_probe_pending {
                self.half_open_probe_pending = false;
                return Ok(());
            }
            let secs = self
                .reset_interval
                .saturating_sub(self.last_transition.elapsed())
                .as_secs()
                .max(1);
            return Err(format!(
                "{} is temporarily unavailable (circuit open, will retry in {}s)",
                self.service_label, secs
            ));
        }

        Ok(())
    }

    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::HalfOpen => {
                self.state = CircuitState::Closed;
                self.consecutive_failures = 0;
                self.half_open_probe_pending = false;
                mac_stats_info!(
                    "circuit",
                    "Circuit closed for {} — service recovered",
                    self.service_label
                );
            }
            CircuitState::Closed => {
                self.consecutive_failures = 0;
            }
            CircuitState::Open => {
                self.state = CircuitState::Closed;
                self.consecutive_failures = 0;
                self.half_open_probe_pending = false;
            }
        }
    }

    /// Count a failed attempt when `should_trip` is true (e.g. 5xx, timeouts, connection errors).
    pub fn record_failure(&mut self, should_trip: bool) {
        if !should_trip {
            return;
        }

        self.consecutive_failures = self.consecutive_failures.saturating_add(1);

        match self.state {
            CircuitState::Closed => {
                if self.consecutive_failures >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.last_transition = Instant::now();
                    self.half_open_probe_pending = false;
                    self.log_opened(self.consecutive_failures);
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.last_transition = Instant::now();
                self.half_open_probe_pending = false;
                self.log_opened(self.failure_threshold);
            }
            CircuitState::Open => {}
        }
    }

    fn log_opened(&mut self, failure_count: u32) {
        let now = Instant::now();
        let msg = format!(
            "Circuit opened for {} after {} consecutive failures",
            self.service_label, failure_count
        );
        if should_warn_circuit_open(self.last_open_warn, now, OPEN_WARN_INTERVAL) {
            self.last_open_warn = Some(now);
            mac_stats_warn!("circuit", "{}", msg);
        } else {
            mac_stats_debug!("circuit", "{} (rate-limited)", msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_after_threshold() {
        let mut c = CircuitBreaker::new("test", 3, Duration::from_secs(30));
        assert!(c.allow_request().is_ok());
        c.record_failure(true);
        c.record_failure(true);
        assert_eq!(c.state(), CircuitState::Closed);
        c.record_failure(true);
        assert_eq!(c.state(), CircuitState::Open);
        assert!(c.allow_request().is_err());
    }

    #[test]
    fn half_open_probe_then_close() {
        let mut c = CircuitBreaker::new("test", 2, Duration::from_secs(0));
        c.record_failure(true);
        c.record_failure(true);
        assert_eq!(c.state(), CircuitState::Open);
        std::thread::sleep(Duration::from_millis(5));
        assert!(c.allow_request().is_ok());
        assert_eq!(c.state(), CircuitState::HalfOpen);
        c.record_success();
        assert_eq!(c.state(), CircuitState::Closed);
    }

    #[test]
    fn open_warn_first_and_after_interval() {
        let now = Instant::now();
        assert!(should_warn_circuit_open(None, now, Duration::from_secs(60)));
        assert!(!should_warn_circuit_open(
            Some(now),
            now + Duration::from_secs(30),
            Duration::from_secs(60)
        ));
        assert!(should_warn_circuit_open(
            Some(now),
            now + Duration::from_secs(60),
            Duration::from_secs(60)
        ));
    }

    #[test]
    fn half_open_reopen_rate_limits_warn_clock() {
        let mut c = CircuitBreaker::new("test", 1, Duration::from_secs(0));
        c.record_failure(true);
        assert_eq!(c.state(), CircuitState::Open);
        assert!(c.last_open_warn.is_some());
        let first = c.last_open_warn;
        std::thread::sleep(Duration::from_millis(5));
        assert!(c.allow_request().is_ok());
        assert_eq!(c.state(), CircuitState::HalfOpen);
        c.record_failure(true);
        assert_eq!(c.state(), CircuitState::Open);
        // Same warn clock — half-open reopen inside the interval must not reset last_open_warn.
        assert_eq!(c.last_open_warn, first);
    }
}
