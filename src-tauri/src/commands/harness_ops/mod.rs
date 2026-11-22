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
}
