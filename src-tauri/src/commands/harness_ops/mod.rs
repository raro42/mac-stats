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
}
