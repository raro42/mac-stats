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
}
