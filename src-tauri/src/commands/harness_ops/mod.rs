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
        assert!(try_operator_instant_reply("/queue").is_some());
        assert!(try_operator_instant_reply("/session-queue").is_some());
        assert!(try_operator_instant_reply("session queue").is_some());
        assert!(try_operator_instant_reply("/keep-p47").is_some());
        assert!(try_operator_instant_reply("/keep-p4").is_some());
        assert!(try_operator_instant_reply("results.tsv path").is_some());
    }

    #[test]
    fn session_queue_glance_openclaw_serialization() {
        assert!(looks_like_session_queue_request("/queue"));
        assert!(looks_like_session_queue_request("view session queue"));
        assert!(looks_like_session_queue_request("is the queue busy"));
        // Digester open stays put.
        assert!(!looks_like_session_queue_request("digest queue"));
        assert!(!looks_like_session_queue_request("why is the queue stuck"));
        let idle = crate::keyed_queue::SessionQueueSnapshot {
            tracked_session_keys: 0,
            busy_session_keys: 0,
            waiting_session_refs: 0,
            busy_key_sample: vec![],
        };
        assert_eq!(
            format_session_queue_line(&idle),
            "Session queue: **queue clear**"
        );
        let busy = crate::keyed_queue::SessionQueueSnapshot {
            tracked_session_keys: 3,
            busy_session_keys: 2,
            waiting_session_refs: 1,
            busy_key_sample: vec!["discord:1".into(), "discord:2".into()],
        };
        let line = format_session_queue_line(&busy);
        assert!(line.contains("busy **2/3**"), "{line}");
        assert!(line.contains("wait **1**"), "{line}");
        assert!(line.contains("discord:1"), "{line}");
        let busy_no_wait = crate::keyed_queue::SessionQueueSnapshot {
            tracked_session_keys: 1,
            busy_session_keys: 1,
            waiting_session_refs: 0,
            busy_key_sample: vec!["discord:9".into()],
        };
        let line_nw = format_session_queue_line(&busy_no_wait);
        assert!(line_nw.contains("busy **1/1**"), "{line_nw}");
        assert!(
            !line_nw.contains("wait"),
            "idle wait bit must stay calm: {line_nw}"
        );
        let status = format_status_gateway();
        assert!(
            status.contains("Session queue:"),
            "status must include session queue glance: {status}"
        );
        let focused = try_operator_instant_reply("/queue").expect("queue instant");
        assert!(focused.contains("Session queue:"), "{focused}");
    }

    #[test]
    fn browser_ready_chip_session_tab_glance() {
        use crate::browser_agent::CachedBrowserSessionGlance;
        let absent = format_browser_ready_chip_with_glance(
            9222,
            "Google Chrome",
            CachedBrowserSessionGlance::Absent,
            0,
            0,
            None,
            None,
            None,
        );
        assert!(
            absent.contains("Ready") && absent.contains("idle until BROWSER_*"),
            "{absent}"
        );
        assert!(!absent.contains("tracked"), "{absent}");
        assert!(!absent.contains("owner"), "{absent}");
        assert!(!absent.contains("last fail"), "{absent}");
        let busy = format_browser_ready_chip_with_glance(
            9222,
            "Google Chrome",
            CachedBrowserSessionGlance::Busy,
            0,
            0,
            None,
            None,
            None,
        );
        assert!(
            busy.contains("Busy") && busy.contains("session locked"),
            "{busy}"
        );
        let warm = format_browser_ready_chip_with_glance(
            9222,
            "Google Chrome",
            CachedBrowserSessionGlance::Warm {
                tab_count: 3,
                focus_index: 1,
                focus_host: Some("example.com".into()),
                up_secs: 300,
                idle_secs: 45,
                headless: true,
                chrome_pid: Some(4242),
            },
            0,
            0,
            None,
            None,
            Some("connection refused"),
        );
        assert!(warm.contains("Warm"), "{warm}");
        assert!(warm.contains("tabs **3**"), "{warm}");
        assert!(warm.contains("focus **1**"), "{warm}");
        assert!(warm.contains("page **example.com**"), "{warm}");
        assert!(warm.contains("headless"), "{warm}");
        assert!(warm.contains("up 5m"), "{warm}");
        assert!(warm.contains("idle 45s"), "{warm}");
        assert!(warm.contains("pid **4242**"), "{warm}");
        assert!(!warm.contains("tracked"), "{warm}");
        assert!(
            !warm.contains("last fail"),
            "Warm omits stale last-fail: {warm}"
        );
        let warm_m = format_browser_ready_chip_with_glance(
            9222,
            "Chromium",
            CachedBrowserSessionGlance::Warm {
                tab_count: 1,
                focus_index: 0,
                focus_host: None,
                up_secs: 90,
                idle_secs: 120,
                headless: false,
                chrome_pid: None,
            },
            2,
            1,
            Some(125),
            None,
            None,
        );
        assert!(warm_m.contains("visible"), "{warm_m}");
        assert!(warm_m.contains("up 1m"), "{warm_m}");
        assert!(warm_m.contains("idle 2m"), "{warm_m}");
        assert!(!warm_m.contains("pid"), "{warm_m}");
        assert!(
            !warm_m.contains("page "),
            "blank focus host stays calm: {warm_m}"
        );
        assert!(warm_m.contains("tracked **2**"), "{warm_m}");
        assert!(warm_m.contains("oldest 2m"), "{warm_m}");
        assert!(
            !warm_m.contains("sess"),
            "single-session tracked stays calm: {warm_m}"
        );
        let multi_sess = format_browser_ready_chip_with_glance(
            9222,
            "Google Chrome",
            CachedBrowserSessionGlance::Absent,
            3,
            2,
            Some(40),
            None,
            None,
        );
        assert!(
            multi_sess.contains("tracked **3**")
                && multi_sess.contains("**2** sess")
                && multi_sess.contains("oldest 40s"),
            "{multi_sess}"
        );
        let absent_tracked = format_browser_ready_chip_with_glance(
            9222,
            "Google Chrome",
            CachedBrowserSessionGlance::Absent,
            1,
            1,
            Some(12),
            None,
            None,
        );
        assert!(
            absent_tracked.contains("idle until BROWSER_*")
                && absent_tracked.contains("tracked **1**")
                && absent_tracked.contains("oldest 12s")
                && !absent_tracked.contains("sess"),
            "{absent_tracked}"
        );
        // Count without age stays calm (omit oldest).
        let tracked_no_age = format_browser_ready_chip_with_glance(
            9222,
            "Google Chrome",
            CachedBrowserSessionGlance::Absent,
            3,
            1,
            None,
            None,
            None,
        );
        assert!(
            tracked_no_age.contains("tracked **3**") && !tracked_no_age.contains("oldest"),
            "{tracked_no_age}"
        );
        // Active owner (OpenClaw ActiveBrowserSessionGuard) — distinct from tracked/Warm.
        let with_owner = format_browser_ready_chip_with_glance(
            9222,
            "Google Chrome",
            CachedBrowserSessionGlance::Warm {
                tab_count: 1,
                focus_index: 0,
                focus_host: Some("docs.example.org".into()),
                up_secs: 10,
                idle_secs: 2,
                headless: true,
                chrome_pid: None,
            },
            1,
            1,
            Some(5),
            Some("discord-42"),
            None,
        );
        assert!(
            with_owner.contains("page **docs.example.org**"),
            "{with_owner}"
        );
        assert!(
            with_owner.contains("owner **discord-42**")
                && with_owner.contains("tracked **1**")
                && !with_owner.contains("sess"),
            "{with_owner}"
        );
        let owner_only = format_browser_ready_chip_with_glance(
            9222,
            "Google Chrome",
            CachedBrowserSessionGlance::Absent,
            0,
            0,
            None,
            Some("local-0"),
            None,
        );
        assert!(
            owner_only.contains("owner **local-0**") && !owner_only.contains("tracked"),
            "{owner_only}"
        );
        // OpenClaw detectError leftover — last CDP probe fail on Ready (omit when Warm).
        let ready_fail = format_browser_ready_chip_with_glance(
            9222,
            "Google Chrome",
            CachedBrowserSessionGlance::Absent,
            0,
            0,
            None,
            None,
            Some("connection refused"),
        );
        assert!(
            ready_fail.contains("Ready")
                && ready_fail.contains("last fail **connection refused**"),
            "{ready_fail}"
        );
        let busy_fail = format_browser_ready_chip_with_glance(
            9222,
            "Google Chrome",
            CachedBrowserSessionGlance::Busy,
            0,
            0,
            None,
            None,
            Some("ws attach failed"),
        );
        assert!(
            busy_fail.contains("Busy") && busy_fail.contains("last fail **ws attach failed**"),
            "{busy_fail}"
        );
    }

    #[test]
    fn live_session_idle_near_compaction_threshold() {
        use chrono::{Duration, Utc};
        let fresh = Utc::now().to_rfc3339();
        assert!(
            !live_session_idle_from_rfc3339(&fresh),
            "fresh activity must not be idle"
        );
        let old = (Utc::now() - Duration::minutes(31)).to_rfc3339();
        assert!(
            live_session_idle_from_rfc3339(&old),
            "31m past threshold must be idle"
        );
        let edge = (Utc::now() - Duration::minutes(30)).to_rfc3339();
        assert!(
            live_session_idle_from_rfc3339(&edge),
            "exactly 30m must be idle"
        );
        assert!(
            !live_session_idle_from_rfc3339("not-a-ts"),
            "bad timestamps stay calm"
        );
        assert_eq!(
            format_sessions_title(SessionsListFilter::All, 2, 3, 0),
            "**Sessions** — 2 live · 3 files"
        );
        assert_eq!(
            format_sessions_title(SessionsListFilter::All, 2, 3, 1),
            "**Sessions** — 2 live · 3 files · idle **1**"
        );
        assert_eq!(
            format_sessions_title(SessionsListFilter::Live, 4, 0, 2),
            "**Sessions · Live** — 4 · idle **2**"
        );
        assert_eq!(
            format_sessions_title(SessionsListFilter::Files, 0, 5, 9),
            "**Sessions · Files** — 5"
        );
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
    fn cpu_gpu_freq_temp_ring_slash_does_not_steal_full_rings() {
        // Singular * ring / /*-ring → focused chip (RAM `/ram-ring` parity).
        assert_eq!(parse_ring_chip_ask("/cpu-ring"), Some(RingChipAsk::Cpu));
        assert_eq!(parse_ring_chip_ask("cpu ring"), Some(RingChipAsk::Cpu));
        assert_eq!(
            parse_ring_chip_ask("view cpu ring"),
            Some(RingChipAsk::Cpu)
        );
        assert_eq!(parse_ring_chip_ask("/gpu-ring"), Some(RingChipAsk::Gpu));
        assert_eq!(
            parse_ring_chip_ask("open gpu ring"),
            Some(RingChipAsk::Gpu)
        );
        assert_eq!(parse_ring_chip_ask("/freq-ring"), Some(RingChipAsk::Freq));
        assert_eq!(
            parse_ring_chip_ask("view frequency ring"),
            Some(RingChipAsk::Freq)
        );
        assert_eq!(parse_ring_chip_ask("/temp-ring"), Some(RingChipAsk::Temp));
        assert_eq!(
            parse_ring_chip_ask("open temp ring"),
            Some(RingChipAsk::Temp)
        );
        // Plural cpu rings / view cpu rings stay the full `/rings` panel.
        assert!(parse_ring_chip_ask("cpu rings").is_none());
        assert!(parse_ring_chip_ask("view cpu rings").is_none());
        assert!(looks_like_rings_request("cpu rings"));
        assert!(looks_like_rings_request("view cpu rings"));
        assert!(!looks_like_rings_request("cpu ring"));
        assert!(!looks_like_rings_request("/cpu-ring"));
        let cpu = try_operator_instant_reply("/cpu-ring").unwrap();
        assert!(cpu.contains("**CPU**"), "got: {cpu}");
        assert!(
            !cpu.contains("**CPU rings**"),
            "/cpu-ring must not expand full rings: {cpu}"
        );
        let gpu = try_operator_instant_reply("open gpu ring").unwrap();
        assert!(gpu.contains("**GPU**"), "got: {gpu}");
        let full = try_operator_instant_reply("view cpu rings").unwrap();
        assert!(
            full.contains("**CPU rings**"),
            "got: {full}"
        );
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
    fn ssd_storage_slash_and_view_open_do_not_steal_disk_cleanup() {
        // `/storage` · view/open storage → focused SSD chip (watts `/watts` parity).
        // Bare open disk / view disk stay Disk Cleanup.
        assert_eq!(parse_strip_chip_ask("/storage"), Some(StripChipAsk::Ssd));
        assert_eq!(parse_strip_chip_ask("storage"), Some(StripChipAsk::Ssd));
        assert_eq!(
            parse_strip_chip_ask("view storage"),
            Some(StripChipAsk::Ssd)
        );
        assert_eq!(
            parse_strip_chip_ask("open the storage"),
            Some(StripChipAsk::Ssd)
        );
        assert_eq!(
            parse_strip_chip_ask("show me the storage"),
            Some(StripChipAsk::Ssd)
        );
        assert_eq!(parse_strip_chip_ask("/ssd"), Some(StripChipAsk::Ssd));
        assert!(parse_strip_chip_ask("view disk").is_none());
        assert!(parse_strip_chip_ask("open disk").is_none());
        assert!(looks_like_disk_cleanup_request("view disk"));
        assert!(looks_like_disk_cleanup_request("open disk"));
        assert!(!looks_like_disk_cleanup_request("/storage"));
        assert!(!looks_like_disk_cleanup_request("view storage"));
        let storage = try_operator_instant_reply("/storage").unwrap();
        assert!(storage.contains("**SSD**"), "got: {storage}");
        let viewed = try_operator_instant_reply("view storage").unwrap();
        assert!(viewed.contains("**SSD**"), "got: {viewed}");
        let cleanup = try_operator_instant_reply("view disk").unwrap();
        assert!(
            cleanup.to_lowercase().contains("disk") || cleanup.contains("Cleanup") || cleanup.contains("cleanup") || cleanup.contains("**Disk"),
            "view disk should stay Disk Cleanup: {cleanup}"
        );
    }

    #[test]
    fn ram_capacity_slash_and_view_open_do_not_steal_ram_percent() {
        // Slash + view/open NL → installed RAM (SSD `/free` · watts `/watts` parity).
        // Bare /ram · how much ram · view ram stay the % chip.
        // Also: `ram size` / `how big is ram` must beat execution_prompt size nouns.
        assert!(looks_like_ram_capacity_request("/ram-size"));
        assert!(looks_like_ram_capacity_request("/ram-capacity"));
        assert!(looks_like_ram_capacity_request("/memory-size"));
        assert!(looks_like_ram_capacity_request("/total-ram"));
        assert!(looks_like_ram_capacity_request("view ram size"));
        assert!(looks_like_ram_capacity_request("open ram capacity"));
        assert!(looks_like_ram_capacity_request("list memory capacity"));
        assert!(looks_like_ram_capacity_request("how big is ram"));
        assert!(!looks_like_ram_capacity_request("/ram"));
        assert!(!looks_like_ram_capacity_request("how much ram"));
        assert!(!looks_like_ram_capacity_request("view ram"));
        assert!(!looks_like_ram_capacity_request("how much ram is used"));
        assert_eq!(parse_strip_chip_ask("/ram"), Some(StripChipAsk::Ram));
        assert_eq!(parse_strip_chip_ask("view ram"), Some(StripChipAsk::Ram));
        let cap = try_operator_instant_reply("/ram-size").unwrap();
        assert!(
            cap.contains("**RAM**") && !cap.to_lowercase().contains("execution"),
            "got: {cap}"
        );
        let classic = try_operator_instant_reply("ram size").unwrap();
        assert!(
            classic.contains("**RAM**") && !classic.to_lowercase().contains("execution"),
            "ram size must not hit execution_prompt: {classic}"
        );
        let viewed = try_operator_instant_reply("view ram size").unwrap();
        assert!(
            viewed.contains("**RAM**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let pct = try_operator_instant_reply("/ram").unwrap();
        assert!(pct.contains("**RAM**"), "got: {pct}");
        assert!(
            !pct.to_lowercase().contains("installed"),
            "bare /ram must stay % chip, not capacity: {pct}"
        );
    }

    #[test]
    fn app_uptime_slash_and_view_open_do_not_steal_mac_up_chip() {
        // Slash + view/open NL → process clock (chip `/chip` · RAM `/ram-size` parity).
        // Bare /uptime · view uptime · what's the uptime stay the Mac Up chip.
        assert!(looks_like_app_uptime_request("/app-uptime"));
        assert!(looks_like_app_uptime_request("/process-uptime"));
        assert!(looks_like_app_uptime_request("/bot-uptime"));
        assert!(looks_like_app_uptime_request("/mac-stats-uptime"));
        assert!(looks_like_app_uptime_request("view app uptime"));
        assert!(looks_like_app_uptime_request("open process uptime"));
        assert!(looks_like_app_uptime_request("list bot uptime"));
        assert!(looks_like_app_uptime_request("app uptime"));
        assert!(looks_like_app_uptime_request("how long have you been running"));
        assert!(!looks_like_app_uptime_request("/uptime"));
        assert!(!looks_like_app_uptime_request("uptime"));
        assert!(!looks_like_app_uptime_request("view uptime"));
        assert!(!looks_like_app_uptime_request("what's the uptime"));
        assert!(!looks_like_app_uptime_request("how long has the mac been up"));
        assert_eq!(parse_strip_chip_ask("/uptime"), Some(StripChipAsk::Uptime));
        assert_eq!(parse_strip_chip_ask("view uptime"), Some(StripChipAsk::Uptime));
        let app = try_operator_instant_reply("/app-uptime").unwrap();
        assert!(
            app.contains("**App up**") && !app.to_lowercase().contains("execution"),
            "got: {app}"
        );
        let viewed = try_operator_instant_reply("view app uptime").unwrap();
        assert!(
            viewed.contains("**App up**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let mac = try_operator_instant_reply("/uptime").unwrap();
        assert!(
            !mac.contains("**App up**"),
            "bare /uptime must stay Mac Up chip, not process clock: {mac}"
        );
    }

    #[test]
    fn chip_identity_slash_and_view_open_do_not_steal_cpu_ring_or_ollama_model() {
        // Slash + view/open NL → Apple chip name (RAM `/ram-size` · SSD `/free` parity).
        // Bare /cpu · view cpu · what's the cpu stay the CPU usage ring; /model stays Ollama.
        assert!(looks_like_chip_identity_request("/chip"));
        assert!(looks_like_chip_identity_request("/chip-name"));
        assert!(looks_like_chip_identity_request("/processor"));
        assert!(looks_like_chip_identity_request("/cpu-name"));
        assert!(looks_like_chip_identity_request("/apple-chip"));
        assert!(looks_like_chip_identity_request("view chip"));
        assert!(looks_like_chip_identity_request("open processor"));
        assert!(looks_like_chip_identity_request("list cpu name"));
        assert!(looks_like_chip_identity_request("what chip is this"));
        assert!(!looks_like_chip_identity_request("/cpu"));
        assert!(!looks_like_chip_identity_request("view cpu"));
        assert!(!looks_like_chip_identity_request("what's the cpu"));
        assert!(!looks_like_chip_identity_request("/model"));
        assert!(!looks_like_chip_identity_request("which model"));
        assert_eq!(parse_ring_chip_ask("/cpu"), Some(RingChipAsk::Cpu));
        assert_eq!(parse_ring_chip_ask("view cpu"), Some(RingChipAsk::Cpu));
        let chip = try_operator_instant_reply("/chip").unwrap();
        assert!(
            chip.contains("**Chip**") && !chip.to_lowercase().contains("execution"),
            "got: {chip}"
        );
        let viewed = try_operator_instant_reply("view chip").unwrap();
        assert!(
            viewed.contains("**Chip**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let cpu = try_operator_instant_reply("/cpu").unwrap();
        assert!(
            !cpu.contains("**Chip**"),
            "bare /cpu must stay CPU ring, not chip name: {cpu}"
        );
    }

    #[test]
    fn macos_version_slash_and_view_open_do_not_steal_app_version() {
        // Slash + view/open NL → macOS sw_vers (chip `/chip` · RAM `/ram-size` parity).
        // Bare /version · view version · what version stay app status health.
        assert!(looks_like_macos_version_request("/macos"));
        assert!(looks_like_macos_version_request("/os"));
        assert!(looks_like_macos_version_request("/os-version"));
        assert!(looks_like_macos_version_request("/macos-version"));
        assert!(looks_like_macos_version_request("/system-version"));
        assert!(looks_like_macos_version_request("/sw-vers"));
        assert!(looks_like_macos_version_request("/build"));
        assert!(looks_like_macos_version_request("view macos"));
        assert!(looks_like_macos_version_request("open os version"));
        assert!(looks_like_macos_version_request("list system version"));
        assert!(looks_like_macos_version_request("macos version"));
        assert!(looks_like_macos_version_request("what os is this"));
        assert!(!looks_like_macos_version_request("/version"));
        assert!(!looks_like_macos_version_request("view version"));
        assert!(!looks_like_macos_version_request("what version"));
        assert!(!looks_like_macos_version_request("app version"));
        assert!(looks_like_status_request("/version"));
        assert!(looks_like_status_request("view version"));
        let os = try_operator_instant_reply("/macos").unwrap();
        assert!(
            os.contains("**macOS**") && !os.to_lowercase().contains("execution"),
            "got: {os}"
        );
        let viewed = try_operator_instant_reply("view macos").unwrap();
        assert!(
            viewed.contains("**macOS**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let app = try_operator_instant_reply("/version").unwrap();
        assert!(
            !app.contains("**macOS**"),
            "bare /version must stay app status, not macOS: {app}"
        );
    }

    #[test]
    fn hostname_slash_and_view_open_do_not_steal_bare_host() {
        // Slash + view/open NL → ComputerName · LocalHostName (chip `/chip` · macOS `/macos` parity).
        // Bare host / view host stay with the agent (too ambiguous for monitors/URLs).
        assert!(looks_like_hostname_request("/hostname"));
        assert!(looks_like_hostname_request("/computer-name"));
        assert!(looks_like_hostname_request("/machine-name"));
        assert!(looks_like_hostname_request("/local-hostname"));
        assert!(looks_like_hostname_request("view hostname"));
        assert!(looks_like_hostname_request("open computer name"));
        assert!(looks_like_hostname_request("list machine name"));
        assert!(looks_like_hostname_request("hostname"));
        assert!(looks_like_hostname_request("computer name"));
        assert!(looks_like_hostname_request("what's my hostname"));
        assert!(looks_like_hostname_request("what computer name"));
        assert!(!looks_like_hostname_request("/host"));
        assert!(!looks_like_hostname_request("host"));
        assert!(!looks_like_hostname_request("view host"));
        assert!(!looks_like_hostname_request("localhost"));
        assert!(!looks_like_hostname_request("monitor hostname"));
        let host = try_operator_instant_reply("/hostname").unwrap();
        assert!(
            host.contains("**Host**") && !host.to_lowercase().contains("execution"),
            "got: {host}"
        );
        let viewed = try_operator_instant_reply("view hostname").unwrap();
        assert!(
            viewed.contains("**Host**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("view host").is_none());
        assert!(try_operator_instant_reply("/host").is_none());
    }

    #[test]
    fn mac_model_slash_and_view_open_do_not_steal_ollama_model() {
        // Slash + view/open NL → Mac marketing name · machine model (chip `/chip` · hostname parity).
        // Bare /model · which model · model name stay Ollama Ready.
        assert!(looks_like_mac_model_request("/mac-model"));
        assert!(looks_like_mac_model_request("/hardware-model"));
        assert!(looks_like_mac_model_request("/machine-model"));
        assert!(looks_like_mac_model_request("/model-id"));
        assert!(looks_like_mac_model_request("/hw-model"));
        assert!(looks_like_mac_model_request("view mac model"));
        assert!(looks_like_mac_model_request("open hardware model"));
        assert!(looks_like_mac_model_request("list machine model"));
        assert!(looks_like_mac_model_request("mac model"));
        assert!(looks_like_mac_model_request("what mac is this"));
        assert!(looks_like_mac_model_request("what macbook"));
        assert!(looks_like_mac_model_request("hardware model"));
        assert!(!looks_like_mac_model_request("/model"));
        assert!(!looks_like_mac_model_request("which model"));
        assert!(!looks_like_mac_model_request("model name"));
        assert!(!looks_like_mac_model_request("view model"));
        assert!(!looks_like_mac_model_request("/chip"));
        assert!(looks_like_ollama_ready_request("/model"));
        assert!(looks_like_ollama_ready_request("which model"));
        let mac = try_operator_instant_reply("/mac-model").unwrap();
        assert!(
            mac.contains("**Mac**") && !mac.to_lowercase().contains("execution"),
            "got: {mac}"
        );
        let viewed = try_operator_instant_reply("view mac model").unwrap();
        assert!(
            viewed.contains("**Mac**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let ollama = try_operator_instant_reply("/model").unwrap();
        assert!(
            !ollama.contains("**Mac**"),
            "bare /model must stay Ollama, not Mac model: {ollama}"
        );
    }

    #[test]
    fn username_slash_and_view_open_do_not_steal_bare_user_or_user_info() {
        // Slash + view/open NL → login username (hostname `/hostname` · chip `/chip` parity).
        // Bare user / view user stay with the agent; user info path/size/age stay Discord map.
        assert!(looks_like_username_request("/username"));
        assert!(looks_like_username_request("/whoami"));
        assert!(looks_like_username_request("/login-name"));
        assert!(looks_like_username_request("/account-name"));
        assert!(looks_like_username_request("view username"));
        assert!(looks_like_username_request("open whoami"));
        assert!(looks_like_username_request("list login name"));
        assert!(looks_like_username_request("username"));
        assert!(looks_like_username_request("whoami"));
        assert!(looks_like_username_request("who am i"));
        assert!(looks_like_username_request("what's my username"));
        assert!(looks_like_username_request("login name"));
        assert!(!looks_like_username_request("/user"));
        assert!(!looks_like_username_request("user"));
        assert!(!looks_like_username_request("view user"));
        assert!(!looks_like_username_request("user info"));
        assert!(!looks_like_username_request("user info path"));
        assert!(!looks_like_username_request("user-info.json"));
        let user = try_operator_instant_reply("/username").unwrap();
        assert!(
            user.contains("**User**") && !user.to_lowercase().contains("execution"),
            "got: {user}"
        );
        let viewed = try_operator_instant_reply("view username").unwrap();
        assert!(
            viewed.contains("**User**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let who = try_operator_instant_reply("whoami").unwrap();
        assert!(
            who.contains("**User**") && !who.to_lowercase().contains("execution"),
            "got: {who}"
        );
        assert!(try_operator_instant_reply("view user").is_none());
        assert!(try_operator_instant_reply("/user").is_none());
        // Discord map file stays on its resource-stat lane (not this instant username reply).
        let info = try_operator_instant_reply("user info path");
        assert!(
            info.as_ref().is_none_or(|r| !r.contains("**User** · `")),
            "user info path must not become login username: {info:?}"
        );
    }

    #[test]
    fn serial_slash_and_view_open_do_not_steal_bare_sn_or_model() {
        // Slash + view/open NL → hardware serial (username `/username` · Mac model parity).
        // Bare sn stays with the agent; /mac-model stays marketing name; /chip stays chip.
        assert!(looks_like_serial_request("/serial"));
        assert!(looks_like_serial_request("/serial-number"));
        assert!(looks_like_serial_request("/hw-serial"));
        assert!(looks_like_serial_request("/hardware-serial"));
        assert!(looks_like_serial_request("/device-serial"));
        assert!(looks_like_serial_request("/sn"));
        assert!(looks_like_serial_request("view serial"));
        assert!(looks_like_serial_request("open serial number"));
        assert!(looks_like_serial_request("list serial"));
        assert!(looks_like_serial_request("serial"));
        assert!(looks_like_serial_request("serial number"));
        assert!(looks_like_serial_request("what's my serial"));
        assert!(looks_like_serial_request("what serial is this"));
        assert!(!looks_like_serial_request("sn"));
        assert!(!looks_like_serial_request("serialize"));
        assert!(!looks_like_serial_request("serial port"));
        assert!(!looks_like_serial_request("/mac-model"));
        assert!(!looks_like_serial_request("/chip"));
        let serial = try_operator_instant_reply("/serial").unwrap();
        assert!(
            serial.contains("**Serial**") && !serial.to_lowercase().contains("execution"),
            "got: {serial}"
        );
        let viewed = try_operator_instant_reply("view serial").unwrap();
        assert!(
            viewed.contains("**Serial**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("sn").is_none());
        let mac = try_operator_instant_reply("/mac-model").unwrap();
        assert!(
            !mac.contains("**Serial**"),
            "/mac-model must stay Mac model, not serial: {mac}"
        );
    }

#[test]
    fn timezone_slash_and_view_open_do_not_steal_wall_clock_time() {
        // Slash + view/open NL → local timezone (username `/username` · hostname parity).
        // Bare `/time` / what time / view time stay with the agent.
        assert!(looks_like_timezone_request("/timezone"));
        assert!(looks_like_timezone_request("/tz"));
        assert!(looks_like_timezone_request("/time-zone"));
        assert!(looks_like_timezone_request("/local-timezone"));
        assert!(looks_like_timezone_request("/local-tz"));
        assert!(looks_like_timezone_request("view timezone"));
        assert!(looks_like_timezone_request("open tz"));
        assert!(looks_like_timezone_request("list time zone"));
        assert!(looks_like_timezone_request("timezone"));
        assert!(looks_like_timezone_request("time zone"));
        assert!(looks_like_timezone_request("what's my timezone"));
        assert!(looks_like_timezone_request("what timezone am i in"));
        assert!(!looks_like_timezone_request("/time"));
        assert!(!looks_like_timezone_request("time"));
        assert!(!looks_like_timezone_request("what time"));
        assert!(!looks_like_timezone_request("what time is it"));
        assert!(!looks_like_timezone_request("view time"));
        assert!(!looks_like_timezone_request("current time"));
        let tz = try_operator_instant_reply("/timezone").unwrap();
        assert!(
            tz.contains("**Timezone**") && !tz.to_lowercase().contains("execution"),
            "got: {tz}"
        );
        let viewed = try_operator_instant_reply("view timezone").unwrap();
        assert!(
            viewed.contains("**Timezone**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/tz").unwrap();
        assert!(
            short.contains("**Timezone**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("/time").is_none());
        assert!(try_operator_instant_reply("what time").is_none());
        assert!(try_operator_instant_reply("view time").is_none());
    }

#[test]
    fn architecture_slash_and_view_open_do_not_steal_chip_or_architect_theme() {
        // Slash + view/open NL → machine arch (timezone `/tz` · serial `/serial` parity).
        // Bare architect / theme Architect stay out; /chip stays Apple chip.
        assert!(looks_like_architecture_request("/arch"));
        assert!(looks_like_architecture_request("/architecture"));
        assert!(looks_like_architecture_request("/uname-m"));
        assert!(looks_like_architecture_request("/machine-arch"));
        assert!(looks_like_architecture_request("/cpu-arch"));
        assert!(looks_like_architecture_request("/hw-arch"));
        assert!(looks_like_architecture_request("view arch"));
        assert!(looks_like_architecture_request("open architecture"));
        assert!(looks_like_architecture_request("list arch"));
        assert!(looks_like_architecture_request("arch"));
        assert!(looks_like_architecture_request("architecture"));
        assert!(looks_like_architecture_request("cpu architecture"));
        assert!(looks_like_architecture_request("what's my arch"));
        assert!(looks_like_architecture_request("what architecture is this"));
        assert!(looks_like_architecture_request("am i on arm64"));
        assert!(!looks_like_architecture_request("architect"));
        assert!(!looks_like_architecture_request("theme architect"));
        assert!(!looks_like_architecture_request("what chip"));
        assert!(!looks_like_architecture_request("/chip"));
        assert!(!looks_like_architecture_request("software architecture"));
        let arch = try_operator_instant_reply("/arch").unwrap();
        assert!(
            arch.contains("**Arch**") && !arch.to_lowercase().contains("execution"),
            "got: {arch}"
        );
        let viewed = try_operator_instant_reply("view architecture").unwrap();
        assert!(
            viewed.contains("**Arch**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/uname-m").unwrap();
        assert!(
            short.contains("**Arch**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("architect").is_none());
        let chip = try_operator_instant_reply("/chip");
        if let Some(c) = chip {
            assert!(
                !c.contains("**Arch**"),
                "/chip must stay chip, not arch: {c}"
            );
        }
    }

#[test]
    fn locale_slash_and_view_open_do_not_steal_bare_lang_or_timezone() {
        // Slash + view/open NL → system locale (timezone `/tz` · arch `/arch` parity).
        // Bare `lang` / programming language / language model stay with the agent; `/tz` stays timezone.
        assert!(looks_like_locale_request("/locale"));
        assert!(looks_like_locale_request("/lang"));
        assert!(looks_like_locale_request("/language"));
        assert!(looks_like_locale_request("/apple-locale"));
        assert!(looks_like_locale_request("/system-locale"));
        assert!(looks_like_locale_request("/system-language"));
        assert!(looks_like_locale_request("view locale"));
        assert!(looks_like_locale_request("open language"));
        assert!(looks_like_locale_request("list locale"));
        assert!(looks_like_locale_request("locale"));
        assert!(looks_like_locale_request("language"));
        assert!(looks_like_locale_request("system language"));
        assert!(looks_like_locale_request("what's my locale"));
        assert!(looks_like_locale_request("what language is this"));
        assert!(!looks_like_locale_request("lang"));
        assert!(!looks_like_locale_request("programming language"));
        assert!(!looks_like_locale_request("language model"));
        assert!(!looks_like_locale_request("/tz"));
        assert!(!looks_like_locale_request("timezone"));
        let loc = try_operator_instant_reply("/locale").unwrap();
        assert!(
            loc.contains("**Locale**") && !loc.to_lowercase().contains("execution"),
            "got: {loc}"
        );
        let viewed = try_operator_instant_reply("view language").unwrap();
        assert!(
            viewed.contains("**Locale**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/lang").unwrap();
        assert!(
            short.contains("**Locale**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("lang").is_none());
        let tz = try_operator_instant_reply("/tz").unwrap();
        assert!(
            tz.contains("**Timezone**") && !tz.contains("**Locale**"),
            "/tz must stay timezone, not locale: {tz}"
        );
    }

#[test]
    fn kernel_slash_and_view_open_do_not_steal_uname_or_arch() {
        // Slash + view/open NL → kernel release (arch `/arch` · locale `/locale` parity).
        // Bare `uname` / `/uname` stay with the agent; `/uname-m` stays arch; `/macos` stays OS.
        assert!(looks_like_kernel_request("/kernel"));
        assert!(looks_like_kernel_request("/kernel-release"));
        assert!(looks_like_kernel_request("/kernel-version"));
        assert!(looks_like_kernel_request("/uname-r"));
        assert!(looks_like_kernel_request("/kern"));
        assert!(looks_like_kernel_request("view kernel"));
        assert!(looks_like_kernel_request("open kernel release"));
        assert!(looks_like_kernel_request("list kernel"));
        assert!(looks_like_kernel_request("kernel"));
        assert!(looks_like_kernel_request("kernel release"));
        assert!(looks_like_kernel_request("kernel version"));
        assert!(looks_like_kernel_request("what's my kernel"));
        assert!(looks_like_kernel_request("what kernel is this"));
        assert!(looks_like_kernel_request("uname -r"));
        assert!(!looks_like_kernel_request("uname"));
        assert!(!looks_like_kernel_request("/uname"));
        assert!(!looks_like_kernel_request("/uname-m"));
        assert!(!looks_like_kernel_request("kernel panic"));
        assert!(!looks_like_kernel_request("kernel module"));
        let kern = try_operator_instant_reply("/kernel").unwrap();
        assert!(
            kern.contains("**Kernel**") && !kern.to_lowercase().contains("execution"),
            "got: {kern}"
        );
        let viewed = try_operator_instant_reply("view kernel").unwrap();
        assert!(
            viewed.contains("**Kernel**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/uname-r").unwrap();
        assert!(
            short.contains("**Kernel**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("uname").is_none());
        assert!(try_operator_instant_reply("/uname").is_none());
        let arch = try_operator_instant_reply("/uname-m").unwrap();
        assert!(
            arch.contains("**Arch**") && !arch.contains("**Kernel**"),
            "/uname-m must stay arch, not kernel: {arch}"
        );
        let arch2 = try_operator_instant_reply("/arch").unwrap();
        assert!(
            arch2.contains("**Arch**") && !arch2.contains("**Kernel**"),
            "/arch must stay arch, not kernel: {arch2}"
        );
    }

#[test]
    fn cpu_cores_slash_and_view_open_do_not_steal_p_core_or_chip() {
        // Slash + view/open NL → core count (RAM `/ram-size` · locale `/locale` parity).
        // `/p-core` · `/e-core` stay cluster clocks; `/chip` stays name; bare `core` / `/cpu` stay out.
        assert!(looks_like_cpu_cores_request("/cores"));
        assert!(looks_like_cpu_cores_request("/ncpu"));
        assert!(looks_like_cpu_cores_request("/cpu-count"));
        assert!(looks_like_cpu_cores_request("/core-count"));
        assert!(looks_like_cpu_cores_request("/cpu-cores"));
        assert!(looks_like_cpu_cores_request("/logical-cpus"));
        assert!(looks_like_cpu_cores_request("/hw-ncpu"));
        assert!(looks_like_cpu_cores_request("view cores"));
        assert!(looks_like_cpu_cores_request("open ncpu"));
        assert!(looks_like_cpu_cores_request("list cpu cores"));
        assert!(looks_like_cpu_cores_request("cores"));
        assert!(looks_like_cpu_cores_request("core count"));
        assert!(looks_like_cpu_cores_request("how many cores"));
        assert!(looks_like_cpu_cores_request("what's the core count"));
        assert!(!looks_like_cpu_cores_request("core"));
        assert!(!looks_like_cpu_cores_request("/p-core"));
        assert!(!looks_like_cpu_cores_request("/e-cores"));
        assert!(!looks_like_cpu_cores_request("p cores"));
        assert!(!looks_like_cpu_cores_request("e cores"));
        assert!(!looks_like_cpu_cores_request("/chip"));
        assert!(!looks_like_cpu_cores_request("/cpu"));
        assert!(!looks_like_cpu_cores_request("view cpu"));
        let cores = try_operator_instant_reply("/cores").unwrap();
        assert!(
            cores.contains("**Cores**") && !cores.to_lowercase().contains("execution"),
            "got: {cores}"
        );
        let viewed = try_operator_instant_reply("view cores").unwrap();
        assert!(
            viewed.contains("**Cores**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/ncpu").unwrap();
        assert!(
            short.contains("**Cores**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("core").is_none());
        let p = try_operator_instant_reply("/p-core");
        if let Some(reply) = p {
            assert!(
                !reply.contains("**Cores**"),
                "/p-core must stay cluster clock, not cores: {reply}"
            );
        }
        let chip = try_operator_instant_reply("/chip");
        if let Some(c) = chip {
            assert!(
                !c.contains("**Cores**"),
                "/chip must stay chip, not cores: {c}"
            );
        }
    }

#[test]
    fn boot_time_slash_and_view_open_do_not_steal_uptime_or_app_uptime() {
        // Slash + view/open NL → last boot wall-clock (cores `/cores` · kernel `/kernel` parity).
        // Bare `/uptime` / `view uptime` stay Mac Up; `/app-uptime` stays process; bare `boot` stays out.
        assert!(looks_like_boot_time_request("/boot"));
        assert!(looks_like_boot_time_request("/boot-time"));
        assert!(looks_like_boot_time_request("/last-boot"));
        assert!(looks_like_boot_time_request("/last-reboot"));
        assert!(looks_like_boot_time_request("/reboot-time"));
        assert!(looks_like_boot_time_request("view boot time"));
        assert!(looks_like_boot_time_request("open last reboot"));
        assert!(looks_like_boot_time_request("list boot time"));
        assert!(looks_like_boot_time_request("boot time"));
        assert!(looks_like_boot_time_request("last reboot"));
        assert!(looks_like_boot_time_request("when did this mac boot"));
        assert!(looks_like_boot_time_request("when was the last reboot"));
        assert!(looks_like_boot_time_request("what's the boot time"));
        assert!(!looks_like_boot_time_request("boot"));
        assert!(!looks_like_boot_time_request("reboot"));
        assert!(!looks_like_boot_time_request("/uptime"));
        assert!(!looks_like_boot_time_request("view uptime"));
        assert!(!looks_like_boot_time_request("what's the uptime"));
        assert!(!looks_like_boot_time_request("boot disk"));
        assert!(!looks_like_boot_time_request("boot volume"));
        assert!(!looks_like_boot_time_request("/app-uptime"));
        let boot = try_operator_instant_reply("/boot").unwrap();
        assert!(
            boot.contains("**Boot**") && !boot.to_lowercase().contains("execution"),
            "got: {boot}"
        );
        let viewed = try_operator_instant_reply("view boot time").unwrap();
        assert!(
            viewed.contains("**Boot**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/boot-time").unwrap();
        assert!(
            short.contains("**Boot**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("boot").is_none());
        let up = try_operator_instant_reply("/uptime");
        if let Some(reply) = up {
            assert!(
                !reply.contains("**Boot**"),
                "/uptime must stay Mac Up duration, not boot: {reply}"
            );
        }
        let app = try_operator_instant_reply("/app-uptime");
        if let Some(reply) = app {
            assert!(
                !reply.contains("**Boot**"),
                "/app-uptime must stay process uptime, not boot: {reply}"
            );
        }
    }

#[test]
    fn local_ip_slash_and_view_open_do_not_steal_public_ip_asks() {
        // Slash + view/open NL → local LAN IPv4 (boot `/boot` · hostname `/hostname` parity).
        // Bare `ip` / `what's my ip` stay with the agent (often public/WAN).
        assert!(looks_like_local_ip_request("/ip"));
        assert!(looks_like_local_ip_request("/local-ip"));
        assert!(looks_like_local_ip_request("/lan-ip"));
        assert!(looks_like_local_ip_request("/ipv4"));
        assert!(looks_like_local_ip_request("/primary-ip"));
        assert!(looks_like_local_ip_request("/en0"));
        assert!(looks_like_local_ip_request("view local ip"));
        assert!(looks_like_local_ip_request("open lan ip"));
        assert!(looks_like_local_ip_request("list ipv4"));
        assert!(looks_like_local_ip_request("local ip"));
        assert!(looks_like_local_ip_request("lan ip"));
        assert!(looks_like_local_ip_request("what's my local ip"));
        assert!(looks_like_local_ip_request("view ip"));
        assert!(!looks_like_local_ip_request("ip"));
        assert!(!looks_like_local_ip_request("my ip"));
        assert!(!looks_like_local_ip_request("what's my ip"));
        assert!(!looks_like_local_ip_request("what is my ip"));
        assert!(!looks_like_local_ip_request("public ip"));
        assert!(!looks_like_local_ip_request("external ip"));
        assert!(!looks_like_local_ip_request("wan ip"));
        let ip = try_operator_instant_reply("/ip").unwrap();
        assert!(
            ip.contains("**IP**") && !ip.to_lowercase().contains("execution"),
            "got: {ip}"
        );
        let viewed = try_operator_instant_reply("view local ip").unwrap();
        assert!(
            viewed.contains("**IP**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/local-ip").unwrap();
        assert!(
            short.contains("**IP**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("ip").is_none());
        assert!(try_operator_instant_reply("what's my ip").is_none());
        let host = try_operator_instant_reply("/hostname");
        if let Some(reply) = host {
            assert!(
                !reply.contains("**IP**"),
                "/hostname must stay hostname, not IP: {reply}"
            );
        }
    }

    #[test]
    fn default_gateway_slash_and_view_open_do_not_steal_discord_or_lan_ip() {
        // Slash + view/open NL → default IPv4 gateway (`/ip` parity).
        // Discord gateway status/ready stay Discord; bare gateway/router stay with agent; /ip stays LAN.
        assert!(looks_like_default_gateway_request("/gateway"));
        assert!(looks_like_default_gateway_request("/default-gateway"));
        assert!(looks_like_default_gateway_request("/router"));
        assert!(looks_like_default_gateway_request("/gw"));
        assert!(looks_like_default_gateway_request("/default-route"));
        assert!(looks_like_default_gateway_request("/router-ip"));
        assert!(looks_like_default_gateway_request("/gateway-ip"));
        assert!(looks_like_default_gateway_request("view default gateway"));
        assert!(looks_like_default_gateway_request("open router"));
        assert!(looks_like_default_gateway_request("list gateway"));
        assert!(looks_like_default_gateway_request("default gateway"));
        assert!(looks_like_default_gateway_request("router ip"));
        assert!(looks_like_default_gateway_request("what's my gateway"));
        assert!(looks_like_default_gateway_request("what's the default gateway"));
        assert!(!looks_like_default_gateway_request("gateway"));
        assert!(!looks_like_default_gateway_request("router"));
        assert!(!looks_like_default_gateway_request("gateway status"));
        assert!(!looks_like_default_gateway_request("gateway ready"));
        assert!(!looks_like_default_gateway_request("discord gateway"));
        assert!(!looks_like_default_gateway_request("bot gateway"));
        assert!(!looks_like_default_gateway_request("/ip"));
        assert!(!looks_like_default_gateway_request("local ip"));
        let gw = try_operator_instant_reply("/gateway").unwrap();
        assert!(
            gw.contains("**Gateway**") && !gw.to_lowercase().contains("execution"),
            "got: {gw}"
        );
        let viewed = try_operator_instant_reply("view default gateway").unwrap();
        assert!(
            viewed.contains("**Gateway**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/router").unwrap();
        assert!(
            short.contains("**Gateway**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("gateway").is_none());
        assert!(try_operator_instant_reply("router").is_none());
        // Discord chip still owns gateway status (not default-route).
        let discord = try_operator_instant_reply("gateway status");
        if let Some(reply) = discord {
            assert!(
                !reply.contains("**Gateway** · `"),
                "gateway status must stay Discord, not default route: {reply}"
            );
        }
        let ip = try_operator_instant_reply("/ip");
        if let Some(reply) = ip {
            assert!(
                !reply.contains("**Gateway**"),
                "/ip must stay LAN IP, not default gateway: {reply}"
            );
        }
    }

    #[test]
    fn dns_servers_slash_and_view_open_do_not_steal_lookup_or_gateway() {
        // Slash + view/open NL → DNS nameservers (`/gateway` · `/ip` parity).
        // Bare `dns` / dig·lookup·flush stay with the agent; `/gateway` stays default route.
        assert!(looks_like_dns_servers_request("/dns"));
        assert!(looks_like_dns_servers_request("/dns-servers"));
        assert!(looks_like_dns_servers_request("/dns-server"));
        assert!(looks_like_dns_servers_request("/nameserver"));
        assert!(looks_like_dns_servers_request("/nameservers"));
        assert!(looks_like_dns_servers_request("/ns"));
        assert!(looks_like_dns_servers_request("/resolver"));
        assert!(looks_like_dns_servers_request("view dns servers"));
        assert!(looks_like_dns_servers_request("open nameserver"));
        assert!(looks_like_dns_servers_request("list dns"));
        assert!(looks_like_dns_servers_request("dns servers"));
        assert!(looks_like_dns_servers_request("dns server"));
        assert!(looks_like_dns_servers_request("what's my dns"));
        assert!(looks_like_dns_servers_request("what's my dns servers"));
        assert!(looks_like_dns_servers_request("which dns"));
        assert!(!looks_like_dns_servers_request("dns"));
        assert!(!looks_like_dns_servers_request("nameserver"));
        assert!(!looks_like_dns_servers_request("dns lookup"));
        assert!(!looks_like_dns_servers_request("dig google.com"));
        assert!(!looks_like_dns_servers_request("flush dns"));
        assert!(!looks_like_dns_servers_request("/gateway"));
        assert!(!looks_like_dns_servers_request("default gateway"));
        assert!(!looks_like_dns_servers_request("/ip"));
        let dns = try_operator_instant_reply("/dns").unwrap();
        assert!(
            dns.contains("**DNS**") && !dns.to_lowercase().contains("execution"),
            "got: {dns}"
        );
        let viewed = try_operator_instant_reply("view dns servers").unwrap();
        assert!(
            viewed.contains("**DNS**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/nameserver").unwrap();
        assert!(
            short.contains("**DNS**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("dns").is_none());
        let gw = try_operator_instant_reply("/gateway");
        if let Some(reply) = gw {
            assert!(
                !reply.contains("**DNS**"),
                "/gateway must stay default route, not DNS: {reply}"
            );
        }
        let ip = try_operator_instant_reply("/ip");
        if let Some(reply) = ip {
            assert!(
                !reply.contains("**DNS**"),
                "/ip must stay LAN IP, not DNS: {reply}"
            );
        }
    }

    #[test]
    fn wifi_ssid_slash_and_view_open_do_not_steal_password_or_lan() {
        // Slash + view/open NL → current Wi-Fi SSID (`/dns` · `/gateway` · `/ip` parity).
        // Bare `wifi` / `ssid` / password·settings stay with the agent; `/ip` stays LAN.
        assert!(looks_like_wifi_ssid_request("/wifi"));
        assert!(looks_like_wifi_ssid_request("/wi-fi"));
        assert!(looks_like_wifi_ssid_request("/ssid"));
        assert!(looks_like_wifi_ssid_request("/wifi-ssid"));
        assert!(looks_like_wifi_ssid_request("/wifi-network"));
        assert!(looks_like_wifi_ssid_request("/airport"));
        assert!(looks_like_wifi_ssid_request("/wlan"));
        assert!(looks_like_wifi_ssid_request("view wifi"));
        assert!(looks_like_wifi_ssid_request("open wifi network"));
        assert!(looks_like_wifi_ssid_request("list ssid"));
        assert!(looks_like_wifi_ssid_request("wifi network"));
        assert!(looks_like_wifi_ssid_request("wifi name"));
        assert!(looks_like_wifi_ssid_request("what's my wifi"));
        assert!(looks_like_wifi_ssid_request("what's my ssid"));
        assert!(looks_like_wifi_ssid_request("which wifi"));
        assert!(looks_like_wifi_ssid_request("what wifi am i on"));
        assert!(!looks_like_wifi_ssid_request("wifi"));
        assert!(!looks_like_wifi_ssid_request("ssid"));
        assert!(!looks_like_wifi_ssid_request("wifi password"));
        assert!(!looks_like_wifi_ssid_request("wifi settings"));
        assert!(!looks_like_wifi_ssid_request("enable wifi"));
        assert!(!looks_like_wifi_ssid_request("/ip"));
        assert!(!looks_like_wifi_ssid_request("/dns"));
        assert!(!looks_like_wifi_ssid_request("/gateway"));
        let wifi = try_operator_instant_reply("/wifi").unwrap();
        assert!(
            wifi.contains("**Wi-Fi**") && !wifi.to_lowercase().contains("execution"),
            "got: {wifi}"
        );
        let viewed = try_operator_instant_reply("view wifi").unwrap();
        assert!(
            viewed.contains("**Wi-Fi**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/ssid").unwrap();
        assert!(
            short.contains("**Wi-Fi**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("wifi").is_none());
        assert!(try_operator_instant_reply("ssid").is_none());
        let ip = try_operator_instant_reply("/ip");
        if let Some(reply) = ip {
            assert!(
                !reply.contains("**Wi-Fi**"),
                "/ip must stay LAN IP, not Wi-Fi: {reply}"
            );
        }
        let dns = try_operator_instant_reply("/dns");
        if let Some(reply) = dns {
            assert!(
                !reply.contains("**Wi-Fi**"),
                "/dns must stay nameservers, not Wi-Fi: {reply}"
            );
        }
    }

#[test]
    fn mac_address_slash_and_view_open_do_not_steal_mac_model_or_lan_ip() {
        // Slash + view/open NL → primary MAC (`/dns` · `/ip` parity).
        // Bare `mac` / `ether` stay with the agent; `/mac-model` · `what's my mac` stay model.
        assert!(looks_like_mac_address_request("/mac-address"));
        assert!(looks_like_mac_address_request("/mac-addr"));
        assert!(looks_like_mac_address_request("/ether"));
        assert!(looks_like_mac_address_request("/ethernet"));
        assert!(looks_like_mac_address_request("/ethernet-address"));
        assert!(looks_like_mac_address_request("/hw-address"));
        assert!(looks_like_mac_address_request("/hardware-address"));
        assert!(looks_like_mac_address_request("/link-address"));
        assert!(looks_like_mac_address_request("/en0-mac"));
        assert!(looks_like_mac_address_request("view mac address"));
        assert!(looks_like_mac_address_request("open ether"));
        assert!(looks_like_mac_address_request("list ethernet"));
        assert!(looks_like_mac_address_request("mac address"));
        assert!(looks_like_mac_address_request("ethernet address"));
        assert!(looks_like_mac_address_request("what's my mac address"));
        assert!(looks_like_mac_address_request("which mac address"));
        assert!(!looks_like_mac_address_request("mac"));
        assert!(!looks_like_mac_address_request("ether"));
        assert!(!looks_like_mac_address_request("ethernet"));
        assert!(!looks_like_mac_address_request("what's my mac"));
        assert!(!looks_like_mac_address_request("what mac is this"));
        assert!(!looks_like_mac_address_request("/mac-model"));
        assert!(!looks_like_mac_address_request("mac model"));
        assert!(!looks_like_mac_address_request("/serial"));
        assert!(!looks_like_mac_address_request("/ip"));
        assert!(!looks_like_mac_address_request("/dns"));
        assert!(!looks_like_mac_address_request("/wifi"));
        assert!(!looks_like_mac_address_request("what's my wifi"));
        let mac = try_operator_instant_reply("/mac-address").unwrap();
        assert!(
            mac.contains("**MAC**") && !mac.to_lowercase().contains("execution"),
            "got: {mac}"
        );
        let viewed = try_operator_instant_reply("view mac address").unwrap();
        assert!(
            viewed.contains("**MAC**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/ether").unwrap();
        assert!(
            short.contains("**MAC**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("mac").is_none());
        assert!(try_operator_instant_reply("ether").is_none());
        let model = try_operator_instant_reply("/mac-model");
        if let Some(reply) = model {
            assert!(
                !reply.contains("**MAC** · `"),
                "/mac-model must stay marketing name, not ethernet MAC: {reply}"
            );
        }
        let whats_mac = try_operator_instant_reply("what's my mac");
        if let Some(reply) = whats_mac {
            assert!(
                !reply.contains("**MAC** · `"),
                "what's my mac must stay Mac model, not ethernet MAC: {reply}"
            );
        }
        let ip = try_operator_instant_reply("/ip");
        if let Some(reply) = ip {
            assert!(
                !reply.contains("**MAC**"),
                "/ip must stay LAN IP, not MAC: {reply}"
            );
        }
    }

#[test]
    fn mtu_slash_and_view_open_do_not_steal_mac_or_lan_ip() {
        // Slash + view/open NL → primary MTU (`/mac-address` · `/ip` parity).
        // Bare `mtu` stays with the agent; `/mac-address` stays ethernet; `/ip` stays LAN.
        assert!(looks_like_mtu_request("/mtu"));
        assert!(looks_like_mtu_request("/if-mtu"));
        assert!(looks_like_mtu_request("/iface-mtu"));
        assert!(looks_like_mtu_request("/interface-mtu"));
        assert!(looks_like_mtu_request("/link-mtu"));
        assert!(looks_like_mtu_request("/en0-mtu"));
        assert!(looks_like_mtu_request("/network-mtu"));
        assert!(looks_like_mtu_request("view mtu"));
        assert!(looks_like_mtu_request("open interface mtu"));
        assert!(looks_like_mtu_request("list mtu"));
        assert!(looks_like_mtu_request("mtu size"));
        assert!(looks_like_mtu_request("interface mtu"));
        assert!(looks_like_mtu_request("what's my mtu"));
        assert!(looks_like_mtu_request("what's the mtu"));
        assert!(looks_like_mtu_request("which mtu"));
        assert!(!looks_like_mtu_request("mtu"));
        assert!(!looks_like_mtu_request("/mac-address"));
        assert!(!looks_like_mtu_request("mac address"));
        assert!(!looks_like_mtu_request("/ether"));
        assert!(!looks_like_mtu_request("/ip"));
        assert!(!looks_like_mtu_request("/wifi"));
        assert!(!looks_like_mtu_request("what's my wifi"));
        let mtu = try_operator_instant_reply("/mtu").unwrap();
        assert!(
            mtu.contains("**MTU**") && !mtu.to_lowercase().contains("execution"),
            "got: {mtu}"
        );
        let viewed = try_operator_instant_reply("view mtu").unwrap();
        assert!(
            viewed.contains("**MTU**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/if-mtu").unwrap();
        assert!(
            short.contains("**MTU**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("mtu").is_none());
        let mac = try_operator_instant_reply("/mac-address");
        if let Some(reply) = mac {
            assert!(
                !reply.contains("**MTU**"),
                "/mac-address must stay MAC, not MTU: {reply}"
            );
        }
        let ip = try_operator_instant_reply("/ip");
        if let Some(reply) = ip {
            assert!(
                !reply.contains("**MTU**"),
                "/ip must stay LAN IP, not MTU: {reply}"
            );
        }
    }

#[test]
    fn netmask_slash_and_view_open_do_not_steal_lan_ip_or_mac() {
        // Slash + view/open NL → primary IPv4 netmask (`/mac-address` · `/ip` parity).
        // Bare `netmask` / `subnet` / `mask` stay with the agent; `/ip` stays LAN; `/mac-address` stays ethernet.
        assert!(looks_like_netmask_request("/netmask"));
        assert!(looks_like_netmask_request("/subnet"));
        assert!(looks_like_netmask_request("/subnet-mask"));
        assert!(looks_like_netmask_request("/network-mask"));
        assert!(looks_like_netmask_request("/ip-netmask"));
        assert!(looks_like_netmask_request("/lan-netmask"));
        assert!(looks_like_netmask_request("/cidr"));
        assert!(looks_like_netmask_request("/prefix-len"));
        assert!(looks_like_netmask_request("/prefix-length"));
        assert!(looks_like_netmask_request("view netmask"));
        assert!(looks_like_netmask_request("open subnet"));
        assert!(looks_like_netmask_request("list cidr"));
        assert!(looks_like_netmask_request("subnet mask"));
        assert!(looks_like_netmask_request("network mask"));
        assert!(looks_like_netmask_request("what's my netmask"));
        assert!(looks_like_netmask_request("what's my subnet mask"));
        assert!(looks_like_netmask_request("which netmask"));
        assert!(!looks_like_netmask_request("netmask"));
        assert!(!looks_like_netmask_request("subnet"));
        assert!(!looks_like_netmask_request("mask"));
        assert!(!looks_like_netmask_request("cidr"));
        assert!(!looks_like_netmask_request("/ip"));
        assert!(!looks_like_netmask_request("/mac-address"));
        assert!(!looks_like_netmask_request("/wifi"));
        assert!(!looks_like_netmask_request("/gateway"));
        assert!(!looks_like_netmask_request("/dns"));
        assert!(!looks_like_netmask_request("/mtu"));
        let mask = try_operator_instant_reply("/netmask").unwrap();
        assert!(
            mask.contains("**Netmask**") && !mask.to_lowercase().contains("execution"),
            "got: {mask}"
        );
        let viewed = try_operator_instant_reply("view netmask").unwrap();
        assert!(
            viewed.contains("**Netmask**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/subnet").unwrap();
        assert!(
            short.contains("**Netmask**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("netmask").is_none());
        assert!(try_operator_instant_reply("subnet").is_none());
        let ip = try_operator_instant_reply("/ip");
        if let Some(reply) = ip {
            assert!(
                !reply.contains("**Netmask**"),
                "/ip must stay LAN IP, not netmask: {reply}"
            );
        }
        let mac = try_operator_instant_reply("/mac-address");
        if let Some(reply) = mac {
            assert!(
                !reply.contains("**Netmask**"),
                "/mac-address must stay ethernet, not netmask: {reply}"
            );
        }
    }

#[test]
    fn primary_iface_slash_and_view_open_do_not_steal_mtu_or_lan_ip() {
        // Slash + view/open NL → default-route iface name (`/ip` · `/mtu` parity).
        // Bare `iface` / `interface` stay with the agent; `/iface-mtu` stays MTU; `/en0` stays LAN.
        assert!(looks_like_primary_iface_request("/iface"));
        assert!(looks_like_primary_iface_request("/ifname"));
        assert!(looks_like_primary_iface_request("/if-name"));
        assert!(looks_like_primary_iface_request("/interface"));
        assert!(looks_like_primary_iface_request("/interface-name"));
        assert!(looks_like_primary_iface_request("/default-iface"));
        assert!(looks_like_primary_iface_request("/default-interface"));
        assert!(looks_like_primary_iface_request("/primary-iface"));
        assert!(looks_like_primary_iface_request("/primary-interface"));
        assert!(looks_like_primary_iface_request("/netif"));
        assert!(looks_like_primary_iface_request("/nic"));
        assert!(looks_like_primary_iface_request("view interface"));
        assert!(looks_like_primary_iface_request("open iface"));
        assert!(looks_like_primary_iface_request("list interface"));
        assert!(looks_like_primary_iface_request("default interface"));
        assert!(looks_like_primary_iface_request("primary interface"));
        assert!(looks_like_primary_iface_request("network interface"));
        assert!(looks_like_primary_iface_request("what's my interface"));
        assert!(looks_like_primary_iface_request("which interface"));
        assert!(looks_like_primary_iface_request("what interface am i on"));
        assert!(!looks_like_primary_iface_request("iface"));
        assert!(!looks_like_primary_iface_request("interface"));
        assert!(!looks_like_primary_iface_request("nic"));
        assert!(!looks_like_primary_iface_request("/iface-mtu"));
        assert!(!looks_like_primary_iface_request("/interface-mtu"));
        assert!(!looks_like_primary_iface_request("interface mtu"));
        assert!(!looks_like_primary_iface_request("/mtu"));
        assert!(!looks_like_primary_iface_request("/en0"));
        assert!(!looks_like_primary_iface_request("/ip"));
        assert!(!looks_like_primary_iface_request("/mac-address"));
        assert!(!looks_like_primary_iface_request("/netmask"));
        assert!(!looks_like_primary_iface_request("/wifi"));
        assert!(!looks_like_primary_iface_request("list interfaces"));
        let iface = try_operator_instant_reply("/iface").unwrap();
        assert!(
            iface.contains("**Iface**") && !iface.to_lowercase().contains("execution"),
            "got: {iface}"
        );
        let viewed = try_operator_instant_reply("view interface").unwrap();
        assert!(
            viewed.contains("**Iface**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/ifname").unwrap();
        assert!(
            short.contains("**Iface**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("iface").is_none());
        assert!(try_operator_instant_reply("interface").is_none());
        let mtu = try_operator_instant_reply("/iface-mtu");
        if let Some(reply) = mtu {
            assert!(
                !reply.contains("**Iface**"),
                "/iface-mtu must stay MTU, not iface name: {reply}"
            );
        }
        let ip = try_operator_instant_reply("/en0");
        if let Some(reply) = ip {
            assert!(
                !reply.contains("**Iface**"),
                "/en0 must stay LAN IP, not iface name: {reply}"
            );
        }
    }

#[test]
    fn broadcast_slash_and_view_open_do_not_steal_lan_ip_or_netmask() {
        // Slash + view/open NL → primary IPv4 broadcast (`/netmask` · `/ip` parity).
        // Bare `broadcast` / `bcast` stay with the agent; `/ip` stays LAN; `/netmask` stays mask.
        assert!(looks_like_broadcast_request("/broadcast"));
        assert!(looks_like_broadcast_request("/bcast"));
        assert!(looks_like_broadcast_request("/ip-broadcast"));
        assert!(looks_like_broadcast_request("/lan-broadcast"));
        assert!(looks_like_broadcast_request("/broadcast-addr"));
        assert!(looks_like_broadcast_request("/broadcast-address"));
        assert!(looks_like_broadcast_request("/ipv4-broadcast"));
        assert!(looks_like_broadcast_request("view broadcast"));
        assert!(looks_like_broadcast_request("open broadcast address"));
        assert!(looks_like_broadcast_request("list broadcast"));
        assert!(looks_like_broadcast_request("broadcast address"));
        assert!(looks_like_broadcast_request("ip broadcast"));
        assert!(looks_like_broadcast_request("what's my broadcast"));
        assert!(looks_like_broadcast_request("what's my broadcast address"));
        assert!(looks_like_broadcast_request("which broadcast"));
        assert!(!looks_like_broadcast_request("broadcast"));
        assert!(!looks_like_broadcast_request("bcast"));
        assert!(!looks_like_broadcast_request("/ip"));
        assert!(!looks_like_broadcast_request("/netmask"));
        assert!(!looks_like_broadcast_request("/subnet"));
        assert!(!looks_like_broadcast_request("/iface"));
        assert!(!looks_like_broadcast_request("/gateway"));
        assert!(!looks_like_broadcast_request("/mac-address"));
        assert!(!looks_like_broadcast_request("/mtu"));
        let bcast = try_operator_instant_reply("/broadcast").unwrap();
        assert!(
            bcast.contains("**Broadcast**") && !bcast.to_lowercase().contains("execution"),
            "got: {bcast}"
        );
        let viewed = try_operator_instant_reply("view broadcast").unwrap();
        assert!(
            viewed.contains("**Broadcast**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let short = try_operator_instant_reply("/bcast").unwrap();
        assert!(
            short.contains("**Broadcast**") && !short.to_lowercase().contains("execution"),
            "got: {short}"
        );
        assert!(try_operator_instant_reply("broadcast").is_none());
        assert!(try_operator_instant_reply("bcast").is_none());
        let ip = try_operator_instant_reply("/ip");
        if let Some(reply) = ip {
            assert!(
                !reply.contains("**Broadcast**"),
                "/ip must stay LAN IP, not broadcast: {reply}"
            );
        }
        let mask = try_operator_instant_reply("/netmask");
        if let Some(reply) = mask {
            assert!(
                !reply.contains("**Broadcast**"),
                "/netmask must stay subnet mask, not broadcast: {reply}"
            );
        }
    }

#[test]
    fn link_status_slash_and_view_open_do_not_steal_mtu_or_mac() {
        // Slash + view/open NL → primary link status · media (`/broadcast` · `/mtu` parity).
        // Bare `status` / `media` / `speed` / `link` stay with the agent; `/link-mtu` stays MTU; `/link-address` stays MAC.
        assert!(looks_like_link_status_request("/link-status"));
        assert!(looks_like_link_status_request("/link-state"));
        assert!(looks_like_link_status_request("/if-status"));
        assert!(looks_like_link_status_request("/iface-status"));
        assert!(looks_like_link_status_request("/interface-status"));
        assert!(looks_like_link_status_request("/media"));
        assert!(looks_like_link_status_request("/link-media"));
        assert!(looks_like_link_status_request("/link-speed"));
        assert!(looks_like_link_status_request("/if-speed"));
        assert!(looks_like_link_status_request("/network-speed"));
        assert!(looks_like_link_status_request("view link status"));
        assert!(looks_like_link_status_request("open link speed"));
        assert!(looks_like_link_status_request("list link status"));
        assert!(looks_like_link_status_request("link status"));
        assert!(looks_like_link_status_request("interface status"));
        assert!(looks_like_link_status_request("what's my link status"));
        assert!(looks_like_link_status_request("what's my link speed"));
        assert!(looks_like_link_status_request("is the link up"));
        assert!(!looks_like_link_status_request("status"));
        assert!(!looks_like_link_status_request("media"));
        assert!(!looks_like_link_status_request("speed"));
        assert!(!looks_like_link_status_request("link"));
        assert!(!looks_like_link_status_request("/link-mtu"));
        assert!(!looks_like_link_status_request("/link-address"));
        assert!(!looks_like_link_status_request("/mtu"));
        assert!(!looks_like_link_status_request("/mac-address"));
        assert!(!looks_like_link_status_request("/iface"));
        assert!(!looks_like_link_status_request("/broadcast"));
        assert!(!looks_like_link_status_request("/ip"));
        let link = try_operator_instant_reply("/link-status").unwrap();
        assert!(
            link.contains("**Link**") && !link.to_lowercase().contains("execution"),
            "got: {link}"
        );
        let viewed = try_operator_instant_reply("view link status").unwrap();
        assert!(
            viewed.contains("**Link**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let media = try_operator_instant_reply("/media").unwrap();
        assert!(
            media.contains("**Link**") && !media.to_lowercase().contains("execution"),
            "got: {media}"
        );
        // Bare `status` is app health (`/status`); must not become link status.
        if let Some(reply) = try_operator_instant_reply("status") {
            assert!(
                !reply.contains("**Link**"),
                "bare status must stay app health, not link: {reply}"
            );
        }
        assert!(try_operator_instant_reply("media").is_none());
        assert!(try_operator_instant_reply("speed").is_none());
        assert!(try_operator_instant_reply("link").is_none());
        let mtu = try_operator_instant_reply("/link-mtu");
        if let Some(reply) = mtu {
            assert!(
                !reply.contains("**Link**"),
                "/link-mtu must stay MTU, not link status: {reply}"
            );
        }
        let mac = try_operator_instant_reply("/link-address");
        if let Some(reply) = mac {
            assert!(
                !reply.contains("**Link**"),
                "/link-address must stay MAC, not link status: {reply}"
            );
        }
    }

#[test]
    fn battery_time_slash_and_view_open_do_not_steal_bat_percent_or_cycles() {
        // Slash + view/open NL → time-to-empty / full (`/cycles` · `/battery` parity).
        // Bare `time` / `remaining` stay with the agent; Bat % · cycles stay their lanes.
        assert!(looks_like_battery_time_request("/battery-time"));
        assert!(looks_like_battery_time_request("/bat-time"));
        assert!(looks_like_battery_time_request("/time-remaining"));
        assert!(looks_like_battery_time_request("/time-to-empty"));
        assert!(looks_like_battery_time_request("/time-to-full"));
        assert!(looks_like_battery_time_request("/eta-empty"));
        assert!(looks_like_battery_time_request("/eta-full"));
        assert!(looks_like_battery_time_request("view battery time"));
        assert!(looks_like_battery_time_request("open time remaining"));
        assert!(looks_like_battery_time_request("list battery time"));
        assert!(looks_like_battery_time_request("battery time"));
        assert!(looks_like_battery_time_request("time remaining"));
        assert!(looks_like_battery_time_request("time to empty"));
        assert!(looks_like_battery_time_request("how long until empty"));
        assert!(looks_like_battery_time_request("what's my battery time"));
        assert!(!looks_like_battery_time_request("time"));
        assert!(!looks_like_battery_time_request("remaining"));
        assert!(!looks_like_battery_time_request("/battery"));
        assert!(!looks_like_battery_time_request("view battery"));
        assert!(!looks_like_battery_time_request("battery left"));
        assert!(!looks_like_battery_time_request("battery remaining"));
        assert!(!looks_like_battery_time_request("/cycles"));
        assert!(!looks_like_battery_time_request("battery health"));
        assert!(!looks_like_battery_time_request("is it charging"));
        assert_eq!(parse_strip_chip_ask("battery time remaining"), None);
        assert_eq!(parse_strip_chip_ask("time to empty"), None);
        assert_eq!(parse_strip_chip_ask("battery left"), Some(StripChipAsk::Battery));
        let time = try_operator_instant_reply("/battery-time").unwrap();
        assert!(
            time.contains("**Battery time**") && !time.to_lowercase().contains("execution"),
            "got: {time}"
        );
        let viewed = try_operator_instant_reply("view battery time").unwrap();
        assert!(
            viewed.contains("**Battery time**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let bat = try_operator_instant_reply("/battery").unwrap();
        assert!(
            !bat.contains("**Battery time**"),
            "/battery must stay Bat %, not time: {bat}"
        );
        let cycles = try_operator_instant_reply("/cycles").unwrap();
        assert!(
            !cycles.contains("**Battery time**"),
            "/cycles must stay Cycles, not time: {cycles}"
        );
    }

    #[test]
    fn battery_cycles_slash_and_view_open_do_not_steal_bat_percent() {
        // Slash + view/open NL → cycle count · health · max capacity (`/battery` · `/chip` parity).
        // Bare `cycles` / `health` stay with the agent; `/battery` · view battery stay Bat %.
        assert!(looks_like_battery_cycles_request("/cycles"));
        assert!(looks_like_battery_cycles_request("/cycle-count"));
        assert!(looks_like_battery_cycles_request("/battery-cycles"));
        assert!(looks_like_battery_cycles_request("/battery-cycle-count"));
        assert!(looks_like_battery_cycles_request("/bat-cycles"));
        assert!(looks_like_battery_cycles_request("/battery-health"));
        assert!(looks_like_battery_cycles_request("/battery-condition"));
        assert!(looks_like_battery_cycles_request("/max-capacity"));
        assert!(looks_like_battery_cycles_request("/battery-max-capacity"));
        assert!(looks_like_battery_cycles_request("view cycle count"));
        assert!(looks_like_battery_cycles_request("open battery health"));
        assert!(looks_like_battery_cycles_request("list cycles"));
        assert!(looks_like_battery_cycles_request("cycle count"));
        assert!(looks_like_battery_cycles_request("battery cycles"));
        assert!(looks_like_battery_cycles_request("battery health"));
        assert!(looks_like_battery_cycles_request("what's my cycle count"));
        assert!(looks_like_battery_cycles_request("how many battery cycles"));
        assert!(!looks_like_battery_cycles_request("cycles"));
        assert!(!looks_like_battery_cycles_request("health"));
        assert!(!looks_like_battery_cycles_request("/battery"));
        assert!(!looks_like_battery_cycles_request("view battery"));
        assert!(!looks_like_battery_cycles_request("how much battery"));
        assert!(!looks_like_battery_cycles_request("battery charge"));
        assert!(!looks_like_battery_cycles_request("is it charging"));
        let cycles = try_operator_instant_reply("/cycles").unwrap();
        assert!(
            cycles.contains("**Cycles**") && !cycles.to_lowercase().contains("execution"),
            "got: {cycles}"
        );
        let viewed = try_operator_instant_reply("view cycle count").unwrap();
        assert!(
            viewed.contains("**Cycles**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let health = try_operator_instant_reply("/battery-health").unwrap();
        assert!(
            health.contains("**Cycles**") && !health.to_lowercase().contains("execution"),
            "got: {health}"
        );
        assert!(try_operator_instant_reply("cycles").is_none());
        let bat = try_operator_instant_reply("/battery");
        if let Some(reply) = bat {
            assert!(
                !reply.contains("**Cycles**"),
                "/battery must stay Bat %, not cycles: {reply}"
            );
        }
        let viewed_bat = try_operator_instant_reply("view battery");
        if let Some(reply) = viewed_bat {
            assert!(
                !reply.contains("**Cycles**"),
                "view battery must stay Bat %, not cycles: {reply}"
            );
        }
    }

#[test]
    fn vpn_status_slash_and_view_open_do_not_steal_wifi_or_signal() {
        // Slash + view/open NL → VPN status (`/wifi` · `/link-status` parity).
        // Bare `vpn` stays with the agent; `/wifi` stays SSID; `/signal` stays alerts; `/ip` stays LAN.
        assert!(looks_like_vpn_status_request("/vpn"));
        assert!(looks_like_vpn_status_request("/vpn-status"));
        assert!(looks_like_vpn_status_request("/vpn-state"));
        assert!(looks_like_vpn_status_request("/is-vpn"));
        assert!(looks_like_vpn_status_request("/on-vpn"));
        assert!(looks_like_vpn_status_request("view vpn"));
        assert!(looks_like_vpn_status_request("open vpn status"));
        assert!(looks_like_vpn_status_request("list vpn"));
        assert!(looks_like_vpn_status_request("vpn status"));
        assert!(looks_like_vpn_status_request("am i on vpn"));
        assert!(looks_like_vpn_status_request("is vpn connected"));
        assert!(looks_like_vpn_status_request("what's my vpn"));
        assert!(looks_like_vpn_status_request("what's my vpn status"));
        assert!(!looks_like_vpn_status_request("vpn"));
        assert!(!looks_like_vpn_status_request("/wifi"));
        assert!(!looks_like_vpn_status_request("view wifi"));
        assert!(!looks_like_vpn_status_request("/signal"));
        assert!(!looks_like_vpn_status_request("/ip"));
        assert!(!looks_like_vpn_status_request("/link-status"));
        let vpn = try_operator_instant_reply("/vpn").unwrap();
        assert!(
            vpn.contains("**VPN**") && !vpn.to_lowercase().contains("execution"),
            "got: {vpn}"
        );
        let viewed = try_operator_instant_reply("view vpn").unwrap();
        assert!(
            viewed.contains("**VPN**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("vpn").is_none());
        let wifi = try_operator_instant_reply("/wifi");
        if let Some(reply) = wifi {
            assert!(
                !reply.contains("**VPN**"),
                "/wifi must stay SSID, not VPN: {reply}"
            );
        }
    }

    #[test]
    fn hardware_uuid_slash_and_view_open_do_not_steal_serial_or_bare_uuid() {
        // Slash + view/open NL → hardware UUID (`/serial` parity).
        // Bare `uuid` stays with the agent; `/serial` stays serial; `/udid` stays with the agent.
        assert!(looks_like_hardware_uuid_request("/uuid"));
        assert!(looks_like_hardware_uuid_request("/hw-uuid"));
        assert!(looks_like_hardware_uuid_request("/hardware-uuid"));
        assert!(looks_like_hardware_uuid_request("/platform-uuid"));
        assert!(looks_like_hardware_uuid_request("/device-uuid"));
        assert!(looks_like_hardware_uuid_request("/mac-uuid"));
        assert!(looks_like_hardware_uuid_request("view uuid"));
        assert!(looks_like_hardware_uuid_request("open hardware uuid"));
        assert!(looks_like_hardware_uuid_request("list platform uuid"));
        assert!(looks_like_hardware_uuid_request("hardware uuid"));
        assert!(looks_like_hardware_uuid_request("what's my uuid"));
        assert!(looks_like_hardware_uuid_request("what's my hardware uuid"));
        assert!(!looks_like_hardware_uuid_request("uuid"));
        assert!(!looks_like_hardware_uuid_request("/serial"));
        assert!(!looks_like_hardware_uuid_request("view serial"));
        assert!(!looks_like_hardware_uuid_request("/udid"));
        assert!(!looks_like_hardware_uuid_request("udid"));
        assert!(!looks_like_hardware_uuid_request("generate uuid"));
        assert!(!looks_like_serial_request("/uuid"));
        assert!(!looks_like_serial_request("view uuid"));
        let id = try_operator_instant_reply("/uuid").unwrap();
        // Linux rack has no system_profiler/ioreg UUID — empty report is fine (no backticks).
        assert!(
            id.contains("**UUID**") && !id.to_lowercase().contains("execution"),
            "got: {id}"
        );
        let viewed = try_operator_instant_reply("view uuid").unwrap();
        assert!(
            viewed.contains("**UUID**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("uuid").is_none());
        let serial = try_operator_instant_reply("/serial");
        if let Some(reply) = serial {
            assert!(
                !reply.contains("**UUID**"),
                "/serial must stay serial, not UUID: {reply}"
            );
        }
    }

    #[test]
    fn boot_volume_slash_and_view_open_do_not_steal_boot_time_or_disk_cleanup() {
        // Slash + view/open NL → boot volume / startup disk (`/boot` · `/uuid` parity).
        // Bare `boot` / `volume` / `disk` stay with the agent; `/boot` stays boot time;
        // `view disk` stays Disk Cleanup; `/ssd` · `/free` stay capacity · free.
        assert!(looks_like_boot_volume_request("/boot-volume"));
        assert!(looks_like_boot_volume_request("/boot-disk"));
        assert!(looks_like_boot_volume_request("/startup-disk"));
        assert!(looks_like_boot_volume_request("/startup-volume"));
        assert!(looks_like_boot_volume_request("/volume-name"));
        assert!(looks_like_boot_volume_request("/boot-volume-name"));
        assert!(looks_like_boot_volume_request("view boot volume"));
        assert!(looks_like_boot_volume_request("open startup disk"));
        assert!(looks_like_boot_volume_request("list boot disk"));
        assert!(looks_like_boot_volume_request("boot volume"));
        assert!(looks_like_boot_volume_request("startup disk"));
        assert!(looks_like_boot_volume_request("what's my startup disk"));
        assert!(!looks_like_boot_volume_request("boot"));
        assert!(!looks_like_boot_volume_request("volume"));
        assert!(!looks_like_boot_volume_request("disk"));
        assert!(!looks_like_boot_volume_request("/boot"));
        assert!(!looks_like_boot_volume_request("view boot"));
        assert!(!looks_like_boot_volume_request("view boot time"));
        assert!(!looks_like_boot_volume_request("view disk"));
        assert!(!looks_like_boot_volume_request("/ssd"));
        assert!(!looks_like_boot_volume_request("/free"));
        assert!(!looks_like_boot_time_request("/boot-volume"));
        assert!(!looks_like_boot_time_request("view boot volume"));
        assert!(!looks_like_boot_time_request("startup disk"));
        assert!(!looks_like_disk_cleanup_request("/boot-volume"));
        assert!(!looks_like_disk_cleanup_request("view boot volume"));
        let vol = try_operator_instant_reply("/boot-volume").unwrap();
        assert!(
            vol.contains("**Boot volume**") && !vol.to_lowercase().contains("execution"),
            "got: {vol}"
        );
        let viewed = try_operator_instant_reply("view boot volume").unwrap();
        assert!(
            viewed.contains("**Boot volume**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        if let Some(reply) = try_operator_instant_reply("boot") {
            assert!(
                !reply.contains("**Boot volume**"),
                "bare boot must not be boot volume: {reply}"
            );
        }
        let boot = try_operator_instant_reply("/boot");
        if let Some(reply) = boot {
            assert!(
                !reply.contains("**Boot volume**"),
                "/boot must stay boot time, not volume: {reply}"
            );
        }
        let cleanup = try_operator_instant_reply("view disk");
        if let Some(reply) = cleanup {
            assert!(
                !reply.contains("**Boot volume**"),
                "view disk must stay Disk Cleanup, not boot volume: {reply}"
            );
        }
    }

    #[test]
    fn filevault_slash_and_view_open_do_not_steal_boot_volume_or_bare_encryption() {
        // Slash + view/open NL → FileVault (`/boot-volume` · `/uuid` parity).
        // Bare `filevault` / `fv` / `encryption` stay with the agent; `/boot-volume` stays volume;
        // `view disk` stays Disk Cleanup; `/ssd` · `/free` stay capacity · free.
        assert!(looks_like_filevault_request("/filevault"));
        assert!(looks_like_filevault_request("/file-vault"));
        assert!(looks_like_filevault_request("/fv"));
        assert!(looks_like_filevault_request("/fv-status"));
        assert!(looks_like_filevault_request("/fde"));
        assert!(looks_like_filevault_request("/disk-encryption"));
        assert!(looks_like_filevault_request("/encryption-status"));
        assert!(looks_like_filevault_request("view filevault"));
        assert!(looks_like_filevault_request("open filevault status"));
        assert!(looks_like_filevault_request("list encryption"));
        assert!(looks_like_filevault_request("filevault status"));
        assert!(looks_like_filevault_request("is filevault on"));
        assert!(looks_like_filevault_request("what's my filevault"));
        assert!(!looks_like_filevault_request("filevault"));
        assert!(!looks_like_filevault_request("fv"));
        assert!(!looks_like_filevault_request("encryption"));
        assert!(!looks_like_filevault_request("disk encryption"));
        assert!(!looks_like_filevault_request("/boot-volume"));
        assert!(!looks_like_filevault_request("view boot volume"));
        assert!(!looks_like_filevault_request("view disk"));
        assert!(!looks_like_filevault_request("/ssd"));
        assert!(!looks_like_filevault_request("/free"));
        assert!(!looks_like_filevault_request("enable filevault"));
        assert!(!looks_like_boot_volume_request("/filevault"));
        assert!(!looks_like_boot_volume_request("view filevault"));
        assert!(!looks_like_disk_cleanup_request("/filevault"));
        assert!(!looks_like_disk_cleanup_request("view filevault"));
        let fv = try_operator_instant_reply("/filevault").unwrap();
        assert!(
            fv.contains("**FileVault**") && !fv.to_lowercase().contains("execution"),
            "got: {fv}"
        );
        let viewed = try_operator_instant_reply("view filevault").unwrap();
        assert!(
            viewed.contains("**FileVault**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("filevault").is_none());
        assert!(try_operator_instant_reply("encryption").is_none());
        let vol = try_operator_instant_reply("/boot-volume");
        if let Some(reply) = vol {
            assert!(
                !reply.contains("**FileVault**"),
                "/boot-volume must stay boot volume, not FileVault: {reply}"
            );
        }
        let cleanup = try_operator_instant_reply("view disk");
        if let Some(reply) = cleanup {
            assert!(
                !reply.contains("**FileVault**"),
                "view disk must stay Disk Cleanup, not FileVault: {reply}"
            );
        }
    }

    #[test]
    fn sip_slash_and_view_open_do_not_steal_filevault_or_bare_sip() {
        // Slash + view/open NL → SIP (`/filevault` · `/uuid` parity).
        // Bare `sip` / `csrutil` stay with the agent; `/filevault` stays FileVault; `/gatekeeper` stays Gatekeeper.
        assert!(looks_like_sip_request("/sip"));
        assert!(looks_like_sip_request("/sip-status"));
        assert!(looks_like_sip_request("/csrutil"));
        assert!(looks_like_sip_request("/csrutil-status"));
        assert!(looks_like_sip_request("/system-integrity"));
        assert!(looks_like_sip_request("/system-integrity-protection"));
        assert!(looks_like_sip_request("/sip-protection"));
        assert!(looks_like_sip_request("view sip"));
        assert!(looks_like_sip_request("open sip status"));
        assert!(looks_like_sip_request("list sip"));
        assert!(looks_like_sip_request("sip status"));
        assert!(looks_like_sip_request("is sip on"));
        assert!(looks_like_sip_request("is sip enabled"));
        assert!(looks_like_sip_request("what's my sip"));
        assert!(looks_like_sip_request("view system integrity"));
        assert!(!looks_like_sip_request("sip"));
        assert!(!looks_like_sip_request("csrutil"));
        assert!(!looks_like_sip_request("system integrity"));
        assert!(!looks_like_sip_request("system integrity protection"));
        assert!(!looks_like_sip_request("/filevault"));
        assert!(!looks_like_sip_request("view filevault"));
        assert!(!looks_like_sip_request("enable sip"));
        assert!(!looks_like_sip_request("disable sip"));
        assert!(!looks_like_sip_request("gatekeeper"));
        assert!(!looks_like_sip_request("view gatekeeper"));
        assert!(!looks_like_filevault_request("/sip"));
        assert!(!looks_like_filevault_request("view sip"));
        assert!(!looks_like_filevault_request("/csrutil"));
        let sip = try_operator_instant_reply("/sip").unwrap();
        assert!(
            sip.contains("**SIP**") && !sip.to_lowercase().contains("execution"),
            "got: {sip}"
        );
        let viewed = try_operator_instant_reply("view sip").unwrap();
        assert!(
            viewed.contains("**SIP**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("sip").is_none());
        assert!(try_operator_instant_reply("csrutil").is_none());
        let fv = try_operator_instant_reply("/filevault");
        if let Some(reply) = fv {
            assert!(
                !reply.contains("**SIP**"),
                "/filevault must stay FileVault, not SIP: {reply}"
            );
        }
    }

    #[test]
    fn gatekeeper_slash_and_view_open_do_not_steal_sip_or_bare_gatekeeper() {
        // Slash + view/open NL → Gatekeeper (`/sip` · `/filevault` parity).
        // Bare `gatekeeper` / `spctl` stay with the agent; `/sip` stays SIP; `/filevault` stays FileVault.
        assert!(looks_like_gatekeeper_request("/gatekeeper"));
        assert!(looks_like_gatekeeper_request("/gatekeeper-status"));
        assert!(looks_like_gatekeeper_request("/gk"));
        assert!(looks_like_gatekeeper_request("/gk-status"));
        assert!(looks_like_gatekeeper_request("/spctl"));
        assert!(looks_like_gatekeeper_request("/spctl-status"));
        assert!(looks_like_gatekeeper_request("/assessments"));
        assert!(looks_like_gatekeeper_request("view gatekeeper"));
        assert!(looks_like_gatekeeper_request("open gatekeeper status"));
        assert!(looks_like_gatekeeper_request("list spctl"));
        assert!(looks_like_gatekeeper_request("gatekeeper status"));
        assert!(looks_like_gatekeeper_request("is gatekeeper on"));
        assert!(looks_like_gatekeeper_request("is gatekeeper enabled"));
        assert!(looks_like_gatekeeper_request("what's my gatekeeper"));
        assert!(looks_like_gatekeeper_request("view gk"));
        assert!(!looks_like_gatekeeper_request("gatekeeper"));
        assert!(!looks_like_gatekeeper_request("spctl"));
        assert!(!looks_like_gatekeeper_request("gk"));
        assert!(!looks_like_gatekeeper_request("assessments"));
        assert!(!looks_like_gatekeeper_request("/sip"));
        assert!(!looks_like_gatekeeper_request("view sip"));
        assert!(!looks_like_gatekeeper_request("/filevault"));
        assert!(!looks_like_gatekeeper_request("view filevault"));
        assert!(!looks_like_gatekeeper_request("enable gatekeeper"));
        assert!(!looks_like_gatekeeper_request("disable gatekeeper"));
        assert!(!looks_like_sip_request("/gatekeeper"));
        assert!(!looks_like_sip_request("view gatekeeper"));
        assert!(!looks_like_sip_request("/spctl"));
        assert!(!looks_like_filevault_request("/gatekeeper"));
        assert!(!looks_like_filevault_request("view gatekeeper"));
        let gk = try_operator_instant_reply("/gatekeeper").unwrap();
        assert!(
            gk.contains("**Gatekeeper**") && !gk.to_lowercase().contains("execution"),
            "got: {gk}"
        );
        let viewed = try_operator_instant_reply("view gatekeeper").unwrap();
        assert!(
            viewed.contains("**Gatekeeper**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("gatekeeper").is_none());
        assert!(try_operator_instant_reply("spctl").is_none());
        let sip = try_operator_instant_reply("/sip");
        if let Some(reply) = sip {
            assert!(
                !reply.contains("**Gatekeeper**"),
                "/sip must stay SIP, not Gatekeeper: {reply}"
            );
        }
        let fv = try_operator_instant_reply("/filevault");
        if let Some(reply) = fv {
            assert!(
                !reply.contains("**Gatekeeper**"),
                "/filevault must stay FileVault, not Gatekeeper: {reply}"
            );
        }
    }

    #[test]
    fn spotlight_slash_and_view_open_do_not_steal_gatekeeper_or_bare() {
        // Slash + view/open NL → Spotlight (`/gatekeeper` · `/sip` parity).
        // Bare `spotlight` / `mdutil` stay with the agent;
        // `/gatekeeper` stays Gatekeeper; `/timemachine` stays Time Machine;
        // `/ssd` · `/free` stay capacity · free.
        assert!(looks_like_spotlight_request("/spotlight"));
        assert!(looks_like_spotlight_request("/spotlight-status"));
        assert!(looks_like_spotlight_request("/mdutil"));
        assert!(looks_like_spotlight_request("/mdutil-status"));
        assert!(looks_like_spotlight_request("/spotlight-indexing"));
        assert!(looks_like_spotlight_request("/mac-spotlight"));
        assert!(looks_like_spotlight_request("/mac-mdutil"));
        assert!(looks_like_spotlight_request("view spotlight"));
        assert!(looks_like_spotlight_request("open spotlight status"));
        assert!(looks_like_spotlight_request("list mdutil"));
        assert!(looks_like_spotlight_request("spotlight status"));
        assert!(looks_like_spotlight_request("is spotlight on"));
        assert!(looks_like_spotlight_request("is spotlight enabled"));
        assert!(looks_like_spotlight_request("what's my spotlight"));
        assert!(looks_like_spotlight_request("view mdutil"));
        assert!(!looks_like_spotlight_request("spotlight"));
        assert!(!looks_like_spotlight_request("mdutil"));
        assert!(!looks_like_spotlight_request("indexing"));
        assert!(!looks_like_spotlight_request("/gatekeeper"));
        assert!(!looks_like_spotlight_request("view gatekeeper"));
        assert!(!looks_like_spotlight_request("/sip"));
        assert!(!looks_like_spotlight_request("/filevault"));
        assert!(!looks_like_spotlight_request("/timemachine"));
        assert!(!looks_like_spotlight_request("/ssd"));
        assert!(!looks_like_spotlight_request("/free"));
        assert!(!looks_like_spotlight_request("view disk"));
        assert!(!looks_like_spotlight_request("enable spotlight"));
        assert!(!looks_like_spotlight_request("rebuild spotlight"));
        assert!(!looks_like_gatekeeper_request("/spotlight"));
        assert!(!looks_like_gatekeeper_request("view spotlight"));
        assert!(!looks_like_gatekeeper_request("/mdutil"));
        let s = try_operator_instant_reply("/spotlight").unwrap();
        assert!(
            s.contains("**Spotlight**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view spotlight").unwrap();
        assert!(
            viewed.contains("**Spotlight**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open mdutil").unwrap();
        assert!(
            open.contains("**Spotlight**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("spotlight").is_none());
        assert!(try_operator_instant_reply("mdutil").is_none());
        let gk = try_operator_instant_reply("/gatekeeper");
        if let Some(reply) = gk {
            assert!(
                !reply.contains("**Spotlight**"),
                "/gatekeeper must stay Gatekeeper, not Spotlight: {reply}"
            );
        }
        let tm = try_operator_instant_reply("/timemachine");
        if let Some(reply) = tm {
            assert!(
                !reply.contains("**Spotlight**"),
                "/timemachine must stay Time Machine, not Spotlight: {reply}"
            );
        }
    }

    #[test]
    fn firewall_slash_and_view_open_do_not_steal_gatekeeper_or_bare_firewall() {
        // Slash + view/open NL → Application Firewall (`/gatekeeper` · `/sip` parity).
        // Bare `firewall` / `afw` stay with the agent; `/gatekeeper` stays Gatekeeper; `/sip` stays SIP;
        // `/stealth` stays stealth.
        assert!(looks_like_firewall_request("/firewall"));
        assert!(looks_like_firewall_request("/firewall-status"));
        assert!(looks_like_firewall_request("/fw"));
        assert!(looks_like_firewall_request("/fw-status"));
        assert!(looks_like_firewall_request("/afw"));
        assert!(looks_like_firewall_request("/afw-status"));
        assert!(looks_like_firewall_request("/app-firewall"));
        assert!(looks_like_firewall_request("/application-firewall"));
        assert!(looks_like_firewall_request("/socketfilterfw"));
        assert!(looks_like_firewall_request("view firewall"));
        assert!(looks_like_firewall_request("open firewall status"));
        assert!(looks_like_firewall_request("list application firewall"));
        assert!(looks_like_firewall_request("firewall status"));
        assert!(looks_like_firewall_request("is firewall on"));
        assert!(looks_like_firewall_request("is firewall enabled"));
        assert!(looks_like_firewall_request("what's my firewall"));
        assert!(looks_like_firewall_request("view application firewall"));
        assert!(!looks_like_firewall_request("firewall"));
        assert!(!looks_like_firewall_request("afw"));
        assert!(!looks_like_firewall_request("application firewall"));
        assert!(!looks_like_firewall_request("app firewall"));
        assert!(!looks_like_firewall_request("/gatekeeper"));
        assert!(!looks_like_firewall_request("view gatekeeper"));
        assert!(!looks_like_firewall_request("/sip"));
        assert!(!looks_like_firewall_request("view sip"));
        assert!(!looks_like_firewall_request("/filevault"));
        assert!(!looks_like_firewall_request("/stealth"));
        assert!(!looks_like_firewall_request("view stealth"));
        assert!(!looks_like_firewall_request("enable firewall"));
        assert!(!looks_like_firewall_request("disable firewall"));
        assert!(!looks_like_gatekeeper_request("/firewall"));
        assert!(!looks_like_gatekeeper_request("view firewall"));
        assert!(!looks_like_sip_request("/firewall"));
        assert!(!looks_like_sip_request("view firewall"));
        assert!(!looks_like_filevault_request("/firewall"));
        let fw = try_operator_instant_reply("/firewall").unwrap();
        assert!(
            fw.contains("**Firewall**") && !fw.to_lowercase().contains("execution"),
            "got: {fw}"
        );
        let viewed = try_operator_instant_reply("view firewall").unwrap();
        assert!(
            viewed.contains("**Firewall**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("firewall").is_none());
        assert!(try_operator_instant_reply("afw").is_none());
        let gk = try_operator_instant_reply("/gatekeeper");
        if let Some(reply) = gk {
            assert!(
                !reply.contains("**Firewall**"),
                "/gatekeeper must stay Gatekeeper, not Firewall: {reply}"
            );
        }
        let sip = try_operator_instant_reply("/sip");
        if let Some(reply) = sip {
            assert!(
                !reply.contains("**Firewall**"),
                "/sip must stay SIP, not Firewall: {reply}"
            );
        }
    }

    #[test]
    fn firewall_stealth_slash_and_view_open_do_not_steal_firewall_or_bare_stealth() {
        // Slash + view/open NL → Firewall stealth (`/firewall` · `/gatekeeper` parity).
        // Bare `stealth` stays with the agent; `/firewall` stays global state; `/gatekeeper` stays Gatekeeper.
        assert!(looks_like_firewall_stealth_request("/stealth"));
        assert!(looks_like_firewall_stealth_request("/stealth-mode"));
        assert!(looks_like_firewall_stealth_request("/stealth-status"));
        assert!(looks_like_firewall_stealth_request("/firewall-stealth"));
        assert!(looks_like_firewall_stealth_request("/fw-stealth"));
        assert!(looks_like_firewall_stealth_request("/afw-stealth"));
        assert!(looks_like_firewall_stealth_request("view stealth"));
        assert!(looks_like_firewall_stealth_request("open stealth status"));
        assert!(looks_like_firewall_stealth_request("list stealth mode"));
        assert!(looks_like_firewall_stealth_request("stealth status"));
        assert!(looks_like_firewall_stealth_request("is stealth on"));
        assert!(looks_like_firewall_stealth_request("is stealth enabled"));
        assert!(looks_like_firewall_stealth_request("what's my stealth"));
        assert!(looks_like_firewall_stealth_request("view firewall stealth"));
        assert!(!looks_like_firewall_stealth_request("stealth"));
        assert!(!looks_like_firewall_stealth_request("stealth mode"));
        assert!(!looks_like_firewall_stealth_request("firewall stealth"));
        assert!(!looks_like_firewall_stealth_request("/firewall"));
        assert!(!looks_like_firewall_stealth_request("view firewall"));
        assert!(!looks_like_firewall_stealth_request("/gatekeeper"));
        assert!(!looks_like_firewall_stealth_request("/sip"));
        assert!(!looks_like_firewall_stealth_request("/filevault"));
        assert!(!looks_like_firewall_stealth_request("/firewall-logging"));
        assert!(!looks_like_firewall_stealth_request("view firewall logging"));
        assert!(!looks_like_firewall_stealth_request("enable stealth"));
        assert!(!looks_like_firewall_stealth_request("disable stealth"));
        assert!(!looks_like_firewall_request("/stealth"));
        assert!(!looks_like_firewall_request("view stealth"));
        assert!(!looks_like_firewall_request("/firewall-stealth"));
        let stealth = try_operator_instant_reply("/stealth").unwrap();
        assert!(
            stealth.contains("**Stealth**") && !stealth.to_lowercase().contains("execution"),
            "got: {stealth}"
        );
        let viewed = try_operator_instant_reply("view stealth").unwrap();
        assert!(
            viewed.contains("**Stealth**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("stealth").is_none());
        assert!(try_operator_instant_reply("stealth mode").is_none());
        let fw = try_operator_instant_reply("/firewall");
        if let Some(reply) = fw {
            assert!(
                !reply.contains("**Stealth**"),
                "/firewall must stay Firewall, not Stealth: {reply}"
            );
        }
        let gk = try_operator_instant_reply("/gatekeeper");
        if let Some(reply) = gk {
            assert!(
                !reply.contains("**Stealth**"),
                "/gatekeeper must stay Gatekeeper, not Stealth: {reply}"
            );
        }
    }

    #[test]
    fn firewall_logging_slash_and_view_open_do_not_steal_firewall_stealth_or_bare_logging() {
        // Slash + view/open NL → Firewall logging (`/firewall` · `/stealth` parity).
        // Bare `logging` / `firewall logging` stay with the agent; `/firewall` stays global;
        // `/stealth` stays stealth; `/logs` stays debug log.
        assert!(looks_like_firewall_logging_request("/firewall-logging"));
        assert!(looks_like_firewall_logging_request("/firewall-log"));
        assert!(looks_like_firewall_logging_request("/fw-logging"));
        assert!(looks_like_firewall_logging_request("/fw-log"));
        assert!(looks_like_firewall_logging_request("/afw-logging"));
        assert!(looks_like_firewall_logging_request("/afw-log"));
        assert!(looks_like_firewall_logging_request("/logging-mode"));
        assert!(looks_like_firewall_logging_request("/logging-opt"));
        assert!(looks_like_firewall_logging_request("/mac-firewall-logging"));
        assert!(looks_like_firewall_logging_request("view firewall logging"));
        assert!(looks_like_firewall_logging_request("open logging mode"));
        assert!(looks_like_firewall_logging_request("list fw logging"));
        assert!(looks_like_firewall_logging_request("firewall logging status"));
        assert!(looks_like_firewall_logging_request("is firewall logging on"));
        assert!(looks_like_firewall_logging_request("is firewall logging enabled"));
        assert!(looks_like_firewall_logging_request("what's my firewall logging"));
        assert!(looks_like_firewall_logging_request("what's the logging mode"));
        assert!(!looks_like_firewall_logging_request("logging"));
        assert!(!looks_like_firewall_logging_request("firewall logging"));
        assert!(!looks_like_firewall_logging_request("logging mode"));
        assert!(!looks_like_firewall_logging_request("/firewall"));
        assert!(!looks_like_firewall_logging_request("view firewall"));
        assert!(!looks_like_firewall_logging_request("/stealth"));
        assert!(!looks_like_firewall_logging_request("view stealth"));
        assert!(!looks_like_firewall_logging_request("/logs"));
        assert!(!looks_like_firewall_logging_request("/gatekeeper"));
        assert!(!looks_like_firewall_logging_request("/sip"));
        assert!(!looks_like_firewall_logging_request("/filevault"));
        assert!(!looks_like_firewall_logging_request("enable firewall logging"));
        assert!(!looks_like_firewall_logging_request("disable firewall logging"));
        assert!(!looks_like_firewall_request("/firewall-logging"));
        assert!(!looks_like_firewall_request("view firewall logging"));
        assert!(!looks_like_firewall_request("/fw-logging"));
        assert!(!looks_like_firewall_stealth_request("/firewall-logging"));
        assert!(!looks_like_firewall_stealth_request("view firewall logging"));
        let logging = try_operator_instant_reply("/firewall-logging").unwrap();
        assert!(
            logging.contains("**Firewall logging**")
                && !logging.to_lowercase().contains("execution"),
            "got: {logging}"
        );
        let viewed = try_operator_instant_reply("view firewall logging").unwrap();
        assert!(
            viewed.contains("**Firewall logging**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let mode = try_operator_instant_reply("/logging-mode").unwrap();
        assert!(
            mode.contains("**Firewall logging**") && !mode.to_lowercase().contains("execution"),
            "got: {mode}"
        );
        assert!(try_operator_instant_reply("logging").is_none());
        assert!(try_operator_instant_reply("firewall logging").is_none());
        let fw = try_operator_instant_reply("/firewall");
        if let Some(reply) = fw {
            assert!(
                !reply.contains("**Firewall logging**"),
                "/firewall must stay Firewall, not Firewall logging: {reply}"
            );
        }
        let stealth = try_operator_instant_reply("/stealth");
        if let Some(reply) = stealth {
            assert!(
                !reply.contains("**Firewall logging**"),
                "/stealth must stay Stealth, not Firewall logging: {reply}"
            );
        }
    }

    #[test]
    fn firewall_block_all_slash_and_view_open_do_not_steal_firewall_or_bare() {
        // Slash + view/open NL → Block all (`/allow-signed` · `/stealth` parity).
        // Bare `block all` / `blockall` stay with the agent; `/firewall` stays global;
        // `/stealth` stays stealth; `/firewall-logging` stays logging; `/allow-signed` stays allow-signed;
        // `/listapps` stays listapps.
        assert!(looks_like_firewall_block_all_request("/block-all"));
        assert!(looks_like_firewall_block_all_request("/blockall"));
        assert!(looks_like_firewall_block_all_request("/fw-block-all"));
        assert!(looks_like_firewall_block_all_request("/afw-block-all"));
        assert!(looks_like_firewall_block_all_request("/firewall-block-all"));
        assert!(looks_like_firewall_block_all_request("/getblockall"));
        assert!(looks_like_firewall_block_all_request("/mac-block-all"));
        assert!(looks_like_firewall_block_all_request("view block all"));
        assert!(looks_like_firewall_block_all_request("open fw block all"));
        assert!(looks_like_firewall_block_all_request("list block all"));
        assert!(looks_like_firewall_block_all_request("block all status"));
        assert!(looks_like_firewall_block_all_request("is block all on"));
        assert!(looks_like_firewall_block_all_request("is block all enabled"));
        assert!(looks_like_firewall_block_all_request("what's my block all"));
        assert!(looks_like_firewall_block_all_request("what's the block all status"));
        assert!(!looks_like_firewall_block_all_request("block all"));
        assert!(!looks_like_firewall_block_all_request("blockall"));
        assert!(!looks_like_firewall_block_all_request("/firewall"));
        assert!(!looks_like_firewall_block_all_request("view firewall"));
        assert!(!looks_like_firewall_block_all_request("/stealth"));
        assert!(!looks_like_firewall_block_all_request("/firewall-logging"));
        assert!(!looks_like_firewall_block_all_request("/allow-signed"));
        assert!(!looks_like_firewall_block_all_request("view allow signed"));
        assert!(!looks_like_firewall_block_all_request("/listapps"));
        assert!(!looks_like_firewall_block_all_request("view firewall apps"));
        assert!(!looks_like_firewall_block_all_request("/logs"));
        assert!(!looks_like_firewall_block_all_request("/gatekeeper"));
        assert!(!looks_like_firewall_block_all_request("enable block all"));
        assert!(!looks_like_firewall_block_all_request("disable block all"));
        assert!(!looks_like_firewall_request("/block-all"));
        assert!(!looks_like_firewall_request("view block all"));
        assert!(!looks_like_firewall_request("/fw-block-all"));
        assert!(!looks_like_firewall_stealth_request("/block-all"));
        assert!(!looks_like_firewall_stealth_request("view block all"));
        assert!(!looks_like_firewall_allow_signed_request("/block-all"));
        assert!(!looks_like_firewall_allow_signed_request("view block all"));
        assert!(!looks_like_firewall_logging_request("/block-all"));
        assert!(!looks_like_firewall_logging_request("view block all"));
    }

    #[test]
    fn firewall_listapps_slash_and_view_open_do_not_steal_firewall_or_bare() {
        // Slash + view/open NL → Firewall apps (`/block-all` · `/allow-signed` parity).
        // Bare `list apps` / `firewall apps` / `apps` stay with the agent; `/firewall` stays global;
        // `/stealth` stays stealth; `/firewall-logging` stays logging; `/allow-signed` stays allow-signed;
        // `/block-all` stays block-all; `list allow signed apps` stays allow-signed; `list app firewall` stays global.
        assert!(looks_like_firewall_listapps_request("/listapps"));
        assert!(looks_like_firewall_listapps_request("/list-apps"));
        assert!(looks_like_firewall_listapps_request("/firewall-apps"));
        assert!(looks_like_firewall_listapps_request("/fw-apps"));
        assert!(looks_like_firewall_listapps_request("/afw-apps"));
        assert!(looks_like_firewall_listapps_request("/firewall-listapps"));
        assert!(looks_like_firewall_listapps_request("/mac-firewall-apps"));
        assert!(looks_like_firewall_listapps_request("view firewall apps"));
        assert!(looks_like_firewall_listapps_request("open fw apps"));
        assert!(looks_like_firewall_listapps_request("list firewall apps"));
        assert!(looks_like_firewall_listapps_request("list afw apps"));
        assert!(looks_like_firewall_listapps_request("firewall apps status"));
        assert!(looks_like_firewall_listapps_request("what's my firewall apps"));
        assert!(looks_like_firewall_listapps_request("what's the firewall apps status"));
        assert!(!looks_like_firewall_listapps_request("list apps"));
        assert!(!looks_like_firewall_listapps_request("listapps"));
        assert!(!looks_like_firewall_listapps_request("apps"));
        assert!(!looks_like_firewall_listapps_request("firewall apps"));
        assert!(!looks_like_firewall_listapps_request("/firewall"));
        assert!(!looks_like_firewall_listapps_request("view firewall"));
        assert!(!looks_like_firewall_listapps_request("list app firewall"));
        assert!(!looks_like_firewall_listapps_request("/stealth"));
        assert!(!looks_like_firewall_listapps_request("/firewall-logging"));
        assert!(!looks_like_firewall_listapps_request("/allow-signed"));
        assert!(!looks_like_firewall_listapps_request("view allow signed"));
        assert!(!looks_like_firewall_listapps_request("list allow signed apps"));
        assert!(!looks_like_firewall_listapps_request("/block-all"));
        assert!(!looks_like_firewall_listapps_request("view block all"));
        assert!(!looks_like_firewall_listapps_request("/logs"));
        assert!(!looks_like_firewall_listapps_request("/gatekeeper"));
        assert!(!looks_like_firewall_listapps_request("enable listapps"));
        assert!(!looks_like_firewall_request("/listapps"));
        assert!(!looks_like_firewall_request("view firewall apps"));
        assert!(!looks_like_firewall_request("/fw-apps"));
        assert!(!looks_like_firewall_stealth_request("/listapps"));
        assert!(!looks_like_firewall_stealth_request("view firewall apps"));
        assert!(!looks_like_firewall_allow_signed_request("/listapps"));
        assert!(!looks_like_firewall_allow_signed_request("view firewall apps"));
        assert!(!looks_like_firewall_logging_request("/listapps"));
        assert!(!looks_like_firewall_logging_request("view firewall apps"));
        assert!(!looks_like_firewall_block_all_request("/listapps"));
        assert!(!looks_like_firewall_block_all_request("view firewall apps"));
        let apps = try_operator_instant_reply("/listapps").unwrap();
        assert!(
            apps.contains("**Firewall apps**") && !apps.to_lowercase().contains("execution"),
            "got: {apps}"
        );
        let viewed = try_operator_instant_reply("view firewall apps").unwrap();
        assert!(
            viewed.contains("**Firewall apps**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let fw_alias = try_operator_instant_reply("/fw-apps").unwrap();
        assert!(
            fw_alias.contains("**Firewall apps**")
                && !fw_alias.to_lowercase().contains("execution"),
            "got: {fw_alias}"
        );
        assert!(try_operator_instant_reply("list apps").is_none());
        assert!(try_operator_instant_reply("firewall apps").is_none());
        let fw = try_operator_instant_reply("/firewall");
        if let Some(reply) = fw {
            assert!(
                !reply.contains("**Firewall apps**"),
                "/firewall must stay Firewall, not Firewall apps: {reply}"
            );
        }
        let allow = try_operator_instant_reply("/allow-signed");
        if let Some(reply) = allow {
            assert!(
                !reply.contains("**Firewall apps**"),
                "/allow-signed must stay Allow signed, not Firewall apps: {reply}"
            );
        }
        let block = try_operator_instant_reply("/block-all");
        if let Some(reply) = block {
            assert!(
                !reply.contains("**Firewall apps**"),
                "/block-all must stay Block all, not Firewall apps: {reply}"
            );
        }
    }

    #[test]
    fn firewall_allow_signed_slash_and_view_open_do_not_steal_firewall_logging_or_bare() {
        // Slash + view/open NL → Allow signed (`/firewall-logging` · `/stealth` parity).
        // Bare `allow signed` / `signed software` stay with the agent; `/firewall` stays global;
        // `/stealth` stays stealth; `/firewall-logging` stays logging; `/block-all` stays block-all.
        assert!(looks_like_firewall_allow_signed_request("/allow-signed"));
        assert!(looks_like_firewall_allow_signed_request("/allowsigned"));
        assert!(looks_like_firewall_allow_signed_request("/fw-allow-signed"));
        assert!(looks_like_firewall_allow_signed_request("/afw-allow-signed"));
        assert!(looks_like_firewall_allow_signed_request("/firewall-allow-signed"));
        assert!(looks_like_firewall_allow_signed_request("/getallowsigned"));
        assert!(looks_like_firewall_allow_signed_request("/mac-allow-signed"));
        assert!(looks_like_firewall_allow_signed_request("view allow signed"));
        assert!(looks_like_firewall_allow_signed_request("open fw allow signed"));
        assert!(looks_like_firewall_allow_signed_request("list allow signed apps"));
        assert!(looks_like_firewall_allow_signed_request("allow signed status"));
        assert!(looks_like_firewall_allow_signed_request("is allow signed on"));
        assert!(looks_like_firewall_allow_signed_request("is allow signed enabled"));
        assert!(looks_like_firewall_allow_signed_request("what's my allow signed"));
        assert!(looks_like_firewall_allow_signed_request("what's the allow signed status"));
        assert!(!looks_like_firewall_allow_signed_request("allow signed"));
        assert!(!looks_like_firewall_allow_signed_request("signed"));
        assert!(!looks_like_firewall_allow_signed_request("signed software"));
        assert!(!looks_like_firewall_allow_signed_request("signed apps"));
        assert!(!looks_like_firewall_allow_signed_request("/firewall"));
        assert!(!looks_like_firewall_allow_signed_request("view firewall"));
        assert!(!looks_like_firewall_allow_signed_request("/stealth"));
        assert!(!looks_like_firewall_allow_signed_request("/firewall-logging"));
        assert!(!looks_like_firewall_allow_signed_request("view firewall logging"));
        assert!(!looks_like_firewall_allow_signed_request("/block-all"));
        assert!(!looks_like_firewall_allow_signed_request("view block all"));
        assert!(!looks_like_firewall_allow_signed_request("/logs"));
        assert!(!looks_like_firewall_allow_signed_request("/gatekeeper"));
        assert!(!looks_like_firewall_allow_signed_request("enable allow signed"));
        assert!(!looks_like_firewall_allow_signed_request("disable allow signed"));
        assert!(!looks_like_firewall_request("/allow-signed"));
        assert!(!looks_like_firewall_request("view allow signed"));
        assert!(!looks_like_firewall_request("/fw-allow-signed"));
        assert!(!looks_like_firewall_stealth_request("/allow-signed"));
        assert!(!looks_like_firewall_stealth_request("view allow signed"));
        assert!(!looks_like_firewall_logging_request("/allow-signed"));
        assert!(!looks_like_firewall_logging_request("view allow signed"));
        let allow = try_operator_instant_reply("/allow-signed").unwrap();
        assert!(
            allow.contains("**Allow signed**") && !allow.to_lowercase().contains("execution"),
            "got: {allow}"
        );
        let viewed = try_operator_instant_reply("view allow signed").unwrap();
        assert!(
            viewed.contains("**Allow signed**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let fw_alias = try_operator_instant_reply("/fw-allow-signed").unwrap();
        assert!(
            fw_alias.contains("**Allow signed**")
                && !fw_alias.to_lowercase().contains("execution"),
            "got: {fw_alias}"
        );
        assert!(try_operator_instant_reply("allow signed").is_none());
        assert!(try_operator_instant_reply("signed software").is_none());
        let fw = try_operator_instant_reply("/firewall");
        if let Some(reply) = fw {
            assert!(
                !reply.contains("**Allow signed**"),
                "/firewall must stay Firewall, not Allow signed: {reply}"
            );
        }
        let logging = try_operator_instant_reply("/firewall-logging");
        if let Some(reply) = logging {
            assert!(
                !reply.contains("**Allow signed**"),
                "/firewall-logging must stay Firewall logging, not Allow signed: {reply}"
            );
        }
    }

    #[test]
    fn time_machine_slash_and_view_open_do_not_steal_disk_cleanup_or_bare_backup() {
        // Slash + view/open NL → Time Machine (`/firewall` · `/sip` parity).
        // Bare `backup` / `tm` / `time machine` stay with the agent; `view disk` stays Disk Cleanup.
        assert!(looks_like_time_machine_request("/timemachine"));
        assert!(looks_like_time_machine_request("/time-machine"));
        assert!(looks_like_time_machine_request("/tm"));
        assert!(looks_like_time_machine_request("/tm-status"));
        assert!(looks_like_time_machine_request("/tmutil"));
        assert!(looks_like_time_machine_request("/backup-status"));
        assert!(looks_like_time_machine_request("/tm-backup"));
        assert!(looks_like_time_machine_request("view time machine"));
        assert!(looks_like_time_machine_request("open tm status"));
        assert!(looks_like_time_machine_request("list backup status"));
        assert!(looks_like_time_machine_request("time machine status"));
        assert!(looks_like_time_machine_request("is time machine running"));
        assert!(looks_like_time_machine_request("what's my time machine"));
        assert!(looks_like_time_machine_request("view backup status"));
        assert!(!looks_like_time_machine_request("backup"));
        assert!(!looks_like_time_machine_request("tm"));
        assert!(!looks_like_time_machine_request("time machine"));
        assert!(!looks_like_time_machine_request("timemachine"));
        assert!(!looks_like_time_machine_request("view disk"));
        assert!(!looks_like_time_machine_request("open disk"));
        assert!(!looks_like_time_machine_request("/ssd"));
        assert!(!looks_like_time_machine_request("/free"));
        assert!(!looks_like_time_machine_request("/boot-volume"));
        assert!(!looks_like_time_machine_request("/firewall"));
        assert!(!looks_like_time_machine_request("/stealth"));
        assert!(!looks_like_time_machine_request("/tasks"));
        assert!(!looks_like_time_machine_request("enable time machine"));
        assert!(!looks_like_time_machine_request("thin localsnapshots"));
        assert!(!looks_like_disk_cleanup_request("/timemachine"));
        assert!(!looks_like_disk_cleanup_request("view time machine"));
        assert!(!looks_like_firewall_request("/timemachine"));
        assert!(!looks_like_firewall_request("view time machine"));
        let tm = try_operator_instant_reply("/timemachine").unwrap();
        assert!(
            tm.contains("**Time Machine**") && !tm.to_lowercase().contains("execution"),
            "got: {tm}"
        );
        let viewed = try_operator_instant_reply("view time machine").unwrap();
        assert!(
            viewed.contains("**Time Machine**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("backup").is_none());
        assert!(try_operator_instant_reply("tm").is_none());
        assert!(try_operator_instant_reply("time machine").is_none());
        let cleanup = try_operator_instant_reply("view disk");
        if let Some(reply) = cleanup {
            assert!(
                !reply.contains("**Time Machine**"),
                "view disk must stay Disk Cleanup, not Time Machine: {reply}"
            );
        }
    }

    #[test]
    fn remote_login_slash_and_view_open_do_not_steal_vpn_firewall_or_bare_ssh() {
        // Slash + view/open NL → Remote Login (`/firewall` · `/sip` parity).
        // Bare `ssh` / `sshd` / `remote login` stay with the agent; `/vpn` stays VPN; `/firewall` stays Firewall.
        assert!(looks_like_remote_login_request("/remote-login"));
        assert!(looks_like_remote_login_request("/remotelogin"));
        assert!(looks_like_remote_login_request("/ssh"));
        assert!(looks_like_remote_login_request("/sshd"));
        assert!(looks_like_remote_login_request("/ssh-status"));
        assert!(looks_like_remote_login_request("/sshd-status"));
        assert!(looks_like_remote_login_request("/ssh-login"));
        assert!(looks_like_remote_login_request("/remote-ssh"));
        assert!(looks_like_remote_login_request("view remote login"));
        assert!(looks_like_remote_login_request("open ssh status"));
        assert!(looks_like_remote_login_request("list remote login"));
        assert!(looks_like_remote_login_request("remote login status"));
        assert!(looks_like_remote_login_request("is remote login on"));
        assert!(looks_like_remote_login_request("is ssh enabled"));
        assert!(looks_like_remote_login_request("what's my remote login"));
        assert!(looks_like_remote_login_request("view ssh"));
        assert!(!looks_like_remote_login_request("ssh"));
        assert!(!looks_like_remote_login_request("sshd"));
        assert!(!looks_like_remote_login_request("remote login"));
        assert!(!looks_like_remote_login_request("remotelogin"));
        assert!(!looks_like_remote_login_request("/vpn"));
        assert!(!looks_like_remote_login_request("/firewall"));
        assert!(!looks_like_remote_login_request("/stealth"));
        assert!(!looks_like_remote_login_request("/timemachine"));
        assert!(!looks_like_remote_login_request("/tasks"));
        assert!(!looks_like_remote_login_request("enable remote login"));
        assert!(!looks_like_remote_login_request("screen sharing"));
        assert!(!looks_like_remote_login_request("/screen-sharing"));
        assert!(!looks_like_remote_login_request("/vnc"));
        assert!(!looks_like_firewall_request("/remote-login"));
        assert!(!looks_like_firewall_request("view remote login"));
        assert!(!looks_like_time_machine_request("/remote-login"));
        assert!(!looks_like_time_machine_request("view remote login"));
        let rl = try_operator_instant_reply("/remote-login").unwrap();
        assert!(
            rl.contains("**Remote Login**") && !rl.to_lowercase().contains("execution"),
            "got: {rl}"
        );
        let viewed = try_operator_instant_reply("view remote login").unwrap();
        assert!(
            viewed.contains("**Remote Login**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("ssh").is_none());
        assert!(try_operator_instant_reply("sshd").is_none());
        assert!(try_operator_instant_reply("remote login").is_none());
        let vpn = try_operator_instant_reply("/vpn");
        if let Some(reply) = vpn {
            assert!(
                !reply.contains("**Remote Login**"),
                "/vpn must stay VPN, not Remote Login: {reply}"
            );
        }
        let fw = try_operator_instant_reply("/firewall");
        if let Some(reply) = fw {
            assert!(
                !reply.contains("**Remote Login**"),
                "/firewall must stay Firewall, not Remote Login: {reply}"
            );
        }
    }

    #[test]
    fn screen_sharing_slash_and_view_open_do_not_steal_ssh_display_or_bare_vnc() {
        // Slash + view/open NL → Screen Sharing (`/remote-login` · `/display` parity).
        // Bare `screen sharing` / `vnc` stay with the agent; `/ssh` stays Remote Login; `/screen` stays display.
        assert!(looks_like_screen_sharing_request("/screen-sharing"));
        assert!(looks_like_screen_sharing_request("/screensharing"));
        assert!(looks_like_screen_sharing_request("/screen-share"));
        assert!(looks_like_screen_sharing_request("/vnc"));
        assert!(looks_like_screen_sharing_request("/vnc-status"));
        assert!(looks_like_screen_sharing_request("/mac-vnc"));
        assert!(looks_like_screen_sharing_request("view screen sharing"));
        assert!(looks_like_screen_sharing_request("open vnc status"));
        assert!(looks_like_screen_sharing_request("list screen sharing"));
        assert!(looks_like_screen_sharing_request("screen sharing status"));
        assert!(looks_like_screen_sharing_request("is screen sharing on"));
        assert!(looks_like_screen_sharing_request("is vnc enabled"));
        assert!(looks_like_screen_sharing_request("what's my screen sharing"));
        assert!(looks_like_screen_sharing_request("view vnc"));
        assert!(!looks_like_screen_sharing_request("vnc"));
        assert!(!looks_like_screen_sharing_request("screen sharing"));
        assert!(!looks_like_screen_sharing_request("screensharing"));
        assert!(!looks_like_screen_sharing_request("/remote-login"));
        assert!(!looks_like_screen_sharing_request("/ssh"));
        assert!(!looks_like_screen_sharing_request("/sshd"));
        assert!(!looks_like_screen_sharing_request("/firewall"));
        assert!(!looks_like_screen_sharing_request("/screen"));
        assert!(!looks_like_screen_sharing_request("/screens"));
        assert!(!looks_like_screen_sharing_request("/display"));
        assert!(!looks_like_screen_sharing_request("/tasks"));
        assert!(!looks_like_screen_sharing_request("enable screen sharing"));
        assert!(!looks_like_remote_login_request("/screen-sharing"));
        assert!(!looks_like_remote_login_request("view screen sharing"));
        assert!(!looks_like_display_resolution_request("/screen-sharing"));
        assert!(!looks_like_display_resolution_request("view screen sharing"));
        assert!(!looks_like_display_resolution_request("/vnc"));
        let ss = try_operator_instant_reply("/screen-sharing").unwrap();
        assert!(
            ss.contains("**Screen Sharing**") && !ss.to_lowercase().contains("execution"),
            "got: {ss}"
        );
        let viewed = try_operator_instant_reply("view screen sharing").unwrap();
        assert!(
            viewed.contains("**Screen Sharing**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("vnc").is_none());
        assert!(try_operator_instant_reply("screen sharing").is_none());
        let ssh = try_operator_instant_reply("/ssh");
        if let Some(reply) = ssh {
            assert!(
                !reply.contains("**Screen Sharing**"),
                "/ssh must stay Remote Login, not Screen Sharing: {reply}"
            );
        }
        let screen = try_operator_instant_reply("/screen");
        if let Some(reply) = screen {
            assert!(
                !reply.contains("**Screen Sharing**"),
                "/screen must stay display resolution, not Screen Sharing: {reply}"
            );
        }
        assert!(!looks_like_screen_sharing_request("/file-sharing"));
        assert!(!looks_like_screen_sharing_request("/smb"));
        assert!(!looks_like_screen_sharing_request("view file sharing"));
    }

    #[test]
    fn file_sharing_slash_and_view_open_do_not_steal_vnc_filevault_or_bare_smb() {
        // Slash + view/open NL → File Sharing (`/screen-sharing` · `/filevault` parity).
        // Bare `file sharing` / `smb` / `smbd` stay with the agent; `/vnc` stays Screen Sharing;
        // `/filevault` stays FileVault; `/ssh` stays Remote Login.
        assert!(looks_like_file_sharing_request("/file-sharing"));
        assert!(looks_like_file_sharing_request("/filesharing"));
        assert!(looks_like_file_sharing_request("/file-share"));
        assert!(looks_like_file_sharing_request("/smb"));
        assert!(looks_like_file_sharing_request("/smb-status"));
        assert!(looks_like_file_sharing_request("/smbd"));
        assert!(looks_like_file_sharing_request("/mac-smb"));
        assert!(looks_like_file_sharing_request("view file sharing"));
        assert!(looks_like_file_sharing_request("open smb status"));
        assert!(looks_like_file_sharing_request("list file sharing"));
        assert!(looks_like_file_sharing_request("file sharing status"));
        assert!(looks_like_file_sharing_request("is file sharing on"));
        assert!(looks_like_file_sharing_request("is smb enabled"));
        assert!(looks_like_file_sharing_request("what's my file sharing"));
        assert!(looks_like_file_sharing_request("view smb"));
        assert!(!looks_like_file_sharing_request("smb"));
        assert!(!looks_like_file_sharing_request("smbd"));
        assert!(!looks_like_file_sharing_request("file sharing"));
        assert!(!looks_like_file_sharing_request("filesharing"));
        assert!(!looks_like_file_sharing_request("/screen-sharing"));
        assert!(!looks_like_file_sharing_request("/vnc"));
        assert!(!looks_like_file_sharing_request("/remote-login"));
        assert!(!looks_like_file_sharing_request("/ssh"));
        assert!(!looks_like_file_sharing_request("/filevault"));
        assert!(!looks_like_file_sharing_request("/file-vault"));
        assert!(!looks_like_file_sharing_request("/firewall"));
        assert!(!looks_like_file_sharing_request("/tasks"));
        assert!(!looks_like_file_sharing_request("enable file sharing"));
        assert!(!looks_like_screen_sharing_request("/file-sharing"));
        assert!(!looks_like_screen_sharing_request("view file sharing"));
        assert!(!looks_like_remote_login_request("/file-sharing"));
        assert!(!looks_like_remote_login_request("view file sharing"));
        assert!(!looks_like_filevault_request("/file-sharing"));
        assert!(!looks_like_filevault_request("view file sharing"));
        assert!(!looks_like_filevault_request("/smb"));
        let fs = try_operator_instant_reply("/file-sharing").unwrap();
        assert!(
            fs.contains("**File Sharing**") && !fs.to_lowercase().contains("execution"),
            "got: {fs}"
        );
        let viewed = try_operator_instant_reply("view file sharing").unwrap();
        assert!(
            viewed.contains("**File Sharing**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("smb").is_none());
        assert!(try_operator_instant_reply("file sharing").is_none());
        let vnc = try_operator_instant_reply("/vnc");
        if let Some(reply) = vnc {
            assert!(
                !reply.contains("**File Sharing**"),
                "/vnc must stay Screen Sharing, not File Sharing: {reply}"
            );
        }
        let fv = try_operator_instant_reply("/filevault");
        if let Some(reply) = fv {
            assert!(
                !reply.contains("**File Sharing**"),
                "/filevault must stay FileVault, not File Sharing: {reply}"
            );
        }
        assert!(!looks_like_file_sharing_request("/printer-sharing"));
        assert!(!looks_like_file_sharing_request("/share-printers"));
        assert!(!looks_like_file_sharing_request("view printer sharing"));
    }

    #[test]
    fn printer_sharing_slash_and_view_open_do_not_steal_smb_vnc_or_bare_printer() {
        // Slash + view/open NL → Printer Sharing (`/file-sharing` · `/screen-sharing` parity).
        // Bare `printer` / `printers` / `cups` stay with the agent; `/smb` stays File Sharing;
        // `/vnc` stays Screen Sharing; `/ssh` stays Remote Login.
        assert!(looks_like_printer_sharing_request("/printer-sharing"));
        assert!(looks_like_printer_sharing_request("/printersharing"));
        assert!(looks_like_printer_sharing_request("/printer-share"));
        assert!(looks_like_printer_sharing_request("/share-printers"));
        assert!(looks_like_printer_sharing_request("/cups-share"));
        assert!(looks_like_printer_sharing_request("/cups-sharing"));
        assert!(looks_like_printer_sharing_request("/mac-printer-sharing"));
        assert!(looks_like_printer_sharing_request("view printer sharing"));
        assert!(looks_like_printer_sharing_request("open cups share"));
        assert!(looks_like_printer_sharing_request("list printer sharing"));
        assert!(looks_like_printer_sharing_request("printer sharing status"));
        assert!(looks_like_printer_sharing_request("is printer sharing on"));
        assert!(looks_like_printer_sharing_request("is cups sharing enabled"));
        assert!(looks_like_printer_sharing_request("what's my printer sharing"));
        assert!(looks_like_printer_sharing_request("view share printers"));
        assert!(!looks_like_printer_sharing_request("printer"));
        assert!(!looks_like_printer_sharing_request("printers"));
        assert!(!looks_like_printer_sharing_request("cups"));
        assert!(!looks_like_printer_sharing_request("printer sharing"));
        assert!(!looks_like_printer_sharing_request("/file-sharing"));
        assert!(!looks_like_printer_sharing_request("/smb"));
        assert!(!looks_like_printer_sharing_request("/screen-sharing"));
        assert!(!looks_like_printer_sharing_request("/vnc"));
        assert!(!looks_like_printer_sharing_request("/remote-login"));
        assert!(!looks_like_printer_sharing_request("/ssh"));
        assert!(!looks_like_printer_sharing_request("/filevault"));
        assert!(!looks_like_printer_sharing_request("/firewall"));
        assert!(!looks_like_printer_sharing_request("/tasks"));
        assert!(!looks_like_printer_sharing_request("enable printer sharing"));
        assert!(!looks_like_file_sharing_request("/printer-sharing"));
        assert!(!looks_like_file_sharing_request("view printer sharing"));
        assert!(!looks_like_screen_sharing_request("/printer-sharing"));
        assert!(!looks_like_screen_sharing_request("view printer sharing"));
        assert!(!looks_like_remote_login_request("/printer-sharing"));
        assert!(!looks_like_remote_login_request("view printer sharing"));
        let ps = try_operator_instant_reply("/printer-sharing").unwrap();
        assert!(
            ps.contains("**Printer Sharing**") && !ps.to_lowercase().contains("execution"),
            "got: {ps}"
        );
        let viewed = try_operator_instant_reply("view printer sharing").unwrap();
        assert!(
            viewed.contains("**Printer Sharing**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("printer").is_none());
        assert!(try_operator_instant_reply("printer sharing").is_none());
        let smb = try_operator_instant_reply("/smb");
        if let Some(reply) = smb {
            assert!(
                !reply.contains("**Printer Sharing**"),
                "/smb must stay File Sharing, not Printer Sharing: {reply}"
            );
        }
        let vnc = try_operator_instant_reply("/vnc");
        if let Some(reply) = vnc {
            assert!(
                !reply.contains("**Printer Sharing**"),
                "/vnc must stay Screen Sharing, not Printer Sharing: {reply}"
            );
        }
        assert!(!looks_like_printer_sharing_request("/remote-management"));
        assert!(!looks_like_printer_sharing_request("/ard"));
        assert!(!looks_like_printer_sharing_request("view remote management"));
    }

    #[test]
    fn remote_management_slash_and_view_open_do_not_steal_vnc_ssh_or_bare_ard() {
        // Slash + view/open NL → Remote Management (`/screen-sharing` · `/remote-login` parity).
        // Bare `ard` / `remote management` / `remote desktop` stay with the agent; `/vnc` stays Screen Sharing;
        // `/ssh` stays Remote Login; `/smb` stays File Sharing; `/printer-sharing` stays Printer Sharing.
        assert!(looks_like_remote_management_request("/remote-management"));
        assert!(looks_like_remote_management_request("/remotemanagement"));
        assert!(looks_like_remote_management_request("/remote-mgmt"));
        assert!(looks_like_remote_management_request("/ard"));
        assert!(looks_like_remote_management_request("/ard-status"));
        assert!(looks_like_remote_management_request("/ard-state"));
        assert!(looks_like_remote_management_request("/apple-remote-desktop"));
        assert!(looks_like_remote_management_request("/mac-ard"));
        assert!(looks_like_remote_management_request("/ard-agent"));
        assert!(looks_like_remote_management_request("view remote management"));
        assert!(looks_like_remote_management_request("open ard status"));
        assert!(looks_like_remote_management_request("list remote management"));
        assert!(looks_like_remote_management_request("remote management status"));
        assert!(looks_like_remote_management_request("is remote management on"));
        assert!(looks_like_remote_management_request("is ard enabled"));
        assert!(looks_like_remote_management_request("what's my remote management"));
        assert!(looks_like_remote_management_request("view ard"));
        assert!(!looks_like_remote_management_request("ard"));
        assert!(!looks_like_remote_management_request("remote management"));
        assert!(!looks_like_remote_management_request("remote desktop"));
        assert!(!looks_like_remote_management_request("/screen-sharing"));
        assert!(!looks_like_remote_management_request("/vnc"));
        assert!(!looks_like_remote_management_request("/remote-login"));
        assert!(!looks_like_remote_management_request("/ssh"));
        assert!(!looks_like_remote_management_request("/file-sharing"));
        assert!(!looks_like_remote_management_request("/smb"));
        assert!(!looks_like_remote_management_request("/printer-sharing"));
        assert!(!looks_like_remote_management_request("/filevault"));
        assert!(!looks_like_remote_management_request("/firewall"));
        assert!(!looks_like_remote_management_request("/tasks"));
        assert!(!looks_like_remote_management_request("enable remote management"));
        assert!(!looks_like_screen_sharing_request("/remote-management"));
        assert!(!looks_like_screen_sharing_request("/ard"));
        assert!(!looks_like_screen_sharing_request("view remote management"));
        assert!(!looks_like_remote_login_request("/remote-management"));
        assert!(!looks_like_remote_login_request("/ard"));
        assert!(!looks_like_remote_login_request("view remote management"));
        assert!(!looks_like_file_sharing_request("/remote-management"));
        assert!(!looks_like_file_sharing_request("/ard"));
        assert!(!looks_like_printer_sharing_request("/remote-management"));
        assert!(!looks_like_printer_sharing_request("/ard"));
        let rm = try_operator_instant_reply("/remote-management").unwrap();
        assert!(
            rm.contains("**Remote Management**") && !rm.to_lowercase().contains("execution"),
            "got: {rm}"
        );
        let viewed = try_operator_instant_reply("view remote management").unwrap();
        assert!(
            viewed.contains("**Remote Management**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("ard").is_none());
        assert!(try_operator_instant_reply("remote management").is_none());
        let vnc = try_operator_instant_reply("/vnc");
        if let Some(reply) = vnc {
            assert!(
                !reply.contains("**Remote Management**"),
                "/vnc must stay Screen Sharing, not Remote Management: {reply}"
            );
        }
        let ssh = try_operator_instant_reply("/ssh");
        if let Some(reply) = ssh {
            assert!(
                !reply.contains("**Remote Management**"),
                "/ssh must stay Remote Login, not Remote Management: {reply}"
            );
        }
        assert!(!looks_like_remote_management_request("/content-caching"));
        assert!(!looks_like_remote_management_request("/asset-cache"));
        assert!(!looks_like_remote_management_request("view content caching"));
    }

    #[test]
    fn content_caching_slash_and_view_open_do_not_steal_ard_smb_or_bare_caching() {
        // Slash + view/open NL → Content Caching (`/remote-management` · `/printer-sharing` parity).
        // Bare `caching` / `content caching` / `asset cache` stay with the agent; `/ard` stays Remote Management;
        // `/smb` stays File Sharing; `/printer-sharing` stays Printer Sharing; `/vnc` stays Screen Sharing;
        // `/ssh` stays Remote Login.
        assert!(looks_like_content_caching_request("/content-caching"));
        assert!(looks_like_content_caching_request("/contentcaching"));
        assert!(looks_like_content_caching_request("/content-cache"));
        assert!(looks_like_content_caching_request("/asset-cache"));
        assert!(looks_like_content_caching_request("/assetcache"));
        assert!(looks_like_content_caching_request("/asset-caching"));
        assert!(looks_like_content_caching_request("/mac-content-caching"));
        assert!(looks_like_content_caching_request("/mac-asset-cache"));
        assert!(looks_like_content_caching_request("view content caching"));
        assert!(looks_like_content_caching_request("open asset cache status"));
        assert!(looks_like_content_caching_request("list content caching"));
        assert!(looks_like_content_caching_request("content caching status"));
        assert!(looks_like_content_caching_request("is content caching on"));
        assert!(looks_like_content_caching_request("is asset cache enabled"));
        assert!(looks_like_content_caching_request("what's my content caching"));
        assert!(looks_like_content_caching_request("view asset cache"));
        assert!(!looks_like_content_caching_request("caching"));
        assert!(!looks_like_content_caching_request("content caching"));
        assert!(!looks_like_content_caching_request("asset cache"));
        assert!(!looks_like_content_caching_request("/cache"));
        assert!(!looks_like_content_caching_request("/remote-management"));
        assert!(!looks_like_content_caching_request("/ard"));
        assert!(!looks_like_content_caching_request("/file-sharing"));
        assert!(!looks_like_content_caching_request("/smb"));
        assert!(!looks_like_content_caching_request("/printer-sharing"));
        assert!(!looks_like_content_caching_request("/screen-sharing"));
        assert!(!looks_like_content_caching_request("/vnc"));
        assert!(!looks_like_content_caching_request("/remote-login"));
        assert!(!looks_like_content_caching_request("/ssh"));
        assert!(!looks_like_content_caching_request("/tasks"));
        assert!(!looks_like_content_caching_request("enable content caching"));
        assert!(!looks_like_remote_management_request("/content-caching"));
        assert!(!looks_like_remote_management_request("/asset-cache"));
        assert!(!looks_like_remote_management_request("view content caching"));
        assert!(!looks_like_printer_sharing_request("/content-caching"));
        assert!(!looks_like_printer_sharing_request("/asset-cache"));
        assert!(!looks_like_file_sharing_request("/content-caching"));
        assert!(!looks_like_screen_sharing_request("/content-caching"));
        assert!(!looks_like_remote_login_request("/content-caching"));
        let cc = try_operator_instant_reply("/content-caching").unwrap();
        assert!(
            cc.contains("**Content Caching**") && !cc.to_lowercase().contains("execution"),
            "got: {cc}"
        );
        let viewed = try_operator_instant_reply("view content caching").unwrap();
        assert!(
            viewed.contains("**Content Caching**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("caching").is_none());
        assert!(try_operator_instant_reply("content caching").is_none());
        assert!(try_operator_instant_reply("asset cache").is_none());
        let ard = try_operator_instant_reply("/ard");
        if let Some(reply) = ard {
            assert!(
                !reply.contains("**Content Caching**"),
                "/ard must stay Remote Management, not Content Caching: {reply}"
            );
        }
        let smb = try_operator_instant_reply("/smb");
        if let Some(reply) = smb {
            assert!(
                !reply.contains("**Content Caching**"),
                "/smb must stay File Sharing, not Content Caching: {reply}"
            );
        }
        assert!(!looks_like_content_caching_request("/internet-sharing"));
        assert!(!looks_like_content_caching_request("/ics"));
        assert!(!looks_like_content_caching_request("view internet sharing"));
    }

    #[test]
    fn internet_sharing_slash_and_view_open_do_not_steal_vpn_smb_or_bare_internet() {
        // Slash + view/open NL → Internet Sharing (`/content-caching` · `/file-sharing` parity).
        // Bare `internet` / `nat` / `ics` / `internet sharing` stay with the agent; `/vpn` stays VPN;
        // `/smb` stays File Sharing; `/ard` stays Remote Management; `/content-caching` stays Content Caching.
        assert!(looks_like_internet_sharing_request("/internet-sharing"));
        assert!(looks_like_internet_sharing_request("/internetsharing"));
        assert!(looks_like_internet_sharing_request("/internet-share"));
        assert!(looks_like_internet_sharing_request("/network-sharing"));
        assert!(looks_like_internet_sharing_request("/ics"));
        assert!(looks_like_internet_sharing_request("/ics-status"));
        assert!(looks_like_internet_sharing_request("/nat-sharing"));
        assert!(looks_like_internet_sharing_request("/mac-internet-sharing"));
        assert!(looks_like_internet_sharing_request("/mac-ics"));
        assert!(looks_like_internet_sharing_request("view internet sharing"));
        assert!(looks_like_internet_sharing_request("open ics status"));
        assert!(looks_like_internet_sharing_request("list internet sharing"));
        assert!(looks_like_internet_sharing_request("internet sharing status"));
        assert!(looks_like_internet_sharing_request("is internet sharing on"));
        assert!(looks_like_internet_sharing_request("is ics enabled"));
        assert!(looks_like_internet_sharing_request("what's my internet sharing"));
        assert!(looks_like_internet_sharing_request("view ics"));
        assert!(!looks_like_internet_sharing_request("internet"));
        assert!(!looks_like_internet_sharing_request("nat"));
        assert!(!looks_like_internet_sharing_request("ics"));
        assert!(!looks_like_internet_sharing_request("internet sharing"));
        assert!(!looks_like_internet_sharing_request("/vpn"));
        assert!(!looks_like_internet_sharing_request("/file-sharing"));
        assert!(!looks_like_internet_sharing_request("/smb"));
        assert!(!looks_like_internet_sharing_request("/content-caching"));
        assert!(!looks_like_internet_sharing_request("/asset-cache"));
        assert!(!looks_like_internet_sharing_request("/remote-management"));
        assert!(!looks_like_internet_sharing_request("/ard"));
        assert!(!looks_like_internet_sharing_request("/ip"));
        assert!(!looks_like_internet_sharing_request("/tasks"));
        assert!(!looks_like_internet_sharing_request("/bluetooth-sharing"));
        assert!(!looks_like_internet_sharing_request("/bts"));
        assert!(!looks_like_internet_sharing_request("view bluetooth sharing"));
        assert!(!looks_like_internet_sharing_request("enable internet sharing"));
        assert!(!looks_like_content_caching_request("/internet-sharing"));
        assert!(!looks_like_content_caching_request("view internet sharing"));
        assert!(!looks_like_file_sharing_request("/internet-sharing"));
        assert!(!looks_like_remote_management_request("/internet-sharing"));
        let ics = try_operator_instant_reply("/internet-sharing").unwrap();
        assert!(
            ics.contains("**Internet Sharing**") && !ics.to_lowercase().contains("execution"),
            "got: {ics}"
        );
        let viewed = try_operator_instant_reply("view internet sharing").unwrap();
        assert!(
            viewed.contains("**Internet Sharing**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("internet").is_none());
        assert!(try_operator_instant_reply("internet sharing").is_none());
        let vpn = try_operator_instant_reply("/vpn");
        if let Some(reply) = vpn {
            assert!(
                !reply.contains("**Internet Sharing**"),
                "/vpn must stay VPN, not Internet Sharing: {reply}"
            );
        }
        let smb = try_operator_instant_reply("/smb");
        if let Some(reply) = smb {
            assert!(
                !reply.contains("**Internet Sharing**"),
                "/smb must stay File Sharing, not Internet Sharing: {reply}"
            );
        }
    }

    #[test]
    fn bluetooth_sharing_slash_and_view_open_do_not_steal_ics_smb_or_bare_bluetooth() {
        // Slash + view/open NL → Bluetooth Sharing (`/internet-sharing` · `/file-sharing` parity).
        // Bare `bluetooth` / `bluetooth sharing` / `obex` / `bts` stay with the agent; `/ics` stays
        // Internet Sharing; `/smb` stays File Sharing; `/wifi` stays SSID; `/content-caching` stays Content Caching.
        assert!(looks_like_bluetooth_sharing_request("/bluetooth-sharing"));
        assert!(looks_like_bluetooth_sharing_request("/bluetoothsharing"));
        assert!(looks_like_bluetooth_sharing_request("/bluetooth-share"));
        assert!(looks_like_bluetooth_sharing_request("/bt-sharing"));
        assert!(looks_like_bluetooth_sharing_request("/bt-share"));
        assert!(looks_like_bluetooth_sharing_request("/bts"));
        assert!(looks_like_bluetooth_sharing_request("/bts-status"));
        assert!(looks_like_bluetooth_sharing_request("/obex"));
        assert!(looks_like_bluetooth_sharing_request("/obex-sharing"));
        assert!(looks_like_bluetooth_sharing_request("/file-exchange"));
        assert!(looks_like_bluetooth_sharing_request("/bluetooth-file-exchange"));
        assert!(looks_like_bluetooth_sharing_request("/mac-bluetooth-sharing"));
        assert!(looks_like_bluetooth_sharing_request("/mac-bt-sharing"));
        assert!(looks_like_bluetooth_sharing_request("view bluetooth sharing"));
        assert!(looks_like_bluetooth_sharing_request("open bts status"));
        assert!(looks_like_bluetooth_sharing_request("list bluetooth sharing"));
        assert!(looks_like_bluetooth_sharing_request("bluetooth sharing status"));
        assert!(looks_like_bluetooth_sharing_request("is bluetooth sharing on"));
        assert!(looks_like_bluetooth_sharing_request("is bts enabled"));
        assert!(looks_like_bluetooth_sharing_request("what's my bluetooth sharing"));
        assert!(looks_like_bluetooth_sharing_request("view bts"));
        assert!(!looks_like_bluetooth_sharing_request("bluetooth"));
        assert!(!looks_like_bluetooth_sharing_request("bluetooth sharing"));
        assert!(!looks_like_bluetooth_sharing_request("obex"));
        assert!(!looks_like_bluetooth_sharing_request("bts"));
        assert!(!looks_like_bluetooth_sharing_request("/internet-sharing"));
        assert!(!looks_like_bluetooth_sharing_request("/ics"));
        assert!(!looks_like_bluetooth_sharing_request("/file-sharing"));
        assert!(!looks_like_bluetooth_sharing_request("/smb"));
        assert!(!looks_like_bluetooth_sharing_request("/content-caching"));
        assert!(!looks_like_bluetooth_sharing_request("/asset-cache"));
        assert!(!looks_like_bluetooth_sharing_request("/wifi"));
        assert!(!looks_like_bluetooth_sharing_request("/ip"));
        assert!(!looks_like_bluetooth_sharing_request("/tasks"));
        assert!(!looks_like_bluetooth_sharing_request("enable bluetooth sharing"));
        assert!(!looks_like_internet_sharing_request("/bluetooth-sharing"));
        assert!(!looks_like_internet_sharing_request("view bluetooth sharing"));
        assert!(!looks_like_content_caching_request("/bluetooth-sharing"));
        assert!(!looks_like_file_sharing_request("/bluetooth-sharing"));
        let bts = try_operator_instant_reply("/bluetooth-sharing").unwrap();
        assert!(
            bts.contains("**Bluetooth Sharing**") && !bts.to_lowercase().contains("execution"),
            "got: {bts}"
        );
        let viewed = try_operator_instant_reply("view bluetooth sharing").unwrap();
        assert!(
            viewed.contains("**Bluetooth Sharing**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("bluetooth").is_none());
        assert!(try_operator_instant_reply("bluetooth sharing").is_none());
        let ics = try_operator_instant_reply("/internet-sharing");
        if let Some(reply) = ics {
            assert!(
                !reply.contains("**Bluetooth Sharing**"),
                "/internet-sharing must stay Internet Sharing, not Bluetooth Sharing: {reply}"
            );
        }
        let smb = try_operator_instant_reply("/smb");
        if let Some(reply) = smb {
            assert!(
                !reply.contains("**Bluetooth Sharing**"),
                "/smb must stay File Sharing, not Bluetooth Sharing: {reply}"
            );
        }
        assert!(!looks_like_bluetooth_sharing_request("/media-sharing"));
        assert!(!looks_like_bluetooth_sharing_request("/home-sharing"));
        assert!(!looks_like_bluetooth_sharing_request("view media sharing"));
    }

    #[test]
    fn media_sharing_slash_and_view_open_do_not_steal_bts_smb_or_bare_media() {
        // Slash + view/open NL → Media Sharing (`/bluetooth-sharing` · `/file-sharing` parity).
        // Bare `media` / `media sharing` / `home sharing` stay with the agent; `/bts` stays Bluetooth
        // Sharing; `/smb` stays File Sharing; `/ics` stays Internet Sharing; `/wifi` stays SSID.
        assert!(looks_like_media_sharing_request("/media-sharing"));
        assert!(looks_like_media_sharing_request("/mediasharing"));
        assert!(looks_like_media_sharing_request("/media-share"));
        assert!(looks_like_media_sharing_request("/home-sharing"));
        assert!(looks_like_media_sharing_request("/homesharing"));
        assert!(looks_like_media_sharing_request("/home-share"));
        assert!(looks_like_media_sharing_request("/itunes-sharing"));
        assert!(looks_like_media_sharing_request("/amp-sharing"));
        assert!(looks_like_media_sharing_request("/mac-media-sharing"));
        assert!(looks_like_media_sharing_request("/mac-home-sharing"));
        assert!(looks_like_media_sharing_request("view media sharing"));
        assert!(looks_like_media_sharing_request("open home sharing status"));
        assert!(looks_like_media_sharing_request("list media sharing"));
        assert!(looks_like_media_sharing_request("media sharing status"));
        assert!(looks_like_media_sharing_request("is media sharing on"));
        assert!(looks_like_media_sharing_request("is home sharing enabled"));
        assert!(looks_like_media_sharing_request("what's my media sharing"));
        assert!(looks_like_media_sharing_request("view home sharing"));
        assert!(!looks_like_media_sharing_request("media"));
        assert!(!looks_like_media_sharing_request("media sharing"));
        assert!(!looks_like_media_sharing_request("home sharing"));
        assert!(!looks_like_media_sharing_request("/bluetooth-sharing"));
        assert!(!looks_like_media_sharing_request("/bts"));
        assert!(!looks_like_media_sharing_request("/obex"));
        assert!(!looks_like_media_sharing_request("/internet-sharing"));
        assert!(!looks_like_media_sharing_request("/ics"));
        assert!(!looks_like_media_sharing_request("/file-sharing"));
        assert!(!looks_like_media_sharing_request("/smb"));
        assert!(!looks_like_media_sharing_request("/wifi"));
        assert!(!looks_like_media_sharing_request("/ip"));
        assert!(!looks_like_media_sharing_request("/tasks"));
        assert!(!looks_like_media_sharing_request("enable media sharing"));
        assert!(!looks_like_bluetooth_sharing_request("/media-sharing"));
        assert!(!looks_like_bluetooth_sharing_request("view media sharing"));
        assert!(!looks_like_internet_sharing_request("/media-sharing"));
        assert!(!looks_like_file_sharing_request("/media-sharing"));
        let ms = try_operator_instant_reply("/media-sharing").unwrap();
        assert!(
            ms.contains("**Media Sharing**") && !ms.to_lowercase().contains("execution"),
            "got: {ms}"
        );
        let viewed = try_operator_instant_reply("view home sharing").unwrap();
        assert!(
            viewed.contains("**Media Sharing**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("media").is_none());
        assert!(try_operator_instant_reply("media sharing").is_none());
        let bts = try_operator_instant_reply("/bluetooth-sharing");
        if let Some(reply) = bts {
            assert!(
                !reply.contains("**Media Sharing**"),
                "/bluetooth-sharing must stay Bluetooth Sharing, not Media Sharing: {reply}"
            );
        }
        let smb = try_operator_instant_reply("/smb");
        if let Some(reply) = smb {
            assert!(
                !reply.contains("**Media Sharing**"),
                "/smb must stay File Sharing, not Media Sharing: {reply}"
            );
        }
        assert!(!looks_like_media_sharing_request("/remote-apple-events"));
        assert!(!looks_like_media_sharing_request("/rae"));
        assert!(!looks_like_media_sharing_request("view remote apple events"));
    }

    #[test]
    fn remote_apple_events_slash_and_view_open_do_not_steal_ssh_ard_or_bare_rae() {
        // Slash + view/open NL → Remote Apple Events (`/remote-login` · `/remote-management` parity).
        // Bare `apple events` / `remote apple events` / `eppc` / `rae` stay with the agent;
        // `/ssh` stays Remote Login; `/ard` stays Remote Management; `/vnc` stays Screen Sharing;
        // `/media-sharing` stays Media Sharing.
        assert!(looks_like_remote_apple_events_request("/remote-apple-events"));
        assert!(looks_like_remote_apple_events_request("/remoteappleevents"));
        assert!(looks_like_remote_apple_events_request("/remote-apple-event"));
        assert!(looks_like_remote_apple_events_request("/apple-events"));
        assert!(looks_like_remote_apple_events_request("/appleevents"));
        assert!(looks_like_remote_apple_events_request("/rae"));
        assert!(looks_like_remote_apple_events_request("/rae-status"));
        assert!(looks_like_remote_apple_events_request("/eppc"));
        assert!(looks_like_remote_apple_events_request("/eppc-status"));
        assert!(looks_like_remote_apple_events_request("/aeserver"));
        assert!(looks_like_remote_apple_events_request("/mac-rae"));
        assert!(looks_like_remote_apple_events_request("/mac-remote-apple-events"));
        assert!(looks_like_remote_apple_events_request("view remote apple events"));
        assert!(looks_like_remote_apple_events_request("open rae status"));
        assert!(looks_like_remote_apple_events_request("list remote apple events"));
        assert!(looks_like_remote_apple_events_request("remote apple events status"));
        assert!(looks_like_remote_apple_events_request("is remote apple events on"));
        assert!(looks_like_remote_apple_events_request("is rae enabled"));
        assert!(looks_like_remote_apple_events_request("what's my remote apple events"));
        assert!(looks_like_remote_apple_events_request("view eppc"));
        assert!(!looks_like_remote_apple_events_request("rae"));
        assert!(!looks_like_remote_apple_events_request("eppc"));
        assert!(!looks_like_remote_apple_events_request("apple events"));
        assert!(!looks_like_remote_apple_events_request("remote apple events"));
        assert!(!looks_like_remote_apple_events_request("/remote-login"));
        assert!(!looks_like_remote_apple_events_request("/ssh"));
        assert!(!looks_like_remote_apple_events_request("/remote-management"));
        assert!(!looks_like_remote_apple_events_request("/ard"));
        assert!(!looks_like_remote_apple_events_request("/screen-sharing"));
        assert!(!looks_like_remote_apple_events_request("/vnc"));
        assert!(!looks_like_remote_apple_events_request("/media-sharing"));
        assert!(!looks_like_remote_apple_events_request("/home-sharing"));
        assert!(!looks_like_remote_apple_events_request("/tasks"));
        assert!(!looks_like_remote_apple_events_request("enable remote apple events"));
        assert!(!looks_like_remote_login_request("/remote-apple-events"));
        assert!(!looks_like_remote_login_request("/rae"));
        assert!(!looks_like_remote_login_request("view remote apple events"));
        assert!(!looks_like_remote_management_request("/remote-apple-events"));
        assert!(!looks_like_remote_management_request("/rae"));
        assert!(!looks_like_remote_management_request("view remote apple events"));
        assert!(!looks_like_media_sharing_request("/remote-apple-events"));
        let rae = try_operator_instant_reply("/remote-apple-events").unwrap();
        assert!(
            rae.contains("**Remote Apple Events**") && !rae.to_lowercase().contains("execution"),
            "got: {rae}"
        );
        let viewed = try_operator_instant_reply("view rae").unwrap();
        assert!(
            viewed.contains("**Remote Apple Events**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("rae").is_none());
        assert!(try_operator_instant_reply("apple events").is_none());
        assert!(try_operator_instant_reply("remote apple events").is_none());
        let ssh = try_operator_instant_reply("/ssh");
        if let Some(reply) = ssh {
            assert!(
                !reply.contains("**Remote Apple Events**"),
                "/ssh must stay Remote Login, not Remote Apple Events: {reply}"
            );
        }
        let ard = try_operator_instant_reply("/ard");
        if let Some(reply) = ard {
            assert!(
                !reply.contains("**Remote Apple Events**"),
                "/ard must stay Remote Management, not Remote Apple Events: {reply}"
            );
        }
        let ms = try_operator_instant_reply("/media-sharing");
        if let Some(reply) = ms {
            assert!(
                !reply.contains("**Remote Apple Events**"),
                "/media-sharing must stay Media Sharing, not Remote Apple Events: {reply}"
            );
        }
        assert!(!looks_like_remote_apple_events_request("/airplay"));
        assert!(!looks_like_remote_apple_events_request("/airplay-receiver"));
        assert!(!looks_like_remote_apple_events_request("view airplay"));
    }

    #[test]
    fn airplay_receiver_slash_and_view_open_do_not_steal_media_rae_or_bare_airplay() {
        // Slash + view/open NL → AirPlay Receiver (`/media-sharing` · `/rae` parity).
        // Bare `airplay` / `airplay receiver` stay with the agent; `/media-sharing` stays Media Sharing;
        // `/rae` · `/eppc` stay Remote Apple Events; `/vnc` stays Screen Sharing; `/display` stays displays.
        assert!(looks_like_airplay_receiver_request("/airplay"));
        assert!(looks_like_airplay_receiver_request("/airplay-status"));
        assert!(looks_like_airplay_receiver_request("/airplay-receiver"));
        assert!(looks_like_airplay_receiver_request("/airplayreceiver"));
        assert!(looks_like_airplay_receiver_request("/airplay-recv"));
        assert!(looks_like_airplay_receiver_request("/airplay-rx"));
        assert!(looks_like_airplay_receiver_request("/ap-receiver"));
        assert!(looks_like_airplay_receiver_request("/mac-airplay"));
        assert!(looks_like_airplay_receiver_request("/mac-airplay-receiver"));
        assert!(looks_like_airplay_receiver_request("view airplay"));
        assert!(looks_like_airplay_receiver_request("open airplay receiver status"));
        assert!(looks_like_airplay_receiver_request("list airplay"));
        assert!(looks_like_airplay_receiver_request("airplay status"));
        assert!(looks_like_airplay_receiver_request("is airplay on"));
        assert!(looks_like_airplay_receiver_request("is airplay receiver enabled"));
        assert!(looks_like_airplay_receiver_request("what's my airplay"));
        assert!(looks_like_airplay_receiver_request("view airplay receiver"));
        assert!(!looks_like_airplay_receiver_request("airplay"));
        assert!(!looks_like_airplay_receiver_request("airplay receiver"));
        assert!(!looks_like_airplay_receiver_request("/media-sharing"));
        assert!(!looks_like_airplay_receiver_request("/home-sharing"));
        assert!(!looks_like_airplay_receiver_request("/remote-apple-events"));
        assert!(!looks_like_airplay_receiver_request("/rae"));
        assert!(!looks_like_airplay_receiver_request("/eppc"));
        assert!(!looks_like_airplay_receiver_request("/screen-sharing"));
        assert!(!looks_like_airplay_receiver_request("/vnc"));
        assert!(!looks_like_airplay_receiver_request("/display"));
        assert!(!looks_like_airplay_receiver_request("/tasks"));
        assert!(!looks_like_airplay_receiver_request("enable airplay"));
        assert!(!looks_like_media_sharing_request("/airplay"));
        assert!(!looks_like_media_sharing_request("view airplay"));
        assert!(!looks_like_remote_apple_events_request("/airplay"));
        assert!(!looks_like_remote_apple_events_request("view airplay receiver"));
        let ap = try_operator_instant_reply("/airplay").unwrap();
        assert!(
            ap.contains("**AirPlay Receiver**") && !ap.to_lowercase().contains("execution"),
            "got: {ap}"
        );
        let viewed = try_operator_instant_reply("view airplay receiver").unwrap();
        assert!(
            viewed.contains("**AirPlay Receiver**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        assert!(try_operator_instant_reply("airplay").is_none());
        assert!(try_operator_instant_reply("airplay receiver").is_none());
        let ms = try_operator_instant_reply("/media-sharing");
        if let Some(reply) = ms {
            assert!(
                !reply.contains("**AirPlay Receiver**"),
                "/media-sharing must stay Media Sharing, not AirPlay Receiver: {reply}"
            );
        }
        let rae = try_operator_instant_reply("/rae");
        if let Some(reply) = rae {
            assert!(
                !reply.contains("**AirPlay Receiver**"),
                "/rae must stay Remote Apple Events, not AirPlay Receiver: {reply}"
            );
        }
        assert!(!looks_like_airplay_receiver_request("/dvd-sharing"));
        assert!(!looks_like_airplay_receiver_request("/ods"));
        assert!(!looks_like_airplay_receiver_request("view dvd sharing"));
    }

    #[test]
    fn dvd_cd_sharing_slash_and_view_open_do_not_steal_media_airplay_or_bare_dvd() {
        // Slash + view/open NL → DVD or CD Sharing (`/airplay` · `/media-sharing` parity).
        // Bare `dvd` / `cd` / `dvd sharing` / `ods` stay with the agent; `/airplay` stays AirPlay;
        // `/media-sharing` stays Media Sharing; `/smb` stays File Sharing; `/vnc` stays Screen Sharing.
        assert!(looks_like_dvd_cd_sharing_request("/dvd-sharing"));
        assert!(looks_like_dvd_cd_sharing_request("/dvdsharing"));
        assert!(looks_like_dvd_cd_sharing_request("/dvd-sharing-status"));
        assert!(looks_like_dvd_cd_sharing_request("/dvd-share"));
        assert!(looks_like_dvd_cd_sharing_request("/cd-sharing"));
        assert!(looks_like_dvd_cd_sharing_request("/cdsharing"));
        assert!(looks_like_dvd_cd_sharing_request("/cd-share"));
        assert!(looks_like_dvd_cd_sharing_request("/dvd-cd-sharing"));
        assert!(looks_like_dvd_cd_sharing_request("/optical-sharing"));
        assert!(looks_like_dvd_cd_sharing_request("/ods"));
        assert!(looks_like_dvd_cd_sharing_request("/ods-status"));
        assert!(looks_like_dvd_cd_sharing_request("/odsagent"));
        assert!(looks_like_dvd_cd_sharing_request("/mac-dvd-sharing"));
        assert!(looks_like_dvd_cd_sharing_request("/mac-ods"));
        assert!(looks_like_dvd_cd_sharing_request("view dvd sharing"));
        assert!(looks_like_dvd_cd_sharing_request("open cd sharing status"));
        assert!(looks_like_dvd_cd_sharing_request("list ods"));
        assert!(looks_like_dvd_cd_sharing_request("dvd sharing status"));
        assert!(looks_like_dvd_cd_sharing_request("is dvd sharing on"));
        assert!(looks_like_dvd_cd_sharing_request("is cd sharing enabled"));
        assert!(looks_like_dvd_cd_sharing_request("is ods enabled"));
        assert!(looks_like_dvd_cd_sharing_request("what's my dvd sharing"));
        assert!(looks_like_dvd_cd_sharing_request("view optical sharing"));
        assert!(!looks_like_dvd_cd_sharing_request("dvd"));
        assert!(!looks_like_dvd_cd_sharing_request("cd"));
        assert!(!looks_like_dvd_cd_sharing_request("dvd sharing"));
        assert!(!looks_like_dvd_cd_sharing_request("cd sharing"));
        assert!(!looks_like_dvd_cd_sharing_request("ods"));
        assert!(!looks_like_dvd_cd_sharing_request("/media-sharing"));
        assert!(!looks_like_dvd_cd_sharing_request("/home-sharing"));
        assert!(!looks_like_dvd_cd_sharing_request("/airplay"));
        assert!(!looks_like_dvd_cd_sharing_request("/airplay-receiver"));
        assert!(!looks_like_dvd_cd_sharing_request("/file-sharing"));
        assert!(!looks_like_dvd_cd_sharing_request("/smb"));
        assert!(!looks_like_dvd_cd_sharing_request("/screen-sharing"));
        assert!(!looks_like_dvd_cd_sharing_request("/vnc"));
        assert!(!looks_like_dvd_cd_sharing_request("/tasks"));
        assert!(!looks_like_dvd_cd_sharing_request("enable dvd sharing"));
        assert!(!looks_like_airplay_receiver_request("/dvd-sharing"));
        assert!(!looks_like_airplay_receiver_request("view dvd sharing"));
        assert!(!looks_like_media_sharing_request("/dvd-sharing"));
        assert!(!looks_like_media_sharing_request("view cd sharing"));
        let dvd = try_operator_instant_reply("/dvd-sharing").unwrap();
        assert!(
            dvd.contains("**DVD or CD Sharing**") && !dvd.to_lowercase().contains("execution"),
            "got: {dvd}"
        );
        let viewed = try_operator_instant_reply("view dvd sharing").unwrap();
        assert!(
            viewed.contains("**DVD or CD Sharing**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let ods = try_operator_instant_reply("/ods").unwrap();
        assert!(
            ods.contains("**DVD or CD Sharing**") && !ods.to_lowercase().contains("execution"),
            "got: {ods}"
        );
        assert!(try_operator_instant_reply("dvd").is_none());
        assert!(try_operator_instant_reply("cd").is_none());
        assert!(try_operator_instant_reply("dvd sharing").is_none());
        assert!(try_operator_instant_reply("ods").is_none());
        let ap = try_operator_instant_reply("/airplay");
        if let Some(reply) = ap {
            assert!(
                !reply.contains("**DVD or CD Sharing**"),
                "/airplay must stay AirPlay Receiver, not DVD/CD Sharing: {reply}"
            );
        }
        let ms = try_operator_instant_reply("/media-sharing");
        if let Some(reply) = ms {
            assert!(
                !reply.contains("**DVD or CD Sharing**"),
                "/media-sharing must stay Media Sharing, not DVD/CD Sharing: {reply}"
            );
        }
        assert!(!looks_like_dvd_cd_sharing_request("/airdrop"));
        assert!(!looks_like_dvd_cd_sharing_request("view airdrop"));
    }

    #[test]
    fn airdrop_slash_and_view_open_do_not_steal_airplay_bts_or_bare_airdrop() {
        // Slash + view/open NL → AirDrop (`/airplay` · `/bluetooth-sharing` parity).
        // Bare `airdrop` / `air drop` stay with the agent; `/airplay` stays AirPlay Receiver;
        // `/bluetooth-sharing` · `/bts` stay Bluetooth Sharing; `/media-sharing` stays Media Sharing;
        // `/smb` stays File Sharing; `/dvd-sharing` · `/ods` stay DVD/CD Sharing.
        assert!(looks_like_airdrop_request("/airdrop"));
        assert!(looks_like_airdrop_request("/air-drop"));
        assert!(looks_like_airdrop_request("/airdrop-status"));
        assert!(looks_like_airdrop_request("/air-drop-status"));
        assert!(looks_like_airdrop_request("/airdrop-state"));
        assert!(looks_like_airdrop_request("/airdrop-mode"));
        assert!(looks_like_airdrop_request("/mac-airdrop"));
        assert!(looks_like_airdrop_request("/mac-air-drop"));
        assert!(looks_like_airdrop_request("view airdrop"));
        assert!(looks_like_airdrop_request("open air drop status"));
        assert!(looks_like_airdrop_request("list airdrop"));
        assert!(looks_like_airdrop_request("airdrop status"));
        assert!(looks_like_airdrop_request("is airdrop on"));
        assert!(looks_like_airdrop_request("is air drop enabled"));
        assert!(looks_like_airdrop_request("what's my airdrop"));
        assert!(looks_like_airdrop_request("view air-drop"));
        assert!(!looks_like_airdrop_request("airdrop"));
        assert!(!looks_like_airdrop_request("air drop"));
        assert!(!looks_like_airdrop_request("/airplay"));
        assert!(!looks_like_airdrop_request("/airplay-receiver"));
        assert!(!looks_like_airdrop_request("/bluetooth-sharing"));
        assert!(!looks_like_airdrop_request("/bts"));
        assert!(!looks_like_airdrop_request("/media-sharing"));
        assert!(!looks_like_airdrop_request("/file-sharing"));
        assert!(!looks_like_airdrop_request("/smb"));
        assert!(!looks_like_airdrop_request("/dvd-sharing"));
        assert!(!looks_like_airdrop_request("/ods"));
        assert!(!looks_like_airdrop_request("/tasks"));
        assert!(!looks_like_airdrop_request("enable airdrop"));
        assert!(!looks_like_airplay_receiver_request("/airdrop"));
        assert!(!looks_like_airplay_receiver_request("view airdrop"));
        assert!(!looks_like_bluetooth_sharing_request("/airdrop"));
        assert!(!looks_like_bluetooth_sharing_request("view airdrop"));
        assert!(!looks_like_dvd_cd_sharing_request("/airdrop"));
        assert!(!looks_like_dvd_cd_sharing_request("view airdrop"));
        let ad = try_operator_instant_reply("/airdrop").unwrap();
        assert!(
            ad.contains("**AirDrop**") && !ad.to_lowercase().contains("execution"),
            "got: {ad}"
        );
        let viewed = try_operator_instant_reply("view airdrop").unwrap();
        assert!(
            viewed.contains("**AirDrop**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let spaced = try_operator_instant_reply("view air drop").unwrap();
        assert!(
            spaced.contains("**AirDrop**") && !spaced.to_lowercase().contains("execution"),
            "got: {spaced}"
        );
        assert!(try_operator_instant_reply("airdrop").is_none());
        assert!(try_operator_instant_reply("air drop").is_none());
        let ap = try_operator_instant_reply("/airplay");
        if let Some(reply) = ap {
            assert!(
                !reply.contains("**AirDrop**"),
                "/airplay must stay AirPlay Receiver, not AirDrop: {reply}"
            );
        }
        let bts = try_operator_instant_reply("/bts");
        if let Some(reply) = bts {
            assert!(
                !reply.contains("**AirDrop**"),
                "/bts must stay Bluetooth Sharing, not AirDrop: {reply}"
            );
        }
    }

    #[test]
    fn guest_account_slash_and_view_open_do_not_steal_username_airdrop_or_bare_guest() {
        // Slash + view/open NL → Guest Account (`/airdrop` · `/username` parity).
        // Bare `guest` / `guest account` stay with the agent; `/username` · `/whoami` stay login name;
        // `/airdrop` stays AirDrop; `/filevault` stays FileVault; `/sip` stays SIP.
        assert!(looks_like_guest_account_request("/guest"));
        assert!(looks_like_guest_account_request("/guest-account"));
        assert!(looks_like_guest_account_request("/guestaccount"));
        assert!(looks_like_guest_account_request("/guest-status"));
        assert!(looks_like_guest_account_request("/guest-account-status"));
        assert!(looks_like_guest_account_request("/guest-user"));
        assert!(looks_like_guest_account_request("/guest-login"));
        assert!(looks_like_guest_account_request("/mac-guest"));
        assert!(looks_like_guest_account_request("/mac-guest-account"));
        assert!(looks_like_guest_account_request("view guest account"));
        assert!(looks_like_guest_account_request("open guest status"));
        assert!(looks_like_guest_account_request("list guest account"));
        assert!(looks_like_guest_account_request("guest account status"));
        assert!(looks_like_guest_account_request("is guest account on"));
        assert!(looks_like_guest_account_request("is guest enabled"));
        assert!(looks_like_guest_account_request("what's my guest account"));
        assert!(looks_like_guest_account_request("view guest user"));
        assert!(!looks_like_guest_account_request("guest"));
        assert!(!looks_like_guest_account_request("guest account"));
        assert!(!looks_like_guest_account_request("guest user"));
        assert!(!looks_like_guest_account_request("/username"));
        assert!(!looks_like_guest_account_request("/whoami"));
        assert!(!looks_like_guest_account_request("view username"));
        assert!(!looks_like_guest_account_request("/airdrop"));
        assert!(!looks_like_guest_account_request("view airdrop"));
        assert!(!looks_like_guest_account_request("/filevault"));
        assert!(!looks_like_guest_account_request("/sip"));
        assert!(!looks_like_guest_account_request("/tasks"));
        assert!(!looks_like_guest_account_request("enable guest account"));
        assert!(!looks_like_username_request("/guest"));
        assert!(!looks_like_username_request("view guest account"));
        assert!(!looks_like_airdrop_request("/guest"));
        assert!(!looks_like_airdrop_request("view guest account"));
        let g = try_operator_instant_reply("/guest").unwrap();
        assert!(
            g.contains("**Guest Account**") && !g.to_lowercase().contains("execution"),
            "got: {g}"
        );
        let viewed = try_operator_instant_reply("view guest account").unwrap();
        assert!(
            viewed.contains("**Guest Account**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let user = try_operator_instant_reply("view guest user").unwrap();
        assert!(
            user.contains("**Guest Account**") && !user.to_lowercase().contains("execution"),
            "got: {user}"
        );
        assert!(try_operator_instant_reply("guest").is_none());
        assert!(try_operator_instant_reply("guest account").is_none());
        let un = try_operator_instant_reply("/username");
        if let Some(reply) = un {
            assert!(
                !reply.contains("**Guest Account**"),
                "/username must stay User, not Guest Account: {reply}"
            );
        }
        let ad = try_operator_instant_reply("/airdrop");
        if let Some(reply) = ad {
            assert!(
                !reply.contains("**Guest Account**"),
                "/airdrop must stay AirDrop, not Guest Account: {reply}"
            );
        }
    }

    #[test]
    fn autologin_slash_and_view_open_do_not_steal_guest_username_or_bare_autologin() {
        // Slash + view/open NL → Automatic Login (`/guest` · `/username` parity).
        // Bare `autologin` / `automatic login` stay with the agent; `/guest` · `/guest-login` stay
        // Guest Account; `/username` · `/whoami` stay login name; `/remote-login` · `/ssh` stay
        // Remote Login; `/filevault` stays FileVault.
        assert!(looks_like_autologin_request("/autologin"));
        assert!(looks_like_autologin_request("/auto-login"));
        assert!(looks_like_autologin_request("/automatic-login"));
        assert!(looks_like_autologin_request("/autologin-status"));
        assert!(looks_like_autologin_request("/auto-login-status"));
        assert!(looks_like_autologin_request("/mac-autologin"));
        assert!(looks_like_autologin_request("/mac-auto-login"));
        assert!(looks_like_autologin_request("view automatic login"));
        assert!(looks_like_autologin_request("open auto login"));
        assert!(looks_like_autologin_request("list automatic login"));
        assert!(looks_like_autologin_request("automatic login status"));
        assert!(looks_like_autologin_request("is automatic login on"));
        assert!(looks_like_autologin_request("is autologin enabled"));
        assert!(looks_like_autologin_request("what's my automatic login"));
        assert!(looks_like_autologin_request("view autologin"));
        assert!(!looks_like_autologin_request("autologin"));
        assert!(!looks_like_autologin_request("automatic login"));
        assert!(!looks_like_autologin_request("auto login"));
        assert!(!looks_like_autologin_request("/guest"));
        assert!(!looks_like_autologin_request("/guest-login"));
        assert!(!looks_like_autologin_request("view guest account"));
        assert!(!looks_like_autologin_request("/username"));
        assert!(!looks_like_autologin_request("/whoami"));
        assert!(!looks_like_autologin_request("/remote-login"));
        assert!(!looks_like_autologin_request("/ssh"));
        assert!(!looks_like_autologin_request("/filevault"));
        assert!(!looks_like_autologin_request("/tasks"));
        assert!(!looks_like_autologin_request("enable automatic login"));
        assert!(!looks_like_guest_account_request("/autologin"));
        assert!(!looks_like_guest_account_request("view automatic login"));
        assert!(!looks_like_username_request("/autologin"));
        assert!(!looks_like_username_request("view automatic login"));
        assert!(!looks_like_remote_login_request("/autologin"));
        assert!(!looks_like_remote_login_request("view automatic login"));
        let a = try_operator_instant_reply("/autologin").unwrap();
        assert!(
            a.contains("**Automatic Login**") && !a.to_lowercase().contains("execution"),
            "got: {a}"
        );
        let viewed = try_operator_instant_reply("view automatic login").unwrap();
        assert!(
            viewed.contains("**Automatic Login**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open auto login").unwrap();
        assert!(
            open.contains("**Automatic Login**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("autologin").is_none());
        assert!(try_operator_instant_reply("automatic login").is_none());
        let g = try_operator_instant_reply("/guest");
        if let Some(reply) = g {
            assert!(
                !reply.contains("**Automatic Login**"),
                "/guest must stay Guest Account, not Automatic Login: {reply}"
            );
        }
        let un = try_operator_instant_reply("/username");
        if let Some(reply) = un {
            assert!(
                !reply.contains("**Automatic Login**"),
                "/username must stay User, not Automatic Login: {reply}"
            );
        }
        let rl = try_operator_instant_reply("/remote-login");
        if let Some(reply) = rl {
            assert!(
                !reply.contains("**Automatic Login**"),
                "/remote-login must stay Remote Login, not Automatic Login: {reply}"
            );
        }
    }

    #[test]
    fn screen_lock_slash_and_view_open_do_not_steal_screen_sharing_display_or_bare_lock() {
        // Slash + view/open NL → Screen Lock (`/autologin` · `/screen-sharing` parity).
        // Bare `screen lock` / `lock screen` / `lock` stay with the agent; `/screen-sharing` · `/vnc`
        // stay Screen Sharing; `/screen` · `/display` stay display resolution; `/autologin` stays
        // Automatic Login.
        assert!(looks_like_screen_lock_request("/screen-lock"));
        assert!(looks_like_screen_lock_request("/screenlock"));
        assert!(looks_like_screen_lock_request("/lock-screen"));
        assert!(looks_like_screen_lock_request("/lockscreen"));
        assert!(looks_like_screen_lock_request("/screen-lock-status"));
        assert!(looks_like_screen_lock_request("/lock-screen-status"));
        assert!(looks_like_screen_lock_request("/mac-screen-lock"));
        assert!(looks_like_screen_lock_request("/mac-lock-screen"));
        assert!(looks_like_screen_lock_request("view screen lock"));
        assert!(looks_like_screen_lock_request("open lock screen"));
        assert!(looks_like_screen_lock_request("list screen lock"));
        assert!(looks_like_screen_lock_request("screen lock status"));
        assert!(looks_like_screen_lock_request("is screen lock on"));
        assert!(looks_like_screen_lock_request("is lock screen enabled"));
        assert!(looks_like_screen_lock_request("what's my screen lock"));
        assert!(looks_like_screen_lock_request("view lock-screen"));
        assert!(!looks_like_screen_lock_request("screen lock"));
        assert!(!looks_like_screen_lock_request("lock screen"));
        assert!(!looks_like_screen_lock_request("lock"));
        assert!(!looks_like_screen_lock_request("screensaver"));
        assert!(!looks_like_screen_lock_request("/screen-sharing"));
        assert!(!looks_like_screen_lock_request("/vnc"));
        assert!(!looks_like_screen_lock_request("view screen sharing"));
        assert!(!looks_like_screen_lock_request("/screen"));
        assert!(!looks_like_screen_lock_request("/screens"));
        assert!(!looks_like_screen_lock_request("/display"));
        assert!(!looks_like_screen_lock_request("/autologin"));
        assert!(!looks_like_screen_lock_request("view automatic login"));
        assert!(!looks_like_screen_lock_request("/filevault"));
        assert!(!looks_like_screen_lock_request("/tasks"));
        assert!(!looks_like_screen_lock_request("enable screen lock"));
        assert!(!looks_like_screen_sharing_request("/screen-lock"));
        assert!(!looks_like_screen_sharing_request("view screen lock"));
        assert!(!looks_like_display_resolution_request("/screen-lock"));
        assert!(!looks_like_display_resolution_request("view screen lock"));
        assert!(!looks_like_autologin_request("/screen-lock"));
        assert!(!looks_like_autologin_request("view screen lock"));
        let s = try_operator_instant_reply("/screen-lock").unwrap();
        assert!(
            s.contains("**Screen Lock**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view screen lock").unwrap();
        assert!(
            viewed.contains("**Screen Lock**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open lock screen").unwrap();
        assert!(
            open.contains("**Screen Lock**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("screen lock").is_none());
        assert!(try_operator_instant_reply("lock screen").is_none());
        assert!(try_operator_instant_reply("lock").is_none());
        let ss = try_operator_instant_reply("/screen-sharing");
        if let Some(reply) = ss {
            assert!(
                !reply.contains("**Screen Lock**"),
                "/screen-sharing must stay Screen Sharing, not Screen Lock: {reply}"
            );
        }
        let disp = try_operator_instant_reply("/display");
        if let Some(reply) = disp {
            assert!(
                !reply.contains("**Screen Lock**"),
                "/display must stay Display, not Screen Lock: {reply}"
            );
        }
        let al = try_operator_instant_reply("/autologin");
        if let Some(reply) = al {
            assert!(
                !reply.contains("**Screen Lock**"),
                "/autologin must stay Automatic Login, not Screen Lock: {reply}"
            );
        }
    }

    #[test]
    fn smb_guest_slash_and_view_open_do_not_steal_smb_guest_or_bare_access() {
        // Slash + view/open NL → SMB Guest Access (`/smb` · `/guest` parity).
        // Bare `smb guest` / `guest access` stay with the agent; `/smb` · `/file-sharing` stay File
        // Sharing; `/guest` · `/guest-account` stay Guest Account; `/screen-lock` stays Screen Lock.
        assert!(looks_like_smb_guest_request("/smb-guest"));
        assert!(looks_like_smb_guest_request("/smbguest"));
        assert!(looks_like_smb_guest_request("/smb-guest-access"));
        assert!(looks_like_smb_guest_request("/guest-access"));
        assert!(looks_like_smb_guest_request("/guestaccess"));
        assert!(looks_like_smb_guest_request("/smb-guest-status"));
        assert!(looks_like_smb_guest_request("/mac-smb-guest"));
        assert!(looks_like_smb_guest_request("/mac-guest-access"));
        assert!(looks_like_smb_guest_request("view smb guest"));
        assert!(looks_like_smb_guest_request("open guest access"));
        assert!(looks_like_smb_guest_request("list smb guest"));
        assert!(looks_like_smb_guest_request("smb guest status"));
        assert!(looks_like_smb_guest_request("is smb guest on"));
        assert!(looks_like_smb_guest_request("is guest access enabled"));
        assert!(looks_like_smb_guest_request("what's my smb guest"));
        assert!(looks_like_smb_guest_request("view guest-access"));
        assert!(!looks_like_smb_guest_request("smb guest"));
        assert!(!looks_like_smb_guest_request("guest access"));
        assert!(!looks_like_smb_guest_request("smb guest access"));
        assert!(!looks_like_smb_guest_request("/smb"));
        assert!(!looks_like_smb_guest_request("/file-sharing"));
        assert!(!looks_like_smb_guest_request("view smb"));
        assert!(!looks_like_smb_guest_request("/guest"));
        assert!(!looks_like_smb_guest_request("/guest-account"));
        assert!(!looks_like_smb_guest_request("view guest account"));
        assert!(!looks_like_smb_guest_request("/screen-lock"));
        assert!(!looks_like_smb_guest_request("/tasks"));
        assert!(!looks_like_smb_guest_request("enable smb guest"));
        assert!(!looks_like_file_sharing_request("/smb-guest"));
        assert!(!looks_like_file_sharing_request("view smb guest"));
        assert!(!looks_like_guest_account_request("/smb-guest"));
        assert!(!looks_like_guest_account_request("view smb guest"));
        assert!(!looks_like_guest_account_request("/guest-access"));
        assert!(!looks_like_guest_account_request("view guest access"));
        assert!(!looks_like_screen_lock_request("/smb-guest"));
        assert!(!looks_like_screen_lock_request("view smb guest"));
        let s = try_operator_instant_reply("/smb-guest").unwrap();
        assert!(
            s.contains("**SMB Guest**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view smb guest").unwrap();
        assert!(
            viewed.contains("**SMB Guest**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open guest access").unwrap();
        assert!(
            open.contains("**SMB Guest**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("smb guest").is_none());
        assert!(try_operator_instant_reply("guest access").is_none());
        let smb = try_operator_instant_reply("/smb");
        if let Some(reply) = smb {
            assert!(
                !reply.contains("**SMB Guest**"),
                "/smb must stay File Sharing, not SMB Guest: {reply}"
            );
        }
        let g = try_operator_instant_reply("/guest");
        if let Some(reply) = g {
            assert!(
                !reply.contains("**SMB Guest**"),
                "/guest must stay Guest Account, not SMB Guest: {reply}"
            );
        }
        let sl = try_operator_instant_reply("/screen-lock");
        if let Some(reply) = sl {
            assert!(
                !reply.contains("**SMB Guest**"),
                "/screen-lock must stay Screen Lock, not SMB Guest: {reply}"
            );
        }
        assert!(!looks_like_smb_guest_request("/screensaver"));
        assert!(!looks_like_smb_guest_request("view screensaver"));
        assert!(!looks_like_smb_guest_request("/idle-time"));
    }

    #[test]
    fn screensaver_idle_slash_and_view_open_do_not_steal_screen_lock_or_bare_saver() {
        // Slash + view/open NL → Screensaver idle (`/screen-lock` · `/screen` parity).
        // Bare `screensaver` / `screen saver` / `idle time` stay with the agent; `/screen-lock` ·
        // `/lock-screen` stay Screen Lock; `/screen-sharing` · `/vnc` stay Screen Sharing; `/screen` ·
        // `/display` stay display resolution; `/smb-guest` stays SMB Guest.
        assert!(looks_like_screensaver_idle_request("/screensaver"));
        assert!(looks_like_screensaver_idle_request("/screen-saver"));
        assert!(looks_like_screensaver_idle_request("/screensaver-idle"));
        assert!(looks_like_screensaver_idle_request("/idle-time"));
        assert!(looks_like_screensaver_idle_request("/idletime"));
        assert!(looks_like_screensaver_idle_request("/screensaver-status"));
        assert!(looks_like_screensaver_idle_request("/mac-screensaver"));
        assert!(looks_like_screensaver_idle_request("/mac-idle-time"));
        assert!(looks_like_screensaver_idle_request("view screensaver"));
        assert!(looks_like_screensaver_idle_request("open screen saver"));
        assert!(looks_like_screensaver_idle_request("list idle time"));
        assert!(looks_like_screensaver_idle_request("screensaver status"));
        assert!(looks_like_screensaver_idle_request("is screensaver on"));
        assert!(looks_like_screensaver_idle_request("is screen saver enabled"));
        assert!(looks_like_screensaver_idle_request("what's my screensaver"));
        assert!(looks_like_screensaver_idle_request("what's my idle time"));
        assert!(looks_like_screensaver_idle_request("view screensaver idle"));
        assert!(!looks_like_screensaver_idle_request("screensaver"));
        assert!(!looks_like_screensaver_idle_request("screen saver"));
        assert!(!looks_like_screensaver_idle_request("idle time"));
        assert!(!looks_like_screensaver_idle_request("idle"));
        assert!(!looks_like_screensaver_idle_request("/screen-lock"));
        assert!(!looks_like_screensaver_idle_request("/lock-screen"));
        assert!(!looks_like_screensaver_idle_request("view screen lock"));
        assert!(!looks_like_screensaver_idle_request("/screen-sharing"));
        assert!(!looks_like_screensaver_idle_request("/vnc"));
        assert!(!looks_like_screensaver_idle_request("/screen"));
        assert!(!looks_like_screensaver_idle_request("/display"));
        assert!(!looks_like_screensaver_idle_request("/smb-guest"));
        assert!(!looks_like_screensaver_idle_request("/tasks"));
        assert!(!looks_like_screensaver_idle_request("enable screensaver"));
        assert!(!looks_like_screen_lock_request("/screensaver"));
        assert!(!looks_like_screen_lock_request("view screensaver"));
        assert!(!looks_like_screen_lock_request("/idle-time"));
        assert!(!looks_like_smb_guest_request("/screensaver"));
        assert!(!looks_like_smb_guest_request("view screensaver"));
        let s = try_operator_instant_reply("/screensaver").unwrap();
        assert!(
            s.contains("**Screensaver**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view screensaver").unwrap();
        assert!(
            viewed.contains("**Screensaver**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open idle time").unwrap();
        assert!(
            open.contains("**Screensaver**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("screensaver").is_none());
        assert!(try_operator_instant_reply("screen saver").is_none());
        assert!(try_operator_instant_reply("idle time").is_none());
        let sl = try_operator_instant_reply("/screen-lock");
        if let Some(reply) = sl {
            assert!(
                !reply.contains("**Screensaver**"),
                "/screen-lock must stay Screen Lock, not Screensaver: {reply}"
            );
        }
        let screen = try_operator_instant_reply("/screen");
        if let Some(reply) = screen {
            assert!(
                !reply.contains("**Screensaver**"),
                "/screen must stay display, not Screensaver: {reply}"
            );
        }
        let smb = try_operator_instant_reply("/smb-guest");
        if let Some(reply) = smb {
            assert!(
                !reply.contains("**Screensaver**"),
                "/smb-guest must stay SMB Guest, not Screensaver: {reply}"
            );
        }
        assert!(!looks_like_screensaver_idle_request("/powernap"));
        assert!(!looks_like_screensaver_idle_request("view power nap"));
    }

    #[test]
    fn power_nap_slash_and_view_open_do_not_steal_lpm_power_or_bare_nap() {
        // Slash + view/open NL → Power Nap (`/lpm` · `/screensaver` parity).
        // Bare `powernap` / `power nap` stay with the agent; `/lpm` stays Low Power Mode;
        // `/power` stays the strip; `/watts` stays draw; `/screensaver` stays Screensaver idle.
        assert!(looks_like_power_nap_request("/powernap"));
        assert!(looks_like_power_nap_request("/power-nap"));
        assert!(looks_like_power_nap_request("/powernap-status"));
        assert!(looks_like_power_nap_request("/power-nap-status"));
        assert!(looks_like_power_nap_request("/mac-powernap"));
        assert!(looks_like_power_nap_request("/mac-power-nap"));
        assert!(looks_like_power_nap_request("view power nap"));
        assert!(looks_like_power_nap_request("open powernap"));
        assert!(looks_like_power_nap_request("list power nap"));
        assert!(looks_like_power_nap_request("power nap status"));
        assert!(looks_like_power_nap_request("is power nap on"));
        assert!(looks_like_power_nap_request("is powernap enabled"));
        assert!(looks_like_power_nap_request("what's my power nap"));
        assert!(looks_like_power_nap_request("view power-nap"));
        assert!(!looks_like_power_nap_request("powernap"));
        assert!(!looks_like_power_nap_request("power nap"));
        assert!(!looks_like_power_nap_request("/lpm"));
        assert!(!looks_like_power_nap_request("view lpm"));
        assert!(!looks_like_power_nap_request("low power mode"));
        assert!(!looks_like_power_nap_request("/power"));
        assert!(!looks_like_power_nap_request("view power"));
        assert!(!looks_like_power_nap_request("/watts"));
        assert!(!looks_like_power_nap_request("/screensaver"));
        assert!(!looks_like_power_nap_request("view screensaver"));
        assert!(!looks_like_power_nap_request("/battery"));
        assert!(!looks_like_power_nap_request("/tasks"));
        assert!(!looks_like_power_nap_request("enable power nap"));
        assert!(!looks_like_power_nap_request("wake for network"));
        assert!(!looks_like_screensaver_idle_request("/powernap"));
        assert!(!looks_like_screensaver_idle_request("view power nap"));
        let s = try_operator_instant_reply("/powernap").unwrap();
        assert!(
            s.contains("**Power Nap**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view power nap").unwrap();
        assert!(
            viewed.contains("**Power Nap**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open powernap").unwrap();
        assert!(
            open.contains("**Power Nap**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("powernap").is_none());
        assert!(try_operator_instant_reply("power nap").is_none());
        let lpm = try_operator_instant_reply("/lpm");
        if let Some(reply) = lpm {
            assert!(
                !reply.contains("**Power Nap**"),
                "/lpm must stay Low Power Mode, not Power Nap: {reply}"
            );
        }
        let ss = try_operator_instant_reply("/screensaver");
        if let Some(reply) = ss {
            assert!(
                !reply.contains("**Power Nap**"),
                "/screensaver must stay Screensaver, not Power Nap: {reply}"
            );
        }
        let power = try_operator_instant_reply("/power");
        if let Some(reply) = power {
            assert!(
                !reply.contains("**Power Nap**"),
                "/power must stay strip, not Power Nap: {reply}"
            );
        }
    }

    #[test]
    fn wake_for_network_slash_and_view_open_do_not_steal_powernap_wifi_or_bare_womp() {
        // Slash + view/open NL → Wake for network (`/powernap` · `/wifi` parity).
        // Bare `womp` / `wake for network` / `wake on lan` / `wol` stay with the agent;
        // `/powernap` stays Power Nap; `/lpm` stays Low Power Mode; `/power` stays the strip;
        // `/wifi` stays SSID; `/vpn` stays VPN.
        assert!(looks_like_wake_for_network_request("/womp"));
        assert!(looks_like_wake_for_network_request("/womp-status"));
        assert!(looks_like_wake_for_network_request("/wake-for-network"));
        assert!(looks_like_wake_for_network_request("/wake-on-lan"));
        assert!(looks_like_wake_for_network_request("/wol"));
        assert!(looks_like_wake_for_network_request("/mac-womp"));
        assert!(looks_like_wake_for_network_request("/mac-wake-for-network"));
        assert!(looks_like_wake_for_network_request("view wake for network"));
        assert!(looks_like_wake_for_network_request("open womp"));
        assert!(looks_like_wake_for_network_request("list wake on lan"));
        assert!(looks_like_wake_for_network_request("womp status"));
        assert!(looks_like_wake_for_network_request("is wake for network on"));
        assert!(looks_like_wake_for_network_request("is womp enabled"));
        assert!(looks_like_wake_for_network_request("what's my womp"));
        assert!(looks_like_wake_for_network_request("view wake-on-lan"));
        assert!(!looks_like_wake_for_network_request("womp"));
        assert!(!looks_like_wake_for_network_request("wake for network"));
        assert!(!looks_like_wake_for_network_request("wake on lan"));
        assert!(!looks_like_wake_for_network_request("wol"));
        assert!(!looks_like_wake_for_network_request("/powernap"));
        assert!(!looks_like_wake_for_network_request("view power nap"));
        assert!(!looks_like_wake_for_network_request("/lpm"));
        assert!(!looks_like_wake_for_network_request("/power"));
        assert!(!looks_like_wake_for_network_request("/wifi"));
        assert!(!looks_like_wake_for_network_request("/vpn"));
        assert!(!looks_like_wake_for_network_request("/tasks"));
        assert!(!looks_like_wake_for_network_request("enable womp"));
        assert!(!looks_like_power_nap_request("/womp"));
        assert!(!looks_like_power_nap_request("view wake for network"));
        let s = try_operator_instant_reply("/womp").unwrap();
        assert!(
            s.contains("**Wake for network**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view wake for network").unwrap();
        assert!(
            viewed.contains("**Wake for network**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open womp").unwrap();
        assert!(
            open.contains("**Wake for network**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("womp").is_none());
        assert!(try_operator_instant_reply("wake for network").is_none());
        assert!(try_operator_instant_reply("wake on lan").is_none());
        assert!(try_operator_instant_reply("wol").is_none());
        let nap = try_operator_instant_reply("/powernap");
        if let Some(reply) = nap {
            assert!(
                !reply.contains("**Wake for network**"),
                "/powernap must stay Power Nap, not Wake for network: {reply}"
            );
        }
        let wifi = try_operator_instant_reply("/wifi");
        if let Some(reply) = wifi {
            assert!(
                !reply.contains("**Wake for network**"),
                "/wifi must stay SSID, not Wake for network: {reply}"
            );
        }
        let power = try_operator_instant_reply("/power");
        if let Some(reply) = power {
            assert!(
                !reply.contains("**Wake for network**"),
                "/power must stay strip, not Wake for network: {reply}"
            );
        }
    }

    #[test]
    fn displaysleep_slash_and_view_open_do_not_steal_screensaver_display_or_bare() {
        // Slash + view/open NL → Display Sleep (`/screensaver` · `/display` parity).
        // Bare `displaysleep` / `display sleep` / `screen sleep` stay with the agent;
        // `/screensaver` stays Screensaver idle; `/display` · `/screen` stay resolution;
        // `/powernap` stays Power Nap; `/womp` stays Wake for network.
        assert!(looks_like_displaysleep_request("/displaysleep"));
        assert!(looks_like_displaysleep_request("/display-sleep"));
        assert!(looks_like_displaysleep_request("/displaysleep-status"));
        assert!(looks_like_displaysleep_request("/screen-sleep"));
        assert!(looks_like_displaysleep_request("/mac-displaysleep"));
        assert!(looks_like_displaysleep_request("/mac-display-sleep"));
        assert!(looks_like_displaysleep_request("view display sleep"));
        assert!(looks_like_displaysleep_request("open displaysleep"));
        assert!(looks_like_displaysleep_request("list screen sleep"));
        assert!(looks_like_displaysleep_request("display sleep status"));
        assert!(looks_like_displaysleep_request("is display sleep on"));
        assert!(looks_like_displaysleep_request("is displaysleep enabled"));
        assert!(looks_like_displaysleep_request("what's my display sleep"));
        assert!(looks_like_displaysleep_request("view screen-sleep"));
        assert!(!looks_like_displaysleep_request("displaysleep"));
        assert!(!looks_like_displaysleep_request("display sleep"));
        assert!(!looks_like_displaysleep_request("screen sleep"));
        assert!(!looks_like_displaysleep_request("/screensaver"));
        assert!(!looks_like_displaysleep_request("view screensaver"));
        assert!(!looks_like_displaysleep_request("/idle-time"));
        assert!(!looks_like_displaysleep_request("/display"));
        assert!(!looks_like_displaysleep_request("/screen"));
        assert!(!looks_like_displaysleep_request("view display"));
        assert!(!looks_like_displaysleep_request("/powernap"));
        assert!(!looks_like_displaysleep_request("/womp"));
        assert!(!looks_like_displaysleep_request("/power"));
        assert!(!looks_like_displaysleep_request("/tasks"));
        assert!(!looks_like_displaysleep_request("enable display sleep"));
        assert!(!looks_like_displaysleep_request("disk sleep"));
        assert!(!looks_like_displaysleep_request("/disksleep"));
        assert!(!looks_like_screensaver_idle_request("/displaysleep"));
        assert!(!looks_like_screensaver_idle_request("view display sleep"));
        assert!(!looks_like_power_nap_request("/displaysleep"));
        assert!(!looks_like_wake_for_network_request("/displaysleep"));
        let s = try_operator_instant_reply("/displaysleep").unwrap();
        assert!(
            s.contains("**Display Sleep**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view display sleep").unwrap();
        assert!(
            viewed.contains("**Display Sleep**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open displaysleep").unwrap();
        assert!(
            open.contains("**Display Sleep**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("displaysleep").is_none());
        assert!(try_operator_instant_reply("display sleep").is_none());
        assert!(try_operator_instant_reply("screen sleep").is_none());
        let ss = try_operator_instant_reply("/screensaver");
        if let Some(reply) = ss {
            assert!(
                !reply.contains("**Display Sleep**"),
                "/screensaver must stay Screensaver, not Display Sleep: {reply}"
            );
        }
        let disp = try_operator_instant_reply("/display");
        if let Some(reply) = disp {
            assert!(
                !reply.contains("**Display Sleep**"),
                "/display must stay resolution, not Display Sleep: {reply}"
            );
        }
        let nap = try_operator_instant_reply("/powernap");
        if let Some(reply) = nap {
            assert!(
                !reply.contains("**Display Sleep**"),
                "/powernap must stay Power Nap, not Display Sleep: {reply}"
            );
        }
        let womp = try_operator_instant_reply("/womp");
        if let Some(reply) = womp {
            assert!(
                !reply.contains("**Display Sleep**"),
                "/womp must stay Wake for network, not Display Sleep: {reply}"
            );
        }
    }

    #[test]
    fn disksleep_slash_and_view_open_do_not_steal_displaysleep_cleanup_or_bare() {
        // Slash + view/open NL → Disk Sleep (`/displaysleep` · Disk Cleanup parity).
        // Bare `disksleep` / `disk sleep` / `hard disk sleep` stay with the agent;
        // `/displaysleep` stays Display Sleep; `/screensaver` stays Screensaver;
        // `/ssd` / `/free` / Disk Cleanup stay put.
        assert!(looks_like_disksleep_request("/disksleep"));
        assert!(looks_like_disksleep_request("/disk-sleep"));
        assert!(looks_like_disksleep_request("/disksleep-status"));
        assert!(looks_like_disksleep_request("/hard-disk-sleep"));
        assert!(looks_like_disksleep_request("/hdd-sleep"));
        assert!(looks_like_disksleep_request("/mac-disksleep"));
        assert!(looks_like_disksleep_request("/mac-disk-sleep"));
        assert!(looks_like_disksleep_request("view disk sleep"));
        assert!(looks_like_disksleep_request("open disksleep"));
        assert!(looks_like_disksleep_request("list hard disk sleep"));
        assert!(looks_like_disksleep_request("disk sleep status"));
        assert!(looks_like_disksleep_request("is disk sleep on"));
        assert!(looks_like_disksleep_request("is disksleep enabled"));
        assert!(looks_like_disksleep_request("what's my disk sleep"));
        assert!(looks_like_disksleep_request("view hdd-sleep"));
        assert!(!looks_like_disksleep_request("disksleep"));
        assert!(!looks_like_disksleep_request("disk sleep"));
        assert!(!looks_like_disksleep_request("hard disk sleep"));
        assert!(!looks_like_disksleep_request("hdd sleep"));
        assert!(!looks_like_disksleep_request("/displaysleep"));
        assert!(!looks_like_disksleep_request("view display sleep"));
        assert!(!looks_like_disksleep_request("/screen-sleep"));
        assert!(!looks_like_disksleep_request("/screensaver"));
        assert!(!looks_like_disksleep_request("/display"));
        assert!(!looks_like_disksleep_request("/ssd"));
        assert!(!looks_like_disksleep_request("/free"));
        assert!(!looks_like_disksleep_request("/powernap"));
        assert!(!looks_like_disksleep_request("/womp"));
        assert!(!looks_like_disksleep_request("/power"));
        assert!(!looks_like_disksleep_request("/tasks"));
        assert!(!looks_like_disksleep_request("enable disk sleep"));
        assert!(!looks_like_disksleep_request("view disk"));
        assert!(!looks_like_displaysleep_request("/disksleep"));
        assert!(!looks_like_displaysleep_request("view disk sleep"));
        assert!(!looks_like_power_nap_request("/disksleep"));
        assert!(!looks_like_wake_for_network_request("/disksleep"));
        let s = try_operator_instant_reply("/disksleep").unwrap();
        assert!(
            s.contains("**Disk Sleep**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view disk sleep").unwrap();
        assert!(
            viewed.contains("**Disk Sleep**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open disksleep").unwrap();
        assert!(
            open.contains("**Disk Sleep**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("disksleep").is_none());
        assert!(try_operator_instant_reply("disk sleep").is_none());
        assert!(try_operator_instant_reply("hard disk sleep").is_none());
        let ds = try_operator_instant_reply("/displaysleep");
        if let Some(reply) = ds {
            assert!(
                !reply.contains("**Disk Sleep**"),
                "/displaysleep must stay Display Sleep, not Disk Sleep: {reply}"
            );
        }
        let ss = try_operator_instant_reply("/screensaver");
        if let Some(reply) = ss {
            assert!(
                !reply.contains("**Disk Sleep**"),
                "/screensaver must stay Screensaver, not Disk Sleep: {reply}"
            );
        }
        let nap = try_operator_instant_reply("/powernap");
        if let Some(reply) = nap {
            assert!(
                !reply.contains("**Disk Sleep**"),
                "/powernap must stay Power Nap, not Disk Sleep: {reply}"
            );
        }
        let womp = try_operator_instant_reply("/womp");
        if let Some(reply) = womp {
            assert!(
                !reply.contains("**Disk Sleep**"),
                "/womp must stay Wake for network, not Disk Sleep: {reply}"
            );
        }
    }

    #[test]
    fn proximitywake_slash_and_view_open_do_not_steal_womp_or_bare() {
        // Slash + view/open NL → Proximity Wake (`/womp` · `/powernap` parity).
        // Bare `proximitywake` / `proximity wake` stay with the agent;
        // `/womp` stays Wake for network; `/powernap` stays Power Nap;
        // `/displaysleep` / `/disksleep` stay timers.
        assert!(looks_like_proximitywake_request("/proximitywake"));
        assert!(looks_like_proximitywake_request("/proximity-wake"));
        assert!(looks_like_proximitywake_request("/proximitywake-status"));
        assert!(looks_like_proximitywake_request("/mac-proximitywake"));
        assert!(looks_like_proximitywake_request("/mac-proximity-wake"));
        assert!(looks_like_proximitywake_request("view proximity wake"));
        assert!(looks_like_proximitywake_request("open proximitywake"));
        assert!(looks_like_proximitywake_request("list proximity wake"));
        assert!(looks_like_proximitywake_request("proximity wake status"));
        assert!(looks_like_proximitywake_request("is proximity wake on"));
        assert!(looks_like_proximitywake_request("is proximitywake enabled"));
        assert!(looks_like_proximitywake_request("what's my proximity wake"));
        assert!(looks_like_proximitywake_request("view proximity-wake"));
        assert!(!looks_like_proximitywake_request("proximitywake"));
        assert!(!looks_like_proximitywake_request("proximity wake"));
        assert!(!looks_like_proximitywake_request("/womp"));
        assert!(!looks_like_proximitywake_request("view wake for network"));
        assert!(!looks_like_proximitywake_request("/powernap"));
        assert!(!looks_like_proximitywake_request("/displaysleep"));
        assert!(!looks_like_proximitywake_request("/disksleep"));
        assert!(!looks_like_proximitywake_request("/power"));
        assert!(!looks_like_proximitywake_request("/tasks"));
        assert!(!looks_like_proximitywake_request("enable proximity wake"));
        assert!(!looks_like_wake_for_network_request("/proximitywake"));
        assert!(!looks_like_wake_for_network_request("view proximity wake"));
        assert!(!looks_like_power_nap_request("/proximitywake"));
        assert!(!looks_like_displaysleep_request("/proximitywake"));
        assert!(!looks_like_disksleep_request("/proximitywake"));
        let s = try_operator_instant_reply("/proximitywake").unwrap();
        assert!(
            s.contains("**Proximity Wake**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view proximity wake").unwrap();
        assert!(
            viewed.contains("**Proximity Wake**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open proximitywake").unwrap();
        assert!(
            open.contains("**Proximity Wake**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("proximitywake").is_none());
        assert!(try_operator_instant_reply("proximity wake").is_none());
        let womp = try_operator_instant_reply("/womp");
        if let Some(reply) = womp {
            assert!(
                !reply.contains("**Proximity Wake**"),
                "/womp must stay Wake for network, not Proximity Wake: {reply}"
            );
        }
        let nap = try_operator_instant_reply("/powernap");
        if let Some(reply) = nap {
            assert!(
                !reply.contains("**Proximity Wake**"),
                "/powernap must stay Power Nap, not Proximity Wake: {reply}"
            );
        }
        let ds = try_operator_instant_reply("/disksleep");
        if let Some(reply) = ds {
            assert!(
                !reply.contains("**Proximity Wake**"),
                "/disksleep must stay Disk Sleep, not Proximity Wake: {reply}"
            );
        }
    }

    #[test]
    fn ttyskeepawake_slash_and_view_open_do_not_steal_proximitywake_or_bare() {
        // Slash + view/open NL → TTY keep awake (`/proximitywake` · `/womp` parity).
        // Bare `ttyskeepawake` / `tty keep awake` stay with the agent;
        // `/proximitywake` stays Proximity Wake; `/womp` stays Wake for network;
        // `/powernap` stays Power Nap; system sleep stays out.
        assert!(looks_like_ttyskeepawake_request("/ttyskeepawake"));
        assert!(looks_like_ttyskeepawake_request("/ttys-keep-awake"));
        assert!(looks_like_ttyskeepawake_request("/tty-keep-awake"));
        assert!(looks_like_ttyskeepawake_request("/ttyskeepawake-status"));
        assert!(looks_like_ttyskeepawake_request("/mac-ttyskeepawake"));
        assert!(looks_like_ttyskeepawake_request("/mac-tty-keep-awake"));
        assert!(looks_like_ttyskeepawake_request("view tty keep awake"));
        assert!(looks_like_ttyskeepawake_request("open ttyskeepawake"));
        assert!(looks_like_ttyskeepawake_request("list ttys keep awake"));
        assert!(looks_like_ttyskeepawake_request("tty keep awake status"));
        assert!(looks_like_ttyskeepawake_request("is tty keep awake on"));
        assert!(looks_like_ttyskeepawake_request("is ttyskeepawake enabled"));
        assert!(looks_like_ttyskeepawake_request("what's my tty keep awake"));
        assert!(looks_like_ttyskeepawake_request("view tty-keep-awake"));
        assert!(!looks_like_ttyskeepawake_request("ttyskeepawake"));
        assert!(!looks_like_ttyskeepawake_request("tty keep awake"));
        assert!(!looks_like_ttyskeepawake_request("ttys keep awake"));
        assert!(!looks_like_ttyskeepawake_request("/proximitywake"));
        assert!(!looks_like_ttyskeepawake_request("view proximity wake"));
        assert!(!looks_like_ttyskeepawake_request("/womp"));
        assert!(!looks_like_ttyskeepawake_request("/powernap"));
        assert!(!looks_like_ttyskeepawake_request("/displaysleep"));
        assert!(!looks_like_ttyskeepawake_request("/disksleep"));
        assert!(!looks_like_ttyskeepawake_request("/power"));
        assert!(!looks_like_ttyskeepawake_request("/tasks"));
        assert!(!looks_like_ttyskeepawake_request("enable tty keep awake"));
        assert!(!looks_like_proximitywake_request("/ttyskeepawake"));
        assert!(!looks_like_proximitywake_request("view tty keep awake"));
        assert!(!looks_like_wake_for_network_request("/ttyskeepawake"));
        assert!(!looks_like_power_nap_request("/ttyskeepawake"));
        let s = try_operator_instant_reply("/ttyskeepawake").unwrap();
        assert!(
            s.contains("**TTY keep awake**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view tty keep awake").unwrap();
        assert!(
            viewed.contains("**TTY keep awake**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open ttyskeepawake").unwrap();
        assert!(
            open.contains("**TTY keep awake**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("ttyskeepawake").is_none());
        assert!(try_operator_instant_reply("tty keep awake").is_none());
        assert!(try_operator_instant_reply("ttys keep awake").is_none());
        let prox = try_operator_instant_reply("/proximitywake");
        if let Some(reply) = prox {
            assert!(
                !reply.contains("**TTY keep awake**"),
                "/proximitywake must stay Proximity Wake, not TTY keep awake: {reply}"
            );
        }
        let womp = try_operator_instant_reply("/womp");
        if let Some(reply) = womp {
            assert!(
                !reply.contains("**TTY keep awake**"),
                "/womp must stay Wake for network, not TTY keep awake: {reply}"
            );
        }
        let nap = try_operator_instant_reply("/powernap");
        if let Some(reply) = nap {
            assert!(
                !reply.contains("**TTY keep awake**"),
                "/powernap must stay Power Nap, not TTY keep awake: {reply}"
            );
        }
    }

    #[test]
    fn systemsleep_slash_and_view_open_do_not_steal_displaysleep_or_bare() {
        // Slash + view/open NL → System Sleep (`/displaysleep` · `/disksleep` parity).
        // Bare `systemsleep` / `system sleep` / `computer sleep` / `sleep` stay with the agent;
        // `/displaysleep` stays Display Sleep; `/disksleep` stays Disk Sleep;
        // `/screensaver` stays Screensaver; `/ttyskeepawake` stays TTY keep awake.
        assert!(looks_like_systemsleep_request("/systemsleep"));
        assert!(looks_like_systemsleep_request("/system-sleep"));
        assert!(looks_like_systemsleep_request("/systemsleep-status"));
        assert!(looks_like_systemsleep_request("/computer-sleep"));
        assert!(looks_like_systemsleep_request("/mac-systemsleep"));
        assert!(looks_like_systemsleep_request("/mac-system-sleep"));
        assert!(looks_like_systemsleep_request("/sleep"));
        assert!(looks_like_systemsleep_request("view system sleep"));
        assert!(looks_like_systemsleep_request("open systemsleep"));
        assert!(looks_like_systemsleep_request("list computer sleep"));
        assert!(looks_like_systemsleep_request("system sleep status"));
        assert!(looks_like_systemsleep_request("is system sleep on"));
        assert!(looks_like_systemsleep_request("is systemsleep enabled"));
        assert!(looks_like_systemsleep_request("what's my system sleep"));
        assert!(looks_like_systemsleep_request("view system-sleep"));
        assert!(!looks_like_systemsleep_request("systemsleep"));
        assert!(!looks_like_systemsleep_request("system sleep"));
        assert!(!looks_like_systemsleep_request("computer sleep"));
        assert!(!looks_like_systemsleep_request("sleep"));
        assert!(!looks_like_systemsleep_request("/displaysleep"));
        assert!(!looks_like_systemsleep_request("view display sleep"));
        assert!(!looks_like_systemsleep_request("/screen-sleep"));
        assert!(!looks_like_systemsleep_request("/disksleep"));
        assert!(!looks_like_systemsleep_request("view disk sleep"));
        assert!(!looks_like_systemsleep_request("/screensaver"));
        assert!(!looks_like_systemsleep_request("/ttyskeepawake"));
        assert!(!looks_like_systemsleep_request("/powernap"));
        assert!(!looks_like_systemsleep_request("/womp"));
        assert!(!looks_like_systemsleep_request("/power"));
        assert!(!looks_like_systemsleep_request("/tasks"));
        assert!(!looks_like_systemsleep_request("enable system sleep"));
        assert!(!looks_like_displaysleep_request("/systemsleep"));
        assert!(!looks_like_displaysleep_request("view system sleep"));
        assert!(!looks_like_disksleep_request("/systemsleep"));
        assert!(!looks_like_disksleep_request("view system sleep"));
        assert!(!looks_like_ttyskeepawake_request("/systemsleep"));
        assert!(!looks_like_ttyskeepawake_request("view system sleep"));
        let s = try_operator_instant_reply("/systemsleep").unwrap();
        assert!(
            s.contains("**System Sleep**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view system sleep").unwrap();
        assert!(
            viewed.contains("**System Sleep**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open systemsleep").unwrap();
        assert!(
            open.contains("**System Sleep**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        let sleep_slash = try_operator_instant_reply("/sleep").unwrap();
        assert!(
            sleep_slash.contains("**System Sleep**")
                && !sleep_slash.to_lowercase().contains("execution"),
            "got: {sleep_slash}"
        );
        assert!(try_operator_instant_reply("systemsleep").is_none());
        assert!(try_operator_instant_reply("system sleep").is_none());
        assert!(try_operator_instant_reply("computer sleep").is_none());
        assert!(try_operator_instant_reply("sleep").is_none());
        let ds = try_operator_instant_reply("/displaysleep");
        if let Some(reply) = ds {
            assert!(
                !reply.contains("**System Sleep**"),
                "/displaysleep must stay Display Sleep, not System Sleep: {reply}"
            );
        }
        let disk = try_operator_instant_reply("/disksleep");
        if let Some(reply) = disk {
            assert!(
                !reply.contains("**System Sleep**"),
                "/disksleep must stay Disk Sleep, not System Sleep: {reply}"
            );
        }
        let tty = try_operator_instant_reply("/ttyskeepawake");
        if let Some(reply) = tty {
            assert!(
                !reply.contains("**System Sleep**"),
                "/ttyskeepawake must stay TTY keep awake, not System Sleep: {reply}"
            );
        }
    }

    #[test]
    fn tcpkeepalive_slash_and_view_open_do_not_steal_ttyskeepawake_or_bare() {
        // Slash + view/open NL → TCP Keep Alive (`/ttyskeepawake` · `/womp` parity).
        // Bare `tcpkeepalive` / `tcp keep alive` stay with the agent;
        // `/ttyskeepawake` stays TTY keep awake; `/womp` stays Wake for network;
        // `/systemsleep` stays System Sleep; networkoversleep stays out.
        assert!(looks_like_tcpkeepalive_request("/tcpkeepalive"));
        assert!(looks_like_tcpkeepalive_request("/tcp-keep-alive"));
        assert!(looks_like_tcpkeepalive_request("/tcpkeepalive-status"));
        assert!(looks_like_tcpkeepalive_request("/tcp-keepalive"));
        assert!(looks_like_tcpkeepalive_request("/mac-tcpkeepalive"));
        assert!(looks_like_tcpkeepalive_request("/mac-tcp-keep-alive"));
        assert!(looks_like_tcpkeepalive_request("view tcp keep alive"));
        assert!(looks_like_tcpkeepalive_request("open tcpkeepalive"));
        assert!(looks_like_tcpkeepalive_request("list tcp-keep-alive"));
        assert!(looks_like_tcpkeepalive_request("tcp keep alive status"));
        assert!(looks_like_tcpkeepalive_request("is tcp keep alive on"));
        assert!(looks_like_tcpkeepalive_request("is tcpkeepalive enabled"));
        assert!(looks_like_tcpkeepalive_request("what's my tcp keep alive"));
        assert!(looks_like_tcpkeepalive_request("view tcp-keep-alive"));
        assert!(!looks_like_tcpkeepalive_request("tcpkeepalive"));
        assert!(!looks_like_tcpkeepalive_request("tcp keep alive"));
        assert!(!looks_like_tcpkeepalive_request("/ttyskeepawake"));
        assert!(!looks_like_tcpkeepalive_request("view tty keep awake"));
        assert!(!looks_like_tcpkeepalive_request("/womp"));
        assert!(!looks_like_tcpkeepalive_request("/systemsleep"));
        assert!(!looks_like_tcpkeepalive_request("/powernap"));
        assert!(!looks_like_tcpkeepalive_request("/power"));
        assert!(!looks_like_tcpkeepalive_request("/tasks"));
        assert!(!looks_like_tcpkeepalive_request("enable tcp keep alive"));
        assert!(!looks_like_ttyskeepawake_request("/tcpkeepalive"));
        assert!(!looks_like_ttyskeepawake_request("view tcp keep alive"));
        assert!(!looks_like_systemsleep_request("/tcpkeepalive"));
        assert!(!looks_like_systemsleep_request("view tcp keep alive"));
        let s = try_operator_instant_reply("/tcpkeepalive").unwrap();
        assert!(
            s.contains("**TCP Keep Alive**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view tcp keep alive").unwrap();
        assert!(
            viewed.contains("**TCP Keep Alive**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open tcpkeepalive").unwrap();
        assert!(
            open.contains("**TCP Keep Alive**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("tcpkeepalive").is_none());
        assert!(try_operator_instant_reply("tcp keep alive").is_none());
        let tty = try_operator_instant_reply("/ttyskeepawake");
        if let Some(reply) = tty {
            assert!(
                !reply.contains("**TCP Keep Alive**"),
                "/ttyskeepawake must stay TTY keep awake, not TCP Keep Alive: {reply}"
            );
        }
        let sleep = try_operator_instant_reply("/systemsleep");
        if let Some(reply) = sleep {
            assert!(
                !reply.contains("**TCP Keep Alive**"),
                "/systemsleep must stay System Sleep, not TCP Keep Alive: {reply}"
            );
        }
        let womp = try_operator_instant_reply("/womp");
        if let Some(reply) = womp {
            assert!(
                !reply.contains("**TCP Keep Alive**"),
                "/womp must stay Wake for network, not TCP Keep Alive: {reply}"
            );
        }
        assert!(!looks_like_tcpkeepalive_request("/networkoversleep"));
        assert!(!looks_like_tcpkeepalive_request("view network over sleep"));
    }

    #[test]
    fn networkoversleep_slash_and_view_open_do_not_steal_tcpkeepalive_or_bare() {
        // Slash + view/open NL → Network over sleep (`/tcpkeepalive` · `/womp` parity).
        // Bare `networkoversleep` / `network over sleep` stay with the agent;
        // `/tcpkeepalive` stays TCP Keep Alive; `/womp` stays Wake for network;
        // `/systemsleep` stays System Sleep; hibernatemode stays out.
        assert!(looks_like_networkoversleep_request("/networkoversleep"));
        assert!(looks_like_networkoversleep_request("/network-over-sleep"));
        assert!(looks_like_networkoversleep_request("/networkoversleep-status"));
        assert!(looks_like_networkoversleep_request("/network-oversleep"));
        assert!(looks_like_networkoversleep_request("/mac-networkoversleep"));
        assert!(looks_like_networkoversleep_request("/mac-network-over-sleep"));
        assert!(looks_like_networkoversleep_request("view network over sleep"));
        assert!(looks_like_networkoversleep_request("open networkoversleep"));
        assert!(looks_like_networkoversleep_request("list network-over-sleep"));
        assert!(looks_like_networkoversleep_request("network over sleep status"));
        assert!(looks_like_networkoversleep_request("is network over sleep on"));
        assert!(looks_like_networkoversleep_request("is networkoversleep enabled"));
        assert!(looks_like_networkoversleep_request("what's my network over sleep"));
        assert!(looks_like_networkoversleep_request("view network-over-sleep"));
        assert!(!looks_like_networkoversleep_request("networkoversleep"));
        assert!(!looks_like_networkoversleep_request("network over sleep"));
        assert!(!looks_like_networkoversleep_request("/tcpkeepalive"));
        assert!(!looks_like_networkoversleep_request("view tcp keep alive"));
        assert!(!looks_like_networkoversleep_request("/womp"));
        assert!(!looks_like_networkoversleep_request("/systemsleep"));
        assert!(!looks_like_networkoversleep_request("/powernap"));
        assert!(!looks_like_networkoversleep_request("/power"));
        assert!(!looks_like_networkoversleep_request("/tasks"));
        assert!(!looks_like_networkoversleep_request("enable network over sleep"));
        assert!(!looks_like_networkoversleep_request("/hibernate"));
        assert!(!looks_like_tcpkeepalive_request("/networkoversleep"));
        assert!(!looks_like_tcpkeepalive_request("view network over sleep"));
        assert!(!looks_like_systemsleep_request("/networkoversleep"));
        assert!(!looks_like_systemsleep_request("view network over sleep"));
        let s = try_operator_instant_reply("/networkoversleep").unwrap();
        assert!(
            s.contains("**Network over sleep**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view network over sleep").unwrap();
        assert!(
            viewed.contains("**Network over sleep**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open networkoversleep").unwrap();
        assert!(
            open.contains("**Network over sleep**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("networkoversleep").is_none());
        assert!(try_operator_instant_reply("network over sleep").is_none());
        let tcp = try_operator_instant_reply("/tcpkeepalive");
        if let Some(reply) = tcp {
            assert!(
                !reply.contains("**Network over sleep**"),
                "/tcpkeepalive must stay TCP Keep Alive, not Network over sleep: {reply}"
            );
        }
        let sleep = try_operator_instant_reply("/systemsleep");
        if let Some(reply) = sleep {
            assert!(
                !reply.contains("**Network over sleep**"),
                "/systemsleep must stay System Sleep, not Network over sleep: {reply}"
            );
        }
        let womp = try_operator_instant_reply("/womp");
        if let Some(reply) = womp {
            assert!(
                !reply.contains("**Network over sleep**"),
                "/womp must stay Wake for network, not Network over sleep: {reply}"
            );
        }
        assert!(!looks_like_networkoversleep_request("/hibernatemode"));
        assert!(!looks_like_networkoversleep_request("view hibernate mode"));
    }

    #[test]
    fn hibernatemode_slash_and_view_open_do_not_steal_systemsleep_or_bare() {
        // Slash + view/open NL → Hibernate mode (`/networkoversleep` · `/systemsleep` parity).
        // Bare `hibernatemode` / `hibernate` / `hibernate mode` stay with the agent;
        // `/systemsleep` stays System Sleep; `/networkoversleep` stays Network over sleep;
        // `/tcpkeepalive` stays TCP Keep Alive; `/womp` stays Wake for network.
        assert!(looks_like_hibernatemode_request("/hibernatemode"));
        assert!(looks_like_hibernatemode_request("/hibernate-mode"));
        assert!(looks_like_hibernatemode_request("/hibernatemode-status"));
        assert!(looks_like_hibernatemode_request("/hibernate"));
        assert!(looks_like_hibernatemode_request("/mac-hibernatemode"));
        assert!(looks_like_hibernatemode_request("/mac-hibernate"));
        assert!(looks_like_hibernatemode_request("view hibernate mode"));
        assert!(looks_like_hibernatemode_request("open hibernatemode"));
        assert!(looks_like_hibernatemode_request("list hibernate"));
        assert!(looks_like_hibernatemode_request("hibernate mode status"));
        assert!(looks_like_hibernatemode_request("is hibernate mode on"));
        assert!(looks_like_hibernatemode_request("is hibernatemode enabled"));
        assert!(looks_like_hibernatemode_request("what's my hibernate mode"));
        assert!(looks_like_hibernatemode_request("view hibernate"));
        assert!(!looks_like_hibernatemode_request("hibernatemode"));
        assert!(!looks_like_hibernatemode_request("hibernate"));
        assert!(!looks_like_hibernatemode_request("hibernate mode"));
        assert!(!looks_like_hibernatemode_request("/systemsleep"));
        assert!(!looks_like_hibernatemode_request("view system sleep"));
        assert!(!looks_like_hibernatemode_request("/networkoversleep"));
        assert!(!looks_like_hibernatemode_request("/tcpkeepalive"));
        assert!(!looks_like_hibernatemode_request("/womp"));
        assert!(!looks_like_hibernatemode_request("/sleep"));
        assert!(!looks_like_hibernatemode_request("/powernap"));
        assert!(!looks_like_hibernatemode_request("/power"));
        assert!(!looks_like_hibernatemode_request("/tasks"));
        assert!(!looks_like_hibernatemode_request("enable hibernate mode"));
        assert!(!looks_like_hibernatemode_request("/hibernatefile"));
        assert!(!looks_like_hibernatemode_request("view hibernatefile"));
        assert!(!looks_like_hibernatemode_request("/sleepimage"));
        assert!(!looks_like_hibernatemode_request("view sleep image"));
        assert!(!looks_like_networkoversleep_request("/hibernatemode"));
        assert!(!looks_like_networkoversleep_request("view hibernate mode"));
        assert!(!looks_like_systemsleep_request("/hibernatemode"));
        assert!(!looks_like_systemsleep_request("view hibernate mode"));
        let s = try_operator_instant_reply("/hibernatemode").unwrap();
        assert!(
            s.contains("**Hibernate mode**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view hibernate mode").unwrap();
        assert!(
            viewed.contains("**Hibernate mode**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open hibernate").unwrap();
        assert!(
            open.contains("**Hibernate mode**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("hibernatemode").is_none());
        assert!(try_operator_instant_reply("hibernate").is_none());
        assert!(try_operator_instant_reply("hibernate mode").is_none());
        let sleep = try_operator_instant_reply("/systemsleep");
        if let Some(reply) = sleep {
            assert!(
                !reply.contains("**Hibernate mode**"),
                "/systemsleep must stay System Sleep, not Hibernate mode: {reply}"
            );
        }
        let nos = try_operator_instant_reply("/networkoversleep");
        if let Some(reply) = nos {
            assert!(
                !reply.contains("**Hibernate mode**"),
                "/networkoversleep must stay Network over sleep, not Hibernate mode: {reply}"
            );
        }
    }

    #[test]
    fn standby_slash_and_view_open_do_not_steal_hibernatemode_or_bare() {
        // Slash + view/open NL → Standby (`/hibernatemode` · `/powernap` parity).
        // Bare `standby` / `stand by` stay with the agent;
        // `/hibernatemode` stays Hibernate mode; `/systemsleep` stays System Sleep;
        // `/powernap` stays Power Nap; `/networkoversleep` stays Network over sleep.
        assert!(looks_like_standby_request("/standby"));
        assert!(looks_like_standby_request("/standby-status"));
        assert!(looks_like_standby_request("/stand-by"));
        assert!(looks_like_standby_request("/mac-standby"));
        assert!(looks_like_standby_request("/mac-stand-by"));
        assert!(looks_like_standby_request("view standby"));
        assert!(looks_like_standby_request("open standby"));
        assert!(looks_like_standby_request("list stand-by"));
        assert!(looks_like_standby_request("standby status"));
        assert!(looks_like_standby_request("is standby on"));
        assert!(looks_like_standby_request("is stand by enabled"));
        assert!(looks_like_standby_request("what's my standby"));
        assert!(looks_like_standby_request("view stand by"));
        assert!(!looks_like_standby_request("standby"));
        assert!(!looks_like_standby_request("stand by"));
        assert!(!looks_like_standby_request("/hibernatemode"));
        assert!(!looks_like_standby_request("view hibernate mode"));
        assert!(!looks_like_standby_request("/systemsleep"));
        assert!(!looks_like_standby_request("/powernap"));
        assert!(!looks_like_standby_request("/networkoversleep"));
        assert!(!looks_like_standby_request("/womp"));
        assert!(!looks_like_standby_request("/power"));
        assert!(!looks_like_standby_request("/tasks"));
        assert!(!looks_like_standby_request("enable standby"));
        assert!(!looks_like_standby_request("standbydelay"));
        assert!(!looks_like_hibernatemode_request("/standby"));
        assert!(!looks_like_hibernatemode_request("view standby"));
        assert!(!looks_like_systemsleep_request("/standby"));
        let s = try_operator_instant_reply("/standby").unwrap();
        assert!(
            s.contains("**Standby**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view standby").unwrap();
        assert!(
            viewed.contains("**Standby**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open standby").unwrap();
        assert!(
            open.contains("**Standby**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("standby").is_none());
        assert!(try_operator_instant_reply("stand by").is_none());
        let hib = try_operator_instant_reply("/hibernatemode");
        if let Some(reply) = hib {
            assert!(
                !reply.contains("**Standby**"),
                "/hibernatemode must stay Hibernate mode, not Standby: {reply}"
            );
        }
        let nap = try_operator_instant_reply("/powernap");
        if let Some(reply) = nap {
            assert!(
                !reply.contains("**Standby**"),
                "/powernap must stay Power Nap, not Standby: {reply}"
            );
        }
    }

    #[test]
    fn sleep_on_power_button_slash_and_view_open_do_not_steal_sleep_or_bare() {
        // Slash + view/open NL → Sleep On Power Button.
        // Bare `sleep on power button` / `power button sleep` / `sopb` stay with the agent;
        // `/sleep` · `/systemsleep` stay System Sleep; `/standby` stays Standby;
        // `/hibernatemode` stays Hibernate mode; `/powernap` stays Power Nap.
        assert!(looks_like_sleep_on_power_button_request("/sleep-on-power-button"));
        assert!(looks_like_sleep_on_power_button_request("/sleep-on-power-button-status"));
        assert!(looks_like_sleep_on_power_button_request("/sleep-on-power"));
        assert!(looks_like_sleep_on_power_button_request("/power-button-sleep"));
        assert!(looks_like_sleep_on_power_button_request("/sopb"));
        assert!(looks_like_sleep_on_power_button_request("/mac-sopb"));
        assert!(looks_like_sleep_on_power_button_request("view sleep on power button"));
        assert!(looks_like_sleep_on_power_button_request("open sopb"));
        assert!(looks_like_sleep_on_power_button_request("list power-button-sleep"));
        assert!(looks_like_sleep_on_power_button_request("sleep on power button status"));
        assert!(looks_like_sleep_on_power_button_request("is sleep on power button on"));
        assert!(looks_like_sleep_on_power_button_request("is power button sleep enabled"));
        assert!(looks_like_sleep_on_power_button_request("what's my sopb"));
        assert!(looks_like_sleep_on_power_button_request("view power button sleep"));
        assert!(!looks_like_sleep_on_power_button_request("sleep on power button"));
        assert!(!looks_like_sleep_on_power_button_request("power button sleep"));
        assert!(!looks_like_sleep_on_power_button_request("sopb"));
        assert!(!looks_like_sleep_on_power_button_request("/sleep"));
        assert!(!looks_like_sleep_on_power_button_request("/systemsleep"));
        assert!(!looks_like_sleep_on_power_button_request("view system sleep"));
        assert!(!looks_like_sleep_on_power_button_request("/standby"));
        assert!(!looks_like_sleep_on_power_button_request("/hibernatemode"));
        assert!(!looks_like_sleep_on_power_button_request("/powernap"));
        assert!(!looks_like_sleep_on_power_button_request("/power"));
        assert!(!looks_like_sleep_on_power_button_request("/tasks"));
        assert!(!looks_like_sleep_on_power_button_request("enable sleep on power button"));
        assert!(!looks_like_sleep_on_power_button_request("lessbright"));
        assert!(!looks_like_systemsleep_request("/sleep-on-power-button"));
        assert!(!looks_like_systemsleep_request("view sleep on power button"));
        assert!(!looks_like_standby_request("/sleep-on-power-button"));
        assert!(!looks_like_standby_request("view sleep on power button"));
        let s = try_operator_instant_reply("/sleep-on-power-button").unwrap();
        assert!(
            s.contains("**Sleep On Power Button**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view sleep on power button").unwrap();
        assert!(
            viewed.contains("**Sleep On Power Button**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open sopb").unwrap();
        assert!(
            open.contains("**Sleep On Power Button**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("sleep on power button").is_none());
        assert!(try_operator_instant_reply("sopb").is_none());
        let sleep = try_operator_instant_reply("/sleep");
        if let Some(reply) = sleep {
            assert!(
                !reply.contains("**Sleep On Power Button**"),
                "/sleep must stay System Sleep, not Sleep On Power Button: {reply}"
            );
        }
        let standby = try_operator_instant_reply("/standby");
        if let Some(reply) = standby {
            assert!(
                !reply.contains("**Sleep On Power Button**"),
                "/standby must stay Standby, not Sleep On Power Button: {reply}"
            );
        }
    }

    #[test]
    fn lessbright_slash_and_view_open_do_not_steal_displaysleep_or_bare() {
        // Slash + view/open NL → Less Bright (`/sopb` · `/displaysleep` parity).
        // Bare `lessbright` / `less bright` / `slightly dim` stay with the agent;
        // `/displaysleep` stays Display Sleep; `/display` stays resolution;
        // `/lpm` stays Low Power Mode; `/sopb` stays Sleep On Power Button.
        assert!(looks_like_lessbright_request("/lessbright"));
        assert!(looks_like_lessbright_request("/lessbright-status"));
        assert!(looks_like_lessbright_request("/less-bright"));
        assert!(looks_like_lessbright_request("/slightly-dim"));
        assert!(looks_like_lessbright_request("/dim-display"));
        assert!(looks_like_lessbright_request("/mac-lessbright"));
        assert!(looks_like_lessbright_request("/mac-less-bright"));
        assert!(looks_like_lessbright_request("view lessbright"));
        assert!(looks_like_lessbright_request("open slightly-dim"));
        assert!(looks_like_lessbright_request("list dim-display"));
        assert!(looks_like_lessbright_request("lessbright status"));
        assert!(looks_like_lessbright_request("is less bright on"));
        assert!(looks_like_lessbright_request("is slightly dim enabled"));
        assert!(looks_like_lessbright_request("what's my lessbright"));
        assert!(looks_like_lessbright_request("view less bright"));
        assert!(!looks_like_lessbright_request("lessbright"));
        assert!(!looks_like_lessbright_request("less bright"));
        assert!(!looks_like_lessbright_request("slightly dim"));
        assert!(!looks_like_lessbright_request("dim display"));
        assert!(!looks_like_lessbright_request("/displaysleep"));
        assert!(!looks_like_lessbright_request("view display sleep"));
        assert!(!looks_like_lessbright_request("/display"));
        assert!(!looks_like_lessbright_request("/screen"));
        assert!(!looks_like_lessbright_request("/lpm"));
        assert!(!looks_like_lessbright_request("/sopb"));
        assert!(!looks_like_lessbright_request("/sleep-on-power-button"));
        assert!(!looks_like_lessbright_request("/power"));
        assert!(!looks_like_lessbright_request("/tasks"));
        assert!(!looks_like_lessbright_request("enable lessbright"));
        assert!(!looks_like_displaysleep_request("/lessbright"));
        assert!(!looks_like_displaysleep_request("view lessbright"));
        assert!(!looks_like_sleep_on_power_button_request("/lessbright"));
        assert!(!looks_like_sleep_on_power_button_request("view lessbright"));
        let s = try_operator_instant_reply("/lessbright").unwrap();
        assert!(
            s.contains("**Less Bright**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view lessbright").unwrap();
        assert!(
            viewed.contains("**Less Bright**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open slightly-dim").unwrap();
        assert!(
            open.contains("**Less Bright**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("lessbright").is_none());
        assert!(try_operator_instant_reply("less bright").is_none());
        assert!(try_operator_instant_reply("slightly dim").is_none());
        let disp = try_operator_instant_reply("/displaysleep");
        if let Some(reply) = disp {
            assert!(
                !reply.contains("**Less Bright**"),
                "/displaysleep must stay Display Sleep, not Less Bright: {reply}"
            );
        }
        let sopb = try_operator_instant_reply("/sopb");
        if let Some(reply) = sopb {
            assert!(
                !reply.contains("**Less Bright**"),
                "/sopb must stay Sleep On Power Button, not Less Bright: {reply}"
            );
        }
    }

    #[test]
    fn powermode_slash_and_view_open_do_not_steal_lpm_or_bare() {
        // Slash + view/open NL → Power Mode (`/lessbright` · `/hibernatemode` parity).
        // Bare `powermode` / `power mode` stay with the agent;
        // `/lpm` stays Low Power Mode; `/powernap` stays Power Nap; `/power` stays strip;
        // `/lessbright` stays Less Bright; `/halfdim` stays Half Dim.
        assert!(looks_like_powermode_request("/powermode"));
        assert!(looks_like_powermode_request("/powermode-status"));
        assert!(looks_like_powermode_request("/power-mode"));
        assert!(looks_like_powermode_request("/mac-powermode"));
        assert!(looks_like_powermode_request("/mac-power-mode"));
        assert!(looks_like_powermode_request("view powermode"));
        assert!(looks_like_powermode_request("open power-mode"));
        assert!(looks_like_powermode_request("list power mode"));
        assert!(looks_like_powermode_request("powermode status"));
        assert!(looks_like_powermode_request("is power mode on"));
        assert!(looks_like_powermode_request("is powermode enabled"));
        assert!(looks_like_powermode_request("what's my power mode"));
        assert!(looks_like_powermode_request("view power mode"));
        assert!(!looks_like_powermode_request("powermode"));
        assert!(!looks_like_powermode_request("power mode"));
        assert!(!looks_like_powermode_request("/lpm"));
        assert!(!looks_like_powermode_request("view low power mode"));
        assert!(!looks_like_powermode_request("low power mode"));
        assert!(!looks_like_powermode_request("/powernap"));
        assert!(!looks_like_powermode_request("/power"));
        assert!(!looks_like_powermode_request("/watts"));
        assert!(!looks_like_powermode_request("/lessbright"));
        assert!(!looks_like_powermode_request("/sopb"));
        assert!(!looks_like_powermode_request("/tasks"));
        assert!(!looks_like_powermode_request("enable powermode"));
        assert!(!looks_like_lessbright_request("/powermode"));
        assert!(!looks_like_lessbright_request("view power mode"));
        let s = try_operator_instant_reply("/powermode").unwrap();
        assert!(
            s.contains("**Power Mode**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view power mode").unwrap();
        assert!(
            viewed.contains("**Power Mode**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open powermode").unwrap();
        assert!(
            open.contains("**Power Mode**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("powermode").is_none());
        assert!(try_operator_instant_reply("power mode").is_none());
        let lpm = try_operator_instant_reply("/lpm");
        if let Some(reply) = lpm {
            assert!(
                !reply.contains("**Power Mode**"),
                "/lpm must stay Low Power Mode, not Power Mode: {reply}"
            );
        }
        let less = try_operator_instant_reply("/lessbright");
        if let Some(reply) = less {
            assert!(
                !reply.contains("**Power Mode**"),
                "/lessbright must stay Less Bright, not Power Mode: {reply}"
            );
        }
    }

    #[test]
    fn halfdim_slash_and_view_open_do_not_steal_lessbright_or_bare() {
        // Slash + view/open NL → Half Dim (`/lessbright` · `/powermode` parity).
        // Bare `halfdim` / `half dim` / `half brightness` stay with the agent;
        // `/lessbright` stays Less Bright; `/displaysleep` stays Display Sleep;
        // `/powermode` stays Power Mode; `/dim-display` stays Less Bright.
        assert!(looks_like_halfdim_request("/halfdim"));
        assert!(looks_like_halfdim_request("/halfdim-status"));
        assert!(looks_like_halfdim_request("/half-dim"));
        assert!(looks_like_halfdim_request("/half-brightness"));
        assert!(looks_like_halfdim_request("/mac-halfdim"));
        assert!(looks_like_halfdim_request("/mac-half-dim"));
        assert!(looks_like_halfdim_request("view halfdim"));
        assert!(looks_like_halfdim_request("open half-dim"));
        assert!(looks_like_halfdim_request("list half brightness"));
        assert!(looks_like_halfdim_request("halfdim status"));
        assert!(looks_like_halfdim_request("is half dim on"));
        assert!(looks_like_halfdim_request("is half brightness enabled"));
        assert!(looks_like_halfdim_request("what's my halfdim"));
        assert!(looks_like_halfdim_request("view half dim"));
        assert!(!looks_like_halfdim_request("halfdim"));
        assert!(!looks_like_halfdim_request("half dim"));
        assert!(!looks_like_halfdim_request("half brightness"));
        assert!(!looks_like_halfdim_request("/lessbright"));
        assert!(!looks_like_halfdim_request("view lessbright"));
        assert!(!looks_like_halfdim_request("/dim-display"));
        assert!(!looks_like_halfdim_request("/displaysleep"));
        assert!(!looks_like_halfdim_request("/powermode"));
        assert!(!looks_like_halfdim_request("/power"));
        assert!(!looks_like_halfdim_request("/tasks"));
        assert!(!looks_like_halfdim_request("enable halfdim"));
        assert!(!looks_like_lessbright_request("/halfdim"));
        assert!(!looks_like_lessbright_request("view halfdim"));
        assert!(!looks_like_powermode_request("/halfdim"));
        assert!(!looks_like_powermode_request("view half dim"));
        let s = try_operator_instant_reply("/halfdim").unwrap();
        assert!(
            s.contains("**Half Dim**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view halfdim").unwrap();
        assert!(
            viewed.contains("**Half Dim**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open half-dim").unwrap();
        assert!(
            open.contains("**Half Dim**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("halfdim").is_none());
        assert!(try_operator_instant_reply("half dim").is_none());
        let less = try_operator_instant_reply("/lessbright");
        if let Some(reply) = less {
            assert!(
                !reply.contains("**Half Dim**"),
                "/lessbright must stay Less Bright, not Half Dim: {reply}"
            );
        }
        let mode = try_operator_instant_reply("/powermode");
        if let Some(reply) = mode {
            assert!(
                !reply.contains("**Half Dim**"),
                "/powermode must stay Power Mode, not Half Dim: {reply}"
            );
        }
    }

    #[test]
    fn hibernatefile_slash_and_view_open_do_not_steal_hibernatemode_or_bare() {
        // Slash + view/open NL → Hibernate file (`/hibernatemode` · `/halfdim` parity).
        // Bare `hibernatefile` / `hibernate file` / `sleepimage` / `sleep image` stay with the agent;
        // `/hibernatemode` · `/hibernate` stay Hibernate mode; `/systemsleep` stays System Sleep;
        // `/standby` stays Standby; `/halfdim` stays Half Dim.
        assert!(looks_like_hibernatefile_request("/hibernatefile"));
        assert!(looks_like_hibernatefile_request("/hibernatefile-status"));
        assert!(looks_like_hibernatefile_request("/hibernate-file"));
        assert!(looks_like_hibernatefile_request("/sleepimage"));
        assert!(looks_like_hibernatefile_request("/sleep-image"));
        assert!(looks_like_hibernatefile_request("/mac-hibernatefile"));
        assert!(looks_like_hibernatefile_request("/mac-sleepimage"));
        assert!(looks_like_hibernatefile_request("view hibernatefile"));
        assert!(looks_like_hibernatefile_request("open hibernate file"));
        assert!(looks_like_hibernatefile_request("list sleepimage"));
        assert!(looks_like_hibernatefile_request("hibernatefile status"));
        assert!(looks_like_hibernatefile_request("what's my hibernatefile"));
        assert!(looks_like_hibernatefile_request("view sleep image"));
        assert!(!looks_like_hibernatefile_request("hibernatefile"));
        assert!(!looks_like_hibernatefile_request("hibernate file"));
        assert!(!looks_like_hibernatefile_request("sleepimage"));
        assert!(!looks_like_hibernatefile_request("sleep image"));
        assert!(!looks_like_hibernatefile_request("/hibernatemode"));
        assert!(!looks_like_hibernatefile_request("view hibernate mode"));
        assert!(!looks_like_hibernatefile_request("/hibernate"));
        assert!(!looks_like_hibernatefile_request("view hibernate"));
        assert!(!looks_like_hibernatefile_request("/systemsleep"));
        assert!(!looks_like_hibernatefile_request("/standby"));
        assert!(!looks_like_hibernatefile_request("/halfdim"));
        assert!(!looks_like_hibernatefile_request("/power"));
        assert!(!looks_like_hibernatefile_request("/tasks"));
        assert!(!looks_like_hibernatefile_request("enable hibernatefile"));
        assert!(!looks_like_hibernatefile_request("standbydelay"));
        assert!(!looks_like_hibernatemode_request("/hibernatefile"));
        assert!(!looks_like_hibernatemode_request("view hibernatefile"));
        assert!(!looks_like_hibernatemode_request("view sleepimage"));
        assert!(!looks_like_hibernatemode_request("/sleepimage"));
        assert!(!looks_like_halfdim_request("/hibernatefile"));
        assert!(!looks_like_halfdim_request("view hibernate file"));
        let s = try_operator_instant_reply("/hibernatefile").unwrap();
        assert!(
            s.contains("**Hibernate file**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view hibernatefile").unwrap();
        assert!(
            viewed.contains("**Hibernate file**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open sleepimage").unwrap();
        assert!(
            open.contains("**Hibernate file**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("hibernatefile").is_none());
        assert!(try_operator_instant_reply("hibernate file").is_none());
        assert!(try_operator_instant_reply("sleepimage").is_none());
        let mode = try_operator_instant_reply("/hibernatemode");
        if let Some(reply) = mode {
            assert!(
                !reply.contains("**Hibernate file**"),
                "/hibernatemode must stay Hibernate mode, not Hibernate file: {reply}"
            );
        }
        let hib = try_operator_instant_reply("/hibernate");
        if let Some(reply) = hib {
            assert!(
                !reply.contains("**Hibernate file**"),
                "/hibernate must stay Hibernate mode, not Hibernate file: {reply}"
            );
        }
    }

    #[test]
    fn standbydelay_slash_and_view_open_do_not_steal_standby_or_bare() {
        // Slash + view/open NL → Standby delay (`/standby` · `/hibernatefile` parity).
        // Bare `standbydelay` / `standby delay` stay with the agent;
        // `/standby` stays Standby; `/hibernatefile` stays Hibernate file;
        // `/standbydelayhigh` stays Standby delay high; `/autopoweroff` stays Auto Power Off.
        assert!(looks_like_standbydelay_request("/standbydelay"));
        assert!(looks_like_standbydelay_request("/standbydelay-status"));
        assert!(looks_like_standbydelay_request("/standby-delay"));
        assert!(looks_like_standbydelay_request("/mac-standbydelay"));
        assert!(looks_like_standbydelay_request("/mac-standby-delay"));
        assert!(looks_like_standbydelay_request("view standbydelay"));
        assert!(looks_like_standbydelay_request("open standby-delay"));
        assert!(looks_like_standbydelay_request("list standby delay"));
        assert!(looks_like_standbydelay_request("standbydelay status"));
        assert!(looks_like_standbydelay_request("what's my standby delay"));
        assert!(looks_like_standbydelay_request("view standby delay"));
        assert!(!looks_like_standbydelay_request("standbydelay"));
        assert!(!looks_like_standbydelay_request("standby delay"));
        assert!(!looks_like_standbydelay_request("/standby"));
        assert!(!looks_like_standbydelay_request("view standby"));
        assert!(!looks_like_standbydelay_request("/hibernatefile"));
        assert!(!looks_like_standbydelay_request("/hibernatemode"));
        assert!(!looks_like_standbydelay_request("/tasks"));
        assert!(!looks_like_standbydelay_request("enable standbydelay"));
        assert!(!looks_like_standbydelay_request("standbydelayhigh"));
        assert!(!looks_like_standby_request("/standbydelay"));
        assert!(!looks_like_standby_request("view standbydelay"));
        assert!(!looks_like_standby_request("view standby delay"));
        assert!(!looks_like_hibernatefile_request("/standbydelay"));
        let s = try_operator_instant_reply("/standbydelay").unwrap();
        assert!(
            s.contains("**Standby delay**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view standbydelay").unwrap();
        assert!(
            viewed.contains("**Standby delay**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open standby-delay").unwrap();
        assert!(
            open.contains("**Standby delay**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("standbydelay").is_none());
        assert!(try_operator_instant_reply("standby delay").is_none());
        let st = try_operator_instant_reply("/standby");
        if let Some(reply) = st {
            assert!(
                !reply.contains("**Standby delay**"),
                "/standby must stay Standby, not Standby delay: {reply}"
            );
        }
        assert!(!looks_like_standbydelay_request("/autopoweroff"));
        assert!(!looks_like_standbydelay_request("view autopoweroff"));
        assert!(!looks_like_autopoweroff_request("/standbydelay"));
    }

    #[test]
    fn autopoweroff_slash_and_view_open_do_not_steal_standby_or_bare() {
        // Slash + view/open NL → Auto Power Off (`/standby` · `/standbydelay` parity).
        // Bare `autopoweroff` / `auto power off` stay with the agent;
        // `/standby` stays Standby; `/standbydelay` stays Standby delay;
        // autopoweroffdelay stays out.
        assert!(looks_like_autopoweroff_request("/autopoweroff"));
        assert!(looks_like_autopoweroff_request("/autopoweroff-status"));
        assert!(looks_like_autopoweroff_request("/auto-power-off"));
        assert!(looks_like_autopoweroff_request("/auto-poweroff"));
        assert!(looks_like_autopoweroff_request("/mac-autopoweroff"));
        assert!(looks_like_autopoweroff_request("/mac-auto-power-off"));
        assert!(looks_like_autopoweroff_request("view autopoweroff"));
        assert!(looks_like_autopoweroff_request("open auto-power-off"));
        assert!(looks_like_autopoweroff_request("list auto power off"));
        assert!(looks_like_autopoweroff_request("autopoweroff status"));
        assert!(looks_like_autopoweroff_request("is auto power off on"));
        assert!(looks_like_autopoweroff_request("is autopoweroff enabled"));
        assert!(looks_like_autopoweroff_request("what's my autopoweroff"));
        assert!(looks_like_autopoweroff_request("view auto power off"));
        assert!(!looks_like_autopoweroff_request("autopoweroff"));
        assert!(!looks_like_autopoweroff_request("auto power off"));
        assert!(!looks_like_autopoweroff_request("/standby"));
        assert!(!looks_like_autopoweroff_request("view standby"));
        assert!(!looks_like_autopoweroff_request("/standbydelay"));
        assert!(!looks_like_autopoweroff_request("view standbydelay"));
        assert!(!looks_like_autopoweroff_request("/hibernatefile"));
        assert!(!looks_like_autopoweroff_request("/tasks"));
        assert!(!looks_like_autopoweroff_request("enable autopoweroff"));
        assert!(!looks_like_autopoweroff_request("autopoweroffdelay"));
        assert!(!looks_like_autopoweroff_request("view autopoweroffdelay"));
        assert!(!looks_like_standby_request("/autopoweroff"));
        assert!(!looks_like_standby_request("view autopoweroff"));
        assert!(!looks_like_standbydelay_request("/autopoweroff"));
        let s = try_operator_instant_reply("/autopoweroff").unwrap();
        assert!(
            s.contains("**Auto Power Off**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view autopoweroff").unwrap();
        assert!(
            viewed.contains("**Auto Power Off**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open auto-power-off").unwrap();
        assert!(
            open.contains("**Auto Power Off**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("autopoweroff").is_none());
        assert!(try_operator_instant_reply("auto power off").is_none());
        let st = try_operator_instant_reply("/standby");
        if let Some(reply) = st {
            assert!(
                !reply.contains("**Auto Power Off**"),
                "/standby must stay Standby, not Auto Power Off: {reply}"
            );
        }
        let sd = try_operator_instant_reply("/standbydelay");
        if let Some(reply) = sd {
            assert!(
                !reply.contains("**Auto Power Off**"),
                "/standbydelay must stay Standby delay, not Auto Power Off: {reply}"
            );
        }
    }

    #[test]
    fn autopoweroffdelay_slash_and_view_open_do_not_steal_apo_or_bare() {
        // Slash + view/open NL → Autopower Off delay (`/autopoweroff` · `/standbydelay` parity).
        // Bare `autopoweroffdelay` / `auto power off delay` stay with the agent;
        // `/autopoweroff` stays Auto Power Off; `/standbydelay` stays Standby delay;
        // `/standbydelayhigh` stays Standby delay high.
        assert!(looks_like_autopoweroffdelay_request("/autopoweroffdelay"));
        assert!(looks_like_autopoweroffdelay_request("/autopoweroffdelay-status"));
        assert!(looks_like_autopoweroffdelay_request("/auto-power-off-delay"));
        assert!(looks_like_autopoweroffdelay_request("/mac-autopoweroffdelay"));
        assert!(looks_like_autopoweroffdelay_request("/mac-auto-power-off-delay"));
        assert!(looks_like_autopoweroffdelay_request("view autopoweroffdelay"));
        assert!(looks_like_autopoweroffdelay_request("open auto-power-off-delay"));
        assert!(looks_like_autopoweroffdelay_request("list auto power off delay"));
        assert!(looks_like_autopoweroffdelay_request("autopoweroffdelay status"));
        assert!(looks_like_autopoweroffdelay_request("what's my autopoweroffdelay"));
        assert!(looks_like_autopoweroffdelay_request("view auto power off delay"));
        assert!(!looks_like_autopoweroffdelay_request("autopoweroffdelay"));
        assert!(!looks_like_autopoweroffdelay_request("auto power off delay"));
        assert!(!looks_like_autopoweroffdelay_request("/autopoweroff"));
        assert!(!looks_like_autopoweroffdelay_request("view autopoweroff"));
        assert!(!looks_like_autopoweroffdelay_request("/standbydelay"));
        assert!(!looks_like_autopoweroffdelay_request("/standby"));
        assert!(!looks_like_autopoweroffdelay_request("/tasks"));
        assert!(!looks_like_autopoweroffdelay_request("enable autopoweroffdelay"));
        assert!(!looks_like_autopoweroffdelay_request("standbydelayhigh"));
        assert!(!looks_like_autopoweroff_request("/autopoweroffdelay"));
        assert!(!looks_like_autopoweroff_request("view autopoweroffdelay"));
        assert!(!looks_like_standbydelay_request("/autopoweroffdelay"));
        let s = try_operator_instant_reply("/autopoweroffdelay").unwrap();
        assert!(
            s.contains("**Autopower Off delay**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view autopoweroffdelay").unwrap();
        assert!(
            viewed.contains("**Autopower Off delay**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open auto-power-off-delay").unwrap();
        assert!(
            open.contains("**Autopower Off delay**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("autopoweroffdelay").is_none());
        assert!(try_operator_instant_reply("auto power off delay").is_none());
        let apo = try_operator_instant_reply("/autopoweroff");
        if let Some(reply) = apo {
            assert!(
                !reply.contains("**Autopower Off delay**"),
                "/autopoweroff must stay Auto Power Off, not delay: {reply}"
            );
        }
    }

    #[test]
    fn standbydelayhigh_slash_and_view_open_do_not_steal_standbydelay_or_bare() {
        // Slash + view/open NL → Standby delay high (`/standbydelay` · `/autopoweroffdelay` parity).
        // Bare `standbydelayhigh` / `standby delay high` stay with the agent;
        // `/standbydelay` stays Standby delay; `/standby` stays Standby;
        // `/standbydelaylow` stays Standby delay low.
        assert!(looks_like_standbydelayhigh_request("/standbydelayhigh"));
        assert!(looks_like_standbydelayhigh_request("/standbydelayhigh-status"));
        assert!(looks_like_standbydelayhigh_request("/standby-delay-high"));
        assert!(looks_like_standbydelayhigh_request("/mac-standbydelayhigh"));
        assert!(looks_like_standbydelayhigh_request("/mac-standby-delay-high"));
        assert!(looks_like_standbydelayhigh_request("view standbydelayhigh"));
        assert!(looks_like_standbydelayhigh_request("open standby-delay-high"));
        assert!(looks_like_standbydelayhigh_request("list standby delay high"));
        assert!(looks_like_standbydelayhigh_request("standbydelayhigh status"));
        assert!(looks_like_standbydelayhigh_request("what's my standbydelayhigh"));
        assert!(looks_like_standbydelayhigh_request("view standby delay high"));
        assert!(!looks_like_standbydelayhigh_request("standbydelayhigh"));
        assert!(!looks_like_standbydelayhigh_request("standby delay high"));
        assert!(!looks_like_standbydelayhigh_request("/standbydelay"));
        assert!(!looks_like_standbydelayhigh_request("view standbydelay"));
        assert!(!looks_like_standbydelayhigh_request("/standby"));
        assert!(!looks_like_standbydelayhigh_request("/autopoweroffdelay"));
        assert!(!looks_like_standbydelayhigh_request("/tasks"));
        assert!(!looks_like_standbydelayhigh_request("enable standbydelayhigh"));
        assert!(!looks_like_standbydelayhigh_request("standbydelaylow"));
        assert!(!looks_like_standbydelayhigh_request("/standbydelaylow"));
        assert!(!looks_like_standbydelay_request("/standbydelayhigh"));
        assert!(!looks_like_standbydelay_request("view standbydelayhigh"));
        assert!(!looks_like_autopoweroffdelay_request("/standbydelayhigh"));
        let s = try_operator_instant_reply("/standbydelayhigh").unwrap();
        assert!(
            s.contains("**Standby delay high**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view standbydelayhigh").unwrap();
        assert!(
            viewed.contains("**Standby delay high**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open standby-delay-high").unwrap();
        assert!(
            open.contains("**Standby delay high**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("standbydelayhigh").is_none());
        assert!(try_operator_instant_reply("standby delay high").is_none());
        let sd = try_operator_instant_reply("/standbydelay");
        if let Some(reply) = sd {
            assert!(
                !reply.contains("**Standby delay high**"),
                "/standbydelay must stay Standby delay, not high: {reply}"
            );
        }
    }

    #[test]
    fn standbydelaylow_slash_and_view_open_do_not_steal_delay_or_bare() {
        // Slash + view/open NL → Standby delay low (`/standbydelay` · `/standbydelayhigh` parity).
        // Bare `standbydelaylow` / `standby delay low` stay with the agent;
        // `/standbydelay` stays Standby delay; `/standbydelayhigh` stays Standby delay high;
        // `/standby` stays Standby; `/highstandbythreshold` stays High Standby Threshold;
        // `/autopoweroffdelay` stays Autopower Off delay.
        assert!(looks_like_standbydelaylow_request("/standbydelaylow"));
        assert!(looks_like_standbydelaylow_request("/standbydelaylow-status"));
        assert!(looks_like_standbydelaylow_request("/standby-delay-low"));
        assert!(looks_like_standbydelaylow_request("/mac-standbydelaylow"));
        assert!(looks_like_standbydelaylow_request("/mac-standby-delay-low"));
        assert!(looks_like_standbydelaylow_request("view standbydelaylow"));
        assert!(looks_like_standbydelaylow_request("open standby-delay-low"));
        assert!(looks_like_standbydelaylow_request("list standby delay low"));
        assert!(looks_like_standbydelaylow_request("standbydelaylow status"));
        assert!(looks_like_standbydelaylow_request("what's my standby delay low"));
        assert!(looks_like_standbydelaylow_request("view standby delay low"));
        assert!(!looks_like_standbydelaylow_request("standbydelaylow"));
        assert!(!looks_like_standbydelaylow_request("standby delay low"));
        assert!(!looks_like_standbydelaylow_request("/standbydelay"));
        assert!(!looks_like_standbydelaylow_request("view standbydelay"));
        assert!(!looks_like_standbydelaylow_request("/standbydelayhigh"));
        assert!(!looks_like_standbydelaylow_request("view standbydelayhigh"));
        assert!(!looks_like_standbydelaylow_request("/standby"));
        assert!(!looks_like_standbydelaylow_request("/autopoweroffdelay"));
        assert!(!looks_like_standbydelaylow_request("/highstandbythreshold"));
        assert!(!looks_like_standbydelaylow_request("view highstandbythreshold"));
        assert!(!looks_like_standbydelaylow_request("/tasks"));
        assert!(!looks_like_standbydelaylow_request("enable standbydelaylow"));
        assert!(!looks_like_standbydelaylow_request("standbydelayhigh"));
        assert!(!looks_like_standbydelay_request("/standbydelaylow"));
        assert!(!looks_like_standbydelayhigh_request("/standbydelaylow"));
        assert!(!looks_like_autopoweroffdelay_request("/standbydelaylow"));
        let s = try_operator_instant_reply("/standbydelaylow").unwrap();
        assert!(
            s.contains("**Standby delay low**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view standbydelaylow").unwrap();
        assert!(
            viewed.contains("**Standby delay low**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open standby-delay-low").unwrap();
        assert!(
            open.contains("**Standby delay low**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("standbydelaylow").is_none());
        assert!(try_operator_instant_reply("standby delay low").is_none());
        let hi = try_operator_instant_reply("/standbydelayhigh");
        if let Some(reply) = hi {
            assert!(
                !reply.contains("**Standby delay low**"),
                "/standbydelayhigh must stay Standby delay high, not low: {reply}"
            );
        }
        let sd = try_operator_instant_reply("/standbydelay");
        if let Some(reply) = sd {
            assert!(
                !reply.contains("**Standby delay low**"),
                "/standbydelay must stay Standby delay, not low: {reply}"
            );
        }
    }

    #[test]
    fn highstandbythreshold_slash_and_view_open_do_not_steal_delay_or_bare() {
        // Slash + view/open NL → High Standby Threshold (`/standbydelay` · `/standbydelaylow` parity).
        // Bare `highstandbythreshold` / `high standby threshold` / `standby threshold` stay with the agent;
        // `/standbydelay` stays Standby delay; `/standbydelayhigh` stays Standby delay high;
        // `/standbydelaylow` stays Standby delay low; `/standby` stays Standby.
        assert!(looks_like_highstandbythreshold_request("/highstandbythreshold"));
        assert!(looks_like_highstandbythreshold_request("/highstandbythreshold-status"));
        assert!(looks_like_highstandbythreshold_request("/high-standby-threshold"));
        assert!(looks_like_highstandbythreshold_request("/standbythreshold"));
        assert!(looks_like_highstandbythreshold_request("/standby-threshold"));
        assert!(looks_like_highstandbythreshold_request("/mac-highstandbythreshold"));
        assert!(looks_like_highstandbythreshold_request("/mac-high-standby-threshold"));
        assert!(looks_like_highstandbythreshold_request("view highstandbythreshold"));
        assert!(looks_like_highstandbythreshold_request("open high-standby-threshold"));
        assert!(looks_like_highstandbythreshold_request("list standby threshold"));
        assert!(looks_like_highstandbythreshold_request("highstandbythreshold status"));
        assert!(looks_like_highstandbythreshold_request("what's my high standby threshold"));
        assert!(looks_like_highstandbythreshold_request("view standby threshold"));
        assert!(!looks_like_highstandbythreshold_request("highstandbythreshold"));
        assert!(!looks_like_highstandbythreshold_request("high standby threshold"));
        assert!(!looks_like_highstandbythreshold_request("standby threshold"));
        assert!(!looks_like_highstandbythreshold_request("/standbydelay"));
        assert!(!looks_like_highstandbythreshold_request("view standbydelay"));
        assert!(!looks_like_highstandbythreshold_request("/standbydelayhigh"));
        assert!(!looks_like_highstandbythreshold_request("/standbydelaylow"));
        assert!(!looks_like_highstandbythreshold_request("/standby"));
        assert!(!looks_like_highstandbythreshold_request("/tasks"));
        assert!(!looks_like_highstandbythreshold_request("enable highstandbythreshold"));
        assert!(!looks_like_standbydelay_request("/highstandbythreshold"));
        assert!(!looks_like_standbydelayhigh_request("/highstandbythreshold"));
        assert!(!looks_like_standbydelaylow_request("/highstandbythreshold"));
        let s = try_operator_instant_reply("/highstandbythreshold").unwrap();
        assert!(
            s.contains("**High Standby Threshold**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view highstandbythreshold").unwrap();
        assert!(
            viewed.contains("**High Standby Threshold**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open high-standby-threshold").unwrap();
        assert!(
            open.contains("**High Standby Threshold**")
                && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("highstandbythreshold").is_none());
        assert!(try_operator_instant_reply("high standby threshold").is_none());
        assert!(try_operator_instant_reply("standby threshold").is_none());
        let low = try_operator_instant_reply("/standbydelaylow");
        if let Some(reply) = low {
            assert!(
                !reply.contains("**High Standby Threshold**"),
                "/standbydelaylow must stay Standby delay low, not threshold: {reply}"
            );
        }
        let hi = try_operator_instant_reply("/standbydelayhigh");
        if let Some(reply) = hi {
            assert!(
                !reply.contains("**High Standby Threshold**"),
                "/standbydelayhigh must stay Standby delay high, not threshold: {reply}"
            );
        }
    }

    #[test]
    fn destroyfvkeyonstandby_slash_and_view_open_do_not_steal_filevault_or_bare() {
        // Slash + view/open NL → Destroy FV key on standby (`/filevault` · `/standby` parity).
        // Bare `destroyfvkeyonstandby` / `destroy fv key on standby` stay with the agent;
        // `/filevault` · `/fv` stay FileVault; `/standby` stays Standby;
        // `/highstandbythreshold` stays High Standby Threshold; `/hibernatemode` stays Hibernate mode.
        assert!(looks_like_destroyfvkeyonstandby_request("/destroyfvkeyonstandby"));
        assert!(looks_like_destroyfvkeyonstandby_request("/destroyfvkeyonstandby-status"));
        assert!(looks_like_destroyfvkeyonstandby_request("/destroy-fv-key-on-standby"));
        assert!(looks_like_destroyfvkeyonstandby_request("/destroyfvkey"));
        assert!(looks_like_destroyfvkeyonstandby_request("/destroy-fv-key"));
        assert!(looks_like_destroyfvkeyonstandby_request("/fvkeyonstandby"));
        assert!(looks_like_destroyfvkeyonstandby_request("/fv-key-on-standby"));
        assert!(looks_like_destroyfvkeyonstandby_request("/mac-destroyfvkeyonstandby"));
        assert!(looks_like_destroyfvkeyonstandby_request("/mac-destroy-fv-key-on-standby"));
        assert!(looks_like_destroyfvkeyonstandby_request("view destroyfvkeyonstandby"));
        assert!(looks_like_destroyfvkeyonstandby_request("open destroy-fv-key-on-standby"));
        assert!(looks_like_destroyfvkeyonstandby_request("list fv key on standby"));
        assert!(looks_like_destroyfvkeyonstandby_request("destroyfvkeyonstandby status"));
        assert!(looks_like_destroyfvkeyonstandby_request("is destroy fv key on standby on"));
        assert!(looks_like_destroyfvkeyonstandby_request("what's my destroyfvkeyonstandby"));
        assert!(looks_like_destroyfvkeyonstandby_request("view destroy fv key on standby"));
        assert!(!looks_like_destroyfvkeyonstandby_request("destroyfvkeyonstandby"));
        assert!(!looks_like_destroyfvkeyonstandby_request("destroy fv key on standby"));
        assert!(!looks_like_destroyfvkeyonstandby_request("destroyfvkey"));
        assert!(!looks_like_destroyfvkeyonstandby_request("fv key on standby"));
        assert!(!looks_like_destroyfvkeyonstandby_request("/filevault"));
        assert!(!looks_like_destroyfvkeyonstandby_request("view filevault"));
        assert!(!looks_like_destroyfvkeyonstandby_request("/fv"));
        assert!(!looks_like_destroyfvkeyonstandby_request("/standby"));
        assert!(!looks_like_destroyfvkeyonstandby_request("view standby"));
        assert!(!looks_like_destroyfvkeyonstandby_request("/highstandbythreshold"));
        assert!(!looks_like_destroyfvkeyonstandby_request("/hibernatemode"));
        assert!(!looks_like_destroyfvkeyonstandby_request("/tasks"));
        assert!(!looks_like_destroyfvkeyonstandby_request("enable destroyfvkeyonstandby"));
        assert!(!looks_like_filevault_request("/destroyfvkeyonstandby"));
        assert!(!looks_like_filevault_request("view destroyfvkeyonstandby"));
        assert!(!looks_like_standby_request("/destroyfvkeyonstandby"));
        assert!(!looks_like_standby_request("view destroy fv key on standby"));
        assert!(!looks_like_highstandbythreshold_request("/destroyfvkeyonstandby"));
        let s = try_operator_instant_reply("/destroyfvkeyonstandby").unwrap();
        assert!(
            s.contains("**Destroy FV key on standby**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view destroyfvkeyonstandby").unwrap();
        assert!(
            viewed.contains("**Destroy FV key on standby**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open destroy-fv-key-on-standby").unwrap();
        assert!(
            open.contains("**Destroy FV key on standby**")
                && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("destroyfvkeyonstandby").is_none());
        assert!(try_operator_instant_reply("destroy fv key on standby").is_none());
        let fv = try_operator_instant_reply("/filevault");
        if let Some(reply) = fv {
            assert!(
                !reply.contains("**Destroy FV key on standby**"),
                "/filevault must stay FileVault, not destroyfvkey: {reply}"
            );
        }
        let standby = try_operator_instant_reply("/standby");
        if let Some(reply) = standby {
            assert!(
                !reply.contains("**Destroy FV key on standby**"),
                "/standby must stay Standby, not destroyfvkey: {reply}"
            );
        }
    }

    #[test]
    fn acwake_slash_and_view_open_do_not_steal_womp_or_bare() {
        // Slash + view/open NL → AC Wake (`/womp` · `/proximitywake` parity).
        // Bare `acwake` / `ac wake` stay with the agent;
        // `/womp` stays Wake for network; `/proximitywake` stays Proximity Wake;
        // `/lidwake` stays Lid Wake; `/standby` stays Standby; `/destroyfvkeyonstandby` stays Destroy FV key;
        // `/charger` stays AC charger.
        assert!(looks_like_acwake_request("/acwake"));
        assert!(looks_like_acwake_request("/acwake-status"));
        assert!(looks_like_acwake_request("/ac-wake"));
        assert!(looks_like_acwake_request("/wake-on-ac"));
        assert!(looks_like_acwake_request("/mac-acwake"));
        assert!(looks_like_acwake_request("/mac-ac-wake"));
        assert!(looks_like_acwake_request("view acwake"));
        assert!(looks_like_acwake_request("open ac-wake"));
        assert!(looks_like_acwake_request("list wake on ac"));
        assert!(looks_like_acwake_request("acwake status"));
        assert!(looks_like_acwake_request("is ac wake on"));
        assert!(looks_like_acwake_request("what's my acwake"));
        assert!(looks_like_acwake_request("view ac wake"));
        assert!(!looks_like_acwake_request("acwake"));
        assert!(!looks_like_acwake_request("ac wake"));
        assert!(!looks_like_acwake_request("/womp"));
        assert!(!looks_like_acwake_request("view womp"));
        assert!(!looks_like_acwake_request("/proximitywake"));
        assert!(!looks_like_acwake_request("/standby"));
        assert!(!looks_like_acwake_request("/destroyfvkeyonstandby"));
        assert!(!looks_like_acwake_request("/lidwake"));
        assert!(!looks_like_acwake_request("/charger"));
        assert!(!looks_like_acwake_request("/tasks"));
        assert!(!looks_like_acwake_request("enable acwake"));
        assert!(!looks_like_wake_for_network_request("/acwake"));
        assert!(!looks_like_wake_for_network_request("view acwake"));
        assert!(!looks_like_destroyfvkeyonstandby_request("/acwake"));
        let s = try_operator_instant_reply("/acwake").unwrap();
        assert!(
            s.contains("**AC Wake**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view acwake").unwrap();
        assert!(
            viewed.contains("**AC Wake**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open ac-wake").unwrap();
        assert!(
            open.contains("**AC Wake**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("acwake").is_none());
        assert!(try_operator_instant_reply("ac wake").is_none());
        let womp = try_operator_instant_reply("/womp");
        if let Some(reply) = womp {
            assert!(
                !reply.contains("**AC Wake**"),
                "/womp must stay Wake for network, not AC Wake: {reply}"
            );
        }
        let charger = try_operator_instant_reply("/charger");
        if let Some(reply) = charger {
            assert!(
                !reply.contains("**AC Wake**"),
                "/charger must stay AC charger, not AC Wake: {reply}"
            );
        }
    }

    #[test]
    fn lidwake_slash_and_view_open_do_not_steal_womp_or_bare() {
        // Slash + view/open NL → Lid Wake (`/womp` · `/acwake` parity).
        // Bare `lidwake` / `lid wake` stay with the agent;
        // `/womp` stays Wake for network; `/proximitywake` stays Proximity Wake;
        // `/acwake` stays AC Wake; `/standby` stays Standby; `/destroyfvkeyonstandby` stays Destroy FV key.
        assert!(looks_like_lidwake_request("/lidwake"));
        assert!(looks_like_lidwake_request("/lidwake-status"));
        assert!(looks_like_lidwake_request("/lid-wake"));
        assert!(looks_like_lidwake_request("/wake-on-lid"));
        assert!(looks_like_lidwake_request("/mac-lidwake"));
        assert!(looks_like_lidwake_request("/mac-lid-wake"));
        assert!(looks_like_lidwake_request("view lidwake"));
        assert!(looks_like_lidwake_request("open lid-wake"));
        assert!(looks_like_lidwake_request("list wake on lid"));
        assert!(looks_like_lidwake_request("lidwake status"));
        assert!(looks_like_lidwake_request("is lid wake on"));
        assert!(looks_like_lidwake_request("what's my lidwake"));
        assert!(looks_like_lidwake_request("view lid wake"));
        assert!(!looks_like_lidwake_request("lidwake"));
        assert!(!looks_like_lidwake_request("lid wake"));
        assert!(!looks_like_lidwake_request("/womp"));
        assert!(!looks_like_lidwake_request("view womp"));
        assert!(!looks_like_lidwake_request("/proximitywake"));
        assert!(!looks_like_lidwake_request("/acwake"));
        assert!(!looks_like_lidwake_request("/standby"));
        assert!(!looks_like_lidwake_request("/destroyfvkeyonstandby"));
        assert!(!looks_like_lidwake_request("/tasks"));
        assert!(!looks_like_lidwake_request("enable lidwake"));
        assert!(!looks_like_acwake_request("/lidwake"));
        assert!(!looks_like_acwake_request("view lidwake"));
        assert!(!looks_like_wake_for_network_request("/lidwake"));
        assert!(!looks_like_destroyfvkeyonstandby_request("/lidwake"));
        let s = try_operator_instant_reply("/lidwake").unwrap();
        assert!(
            s.contains("**Lid Wake**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view lidwake").unwrap();
        assert!(
            viewed.contains("**Lid Wake**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open lid-wake").unwrap();
        assert!(
            open.contains("**Lid Wake**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("lidwake").is_none());
        assert!(try_operator_instant_reply("lid wake").is_none());
        let acwake = try_operator_instant_reply("/acwake");
        if let Some(reply) = acwake {
            assert!(
                !reply.contains("**Lid Wake**"),
                "/acwake must stay AC Wake, not Lid Wake: {reply}"
            );
        }
        let womp = try_operator_instant_reply("/womp");
        if let Some(reply) = womp {
            assert!(
                !reply.contains("**Lid Wake**"),
                "/womp must stay Wake for network, not Lid Wake: {reply}"
            );
        }
    }

    #[test]
    fn gpuswitch_slash_and_view_open_do_not_steal_gpu_or_bare() {
        // Slash + view/open NL → GPU Switch (`/autorestart` · `/powermode` parity).
        // Bare `gpuswitch` / `gpu switch` / `graphics switching` stay with the agent;
        // `/gpu` stays GPU ring; `/gpu-name` stays GPU identity; `/autorestart` stays Auto Restart.
        assert!(looks_like_gpuswitch_request("/gpuswitch"));
        assert!(looks_like_gpuswitch_request("/gpuswitch-status"));
        assert!(looks_like_gpuswitch_request("/gpu-switch"));
        assert!(looks_like_gpuswitch_request("/graphics-switching"));
        assert!(looks_like_gpuswitch_request("/mac-gpuswitch"));
        assert!(looks_like_gpuswitch_request("/mac-gpu-switch"));
        assert!(looks_like_gpuswitch_request("view gpuswitch"));
        assert!(looks_like_gpuswitch_request("open gpu-switch"));
        assert!(looks_like_gpuswitch_request("list graphics switching"));
        assert!(looks_like_gpuswitch_request("gpuswitch status"));
        assert!(looks_like_gpuswitch_request("is gpu switch on"));
        assert!(looks_like_gpuswitch_request("what's my gpuswitch"));
        assert!(looks_like_gpuswitch_request("view gpu switch"));
        assert!(!looks_like_gpuswitch_request("gpuswitch"));
        assert!(!looks_like_gpuswitch_request("gpu switch"));
        assert!(!looks_like_gpuswitch_request("graphics switching"));
        assert!(!looks_like_gpuswitch_request("/gpu"));
        assert!(!looks_like_gpuswitch_request("view gpu"));
        assert!(!looks_like_gpuswitch_request("/gpu-name"));
        assert!(!looks_like_gpuswitch_request("/autorestart"));
        assert!(!looks_like_gpuswitch_request("/powermode"));
        assert!(!looks_like_gpuswitch_request("/tasks"));
        assert!(!looks_like_gpuswitch_request("enable gpuswitch"));
        assert!(!looks_like_autorestart_request("/gpuswitch"));
        assert!(!looks_like_autorestart_request("view gpuswitch"));
        let s = try_operator_instant_reply("/gpuswitch").unwrap();
        assert!(
            s.contains("**GPU Switch**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view gpuswitch").unwrap();
        assert!(
            viewed.contains("**GPU Switch**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open gpu-switch").unwrap();
        assert!(
            open.contains("**GPU Switch**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("gpuswitch").is_none());
        assert!(try_operator_instant_reply("gpu switch").is_none());
        let gpu = try_operator_instant_reply("/gpu");
        if let Some(reply) = gpu {
            assert!(
                !reply.contains("**GPU Switch**"),
                "/gpu must stay GPU ring, not GPU Switch: {reply}"
            );
        }
        let ar = try_operator_instant_reply("/autorestart");
        if let Some(reply) = ar {
            assert!(
                !reply.contains("**GPU Switch**"),
                "/autorestart must stay Auto Restart, not GPU Switch: {reply}"
            );
        }
    }

    #[test]
    fn darkwakes_slash_and_view_open_do_not_steal_gpuswitch_or_bare() {
        // Slash + view/open NL → Dark Wakes (`/gpuswitch` · `/autorestart` parity).
        // Bare `darkwakes` / `dark wakes` stay with the agent;
        // `/gpuswitch` stays GPU Switch; `/autorestart` stays Auto Restart.
        assert!(looks_like_darkwakes_request("/darkwakes"));
        assert!(looks_like_darkwakes_request("/darkwakes-status"));
        assert!(looks_like_darkwakes_request("/dark-wakes"));
        assert!(looks_like_darkwakes_request("/darkwake"));
        assert!(looks_like_darkwakes_request("/mac-darkwakes"));
        assert!(looks_like_darkwakes_request("/mac-dark-wakes"));
        assert!(looks_like_darkwakes_request("view darkwakes"));
        assert!(looks_like_darkwakes_request("open dark-wakes"));
        assert!(looks_like_darkwakes_request("list dark wake"));
        assert!(looks_like_darkwakes_request("darkwakes status"));
        assert!(looks_like_darkwakes_request("is dark wakes on"));
        assert!(looks_like_darkwakes_request("what's my darkwakes"));
        assert!(looks_like_darkwakes_request("view dark wakes"));
        assert!(!looks_like_darkwakes_request("darkwakes"));
        assert!(!looks_like_darkwakes_request("dark wakes"));
        assert!(!looks_like_darkwakes_request("/gpuswitch"));
        assert!(!looks_like_darkwakes_request("view gpuswitch"));
        assert!(!looks_like_darkwakes_request("/autorestart"));
        assert!(!looks_like_darkwakes_request("/lidwake"));
        assert!(!looks_like_darkwakes_request("/acwake"));
        assert!(!looks_like_darkwakes_request("/standby"));
        assert!(!looks_like_darkwakes_request("/tasks"));
        assert!(!looks_like_darkwakes_request("enable darkwakes"));
        assert!(!looks_like_darkwakes_request("/gpu"));
        assert!(!looks_like_darkwakes_request("/ttysclobber"));
        assert!(!looks_like_gpuswitch_request("/darkwakes"));
        assert!(!looks_like_gpuswitch_request("view darkwakes"));
        assert!(!looks_like_autorestart_request("/darkwakes"));
        let s = try_operator_instant_reply("/darkwakes").unwrap();
        assert!(
            s.contains("**Dark Wakes**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view darkwakes").unwrap();
        assert!(
            viewed.contains("**Dark Wakes**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open dark-wakes").unwrap();
        assert!(
            open.contains("**Dark Wakes**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("darkwakes").is_none());
        assert!(try_operator_instant_reply("dark wakes").is_none());
        let gs = try_operator_instant_reply("/gpuswitch");
        if let Some(reply) = gs {
            assert!(
                !reply.contains("**Dark Wakes**"),
                "/gpuswitch must stay GPU Switch, not Dark Wakes: {reply}"
            );
        }
        let ar = try_operator_instant_reply("/autorestart");
        if let Some(reply) = ar {
            assert!(
                !reply.contains("**Dark Wakes**"),
                "/autorestart must stay Auto Restart, not Dark Wakes: {reply}"
            );
        }
    }

    #[test]
    fn ttysclobber_slash_and_view_open_do_not_steal_ttyskeepawake_or_bare() {
        // Slash + view/open NL → TTY Clobber (`/ttyskeepawake` · `/darkwakes` parity).
        // Bare `ttysclobber` / `ttys clobber` / `tty clobber` stay with the agent;
        // `/ttyskeepawake` stays TTY keep awake; `/darkwakes` stays Dark Wakes.
        assert!(looks_like_ttysclobber_request("/ttysclobber"));
        assert!(looks_like_ttysclobber_request("/ttysclobber-status"));
        assert!(looks_like_ttysclobber_request("/ttys-clobber"));
        assert!(looks_like_ttysclobber_request("/tty-clobber"));
        assert!(looks_like_ttysclobber_request("/mac-ttysclobber"));
        assert!(looks_like_ttysclobber_request("/mac-ttys-clobber"));
        assert!(looks_like_ttysclobber_request("view ttysclobber"));
        assert!(looks_like_ttysclobber_request("open ttys-clobber"));
        assert!(looks_like_ttysclobber_request("list tty clobber"));
        assert!(looks_like_ttysclobber_request("ttysclobber status"));
        assert!(looks_like_ttysclobber_request("is ttys clobber on"));
        assert!(looks_like_ttysclobber_request("what's my ttysclobber"));
        assert!(looks_like_ttysclobber_request("view ttys clobber"));
        assert!(!looks_like_ttysclobber_request("ttysclobber"));
        assert!(!looks_like_ttysclobber_request("ttys clobber"));
        assert!(!looks_like_ttysclobber_request("tty clobber"));
        assert!(!looks_like_ttysclobber_request("/ttyskeepawake"));
        assert!(!looks_like_ttysclobber_request("view tty keep awake"));
        assert!(!looks_like_ttysclobber_request("/darkwakes"));
        assert!(!looks_like_ttysclobber_request("/tcpkeepalive"));
        assert!(!looks_like_ttysclobber_request("/standby"));
        assert!(!looks_like_ttysclobber_request("/tasks"));
        assert!(!looks_like_ttysclobber_request("enable ttysclobber"));
        assert!(!looks_like_ttyskeepawake_request("/ttysclobber"));
        assert!(!looks_like_ttyskeepawake_request("view ttysclobber"));
        assert!(!looks_like_darkwakes_request("/ttysclobber"));
        let s = try_operator_instant_reply("/ttysclobber").unwrap();
        assert!(
            s.contains("**TTY Clobber**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view ttysclobber").unwrap();
        assert!(
            viewed.contains("**TTY Clobber**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open ttys-clobber").unwrap();
        assert!(
            open.contains("**TTY Clobber**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("ttysclobber").is_none());
        assert!(try_operator_instant_reply("ttys clobber").is_none());
        let tk = try_operator_instant_reply("/ttyskeepawake");
        if let Some(reply) = tk {
            assert!(
                !reply.contains("**TTY Clobber**"),
                "/ttyskeepawake must stay TTY keep awake, not TTY Clobber: {reply}"
            );
        }
        let dw = try_operator_instant_reply("/darkwakes");
        if let Some(reply) = dw {
            assert!(
                !reply.contains("**TTY Clobber**"),
                "/darkwakes must stay Dark Wakes, not TTY Clobber: {reply}"
            );
        }
    }

    #[test]
    fn reducebrightness_slash_and_view_open_do_not_steal_lessbright_or_bare() {
        // Slash + view/open NL → Reduce Brightness (`/lessbright` · `/halfdim` parity).
        // Bare `reducebrightness` / `reduce brightness` stay with the agent;
        // `/lessbright` stays Less Bright; `/halfdim` stays Half Dim;
        // `/dim-display` stays Less Bright; `/norestoresleep` stays No Restore Sleep.
        assert!(looks_like_reducebrightness_request("/reducebrightness"));
        assert!(looks_like_reducebrightness_request("/reducebrightness-status"));
        assert!(looks_like_reducebrightness_request("/reduce-brightness"));
        assert!(looks_like_reducebrightness_request("/reducebright"));
        assert!(looks_like_reducebrightness_request("/mac-reducebrightness"));
        assert!(looks_like_reducebrightness_request("/mac-reduce-brightness"));
        assert!(looks_like_reducebrightness_request("view reducebrightness"));
        assert!(looks_like_reducebrightness_request("open reduce-brightness"));
        assert!(looks_like_reducebrightness_request("list reduce brightness"));
        assert!(looks_like_reducebrightness_request("reducebrightness status"));
        assert!(looks_like_reducebrightness_request("is reduce brightness on"));
        assert!(looks_like_reducebrightness_request("what's my reducebrightness"));
        assert!(looks_like_reducebrightness_request("view reduce brightness"));
        assert!(!looks_like_reducebrightness_request("reducebrightness"));
        assert!(!looks_like_reducebrightness_request("reduce brightness"));
        assert!(!looks_like_reducebrightness_request("/lessbright"));
        assert!(!looks_like_reducebrightness_request("view less bright"));
        assert!(!looks_like_reducebrightness_request("/halfdim"));
        assert!(!looks_like_reducebrightness_request("/dim-display"));
        assert!(!looks_like_reducebrightness_request("/norestoresleep"));
        assert!(!looks_like_reducebrightness_request("/displaysleep"));
        assert!(!looks_like_reducebrightness_request("/tasks"));
        assert!(!looks_like_reducebrightness_request("enable reducebrightness"));
        assert!(!looks_like_lessbright_request("/reducebrightness"));
        assert!(!looks_like_lessbright_request("view reducebrightness"));
        assert!(!looks_like_halfdim_request("/reducebrightness"));
        assert!(!looks_like_norestoresleep_request("/reducebrightness"));
        let s = try_operator_instant_reply("/reducebrightness").unwrap();
        assert!(
            s.contains("**Reduce Brightness**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view reducebrightness").unwrap();
        assert!(
            viewed.contains("**Reduce Brightness**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open reduce-brightness").unwrap();
        assert!(
            open.contains("**Reduce Brightness**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("reducebrightness").is_none());
        assert!(try_operator_instant_reply("reduce brightness").is_none());
        let lb = try_operator_instant_reply("/lessbright");
        if let Some(reply) = lb {
            assert!(
                !reply.contains("**Reduce Brightness**"),
                "/lessbright must stay Less Bright, not Reduce Brightness: {reply}"
            );
        }
        let hd = try_operator_instant_reply("/halfdim");
        if let Some(reply) = hd {
            assert!(
                !reply.contains("**Reduce Brightness**"),
                "/halfdim must stay Half Dim, not Reduce Brightness: {reply}"
            );
        }
    }

    #[test]
    fn sms_slash_and_view_open_do_not_steal_systemsleep_or_bare() {
        // Slash + view/open NL → Sudden Motion Sensor (`/systemsleep` · `/reducebrightness` parity).
        // Bare `sms` / `sudden motion` stay with the agent;
        // `/systemsleep` stays System Sleep; `/reducebrightness` stays Reduce Brightness.
        assert!(looks_like_sms_request("/sms"));
        assert!(looks_like_sms_request("/sms-status"));
        assert!(looks_like_sms_request("/sudden-motion"));
        assert!(looks_like_sms_request("/sudden-motion-sensor"));
        assert!(looks_like_sms_request("/mac-sms"));
        assert!(looks_like_sms_request("/mac-sudden-motion"));
        assert!(looks_like_sms_request("view sms"));
        assert!(looks_like_sms_request("open sudden-motion"));
        assert!(looks_like_sms_request("list sudden motion sensor"));
        assert!(looks_like_sms_request("sms status"));
        assert!(looks_like_sms_request("is sudden motion on"));
        assert!(looks_like_sms_request("what's my sms"));
        assert!(looks_like_sms_request("view sudden motion"));
        assert!(!looks_like_sms_request("sms"));
        assert!(!looks_like_sms_request("sudden motion"));
        assert!(!looks_like_sms_request("sudden motion sensor"));
        assert!(!looks_like_sms_request("/systemsleep"));
        assert!(!looks_like_sms_request("view system sleep"));
        assert!(!looks_like_sms_request("/reducebrightness"));
        assert!(!looks_like_sms_request("/halfdim"));
        assert!(!looks_like_sms_request("/lessbright"));
        assert!(!looks_like_sms_request("/ring"));
        assert!(!looks_like_sms_request("/tasks"));
        assert!(!looks_like_sms_request("enable sms"));
        assert!(!looks_like_systemsleep_request("/sms"));
        assert!(!looks_like_systemsleep_request("view sms"));
        assert!(!looks_like_reducebrightness_request("/sms"));
        let s = try_operator_instant_reply("/sms").unwrap();
        assert!(
            s.contains("**Sudden Motion Sensor**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view sms").unwrap();
        assert!(
            viewed.contains("**Sudden Motion Sensor**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open sudden-motion").unwrap();
        assert!(
            open.contains("**Sudden Motion Sensor**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("sms").is_none());
        assert!(try_operator_instant_reply("sudden motion").is_none());
        let ss = try_operator_instant_reply("/systemsleep");
        if let Some(reply) = ss {
            assert!(
                !reply.contains("**Sudden Motion Sensor**"),
                "/systemsleep must stay System Sleep, not SMS: {reply}"
            );
        }
        let rb = try_operator_instant_reply("/reducebrightness");
        if let Some(reply) = rb {
            assert!(
                !reply.contains("**Sudden Motion Sensor**"),
                "/reducebrightness must stay Reduce Brightness, not SMS: {reply}"
            );
        }
    }

    #[test]
    fn modem_ring_slash_and_view_open_do_not_steal_rings_sms_or_bare() {
        // Slash + view/open NL → Wake on Modem Ring (`/sms` · `/rings` parity).
        // Bare `ring` / `modem ring` stay with the agent;
        // `/rings` · `/cpu-ring` stay CPU rings; `/sms` stays Sudden Motion Sensor.
        assert!(looks_like_modem_ring_request("/ring"));
        assert!(looks_like_modem_ring_request("/ring-status"));
        assert!(looks_like_modem_ring_request("/modem-ring"));
        assert!(looks_like_modem_ring_request("/wake-on-ring"));
        assert!(looks_like_modem_ring_request("/wake-on-modem-ring"));
        assert!(looks_like_modem_ring_request("/mac-ring"));
        assert!(looks_like_modem_ring_request("/mac-modem-ring"));
        assert!(looks_like_modem_ring_request("view ring"));
        assert!(looks_like_modem_ring_request("open modem-ring"));
        assert!(looks_like_modem_ring_request("list wake on ring"));
        assert!(looks_like_modem_ring_request("ring status"));
        assert!(looks_like_modem_ring_request("is modem ring on"));
        assert!(looks_like_modem_ring_request("what's my ring"));
        assert!(looks_like_modem_ring_request("view modem ring"));
        assert!(!looks_like_modem_ring_request("ring"));
        assert!(!looks_like_modem_ring_request("modem ring"));
        assert!(!looks_like_modem_ring_request("wake on ring"));
        assert!(!looks_like_modem_ring_request("/rings"));
        assert!(!looks_like_modem_ring_request("view rings"));
        assert!(!looks_like_modem_ring_request("/cpu-ring"));
        assert!(!looks_like_modem_ring_request("cpu ring"));
        assert!(!looks_like_modem_ring_request("/sms"));
        assert!(!looks_like_modem_ring_request("/womp"));
        assert!(!looks_like_modem_ring_request("/systemsleep"));
        assert!(!looks_like_modem_ring_request("/tasks"));
        assert!(!looks_like_modem_ring_request("enable ring"));
        assert!(!looks_like_sms_request("/ring"));
        assert!(!looks_like_sms_request("view modem ring"));
        assert!(!looks_like_rings_request("/ring"));
        assert!(!looks_like_ring_chip_request("/ring"));
        let s = try_operator_instant_reply("/ring").unwrap();
        assert!(
            s.contains("**Wake on Modem Ring**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view ring").unwrap();
        assert!(
            viewed.contains("**Wake on Modem Ring**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open modem-ring").unwrap();
        assert!(
            open.contains("**Wake on Modem Ring**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("ring").is_none());
        assert!(try_operator_instant_reply("modem ring").is_none());
        let sms = try_operator_instant_reply("/sms");
        if let Some(reply) = sms {
            assert!(
                !reply.contains("**Wake on Modem Ring**"),
                "/sms must stay Sudden Motion Sensor, not modem ring: {reply}"
            );
        }
        let rings = try_operator_instant_reply("/rings");
        if let Some(reply) = rings {
            assert!(
                !reply.contains("**Wake on Modem Ring**"),
                "/rings must stay CPU rings, not modem ring: {reply}"
            );
        }
    }

    #[test]
    fn sleepdisabled_slash_and_view_open_do_not_steal_systemsleep_ring_or_bare() {
        // Slash + view/open NL → Sleep Disabled (`/systemsleep` · `/ring` parity).
        // Bare `sleepdisabled` / `sleep disabled` / `disable sleep` stay with the agent;
        // `/systemsleep` · `/sleep` stay System Sleep; `/ring` stays Wake on Modem Ring.
        assert!(looks_like_sleepdisabled_request("/sleepdisabled"));
        assert!(looks_like_sleepdisabled_request("/sleepdisabled-status"));
        assert!(looks_like_sleepdisabled_request("/sleep-disabled"));
        assert!(looks_like_sleepdisabled_request("/disable-sleep"));
        assert!(looks_like_sleepdisabled_request("/disablesleep"));
        assert!(looks_like_sleepdisabled_request("/mac-sleepdisabled"));
        assert!(looks_like_sleepdisabled_request("/mac-sleep-disabled"));
        assert!(looks_like_sleepdisabled_request("view sleepdisabled"));
        assert!(looks_like_sleepdisabled_request("open sleep-disabled"));
        assert!(looks_like_sleepdisabled_request("list disable sleep"));
        assert!(looks_like_sleepdisabled_request("sleepdisabled status"));
        assert!(looks_like_sleepdisabled_request("is sleep disabled"));
        assert!(looks_like_sleepdisabled_request("is sleepdisabled enabled"));
        assert!(looks_like_sleepdisabled_request("what's my sleepdisabled"));
        assert!(looks_like_sleepdisabled_request("view sleep disabled"));
        assert!(!looks_like_sleepdisabled_request("sleepdisabled"));
        assert!(!looks_like_sleepdisabled_request("sleep disabled"));
        assert!(!looks_like_sleepdisabled_request("disable sleep"));
        assert!(!looks_like_sleepdisabled_request("/systemsleep"));
        assert!(!looks_like_sleepdisabled_request("/sleep"));
        assert!(!looks_like_sleepdisabled_request("view system sleep"));
        assert!(!looks_like_sleepdisabled_request("/ring"));
        assert!(!looks_like_sleepdisabled_request("/displaysleep"));
        assert!(!looks_like_sleepdisabled_request("/disksleep"));
        assert!(!looks_like_sleepdisabled_request("/norestoresleep"));
        assert!(!looks_like_sleepdisabled_request("/tasks"));
        assert!(!looks_like_sleepdisabled_request("enable sleepdisabled"));
        assert!(!looks_like_systemsleep_request("/sleepdisabled"));
        assert!(!looks_like_systemsleep_request("view sleep disabled"));
        assert!(!looks_like_modem_ring_request("/sleepdisabled"));
        let s = try_operator_instant_reply("/sleepdisabled").unwrap();
        assert!(
            s.contains("**Sleep Disabled**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view sleep disabled").unwrap();
        assert!(
            viewed.contains("**Sleep Disabled**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open sleep-disabled").unwrap();
        assert!(
            open.contains("**Sleep Disabled**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("sleepdisabled").is_none());
        assert!(try_operator_instant_reply("sleep disabled").is_none());
        assert!(try_operator_instant_reply("disable sleep").is_none());
        let sys = try_operator_instant_reply("/systemsleep");
        if let Some(reply) = sys {
            assert!(
                !reply.contains("**Sleep Disabled**"),
                "/systemsleep must stay System Sleep, not Sleep Disabled: {reply}"
            );
        }
        let ring = try_operator_instant_reply("/ring");
        if let Some(reply) = ring {
            assert!(
                !reply.contains("**Sleep Disabled**"),
                "/ring must stay Wake on Modem Ring, not Sleep Disabled: {reply}"
            );
        }
    }

    #[test]
    fn assertions_slash_and_view_open_do_not_steal_systemsleep_sleepdisabled_or_bare() {
        // Slash + view/open NL → Sleep assertions (`/systemsleep` · `/sleepdisabled` parity).
        // Bare `assertions` / `sleep preventers` / `preventers` stay with the agent;
        // `/systemsleep` · `/sleep` stay System Sleep; `/sleepdisabled` stays Sleep Disabled.
        assert!(looks_like_assertions_request("/assertions"));
        assert!(looks_like_assertions_request("/assertions-status"));
        assert!(looks_like_assertions_request("/sleep-assertions"));
        assert!(looks_like_assertions_request("/sleep-preventers"));
        assert!(looks_like_assertions_request("/preventers"));
        assert!(looks_like_assertions_request("/who-keeps-awake"));
        assert!(looks_like_assertions_request("/mac-assertions"));
        assert!(looks_like_assertions_request("/mac-sleep-preventers"));
        assert!(looks_like_assertions_request("view assertions"));
        assert!(looks_like_assertions_request("open sleep-preventers"));
        assert!(looks_like_assertions_request("list preventers"));
        assert!(looks_like_assertions_request("assertions status"));
        assert!(looks_like_assertions_request("is sleep prevented"));
        assert!(looks_like_assertions_request("what's my assertions"));
        assert!(looks_like_assertions_request("view sleep preventers"));
        assert!(!looks_like_assertions_request("assertions"));
        assert!(!looks_like_assertions_request("sleep preventers"));
        assert!(!looks_like_assertions_request("preventers"));
        assert!(!looks_like_assertions_request("who keeps awake"));
        assert!(!looks_like_assertions_request("/systemsleep"));
        assert!(!looks_like_assertions_request("/sleep"));
        assert!(!looks_like_assertions_request("view system sleep"));
        assert!(!looks_like_assertions_request("/sleepdisabled"));
        assert!(!looks_like_assertions_request("/displaysleep"));
        assert!(!looks_like_assertions_request("/disksleep"));
        assert!(!looks_like_assertions_request("/norestoresleep"));
        assert!(!looks_like_assertions_request("/tasks"));
        assert!(!looks_like_assertions_request("enable assertions"));
        assert!(!looks_like_systemsleep_request("/assertions"));
        assert!(!looks_like_systemsleep_request("view sleep preventers"));
        assert!(!looks_like_sleepdisabled_request("/assertions"));
        assert!(!looks_like_sleepdisabled_request("view preventers"));
        let s = try_operator_instant_reply("/assertions").unwrap();
        assert!(
            s.contains("**Sleep assertions**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view assertions").unwrap();
        assert!(
            viewed.contains("**Sleep assertions**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open sleep-preventers").unwrap();
        assert!(
            open.contains("**Sleep assertions**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("assertions").is_none());
        assert!(try_operator_instant_reply("sleep preventers").is_none());
        assert!(try_operator_instant_reply("preventers").is_none());
        let sys = try_operator_instant_reply("/systemsleep");
        if let Some(reply) = sys {
            assert!(
                !reply.contains("**Sleep assertions**"),
                "/systemsleep must stay System Sleep, not assertions: {reply}"
            );
        }
        let sd = try_operator_instant_reply("/sleepdisabled");
        if let Some(reply) = sd {
            assert!(
                !reply.contains("**Sleep assertions**"),
                "/sleepdisabled must stay Sleep Disabled, not assertions: {reply}"
            );
        }
    }


    #[test]
    fn powersched_slash_and_view_open_do_not_steal_schedules_assertions_or_bare() {
        // Slash + view/open NL → Power schedule (`/schedules` · `/assertions` parity).
        // Bare `powersched` / `power schedule` / `scheduled power` stay with the agent;
        // `/schedules` · `/cron` stay Agent Ops; `/assertions` stays Sleep assertions.
        assert!(looks_like_powersched_request("/powersched"));
        assert!(looks_like_powersched_request("/powersched-status"));
        assert!(looks_like_powersched_request("/power-schedule"));
        assert!(looks_like_powersched_request("/pmset-sched"));
        assert!(looks_like_powersched_request("/scheduled-power"));
        assert!(looks_like_powersched_request("/mac-powersched"));
        assert!(looks_like_powersched_request("/mac-power-schedule"));
        assert!(looks_like_powersched_request("view powersched"));
        assert!(looks_like_powersched_request("open power-schedule"));
        assert!(looks_like_powersched_request("list scheduled power"));
        assert!(looks_like_powersched_request("powersched status"));
        assert!(looks_like_powersched_request("any power schedule"));
        assert!(looks_like_powersched_request("what's my power schedule"));
        assert!(looks_like_powersched_request("view power schedule"));
        assert!(!looks_like_powersched_request("powersched"));
        assert!(!looks_like_powersched_request("power schedule"));
        assert!(!looks_like_powersched_request("scheduled power"));
        assert!(!looks_like_powersched_request("/schedules"));
        assert!(!looks_like_powersched_request("/cron"));
        assert!(!looks_like_powersched_request("view schedules"));
        assert!(!looks_like_powersched_request("/assertions"));
        assert!(!looks_like_powersched_request("/systemsleep"));
        assert!(!looks_like_powersched_request("/sleep"));
        assert!(!looks_like_powersched_request("/sleepdisabled"));
        assert!(!looks_like_powersched_request("/tasks"));
        assert!(!looks_like_powersched_request("enable powersched"));
        assert!(!looks_like_schedules_request("/powersched"));
        assert!(!looks_like_schedules_request("view power schedule"));
        assert!(!looks_like_assertions_request("/powersched"));
        assert!(!looks_like_assertions_request("view power schedule"));
        let s = try_operator_instant_reply("/powersched").unwrap();
        assert!(
            s.contains("**Power schedule**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view power schedule").unwrap();
        assert!(
            viewed.contains("**Power schedule**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open power-schedule").unwrap();
        assert!(
            open.contains("**Power schedule**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("powersched").is_none());
        assert!(try_operator_instant_reply("power schedule").is_none());
        assert!(try_operator_instant_reply("scheduled power").is_none());
        let sched = try_operator_instant_reply("/schedules");
        if let Some(reply) = sched {
            assert!(
                !reply.contains("**Power schedule**"),
                "/schedules must stay Agent Ops, not Power schedule: {reply}"
            );
        }
        let assertions = try_operator_instant_reply("/assertions");
        if let Some(reply) = assertions {
            assert!(
                !reply.contains("**Power schedule**"),
                "/assertions must stay Sleep assertions, not Power schedule: {reply}"
            );
        }
    }

    #[test]
    fn ups_slash_and_view_open_do_not_steal_battery_charger_or_bare() {
        // Slash + view/open NL → UPS (`/battery` · `/charger` parity).
        // Bare `ups` / `ups settings` stay with the agent;
        // `/battery` stays Bat %; `/charger` stays AC charger; `/sysload` stays System load.
        assert!(looks_like_ups_request("/ups"));
        assert!(looks_like_ups_request("/ups-status"));
        assert!(looks_like_ups_request("/ups-settings"));
        assert!(looks_like_ups_request("/pmset-ups"));
        assert!(looks_like_ups_request("/uninterruptible-power"));
        assert!(looks_like_ups_request("/mac-ups"));
        assert!(looks_like_ups_request("/mac-ups-settings"));
        assert!(looks_like_ups_request("view ups"));
        assert!(looks_like_ups_request("open ups-settings"));
        assert!(looks_like_ups_request("list ups"));
        assert!(looks_like_ups_request("ups status"));
        assert!(looks_like_ups_request("is ups attached"));
        assert!(looks_like_ups_request("what's my ups"));
        assert!(looks_like_ups_request("view ups settings"));
        assert!(!looks_like_ups_request("ups"));
        assert!(!looks_like_ups_request("ups settings"));
        assert!(!looks_like_ups_request("/battery"));
        assert!(!looks_like_ups_request("/bat"));
        assert!(!looks_like_ups_request("/charger"));
        assert!(!looks_like_ups_request("/adapter"));
        assert!(!looks_like_ups_request("/sysload"));
        assert!(!looks_like_ups_request("/powersched"));
        assert!(!looks_like_ups_request("/tasks"));
        assert!(!looks_like_ups_request("enable ups"));
        assert!(!looks_like_sysload_request("/ups"));
        assert!(!looks_like_sysload_request("view ups"));
        let s = try_operator_instant_reply("/ups").unwrap();
        assert!(
            s.contains("**UPS**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view ups").unwrap();
        assert!(
            viewed.contains("**UPS**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open ups-settings").unwrap();
        assert!(
            open.contains("**UPS**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("ups").is_none());
        assert!(try_operator_instant_reply("ups settings").is_none());
        let bat = try_operator_instant_reply("/battery");
        if let Some(reply) = bat {
            assert!(
                !reply.contains("**UPS**"),
                "/battery must stay Bat %, not UPS: {reply}"
            );
        }
        let charger = try_operator_instant_reply("/charger");
        if let Some(reply) = charger {
            assert!(
                !reply.contains("**UPS**"),
                "/charger must stay AC charger, not UPS: {reply}"
            );
        }
    }

    #[test]
    fn pmlog_slash_and_view_open_do_not_steal_logs_assertions_or_bare() {
        // Slash + view/open NL → Power management log (`/assertions` · `/logs` parity).
        // Bare `pmlog` / `power log` stay with the agent;
        // `/logs` stays Debug Log; `/assertions` stays Sleep assertions;
        // `/sysload` stays System load; `/powersched` stays Power schedule; `/ups` stays UPS.
        assert!(looks_like_pmlog_request("/pmlog"));
        assert!(looks_like_pmlog_request("/pmlog-status"));
        assert!(looks_like_pmlog_request("/pm-log"));
        assert!(looks_like_pmlog_request("/powerlog"));
        assert!(looks_like_pmlog_request("/power-log"));
        assert!(looks_like_pmlog_request("/powermanagement-log"));
        assert!(looks_like_pmlog_request("/pmset-log"));
        assert!(looks_like_pmlog_request("/sleep-wake-log"));
        assert!(looks_like_pmlog_request("/mac-pmlog"));
        assert!(looks_like_pmlog_request("/mac-powerlog"));
        assert!(looks_like_pmlog_request("view pmlog"));
        assert!(looks_like_pmlog_request("open powerlog"));
        assert!(looks_like_pmlog_request("list power management log"));
        assert!(looks_like_pmlog_request("pmlog status"));
        assert!(looks_like_pmlog_request("is pmlog ok"));
        assert!(looks_like_pmlog_request("what's my pmlog"));
        assert!(looks_like_pmlog_request("view power log"));
        assert!(!looks_like_pmlog_request("pmlog"));
        assert!(!looks_like_pmlog_request("power log"));
        assert!(!looks_like_pmlog_request("power management log"));
        assert!(!looks_like_pmlog_request("/logs"));
        assert!(!looks_like_pmlog_request("view logs"));
        assert!(!looks_like_pmlog_request("/assertions"));
        assert!(!looks_like_pmlog_request("view assertions"));
        assert!(!looks_like_pmlog_request("/sysload"));
        assert!(!looks_like_pmlog_request("/powersched"));
        assert!(!looks_like_pmlog_request("/ups"));
        assert!(!looks_like_pmlog_request("/thermal"));
        assert!(!looks_like_pmlog_request("/tasks"));
        assert!(!looks_like_pmlog_request("enable pmlog"));
        assert!(!looks_like_sysload_request("/pmlog"));
        assert!(!looks_like_sysload_request("view pmlog"));
        assert!(!looks_like_assertions_request("/pmlog"));
        assert!(!looks_like_assertions_request("view pmlog"));
        assert!(!looks_like_ups_request("/pmlog"));
        assert!(!looks_like_ups_request("view pmlog"));
        let s = try_operator_instant_reply("/pmlog").unwrap();
        assert!(
            s.contains("**Power log**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view pmlog").unwrap();
        assert!(
            viewed.contains("**Power log**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open powerlog").unwrap();
        assert!(
            open.contains("**Power log**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("pmlog").is_none());
        assert!(try_operator_instant_reply("power log").is_none());
        let assertions = try_operator_instant_reply("/assertions");
        if let Some(reply) = assertions {
            assert!(
                !reply.contains("**Power log**"),
                "/assertions must stay Sleep assertions, not Power log: {reply}"
            );
        }
        let ups = try_operator_instant_reply("/ups");
        if let Some(reply) = ups {
            assert!(
                !reply.contains("**Power log**"),
                "/ups must stay UPS, not Power log: {reply}"
            );
        }
    }

    #[test]
    fn therm_slash_and_view_open_do_not_steal_heat_pmlog_or_bare() {
        // Slash + view/open NL → Therm / CPU speed limits (`/thermal` · `/heat` parity).
        // Bare `therm` / `cpu speed limit` / `thermal limits` stay with the agent;
        // `/thermal` · `/heat` stay Heat; `/pmlog` stays Power log; `/sysload` stays System load.
        assert!(looks_like_therm_request("/therm"));
        assert!(looks_like_therm_request("/therm-status"));
        assert!(looks_like_therm_request("/pmset-therm"));
        assert!(looks_like_therm_request("/cpu-speed-limit"));
        assert!(looks_like_therm_request("/speed-limit"));
        assert!(looks_like_therm_request("/thermal-limits"));
        assert!(looks_like_therm_request("/thermal-limit"));
        assert!(looks_like_therm_request("/sched-limit"));
        assert!(looks_like_therm_request("/mac-therm"));
        assert!(looks_like_therm_request("/mac-cpu-speed-limit"));
        assert!(looks_like_therm_request("view therm"));
        assert!(looks_like_therm_request("open cpu-speed-limit"));
        assert!(looks_like_therm_request("list thermal limits"));
        assert!(looks_like_therm_request("therm status"));
        assert!(looks_like_therm_request("is therm ok"));
        assert!(looks_like_therm_request("what's my therm"));
        assert!(looks_like_therm_request("view thermal limits"));
        assert!(!looks_like_therm_request("therm"));
        assert!(!looks_like_therm_request("cpu speed limit"));
        assert!(!looks_like_therm_request("thermal limits"));
        assert!(!looks_like_therm_request("/thermal"));
        assert!(!looks_like_therm_request("view thermal"));
        assert!(!looks_like_therm_request("/heat"));
        assert!(!looks_like_therm_request("view heat"));
        assert!(!looks_like_therm_request("/pmlog"));
        assert!(!looks_like_therm_request("/sysload"));
        assert!(!looks_like_therm_request("/ups"));
        assert!(!looks_like_therm_request("/pmstats"));
        assert!(!looks_like_therm_request("view pmstats"));
        assert!(!looks_like_therm_request("/tasks"));
        assert!(!looks_like_therm_request("enable therm"));
        assert!(!looks_like_pmlog_request("/therm"));
        assert!(!looks_like_pmlog_request("view therm"));
        assert!(!looks_like_sysload_request("/therm"));
        assert!(!looks_like_sysload_request("view therm"));
        assert!(!looks_like_ups_request("/therm"));
        let s = try_operator_instant_reply("/therm").unwrap();
        assert!(
            s.contains("**Therm**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view therm").unwrap();
        assert!(
            viewed.contains("**Therm**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open cpu-speed-limit").unwrap();
        assert!(
            open.contains("**Therm**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("therm").is_none());
        assert!(try_operator_instant_reply("cpu speed limit").is_none());
        assert!(try_operator_instant_reply("thermal limits").is_none());
        let heat = try_operator_instant_reply("/thermal");
        if let Some(reply) = heat {
            assert!(
                !reply.contains("**Therm**"),
                "/thermal must stay Heat, not Therm: {reply}"
            );
        }
        let pmlog = try_operator_instant_reply("/pmlog");
        if let Some(reply) = pmlog {
            assert!(
                !reply.contains("**Therm**"),
                "/pmlog must stay Power log, not Therm: {reply}"
            );
        }
    }

    #[test]
    fn pmstats_slash_and_view_open_do_not_steal_pmlog_therm_or_bare() {
        // Slash + view/open NL -> Power stats (/pmlog · /therm parity).
        assert!(looks_like_pmstats_request("/pmstats"));
        assert!(looks_like_pmstats_request("/pmstats-status"));
        assert!(looks_like_pmstats_request("/pm-stats"));
        assert!(looks_like_pmstats_request("/powerstats"));
        assert!(looks_like_pmstats_request("/power-stats"));
        assert!(looks_like_pmstats_request("/sleep-stats"));
        assert!(looks_like_pmstats_request("/wake-stats"));
        assert!(looks_like_pmstats_request("/sleep-wake-stats"));
        assert!(looks_like_pmstats_request("/pmset-stats"));
        assert!(looks_like_pmstats_request("/mac-pmstats"));
        assert!(looks_like_pmstats_request("/mac-power-stats"));
        assert!(looks_like_pmstats_request("view pmstats"));
        assert!(looks_like_pmstats_request("open power-stats"));
        assert!(looks_like_pmstats_request("list sleep stats"));
        assert!(looks_like_pmstats_request("pmstats status"));
        assert!(looks_like_pmstats_request("is pmstats ok"));
        assert!(looks_like_pmstats_request("what's my pmstats"));
        assert!(looks_like_pmstats_request("view power stats"));
        assert!(!looks_like_pmstats_request("pmstats"));
        assert!(!looks_like_pmstats_request("power stats"));
        assert!(!looks_like_pmstats_request("sleep stats"));
        assert!(!looks_like_pmstats_request("wake stats"));
        assert!(!looks_like_pmstats_request("/stats"));
        assert!(!looks_like_pmstats_request("/pmlog"));
        assert!(!looks_like_pmstats_request("view pmlog"));
        assert!(!looks_like_pmstats_request("/therm"));
        assert!(!looks_like_pmstats_request("view therm"));
        assert!(!looks_like_pmstats_request("/darkwakes"));
        assert!(!looks_like_pmstats_request("view dark wakes"));
        assert!(!looks_like_pmstats_request("/assertions"));
        assert!(!looks_like_pmstats_request("/sysload"));
        assert!(!looks_like_pmstats_request("/powersched"));
        assert!(!looks_like_pmstats_request("/ups"));
        assert!(!looks_like_pmstats_request("/thermal"));
        assert!(!looks_like_pmstats_request("/tasks"));
        assert!(!looks_like_pmstats_request("enable pmstats"));
        assert!(!looks_like_pmlog_request("/pmstats"));
        assert!(!looks_like_pmlog_request("view pmstats"));
        assert!(!looks_like_therm_request("/pmstats"));
        assert!(!looks_like_therm_request("view pmstats"));
        assert!(!looks_like_darkwakes_request("/pmstats"));
        let s = try_operator_instant_reply("/pmstats").unwrap();
        assert!(
            s.contains("**Power stats**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view pmstats").unwrap();
        assert!(
            viewed.contains("**Power stats**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open power-stats").unwrap();
        assert!(
            open.contains("**Power stats**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("pmstats").is_none());
        assert!(try_operator_instant_reply("power stats").is_none());
        let pmlog = try_operator_instant_reply("/pmlog");
        if let Some(reply) = pmlog {
            assert!(
                !reply.contains("**Power stats**"),
                "/pmlog must stay Power log, not Power stats: {reply}"
            );
        }
        let therm = try_operator_instant_reply("/therm");
        if let Some(reply) = therm {
            assert!(
                !reply.contains("**Power stats**"),
                "/therm must stay Therm, not Power stats: {reply}"
            );
        }
    }

    #[test]
    fn pmcap_slash_and_view_open_do_not_steal_pmstats_adapter_or_bare() {
        // Slash + view/open NL -> Power capabilities (/pmstats · /adapter parity).
        assert!(looks_like_pmcap_request("/pmcap"));
        assert!(looks_like_pmcap_request("/pmcap-status"));
        assert!(looks_like_pmcap_request("/pm-cap"));
        assert!(looks_like_pmcap_request("/pmset-cap"));
        assert!(looks_like_pmcap_request("/power-cap"));
        assert!(looks_like_pmcap_request("/power-capabilities"));
        assert!(looks_like_pmcap_request("/pm-capabilities"));
        assert!(looks_like_pmcap_request("/mac-pmcap"));
        assert!(looks_like_pmcap_request("/mac-power-cap"));
        assert!(looks_like_pmcap_request("view pmcap"));
        assert!(looks_like_pmcap_request("open power-cap"));
        assert!(looks_like_pmcap_request("list power capabilities"));
        assert!(looks_like_pmcap_request("pmcap status"));
        assert!(looks_like_pmcap_request("is pmcap ok"));
        assert!(looks_like_pmcap_request("what's my pmcap"));
        assert!(looks_like_pmcap_request("view power capabilities"));
        assert!(!looks_like_pmcap_request("pmcap"));
        assert!(!looks_like_pmcap_request("power cap"));
        assert!(!looks_like_pmcap_request("power capabilities"));
        assert!(!looks_like_pmcap_request("pmset cap"));
        assert!(!looks_like_pmcap_request("capabilities"));
        assert!(!looks_like_pmcap_request("/pmstats"));
        assert!(!looks_like_pmcap_request("view pmstats"));
        assert!(!looks_like_pmcap_request("/therm"));
        assert!(!looks_like_pmcap_request("view therm"));
        assert!(!looks_like_pmcap_request("/pmlog"));
        assert!(!looks_like_pmcap_request("/adapter"));
        assert!(!looks_like_pmcap_request("/charger"));
        assert!(!looks_like_pmcap_request("/ram-capacity"));
        assert!(!looks_like_pmcap_request("/disk-capacity"));
        assert!(!looks_like_pmcap_request("/thermal"));
        assert!(!looks_like_pmcap_request("/tasks"));
        assert!(!looks_like_pmcap_request("enable pmcap"));
        assert!(!looks_like_pmstats_request("/pmcap"));
        assert!(!looks_like_pmstats_request("view pmcap"));
        assert!(!looks_like_therm_request("/pmcap"));
        assert!(!looks_like_therm_request("view pmcap"));
        assert!(!looks_like_ac_charger_request("/pmcap"));
        let s = try_operator_instant_reply("/pmcap").unwrap();
        assert!(
            s.contains("**Power capabilities**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view pmcap").unwrap();
        assert!(
            viewed.contains("**Power capabilities**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open power-cap").unwrap();
        assert!(
            open.contains("**Power capabilities**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("pmcap").is_none());
        assert!(try_operator_instant_reply("power capabilities").is_none());
        let pmstats = try_operator_instant_reply("/pmstats");
        if let Some(reply) = pmstats {
            assert!(
                !reply.contains("**Power capabilities**"),
                "/pmstats must stay Power stats, not Power capabilities: {reply}"
            );
        }
        let adapter = try_operator_instant_reply("/adapter");
        if let Some(reply) = adapter {
            assert!(
                !reply.contains("**Power capabilities**"),
                "/adapter must stay AC charger, not Power capabilities: {reply}"
            );
        }
    }

    #[test]
    fn useractivity_slash_and_view_open_do_not_steal_idle_username_or_bare() {
        // Slash + view/open NL -> User activity (/idle-time · /username parity).
        assert!(looks_like_useractivity_request("/useractivity"));
        assert!(looks_like_useractivity_request("/useractivity-status"));
        assert!(looks_like_useractivity_request("/user-activity"));
        assert!(looks_like_useractivity_request("/pmset-useractivity"));
        assert!(looks_like_useractivity_request("/user-activity-level"));
        assert!(looks_like_useractivity_request("/activity-level"));
        assert!(looks_like_useractivity_request("/mac-useractivity"));
        assert!(looks_like_useractivity_request("/mac-activity-level"));
        assert!(looks_like_useractivity_request("view useractivity"));
        assert!(looks_like_useractivity_request("open user-activity"));
        assert!(looks_like_useractivity_request("list activity level"));
        assert!(looks_like_useractivity_request("useractivity status"));
        assert!(looks_like_useractivity_request("is useractivity ok"));
        assert!(looks_like_useractivity_request("what's my user activity"));
        assert!(looks_like_useractivity_request("view activity level"));
        assert!(!looks_like_useractivity_request("useractivity"));
        assert!(!looks_like_useractivity_request("user activity"));
        assert!(!looks_like_useractivity_request("activity level"));
        assert!(!looks_like_useractivity_request("pmset useractivity"));
        assert!(!looks_like_useractivity_request("/idle-time"));
        assert!(!looks_like_useractivity_request("view screensaver"));
        assert!(!looks_like_useractivity_request("/username"));
        assert!(!looks_like_useractivity_request("/whoami"));
        assert!(!looks_like_useractivity_request("/user"));
        assert!(!looks_like_useractivity_request("/pmcap"));
        assert!(!looks_like_useractivity_request("view pmcap"));
        assert!(!looks_like_useractivity_request("/pmstats"));
        assert!(!looks_like_useractivity_request("/tasks"));
        assert!(!looks_like_useractivity_request("enable useractivity"));
        assert!(!looks_like_pmcap_request("/useractivity"));
        assert!(!looks_like_pmcap_request("view useractivity"));
        assert!(!looks_like_screensaver_idle_request("/useractivity"));
        assert!(!looks_like_username_request("/useractivity"));
        let s = try_operator_instant_reply("/useractivity").unwrap();
        assert!(
            s.contains("**User activity**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view useractivity").unwrap();
        assert!(
            viewed.contains("**User activity**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open user-activity").unwrap();
        assert!(
            open.contains("**User activity**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("useractivity").is_none());
        assert!(try_operator_instant_reply("user activity").is_none());
        let idle = try_operator_instant_reply("/idle-time");
        if let Some(reply) = idle {
            assert!(
                !reply.contains("**User activity**"),
                "/idle-time must stay Screensaver idle, not User activity: {reply}"
            );
        }
        let username = try_operator_instant_reply("/username");
        if let Some(reply) = username {
            assert!(
                !reply.contains("**User activity**"),
                "/username must stay Username, not User activity: {reply}"
            );
        }
        let pmcap = try_operator_instant_reply("/pmcap");
        if let Some(reply) = pmcap {
            assert!(
                !reply.contains("**User activity**"),
                "/pmcap must stay Power capabilities, not User activity: {reply}"
            );
        }
    }

    #[test]
    fn pmps_slash_and_view_open_do_not_steal_battery_adapter_or_bare() {
        // Slash + view/open NL → Power source (`/battery` · `/adapter` parity).
        assert!(looks_like_pmps_request("/pmps"));
        assert!(looks_like_pmps_request("/pmps-status"));
        assert!(looks_like_pmps_request("/pm-ps"));
        assert!(looks_like_pmps_request("/pmset-ps"));
        assert!(looks_like_pmps_request("/power-source"));
        assert!(looks_like_pmps_request("/now-drawing"));
        assert!(looks_like_pmps_request("/drawing-from"));
        assert!(looks_like_pmps_request("/ps"));
        assert!(looks_like_pmps_request("/mac-pmps"));
        assert!(looks_like_pmps_request("/mac-power-source"));
        assert!(looks_like_pmps_request("view pmps"));
        assert!(looks_like_pmps_request("open power-source"));
        assert!(looks_like_pmps_request("list now drawing"));
        assert!(looks_like_pmps_request("pmps status"));
        assert!(looks_like_pmps_request("is pmps ok"));
        assert!(looks_like_pmps_request("what's my power source"));
        assert!(looks_like_pmps_request("view now drawing"));
        assert!(!looks_like_pmps_request("pmps"));
        assert!(!looks_like_pmps_request("power source"));
        assert!(!looks_like_pmps_request("now drawing"));
        assert!(!looks_like_pmps_request("ps"));
        assert!(!looks_like_pmps_request("pmset ps"));
        assert!(!looks_like_pmps_request("/battery"));
        assert!(!looks_like_pmps_request("view battery"));
        assert!(!looks_like_pmps_request("/adapter"));
        assert!(!looks_like_pmps_request("/charger"));
        assert!(!looks_like_pmps_request("/ups"));
        assert!(!looks_like_pmps_request("/useractivity"));
        assert!(!looks_like_pmps_request("/processes"));
        assert!(!looks_like_pmps_request("/top"));
        assert!(!looks_like_pmps_request("/tasks"));
        assert!(!looks_like_pmps_request("enable pmps"));
        assert!(!looks_like_ac_charger_request("/pmps"));
        assert!(!looks_like_ac_charger_request("view pmps"));
        assert!(!looks_like_ups_request("/pmps"));
        assert!(!looks_like_ups_request("view pmps"));
        assert!(!looks_like_useractivity_request("/pmps"));
        assert!(!looks_like_useractivity_request("view pmps"));
        let s = try_operator_instant_reply("/pmps").unwrap();
        assert!(
            s.contains("**Power source**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view pmps").unwrap();
        assert!(
            viewed.contains("**Power source**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open power-source").unwrap();
        assert!(
            open.contains("**Power source**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("pmps").is_none());
        assert!(try_operator_instant_reply("power source").is_none());
        let battery = try_operator_instant_reply("/battery");
        if let Some(reply) = battery {
            assert!(
                !reply.contains("**Power source**"),
                "/battery must stay Bat %, not Power source: {reply}"
            );
        }
        let adapter = try_operator_instant_reply("/adapter");
        if let Some(reply) = adapter {
            assert!(
                !reply.contains("**Power source**"),
                "/adapter must stay AC charger, not Power source: {reply}"
            );
        }
        let useractivity = try_operator_instant_reply("/useractivity");
        if let Some(reply) = useractivity {
            assert!(
                !reply.contains("**Power source**"),
                "/useractivity must stay User activity, not Power source: {reply}"
            );
        }
    }

    #[test]
    fn pmcustom_slash_and_view_open_do_not_steal_keys_pmcap_or_bare() {
        // Slash + view/open NL -> Power custom (/pmcap · /pmps · /useractivity parity).
        // `/pmdisk` aliases the same `pmset -g custom` dump (`pmset -g disk` ≡ custom).
        assert!(looks_like_pmcustom_request("/pmcustom"));
        assert!(looks_like_pmcustom_request("/pmcustom-status"));
        assert!(looks_like_pmcustom_request("/pm-custom"));
        assert!(looks_like_pmcustom_request("/pmset-custom"));
        assert!(looks_like_pmcustom_request("/pmdisk"));
        assert!(looks_like_pmcustom_request("/pmdisk-status"));
        assert!(looks_like_pmcustom_request("/pm-disk"));
        assert!(looks_like_pmcustom_request("/pmset-disk"));
        assert!(looks_like_pmcustom_request("/power-disk"));
        assert!(looks_like_pmcustom_request("/mac-pmdisk"));
        assert!(looks_like_pmcustom_request("/power-custom"));
        assert!(looks_like_pmcustom_request("/power-profiles"));
        assert!(looks_like_pmcustom_request("/battery-ac-profiles"));
        assert!(looks_like_pmcustom_request("/mac-pmcustom"));
        assert!(looks_like_pmcustom_request("/mac-power-profiles"));
        assert!(looks_like_pmcustom_request("view pmcustom"));
        assert!(looks_like_pmcustom_request("view pmdisk"));
        assert!(looks_like_pmcustom_request("open power-profiles"));
        assert!(looks_like_pmcustom_request("open pmdisk"));
        assert!(looks_like_pmcustom_request("list power profiles"));
        assert!(looks_like_pmcustom_request("list pmdisk"));
        assert!(looks_like_pmcustom_request("pmcustom status"));
        assert!(looks_like_pmcustom_request("pmdisk status"));
        assert!(looks_like_pmcustom_request("is pmcustom ok"));
        assert!(looks_like_pmcustom_request("is pmdisk ok"));
        assert!(looks_like_pmcustom_request("what's my power profiles"));
        assert!(looks_like_pmcustom_request("what's my pmdisk"));
        assert!(looks_like_pmcustom_request("view battery ac profiles"));
        assert!(!looks_like_pmcustom_request("pmcustom"));
        assert!(!looks_like_pmcustom_request("pmdisk"));
        assert!(!looks_like_pmcustom_request("power custom"));
        assert!(!looks_like_pmcustom_request("power profiles"));
        assert!(!looks_like_pmcustom_request("pmset custom"));
        assert!(!looks_like_pmcustom_request("pmset disk"));
        assert!(!looks_like_pmcustom_request("/disk"));
        assert!(!looks_like_pmcustom_request("view disk"));
        assert!(!looks_like_pmcustom_request("open disk"));
        assert!(!looks_like_pmcustom_request("list disk"));
        assert!(!looks_like_pmcustom_request("/disksleep"));
        assert!(!looks_like_pmcustom_request("/powernap"));
        assert!(!looks_like_pmcustom_request("view powernap"));
        assert!(!looks_like_pmcustom_request("/womp"));
        assert!(!looks_like_pmcustom_request("/displaysleep"));
        assert!(!looks_like_pmcustom_request("/pmcap"));
        assert!(!looks_like_pmcustom_request("view pmcap"));
        assert!(!looks_like_pmcustom_request("/useractivity"));
        assert!(!looks_like_pmcustom_request("view useractivity"));
        assert!(!looks_like_pmcustom_request("/pmps"));
        assert!(!looks_like_pmcustom_request("view pmps"));
        assert!(!looks_like_pmcustom_request("/pmstats"));
        assert!(!looks_like_pmcustom_request("/adapter"));
        assert!(!looks_like_pmcustom_request("/pmadapters"));
        assert!(!looks_like_pmcustom_request("/adapters"));
        assert!(!looks_like_pmcustom_request("/tasks"));
        assert!(!looks_like_pmcustom_request("enable pmcustom"));
        assert!(!looks_like_pmcap_request("/pmcustom"));
        assert!(!looks_like_pmcap_request("view pmcustom"));
        assert!(!looks_like_useractivity_request("/pmcustom"));
        assert!(!looks_like_useractivity_request("view pmcustom"));
        assert!(!looks_like_pmps_request("/pmcustom"));
        assert!(!looks_like_power_nap_request("/pmcustom"));
        assert!(!looks_like_disk_cleanup_request("/pmdisk"));
        assert!(!looks_like_disk_cleanup_request("view pmdisk"));
        let s = try_operator_instant_reply("/pmcustom").unwrap();
        assert!(
            s.contains("**Power custom**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let disk = try_operator_instant_reply("/pmdisk").unwrap();
        assert!(
            disk.contains("**Power custom**") && !disk.to_lowercase().contains("execution"),
            "got: {disk}"
        );
        let viewed = try_operator_instant_reply("view pmcustom").unwrap();
        assert!(
            viewed.contains("**Power custom**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let viewed_disk = try_operator_instant_reply("view pmdisk").unwrap();
        assert!(
            viewed_disk.contains("**Power custom**")
                && !viewed_disk.to_lowercase().contains("execution"),
            "got: {viewed_disk}"
        );
        let open = try_operator_instant_reply("open power-profiles").unwrap();
        assert!(
            open.contains("**Power custom**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("pmcustom").is_none());
        assert!(try_operator_instant_reply("pmdisk").is_none());
        assert!(try_operator_instant_reply("power profiles").is_none());
        let pmcap = try_operator_instant_reply("/pmcap");
        if let Some(reply) = pmcap {
            assert!(
                !reply.contains("**Power custom**"),
                "/pmcap must stay Power capabilities, not Power custom: {reply}"
            );
        }
        let pmps = try_operator_instant_reply("/pmps");
        if let Some(reply) = pmps {
            assert!(
                !reply.contains("**Power custom**"),
                "/pmps must stay Power source, not Power custom: {reply}"
            );
        }
        let nap = try_operator_instant_reply("/powernap");
        if let Some(reply) = nap {
            assert!(
                !reply.contains("**Power custom**"),
                "/powernap must stay Power Nap, not Power custom: {reply}"
            );
        }
    }

    #[test]
    fn pmadapters_slash_and_view_open_do_not_steal_adapter_pmcustom_or_bare() {
        // Slash + view/open NL → Power adapters (`/adapter` · `/pmcustom` parity).
        assert!(looks_like_pmadapters_request("/pmadapters"));
        assert!(looks_like_pmadapters_request("/pmadapters-status"));
        assert!(looks_like_pmadapters_request("/pm-adapters"));
        assert!(looks_like_pmadapters_request("/pmset-adapters"));
        assert!(looks_like_pmadapters_request("/power-adapters"));
        assert!(looks_like_pmadapters_request("/adapters"));
        assert!(looks_like_pmadapters_request("/adapter-list"));
        assert!(looks_like_pmadapters_request("/mac-pmadapters"));
        assert!(looks_like_pmadapters_request("/mac-power-adapters"));
        assert!(looks_like_pmadapters_request("view pmadapters"));
        assert!(looks_like_pmadapters_request("open power-adapters"));
        assert!(looks_like_pmadapters_request("list adapters"));
        assert!(looks_like_pmadapters_request("pmadapters status"));
        assert!(looks_like_pmadapters_request("is pmadapters ok"));
        assert!(looks_like_pmadapters_request("what's my power adapters"));
        assert!(looks_like_pmadapters_request("view adapters"));
        assert!(!looks_like_pmadapters_request("pmadapters"));
        assert!(!looks_like_pmadapters_request("power adapters"));
        assert!(!looks_like_pmadapters_request("adapters"));
        assert!(!looks_like_pmadapters_request("pmset adapters"));
        assert!(!looks_like_pmadapters_request("/adapter"));
        assert!(!looks_like_pmadapters_request("view adapter"));
        assert!(!looks_like_pmadapters_request("/charger"));
        assert!(!looks_like_pmadapters_request("view charger"));
        assert!(!looks_like_pmadapters_request("/pmcustom"));
        assert!(!looks_like_pmadapters_request("view pmcustom"));
        assert!(!looks_like_pmadapters_request("/pmps"));
        assert!(!looks_like_pmadapters_request("/ups"));
        assert!(!looks_like_pmadapters_request("/battery"));
        assert!(!looks_like_pmadapters_request("/tasks"));
        assert!(!looks_like_pmadapters_request("enable pmadapters"));
        assert!(!looks_like_ac_charger_request("/pmadapters"));
        assert!(!looks_like_ac_charger_request("/adapters"));
        assert!(!looks_like_ac_charger_request("view pmadapters"));
        assert!(!looks_like_ac_charger_request("list adapters"));
        assert!(!looks_like_pmcustom_request("/pmadapters"));
        assert!(!looks_like_pmcustom_request("view pmadapters"));
        assert!(!looks_like_pmps_request("/pmadapters"));
        let s = try_operator_instant_reply("/pmadapters").unwrap();
        assert!(
            s.contains("**Power adapters**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view pmadapters").unwrap();
        assert!(
            viewed.contains("**Power adapters**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open power-adapters").unwrap();
        assert!(
            open.contains("**Power adapters**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("pmadapters").is_none());
        assert!(try_operator_instant_reply("power adapters").is_none());
        assert!(try_operator_instant_reply("adapters").is_none());
        let adapter = try_operator_instant_reply("/adapter");
        if let Some(reply) = adapter {
            assert!(
                !reply.contains("**Power adapters**"),
                "/adapter must stay AC charger, not Power adapters: {reply}"
            );
        }
        let pmcustom = try_operator_instant_reply("/pmcustom");
        if let Some(reply) = pmcustom {
            assert!(
                !reply.contains("**Power adapters**"),
                "/pmcustom must stay Power custom, not Power adapters: {reply}"
            );
        }
    }



    #[test]
    fn pmlive_slash_and_view_open_do_not_steal_pmcustom_pmadapters_or_bare() {
        assert!(looks_like_pmlive_request("/pmlive"));
        assert!(looks_like_pmlive_request("/pmlive-status"));
        assert!(looks_like_pmlive_request("/pm-live"));
        assert!(looks_like_pmlive_request("/pmset-live"));
        assert!(looks_like_pmlive_request("/power-live"));
        assert!(looks_like_pmlive_request("/currently-in-use"));
        assert!(looks_like_pmlive_request("/in-use"));
        assert!(looks_like_pmlive_request("/live-settings"));
        assert!(looks_like_pmlive_request("/mac-pmlive"));
        assert!(looks_like_pmlive_request("/mac-power-live"));
        assert!(looks_like_pmlive_request("view pmlive"));
        assert!(looks_like_pmlive_request("open power-live"));
        assert!(looks_like_pmlive_request("list currently in use"));
        assert!(looks_like_pmlive_request("pmlive status"));
        assert!(looks_like_pmlive_request("is pmlive ok"));
        assert!(looks_like_pmlive_request("what's my currently in use"));
        assert!(looks_like_pmlive_request("view currently in use"));
        assert!(!looks_like_pmlive_request("pmlive"));
        assert!(!looks_like_pmlive_request("power live"));
        assert!(!looks_like_pmlive_request("currently in use"));
        assert!(!looks_like_pmlive_request("pmset live"));
        assert!(!looks_like_pmlive_request("/pmcustom"));
        assert!(!looks_like_pmlive_request("view pmcustom"));
        assert!(!looks_like_pmlive_request("/pmadapters"));
        assert!(!looks_like_pmlive_request("view pmadapters"));
        assert!(!looks_like_pmlive_request("/pmeverything"));
        assert!(!looks_like_pmlive_request("view pmeverything"));
        assert!(!looks_like_pmlive_request("/pmps"));
        assert!(!looks_like_pmlive_request("/pmcap"));
        assert!(!looks_like_pmlive_request("/powernap"));
        assert!(!looks_like_pmlive_request("/tasks"));
        assert!(!looks_like_pmlive_request("enable pmlive"));
        assert!(!looks_like_pmcustom_request("/pmlive"));
        assert!(!looks_like_pmcustom_request("view pmlive"));
        assert!(!looks_like_pmadapters_request("/pmlive"));
        assert!(!looks_like_pmadapters_request("view pmlive"));
        let s = try_operator_instant_reply("/pmlive").unwrap();
        assert!(
            s.contains("**Power live**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view pmlive").unwrap();
        assert!(
            viewed.contains("**Power live**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open currently-in-use").unwrap();
        assert!(
            open.contains("**Power live**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("pmlive").is_none());
        assert!(try_operator_instant_reply("power live").is_none());
        assert!(try_operator_instant_reply("currently in use").is_none());
    }

    #[test]
    fn pmeverything_slash_and_view_open_do_not_steal_pmlive_or_bare_or_health() {
        assert!(looks_like_pmeverything_request("/pmeverything"));
        assert!(looks_like_pmeverything_request("/pmeverything-status"));
        assert!(looks_like_pmeverything_request("/pm-everything"));
        assert!(looks_like_pmeverything_request("/pmset-everything"));
        assert!(looks_like_pmeverything_request("/power-everything"));
        assert!(looks_like_pmeverything_request("/all-power-settings"));
        assert!(looks_like_pmeverything_request("/power-all-settings"));
        assert!(looks_like_pmeverything_request("/mac-pmeverything"));
        assert!(looks_like_pmeverything_request("/mac-power-everything"));
        assert!(looks_like_pmeverything_request("view pmeverything"));
        assert!(looks_like_pmeverything_request("open power-everything"));
        assert!(looks_like_pmeverything_request("list all power settings"));
        assert!(looks_like_pmeverything_request("pmeverything status"));
        assert!(looks_like_pmeverything_request("is pmeverything ok"));
        assert!(looks_like_pmeverything_request("what's my all power settings"));
        assert!(looks_like_pmeverything_request("view all power settings"));
        assert!(!looks_like_pmeverything_request("pmeverything"));
        assert!(!looks_like_pmeverything_request("power everything"));
        assert!(!looks_like_pmeverything_request("all power settings"));
        assert!(!looks_like_pmeverything_request("pmset everything"));
        assert!(!looks_like_pmeverything_request("everything"));
        assert!(!looks_like_pmeverything_request("is everything ok"));
        assert!(!looks_like_pmeverything_request("everything ok"));
        assert!(!looks_like_pmeverything_request("everything working"));
        assert!(!looks_like_pmeverything_request("/pmlive"));
        assert!(!looks_like_pmeverything_request("view pmlive"));
        assert!(!looks_like_pmeverything_request("/pmcustom"));
        assert!(!looks_like_pmeverything_request("/pmadapters"));
        assert!(!looks_like_pmeverything_request("/pmps"));
        assert!(!looks_like_pmeverything_request("/tasks"));
        assert!(!looks_like_pmeverything_request("enable pmeverything"));
        assert!(!looks_like_pmlive_request("/pmeverything"));
        assert!(!looks_like_pmlive_request("view pmeverything"));
        assert!(!looks_like_pmcustom_request("/pmeverything"));
        assert!(!looks_like_pmadapters_request("/pmeverything"));
        let s = try_operator_instant_reply("/pmeverything").unwrap();
        assert!(
            s.contains("**Power everything**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view pmeverything").unwrap();
        assert!(
            viewed.contains("**Power everything**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open all-power-settings").unwrap();
        assert!(
            open.contains("**Power everything**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("pmeverything").is_none());
        assert!(try_operator_instant_reply("power everything").is_none());
        assert!(try_operator_instant_reply("everything").is_none());
        // Health lane must stay put (not Power everything).
        if let Some(reply) = try_operator_instant_reply("is everything ok") {
            assert!(
                !reply.contains("**Power everything**"),
                "is everything ok must not become Power everything: {reply}"
            );
        }
    }

    #[test]
    fn accps_slash_and_view_open_do_not_steal_pmps_battery_or_bare() {
        assert!(looks_like_accps_request("/accps"));
        assert!(looks_like_accps_request("/accps-status"));
        assert!(looks_like_accps_request("/pmaccps"));
        assert!(looks_like_accps_request("/pm-accps"));
        assert!(looks_like_accps_request("/pmset-accps"));
        assert!(looks_like_accps_request("/accessory-power"));
        assert!(looks_like_accps_request("/accessory-batteries"));
        assert!(looks_like_accps_request("/peripheral-batteries"));
        assert!(looks_like_accps_request("/bt-batteries"));
        assert!(looks_like_accps_request("/mac-accps"));
        assert!(looks_like_accps_request("/mac-accessory-power"));
        assert!(looks_like_accps_request("view accps"));
        assert!(looks_like_accps_request("open accessory-power"));
        assert!(looks_like_accps_request("list accessory batteries"));
        assert!(looks_like_accps_request("accps status"));
        assert!(looks_like_accps_request("is accps ok"));
        assert!(looks_like_accps_request("what's my accessory batteries"));
        assert!(looks_like_accps_request("view accessory power"));
        assert!(!looks_like_accps_request("accps"));
        assert!(!looks_like_accps_request("accessory power"));
        assert!(!looks_like_accps_request("accessory batteries"));
        assert!(!looks_like_accps_request("pmset accps"));
        assert!(!looks_like_accps_request("accessories"));
        assert!(!looks_like_accps_request("/pmps"));
        assert!(!looks_like_accps_request("view pmps"));
        assert!(!looks_like_accps_request("/battery"));
        assert!(!looks_like_accps_request("view battery"));
        assert!(!looks_like_accps_request("/ups"));
        assert!(!looks_like_accps_request("/adapter"));
        assert!(!looks_like_accps_request("/pmadapters"));
        assert!(!looks_like_accps_request("/pmeverything"));
        assert!(!looks_like_accps_request("/pmlive"));
        assert!(!looks_like_accps_request("/tasks"));
        assert!(!looks_like_accps_request("enable accps"));
        assert!(!looks_like_pmps_request("/accps"));
        assert!(!looks_like_pmps_request("view accps"));
        assert!(!looks_like_pmlive_request("/accps"));
        assert!(!looks_like_pmeverything_request("/accps"));
        assert!(!looks_like_pmadapters_request("/accps"));
        let s = try_operator_instant_reply("/accps").unwrap();
        assert!(
            s.contains("**Accessory power**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view accps").unwrap();
        assert!(
            viewed.contains("**Accessory power**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open accessory-batteries").unwrap();
        assert!(
            open.contains("**Accessory power**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("accps").is_none());
        assert!(try_operator_instant_reply("accessory power").is_none());
        assert!(try_operator_instant_reply("accessory batteries").is_none());
    }

    #[test]
    fn pmuuid_slash_and_view_open_do_not_steal_hardware_uuid_or_bare() {
        assert!(looks_like_pmuuid_request("/pmuuid"));
        assert!(looks_like_pmuuid_request("/pmuuid-status"));
        assert!(looks_like_pmuuid_request("/pm-uuid"));
        assert!(looks_like_pmuuid_request("/pmset-uuid"));
        assert!(looks_like_pmuuid_request("/sleep-wake-uuid"));
        assert!(looks_like_pmuuid_request("/sleepwake-uuid"));
        assert!(looks_like_pmuuid_request("/sw-uuid"));
        assert!(looks_like_pmuuid_request("/mac-pmuuid"));
        assert!(looks_like_pmuuid_request("/mac-sleep-wake-uuid"));
        assert!(looks_like_pmuuid_request("view pmuuid"));
        assert!(looks_like_pmuuid_request("open sleep-wake-uuid"));
        assert!(looks_like_pmuuid_request("list sleep wake uuid"));
        assert!(looks_like_pmuuid_request("pmuuid status"));
        assert!(looks_like_pmuuid_request("is pmuuid ok"));
        assert!(looks_like_pmuuid_request("what's my pmuuid"));
        assert!(looks_like_pmuuid_request("what's the sleep wake uuid"));
        assert!(looks_like_pmuuid_request("view sleep wake uuid"));
        assert!(!looks_like_pmuuid_request("pmuuid"));
        assert!(!looks_like_pmuuid_request("sleep wake uuid"));
        assert!(!looks_like_pmuuid_request("pmset uuid"));
        assert!(!looks_like_pmuuid_request("/uuid"));
        assert!(!looks_like_pmuuid_request("view uuid"));
        assert!(!looks_like_pmuuid_request("/hw-uuid"));
        assert!(!looks_like_pmuuid_request("view hardware uuid"));
        assert!(!looks_like_pmuuid_request("/pmlog"));
        assert!(!looks_like_pmuuid_request("/pmstats"));
        assert!(!looks_like_pmuuid_request("/assertions"));
        assert!(!looks_like_pmuuid_request("/accps"));
        assert!(!looks_like_pmuuid_request("/tasks"));
        assert!(!looks_like_pmuuid_request("enable pmuuid"));
        assert!(!looks_like_hardware_uuid_request("/pmuuid"));
        assert!(!looks_like_hardware_uuid_request("view pmuuid"));
        assert!(!looks_like_hardware_uuid_request("view sleep-wake-uuid"));
        assert!(!looks_like_hardware_uuid_request("what's my sleep wake uuid"));
        assert!(looks_like_hardware_uuid_request("/uuid"));
        let s = try_operator_instant_reply("/pmuuid").unwrap();
        assert!(
            s.contains("**Sleep/Wake UUID**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view pmuuid").unwrap();
        assert!(
            viewed.contains("**Sleep/Wake UUID**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open sleep-wake-uuid").unwrap();
        assert!(
            open.contains("**Sleep/Wake UUID**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("pmuuid").is_none());
        assert!(try_operator_instant_reply("sleep wake uuid").is_none());
        let hw = try_operator_instant_reply("/uuid").unwrap();
        assert!(
            hw.contains("**Hardware UUID**") || hw.to_lowercase().contains("uuid"),
            "hardware /uuid must stay put, got: {hw}"
        );
        assert!(!hw.contains("**Sleep/Wake UUID**"), "got: {hw}");
    }

    #[test]
    fn rawbatt_slash_and_view_open_do_not_steal_battery_cycles_or_bare() {
        assert!(looks_like_rawbatt_request("/rawbatt"));
        assert!(looks_like_rawbatt_request("/rawbatt-status"));
        assert!(looks_like_rawbatt_request("/raw-batt"));
        assert!(looks_like_rawbatt_request("/pmset-rawbatt"));
        assert!(looks_like_rawbatt_request("/raw-battery"));
        assert!(looks_like_rawbatt_request("/battery-raw"));
        assert!(looks_like_rawbatt_request("/mac-rawbatt"));
        assert!(looks_like_rawbatt_request("/mac-raw-battery"));
        assert!(looks_like_rawbatt_request("view rawbatt"));
        assert!(looks_like_rawbatt_request("open raw-battery"));
        assert!(looks_like_rawbatt_request("list raw battery"));
        assert!(looks_like_rawbatt_request("rawbatt status"));
        assert!(looks_like_rawbatt_request("is rawbatt ok"));
        assert!(looks_like_rawbatt_request("what's my rawbatt"));
        assert!(looks_like_rawbatt_request("what's the raw battery"));
        assert!(looks_like_rawbatt_request("view raw battery"));
        assert!(!looks_like_rawbatt_request("rawbatt"));
        assert!(!looks_like_rawbatt_request("raw battery"));
        assert!(!looks_like_rawbatt_request("pmset rawbatt"));
        assert!(!looks_like_rawbatt_request("/battery"));
        assert!(!looks_like_rawbatt_request("view battery"));
        assert!(!looks_like_rawbatt_request("/cycles"));
        assert!(!looks_like_rawbatt_request("battery health"));
        assert!(!looks_like_rawbatt_request("/battery-time"));
        assert!(!looks_like_rawbatt_request("/accps"));
        assert!(!looks_like_rawbatt_request("/pmps"));
        assert!(!looks_like_rawbatt_request("/pmuuid"));
        assert!(!looks_like_rawbatt_request("/tasks"));
        assert!(!looks_like_rawbatt_request("enable rawbatt"));
        assert!(!looks_like_battery_cycles_request("/rawbatt"));
        assert!(!looks_like_battery_cycles_request("view rawbatt"));
        assert!(!looks_like_accps_request("/rawbatt"));
        assert!(!looks_like_pmuuid_request("/rawbatt"));
        let s = try_operator_instant_reply("/rawbatt").unwrap();
        assert!(
            s.contains("**Raw battery**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view rawbatt").unwrap();
        assert!(
            viewed.contains("**Raw battery**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open raw-battery").unwrap();
        assert!(
            open.contains("**Raw battery**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("rawbatt").is_none());
        assert!(try_operator_instant_reply("raw battery").is_none());
        let bat = try_operator_instant_reply("/battery").unwrap();
        assert!(
            !bat.contains("**Raw battery**"),
            "Bat % /battery must stay put, got: {bat}"
        );
    }

    #[test]
    fn systemstate_slash_and_view_open_do_not_steal_pmcap_sysload_or_bare() {
        assert!(looks_like_systemstate_request("/systemstate"));
        assert!(looks_like_systemstate_request("/systemstate-status"));
        assert!(looks_like_systemstate_request("/system-state"));
        assert!(looks_like_systemstate_request("/pmset-systemstate"));
        assert!(looks_like_systemstate_request("/sysstate"));
        assert!(looks_like_systemstate_request("/system-power-state"));
        assert!(looks_like_systemstate_request("/system-capabilities"));
        assert!(looks_like_systemstate_request("/mac-systemstate"));
        assert!(looks_like_systemstate_request("/mac-sysstate"));
        assert!(looks_like_systemstate_request("view systemstate"));
        assert!(looks_like_systemstate_request("open system-state"));
        assert!(looks_like_systemstate_request("list system capabilities"));
        assert!(looks_like_systemstate_request("systemstate status"));
        assert!(looks_like_systemstate_request("is systemstate ok"));
        assert!(looks_like_systemstate_request("what's my systemstate"));
        assert!(looks_like_systemstate_request("what's the system state"));
        assert!(looks_like_systemstate_request("view system capabilities"));
        assert!(!looks_like_systemstate_request("systemstate"));
        assert!(!looks_like_systemstate_request("system state"));
        assert!(!looks_like_systemstate_request("sysstate"));
        assert!(!looks_like_systemstate_request("system capabilities"));
        assert!(!looks_like_systemstate_request("/pmcap"));
        assert!(!looks_like_systemstate_request("view power capabilities"));
        assert!(!looks_like_systemstate_request("/sysload"));
        assert!(!looks_like_systemstate_request("view sysload"));
        assert!(!looks_like_systemstate_request("/pmlive"));
        assert!(!looks_like_systemstate_request("/powerstate"));
        assert!(!looks_like_systemstate_request("/tasks"));
        assert!(!looks_like_systemstate_request("enable systemstate"));
        assert!(!looks_like_pmcap_request("/systemstate"));
        assert!(!looks_like_pmcap_request("view systemstate"));
        assert!(!looks_like_sysload_request("/systemstate"));
        assert!(!looks_like_sysload_request("view systemstate"));
        assert!(!looks_like_rawbatt_request("/systemstate"));
        let s = try_operator_instant_reply("/systemstate").unwrap();
        assert!(
            s.contains("**System state**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view systemstate").unwrap();
        assert!(
            viewed.contains("**System state**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open system-state").unwrap();
        assert!(
            open.contains("**System state**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("systemstate").is_none());
        assert!(try_operator_instant_reply("system state").is_none());
        let pmcap = try_operator_instant_reply("/pmcap").unwrap();
        assert!(
            !pmcap.contains("**System state**"),
            "Power capabilities /pmcap must stay put, got: {pmcap}"
        );
    }

    #[test]
    fn battlimit_slash_and_view_open_do_not_steal_battery_rawbatt_or_bare() {
        assert!(looks_like_battlimit_request("/battlimit"));
        assert!(looks_like_battlimit_request("/battlimit-status"));
        assert!(looks_like_battlimit_request("/batt-limit"));
        assert!(looks_like_battlimit_request("/pmset-battlimit"));
        assert!(looks_like_battlimit_request("/battery-limit"));
        assert!(looks_like_battlimit_request("/charge-limit"));
        assert!(looks_like_battlimit_request("/battery-level-limits"));
        assert!(looks_like_battlimit_request("/mac-battlimit"));
        assert!(looks_like_battlimit_request("/mac-charge-limit"));
        assert!(looks_like_battlimit_request("view battlimit"));
        assert!(looks_like_battlimit_request("open charge-limit"));
        assert!(looks_like_battlimit_request("list battery limit"));
        assert!(looks_like_battlimit_request("battlimit status"));
        assert!(looks_like_battlimit_request("is battlimit ok"));
        assert!(looks_like_battlimit_request("what's my battlimit"));
        assert!(looks_like_battlimit_request("what's the charge limit"));
        assert!(looks_like_battlimit_request("view battery level limits"));
        assert!(!looks_like_battlimit_request("battlimit"));
        assert!(!looks_like_battlimit_request("battery limit"));
        assert!(!looks_like_battlimit_request("charge limit"));
        assert!(!looks_like_battlimit_request("battery level"));
        assert!(!looks_like_battlimit_request("/battery"));
        assert!(!looks_like_battlimit_request("view battery"));
        assert!(!looks_like_battlimit_request("/cycles"));
        assert!(!looks_like_battlimit_request("/battery-time"));
        assert!(!looks_like_battlimit_request("/rawbatt"));
        assert!(!looks_like_battlimit_request("/systemstate"));
        assert!(!looks_like_battlimit_request("/tasks"));
        assert!(!looks_like_battlimit_request("enable battlimit"));
        assert!(!looks_like_rawbatt_request("/battlimit"));
        assert!(!looks_like_rawbatt_request("view battlimit"));
        assert!(!looks_like_systemstate_request("/battlimit"));
        assert!(!looks_like_battery_cycles_request("/battlimit"));
        assert!(!looks_like_battery_time_request("/battlimit"));
        let s = try_operator_instant_reply("/battlimit").unwrap();
        assert!(
            s.contains("**Battery limit**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view battlimit").unwrap();
        assert!(
            viewed.contains("**Battery limit**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open charge-limit").unwrap();
        assert!(
            open.contains("**Battery limit**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("battlimit").is_none());
        assert!(try_operator_instant_reply("battery limit").is_none());
        let bat = try_operator_instant_reply("/battery").unwrap();
        assert!(
            !bat.contains("**Battery limit**"),
            "Bat % /battery must stay put, got: {bat}"
        );
    }

    #[test]
    fn userclients_slash_and_view_open_do_not_steal_useractivity_assertions_or_bare() {
        assert!(looks_like_userclients_request("/userclients"));
        assert!(looks_like_userclients_request("/userclients-status"));
        assert!(looks_like_userclients_request("/user-clients"));
        assert!(looks_like_userclients_request("/pmset-userclients"));
        assert!(looks_like_userclients_request("/power-clients"));
        assert!(looks_like_userclients_request("/pm-clients"));
        assert!(looks_like_userclients_request("/iokit-clients"));
        assert!(looks_like_userclients_request("/mac-userclients"));
        assert!(looks_like_userclients_request("/mac-power-clients"));
        assert!(looks_like_userclients_request("view userclients"));
        assert!(looks_like_userclients_request("open power-clients"));
        assert!(looks_like_userclients_request("list user clients"));
        assert!(looks_like_userclients_request("userclients status"));
        assert!(looks_like_userclients_request("is userclients ok"));
        assert!(looks_like_userclients_request("what's my userclients"));
        assert!(looks_like_userclients_request("what's the power clients"));
        assert!(looks_like_userclients_request("view user clients"));
        assert!(!looks_like_userclients_request("userclients"));
        assert!(!looks_like_userclients_request("user clients"));
        assert!(!looks_like_userclients_request("power clients"));
        assert!(!looks_like_userclients_request("/useractivity"));
        assert!(!looks_like_userclients_request("view useractivity"));
        assert!(!looks_like_userclients_request("/assertions"));
        assert!(!looks_like_userclients_request("/who-keeps-awake"));
        assert!(!looks_like_userclients_request("/tasks"));
        assert!(!looks_like_userclients_request("/powerstate"));
        assert!(!looks_like_userclients_request("enable userclients"));
        assert!(!looks_like_useractivity_request("/userclients"));
        assert!(!looks_like_useractivity_request("view userclients"));
        assert!(!looks_like_assertions_request("/userclients"));
        assert!(!looks_like_assertions_request("view power clients"));
        let s = try_operator_instant_reply("/userclients").unwrap();
        assert!(
            s.contains("**User clients**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view userclients").unwrap();
        assert!(
            viewed.contains("**User clients**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open power-clients").unwrap();
        assert!(
            open.contains("**User clients**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("userclients").is_none());
        assert!(try_operator_instant_reply("user clients").is_none());
        let ua = try_operator_instant_reply("/useractivity").unwrap();
        assert!(
            !ua.contains("**User clients**"),
            "User activity /useractivity must stay put, got: {ua}"
        );
        let assertions = try_operator_instant_reply("/assertions").unwrap();
        assert!(
            !assertions.contains("**User clients**"),
            "Sleep assertions /assertions must stay put, got: {assertions}"
        );
    }

    #[test]
    fn powerstate_slash_and_view_open_do_not_steal_systemstate_userclients_or_bare() {
        assert!(looks_like_powerstate_request("/powerstate"));
        assert!(looks_like_powerstate_request("/powerstate-status"));
        assert!(looks_like_powerstate_request("/power-state"));
        assert!(looks_like_powerstate_request("/pmset-powerstate"));
        assert!(looks_like_powerstate_request("/iokit-drivers"));
        assert!(looks_like_powerstate_request("/iokit-powerstate"));
        assert!(looks_like_powerstate_request("/mac-powerstate"));
        assert!(looks_like_powerstate_request("/mac-iokit-drivers"));
        assert!(looks_like_powerstate_request("view powerstate"));
        assert!(looks_like_powerstate_request("open iokit-drivers"));
        assert!(looks_like_powerstate_request("list iokit drivers"));
        assert!(looks_like_powerstate_request("powerstate status"));
        assert!(looks_like_powerstate_request("is powerstate ok"));
        assert!(looks_like_powerstate_request("what's my powerstate"));
        assert!(looks_like_powerstate_request("what's the power state"));
        assert!(looks_like_powerstate_request("view power state"));
        assert!(!looks_like_powerstate_request("powerstate"));
        assert!(!looks_like_powerstate_request("power state"));
        assert!(!looks_like_powerstate_request("iokit drivers"));
        assert!(!looks_like_powerstate_request("/systemstate"));
        assert!(!looks_like_powerstate_request("view systemstate"));
        assert!(!looks_like_powerstate_request("/userclients"));
        assert!(!looks_like_powerstate_request("/assertions"));
        assert!(!looks_like_powerstate_request("/tasks"));
        assert!(!looks_like_powerstate_request("enable powerstate"));
        assert!(!looks_like_systemstate_request("/powerstate"));
        assert!(!looks_like_systemstate_request("view powerstate"));
        assert!(!looks_like_userclients_request("/powerstate"));
        assert!(!looks_like_userclients_request("view power state"));
        let s = try_operator_instant_reply("/powerstate").unwrap();
        assert!(
            s.contains("**Power state**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view powerstate").unwrap();
        assert!(
            viewed.contains("**Power state**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open iokit-drivers").unwrap();
        assert!(
            open.contains("**Power state**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("powerstate").is_none());
        assert!(try_operator_instant_reply("power state").is_none());
        let systemstate = try_operator_instant_reply("/systemstate").unwrap();
        assert!(
            !systemstate.contains("**Power state**") || systemstate.contains("**System state**"),
            "System state /systemstate must stay put, got: {systemstate}"
        );
        assert!(!systemstate.starts_with("**Power state**"));
        let uc = try_operator_instant_reply("/userclients").unwrap();
        assert!(
            !uc.contains("**Power state**"),
            "User clients /userclients must stay put, got: {uc}"
        );
    }

    #[test]
    fn pmhistory_slash_and_view_open_do_not_steal_pmlog_pmuuid_or_bare() {
        assert!(looks_like_pmhistory_request("/pmhistory"));
        assert!(looks_like_pmhistory_request("/pmhistory-status"));
        assert!(looks_like_pmhistory_request("/pm-history"));
        assert!(looks_like_pmhistory_request("/pmset-history"));
        assert!(looks_like_pmhistory_request("/power-history"));
        assert!(looks_like_pmhistory_request("/sleep-wake-history"));
        assert!(looks_like_pmhistory_request("/mac-pmhistory"));
        assert!(looks_like_pmhistory_request("/mac-power-history"));
        assert!(looks_like_pmhistory_request("view pmhistory"));
        assert!(looks_like_pmhistory_request("open power-history"));
        assert!(looks_like_pmhistory_request("list sleep wake history"));
        assert!(looks_like_pmhistory_request("pmhistory status"));
        assert!(looks_like_pmhistory_request("is pmhistory ok"));
        assert!(looks_like_pmhistory_request("what's my pmhistory"));
        assert!(looks_like_pmhistory_request("what's the power history"));
        assert!(looks_like_pmhistory_request("view sleep wake history"));
        assert!(!looks_like_pmhistory_request("pmhistory"));
        assert!(!looks_like_pmhistory_request("power history"));
        assert!(!looks_like_pmhistory_request("sleep wake history"));
        assert!(!looks_like_pmhistory_request("history"));
        assert!(!looks_like_pmhistory_request("/pmlog"));
        assert!(!looks_like_pmhistory_request("view pmlog"));
        assert!(!looks_like_pmhistory_request("/pmuuid"));
        assert!(!looks_like_pmhistory_request("/pmstats"));
        assert!(!looks_like_pmhistory_request("/userclients"));
        assert!(!looks_like_pmhistory_request("history path"));
        assert!(!looks_like_pmhistory_request("enable pmhistory"));
        assert!(!looks_like_pmlog_request("/pmhistory"));
        assert!(!looks_like_pmlog_request("view pmhistory"));
        assert!(!looks_like_pmuuid_request("/pmhistory"));
        assert!(!looks_like_userclients_request("/pmhistory"));
        let s = try_operator_instant_reply("/pmhistory").unwrap();
        assert!(
            s.contains("**Power history**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view pmhistory").unwrap();
        assert!(
            viewed.contains("**Power history**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open power-history").unwrap();
        assert!(
            open.contains("**Power history**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("pmhistory").is_none());
        assert!(try_operator_instant_reply("power history").is_none());
        let pmlog = try_operator_instant_reply("/pmlog").unwrap();
        assert!(
            !pmlog.contains("**Power history**"),
            "Power log /pmlog must stay put, got: {pmlog}"
        );
    }


    #[test]
    fn sysload_slash_and_view_open_do_not_steal_load_heat_or_bare() {
        // Slash + view/open NL → System load (`/load` · `/thermal` parity).
        // Bare `sysload` / `load level` stay with the agent;
        // `/load` stays Details; `/thermal` · `/heat` stay Heat; `/powersched` stays Power schedule.
        assert!(looks_like_sysload_request("/sysload"));
        assert!(looks_like_sysload_request("/sysload-status"));
        assert!(looks_like_sysload_request("/sys-load"));
        assert!(looks_like_sysload_request("/pmset-sysload"));
        assert!(looks_like_sysload_request("/load-level"));
        assert!(looks_like_sysload_request("/system-load-level"));
        assert!(looks_like_sysload_request("/mac-sysload"));
        assert!(looks_like_sysload_request("/mac-load-level"));
        assert!(looks_like_sysload_request("view sysload"));
        assert!(looks_like_sysload_request("open load-level"));
        assert!(looks_like_sysload_request("list system load level"));
        assert!(looks_like_sysload_request("sysload status"));
        assert!(looks_like_sysload_request("is sysload ok"));
        assert!(looks_like_sysload_request("what's my sysload"));
        assert!(looks_like_sysload_request("view load level"));
        assert!(!looks_like_sysload_request("sysload"));
        assert!(!looks_like_sysload_request("sys load"));
        assert!(!looks_like_sysload_request("load level"));
        assert!(!looks_like_sysload_request("/load"));
        assert!(!looks_like_sysload_request("view load"));
        assert!(!looks_like_sysload_request("/load-1m"));
        assert!(!looks_like_sysload_request("load 1"));
        assert!(!looks_like_sysload_request("/thermal"));
        assert!(!looks_like_sysload_request("view heat"));
        assert!(!looks_like_sysload_request("/battery"));
        assert!(!looks_like_sysload_request("/powersched"));
        assert!(!looks_like_sysload_request("/tasks"));
        assert!(!looks_like_sysload_request("enable sysload"));
        assert!(!looks_like_details_request("/sysload"));
        assert!(!looks_like_details_request("view sysload"));
        assert!(!looks_like_powersched_request("/sysload"));
        assert!(!looks_like_powersched_request("view sysload"));
        let s = try_operator_instant_reply("/sysload").unwrap();
        assert!(
            s.contains("**System load**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view sysload").unwrap();
        assert!(
            viewed.contains("**System load**")
                && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open load-level").unwrap();
        assert!(
            open.contains("**System load**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("sysload").is_none());
        assert!(try_operator_instant_reply("load level").is_none());
        let load = try_operator_instant_reply("/load");
        if let Some(reply) = load {
            assert!(
                !reply.contains("**System load**"),
                "/load must stay Details, not sysload: {reply}"
            );
        }
        let heat = try_operator_instant_reply("/thermal");
        if let Some(reply) = heat {
            assert!(
                !reply.contains("**System load**"),
                "/thermal must stay Heat, not sysload: {reply}"
            );
        }
    }

    #[test]
    fn norestoresleep_slash_and_view_open_do_not_steal_systemsleep_or_bare() {
        // Slash + view/open NL → No Restore Sleep (`/systemsleep` · `/ttysclobber` parity).
        // Bare `norestoresleep` / `no restore sleep` stay with the agent;
        // `/systemsleep` stays System Sleep; `/ttysclobber` stays TTY Clobber.
        assert!(looks_like_norestoresleep_request("/norestoresleep"));
        assert!(looks_like_norestoresleep_request("/norestoresleep-status"));
        assert!(looks_like_norestoresleep_request("/no-restore-sleep"));
        assert!(looks_like_norestoresleep_request("/norestore-sleep"));
        assert!(looks_like_norestoresleep_request("/mac-norestoresleep"));
        assert!(looks_like_norestoresleep_request("/mac-no-restore-sleep"));
        assert!(looks_like_norestoresleep_request("view norestoresleep"));
        assert!(looks_like_norestoresleep_request("open no-restore-sleep"));
        assert!(looks_like_norestoresleep_request("list no restore sleep"));
        assert!(looks_like_norestoresleep_request("norestoresleep status"));
        assert!(looks_like_norestoresleep_request("is no restore sleep on"));
        assert!(looks_like_norestoresleep_request("what's my norestoresleep"));
        assert!(looks_like_norestoresleep_request("view no restore sleep"));
        assert!(!looks_like_norestoresleep_request("norestoresleep"));
        assert!(!looks_like_norestoresleep_request("no restore sleep"));
        assert!(!looks_like_norestoresleep_request("/systemsleep"));
        assert!(!looks_like_norestoresleep_request("view system sleep"));
        assert!(!looks_like_norestoresleep_request("/ttysclobber"));
        assert!(!looks_like_norestoresleep_request("/tcpkeepalive"));
        assert!(!looks_like_norestoresleep_request("/standby"));
        assert!(!looks_like_norestoresleep_request("/tasks"));
        assert!(!looks_like_norestoresleep_request("enable norestoresleep"));
        assert!(!looks_like_systemsleep_request("/norestoresleep"));
        assert!(!looks_like_systemsleep_request("view norestoresleep"));
        assert!(!looks_like_ttysclobber_request("/norestoresleep"));
        let s = try_operator_instant_reply("/norestoresleep").unwrap();
        assert!(
            s.contains("**No Restore Sleep**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view norestoresleep").unwrap();
        assert!(
            viewed.contains("**No Restore Sleep**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open no-restore-sleep").unwrap();
        assert!(
            open.contains("**No Restore Sleep**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("norestoresleep").is_none());
        assert!(try_operator_instant_reply("no restore sleep").is_none());
        let ss = try_operator_instant_reply("/systemsleep");
        if let Some(reply) = ss {
            assert!(
                !reply.contains("**No Restore Sleep**"),
                "/systemsleep must stay System Sleep, not No Restore Sleep: {reply}"
            );
        }
        let tc = try_operator_instant_reply("/ttysclobber");
        if let Some(reply) = tc {
            assert!(
                !reply.contains("**No Restore Sleep**"),
                "/ttysclobber must stay TTY Clobber, not No Restore Sleep: {reply}"
            );
        }
    }

    #[test]
    fn autorestart_slash_and_view_open_do_not_steal_autopoweroff_or_bare() {
        // Slash + view/open NL → Auto Restart (`/lidwake` · `/autopoweroff` parity).
        // Bare `autorestart` / `auto restart` stay with the agent;
        // `/autopoweroff` stays Auto Power Off; `/lidwake` stays Lid Wake; `/acwake` stays AC Wake.
        assert!(looks_like_autorestart_request("/autorestart"));
        assert!(looks_like_autorestart_request("/autorestart-status"));
        assert!(looks_like_autorestart_request("/auto-restart"));
        assert!(looks_like_autorestart_request("/restart-on-power-loss"));
        assert!(looks_like_autorestart_request("/mac-autorestart"));
        assert!(looks_like_autorestart_request("/mac-auto-restart"));
        assert!(looks_like_autorestart_request("view autorestart"));
        assert!(looks_like_autorestart_request("open auto-restart"));
        assert!(looks_like_autorestart_request("list restart on power loss"));
        assert!(looks_like_autorestart_request("autorestart status"));
        assert!(looks_like_autorestart_request("is auto restart on"));
        assert!(looks_like_autorestart_request("what's my autorestart"));
        assert!(looks_like_autorestart_request("view auto restart"));
        assert!(!looks_like_autorestart_request("autorestart"));
        assert!(!looks_like_autorestart_request("auto restart"));
        assert!(!looks_like_autorestart_request("/autopoweroff"));
        assert!(!looks_like_autorestart_request("view autopoweroff"));
        assert!(!looks_like_autorestart_request("/lidwake"));
        assert!(!looks_like_autorestart_request("/acwake"));
        assert!(!looks_like_autorestart_request("/standby"));
        assert!(!looks_like_autorestart_request("/tasks"));
        assert!(!looks_like_autorestart_request("enable autorestart"));
        assert!(!looks_like_autorestart_request("/restart"));
        assert!(!looks_like_autopoweroff_request("/autorestart"));
        assert!(!looks_like_autopoweroff_request("view autorestart"));
        assert!(!looks_like_lidwake_request("/autorestart"));
        let s = try_operator_instant_reply("/autorestart").unwrap();
        assert!(
            s.contains("**Auto Restart**") && !s.to_lowercase().contains("execution"),
            "got: {s}"
        );
        let viewed = try_operator_instant_reply("view autorestart").unwrap();
        assert!(
            viewed.contains("**Auto Restart**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let open = try_operator_instant_reply("open auto-restart").unwrap();
        assert!(
            open.contains("**Auto Restart**") && !open.to_lowercase().contains("execution"),
            "got: {open}"
        );
        assert!(try_operator_instant_reply("autorestart").is_none());
        assert!(try_operator_instant_reply("auto restart").is_none());
        let apo = try_operator_instant_reply("/autopoweroff");
        if let Some(reply) = apo {
            assert!(
                !reply.contains("**Auto Restart**"),
                "/autopoweroff must stay Auto Power Off, not Auto Restart: {reply}"
            );
        }
        let lid = try_operator_instant_reply("/lidwake");
        if let Some(reply) = lid {
            assert!(
                !reply.contains("**Auto Restart**"),
                "/lidwake must stay Lid Wake, not Auto Restart: {reply}"
            );
        }
    }

    #[test]
    fn ac_charger_slash_and_view_open_do_not_steal_watts_or_battery() {
        // Slash + view/open NL → AC charger · adapter watts (`/cycles` · `/watts` parity).
        // Bare `charger` / `adapter` stay with the agent; `/watts` stays CPU+GPU; `/battery` · is it charging stay Bat %.
        assert!(looks_like_ac_charger_request("/charger"));
        assert!(looks_like_ac_charger_request("/ac-charger"));
        assert!(looks_like_ac_charger_request("/adapter"));
        assert!(looks_like_ac_charger_request("/power-adapter"));
        assert!(looks_like_ac_charger_request("/ac-adapter"));
        assert!(looks_like_ac_charger_request("/charger-watts"));
        assert!(looks_like_ac_charger_request("/ac-watts"));
        assert!(looks_like_ac_charger_request("/adapter-watts"));
        assert!(looks_like_ac_charger_request("view charger"));
        assert!(looks_like_ac_charger_request("open adapter"));
        assert!(looks_like_ac_charger_request("list charger watts"));
        assert!(looks_like_ac_charger_request("charger watts"));
        assert!(looks_like_ac_charger_request("ac charger"));
        assert!(looks_like_ac_charger_request("power adapter"));
        assert!(looks_like_ac_charger_request("what's my charger"));
        assert!(looks_like_ac_charger_request("what's my adapter watts"));
        assert!(!looks_like_ac_charger_request("charger"));
        assert!(!looks_like_ac_charger_request("adapter"));
        assert!(!looks_like_ac_charger_request("/watts"));
        assert!(!looks_like_ac_charger_request("how many watts"));
        assert!(!looks_like_ac_charger_request("view watts"));
        assert!(!looks_like_ac_charger_request("/battery"));
        assert!(!looks_like_ac_charger_request("view battery"));
        assert!(!looks_like_ac_charger_request("is it charging"));
        assert!(!looks_like_ac_charger_request("is it plugged in"));
        assert!(!looks_like_ac_charger_request("/cycles"));
        let charger = try_operator_instant_reply("/charger").unwrap();
        assert!(
            charger.contains("**Charger**") && !charger.to_lowercase().contains("execution"),
            "got: {charger}"
        );
        let viewed = try_operator_instant_reply("view charger").unwrap();
        assert!(
            viewed.contains("**Charger**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let watts_lane = try_operator_instant_reply("/charger-watts").unwrap();
        assert!(
            watts_lane.contains("**Charger**") && !watts_lane.to_lowercase().contains("execution"),
            "got: {watts_lane}"
        );
        assert!(try_operator_instant_reply("charger").is_none());
        let sys_watts = try_operator_instant_reply("/watts");
        if let Some(reply) = sys_watts {
            assert!(
                !reply.contains("**Charger**"),
                "/watts must stay system power draw, not charger: {reply}"
            );
        }
        let bat = try_operator_instant_reply("/battery");
        if let Some(reply) = bat {
            assert!(
                !reply.contains("**Charger**"),
                "/battery must stay Bat %, not charger: {reply}"
            );
        }
    }

#[test]
    fn display_resolution_slash_and_view_open_do_not_steal_gpu_or_screenshots() {
        // Slash + view/open NL → display names · resolutions (`/charger` · `/chip` parity).
        // Bare `display` / `resolution` / `screen` stay with the agent; `/gpu` stays GPU ring;
        // screenshot path/size/age stay screenshots; monitor phrases stay website Monitors.
        assert!(looks_like_display_resolution_request("/display"));
        assert!(looks_like_display_resolution_request("/displays"));
        assert!(looks_like_display_resolution_request("/resolution"));
        assert!(looks_like_display_resolution_request("/screen-resolution"));
        assert!(looks_like_display_resolution_request("/display-resolution"));
        assert!(looks_like_display_resolution_request("/display-res"));
        assert!(looks_like_display_resolution_request("/screen-res"));
        assert!(looks_like_display_resolution_request("/pixel-resolution"));
        assert!(looks_like_display_resolution_request("/main-display"));
        assert!(looks_like_display_resolution_request("/screen"));
        assert!(looks_like_display_resolution_request("/screens"));
        assert!(looks_like_display_resolution_request("view display"));
        assert!(looks_like_display_resolution_request("open resolution"));
        assert!(looks_like_display_resolution_request("list displays"));
        assert!(looks_like_display_resolution_request("display resolution"));
        assert!(looks_like_display_resolution_request("screen resolution"));
        assert!(looks_like_display_resolution_request("what's my resolution"));
        assert!(looks_like_display_resolution_request("what's my display"));
        assert!(looks_like_display_resolution_request("what's my screen resolution"));
        assert!(!looks_like_display_resolution_request("display"));
        assert!(!looks_like_display_resolution_request("resolution"));
        assert!(!looks_like_display_resolution_request("screen"));
        assert!(!looks_like_display_resolution_request("/gpu"));
        assert!(!looks_like_display_resolution_request("view gpu"));
        assert!(!looks_like_display_resolution_request("screenshot path"));
        assert!(!looks_like_display_resolution_request("screenshots size"));
        assert!(!looks_like_display_resolution_request("view monitors"));
        assert!(!looks_like_display_resolution_request("monitors"));
        let display = try_operator_instant_reply("/display").unwrap();
        assert!(
            display.contains("**Display**") && !display.to_lowercase().contains("execution"),
            "got: {display}"
        );
        let viewed = try_operator_instant_reply("view display").unwrap();
        assert!(
            viewed.contains("**Display**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let res_lane = try_operator_instant_reply("/resolution").unwrap();
        assert!(
            res_lane.contains("**Display**") && !res_lane.to_lowercase().contains("execution"),
            "got: {res_lane}"
        );
        assert!(try_operator_instant_reply("display").is_none());
        let gpu = try_operator_instant_reply("/gpu");
        if let Some(reply) = gpu {
            assert!(
                !reply.contains("**Display**"),
                "/gpu must stay GPU ring, not display: {reply}"
            );
        }
    }

    #[test]
    fn gpu_identity_slash_and_view_open_do_not_steal_usage_ring_or_chip() {
        // Slash + view/open NL → GPU chipset · cores · Metal (`/chip` · `/display` parity).
        // Bare `gpu` / `view gpu` / `what's the gpu` stay usage ring; `/chip` stays CPU chip.
        assert!(looks_like_gpu_identity_request("/gpu-name"));
        assert!(looks_like_gpu_identity_request("/gpu-model"));
        assert!(looks_like_gpu_identity_request("/gpu-chip"));
        assert!(looks_like_gpu_identity_request("/gpu-chipset"));
        assert!(looks_like_gpu_identity_request("/chipset"));
        assert!(looks_like_gpu_identity_request("/gpu-cores"));
        assert!(looks_like_gpu_identity_request("/metal"));
        assert!(looks_like_gpu_identity_request("/metal-support"));
        assert!(looks_like_gpu_identity_request("view gpu name"));
        assert!(looks_like_gpu_identity_request("open chipset"));
        assert!(looks_like_gpu_identity_request("list gpu cores"));
        assert!(looks_like_gpu_identity_request("gpu name"));
        assert!(looks_like_gpu_identity_request("what gpu do i have"));
        assert!(looks_like_gpu_identity_request("what's my gpu"));
        assert!(looks_like_gpu_identity_request("how many gpu cores"));
        assert!(!looks_like_gpu_identity_request("/gpu"));
        assert!(!looks_like_gpu_identity_request("gpu"));
        assert!(!looks_like_gpu_identity_request("view gpu"));
        assert!(!looks_like_gpu_identity_request("what's the gpu"));
        assert!(!looks_like_gpu_identity_request("/gpu-ring"));
        assert!(!looks_like_gpu_identity_request("/chip"));
        assert!(!looks_like_gpu_identity_request("view chip"));
        assert!(!looks_like_gpu_identity_request("/display"));
        assert!(!looks_like_gpu_identity_request("view display"));
        let id = try_operator_instant_reply("/gpu-name").unwrap();
        assert!(
            id.contains("**GPU**") && id.contains('`') && !id.to_lowercase().contains("execution"),
            "got: {id}"
        );
        let viewed = try_operator_instant_reply("view gpu name").unwrap();
        assert!(
            viewed.contains("**GPU**") && !viewed.to_lowercase().contains("execution"),
            "got: {viewed}"
        );
        let chipset = try_operator_instant_reply("/chipset").unwrap();
        assert!(
            chipset.contains("**GPU**") && !chipset.to_lowercase().contains("execution"),
            "got: {chipset}"
        );
        let usage = try_operator_instant_reply("/gpu").unwrap();
        assert!(
            usage.contains("**GPU**") && !usage.contains('`'),
            "/gpu must stay usage ring (no identity backticks): {usage}"
        );
        let chip = try_operator_instant_reply("/chip").unwrap();
        assert!(
            chip.contains("**Chip**") && !chip.contains("**GPU**"),
            "/chip must stay CPU chip: {chip}"
        );
    }

#[test]
    fn disk_capacity_slash_and_view_open_do_not_steal_ssd_percent_or_cleanup() {
        // Slash + view/open NL → capacity (RAM `/ram-size` · SSD `/free` parity).
        // Bare /ssd · /storage · view ssd · how much storage stay %; view disk stays Cleanup; /free stays free.
        assert!(looks_like_disk_capacity_request("/ssd-size"));
        assert!(looks_like_disk_capacity_request("/disk-size"));
        assert!(looks_like_disk_capacity_request("/storage-size"));
        assert!(looks_like_disk_capacity_request("/ssd-capacity"));
        assert!(looks_like_disk_capacity_request("/total-ssd"));
        assert!(looks_like_disk_capacity_request("view ssd size"));
        assert!(looks_like_disk_capacity_request("open disk capacity"));
        assert!(looks_like_disk_capacity_request("list storage size"));
        assert!(looks_like_disk_capacity_request("how big is the ssd"));
        assert!(looks_like_disk_capacity_request("ssd size"));
        assert!(!looks_like_disk_capacity_request("/ssd"));
        assert!(!looks_like_disk_capacity_request("/storage"));
        assert!(!looks_like_disk_capacity_request("view ssd"));
        assert!(!looks_like_disk_capacity_request("how much storage"));
        assert!(!looks_like_disk_capacity_request("view disk"));
        assert!(!looks_like_disk_capacity_request("/free"));
        assert!(!looks_like_disk_capacity_request("how much free space"));
        assert!(!looks_like_disk_capacity_request("disk cleanup size"));
        assert_eq!(parse_strip_chip_ask("/ssd"), Some(StripChipAsk::Ssd));
        assert_eq!(parse_strip_chip_ask("view storage"), Some(StripChipAsk::Ssd));
        let cap = try_operator_instant_reply("/ssd-size").unwrap();
        assert!(
            cap.contains("**SSD**")
                && cap.to_lowercase().contains("capacity")
                && !cap.to_lowercase().contains("execution"),
            "got: {cap}"
        );
        let viewed = try_operator_instant_reply("view ssd size").unwrap();
        assert!(
            viewed.contains("**SSD**") && viewed.to_lowercase().contains("capacity"),
            "got: {viewed}"
        );
        let pct = try_operator_instant_reply("/ssd").unwrap();
        assert!(
            pct.contains("**SSD**") && !pct.to_lowercase().contains("capacity"),
            "bare /ssd must stay % chip, not capacity: {pct}"
        );
        let free = try_operator_instant_reply("/free").unwrap();
        assert!(
            free.contains("**SSD**") && free.to_lowercase().contains("free"),
            "got: {free}"
        );
    }

    #[test]
    fn ori_vault_size_slash_does_not_steal_ori_ready_or_other_sizes() {
        // Broad ori_vault nouns (bare `ori` / `size`) used to make `/ori` and `config size`
        // hit vault-size before Ready / config lanes. Slash + size NL stay vault bytes.
        assert!(looks_like_ori_vault_size_request("/ori-vault-size"));
        assert!(looks_like_ori_vault_size_request("/vault-size"));
        assert!(looks_like_ori_vault_size_request("/mnemos-vault-size"));
        assert!(looks_like_ori_vault_size_request("ori vault size"));
        assert!(looks_like_ori_vault_size_request("how big is ori vault"));
        assert!(looks_like_ori_vault_size_request("view ori vault size"));
        assert!(looks_like_ori_vault_size_request("open ori vault size"));
        assert!(!looks_like_ori_vault_size_request("/ori"));
        assert!(!looks_like_ori_vault_size_request("/mnemos"));
        assert!(!looks_like_ori_vault_size_request("ori"));
        assert!(!looks_like_ori_vault_size_request("ori vault"));
        assert!(!looks_like_ori_vault_size_request("view ori"));
        assert!(!looks_like_ori_vault_size_request("config size"));
        assert!(!looks_like_ori_vault_size_request("overnight agent log size"));
        assert!(looks_like_ori_ready_request("/ori"));
        assert!(looks_like_ori_ready_request("view ori"));
        assert!(looks_like_ori_ready_request("ori vault"));
        assert!(!looks_like_ori_ready_request("/ori-vault-size"));
        assert!(!looks_like_ori_ready_request("ori vault size"));
        let ready = try_operator_instant_reply("/ori").unwrap();
        assert!(
            ready.contains("**Ori**") && !ready.to_lowercase().contains("on disk"),
            "bare /ori must stay Ready chip, not vault size: {ready}"
        );
        let sized = try_operator_instant_reply("/ori-vault-size").unwrap();
        assert!(
            sized.contains("**Ori vault:**"),
            "got: {sized}"
        );
        let viewed = try_operator_instant_reply("view ori vault size").unwrap();
        assert!(
            viewed.contains("**Ori vault:**"),
            "got: {viewed}"
        );
        let cfg = try_operator_instant_reply("config size").unwrap();
        assert!(
            cfg.to_lowercase().contains("config") && !cfg.contains("**Ori vault:"),
            "config size must stay config lane, not ori/execution: {cfg}"
        );
        let exec = try_operator_instant_reply("execution prompt size").unwrap();
        assert!(
            exec.contains("**Execution prompt:**"),
            "execution prompt size must still resolve: {exec}"
        );
    }

    #[test]
    fn disk_free_slash_and_view_open_do_not_steal_ssd_percent_or_cleanup() {
        // Slash + view/open NL → free bytes (SSD `/storage` · watts `/watts` parity).
        // Bare free space / disk free stay SSD %; view disk stays Disk Cleanup.
        assert!(looks_like_disk_free_request("/free"));
        assert!(looks_like_disk_free_request("/ssd-free"));
        assert!(looks_like_disk_free_request("/disk-free"));
        assert!(looks_like_disk_free_request("/free-space"));
        assert!(looks_like_disk_free_request("view free space"));
        assert!(looks_like_disk_free_request("open free space"));
        assert!(looks_like_disk_free_request("list ssd free"));
        assert!(looks_like_disk_free_request("how much free space"));
        assert!(!looks_like_disk_free_request("free space"));
        assert!(!looks_like_disk_free_request("disk free"));
        assert!(!looks_like_disk_free_request("view disk"));
        assert!(!looks_like_disk_free_request("/ssd"));
        assert!(!looks_like_disk_free_request("/storage"));
        assert!(!looks_like_disk_free_request("how much disk is used"));
        assert_eq!(parse_strip_chip_ask("free space"), Some(StripChipAsk::Ssd));
        assert_eq!(parse_strip_chip_ask("disk free"), Some(StripChipAsk::Ssd));
        assert!(looks_like_disk_cleanup_request("view disk"));
        let free = try_operator_instant_reply("/free").unwrap();
        assert!(
            free.contains("**SSD**") && free.to_lowercase().contains("free"),
            "got: {free}"
        );
        let viewed = try_operator_instant_reply("view free space").unwrap();
        assert!(
            viewed.contains("**SSD**") && viewed.to_lowercase().contains("free"),
            "got: {viewed}"
        );
        let pct = try_operator_instant_reply("free space").unwrap();
        assert!(pct.contains("**SSD**"), "got: {pct}");
        // Percent chip is short; free-bytes line mentions "total".
        assert!(
            !pct.to_lowercase().contains(" total"),
            "bare free space must stay % chip, not free bytes: {pct}"
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

    #[test]
    fn runs_activity_patterns_busiest_and_streak() {
        use chrono::{TimeZone, Utc};
        // Mon 10:00, Tue 10:00, Tue 15:00, Thu 15:00 (UTC; local may shift hour — day/streak still hold).
        let tss = vec![
            Utc.with_ymd_and_hms(2026, 11, 23, 10, 0, 0).unwrap(), // Mon
            Utc.with_ymd_and_hms(2026, 11, 24, 10, 0, 0).unwrap(), // Tue
            Utc.with_ymd_and_hms(2026, 11, 24, 15, 0, 0).unwrap(), // Tue
            Utc.with_ymd_and_hms(2026, 11, 26, 15, 0, 0).unwrap(), // Thu
        ];
        let act = compute_runs_activity_patterns(&tss);
        assert_eq!(act.busiest_day.as_deref(), Some("Tue"));
        assert_eq!(act.busiest_day_count, 2);
        assert_eq!(act.active_days, 3);
        assert_eq!(act.max_streak, 2); // Mon–Tue consecutive
        assert!(act.busiest_hour.is_some());
        assert!(act.busiest_hour_count >= 1);
        assert_eq!(format_busiest_hour_label(0), "12AM");
        assert_eq!(format_busiest_hour_label(15), "3PM");
        assert_eq!(format_busiest_hour_label(12), "12PM");

        let mut insights = compute_runs_insights(5);
        insights.turns = 4;
        insights.busiest_day = act.busiest_day.clone();
        insights.busiest_day_count = act.busiest_day_count;
        insights.busiest_hour = act.busiest_hour;
        insights.busiest_hour_count = act.busiest_hour_count;
        insights.active_days = act.active_days;
        insights.max_streak = act.max_streak;
        let text = format_runs_insights_gateway(&insights);
        assert!(text.contains("Busiest:"), "got: {text}");
        assert!(text.contains("**Tues**") || text.contains("Tue"), "got: {text}");
    }

    #[test]
    fn format_tool_share_hermes_pct() {
        assert_eq!(format_tool_share("RUN_CMD", 3, 10), "RUN_CMD×3 (30.0%)");
        assert_eq!(format_tool_share("FETCH_URL", 1, 3), "FETCH_URL×1 (33.3%)");
        assert_eq!(format_tool_share("none", 0, 0), "none×0 (0.0%)");
        let mut insights = compute_runs_insights(1);
        insights.turns = 2;
        insights.by_tool = vec![("RUN_CMD".into(), 3), ("FETCH_URL".into(), 1)];
        let text = format_runs_insights_gateway(&insights);
        assert!(text.contains("RUN_CMD×3 (75.0%)"), "got: {text}");
        assert!(text.contains("FETCH_URL×1 (25.0%)"), "got: {text}");
    }

    #[test]
    fn format_routing_insights_hermes_overview() {
        let mut insights = compute_runs_insights(1);
        insights.turns = 10;
        insights.pre_routed_count = 4;
        insights.avg_tool_steps = 1.5;
        insights.verify_scored = 6;
        insights.verify_pass_count = 5;
        let line = format_routing_insights_line(&insights);
        assert!(
            line.contains("pre-routed **4** (40.0%)"),
            "got: {line}"
        );
        assert!(line.contains("avg tools **1.5**"), "got: {line}");
        assert!(line.contains("verify **5/6**"), "got: {line}");
        let text = format_runs_insights_gateway(&insights);
        assert!(text.contains("Routing:"), "got: {text}");
        assert!(text.contains("pre-routed **4** (40.0%)"), "got: {text}");
    }

    #[test]
    fn format_notable_insights_hermes_most_tools() {
        let mut insights = compute_runs_insights(1);
        insights.turns = 3;
        insights.most_tools_steps = 15;
        insights.most_tools_lane = "direct".into();
        insights.most_tools_wall_ms = 35586;
        insights.most_tools_preview = "Current task file content".into();
        let line = format_notable_insights_line(&insights);
        assert!(line.contains("most tools **15**"), "got: {line}");
        assert!(line.contains("direct"), "got: {line}");
        assert!(line.contains("35.6s"), "got: {line}");
        assert!(line.contains("Current task file content"), "got: {line}");
        let text = format_runs_insights_gateway(&insights);
        assert!(text.contains("Notable:"), "got: {text}");
        assert!(text.contains("most tools **15**"), "got: {text}");
        insights.most_tools_steps = 0;
        let empty = format_notable_insights_line(&insights);
        assert!(empty.contains("no tools yet"), "got: {empty}");
    }

    #[test]
    fn format_longest_insights_hermes_notable() {
        let mut insights = compute_runs_insights(1);
        insights.turns = 4;
        insights.longest_wall_ms = 125_000;
        insights.longest_wall_lane = "direct".into();
        insights.longest_wall_preview = "Overnight harness tick".into();
        let line = format_longest_insights_line(&insights);
        assert!(line.contains("Longest: **2m**"), "got: {line}");
        assert!(line.contains("direct"), "got: {line}");
        assert!(line.contains("Overnight harness tick"), "got: {line}");
        let text = format_runs_insights_gateway(&insights);
        assert!(text.contains("Longest: **2m**"), "got: {text}");
        insights.longest_wall_ms = 0;
        let empty = format_longest_insights_line(&insights);
        assert!(empty.contains("no time yet"), "got: {empty}");
    }

    #[test]
    fn format_most_reply_insights_hermes_notable() {
        let mut insights = compute_runs_insights(1);
        insights.turns = 5;
        insights.most_reply_chars = 1847;
        insights.most_reply_lane = "direct".into();
        insights.most_reply_wall_ms = 12_500;
        insights.most_reply_preview = "Summarize overnight keeps".into();
        let line = format_most_reply_insights_line(&insights);
        assert!(line.contains("Most reply: **1847** chars"), "got: {line}");
        assert!(line.contains("direct"), "got: {line}");
        assert!(line.contains("12s") || line.contains("13s"), "got: {line}");
        assert!(line.contains("Summarize overnight keeps"), "got: {line}");
        let text = format_runs_insights_gateway(&insights);
        assert!(text.contains("Most reply: **1847** chars"), "got: {text}");
        insights.most_reply_chars = 0;
        let empty = format_most_reply_insights_line(&insights);
        assert!(empty.contains("no replies yet"), "got: {empty}");
    }

    #[test]
    fn format_llm_skips_insights_capex_glance() {
        let mut insights = compute_runs_insights(1);
        insights.turns = 10;
        insights.skipped_plan_count = 8;
        insights.skipped_verify_count = 7;
        insights.skipped_criteria_count = 9;
        insights.skipped_topic_count = 6;
        let line = format_llm_skips_insights_line(&insights);
        assert!(line.contains("plan **8** (80%)"), "got: {line}");
        assert!(line.contains("verify **7** (70%)"), "got: {line}");
        assert!(line.contains("criteria **9** (90%)"), "got: {line}");
        assert!(line.contains("topic **6** (60%)"), "got: {line}");
        let text = format_runs_insights_gateway(&insights);
        assert!(text.contains("LLM skips:"), "got: {text}");
        assert!(text.contains("plan **8** (80%)"), "got: {text}");
    }

    #[test]
    fn format_replies_insights_hermes_overview() {
        let mut insights = compute_runs_insights(1);
        insights.turns = 5;
        insights.reply_chars_mean = 328;
        insights.reply_chars_p50 = 279;
        insights.reply_chars_max = 1847;
        let line = format_replies_insights_line(&insights);
        assert!(line.contains("mean **328**"), "got: {line}");
        assert!(line.contains("p50 **279**"), "got: {line}");
        assert!(line.contains("max **1847**"), "got: {line}");
        let text = format_runs_insights_gateway(&insights);
        assert!(text.contains("Replies:"), "got: {text}");
        assert!(text.contains("mean **328**"), "got: {text}");
        insights.reply_chars_mean = 0;
        insights.reply_chars_p50 = 0;
        insights.reply_chars_max = 0;
        let empty = format_replies_insights_line(&insights);
        assert!(empty.contains("no replies yet"), "got: {empty}");
    }

    #[test]
    fn format_active_time_insights_hermes_overview() {
        let mut insights = compute_runs_insights(1);
        insights.turns = 4;
        insights.active_wall_ms = 125_000;
        insights.avg_wall_ms = 31_250;
        let line = format_active_time_insights_line(&insights);
        assert!(line.contains("Active: **2m**"), "got: {line}");
        assert!(line.contains("avg turn **31s**"), "got: {line}");
        let text = format_runs_insights_gateway(&insights);
        assert!(text.contains("Active:"), "got: {text}");
        assert!(text.contains("**2m**"), "got: {text}");
        insights.active_wall_ms = 0;
        insights.avg_wall_ms = 0;
        let empty = format_active_time_insights_line(&insights);
        assert!(empty.contains("no time yet"), "got: {empty}");
        assert_eq!(format_wall_duration_ms(450), "450ms");
        assert_eq!(format_wall_duration_ms(3200), "3.2s");
    }

    #[test]
    fn format_period_insights_hermes_overview() {
        let mut insights = compute_runs_insights(1);
        insights.turns = 4;
        insights.period_start = Some("Jul 20".into());
        insights.period_end = Some("Nov 26".into());
        let line = format_period_insights_line(&insights);
        assert!(line.contains("Period: **Jul 20** — **Nov 26**"), "got: {line}");
        let text = format_runs_insights_gateway(&insights);
        assert!(text.contains("Period:"), "got: {text}");
        assert!(text.contains("Jul 20"), "got: {text}");
        insights.period_start = Some("Nov 26".into());
        insights.period_end = Some("Nov 26".into());
        let same = format_period_insights_line(&insights);
        assert_eq!(same, "Period: **Nov 26**");
        insights.period_start = None;
        insights.period_end = None;
        let empty = format_period_insights_line(&insights);
        assert!(empty.contains("no dates yet"), "got: {empty}");
    }

    #[test]
    fn format_channels_insights_hermes_platforms() {
        assert_eq!(format_channel_short("1467450744864243725"), "…3725");
        assert_eq!(format_channel_short("abc"), "abc");
        let mut insights = compute_runs_insights(1);
        insights.turns = 100;
        insights.channel_count = 3;
        insights.busiest_channel = Some("…3725".into());
        insights.busiest_channel_count = 80;
        let line = format_channels_insights_line(&insights);
        assert!(line.contains("Channels: **3**"), "got: {line}");
        assert!(line.contains("busiest **…3725**×80 (80%)"), "got: {line}");
        let text = format_runs_insights_gateway(&insights);
        assert!(text.contains("Channels:"), "got: {text}");
        assert!(text.contains("…3725"), "got: {text}");
        insights.channel_count = 0;
        insights.busiest_channel = None;
        insights.busiest_channel_count = 0;
        let empty = format_channels_insights_line(&insights);
        assert!(empty.contains("no channels yet"), "got: {empty}");
    }

    #[test]
    fn format_tool_calls_insights_hermes_overview() {
        let mut insights = compute_runs_insights(1);
        insights.turns = 12;
        insights.total_tool_calls = 47;
        let line = format_tool_calls_insights_line(&insights);
        assert!(line.contains("Tool calls: **47**"), "got: {line}");
        let text = format_runs_insights_gateway(&insights);
        assert!(text.contains("Tool calls: **47**"), "got: {text}");
        assert!(text.contains("tools **47**"), "got: {text}");
        insights.total_tool_calls = 0;
        let empty = format_tool_calls_insights_line(&insights);
        assert!(empty.contains("none yet"), "got: {empty}");
    }
}
