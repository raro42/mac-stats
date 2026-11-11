//! Dashboard "Agent Ops" APIs + Discord operator instant lane.
//! Domain fragments live in `*.inc.rs` and are compiled as one module via
//! [`gateway`] (shared private helpers). See docs/ops/harness_ops_refactor.md.

pub mod normalize;
mod gateway;

pub use gateway::*;
pub use normalize::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_session_name() {
        let (src, slug) =
            parse_session_filename("session-memory-discord-20260720-181500-weather.md");
        assert_eq!(src, "discord");
        assert!(slug.contains("weather"));
    }

    #[test]
    fn keep_percentile_digit_safe() {
        assert_eq!(parse_keep_percentile("/keep-p47"), Some(47));
        assert_eq!(parse_keep_percentile("/keep-p4"), Some(4));
        assert_ne!(parse_keep_percentile("/keep-p47"), Some(4));
        assert_eq!(parse_keep_percentile("keep p47"), Some(47));
        assert_eq!(parse_keep_percentile("47th percentile keep gap"), Some(47));
        assert!(parse_keep_percentile("results.tsv path").is_none());
        assert!(parse_keep_percentile("/keep-pace").is_none());
        assert!(looks_like_keep_percentile_request("/keep-p90"));
    }

    #[test]
    fn resource_path_size_age_do_not_steal() {
        assert!(looks_like_debug_log_path_request("debug.log path"));
        assert!(looks_like_debug_log_size_request("debug.log size"));
        assert!(!looks_like_debug_log_path_request("debug.log size"));
        assert!(!looks_like_debug_log_size_request("debug.log path"));
        assert!(looks_like_results_tsv_path_request("results.tsv path"));
    }

    #[test]
    fn path_request_matchers_stay_fast_without_sibling_nesting() {
        let start = std::time::Instant::now();
        for _ in 0..2000 {
            let _ = looks_like_debug_log_path_request("hello world path maybe");
            let _ = looks_like_config_path_request("config somewhere");
            let _ = looks_like_disk_cleanup_path_request("cleanup path?");
        }
        assert!(
            start.elapsed().as_millis() < 500,
            "path matchers too slow — check for nested looks_like_* calls"
        );
    }

    #[test]
    fn operator_instant_reply_covers_gateway_commands() {
        assert!(try_operator_instant_reply("/status").is_some());
        assert!(try_operator_instant_reply("/keep-p47").is_some());
        assert!(try_operator_instant_reply("/keep-p4").is_some());
        assert!(try_operator_instant_reply("results.tsv path").is_some());
    }
}
