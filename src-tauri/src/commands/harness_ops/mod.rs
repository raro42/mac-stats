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

    #[test]
    fn ram_ring_chip_ask_does_not_steal_strip_ram() {
        assert_eq!(parse_ring_chip_ask("/ram-ring"), Some(RingChipAsk::Ram));
        assert_eq!(parse_ring_chip_ask("ram ring"), Some(RingChipAsk::Ram));
        assert_eq!(
            parse_ring_chip_ask("show me the ram ring"),
            Some(RingChipAsk::Ram)
        );
        assert_eq!(parse_ring_chip_ask("memory gauge"), Some(RingChipAsk::Ram));
        // Strip owns bare /ram · how much ram (power-strip chip).
        assert!(parse_ring_chip_ask("/ram").is_none());
        assert!(parse_ring_chip_ask("how much ram").is_none());
        assert!(parse_ring_chip_ask("ram").is_none());
        assert_eq!(parse_strip_chip_ask("/ram"), Some(StripChipAsk::Ram));
        assert_eq!(
            parse_strip_chip_ask("how much ram"),
            Some(StripChipAsk::Ram)
        );
        let reply = try_operator_instant_reply("ram ring");
        assert!(reply.is_some());
        assert!(reply.unwrap().contains("**RAM**"));
    }

    #[test]
    fn load_1m_chip_does_not_steal_full_load_panel() {
        // Explicit 1m window + view/open NL → focused Load chip (parity with load 5 / load 15).
        assert!(looks_like_load_chip_request("load 1"));
        assert!(looks_like_load_chip_request("load 1m"));
        assert!(looks_like_load_chip_request("1-minute load"));
        assert!(looks_like_load_chip_request("/load-1m"));
        assert!(looks_like_load_chip_request("/load1m"));
        assert!(looks_like_load_chip_request("1m load"));
        assert!(looks_like_load_chip_request("view load 1"));
        assert!(looks_like_load_chip_request("open load 1m"));
        assert!(looks_like_load_chip_request("show me the 1 minute load"));
        // Full Details panel still owns bare /load · view load · what's the load.
        assert!(!looks_like_load_chip_request("/load"));
        assert!(!looks_like_load_chip_request("view load"));
        assert!(!looks_like_load_chip_request("open load"));
        assert!(!looks_like_load_chip_request("what's the load"));
        assert!(looks_like_details_request("/load"));
        assert!(looks_like_details_request("view load"));
        assert!(looks_like_details_request("what's the load"));
        assert!(!looks_like_details_request("load 1"));
        assert!(!looks_like_details_request("1-minute load"));
        assert!(!looks_like_details_request("view load 1"));
        assert!(!looks_like_details_request("open load 1m"));
        let reply = try_operator_instant_reply("load 1");
        assert!(reply.is_some());
        let text = reply.unwrap();
        assert!(text.contains("**Load**"), "got: {text}");
        assert!(
            !text.contains("**Details**"),
            "load 1 must not expand full Details: {text}"
        );
        let viewed = try_operator_instant_reply("view load 1").unwrap();
        assert!(viewed.contains("**Load**"), "got: {viewed}");
        assert!(
            !viewed.contains("**Details**"),
            "view load 1 must not expand full Details: {viewed}"
        );
        let opened = try_operator_instant_reply("open load 1m").unwrap();
        assert!(opened.contains("**Load**"), "got: {opened}");
        assert!(
            !opened.contains("**Details**"),
            "open load 1m must not expand full Details: {opened}"
        );
        let full = try_operator_instant_reply("/load").unwrap();
        assert!(full.contains("**Details**"), "got: {full}");
    }

    #[test]
    fn load_5m_15m_slash_and_view_open_do_not_steal_full_load() {
        // Slash + view/open NL → focused 5m / 15m chips (Load 1m `/load-1m` parity).
        assert_eq!(
            parse_load_window_chip("/load-5m"),
            Some(LoadWindowChip::Five)
        );
        assert_eq!(
            parse_load_window_chip("/load-15m"),
            Some(LoadWindowChip::Fifteen)
        );
        assert_eq!(
            parse_load_window_chip("view load 5"),
            Some(LoadWindowChip::Five)
        );
        assert_eq!(
            parse_load_window_chip("open load 15m"),
            Some(LoadWindowChip::Fifteen)
        );
        assert_eq!(
            parse_load_window_chip("show me the 5 minute load"),
            Some(LoadWindowChip::Five)
        );
        // Bare view/open load still Details; why stays with the agent.
        assert!(parse_load_window_chip("view load").is_none());
        assert!(parse_load_window_chip("open load").is_none());
        assert!(parse_load_window_chip("/load").is_none());
        assert!(parse_load_window_chip("why is the 5 minute load high").is_none());
        assert!(looks_like_details_request("view load"));
        assert!(looks_like_details_request("/load"));
        assert!(!looks_like_details_request("view load 5"));
        assert!(!looks_like_details_request("/load-5m"));
        let five = try_operator_instant_reply("/load-5m").unwrap();
        assert!(five.contains("**Load 5m**"), "got: {five}");
        assert!(
            !five.contains("**Details**"),
            "/load-5m must not expand full Details: {five}"
        );
        let fifteen = try_operator_instant_reply("open load 15m").unwrap();
        assert!(fifteen.contains("**Load 15m**"), "got: {fifteen}");
        assert!(
            !fifteen.contains("**Details**"),
            "open load 15m must not expand full Details: {fifteen}"
        );
        let full = try_operator_instant_reply("view load").unwrap();
        assert!(full.contains("**Details**"), "got: {full}");
    }

    #[test]
    fn power_watts_slash_and_view_open_do_not_steal_full_strip() {
        // Slash + view/open NL → focused Power watts chip (Bat `/battery` shape).
        assert_eq!(parse_strip_chip_ask("/watts"), Some(StripChipAsk::Power));
        assert_eq!(
            parse_strip_chip_ask("/power-draw"),
            Some(StripChipAsk::Power)
        );
        assert_eq!(
            parse_strip_chip_ask("view watts"),
            Some(StripChipAsk::Power)
        );
        assert_eq!(
            parse_strip_chip_ask("open power draw"),
            Some(StripChipAsk::Power)
        );
        assert_eq!(
            parse_strip_chip_ask("show me the watts"),
            Some(StripChipAsk::Power)
        );
        // Bare /power · view power · power stay the full strip.
        assert!(parse_strip_chip_ask("/power").is_none());
        assert!(parse_strip_chip_ask("view power").is_none());
        assert!(parse_strip_chip_ask("power").is_none());
        assert!(looks_like_strip_request("/power"));
        assert!(looks_like_strip_request("view power"));
        assert!(!looks_like_strip_request("/watts"));
        assert!(!looks_like_strip_request("view watts"));
        let watts = try_operator_instant_reply("/watts").unwrap();
        assert!(watts.contains("**Power**"), "got: {watts}");
        assert!(
            !watts.contains("**Power strip**"),
            "/watts must not expand full strip: {watts}"
        );
        let open = try_operator_instant_reply("open power draw").unwrap();
        assert!(open.contains("**Power**"), "got: {open}");
        assert!(
            !open.contains("**Power strip**"),
            "open power draw must not expand full strip: {open}"
        );
        let full = try_operator_instant_reply("/power").unwrap();
        assert!(
            full.contains("**Power strip**"),
            "got: {full}"
        );
    }

    #[test]
    fn p_e_core_slash_and_view_open_do_not_steal_freq_ring() {
        // Slash + view/open NL → focused P/E-core chips (Load `/load-5m` parity).
        assert_eq!(
            parse_core_cluster_freq("/p-core"),
            Some(CoreClusterChip::Performance)
        );
        assert_eq!(
            parse_core_cluster_freq("/e-core"),
            Some(CoreClusterChip::Efficiency)
        );
        assert_eq!(
            parse_core_cluster_freq("view p core"),
            Some(CoreClusterChip::Performance)
        );
        assert_eq!(
            parse_core_cluster_freq("open e core"),
            Some(CoreClusterChip::Efficiency)
        );
        assert_eq!(
            parse_core_cluster_freq("list the p cores"),
            Some(CoreClusterChip::Performance)
        );
        assert_eq!(
            parse_core_cluster_freq("see e-core frequency"),
            Some(CoreClusterChip::Efficiency)
        );
        // Combined Freq ring still owns bare /freq · view freq · how fast is the cpu.
        assert!(parse_core_cluster_freq("/freq").is_none());
        assert!(parse_core_cluster_freq("view freq").is_none());
        assert!(parse_core_cluster_freq("how fast is the cpu").is_none());
        assert!(parse_core_cluster_freq("why is the p core high").is_none());
        assert_eq!(parse_ring_chip_ask("/freq"), Some(RingChipAsk::Freq));
        assert_eq!(parse_ring_chip_ask("view freq"), Some(RingChipAsk::Freq));
        let p = try_operator_instant_reply("/p-core").unwrap();
        assert!(p.contains("**P-core**"), "got: {p}");
        let e = try_operator_instant_reply("open e core").unwrap();
        assert!(e.contains("**E-core**"), "got: {e}");
        let freq = try_operator_instant_reply("view freq").unwrap();
        assert!(freq.contains("**Freq**") || freq.contains("**Frequency**") || freq.contains("GHz"), "got: {freq}");
    }
}
