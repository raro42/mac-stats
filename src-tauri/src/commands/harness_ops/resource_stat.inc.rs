// included by gateway.rs — resource_stat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatKind {
    Path,
    Size,
    Age,
}

#[derive(Debug, Clone)]
pub struct ResourceLane {
    pub id: &'static str,
    pub nouns: &'static [&'static str],
    pub max_len: usize,
    pub path_phrases: &'static [&'static str],
    pub size_phrases: &'static [&'static str],
    pub age_phrases: &'static [&'static str],
}

pub fn looks_like_resource_stat(content: &str, lane: &ResourceLane, kind: StatKind) -> bool {
    let n = normalize_operator_command(content);
    if n.chars().count() > lane.max_len {
        return false;
    }
    if !lane.nouns.iter().any(|noun| n.contains(noun)) {
        // Exact phrase lists may already include the noun.
        let phrases = match kind {
            StatKind::Path => lane.path_phrases,
            StatKind::Size => lane.size_phrases,
            StatKind::Age => lane.age_phrases,
        };
        if !phrases.iter().any(|p| *p == n) {
            return false;
        }
    }
    match kind {
        StatKind::Path => {
            if is_size_ask(&n) || is_age_ask(&n) {
                return false;
            }
            if n.contains("path")
                || n.contains("where")
                || n.contains("location")
                || n.contains("file is")
            {
                return true;
            }
            lane.path_phrases.iter().any(|p| *p == n)
        }
        StatKind::Size => {
            if n.contains("path")
                || n.contains("where")
                || n.contains("location")
                || is_age_ask(&n)
            {
                return false;
            }
            if is_size_ask(&n) {
                return true;
            }
            lane.size_phrases.iter().any(|p| *p == n)
        }
        StatKind::Age => {
            if n.contains("path")
                || n.contains("where")
                || n.contains("location")
                || is_size_ask(&n)
            {
                return false;
            }
            if is_age_ask(&n) {
                return true;
            }
            lane.age_phrases.iter().any(|p| *p == n)
        }
    }
}

pub(super) fn is_size_ask(n: &str) -> bool {
    n.contains("size")
        || n.contains("how big")
        || n.contains("bytes")
        || n.contains("disk use")
        || n.contains("on disk")
}

pub(super) fn is_age_ask(n: &str) -> bool {
    contains_age_token(n)
        || n.contains("how old")
        || n.contains("last write")
        || n.contains("mtime")
        || n.contains("modified")
}

pub static RESOURCE_LANES: &[ResourceLane] = &[
    ResourceLane {
        id: "agent_json",
        nouns: &["agent json", "agent-json", "agent_json", "json", "agent.json", "agent config", "agent-config", "agent", "path", "where", "location", "file"],
        max_len: 80,
        path_phrases: &["agent.json", "agent.json path", "agent.json file", "agent.json file path", "agent json path", "agent json file"],
        size_phrases: &["agent.json size", "agent json size", "agent.json file size", "agent json file size", "agent config size", "agent-config size"],
        age_phrases: &["agent.json age", "agent json age", "agent.json file age", "agent json file age", "agent config age", "agent-config age"],
    },
    ResourceLane {
        id: "agents",
        nouns: &["agents", "agents folder", "agent folder", "agents directory", "agent directory", "agents dir", "agent dir", "agents size", "agent size", "mac-stats agents", "mac stats agents", "agent"],
        max_len: 72,
        path_phrases: &["agents path", "agent path", "agents folder", "agent folder", "agents directory", "agent directory"],
        size_phrases: &["agents size", "agent size", "agents folder size", "agent folder size", "agents directory size", "agent directory size"],
        age_phrases: &["agents age", "agent age", "agents folder age", "agent folder age", "agents directory age", "agent directory age"],
    },
    ResourceLane {
        id: "before_compaction_transcript",
        nouns: &["before compaction transcript", "before-compaction-transcript", "before_compaction_transcript", "transcript", "last_session_before_compaction", "last-session-before-compaction", "beforecompactiontranscript", "before-compaction transcript", "before_compaction transcript", "before compaction jsonl", "before-compaction jsonl", "before_compaction jsonl"],
        max_len: 96,
        path_phrases: &["before compaction transcript path", "before-compaction transcript path", "before_compaction transcript path", "before compaction transcript", "before-compaction transcript", "before_compaction transcript"],
        size_phrases: &["before compaction transcript size", "before-compaction transcript size", "before_compaction transcript size", "before compaction transcript file size", "before-compaction transcript file size", "before_compaction transcript file size"],
        age_phrases: &["before compaction transcript age", "before-compaction transcript age", "before_compaction transcript age", "before compaction transcript file age", "before-compaction transcript file age", "before_compaction transcript file age"],
    },
    ResourceLane {
        id: "before_reset_transcript",
        nouns: &["before reset transcript", "before-reset-transcript", "before_reset_transcript", "transcript", "last_session_before_reset", "last-session-before-reset", "beforeresettranscript", "before-reset transcript", "before_reset transcript", "before reset jsonl", "before-reset jsonl", "before_reset jsonl"],
        max_len: 96,
        path_phrases: &["before reset transcript path", "before-reset transcript path", "before_reset transcript path", "before reset transcript", "before-reset transcript", "before_reset transcript"],
        size_phrases: &["before reset transcript size", "before-reset transcript size", "before_reset transcript size", "before reset transcript file size", "before-reset transcript file size", "before_reset transcript file size"],
        age_phrases: &["before reset transcript age", "before-reset transcript age", "before_reset transcript age", "before reset transcript file age", "before-reset transcript file age", "before_reset transcript file age"],
    },
    ResourceLane {
        id: "browser_credentials",
        nouns: &["browser credentials", "browser-credentials", "browser_credentials", "credentials", "browser credential", "cdp credentials", "cdp credential", "browser secrets", "browser secret", "mac-stats credentials", "mac stats credentials", "credentials.toml"],
        max_len: 88,
        path_phrases: &["browser credentials path", "browser credential path", "browser credentials", "browser credential", "browser credentials file", "browser credential file"],
        size_phrases: &["browser credentials size", "browser credential size", "browser-credentials size", "browser-credentials.toml size", "browser credentials file size", "browser credential file size"],
        age_phrases: &["browser credentials age", "browser credential age", "browser-credentials age", "browser-credentials.toml age", "browser credentials file age", "browser credential file age"],
    },
    ResourceLane {
        id: "browser_downloads",
        nouns: &["browser downloads", "browser-downloads", "browser_downloads", "downloads", "browser download", "browser_download", "cdp downloads", "cdp download", "mac-stats downloads", "mac stats downloads", "browser", "cdp"],
        max_len: 80,
        path_phrases: &["browser downloads path", "browser download path", "browser downloads", "browser download folder", "browser downloads folder", "browser downloads directory"],
        size_phrases: &["browser downloads size", "browser download size", "browser-downloads size", "browser_downloads size", "browser downloads folder size", "browser download folder size"],
        age_phrases: &["browser downloads age", "browser download age", "browser-downloads age", "browser_downloads age", "browser downloads folder age", "browser download folder age"],
    },
    ResourceLane {
        id: "browser_storage_state",
        nouns: &["browser storage state", "browser-storage-state", "browser_storage_state", "state", "storage state", "storage_state", "cookie jar", "cookies jar", "cdp cookies", "cdp cookie", "browser cookies", "browser cookie"],
        max_len: 88,
        path_phrases: &["storage state path", "storage state", "storage_state", "storage_state path", "storage_state.json", "browser storage state"],
        size_phrases: &["storage state size", "storage_state size", "storage_state.json size", "browser storage state size", "browser_storage_state size", "browser_storage_state.json size"],
        age_phrases: &["storage state age", "storage_state age", "storage_state.json age", "browser storage state age", "browser_storage_state age", "browser_storage_state.json age"],
    },
    ResourceLane {
        id: "cleanup_quarantine",
        nouns: &["cleanup quarantine", "cleanup-quarantine", "cleanup_quarantine", "quarantine", "clean-up quarantine", "disk quarantine", "soft-delete quarantine", "soft delete quarantine", "mac-stats quarantine", "mac stats quarantine", "how old", "stale"],
        max_len: 80,
        path_phrases: &["cleanup quarantine path", "cleanup-quarantine", "cleanup-quarantine path", "cleanup-quarantine folder", "cleanup-quarantine directory", "cleanup-quarantine dir"],
        size_phrases: &["cleanup quarantine size", "cleanup-quarantine size", "cleanup quarantine folder size", "cleanup-quarantine folder size", "cleanup quarantine directory size", "cleanup-quarantine directory size"],
        age_phrases: &["cleanup quarantine age", "cleanup-quarantine age", "cleanup quarantine folder age", "cleanup-quarantine folder age", "cleanup quarantine directory age", "cleanup-quarantine directory age"],
    },
    ResourceLane {
        id: "config",
        nouns: &["config", "config.json", "mac-stats home", "mac stats home", "data home", "data dir", "data directory", "mac-stats data", "mac stats data", "mac-stats folder", "mac stats folder", ".mac-stats"],
        max_len: 72,
        path_phrases: &["where is config", "where is the config", "where is config.json", "where is the config.json", "where is mac-stats config", "where is mac stats config"],
        size_phrases: &["config size", "config file size", "config.json size", "config json size", "mac-stats config size", "mac stats config size"],
        age_phrases: &["config age", "config file age", "config.json age", "config json age", "mac-stats config age", "mac stats config age"],
    },
    ResourceLane {
        id: "config_env",
        nouns: &["config env", "config-env", "config_env", "env", ".config.env", "config.env", ".env.config", "env.config", "secrets env", "secrets file", "config.env size", "config env size"],
        max_len: 88,
        path_phrases: &[".config.env", "config.env", "config.env path", "config.env file", "config env", "config env path"],
        size_phrases: &["config.env size", ".config.env size", "config env size", "config.env file size", ".config.env file size", "config env file size"],
        age_phrases: &["config.env age", ".config.env age", "config env age", "config.env file age", ".config.env file age", "config env file age"],
    },
    ResourceLane {
        id: "cookie_reject_patterns",
        nouns: &["cookie reject patterns", "cookie-reject-patterns", "cookie_reject_patterns", "patterns", "cookie_reject_patterns.md", "cookie reject pattern", "cookie_reject", "cookie-reject", "cookie reject", "reject patterns", "reject pattern", "cookie reject size"],
        max_len: 88,
        path_phrases: &["cookie reject patterns path", "cookie reject pattern path", "cookie_reject_patterns", "cookie_reject_patterns.md", "cookie_reject_patterns.md path", "cookie_reject_patterns path"],
        size_phrases: &["cookie reject size", "cookie reject patterns size", "cookie reject pattern size", "cookie_reject_patterns size", "cookie_reject_patterns.md size", "cookie_reject_patterns file size"],
        age_phrases: &["cookie reject age", "cookie reject patterns age", "cookie reject pattern age", "cookie_reject_patterns age", "cookie_reject_patterns.md age", "cookie_reject_patterns file age"],
    },
    ResourceLane {
        id: "credential_accounts",
        nouns: &["credential accounts", "credential-accounts", "credential_accounts", "accounts", "credential_accounts.json", "credentials accounts", "keychain accounts", "keychain account list", "accounts.json", "credential accounts size", "credential_accounts size", "keychain accounts size"],
        max_len: 88,
        path_phrases: &["credential accounts path", "credential account path", "credential_accounts", "credential_accounts.json", "credential_accounts.json path", "credential_accounts path"],
        size_phrases: &["credential accounts size", "credential account size", "credential_accounts size", "credential_accounts.json size", "credential_accounts file size", "credential-accounts size"],
        age_phrases: &["credential accounts age", "credential account age", "credential_accounts age", "credential_accounts.json age", "credential_accounts file age", "credential-accounts age"],
    },
    ResourceLane {
        id: "debug_log",
        nouns: &["debug log", "debug-log", "debug_log", "log", "debug.log", "debug"],
        max_len: 72,
        path_phrases: &["where is the log", "where is debug log", "where is the debug log", "where is debug.log", "log file path", "debug log path"],
        size_phrases: &["how big is the log", "how big is debug log", "how big is the debug log", "how large is the log", "how large is debug log", "log file size"],
        age_phrases: &["log age", "debug log age", "how old is the log", "how old is debug log", "how old is the debug log", "when was the log updated"],
    },
    ResourceLane {
        id: "digest",
        nouns: &["digest", "digester", "latest.json", "latest json"],
        max_len: 72,
        path_phrases: &[],
        size_phrases: &["digest size", "digest file size", "digester size", "latest.json size", "latest json size", "how big is the digest"],
        age_phrases: &["digest age", "digest stale", "how old is the digest", "how old is digest", "when was the digest updated", "when was digest updated"],
    },
    ResourceLane {
        id: "digest_md",
        nouns: &["digest md", "digest-md", "digest_md", "md", "latest.md", "digest.md", "latest md", "digest markdown", "md digest"],
        max_len: 72,
        path_phrases: &[],
        size_phrases: &["digest.md size", "digest md size", "latest.md size", "latest md size", "how big is digest.md", "how big is latest.md"],
        age_phrases: &["digest.md age", "digest md age", "latest.md age", "latest md age", "how old is digest.md", "how old is latest.md"],
    },
    ResourceLane {
        id: "discord_channels",
        nouns: &["discord channels", "discord-channels", "discord_channels", "channels", "discord_channels.json", "discord-channels.json", "discord channels.json", "discord channels file", "discord channel file", "discord channels config", "discord channel config", "channels.json"],
        max_len: 72,
        path_phrases: &["discord channels path", "discord_channels", "discord_channels.json", "discord_channels.json path", "discord_channels path", "discord_channels file"],
        size_phrases: &["discord channels size", "discord channel size", "discord_channels.json size", "discord_channels size", "discord_channels file size", "discord-channels size"],
        age_phrases: &["discord channels age", "discord channel age", "discord_channels.json age", "discord_channels age", "discord_channels file age", "discord-channels age"],
    },
    ResourceLane {
        id: "discord_memory",
        nouns: &["discord memory", "discord-memory", "discord_memory", "memory", "memory-discord", "memory_discord", "memorydiscord", "discord memories", "channel memory", "channel memories", "discord channel memory", "discord-channel-memory"],
        max_len: 88,
        path_phrases: &["discord memory", "discord memories", "channel memory", "channel memories", "/knowledge", "/knowledge discord"],
        size_phrases: &["discord memory", "discord memories", "channel memory", "channel memories", "/knowledge", "/knowledge discord"],
        age_phrases: &["discord memory", "discord memories", "channel memory", "channel memories", "/knowledge", "/knowledge discord"],
    },
    ResourceLane {
        id: "disk_cleanup",
        nouns: &["disk cleanup", "disk-cleanup", "disk_cleanup", "cleanup", "disk_cleanup.json", "disk-cleanup.json", "disk cleanup.json", "disk cleanup file", "cleanup file", "cleanup config", "disk cleanup config", "old"],
        max_len: 72,
        path_phrases: &["disk cleanup path", "disk_cleanup", "disk_cleanup.json", "disk_cleanup.json path", "disk_cleanup path", "disk_cleanup file"],
        size_phrases: &["disk cleanup size", "disk cleanup file size", "disk_cleanup.json size", "disk_cleanup size", "disk_cleanup file size", "disk-cleanup size"],
        age_phrases: &["disk cleanup age", "disk cleanup file age", "disk_cleanup.json age", "disk_cleanup age", "disk_cleanup file age", "disk-cleanup age"],
    },
    ResourceLane {
        id: "downloads_organizer_rules",
        nouns: &["downloads organizer rules", "downloads-organizer-rules", "downloads_organizer_rules", "rules", "downloads-organizer-rules.md", "downloads organizer rule", "organizer-rules.md", "organizer-rules", "organizer rules", "organizer rule", "organizer rules size", "downloads-organizer"],
        max_len: 88,
        path_phrases: &["downloads organizer rules path", "downloads organizer rule path", "downloads-organizer-rules", "downloads-organizer-rules.md", "downloads-organizer-rules.md path", "downloads-organizer-rules path"],
        size_phrases: &["organizer rules size", "organizer rule size", "downloads organizer rules size", "downloads organizer rule size", "downloads-organizer-rules size", "downloads-organizer-rules.md size"],
        age_phrases: &["organizer rules age", "organizer rule age", "downloads organizer rules age", "downloads organizer rule age", "downloads-organizer-rules age", "downloads-organizer-rules.md age"],
    },
    ResourceLane {
        id: "downloads_organizer_state",
        nouns: &["downloads organizer state", "downloads-organizer-state", "downloads_organizer_state", "state", "downloads-organizer-state.json", "organizer-state.json", "organizer-state", "organizer state", "organizer state size", "downloads-organizer", "downloads organizer", "organizer"],
        max_len: 88,
        path_phrases: &["downloads organizer state path", "downloads-organizer-state", "downloads-organizer-state.json", "downloads-organizer-state.json path", "downloads-organizer-state path", "downloads-organizer-state file"],
        size_phrases: &["organizer state size", "downloads organizer state size", "downloads-organizer-state size", "downloads-organizer-state.json size", "downloads-organizer-state file size", "downloads_organizer_state size"],
        age_phrases: &["organizer state age", "downloads organizer state age", "downloads-organizer-state age", "downloads-organizer-state.json age", "downloads-organizer-state file age", "downloads_organizer_state age"],
    },
    ResourceLane {
        id: "escalation_patterns",
        nouns: &["escalation patterns", "escalation-patterns", "escalation_patterns", "patterns", "escalation_patterns.md", "escalation pattern", "escalation phrases", "escalation phrase", "escalation file", "escalation size", "escalation", "size"],
        max_len: 88,
        path_phrases: &["escalation patterns path", "escalation pattern path", "escalation_patterns", "escalation_patterns.md", "escalation_patterns.md path", "escalation_patterns path"],
        size_phrases: &["escalation size", "escalation patterns size", "escalation pattern size", "escalation_patterns size", "escalation_patterns.md size", "escalation_patterns file size"],
        age_phrases: &["escalation age", "escalation patterns age", "escalation pattern age", "escalation_patterns age", "escalation_patterns.md age", "escalation_patterns file age"],
    },
    ResourceLane {
        id: "execution_prompt",
        nouns: &["execution prompt", "execution-prompt", "execution_prompt", "prompt", "execution_prompt.md", "execution file", "execution md", "execution", "size", "big", "large", "bytes"],
        max_len: 80,
        path_phrases: &["execution_prompt.md", "execution_prompt.md path", "execution_prompt path", "execution prompt path", "execution prompt file path", "execution file path"],
        size_phrases: &["execution size", "execution_prompt.md size", "execution_prompt size", "execution prompt size", "execution file size", "execution md size"],
        age_phrases: &["execution age", "execution_prompt.md age", "execution_prompt age", "execution prompt age", "execution file age", "execution md age"],
    },
    ResourceLane {
        id: "harness_loop_stderr",
        nouns: &["harness loop stderr", "harness-loop-stderr", "harness_loop_stderr", "stderr", "stderr.log"],
        max_len: 88,
        path_phrases: &["harness loop stderr path", "harness loop stderr log path", "harness loop stderr.log path", "overnight harness loop stderr path", "overnight harness loop stderr log path", "overnight_harness_loop.stderr.log path"],
        size_phrases: &["harness loop stderr size", "harness loop stderr log size", "harness loop stderr.log size", "overnight harness loop stderr size", "overnight harness loop stderr log size", "overnight_harness_loop.stderr.log size"],
        age_phrases: &["harness loop stderr age", "harness loop stderr log age", "harness loop stderr.log age", "overnight harness loop stderr age", "overnight harness loop stderr log age", "overnight_harness_loop.stderr.log age"],
    },
    ResourceLane {
        id: "harness_loop_stdout",
        nouns: &["harness loop stdout", "harness-loop-stdout", "harness_loop_stdout", "stdout", "harness_loop", "harness loop", "stdout.log"],
        max_len: 88,
        path_phrases: &["harness loop stdout path", "harness loop stdout log path", "harness loop stdout.log path", "overnight harness loop stdout path", "overnight harness loop stdout log path", "overnight_harness_loop.stdout.log path"],
        size_phrases: &["harness loop stdout size", "harness loop stdout log size", "harness loop stdout.log size", "overnight harness loop stdout size", "overnight harness loop stdout log size", "overnight_harness_loop.stdout.log size"],
        age_phrases: &["harness loop stdout age", "harness loop stdout log age", "harness loop stdout.log age", "overnight harness loop stdout age", "overnight harness loop stdout log age", "overnight_harness_loop.stdout.log age"],
    },
    ResourceLane {
        id: "history",
        nouns: &["history", "history.json", "history json", "history file", "metrics history", "cpu history", "old", "stale", "when", "updated", "modified", "size"],
        max_len: 72,
        path_phrases: &["history path", "history.json", "history.json path", "history file", "history file path", "history_file"],
        size_phrases: &["history size", "history file size", "history.json size", "history json size", "metrics history size", "cpu history size"],
        age_phrases: &["history age", "history file age", "history.json age", "history json age", "metrics history age", "cpu history age"],
    },
    ResourceLane {
        id: "improvements",
        nouns: &["improvements", "improvements folder", "improvements directory", "improvements dir", "improvements path", "improvement folder", "improvement directory", "improvement dir", "improvement path", "mac-stats improvements", "mac stats improvements", "autoresearch folder"],
        max_len: 72,
        path_phrases: &["improvements path", "improvement path", "improvements folder", "improvement folder", "improvements directory", "improvement directory"],
        size_phrases: &["improvements size", "improvement size", "improvements folder size", "improvements directory size", "improvements dir size", "improvement folder size"],
        age_phrases: &["improvements age", "improvement age", "improvements folder age", "improvements directory age", "improvements dir age", "improvement folder age"],
    },
    ResourceLane {
        id: "launchagent",
        nouns: &["launchagent", "launch agent", "plist", "launch-agent", "launch_agent", "com.raro42.mac-stats.plist", "com.raro42.mac-stats-overnight-harness.plist", "mac-stats.plist", "mac_stats.plist", "overnight-harness.plist", "overnight harness plist", "harness plist"],
        max_len: 88,
        path_phrases: &["launchagent path", "launch agent path", "launch-agent path", "launch_agent path", "launchagent plist", "launch agent plist"],
        size_phrases: &["launchagent size", "launch agent size", "launch-agent size", "launch_agent size", "launchagent plist size", "launch agent plist size"],
        age_phrases: &["launchagent age", "launch agent age", "launch-agent age", "launch_agent age", "launchagent plist age", "launch agent plist age"],
    },
    ResourceLane {
        id: "launchd_stderr",
        nouns: &["launchd stderr", "launchd-stderr", "launchd_stderr", "stderr"],
        max_len: 88,
        path_phrases: &["launchd stderr path", "launchd stderr log path", "launchd.stderr.log path", "launchd.stderr.log", "launchd.stderr path", "launchd stderr"],
        size_phrases: &["launchd stderr size", "launchd stderr log size", "launchd.stderr.log size", "launchd.stderr size", "launchd-stderr size", "launchd-stderr.log size"],
        age_phrases: &["launchd stderr age", "launchd stderr log age", "launchd.stderr.log age", "launchd.stderr age", "launchd-stderr age", "launchd-stderr.log age"],
    },
    ResourceLane {
        id: "launchd_stdout",
        nouns: &["launchd stdout", "launchd-stdout", "launchd_stdout", "stdout"],
        max_len: 88,
        path_phrases: &["launchd stdout path", "launchd stdout log path", "launchd.stdout.log path", "launchd.stdout.log", "launchd.stdout path", "launchd stdout"],
        size_phrases: &["launchd stdout size", "launchd stdout log size", "launchd.stdout.log size", "launchd.stdout size", "launchd-stdout size", "launchd-stdout.log size"],
        age_phrases: &["launchd stdout age", "launchd stdout log age", "launchd.stdout.log age", "launchd.stdout age", "launchd-stdout age", "launchd-stdout.log age"],
    },
    ResourceLane {
        id: "loop_backlog",
        nouns: &["loop backlog", "loop-backlog", "loop_backlog", "backlog", "tick log", "harness", "overnight", "loop", "standing"],
        max_len: 72,
        path_phrases: &["loop_backlog", "loop-backlog", "loop backlog", "loop_backlog.md", "loop-backlog.md", "loop backlog.md"],
        size_phrases: &["loop_backlog size", "loop-backlog size", "loop backlog size", "loop_backlog.md size", "loop-backlog.md size", "loop backlog.md size"],
        age_phrases: &["loop_backlog age", "loop-backlog age", "loop backlog age", "loop_backlog.md age", "loop-backlog.md age", "loop backlog.md age"],
    },
    ResourceLane {
        id: "memory",
        nouns: &["memory", "notes folder", "notes directory", "notes dir", "notes size", "memory folder", "memory directory", "memory dir", "memory notes", "mac-stats notes", "mac stats notes", "mac-stats memory folder"],
        max_len: 80,
        path_phrases: &["memory path", "notes path", "memory folder", "notes folder", "memory directory", "notes directory"],
        size_phrases: &["notes size", "notes folder size", "notes directory size", "notes dir size", "memory folder size", "memory directory size"],
        age_phrases: &["notes age", "notes folder age", "notes directory age", "notes dir age", "memory folder age", "memory directory age"],
    },
    ResourceLane {
        id: "memory_md",
        nouns: &["memory md", "memory-md", "memory_md", "md", "memory.md", "curated memory", "memory file", "memory", "size", "big", "large", "bytes"],
        max_len: 80,
        path_phrases: &["memory.md", "memory.md path", "memory.md file", "memory.md file path", "memory md", "memory md path"],
        size_phrases: &["memory.md size", "memory md size", "memory file size", "memory.md file size", "curated memory size", "curated memory file size"],
        age_phrases: &["memory.md age", "memory md age", "memory file age", "memory.md file age", "curated memory age", "curated memory file age"],
    },
    ResourceLane {
        id: "monitors",
        nouns: &["monitors", "monitors.json", "monitors json", "monitor file", "monitors file", "sites file", "old", "stale", "when", "updated", "modified", "monitor"],
        max_len: 72,
        path_phrases: &["monitors path", "monitor path", "monitors.json", "monitors.json path", "monitors file", "monitors file path"],
        size_phrases: &["monitors size", "monitor size", "monitors file size", "monitor file size", "monitors.json size", "monitors json size"],
        age_phrases: &["monitors age", "monitor age", "monitors file age", "monitor file age", "monitors.json age", "monitors json age"],
    },
    ResourceLane {
        id: "mood",
        nouns: &["mood", "mood.md", "mood file", "mood path", "moods path", "path", "where", "location", "folder", "directory", "dir", "file"],
        max_len: 80,
        path_phrases: &["mood path", "mood.md", "mood.md path", "mood file", "mood file path", "mood md"],
        size_phrases: &["mood size", "mood.md size", "mood file size", "mood md size", "mood.md file size", "mood bytes"],
        age_phrases: &["mood age", "mood.md age", "mood file age", "mood md age", "mood.md file age", "mac-stats mood age"],
    },
    ResourceLane {
        id: "morning_surprise",
        nouns: &["morning surprise", "morning-surprise", "morning_surprise", "surprise"],
        max_len: 72,
        path_phrases: &["morning_surprise path", "morning-surprise path", "morning surprise path", "morning_surprise.md path", "morning-surprise.md path", "morning surprise.md path"],
        size_phrases: &["morning_surprise size", "morning-surprise size", "morning surprise size", "morning_surprise.md size", "morning-surprise.md size", "morning surprise.md size"],
        age_phrases: &["morning_surprise age", "morning-surprise age", "morning surprise age", "morning_surprise.md age", "morning-surprise.md age", "morning surprise.md age"],
    },
    ResourceLane {
        id: "ori_vault",
        nouns: &["ori vault", "ori-vault", "ori_vault", "vault", "mnemos vault", "orivault", "ori", "size", "big", "large", "bytes", "folder"],
        max_len: 88,
        path_phrases: &["ori vault path", "ori vault folder", "ori vault directory", "ori vault dir", "ori vault location", "mnemos vault path"],
        size_phrases: &["/ori", "/mnemos", "/ori-mnemos", "ori", "mnemos", "ori vault"],
        age_phrases: &["/ori", "/mnemos", "/ori-mnemos", "ori", "mnemos", "ori vault"],
    },
    ResourceLane {
        id: "overnight_agent_log",
        nouns: &["overnight agent log", "overnight-agent-log", "overnight_agent_log", "log", "overnight_agent", "overnight agent"],
        max_len: 72,
        path_phrases: &["overnight_agent path", "overnight-agent path", "overnight agent path", "overnight_agent.log path", "overnight-agent.log path", "overnight agent.log path"],
        size_phrases: &["overnight_agent size", "overnight-agent size", "overnight agent size", "overnight_agent.log size", "overnight-agent.log size", "overnight agent.log size"],
        age_phrases: &["overnight_agent age", "overnight-agent age", "overnight agent age", "overnight_agent.log age", "overnight-agent.log age", "overnight agent.log age"],
    },
    ResourceLane {
        id: "pdfs",
        nouns: &["pdfs", "pdfs folder", "pdfs directory", "pdfs dir", "pdfs age", "pdf folder", "pdf directory", "pdf dir", "pdf age", "pdf exports", "pdf export", "browser pdfs"],
        max_len: 72,
        path_phrases: &["pdfs path", "pdf path", "pdfs folder", "pdf folder", "pdfs directory", "pdf directory"],
        size_phrases: &["pdfs size", "pdf size", "pdfs folder size", "pdf folder size", "pdfs directory size", "pdf directory size"],
        age_phrases: &["pdfs age", "pdf age", "pdfs folder age", "pdf folder age", "pdfs directory age", "pdf directory age"],
    },
    ResourceLane {
        id: "perplexity_last",
        nouns: &["perplexity last", "perplexity-last", "perplexity_last", "last", "perplexity_last.json", "perplexity-last.json", "perplexity last.json", "perplexity last file", "perplexity cache", "last search file", "last search cache", "perplexity cache file"],
        max_len: 72,
        path_phrases: &["perplexity last path", "perplexity_last", "perplexity_last.json", "perplexity_last.json path", "perplexity_last path", "perplexity_last file"],
        size_phrases: &["perplexity last size", "perplexity_last.json size", "perplexity_last size", "perplexity_last file size", "perplexity-last size", "perplexity-last.json size"],
        age_phrases: &["perplexity last age", "perplexity_last.json age", "perplexity_last age", "perplexity_last file age", "perplexity-last age", "perplexity-last.json age"],
    },
    ResourceLane {
        id: "pinned_processes",
        nouns: &["pinned processes", "pinned-processes", "pinned_processes", "processes", "pinned processes.json", "pinned process file", "pin file", "pins file", "favorites file", "process favorites file", "pinned favorites file", "pinned favorites"],
        max_len: 72,
        path_phrases: &["pinned processes path", "pinned process path", "pinned_processes", "pinned_processes.json", "pinned_processes path", "pinned_processes file"],
        size_phrases: &["pinned processes size", "pinned process size", "pinned_processes.json size", "pinned_processes size", "pinned_processes file size", "pinned-processes size"],
        age_phrases: &["pinned processes age", "pinned process age", "pinned_processes.json age", "pinned_processes age", "pinned_processes file age", "pinned-processes age"],
    },
    ResourceLane {
        id: "planning_prompt",
        nouns: &["planning prompt", "planning-prompt", "planning_prompt", "prompt", "planning_prompt.md", "planning file", "planning md", "planning", "size", "big", "large", "bytes"],
        max_len: 80,
        path_phrases: &["planning_prompt.md", "planning_prompt.md path", "planning_prompt path", "planning prompt path", "planning prompt file path", "planning file path"],
        size_phrases: &["planning size", "planning_prompt.md size", "planning_prompt size", "planning prompt size", "planning file size", "planning md size"],
        age_phrases: &["planning age", "planning_prompt.md age", "planning_prompt age", "planning prompt age", "planning file age", "planning md age"],
    },
    ResourceLane {
        id: "plugins",
        nouns: &["plugins", "plugins folder", "plugin folder", "plugins directory", "plugin directory", "plugins dir", "plugin dir", "plugins size", "plugin size", "scripts folder", "script folder", "scripts directory"],
        max_len: 72,
        path_phrases: &["plugins path", "plugin path", "plugins folder", "plugin folder", "plugins directory", "plugin directory"],
        size_phrases: &["plugins size", "plugin size", "plugins folder size", "plugin folder size", "plugins directory size", "plugin directory size"],
        age_phrases: &["plugins age", "plugin age", "plugins folder age", "plugin folder age", "plugins directory age", "plugin directory age"],
    },
    ResourceLane {
        id: "prompts",
        nouns: &["prompts", "prompts folder", "prompt folder", "prompts directory", "prompt directory", "prompts dir", "prompt dir", "prompts age", "prompt age", "mac-stats prompts", "mac stats prompts", "prompt"],
        max_len: 72,
        path_phrases: &["prompts path", "prompt path", "prompts folder", "prompt folder", "prompts directory", "prompt directory"],
        size_phrases: &["prompts size", "prompt size", "prompts folder size", "prompt folder size", "prompts directory size", "prompt directory size"],
        age_phrases: &["prompts age", "prompt age", "prompts folder age", "prompt folder age", "prompts directory age", "prompt directory age"],
    },
    ResourceLane {
        id: "results_tsv",
        nouns: &["results tsv", "results-tsv", "results_tsv", "tsv", "results.tsv", "autoresearch results", "results file", "ratchet results", "keep discard results", "keep discard log", "results", "ratchet"],
        max_len: 72,
        path_phrases: &["results.tsv", "results.tsv path", "results tsv", "results tsv path", "results.tsv file", "results tsv file"],
        size_phrases: &["results.tsv size", "results tsv size", "results size", "results file size", "autoresearch results size", "ratchet results size"],
        age_phrases: &["results.tsv age", "results tsv age", "results age", "autoresearch results age", "ratchet results age", "keep discard results age"],
    },
    ResourceLane {
        id: "runs",
        nouns: &["runs", "runs.jsonl", "runs jsonl", "run log", "runs file", "runs log", "size", "big", "large", "bytes", " mb", " kb"],
        max_len: 72,
        path_phrases: &["runs path", "runs.jsonl path", "runs jsonl path", "runs file path", "run log path", "runs log path"],
        size_phrases: &["runs size", "runs.jsonl size", "runs jsonl size", "run log size", "runs log size", "runs file size"],
        age_phrases: &["runs age", "runs.jsonl age", "runs jsonl age", "run log age", "runs log age", "runs file age"],
    },
    ResourceLane {
        id: "scheduler_delivery_awareness",
        nouns: &["scheduler delivery awareness", "scheduler-delivery-awareness", "scheduler_delivery_awareness", "awareness", "scheduler_delivery_awareness.json", "delivery_awareness.json", "delivery_awareness", "delivery-awareness", "delivery awareness.json", "delivery awareness file", "delivery awareness size", "scheduler awareness"],
        max_len: 88,
        path_phrases: &["delivery awareness path", "delivery_awareness", "delivery_awareness.json", "delivery_awareness.json path", "delivery_awareness path", "delivery_awareness file"],
        size_phrases: &["delivery awareness size", "delivery_awareness size", "delivery_awareness.json size", "delivery_awareness file size", "delivery-awareness size", "delivery-awareness.json size"],
        age_phrases: &["delivery awareness age", "delivery_awareness age", "delivery_awareness.json age", "delivery_awareness file age", "delivery-awareness age", "delivery-awareness.json age"],
    },
    ResourceLane {
        id: "schedules",
        nouns: &["schedules", "schedules.json", "schedules_file", "schedule file", "schedules file", "cron file", "cron.json", "path", "where", "location", "folder", "directory"],
        max_len: 72,
        path_phrases: &["schedules path", "schedule path", "schedules.json", "schedules.json path", "schedules file", "schedules file path"],
        size_phrases: &["schedules size", "schedule size", "schedules file size", "schedule file size", "schedules.json size", "schedules json size"],
        age_phrases: &["schedules age", "schedule age", "schedules file age", "schedule file age", "schedules.json age", "schedules json age"],
    },
    ResourceLane {
        id: "screenshots",
        nouns: &["screenshots", "screenshot"],
        max_len: 72,
        path_phrases: &["screenshot path", "screenshots path", "screenshot folder", "screenshots folder", "screenshot directory", "screenshots directory"],
        size_phrases: &["screenshots size", "screenshot size", "screenshots folder size", "screenshot folder size", "screenshots directory size", "screenshot directory size"],
        age_phrases: &["screenshots age", "screenshot age", "screenshots folder age", "screenshot folder age", "screenshots directory age", "screenshot directory age"],
    },
    ResourceLane {
        id: "session",
        nouns: &["session", "session folder", "sessions folder", "session directory", "sessions directory", "session dir", "sessions dir", "session age", "sessions age", "mac-stats session", "mac stats session", "mac-stats sessions"],
        max_len: 72,
        path_phrases: &["session path", "sessions path", "session folder", "sessions folder", "session directory", "sessions directory"],
        size_phrases: &["session size", "sessions size", "session folder size", "sessions folder size", "session directory size", "sessions directory size"],
        age_phrases: &["session age", "sessions age", "session folder age", "sessions folder age", "session directory age", "sessions directory age"],
    },
    ResourceLane {
        id: "session_memory",
        nouns: &["session memory", "session-memory", "session_memory", "memory", "sessionmemory", "session memories", "session", "memories", "size", "big", "large", "bytes"],
        max_len: 88,
        path_phrases: &["/sessions", "/sessions live", "/sessions files", "sessions", "live sessions", "session files"],
        size_phrases: &["/sessions", "/sessions live", "/sessions files", "sessions", "live sessions", "session files"],
        age_phrases: &["/sessions", "/sessions live", "/sessions files", "sessions", "live sessions", "session files"],
    },
    ResourceLane {
        id: "session_reset_phrases",
        nouns: &["session reset phrases", "session-reset-phrases", "session_reset_phrases", "phrases", "session_reset_phrases.md", "session reset phrase", "session-reset phrases", "session_reset phrases", "reset phrases", "reset phrase", "reset phrases file", "reset phrase file"],
        max_len: 88,
        path_phrases: &["session reset phrases path", "session reset phrase path", "session_reset_phrases", "session_reset_phrases.md", "session_reset_phrases.md path", "session_reset_phrases path"],
        size_phrases: &["session reset size", "session reset phrases size", "session reset phrase size", "session_reset_phrases size", "session_reset_phrases.md size", "session_reset_phrases file size"],
        age_phrases: &["session reset age", "session reset phrases age", "session reset phrase age", "session_reset_phrases age", "session_reset_phrases.md age", "session_reset_phrases file age"],
    },
    ResourceLane {
        id: "sibling_harness",
        nouns: &["sibling harness", "sibling-harness", "sibling_harness", "harness"],
        max_len: 72,
        path_phrases: &["sibling_harness", "sibling-harness", "sibling harness", "sibling_harness.md", "sibling-harness.md", "sibling harness.md"],
        size_phrases: &["sibling_harness size", "sibling-harness size", "sibling harness size", "sibling_harness.md size", "sibling-harness.md size", "sibling harness.md size"],
        age_phrases: &["sibling_harness age", "sibling-harness age", "sibling harness age", "sibling_harness.md age", "sibling-harness.md age", "sibling harness.md age"],
    },
    ResourceLane {
        id: "skill_md",
        nouns: &["skill md", "skill-md", "skill_md", "md", "skill.md", "skill file", "skill", "skills", "path", "where", "location", "folder"],
        max_len: 80,
        path_phrases: &["skill.md", "skill.md path", "skill file", "skill file path", "skill md", "skill md path"],
        size_phrases: &["skill.md size", "skill file size", "skill md size", "skill.md file size", "skill.md bytes", "skill file bytes"],
        age_phrases: &["skill.md age", "skill file age", "skill md age", "skill.md file age", "agent skill.md age", "agent skill file age"],
    },
    ResourceLane {
        id: "skills",
        nouns: &["skills", "skills folder", "skill folder", "skills directory", "skill directory", "skills dir", "skill dir", "skills size", "skill size", "mac-stats skills", "mac stats skills", "skill"],
        max_len: 72,
        path_phrases: &["skills path", "skill path", "skills folder", "skill folder", "skills directory", "skill directory"],
        size_phrases: &["skills size", "skill size", "skills folder size", "skill folder size", "skills directory size", "skill directory size"],
        age_phrases: &["skills age", "skill age", "skills folder age", "skill folder age", "skills directory age", "skill directory age"],
    },
    ResourceLane {
        id: "soul",
        nouns: &["soul", "soul.md", "soul file", "soul size", "soul md", "size", "big", "large", "bytes", " mb", " kb", " gi"],
        max_len: 80,
        path_phrases: &["soul path", "soul.md", "soul.md path", "soul file", "soul file path", "soul md"],
        size_phrases: &["soul size", "soul.md size", "soul file size", "soul md size", "soul.md file size", "soul bytes"],
        age_phrases: &["soul age", "soul.md age", "soul file age", "soul md age", "soul.md file age", "mac-stats soul age"],
    },
    ResourceLane {
        id: "standing_backlog",
        nouns: &["standing backlog", "standing-backlog", "standing_backlog", "backlog"],
        max_len: 72,
        path_phrases: &["standing_backlog", "standing-backlog", "standing backlog", "standing_backlog.md", "standing-backlog.md", "standing backlog.md"],
        size_phrases: &["standing_backlog size", "standing-backlog size", "standing backlog size", "standing_backlog.md size", "standing-backlog.md size", "standing backlog.md size"],
        age_phrases: &["standing_backlog age", "standing-backlog age", "standing backlog age", "standing_backlog.md age", "standing-backlog.md age", "standing backlog.md age"],
    },
    ResourceLane {
        id: "task",
        nouns: &["task", "task folder", "tasks folder", "task directory", "tasks directory", "task dir", "tasks dir", "task age", "tasks age", "mac-stats task", "mac stats task", "mac-stats tasks"],
        max_len: 72,
        path_phrases: &["task path", "tasks path", "task folder", "tasks folder", "task directory", "tasks directory"],
        size_phrases: &["task size", "tasks size", "task folder size", "tasks folder size", "task directory size", "tasks directory size"],
        age_phrases: &["task age", "tasks age", "task folder age", "tasks folder age", "task directory age", "tasks directory age"],
    },
    ResourceLane {
        id: "testing_md",
        nouns: &["testing md", "testing-md", "testing_md", "md", "testing.md", "testing file", "testing", "path", "where", "location", "folder", "directory"],
        max_len: 80,
        path_phrases: &["testing.md", "testing.md path", "testing path", "testing file", "testing file path", "testing md"],
        size_phrases: &["testing size", "testing.md size", "testing file size", "testing md size", "testing.md file size", "testing.md bytes"],
        age_phrases: &["testing age", "testing.md age", "testing file age", "testing md age", "testing.md file age", "agent testing.md age"],
    },
    ResourceLane {
        id: "tmp",
        nouns: &["tmp", "tmp folder", "tmp directory", "tmp dir", "tmp age", "temp folder", "temp directory", "temp dir", "temp age", "temporary folder", "temporary directory", "temporary dir"],
        max_len: 72,
        path_phrases: &["tmp path", "tmp folder", "tmp directory", "tmp dir", "temp path", "temp folder"],
        size_phrases: &["tmp size", "temp size", "temporary size", "scratch size", "tmp folder size", "temp folder size"],
        age_phrases: &["tmp age", "temp age", "temporary age", "scratch age", "tmp folder age", "temp folder age"],
    },
    ResourceLane {
        id: "traces",
        nouns: &["traces", "traces folder", "traces directory", "traces dir", "traces age", "trace folder", "trace directory", "trace dir", "trace age", "cdp traces", "cdp trace", "browser traces"],
        max_len: 72,
        path_phrases: &["traces path", "trace path", "traces folder", "trace folder", "traces directory", "trace directory"],
        size_phrases: &["traces size", "trace size", "traces folder size", "trace folder size", "traces directory size", "trace directory size"],
        age_phrases: &["traces age", "trace age", "traces folder age", "trace folder age", "traces directory age", "trace directory age"],
    },
    ResourceLane {
        id: "uploads",
        nouns: &["uploads", "uploads folder", "uploads directory", "uploads dir", "uploads age", "upload folder", "upload directory", "upload dir", "upload age", "browser uploads", "browser upload", "mac-stats uploads"],
        max_len: 72,
        path_phrases: &["uploads path", "upload path", "uploads folder", "upload folder", "uploads directory", "upload directory"],
        size_phrases: &["uploads size", "upload size", "uploads folder size", "upload folder size", "uploads directory size", "upload directory size"],
        age_phrases: &["uploads age", "upload age", "uploads folder age", "upload folder age", "uploads directory age", "upload directory age"],
    },
    ResourceLane {
        id: "user_info",
        nouns: &["user info", "user-info", "user_info", "info", "user-info.json", "user_info.json", "userinfo.json", "userinfo", "user info.json", "user info file", "user info size", "user-info size"],
        max_len: 88,
        path_phrases: &["user info path", "user-info", "user-info.json", "user-info.json path", "user-info path", "user-info file"],
        size_phrases: &["user info size", "user-info size", "user-info.json size", "user-info file size", "user_info size", "user_info.json size"],
        age_phrases: &["user info age", "user-info age", "user-info.json age", "user-info file age", "user_info age", "user_info.json age"],
    },
];

pub fn lane_by_id(id: &str) -> Option<&'static ResourceLane> {
    RESOURCE_LANES.iter().find(|l| l.id == id)
}


// Thin wrappers preserving `looks_like_*_{path,size,age}_request` names.

pub fn looks_like_agent_json_path_request(content: &str) -> bool {
    lane_by_id("agent_json").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_agent_json_size_request(content: &str) -> bool {
    lane_by_id("agent_json").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_agent_json_age_request(content: &str) -> bool {
    lane_by_id("agent_json").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_agents_path_request(content: &str) -> bool {
    lane_by_id("agents").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_agents_size_request(content: &str) -> bool {
    lane_by_id("agents").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_agents_age_request(content: &str) -> bool {
    lane_by_id("agents").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_before_compaction_transcript_path_request(content: &str) -> bool {
    lane_by_id("before_compaction_transcript").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_before_compaction_transcript_size_request(content: &str) -> bool {
    lane_by_id("before_compaction_transcript").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_before_compaction_transcript_age_request(content: &str) -> bool {
    lane_by_id("before_compaction_transcript").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_before_reset_transcript_path_request(content: &str) -> bool {
    lane_by_id("before_reset_transcript").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_before_reset_transcript_size_request(content: &str) -> bool {
    lane_by_id("before_reset_transcript").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_before_reset_transcript_age_request(content: &str) -> bool {
    lane_by_id("before_reset_transcript").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_browser_credentials_path_request(content: &str) -> bool {
    lane_by_id("browser_credentials").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_browser_credentials_size_request(content: &str) -> bool {
    lane_by_id("browser_credentials").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_browser_credentials_age_request(content: &str) -> bool {
    lane_by_id("browser_credentials").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_browser_downloads_path_request(content: &str) -> bool {
    lane_by_id("browser_downloads").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_browser_downloads_size_request(content: &str) -> bool {
    lane_by_id("browser_downloads").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_browser_downloads_age_request(content: &str) -> bool {
    lane_by_id("browser_downloads").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_browser_storage_state_path_request(content: &str) -> bool {
    lane_by_id("browser_storage_state").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_browser_storage_state_size_request(content: &str) -> bool {
    lane_by_id("browser_storage_state").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_browser_storage_state_age_request(content: &str) -> bool {
    lane_by_id("browser_storage_state").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_cleanup_quarantine_path_request(content: &str) -> bool {
    lane_by_id("cleanup_quarantine").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_cleanup_quarantine_size_request(content: &str) -> bool {
    lane_by_id("cleanup_quarantine").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_cleanup_quarantine_age_request(content: &str) -> bool {
    lane_by_id("cleanup_quarantine").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_config_path_request(content: &str) -> bool {
    lane_by_id("config").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_config_size_request(content: &str) -> bool {
    lane_by_id("config").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_config_age_request(content: &str) -> bool {
    lane_by_id("config").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_config_env_path_request(content: &str) -> bool {
    lane_by_id("config_env").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_config_env_size_request(content: &str) -> bool {
    lane_by_id("config_env").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_config_env_age_request(content: &str) -> bool {
    lane_by_id("config_env").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_cookie_reject_patterns_path_request(content: &str) -> bool {
    lane_by_id("cookie_reject_patterns").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_cookie_reject_patterns_size_request(content: &str) -> bool {
    lane_by_id("cookie_reject_patterns").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_cookie_reject_patterns_age_request(content: &str) -> bool {
    lane_by_id("cookie_reject_patterns").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_credential_accounts_path_request(content: &str) -> bool {
    lane_by_id("credential_accounts").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_credential_accounts_size_request(content: &str) -> bool {
    lane_by_id("credential_accounts").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_credential_accounts_age_request(content: &str) -> bool {
    lane_by_id("credential_accounts").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_debug_log_path_request(content: &str) -> bool {
    lane_by_id("debug_log").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_debug_log_size_request(content: &str) -> bool {
    lane_by_id("debug_log").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_debug_log_age_request(content: &str) -> bool {
    lane_by_id("debug_log").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_digest_path_request(content: &str) -> bool {
    lane_by_id("digest").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_digest_size_request(content: &str) -> bool {
    lane_by_id("digest").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_digest_age_request(content: &str) -> bool {
    lane_by_id("digest").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_digest_md_path_request(content: &str) -> bool {
    lane_by_id("digest_md").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_digest_md_size_request(content: &str) -> bool {
    lane_by_id("digest_md").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_digest_md_age_request(content: &str) -> bool {
    lane_by_id("digest_md").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_discord_channels_path_request(content: &str) -> bool {
    lane_by_id("discord_channels").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_discord_channels_size_request(content: &str) -> bool {
    lane_by_id("discord_channels").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_discord_channels_age_request(content: &str) -> bool {
    lane_by_id("discord_channels").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_discord_memory_path_request(content: &str) -> bool {
    lane_by_id("discord_memory").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_discord_memory_size_request(content: &str) -> bool {
    lane_by_id("discord_memory").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_discord_memory_age_request(content: &str) -> bool {
    lane_by_id("discord_memory").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_disk_cleanup_path_request(content: &str) -> bool {
    lane_by_id("disk_cleanup").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_disk_cleanup_size_request(content: &str) -> bool {
    lane_by_id("disk_cleanup").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_disk_cleanup_age_request(content: &str) -> bool {
    lane_by_id("disk_cleanup").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_downloads_organizer_rules_path_request(content: &str) -> bool {
    lane_by_id("downloads_organizer_rules").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_downloads_organizer_rules_size_request(content: &str) -> bool {
    lane_by_id("downloads_organizer_rules").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_downloads_organizer_rules_age_request(content: &str) -> bool {
    lane_by_id("downloads_organizer_rules").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_downloads_organizer_state_path_request(content: &str) -> bool {
    lane_by_id("downloads_organizer_state").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_downloads_organizer_state_size_request(content: &str) -> bool {
    lane_by_id("downloads_organizer_state").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_downloads_organizer_state_age_request(content: &str) -> bool {
    lane_by_id("downloads_organizer_state").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_escalation_patterns_path_request(content: &str) -> bool {
    lane_by_id("escalation_patterns").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_escalation_patterns_size_request(content: &str) -> bool {
    lane_by_id("escalation_patterns").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_escalation_patterns_age_request(content: &str) -> bool {
    lane_by_id("escalation_patterns").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_execution_prompt_path_request(content: &str) -> bool {
    lane_by_id("execution_prompt").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_execution_prompt_size_request(content: &str) -> bool {
    lane_by_id("execution_prompt").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_execution_prompt_age_request(content: &str) -> bool {
    lane_by_id("execution_prompt").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_harness_loop_stderr_path_request(content: &str) -> bool {
    lane_by_id("harness_loop_stderr").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_harness_loop_stderr_size_request(content: &str) -> bool {
    lane_by_id("harness_loop_stderr").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_harness_loop_stderr_age_request(content: &str) -> bool {
    lane_by_id("harness_loop_stderr").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_harness_loop_stdout_path_request(content: &str) -> bool {
    lane_by_id("harness_loop_stdout").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_harness_loop_stdout_size_request(content: &str) -> bool {
    lane_by_id("harness_loop_stdout").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_harness_loop_stdout_age_request(content: &str) -> bool {
    lane_by_id("harness_loop_stdout").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_history_path_request(content: &str) -> bool {
    lane_by_id("history").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_history_size_request(content: &str) -> bool {
    lane_by_id("history").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_history_age_request(content: &str) -> bool {
    lane_by_id("history").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_improvements_path_request(content: &str) -> bool {
    lane_by_id("improvements").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_improvements_size_request(content: &str) -> bool {
    lane_by_id("improvements").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_improvements_age_request(content: &str) -> bool {
    lane_by_id("improvements").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_launchagent_path_request(content: &str) -> bool {
    lane_by_id("launchagent").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_launchagent_size_request(content: &str) -> bool {
    lane_by_id("launchagent").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_launchagent_age_request(content: &str) -> bool {
    lane_by_id("launchagent").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_launchd_stderr_path_request(content: &str) -> bool {
    lane_by_id("launchd_stderr").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_launchd_stderr_size_request(content: &str) -> bool {
    lane_by_id("launchd_stderr").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_launchd_stderr_age_request(content: &str) -> bool {
    lane_by_id("launchd_stderr").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_launchd_stdout_path_request(content: &str) -> bool {
    lane_by_id("launchd_stdout").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_launchd_stdout_size_request(content: &str) -> bool {
    lane_by_id("launchd_stdout").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_launchd_stdout_age_request(content: &str) -> bool {
    lane_by_id("launchd_stdout").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_loop_backlog_path_request(content: &str) -> bool {
    lane_by_id("loop_backlog").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_loop_backlog_size_request(content: &str) -> bool {
    lane_by_id("loop_backlog").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_loop_backlog_age_request(content: &str) -> bool {
    lane_by_id("loop_backlog").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_memory_path_request(content: &str) -> bool {
    lane_by_id("memory").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_memory_size_request(content: &str) -> bool {
    lane_by_id("memory").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_memory_age_request(content: &str) -> bool {
    lane_by_id("memory").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_memory_md_path_request(content: &str) -> bool {
    lane_by_id("memory_md").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_memory_md_size_request(content: &str) -> bool {
    lane_by_id("memory_md").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_memory_md_age_request(content: &str) -> bool {
    lane_by_id("memory_md").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_monitors_path_request(content: &str) -> bool {
    lane_by_id("monitors").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_monitors_size_request(content: &str) -> bool {
    lane_by_id("monitors").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_monitors_age_request(content: &str) -> bool {
    lane_by_id("monitors").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_mood_path_request(content: &str) -> bool {
    lane_by_id("mood").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_mood_size_request(content: &str) -> bool {
    lane_by_id("mood").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_mood_age_request(content: &str) -> bool {
    lane_by_id("mood").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_morning_surprise_path_request(content: &str) -> bool {
    lane_by_id("morning_surprise").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_morning_surprise_size_request(content: &str) -> bool {
    lane_by_id("morning_surprise").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_morning_surprise_age_request(content: &str) -> bool {
    lane_by_id("morning_surprise").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_ori_vault_path_request(content: &str) -> bool {
    lane_by_id("ori_vault").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_ori_vault_size_request(content: &str) -> bool {
    lane_by_id("ori_vault").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_ori_vault_age_request(content: &str) -> bool {
    lane_by_id("ori_vault").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_overnight_agent_log_path_request(content: &str) -> bool {
    lane_by_id("overnight_agent_log").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_overnight_agent_log_size_request(content: &str) -> bool {
    lane_by_id("overnight_agent_log").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_overnight_agent_log_age_request(content: &str) -> bool {
    lane_by_id("overnight_agent_log").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_pdfs_path_request(content: &str) -> bool {
    lane_by_id("pdfs").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_pdfs_size_request(content: &str) -> bool {
    lane_by_id("pdfs").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_pdfs_age_request(content: &str) -> bool {
    lane_by_id("pdfs").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_perplexity_last_path_request(content: &str) -> bool {
    lane_by_id("perplexity_last").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_perplexity_last_size_request(content: &str) -> bool {
    lane_by_id("perplexity_last").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_perplexity_last_age_request(content: &str) -> bool {
    lane_by_id("perplexity_last").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_pinned_processes_path_request(content: &str) -> bool {
    lane_by_id("pinned_processes").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_pinned_processes_size_request(content: &str) -> bool {
    lane_by_id("pinned_processes").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_pinned_processes_age_request(content: &str) -> bool {
    lane_by_id("pinned_processes").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_planning_prompt_path_request(content: &str) -> bool {
    lane_by_id("planning_prompt").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_planning_prompt_size_request(content: &str) -> bool {
    lane_by_id("planning_prompt").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_planning_prompt_age_request(content: &str) -> bool {
    lane_by_id("planning_prompt").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_plugins_path_request(content: &str) -> bool {
    lane_by_id("plugins").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_plugins_size_request(content: &str) -> bool {
    lane_by_id("plugins").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_plugins_age_request(content: &str) -> bool {
    lane_by_id("plugins").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_prompts_path_request(content: &str) -> bool {
    lane_by_id("prompts").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_prompts_size_request(content: &str) -> bool {
    lane_by_id("prompts").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_prompts_age_request(content: &str) -> bool {
    lane_by_id("prompts").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_results_tsv_path_request(content: &str) -> bool {
    lane_by_id("results_tsv").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_results_tsv_size_request(content: &str) -> bool {
    lane_by_id("results_tsv").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_results_tsv_age_request(content: &str) -> bool {
    lane_by_id("results_tsv").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_runs_path_request(content: &str) -> bool {
    lane_by_id("runs").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_runs_size_request(content: &str) -> bool {
    lane_by_id("runs").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_runs_age_request(content: &str) -> bool {
    lane_by_id("runs").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_scheduler_delivery_awareness_path_request(content: &str) -> bool {
    lane_by_id("scheduler_delivery_awareness").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_scheduler_delivery_awareness_size_request(content: &str) -> bool {
    lane_by_id("scheduler_delivery_awareness").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_scheduler_delivery_awareness_age_request(content: &str) -> bool {
    lane_by_id("scheduler_delivery_awareness").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_schedules_path_request(content: &str) -> bool {
    lane_by_id("schedules").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_schedules_size_request(content: &str) -> bool {
    lane_by_id("schedules").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_schedules_age_request(content: &str) -> bool {
    lane_by_id("schedules").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_screenshots_path_request(content: &str) -> bool {
    lane_by_id("screenshots").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_screenshots_size_request(content: &str) -> bool {
    lane_by_id("screenshots").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_screenshots_age_request(content: &str) -> bool {
    lane_by_id("screenshots").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_session_path_request(content: &str) -> bool {
    lane_by_id("session").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_session_size_request(content: &str) -> bool {
    lane_by_id("session").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_session_age_request(content: &str) -> bool {
    lane_by_id("session").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_session_memory_path_request(content: &str) -> bool {
    lane_by_id("session_memory").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_session_memory_size_request(content: &str) -> bool {
    lane_by_id("session_memory").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_session_memory_age_request(content: &str) -> bool {
    lane_by_id("session_memory").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_session_reset_phrases_path_request(content: &str) -> bool {
    lane_by_id("session_reset_phrases").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_session_reset_phrases_size_request(content: &str) -> bool {
    lane_by_id("session_reset_phrases").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_session_reset_phrases_age_request(content: &str) -> bool {
    lane_by_id("session_reset_phrases").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_sibling_harness_path_request(content: &str) -> bool {
    lane_by_id("sibling_harness").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_sibling_harness_size_request(content: &str) -> bool {
    lane_by_id("sibling_harness").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_sibling_harness_age_request(content: &str) -> bool {
    lane_by_id("sibling_harness").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_skill_md_path_request(content: &str) -> bool {
    lane_by_id("skill_md").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_skill_md_size_request(content: &str) -> bool {
    lane_by_id("skill_md").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_skill_md_age_request(content: &str) -> bool {
    lane_by_id("skill_md").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_skills_path_request(content: &str) -> bool {
    lane_by_id("skills").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_skills_size_request(content: &str) -> bool {
    lane_by_id("skills").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_skills_age_request(content: &str) -> bool {
    lane_by_id("skills").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_soul_path_request(content: &str) -> bool {
    lane_by_id("soul").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_soul_size_request(content: &str) -> bool {
    lane_by_id("soul").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_soul_age_request(content: &str) -> bool {
    lane_by_id("soul").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_standing_backlog_path_request(content: &str) -> bool {
    lane_by_id("standing_backlog").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_standing_backlog_size_request(content: &str) -> bool {
    lane_by_id("standing_backlog").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_standing_backlog_age_request(content: &str) -> bool {
    lane_by_id("standing_backlog").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_task_path_request(content: &str) -> bool {
    lane_by_id("task").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_task_size_request(content: &str) -> bool {
    lane_by_id("task").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_task_age_request(content: &str) -> bool {
    lane_by_id("task").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_testing_md_path_request(content: &str) -> bool {
    lane_by_id("testing_md").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_testing_md_size_request(content: &str) -> bool {
    lane_by_id("testing_md").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_testing_md_age_request(content: &str) -> bool {
    lane_by_id("testing_md").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_tmp_path_request(content: &str) -> bool {
    lane_by_id("tmp").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_tmp_size_request(content: &str) -> bool {
    lane_by_id("tmp").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_tmp_age_request(content: &str) -> bool {
    lane_by_id("tmp").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_traces_path_request(content: &str) -> bool {
    lane_by_id("traces").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_traces_size_request(content: &str) -> bool {
    lane_by_id("traces").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_traces_age_request(content: &str) -> bool {
    lane_by_id("traces").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_uploads_path_request(content: &str) -> bool {
    lane_by_id("uploads").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_uploads_size_request(content: &str) -> bool {
    lane_by_id("uploads").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_uploads_age_request(content: &str) -> bool {
    lane_by_id("uploads").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

pub fn looks_like_user_info_path_request(content: &str) -> bool {
    lane_by_id("user_info").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Path))
}

pub fn looks_like_user_info_size_request(content: &str) -> bool {
    lane_by_id("user_info").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Size))
}

pub fn looks_like_user_info_age_request(content: &str) -> bool {
    lane_by_id("user_info").is_some_and(|lane| looks_like_resource_stat(content, lane, StatKind::Age))
}

/// Zero-LLM debug.log file size (stat only; no tail read).
pub fn format_debug_log_size_gateway() -> String {
    let bytes = match snapshot_debug_log_file_bytes() {
        Ok(b) => b,
        Err(e) => return format!("**Debug Log** — could not stat file: {e}"),
    };
    let label = crate::commands::disk_cleanup::format_bytes(bytes);
    if bytes == 0 {
        "**Debug Log:** empty or not created yet · `/logs` when lines appear.".to_string()
    } else {
        format!(
            "**Debug Log:** **{label}** on disk · `/logs` for tail · path in Settings → View logs."
        )
    }
}


/// Zero-LLM debug.log path (config only; no tail read).
pub fn format_debug_log_path_gateway() -> String {
    let path = crate::config::Config::log_file_path();
    let display = path.display().to_string();
    format!("**Debug Log:** `{display}` · Settings → View logs · `/logs` for tail.")
}


/// Zero-LLM debug.log age from file mtime (stat only; no tail read).
pub fn format_debug_log_age_gateway() -> String {
    match snapshot_debug_log_modified_ms() {
        Ok(None) => {
            "**Debug Log:** no file yet · logging starts on first write · `/logs` when lines appear."
                .to_string()
        }
        Ok(Some(ms)) => {
            let age = age_from_ms(ms);
            format!(
                "**Debug Log:** last write **{age}** ago · `/logs` for tail · Settings → View logs."
            )
        }
        Err(e) => format!("**Debug Log** — could not stat file: {e}"),
    }
}


/// Zero-LLM config.json + data-home paths (config only; no file read).
pub fn format_config_path_gateway() -> String {
    let config = crate::config::Config::config_file_path();
    let home = config
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "~/.mac-stats".into());
    let config_display = config.display().to_string();
    format!(
        "**Config:** `{config_display}` · data home `{home}` · Settings for credentials · `config size` for on-disk bytes · `config age` for last write · `/logs` for debug.log."
    )
}


/// Zero-LLM app config.json file size (stat only; no dump/edit).
pub fn format_config_size_gateway() -> String {
    let path = crate::config::Config::config_file_path();
    if !path.exists() {
        return "**Config:** no `config.json` yet · app will create it on first save · `config path` for the file · `config age` after first save.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Config:** empty `config.json` · `config path` for the file · `config age` for last write.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Config:** **{label}** on disk · app settings JSON · `config path` for the file · `config age` for last write · Settings for credentials."
            )
        }
        Err(e) => format!("**Config** — could not stat `config.json`: {e}"),
    }
}


/// Zero-LLM app config.json age from file mtime (stat only; no dump/edit).
pub fn format_config_age_gateway() -> String {
    let path = crate::config::Config::config_file_path();
    if !path.exists() {
        return "**Config:** no `config.json` yet · app will create it on first save · `config path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Config:** last write **{age}** ago · app settings JSON · `config path` for the file · `config size` for on-disk bytes · Settings for credentials."
            )
        }
        Err(e) => format!("**Config** — could not stat `config.json`: {e}"),
    }
}


/// Zero-LLM BROWSER_SCREENSHOT save directory (config only; no list/prune).
pub fn format_screenshots_path_gateway() -> String {
    let dir = crate::config::Config::screenshots_dir();
    let display = dir.display().to_string();
    format!(
        "**Screenshots:** `{display}` · BROWSER_SCREENSHOT saves PNG here · Disk Cleanup can prune old files · `screenshots size` for disk use · `screenshots age` for newest mtime."
    )
}


/// Zero-LLM screenshots directory size (recursive file bytes; no list dump).
pub fn format_screenshots_size_gateway() -> String {
    let dir = crate::config::Config::screenshots_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Screenshots:** not created yet · BROWSER_SCREENSHOT writes PNG here · `screenshot path` for the folder · `screenshots age` for newest mtime."
                .to_string()
        }
        Err(e) => format!("**Screenshots** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Screenshots:** empty · `screenshot path` for the folder · BROWSER_SCREENSHOT saves PNG here · `screenshots age` for newest mtime."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Screenshots:** **{label}** on disk ({files} files) · BROWSER_SCREENSHOT · `screenshot path` for the folder · `screenshots age` for newest mtime · does not list names."
            )
        }
    }
}


/// Zero-LLM screenshots directory age (newest file mtime; no list dump).
pub fn format_screenshots_age_gateway() -> String {
    let dir = crate::config::Config::screenshots_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Screenshots:** not created yet · BROWSER_SCREENSHOT writes PNG here · `screenshot path` for the folder · `screenshots size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**Screenshots** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Screenshots:** empty · `screenshot path` for the folder · BROWSER_SCREENSHOT saves PNG here · `screenshots size` for on-disk bytes."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Screenshots:** last write **{age}** ago · 1 file · BROWSER_SCREENSHOT · `screenshot path` for the folder · `screenshots size` for on-disk bytes · does not list names."
                )
            } else {
                format!(
                    "**Screenshots:** newest write **{age}** ago · {files} files · BROWSER_SCREENSHOT · `screenshot path` for the folder · `screenshots size` for on-disk bytes · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Screenshots** — could not read mtime · `screenshot path` for the folder · `screenshots size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM runs.jsonl file size (stat only; no list/count/prune).
pub fn format_runs_size_gateway() -> String {
    let path = crate::commands::run_telemetry::runs_jsonl_path();
    if !path.exists() {
        return "**Runs:** no `runs.jsonl` yet · `/insights` after the first chat turn · `runs path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Runs:** empty `runs.jsonl` · `runs path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Runs:** **{label}** on disk · `/insights` or Agent Ops → Runs · `runs path` for the file · `runs age` for last write."
            )
        }
        Err(e) => format!("**Runs** — could not stat `runs.jsonl`: {e}"),
    }
}


/// Zero-LLM runs.jsonl age from file mtime (stat only; no list/count/prune).
pub fn format_runs_age_gateway() -> String {
    let path = crate::commands::run_telemetry::runs_jsonl_path();
    if !path.exists() {
        return "**Runs:** no `runs.jsonl` yet · `/insights` after the first chat turn.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Runs:** last write **{age}** ago · `/insights` or Agent Ops → Runs · `runs path` for the file."
            )
        }
        Err(e) => format!("**Runs** — could not stat `runs.jsonl`: {e}"),
    }
}


/// Zero-LLM runs.jsonl path (config only; no list/count/prune).
pub fn format_runs_path_gateway() -> String {
    let path = crate::commands::run_telemetry::runs_jsonl_path();
    let display = path.display().to_string();
    format!(
        "**Runs:** `{display}` · Agent Ops → Runs · `/insights` · Disk Cleanup can prune old lines."
    )
}


/// Zero-LLM task directory age (newest file mtime; no list dump).
pub fn format_task_age_gateway() -> String {
    let dir = crate::config::Config::task_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Tasks:** not created yet · app recreates under `~/.mac-stats/task/` · `task path` for the folder · `task size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**Tasks** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Tasks:** empty · `task path` for the folder · `/tasks` for Active list · `task size` for on-disk bytes · no dump from this ask."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Tasks:** last write **{age}** ago · 1 file · `task path` for the folder · `task size` for on-disk bytes · `/tasks` for Active list · does not list names."
                )
            } else {
                format!(
                    "**Tasks:** newest write **{age}** ago · {files} files · `task path` for the folder · `task size` for on-disk bytes · `/tasks` for Active list · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Tasks** — could not read mtime · `task path` for the folder · `task size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM task directory size (recursive file bytes; no list dump).
pub fn format_task_size_gateway() -> String {
    let dir = crate::config::Config::task_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Tasks:** not created yet · app recreates under `~/.mac-stats/task/` · `task path` for the folder · `task age` for newest mtime."
                .to_string()
        }
        Err(e) => format!("**Tasks** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Tasks:** empty · `task path` for the folder · `/tasks` for Active list · `task age` for newest mtime · `TASK_CREATE:` to add one."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Tasks:** **{label}** on disk ({files} files) · `/tasks` for Active list · `task path` for the folder · `task age` for newest mtime · does not list names."
            )
        }
    }
}


/// Zero-LLM task directory path (config only; no list/create).
pub fn format_task_path_gateway() -> String {
    let dir = crate::config::Config::task_dir();
    let display = dir.display().to_string();
    format!(
        "**Tasks:** `{display}` · `/tasks` for Active list · `task size` for disk use · `task age` for newest mtime · `TASK_CREATE:` to add one."
    )
}


/// Zero-LLM memory notes directory size (recursive file bytes; no list dump).
pub fn format_memory_size_gateway() -> String {
    let dir = crate::config::Config::memory_notes_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Notes:** not created yet · app recreates under `~/.mac-stats/agents/notes/` · `memory path` / `notes path` for the folder."
                .to_string()
        }
        Err(e) => format!("**Notes** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Notes:** empty · `memory path` / `notes path` for the folder · `MEMORY: save <slug>` to add · does not list names."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Notes:** **{label}** on disk ({files} files) · `memory path` / `notes path` for the folder · `notes age` for last write · does not list names."
            )
        }
    }
}


/// Zero-LLM memory notes directory age (newest file mtime; no list dump).
pub fn format_memory_age_gateway() -> String {
    let dir = crate::config::Config::memory_notes_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Notes:** not created yet · app recreates under `~/.mac-stats/agents/notes/` · `memory path` / `notes path` for the folder."
                .to_string()
        }
        Err(e) => format!("**Notes** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Notes:** empty · `memory path` / `notes path` for the folder · `MEMORY: save <slug>` to add · does not list names."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Notes:** last write **{age}** ago · 1 file · `memory path` / `notes path` for the folder · `notes size` for disk use · does not list names."
                )
            } else {
                format!(
                    "**Notes:** newest write **{age}** ago · {files} files · `memory path` / `notes path` for the folder · `notes size` for disk use · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Notes** — could not read mtime under the notes folder.".to_string()
        }
    }
}


/// Zero-LLM memory notes dir + curated memory.md paths (config only; no list/read/save).
pub fn format_memory_path_gateway() -> String {
    let notes = crate::config::Config::memory_notes_dir()
        .display()
        .to_string();
    let curated = crate::config::Config::memory_file_path()
        .display()
        .to_string();
    format!(
        "**Memory:** notes `{notes}` · curated `{curated}` · `MEMORY: save <slug>` for verbatim · `scrub memory` to clean · `notes size` for disk use · `notes age` for last write · `memory.md path` for curated file only · `memory.md size` for curated bytes · `memory.md age` for last write."
    )
}


/// Zero-LLM curated memory.md file size (stat only; no dump/edit).
pub fn format_memory_md_size_gateway() -> String {
    let path = crate::config::Config::memory_file_path();
    if !path.exists() {
        return "**Curated memory:** no `memory.md` yet · `memory.md path` for the file · `memory.md age` for last write · `notes size` for the notes folder."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Curated memory:** empty `memory.md` · `memory.md path` for the file · `memory.md age` for last write."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Curated memory:** **{label}** on disk · shared lessons · `memory.md path` for the file · `memory.md age` for last write · does not dump memory text."
            )
        }
        Err(e) => format!("**Curated memory** — could not stat `memory.md`: {e}"),
    }
}


/// Zero-LLM curated memory.md age from file mtime (stat only; no dump/edit).
pub fn format_memory_md_age_gateway() -> String {
    let path = crate::config::Config::memory_file_path();
    if !path.exists() {
        return "**Curated memory:** no `memory.md` yet · `memory.md path` for the file · `notes size` for the notes folder."
            .to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Curated memory:** last write **{age}** ago · shared lessons · `memory.md path` for the file · `memory.md size` for on-disk bytes · does not dump memory text."
            )
        }
        Err(e) => format!("**Curated memory** — could not stat `memory.md`: {e}"),
    }
}


/// Zero-LLM curated memory.md path (config only; no dump/edit/scrub).
pub fn format_memory_md_path_gateway() -> String {
    let display = crate::config::Config::memory_file_path()
        .display()
        .to_string();
    format!(
        "**Curated memory:** `{display}` · shared lessons across agents · path only · `memory.md size` / `memory.md age` for bytes / mtime · does not dump or edit · `memory path` / `notes path` for the notes folder · `discord memory path` for channel files · `scrub memory` to clean."
    )
}


/// Zero-LLM Discord channel memory path (config only; no list/dump/scrub).
/// Size asks use the Discord channel memory size lane (v0.1.918).
/// Age asks use the Discord channel memory age lane (v0.1.948).
pub fn format_discord_memory_path_gateway() -> String {
    let agents = crate::config::Config::agents_dir().display().to_string();
    format!(
        "**Discord channel memory:** `{agents}/memory-discord-<channelId>.md` · under agents/ · path only · `discord memory size` for on-disk bytes · `discord memory age` for newest mtime · does not list or dump · `/knowledge discord` to list · `memory.md path` for curated · `memory path` for notes."
    )
}


/// Zero-LLM Discord channel memory file sizes (sum `memory-discord-*.md`; no list/dump).
pub fn format_discord_memory_size_gateway() -> String {
    let agents = crate::config::Config::agents_dir();
    let mut total: u64 = 0;
    let mut count: usize = 0;
    if let Ok(entries) = std::fs::read_dir(&agents) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("memory-discord-") || !name.ends_with(".md") {
                continue;
            }
            let path = entry.path();
            if let Ok(meta) = std::fs::metadata(&path) {
                if meta.is_file() {
                    total = total.saturating_add(meta.len());
                    count = count.saturating_add(1);
                }
            }
        }
    }
    if count == 0 {
        return "**Discord channel memory:** no `memory-discord-*.md` yet · `discord memory path` for the pattern · `discord memory age` for newest mtime · `/knowledge discord` to list · does not dump notes."
            .to_string();
    }
    let label = crate::commands::disk_cleanup::format_bytes(total);
    if count == 1 {
        format!(
            "**Discord channel memory:** **{label}** on disk · 1 `memory-discord-*.md` · `discord memory path` for the pattern · `discord memory age` for newest mtime · does not list or dump."
        )
    } else {
        format!(
            "**Discord channel memory:** **{label}** on disk · {count} `memory-discord-*.md` files · `discord memory path` for the pattern · `discord memory age` for newest mtime · does not list or dump."
        )
    }
}


/// Zero-LLM Discord channel memory age (newest `memory-discord-*.md` mtime; no list/dump).
pub fn format_discord_memory_age_gateway() -> String {
    let agents = crate::config::Config::agents_dir();
    let mut newest_ms: Option<u64> = None;
    let mut count: usize = 0;
    if let Ok(entries) = std::fs::read_dir(&agents) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("memory-discord-") || !name.ends_with(".md") {
                continue;
            }
            let path = entry.path();
            if let Ok(meta) = std::fs::metadata(&path) {
                if meta.is_file() {
                    count = count.saturating_add(1);
                    let ms = file_mtime_ms(&meta);
                    if ms > 0 {
                        newest_ms = Some(newest_ms.map_or(ms, |cur| cur.max(ms)));
                    }
                }
            }
        }
    }
    if count == 0 {
        return "**Discord channel memory:** no `memory-discord-*.md` yet · `discord memory path` for the pattern · `discord memory size` for on-disk bytes · `/knowledge discord` to list · does not dump notes."
            .to_string();
    }
    match newest_ms {
        Some(ms) => {
            let age = age_from_ms(ms);
            if count == 1 {
                format!(
                    "**Discord channel memory:** last write **{age}** ago · 1 `memory-discord-*.md` · `discord memory path` for the pattern · `discord memory size` for on-disk bytes · does not list or dump."
                )
            } else {
                format!(
                    "**Discord channel memory:** newest write **{age}** ago · {count} `memory-discord-*.md` files · `discord memory path` for the pattern · `discord memory size` for on-disk bytes · does not list or dump."
                )
            }
        }
        None => {
            "**Discord channel memory** — could not read mtime on `memory-discord-*.md` files."
                .to_string()
        }
    }
}


/// Zero-LLM session-memory file sizes (sum `session-memory-*.md`; no list/dump).
pub fn format_session_memory_size_gateway() -> String {
    let dir = crate::config::Config::session_dir();
    let mut total: u64 = 0;
    let mut count: usize = 0;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("session-memory-") || !name.ends_with(".md") {
                continue;
            }
            let path = entry.path();
            if let Ok(meta) = std::fs::metadata(&path) {
                if meta.is_file() {
                    total = total.saturating_add(meta.len());
                    count = count.saturating_add(1);
                }
            }
        }
    }
    if count == 0 {
        return "**Session memory:** no `session-memory-*.md` yet · `session memory path` for the pattern · `session memory age` for newest mtime · `/sessions` for Live/Files · does not dump transcripts."
            .to_string();
    }
    let label = crate::commands::disk_cleanup::format_bytes(total);
    if count == 1 {
        format!(
            "**Session memory:** **{label}** on disk · 1 `session-memory-*.md` · `session memory path` for the pattern · `session memory age` for newest mtime · does not list or dump."
        )
    } else {
        format!(
            "**Session memory:** **{label}** on disk · {count} `session-memory-*.md` files · `session memory path` for the pattern · `session memory age` for newest mtime · does not list or dump."
        )
    }
}


/// Zero-LLM session-memory age (newest `session-memory-*.md` mtime; no list/dump).
pub fn format_session_memory_age_gateway() -> String {
    let dir = crate::config::Config::session_dir();
    let mut newest_ms: Option<u64> = None;
    let mut count: usize = 0;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("session-memory-") || !name.ends_with(".md") {
                continue;
            }
            let path = entry.path();
            if let Ok(meta) = std::fs::metadata(&path) {
                if meta.is_file() {
                    count = count.saturating_add(1);
                    let ms = file_mtime_ms(&meta);
                    if ms > 0 {
                        newest_ms = Some(newest_ms.map_or(ms, |cur| cur.max(ms)));
                    }
                }
            }
        }
    }
    if count == 0 {
        return "**Session memory:** no `session-memory-*.md` yet · `session memory path` for the pattern · `session memory size` for on-disk bytes · `/sessions` for Live/Files · does not dump transcripts."
            .to_string();
    }
    match newest_ms {
        Some(ms) => {
            let age = age_from_ms(ms);
            if count == 1 {
                format!(
                    "**Session memory:** last write **{age}** ago · 1 `session-memory-*.md` · `session memory path` for the pattern · `session memory size` for on-disk bytes · does not list or dump."
                )
            } else {
                format!(
                    "**Session memory:** newest write **{age}** ago · {count} `session-memory-*.md` files · `session memory path` for the pattern · `session memory size` for on-disk bytes · does not list or dump."
                )
            }
        }
        None => {
            "**Session memory** — could not read mtime on `session-memory-*.md` files.".to_string()
        }
    }
}


/// Zero-LLM session-memory file path pattern (config only; no list/dump/prune).
pub fn format_session_memory_path_gateway() -> String {
    let dir = crate::config::Config::session_dir().display().to_string();
    format!(
        "**Session memory files:** `{dir}/session-memory-<id>-<timestamp>-<topic>.md` · under session/ · path only · `session memory size` for on-disk bytes · `session memory age` for newest mtime · does not list or dump · `session path` for the folder · `/sessions` for Live/Files."
    )
}


/// Zero-LLM LaunchAgent plist paths (config only; no load/unload/edit).
pub fn format_launchagent_path_gateway() -> String {
    let app = crate::config::Config::mac_stats_launch_agent_plist()
        .display()
        .to_string();
    let harness = crate::config::Config::overnight_harness_launch_agent_plist()
        .display()
        .to_string();
    format!(
        "**LaunchAgents:** `{app}` · app KeepAlive · overnight harness: `{harness}` · `launchagent size` for on-disk bytes · `launchagent age` for mtime · path only · does not load or unload."
    )
}


/// Zero-LLM LaunchAgent plist file sizes (stat only; no dump/load/unload).
pub fn format_launchagent_size_gateway() -> String {
    let app = crate::config::Config::mac_stats_launch_agent_plist();
    let harness = crate::config::Config::overnight_harness_launch_agent_plist();
    let app_line = format_one_plist_size_line("App LaunchAgent", &app);
    let harness_line = format_one_plist_size_line("Overnight harness", &harness);
    format!(
        "{app_line} · {harness_line} · `launchagent path` for paths · `launchagent age` for mtime · does not load, unload, or dump XML."
    )
}


/// Zero-LLM LaunchAgent plist ages (mtime only; no dump/load/unload).
pub fn format_launchagent_age_gateway() -> String {
    let app = crate::config::Config::mac_stats_launch_agent_plist();
    let harness = crate::config::Config::overnight_harness_launch_agent_plist();
    let app_line = format_one_plist_age_line("App LaunchAgent", &app);
    let harness_line = format_one_plist_age_line("Overnight harness", &harness);
    format!(
        "{app_line} · {harness_line} · `launchagent path` for paths · `launchagent size` for on-disk bytes · does not load, unload, or dump XML."
    )
}


/// Zero-LLM autoresearch results.tsv file size (stat only; no dump/list).
pub fn format_results_tsv_size_gateway() -> String {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return "**Autoresearch results:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Autoresearch results:** empty `results.tsv` · `results.tsv path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Autoresearch results:** **{label}** on disk · keep/discard log · `/keeps` · `keeps tonight` for counts · `results.tsv path` for the file · `results.tsv age` for last write."
            )
        }
        Err(e) => format!("**Autoresearch results** — could not stat `results.tsv`: {e}"),
    }
}


/// Zero-LLM autoresearch results.tsv age from file mtime (stat only; no dump/list).
pub fn format_results_tsv_age_gateway() -> String {
    let path = crate::config::Config::autoresearch_results_tsv();
    if !path.exists() {
        return "**Autoresearch results:** no `results.tsv` yet · overnight keep/discard will create it · `results.tsv path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Autoresearch results:** last write **{age}** ago · keep/discard log · `results.tsv path` for the file · `improvements path` for the parent folder."
            )
        }
        Err(e) => format!("**Autoresearch results** — could not stat `results.tsv`: {e}"),
    }
}


/// Zero-LLM autoresearch results.tsv path (config only; no dump/list).
pub fn format_results_tsv_path_gateway() -> String {
    let path = crate::config::Config::autoresearch_results_tsv();
    let display = path.display().to_string();
    format!(
        "**Autoresearch results:** `{display}` · keep/discard log · path only · does not dump rows · `/keeps` · `keeps tonight` for keep/discard counts · `improvements path` for the parent folder · `loop backlog path` for the tick log."
    )
}


/// Zero-LLM overnight loop_backlog.md path (config only; no dump/list).
pub fn format_loop_backlog_path_gateway() -> String {
    let path = crate::config::Config::loop_backlog_path();
    let display = path.display().to_string();
    format!(
        "**Loop backlog:** `{display}` · overnight harness tick log · path only · does not dump notes · `improvements path` for the parent folder · `results.tsv path` for keep/discard · `loop backlog size` for on-disk bytes · `loop backlog age` for last write."
    )
}


/// Zero-LLM overnight loop_backlog.md file size (stat only; no dump/list).
pub fn format_loop_backlog_size_gateway() -> String {
    let path = crate::config::Config::loop_backlog_path();
    if !path.exists() {
        return "**Loop backlog:** no `loop_backlog.md` yet · overnight ticks will create it · `loop backlog path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Loop backlog:** empty `loop_backlog.md` · `loop backlog path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Loop backlog:** **{label}** on disk · overnight harness tick log · `loop backlog path` for the file · `loop backlog age` for last write."
            )
        }
        Err(e) => format!("**Loop backlog** — could not stat `loop_backlog.md`: {e}"),
    }
}


/// Zero-LLM overnight loop_backlog.md age from file mtime (stat only; no dump/list).
pub fn format_loop_backlog_age_gateway() -> String {
    let path = crate::config::Config::loop_backlog_path();
    if !path.exists() {
        return "**Loop backlog:** no `loop_backlog.md` yet · overnight ticks will create it · `loop backlog path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Loop backlog:** last write **{age}** ago · overnight harness tick log · `loop backlog path` for the file · `loop backlog size` for on-disk bytes."
            )
        }
        Err(e) => format!("**Loop backlog** — could not stat `loop_backlog.md`: {e}"),
    }
}


/// Zero-LLM overnight sibling_harness.md path (config only; no dump/list).
pub fn format_sibling_harness_path_gateway() -> String {
    let path = crate::config::Config::sibling_harness_path();
    let display = path.display().to_string();
    format!(
        "**Sibling harness:** `{display}` · OpenClaw/Hermes overnight scan note · path only · does not dump notes · `improvements path` for the parent folder · `loop backlog path` for the tick log · `sibling harness size` for on-disk bytes · `sibling harness age` for last write."
    )
}


/// Zero-LLM overnight sibling_harness.md file size (stat only; no dump/list).
pub fn format_sibling_harness_size_gateway() -> String {
    let path = crate::config::Config::sibling_harness_path();
    if !path.exists() {
        return "**Sibling harness:** no `sibling_harness.md` yet · overnight sibling scan will create it · `sibling harness path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Sibling harness:** empty `sibling_harness.md` · `sibling harness path` for the file."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Sibling harness:** **{label}** on disk · OpenClaw/Hermes overnight scan note · `sibling harness path` for the file · `sibling harness age` for last write."
            )
        }
        Err(e) => format!("**Sibling harness** — could not stat `sibling_harness.md`: {e}"),
    }
}


/// Zero-LLM overnight sibling_harness.md age from file mtime (stat only; no dump/list).
pub fn format_sibling_harness_age_gateway() -> String {
    let path = crate::config::Config::sibling_harness_path();
    if !path.exists() {
        return "**Sibling harness:** no `sibling_harness.md` yet · overnight sibling scan will create it · `sibling harness path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Sibling harness:** last write **{age}** ago · OpenClaw/Hermes overnight scan note · `sibling harness path` for the file · `sibling harness size` for on-disk bytes."
            )
        }
        Err(e) => format!("**Sibling harness** — could not stat `sibling_harness.md`: {e}"),
    }
}


/// Zero-LLM overnight standing_backlog.md path (config only; no dump/list).
pub fn format_standing_backlog_path_gateway() -> String {
    let path = crate::config::Config::standing_backlog_path();
    let display = path.display().to_string();
    format!(
        "**Standing backlog:** `{display}` · overnight Track B fuel note · path only · does not dump notes · `improvements path` for the parent folder · `loop backlog path` for the tick log · `sibling harness path` for the sibling scan · `standing backlog size` for on-disk bytes · `standing backlog age` for last write."
    )
}


/// Zero-LLM overnight standing_backlog.md file size (stat only; no dump/list).
pub fn format_standing_backlog_size_gateway() -> String {
    let path = crate::config::Config::standing_backlog_path();
    if !path.exists() {
        return "**Standing backlog:** no `standing_backlog.md` yet · overnight Track B fuel will create it · `standing backlog path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Standing backlog:** empty `standing_backlog.md` · `standing backlog path` for the file."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Standing backlog:** **{label}** on disk · overnight Track B fuel note · `standing backlog path` for the file · `standing backlog age` for last write."
            )
        }
        Err(e) => format!("**Standing backlog** — could not stat `standing_backlog.md`: {e}"),
    }
}


/// Zero-LLM overnight standing_backlog.md age from file mtime (stat only; no dump/list).
pub fn format_standing_backlog_age_gateway() -> String {
    let path = crate::config::Config::standing_backlog_path();
    if !path.exists() {
        return "**Standing backlog:** no `standing_backlog.md` yet · overnight Track B fuel will create it · `standing backlog path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Standing backlog:** last write **{age}** ago · overnight Track B fuel note · `standing backlog path` for the file · `standing backlog size` for on-disk bytes."
            )
        }
        Err(e) => format!("**Standing backlog** — could not stat `standing_backlog.md`: {e}"),
    }
}


/// Zero-LLM today's morning_surprise_YYYY-MM-DD.md path (config only; no dump/list).
pub fn format_morning_surprise_path_gateway() -> String {
    let path = crate::config::Config::morning_surprise_path_today();
    let display = path.display().to_string();
    format!(
        "**Morning surprise:** `{display}` · today's overnight note · path only · does not dump highlights · `morning surprise size` for on-disk bytes · `morning surprise age` for last write · `improvements path` for the parent folder · `standing backlog path` for Track B fuel · ask *morning surprise?* for the ship list."
    )
}


/// Zero-LLM today's morning_surprise_YYYY-MM-DD.md file size (stat only; no dump/list).
pub fn format_morning_surprise_size_gateway() -> String {
    let path = crate::config::Config::morning_surprise_path_today();
    if !path.exists() {
        return "**Morning surprise:** no note for today yet · overnight Track B will create it · `morning surprise path` for today's file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Morning surprise:** empty note for today · `morning surprise path` for the file."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Morning surprise:** **{label}** on disk · today's overnight note · `morning surprise path` for the file · `morning surprise age` for last write · ask *morning surprise?* for the ship list."
            )
        }
        Err(e) => format!("**Morning surprise** — could not stat today's note: {e}"),
    }
}


/// Zero-LLM today's morning_surprise_YYYY-MM-DD.md age from file mtime (stat only; no dump/list).
pub fn format_morning_surprise_age_gateway() -> String {
    let path = crate::config::Config::morning_surprise_path_today();
    if !path.exists() {
        return "**Morning surprise:** no note for today yet · overnight Track B will create it · `morning surprise path` for today's file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Morning surprise:** last write **{age}** ago · today's overnight note · `morning surprise path` for the file · `morning surprise size` for on-disk bytes · ask *morning surprise?* for the ship list."
            )
        }
        Err(e) => format!("**Morning surprise** — could not stat today's note: {e}"),
    }
}


/// Zero-LLM overnight_agent.log path (config only; no dump/tail).
pub fn format_overnight_agent_log_path_gateway() -> String {
    let path = crate::config::Config::overnight_agent_log_path();
    let display = path.display().to_string();
    format!(
        "**Overnight agent log:** `{display}` · Track B harness transcript · path only · does not dump or tail · `overnight agent log size` for on-disk bytes · `overnight agent log age` for last write · `debug.log path` for the app log · `improvements path` for the parent folder."
    )
}


/// Zero-LLM overnight_agent.log file size (stat only; no dump/tail).
pub fn format_overnight_agent_log_size_gateway() -> String {
    let path = crate::config::Config::overnight_agent_log_path();
    if !path.exists() {
        return "**Overnight agent log:** no file yet · Track B creates it on harness ticks · `overnight agent log path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Overnight agent log:** empty · `overnight agent log path` for the file."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Overnight agent log:** **{label}** on disk · Track B harness transcript · `overnight agent log path` for the file · `overnight agent log age` for last write · `debug.log size` for the app log."
            )
        }
        Err(e) => format!("**Overnight agent log** — could not stat file: {e}"),
    }
}


/// Zero-LLM overnight_agent.log age from file mtime (stat only; no dump/tail).
pub fn format_overnight_agent_log_age_gateway() -> String {
    let path = crate::config::Config::overnight_agent_log_path();
    if !path.exists() {
        return "**Overnight agent log:** no file yet · Track B creates it on harness ticks · `overnight agent log path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Overnight agent log:** last write **{age}** ago · Track B harness transcript · `overnight agent log path` for the file · `overnight agent log size` for on-disk bytes · `debug.log age` for the app log."
            )
        }
        Err(e) => format!("**Overnight agent log** — could not stat file: {e}"),
    }
}


/// Zero-LLM overnight_harness_loop.stdout.log path (config only; no dump/tail).
pub fn format_harness_loop_stdout_path_gateway() -> String {
    let path = crate::config::Config::overnight_harness_loop_stdout_log_path();
    let display = path.display().to_string();
    format!(
        "**Harness loop stdout:** `{display}` · Track B loop capture · path only · does not dump or tail · `harness loop stdout size` for on-disk bytes · `harness loop stdout age` for last write · `harness loop stderr path` for stderr · `overnight agent log path` for the agent transcript · `debug.log path` for the app log · `improvements path` for the parent folder."
    )
}


/// Zero-LLM overnight_harness_loop.stdout.log file size (stat only; no dump/tail).
pub fn format_harness_loop_stdout_size_gateway() -> String {
    let path = crate::config::Config::overnight_harness_loop_stdout_log_path();
    if !path.exists() {
        return "**Harness loop stdout:** no file yet · Track B creates it on harness ticks · `harness loop stdout path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Harness loop stdout:** empty · `harness loop stdout path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Harness loop stdout:** **{label}** on disk · Track B loop capture · `harness loop stdout path` for the file · `harness loop stdout age` for last write · `overnight agent log size` for the agent transcript · `debug.log size` for the app log."
            )
        }
        Err(e) => format!("**Harness loop stdout** — could not stat file: {e}"),
    }
}


/// Zero-LLM overnight_harness_loop.stdout.log age from file mtime (stat only; no dump/tail).
pub fn format_harness_loop_stdout_age_gateway() -> String {
    let path = crate::config::Config::overnight_harness_loop_stdout_log_path();
    if !path.exists() {
        return "**Harness loop stdout:** no file yet · Track B creates it on harness ticks · `harness loop stdout path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Harness loop stdout:** last write **{age}** ago · Track B loop capture · `harness loop stdout path` for the file · `harness loop stdout size` for on-disk bytes · `overnight agent log age` for the agent transcript · `debug.log age` for the app log."
            )
        }
        Err(e) => format!("**Harness loop stdout** — could not stat file: {e}"),
    }
}


/// Zero-LLM overnight_harness_loop.stderr.log path (config only; no dump/tail).
pub fn format_harness_loop_stderr_path_gateway() -> String {
    let path = crate::config::Config::overnight_harness_loop_stderr_log_path();
    let display = path.display().to_string();
    format!(
        "**Harness loop stderr:** `{display}` · Track B loop capture · path only · does not dump or tail · `harness loop stderr size` for on-disk bytes · `harness loop stderr age` for last write · `harness loop stdout path` for stdout · `overnight agent log path` for the agent transcript · `debug.log path` for the app log · `improvements path` for the parent folder."
    )
}


/// Zero-LLM overnight_harness_loop.stderr.log file size (stat only; no dump/tail).
pub fn format_harness_loop_stderr_size_gateway() -> String {
    let path = crate::config::Config::overnight_harness_loop_stderr_log_path();
    if !path.exists() {
        return "**Harness loop stderr:** no file yet · Track B creates it on harness ticks · `harness loop stderr path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Harness loop stderr:** empty · `harness loop stderr path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Harness loop stderr:** **{label}** on disk · Track B loop capture · `harness loop stderr path` for the file · `harness loop stderr age` for last write · `overnight agent log size` for the agent transcript · `debug.log size` for the app log."
            )
        }
        Err(e) => format!("**Harness loop stderr** — could not stat file: {e}"),
    }
}


/// Zero-LLM overnight_harness_loop.stderr.log age from file mtime (stat only; no dump/tail).
pub fn format_harness_loop_stderr_age_gateway() -> String {
    let path = crate::config::Config::overnight_harness_loop_stderr_log_path();
    if !path.exists() {
        return "**Harness loop stderr:** no file yet · Track B creates it on harness ticks · `harness loop stderr path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Harness loop stderr:** last write **{age}** ago · Track B loop capture · `harness loop stderr path` for the file · `harness loop stderr size` for on-disk bytes · `overnight agent log age` for the agent transcript · `debug.log age` for the app log."
            )
        }
        Err(e) => format!("**Harness loop stderr** — could not stat file: {e}"),
    }
}


/// Zero-LLM launchd.stderr.log path (config only; no dump/tail).
pub fn format_launchd_stderr_path_gateway() -> String {
    let path = crate::config::Config::launchd_stderr_log_path();
    let display = path.display().to_string();
    format!(
        "**Launchd stderr:** `{display}` · app LaunchAgent stderr redirect · path only · does not dump or tail · `launchd stderr size` for on-disk bytes · `launchd stderr age` for last write · `launchd stdout path` for stdout · `harness loop stderr path` for Track B loop stderr · `debug.log path` for the app log · `launchagent path` for the plists."
    )
}


/// Zero-LLM launchd.stderr.log file size (stat only; no dump/tail).
pub fn format_launchd_stderr_size_gateway() -> String {
    let path = crate::config::Config::launchd_stderr_log_path();
    if !path.exists() {
        return "**Launchd stderr:** no file yet · LaunchAgent creates it when the app runs · `launchd stderr path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Launchd stderr:** empty · `launchd stderr path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Launchd stderr:** **{label}** on disk · app LaunchAgent stderr redirect · `launchd stderr path` for the file · `launchd stderr age` for last write · `harness loop stderr size` for Track B loop stderr · `debug.log size` for the app log."
            )
        }
        Err(e) => format!("**Launchd stderr** — could not stat file: {e}"),
    }
}


/// Zero-LLM launchd.stderr.log age from file mtime (stat only; no dump/tail).
pub fn format_launchd_stderr_age_gateway() -> String {
    let path = crate::config::Config::launchd_stderr_log_path();
    if !path.exists() {
        return "**Launchd stderr:** no file yet · LaunchAgent creates it when the app runs · `launchd stderr path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Launchd stderr:** last write **{age}** ago · app LaunchAgent stderr redirect · `launchd stderr path` for the file · `launchd stderr size` for on-disk bytes · `harness loop stderr age` for Track B loop stderr · `debug.log age` for the app log."
            )
        }
        Err(e) => format!("**Launchd stderr** — could not stat file: {e}"),
    }
}


/// Zero-LLM launchd.stdout.log path (config only; no dump/tail).
pub fn format_launchd_stdout_path_gateway() -> String {
    let path = crate::config::Config::launchd_stdout_log_path();
    let display = path.display().to_string();
    format!(
        "**Launchd stdout:** `{display}` · app LaunchAgent stdout redirect · path only · does not dump or tail · `launchd stdout size` for on-disk bytes · `launchd stdout age` for last write · `launchd stderr path` for stderr · `harness loop stdout path` for Track B loop stdout · `debug.log path` for the app log · `launchagent path` for the plists."
    )
}


/// Zero-LLM launchd.stdout.log file size (stat only; no dump/tail).
pub fn format_launchd_stdout_size_gateway() -> String {
    let path = crate::config::Config::launchd_stdout_log_path();
    if !path.exists() {
        return "**Launchd stdout:** no file yet · LaunchAgent creates it when the app runs · `launchd stdout path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Launchd stdout:** empty · `launchd stdout path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Launchd stdout:** **{label}** on disk · app LaunchAgent stdout redirect · `launchd stdout path` for the file · `launchd stdout age` for last write · `harness loop stdout size` for Track B loop stdout · `debug.log size` for the app log."
            )
        }
        Err(e) => format!("**Launchd stdout** — could not stat file: {e}"),
    }
}


/// Zero-LLM launchd.stdout.log age from file mtime (stat only; no dump/tail).
pub fn format_launchd_stdout_age_gateway() -> String {
    let path = crate::config::Config::launchd_stdout_log_path();
    if !path.exists() {
        return "**Launchd stdout:** no file yet · LaunchAgent creates it when the app runs · `launchd stdout path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Launchd stdout:** last write **{age}** ago · app LaunchAgent stdout redirect · `launchd stdout path` for the file · `launchd stdout size` for on-disk bytes · `harness loop stdout age` for Track B loop stdout · `debug.log age` for the app log."
            )
        }
        Err(e) => format!("**Launchd stdout** — could not stat file: {e}"),
    }
}


/// Zero-LLM session directory age (newest file mtime; no list dump).
pub fn format_session_age_gateway() -> String {
    let dir = crate::config::Config::session_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Sessions:** not created yet · app recreates under `~/.mac-stats/session/` · `session path` for the folder · `session size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**Sessions** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Sessions:** empty · `session path` for the folder · `/sessions` for Live/Files · `session size` for on-disk bytes · no dump from this ask."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Sessions:** last write **{age}** ago · 1 file · `session path` for the folder · `session size` for on-disk bytes · `/sessions` for Live/Files · does not list names."
                )
            } else {
                format!(
                    "**Sessions:** newest write **{age}** ago · {files} files · `session path` for the folder · `session size` for on-disk bytes · `/sessions` for Live/Files · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Sessions** — could not read mtime · `session path` for the folder · `session size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM session directory size (recursive file bytes; no list dump).
pub fn format_session_size_gateway() -> String {
    let dir = crate::config::Config::session_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Sessions:** not created yet · app recreates under `~/.mac-stats/session/` · `session path` for the folder · `session age` for newest mtime."
                .to_string()
        }
        Err(e) => format!("**Sessions** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Sessions:** empty · `session path` for the folder · `/sessions` for Live/Files · `session age` for newest mtime · Agent Ops → Sessions."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Sessions:** **{label}** on disk ({files} files) · `/sessions` for Live/Files · `session path` for the folder · `session age` for newest mtime · does not list names."
            )
        }
    }
}


/// Zero-LLM session directory path (config only; no list/resume).
pub fn format_session_path_gateway() -> String {
    let dir = crate::config::Config::session_dir();
    let display = dir.display().to_string();
    format!(
        "**Sessions:** `{display}` · `/sessions` for Live/Files · `session size` for disk use · `session age` for newest mtime · Agent Ops → Sessions."
    )
}


/// Zero-LLM agents directory size (recursive file bytes; no list dump).
pub fn format_agents_size_gateway() -> String {
    let dir = crate::config::Config::agents_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Agents:** not created yet · app recreates under `~/.mac-stats/agents/` · `agents path` for the folder · `agents age` for newest mtime."
                .to_string()
        }
        Err(e) => format!("**Agents** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Agents:** empty · `agents path` for the folder · `/agents` for On/Off · Agent Ops → Agents · `agents age` for newest mtime."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Agents:** **{label}** on disk ({files} files) · `/agents` for On/Off · `agents path` for the folder · `agents age` for newest mtime · does not list names."
            )
        }
    }
}


/// Zero-LLM agents directory age (newest file mtime; no list dump).
pub fn format_agents_age_gateway() -> String {
    let dir = crate::config::Config::agents_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Agents:** not created yet · app recreates under `~/.mac-stats/agents/` · `agents path` for the folder · `agents size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**Agents** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Agents:** empty · `agents path` for the folder · `/agents` for On/Off · Agent Ops → Agents · `agents size` for on-disk bytes."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Agents:** last write **{age}** ago · 1 file · `/agents` for On/Off · `agents path` for the folder · `agents size` for on-disk bytes · does not list names."
                )
            } else {
                format!(
                    "**Agents:** newest write **{age}** ago · {files} files · `/agents` for On/Off · `agents path` for the folder · `agents size` for on-disk bytes · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Agents** — could not read mtime · `agents path` for the folder · `agents size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM agents directory path (config only; no list/create).
pub fn format_agents_path_gateway() -> String {
    let dir = crate::config::Config::agents_dir();
    let display = dir.display().to_string();
    format!(
        "**Agents dir:** `{display}` · `/agents` for On/Off · Agent Ops → Agents · `agents size` for disk use · `agents age` for newest mtime."
    )
}


/// Zero-LLM skills directory size (recursive file bytes; no list dump).
pub fn format_skills_size_gateway() -> String {
    let dir = crate::config::Config::skills_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Skills:** not created yet · app recreates under skills/ · `skills path` for the folder · `skills age` for newest mtime."
                .to_string()
        }
        Err(e) => format!("**Skills** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Skills:** empty · `skills path` for the folder · `/skills` for catalog · `SKILL: <n|topic>` to run · `skills age` for newest mtime."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Skills:** **{label}** on disk ({files} files) · `/skills` for catalog · `skills path` for the folder · `skills age` for newest mtime · does not list names."
            )
        }
    }
}


/// Zero-LLM skills directory age (newest file mtime; no list dump).
pub fn format_skills_age_gateway() -> String {
    let dir = crate::config::Config::skills_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Skills:** not created yet · app recreates under skills/ · `skills path` for the folder · `skills size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**Skills** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Skills:** empty · `skills path` for the folder · `/skills` for catalog · `SKILL: <n|topic>` to run · `skills size` for on-disk bytes."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Skills:** last write **{age}** ago · 1 file · `/skills` for catalog · `skills path` for the folder · `skills size` for on-disk bytes · does not list names."
                )
            } else {
                format!(
                    "**Skills:** newest write **{age}** ago · {files} files · `/skills` for catalog · `skills path` for the folder · `skills size` for on-disk bytes · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Skills** — could not read mtime · `skills path` for the folder · `skills size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM skills directory path (config only; no list/run).
pub fn format_skills_path_gateway() -> String {
    let dir = crate::config::Config::skills_dir();
    let display = dir.display().to_string();
    format!(
        "**Skills dir:** `{display}` · `/skills` for catalog · `SKILL: <n|topic>` to run · `skills size` for disk use · `skills age` for newest mtime."
    )
}


/// Zero-LLM plugins/scripts directory size (recursive file bytes; no list dump).
pub fn format_plugins_size_gateway() -> String {
    let dir = crate::config::Config::scripts_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Plugins/scripts:** not created yet · app recreates under scripts/ · `plugins path` for the folder · `plugins age` for newest mtime."
                .to_string()
        }
        Err(e) => format!("**Plugins/scripts** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Plugins/scripts:** empty · `plugins path` for the folder · `/plugins` for On/Off · `plugins age` for newest mtime · no run from this ask."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Plugins/scripts:** **{label}** on disk ({files} files) · `/plugins` for On/Off · `plugins path` for the folder · `plugins age` for newest mtime · does not list names."
            )
        }
    }
}


/// Zero-LLM plugins/scripts directory age (newest file mtime; no list dump).
pub fn format_plugins_age_gateway() -> String {
    let dir = crate::config::Config::scripts_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Plugins/scripts:** not created yet · app recreates under scripts/ · `plugins path` for the folder · `plugins size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**Plugins/scripts** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Plugins/scripts:** empty · `plugins path` for the folder · `/plugins` for On/Off · `plugins size` for on-disk bytes · no run from this ask."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Plugins/scripts:** last write **{age}** ago · 1 file · `/plugins` for On/Off · `plugins path` for the folder · `plugins size` for on-disk bytes · does not list names."
                )
            } else {
                format!(
                    "**Plugins/scripts:** newest write **{age}** ago · {files} files · `/plugins` for On/Off · `plugins path` for the folder · `plugins size` for on-disk bytes · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Plugins/scripts** — could not read mtime · `plugins path` for the folder · `plugins size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM plugins/scripts directory path (config only; no list/run).
/// Registered plugins point at script paths; agent scripts live under `scripts/`.
pub fn format_plugins_path_gateway() -> String {
    let dir = crate::config::Config::scripts_dir();
    let display = dir.display().to_string();
    format!(
        "**Plugins/scripts dir:** `{display}` · `/plugins` for On/Off list · `plugins size` for disk use · `plugins age` for newest mtime · no run from this ask."
    )
}


/// Zero-LLM prompts directory age (newest file mtime; no list dump).
pub fn format_prompts_age_gateway() -> String {
    let dir = crate::config::Config::prompts_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Prompts:** not created yet · app recreates under agents/prompts/ · `prompts path` for the folder · `prompts size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**Prompts** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Prompts:** empty · `prompts path` for the folder · planning + execution `.md` · `prompts size` for on-disk bytes · no dump from this ask."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Prompts:** last write **{age}** ago · 1 file · `prompts path` for the folder · `prompts size` for on-disk bytes · planning + execution `.md` · does not list names."
                )
            } else {
                format!(
                    "**Prompts:** newest write **{age}** ago · {files} files · `prompts path` for the folder · `prompts size` for on-disk bytes · planning + execution `.md` · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Prompts** — could not read mtime · `prompts path` for the folder · `prompts size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM prompts directory size (recursive file bytes; no list dump).
pub fn format_prompts_size_gateway() -> String {
    let dir = crate::config::Config::prompts_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Prompts:** not created yet · app recreates under agents/prompts/ · `prompts path` for the folder · `prompts age` for newest mtime."
                .to_string()
        }
        Err(e) => format!("**Prompts** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Prompts:** empty · `prompts path` for the folder · planning + execution `.md` · `prompts age` for newest mtime · no dump from this ask."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Prompts:** **{label}** on disk ({files} files) · `prompts path` for the folder · `prompts age` for newest mtime · planning + execution `.md` · does not list names."
            )
        }
    }
}


/// Zero-LLM prompts directory path (config only; no open/edit).
pub fn format_prompts_path_gateway() -> String {
    let dir = crate::config::Config::prompts_dir();
    let display = dir.display().to_string();
    format!(
        "**Prompts dir:** `{display}` · planning + execution `.md` · `prompts size` for disk use · `prompts age` for newest mtime · edit files on disk · `/agents` for soul/skill."
    )
}


/// Zero-LLM tmp directory age (newest file mtime; no list dump).
pub fn format_tmp_age_gateway() -> String {
    let dir = crate::config::Config::tmp_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Tmp:** not created yet · app recreates under `~/.mac-stats/tmp/` · `tmp path` for the folder · `tmp size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**Tmp** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Tmp:** empty · `tmp path` for the folder · `tmp size` for on-disk bytes · safe to delete contents · no dump from this ask."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Tmp:** last write **{age}** ago · 1 file · `tmp path` for the folder · `tmp size` for on-disk bytes · does not list names."
                )
            } else {
                format!(
                    "**Tmp:** newest write **{age}** ago · {files} files · `tmp path` for the folder · `tmp size` for on-disk bytes · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Tmp** — could not read mtime · `tmp path` for the folder · `tmp size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM tmp directory path (config only; no list/prune).
pub fn format_tmp_path_gateway() -> String {
    let dir = crate::config::Config::tmp_dir();
    let js = crate::config::Config::tmp_js_dir();
    let display = dir.display().to_string();
    let js_display = js.display().to_string();
    format!(
        "**Tmp dir:** `{display}` · JS scratch: `{js_display}` · safe to delete contents · app recreates as needed · `tmp size` for disk use · `tmp age` for newest mtime."
    )
}


/// Zero-LLM tmp directory size (recursive file bytes; no list dump).
pub fn format_tmp_size_gateway() -> String {
    let dir = crate::config::Config::tmp_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Tmp:** not created yet · app recreates under `~/.mac-stats/tmp/` · `tmp path` for the folder · `tmp age` for newest mtime."
                .to_string()
        }
        Err(e) => format!("**Tmp** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Tmp:** empty · `tmp path` for the folder · `tmp age` for newest mtime · safe to delete contents · app recreates as needed."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Tmp:** **{label}** on disk ({files} files) · scratch · `tmp path` for the folder · `tmp age` for newest mtime · does not list names."
            )
        }
    }
}


/// Zero-LLM uploads directory age (newest file mtime; no list dump).
pub fn format_uploads_age_gateway() -> String {
    let dir = crate::config::Config::browser_uploads_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Uploads:** not created yet · app recreates under `~/.mac-stats/uploads/` · `uploads path` for the folder · `uploads size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**Uploads** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Uploads:** empty · `uploads path` for the folder · `uploads size` for on-disk bytes · BROWSER_UPLOAD roots · no dump from this ask."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Uploads:** last write **{age}** ago · 1 file · `uploads path` for the folder · `uploads size` for on-disk bytes · does not list names."
                )
            } else {
                format!(
                    "**Uploads:** newest write **{age}** ago · {files} files · `uploads path` for the folder · `uploads size` for on-disk bytes · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Uploads** — could not read mtime · `uploads path` for the folder · `uploads size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM uploads directory size (recursive file bytes; no list dump).
pub fn format_uploads_size_gateway() -> String {
    let dir = crate::config::Config::browser_uploads_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Uploads:** not created yet · app recreates under `~/.mac-stats/uploads/` · `uploads path` for the folder · `uploads age` for newest mtime."
                .to_string()
        }
        Err(e) => format!("**Uploads** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Uploads:** empty · `uploads path` for the folder · `uploads age` for newest mtime · BROWSER_UPLOAD roots · `/browser` for CDP status."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Uploads:** **{label}** on disk ({files} files) · BROWSER_UPLOAD · `uploads path` for the folder · `uploads age` for newest mtime · does not list names."
            )
        }
    }
}


/// Zero-LLM uploads directory path (config only; no list/upload).
pub fn format_uploads_path_gateway() -> String {
    let dir = crate::config::Config::browser_uploads_dir();
    let display = dir.display().to_string();
    format!(
        "**Uploads dir:** `{display}` · BROWSER_UPLOAD roots · put files here · `/browser` for CDP status · `uploads size` for disk use · `uploads age` for newest mtime."
    )
}


/// Zero-LLM CDP traces directory age (newest file mtime; no list dump).
pub fn format_traces_age_gateway() -> String {
    let dir = crate::config::Config::browser_cdp_traces_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Traces:** not created yet · app recreates under `~/.mac-stats/traces/` · `traces path` for the folder · `traces size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**Traces** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Traces:** empty · `traces path` for the folder · `traces size` for on-disk bytes · CDP `*_cdp_trace.json` · `/browser` for CDP status · no dump from this ask."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Traces:** last write **{age}** ago · 1 file · `traces path` for the folder · `traces size` for on-disk bytes · does not list names."
                )
            } else {
                format!(
                    "**Traces:** newest write **{age}** ago · {files} files · `traces path` for the folder · `traces size` for on-disk bytes · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Traces** — could not read mtime · `traces path` for the folder · `traces size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM CDP traces directory size (recursive file bytes; no list dump).
pub fn format_traces_size_gateway() -> String {
    let dir = crate::config::Config::browser_cdp_traces_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Traces:** not created yet · app recreates under `~/.mac-stats/traces/` · `traces path` for the folder · `traces age` for newest mtime."
                .to_string()
        }
        Err(e) => format!("**Traces** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Traces:** empty · `traces path` for the folder · `traces age` for newest mtime · CDP `*_cdp_trace.json` · `/browser` for CDP status."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Traces:** **{label}** on disk ({files} files) · CDP · `traces path` for the folder · `traces age` for newest mtime · does not list names."
            )
        }
    }
}


/// Zero-LLM CDP traces directory path (config only; no list/prune).
pub fn format_traces_path_gateway() -> String {
    let dir = crate::config::Config::browser_cdp_traces_dir();
    let display = dir.display().to_string();
    format!(
        "**Traces dir:** `{display}` · CDP `*_cdp_trace.json` · pruned by retention · `/browser` for CDP status · `traces size` for disk use · `traces age` for newest mtime."
    )
}


/// Zero-LLM PDF exports directory age (newest file mtime; no list dump).
pub fn format_pdfs_age_gateway() -> String {
    let dir = crate::config::Config::pdfs_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**PDFs:** not created yet · app recreates under `~/.mac-stats/pdfs/` · `pdfs path` for the folder · `pdfs size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**PDFs** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**PDFs:** empty · `pdfs path` for the folder · `pdfs size` for on-disk bytes · BROWSER_SAVE_PDF exports · `/browser` for CDP status · no dump from this ask."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**PDFs:** last write **{age}** ago · 1 file · `pdfs path` for the folder · `pdfs size` for on-disk bytes · does not list names."
                )
            } else {
                format!(
                    "**PDFs:** newest write **{age}** ago · {files} files · `pdfs path` for the folder · `pdfs size` for on-disk bytes · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**PDFs** — could not read mtime · `pdfs path` for the folder · `pdfs size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM PDF exports directory size (recursive file bytes; no list dump).
pub fn format_pdfs_size_gateway() -> String {
    let dir = crate::config::Config::pdfs_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**PDFs:** not created yet · app recreates under `~/.mac-stats/pdfs/` · `pdfs path` for the folder · `pdfs age` for newest mtime."
                .to_string()
        }
        Err(e) => format!("**PDFs** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**PDFs:** empty · `pdfs path` for the folder · `pdfs age` for newest mtime · BROWSER_SAVE_PDF exports · `/browser` for CDP status."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**PDFs:** **{label}** on disk ({files} files) · exports · `pdfs path` for the folder · `pdfs age` for newest mtime · does not list names."
            )
        }
    }
}


/// Zero-LLM PDF exports directory path (config only; no list/prune/save).
pub fn format_pdfs_path_gateway() -> String {
    let dir = crate::config::Config::pdfs_dir();
    let display = dir.display().to_string();
    format!(
        "**PDFs dir:** `{display}` · BROWSER_SAVE_PDF exports · pruned by retention · `/browser` for CDP status · `pdfs size` for disk use · `pdfs age` for newest mtime."
    )
}


/// Zero-LLM browser credentials file path (config only; no list/edit/dump secrets).
pub fn format_browser_credentials_path_gateway() -> String {
    let path = crate::config::Config::browser_credentials_toml_path();
    let display = path.display().to_string();
    format!(
        "**Browser credentials:** `{display}` · BROWSER_INPUT `<secret>…</secret>` · edit the TOML · `browser credentials size` for on-disk bytes · `browser credentials age` for last write · `/browser` for CDP status."
    )
}


/// Zero-LLM browser-credentials.toml file size (stat only; no dump/list/edit secrets).
pub fn format_browser_credentials_size_gateway() -> String {
    let path = crate::config::Config::browser_credentials_toml_path();
    if !path.exists() {
        return "**Browser credentials:** no `browser-credentials.toml` yet · `browser credentials path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Browser credentials:** empty `browser-credentials.toml` · `browser credentials path` for the file."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Browser credentials:** **{label}** on disk · BROWSER_INPUT secrets TOML · `browser credentials path` for the file · `browser credentials age` for last write · does not dump secrets or edit the file."
            )
        }
        Err(e) => {
            format!("**Browser credentials** — could not stat `browser-credentials.toml`: {e}")
        }
    }
}


/// Zero-LLM browser-credentials.toml age from file mtime (stat only; no dump/list secrets).
pub fn format_browser_credentials_age_gateway() -> String {
    let path = crate::config::Config::browser_credentials_toml_path();
    if !path.exists() {
        return "**Browser credentials:** no `browser-credentials.toml` yet · `browser credentials path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Browser credentials:** last write **{age}** ago · BROWSER_INPUT secrets TOML · `browser credentials path` for the file · `browser credentials size` for on-disk bytes."
            )
        }
        Err(e) => {
            format!("**Browser credentials** — could not stat `browser-credentials.toml`: {e}")
        }
    }
}


/// Zero-LLM browser_storage_state.json file size (stat only; no dump/list/clear cookies).
pub fn format_browser_storage_state_size_gateway() -> String {
    let path = crate::config::Config::browser_storage_state_json_path();
    if !path.exists() {
        return "**Browser storage state:** no `browser_storage_state.json` yet · `storage state path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Browser storage state:** empty `browser_storage_state.json` · `storage state path` for the file · `storage state age` for last write."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Browser storage state:** **{label}** on disk · CDP cookie jar · `storage state path` for the file · `storage state age` for last write · does not dump cookies or clear the jar."
            )
        }
        Err(e) => {
            format!(
                "**Browser storage state** — could not stat `browser_storage_state.json`: {e}"
            )
        }
    }
}


/// Zero-LLM browser_storage_state.json age from file mtime (stat only; no dump/list cookies).
pub fn format_browser_storage_state_age_gateway() -> String {
    let path = crate::config::Config::browser_storage_state_json_path();
    if !path.exists() {
        return "**Browser storage state:** no `browser_storage_state.json` yet · `storage state path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Browser storage state:** last write **{age}** ago · CDP cookie jar · `storage state path` for the file · `storage state size` for on-disk bytes."
            )
        }
        Err(e) => {
            format!(
                "**Browser storage state** — could not stat `browser_storage_state.json`: {e}"
            )
        }
    }
}


/// Zero-LLM browser cookie-jar / storage-state path (config only; no list/dump/clear).
pub fn format_browser_storage_state_path_gateway() -> String {
    let path = crate::config::Config::browser_storage_state_json_path();
    let display = path.display().to_string();
    format!(
        "**Browser storage state:** `{display}` · CDP cookie jar · path only · `storage state size` for on-disk bytes · `storage state age` for last write · `/browser` for CDP status."
    )
}


/// Zero-LLM browser downloads directory age (newest file mtime; no list dump).
pub fn format_browser_downloads_age_gateway() -> String {
    let dir = crate::config::Config::browser_downloads_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Browser downloads:** not created yet · app recreates under `~/.mac-stats/browser-downloads/` · `browser downloads path` for the folder · `browser downloads size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**Browser downloads** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Browser downloads:** empty · `browser downloads path` for the folder · `browser downloads size` for on-disk bytes · BROWSER_DOWNLOAD artifacts · `/browser` for CDP status · no dump from this ask."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Browser downloads:** last write **{age}** ago · 1 file · `browser downloads path` for the folder · `browser downloads size` for on-disk bytes · does not list names."
                )
            } else {
                format!(
                    "**Browser downloads:** newest write **{age}** ago · {files} files · `browser downloads path` for the folder · `browser downloads size` for on-disk bytes · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Browser downloads** — could not read mtime · `browser downloads path` for the folder · `browser downloads size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM browser downloads directory size (recursive file bytes; no list dump).
pub fn format_browser_downloads_size_gateway() -> String {
    let dir = crate::config::Config::browser_downloads_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Browser downloads:** not created yet · app recreates under `~/.mac-stats/browser-downloads/` · `browser downloads path` for the folder · `browser downloads age` for newest mtime."
                .to_string()
        }
        Err(e) => format!("**Browser downloads** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Browser downloads:** empty · `browser downloads path` for the folder · `browser downloads age` for newest mtime · BROWSER_DOWNLOAD artifacts · `/browser` for CDP status."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Browser downloads:** **{label}** on disk ({files} files) · BROWSER_DOWNLOAD · `browser downloads path` for the folder · `browser downloads age` for newest mtime · does not list names."
            )
        }
    }
}


/// Zero-LLM browser downloads directory path (config only; no list/prune/download).
pub fn format_browser_downloads_path_gateway() -> String {
    let dir = crate::config::Config::browser_downloads_dir();
    let display = dir.display().to_string();
    format!(
        "**Browser downloads:** `{display}` · BROWSER_DOWNLOAD artifacts · pruned by retention · `/browser` for CDP status · `/downloads` for organizer · `browser downloads size` for disk use · `browser downloads age` for newest mtime."
    )
}


/// Zero-LLM Disk Cleanup quarantine directory age (newest file mtime; no list dump).
pub fn format_cleanup_quarantine_age_gateway() -> String {
    let dir = crate::config::Config::cleanup_quarantine_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Cleanup quarantine:** not created yet · app recreates under `~/.mac-stats/cleanup-quarantine/` · `cleanup quarantine path` for the folder · `cleanup quarantine size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**Cleanup quarantine** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Cleanup quarantine:** empty · `cleanup quarantine path` for the folder · `cleanup quarantine size` for on-disk bytes · `/disk` for scopes · soft-delete only · no dump from this ask."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Cleanup quarantine:** last write **{age}** ago · 1 file · soft-delete · `cleanup quarantine path` for the folder · `cleanup quarantine size` for on-disk bytes · does not list names."
                )
            } else {
                format!(
                    "**Cleanup quarantine:** newest write **{age}** ago · {files} files · soft-delete · `cleanup quarantine path` for the folder · `cleanup quarantine size` for on-disk bytes · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Cleanup quarantine** — could not read mtime · `cleanup quarantine path` for the folder · `cleanup quarantine size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM Disk Cleanup quarantine directory size (recursive file bytes; no list dump).
pub fn format_cleanup_quarantine_size_gateway() -> String {
    let dir = crate::config::Config::cleanup_quarantine_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Cleanup quarantine:** not created yet · app recreates under `~/.mac-stats/cleanup-quarantine/` · `cleanup quarantine path` for the folder · `cleanup quarantine age` for newest mtime."
                .to_string()
        }
        Err(e) => format!("**Cleanup quarantine** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Cleanup quarantine:** empty · `cleanup quarantine path` for the folder · `cleanup quarantine age` for newest mtime · `/disk` for scopes · soft-delete only."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Cleanup quarantine:** **{label}** on disk ({files} files) · soft-delete · `cleanup quarantine path` for the folder · `cleanup quarantine age` for newest mtime · does not list names."
            )
        }
    }
}


/// Zero-LLM Disk Cleanup quarantine directory path (config only; no list/prune/restore).
pub fn format_cleanup_quarantine_path_gateway() -> String {
    let dir = crate::config::Config::cleanup_quarantine_dir();
    let display = dir.display().to_string();
    format!(
        "**Cleanup quarantine:** `{display}` · Disk Cleanup auto soft-delete · `/disk` for scopes · `cleanup quarantine size` for disk use · `cleanup quarantine age` for newest mtime · does not touch system Trash."
    )
}


/// Zero-LLM pinned_processes.json file size (stat only; no list/pin/unpin).
pub fn format_pinned_processes_size_gateway() -> String {
    let path = crate::config::Config::pinned_processes_file_path();
    if !path.exists() {
        return "**Pinned processes:** no `pinned_processes.json` yet · app writes it after you pin a process · `pinned processes path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Pinned processes:** empty `pinned_processes.json` · `pinned processes path` for the file · `pinned processes age` for last write.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Pinned processes:** **{label}** on disk · Top Processes favorites · `pinned processes path` for the file · `pinned processes age` for last write · `/pinned` for the live list."
            )
        }
        Err(e) => format!("**Pinned processes** — could not stat `pinned_processes.json`: {e}"),
    }
}


/// Zero-LLM pinned_processes.json age from file mtime (stat only; no list/pin/unpin).
pub fn format_pinned_processes_age_gateway() -> String {
    let path = crate::config::Config::pinned_processes_file_path();
    if !path.exists() {
        return "**Pinned processes:** no `pinned_processes.json` yet · app writes it after you pin a process · `pinned processes path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Pinned processes:** last write **{age}** ago · Top Processes favorites · `pinned processes path` for the file · `pinned processes size` for on-disk bytes · `/pinned` for the live list."
            )
        }
        Err(e) => format!("**Pinned processes** — could not stat `pinned_processes.json`: {e}"),
    }
}


/// Zero-LLM Top Processes pinned favorites file path (config only; no list/pin/unpin).
pub fn format_pinned_processes_path_gateway() -> String {
    let path = crate::config::Config::pinned_processes_file_path();
    let display = path.display().to_string();
    format!(
        "**Pinned processes file:** `{display}` · Top Processes favorites · `pinned processes size` / `pinned processes age` for bytes / mtime · `/pinned` for the live list · does not pin or unpin."
    )
}


/// Zero-LLM schedules.json path (config only; no list/count/create).
pub fn format_schedules_path_gateway() -> String {
    let path = crate::config::Config::schedules_file_path();
    let display = path.display().to_string();
    format!(
        "**Schedules file:** `{display}` · Jobs + deliveries config · `schedules size` for on-disk bytes · `schedules age` for last write · `/schedules` for the live list · does not create or remove jobs."
    )
}


/// Zero-LLM schedules.json file size (stat only; no dump/list/create).
pub fn format_schedules_size_gateway() -> String {
    let path = crate::config::Config::schedules_file_path();
    if !path.exists() {
        return "**Schedules:** no `schedules.json` yet · app will create it when you add a job · `schedules path` for the file · `schedules age` after first save.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Schedules:** empty `schedules.json` · `schedules path` for the file · `schedules age` for last write.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Schedules:** **{label}** on disk · jobs + deliveries JSON · `schedules path` for the file · `schedules age` for last write · `/schedules` for the live list."
            )
        }
        Err(e) => format!("**Schedules** — could not stat `schedules.json`: {e}"),
    }
}


/// Zero-LLM schedules.json age from file mtime (stat only; no dump/list/create).
pub fn format_schedules_age_gateway() -> String {
    let path = crate::config::Config::schedules_file_path();
    if !path.exists() {
        return "**Schedules:** no `schedules.json` yet · app will create it when you add a job · `schedules path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Schedules:** last write **{age}** ago · jobs + deliveries JSON · `schedules path` for the file · `schedules size` for on-disk bytes · `/schedules` for the live list."
            )
        }
        Err(e) => format!("**Schedules** — could not stat `schedules.json`: {e}"),
    }
}


/// Zero-LLM monitors.json age from file mtime (stat only; no dump/list/add/check).
pub fn format_monitors_age_gateway() -> String {
    let path = crate::config::Config::monitors_file_path();
    if !path.exists() {
        return "**Monitors:** no `monitors.json` yet · app will create it when you add a site · `monitors path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Monitors:** last write **{age}** ago · External / website monitors JSON · `monitors path` for the file · `monitors size` for on-disk bytes · `/monitors` for the live list."
            )
        }
        Err(e) => format!("**Monitors** — could not stat `monitors.json`: {e}"),
    }
}


/// Zero-LLM monitors.json file size (stat only; no dump/list/add/check).
pub fn format_monitors_size_gateway() -> String {
    let path = crate::config::Config::monitors_file_path();
    if !path.exists() {
        return "**Monitors:** no `monitors.json` yet · app will create it when you add a site · `monitors path` for the file · `monitors age` after first save.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Monitors:** empty `monitors.json` · `monitors path` for the file · `monitors age` for last write.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Monitors:** **{label}** on disk · website monitors JSON · `monitors path` for the file · `monitors age` for last write · `/monitors` for the live list."
            )
        }
        Err(e) => format!("**Monitors** — could not stat `monitors.json`: {e}"),
    }
}


/// Zero-LLM monitors.json path (config only; no list/add/check).
pub fn format_monitors_path_gateway() -> String {
    let path = crate::config::Config::monitors_file_path();
    let display = path.display().to_string();
    format!(
        "**Monitors file:** `{display}` · External / website monitors config · `monitors size` / `monitors age` for bytes / mtime · `/monitors` for the live list · does not add or check sites."
    )
}


/// Zero-LLM history.json age from file mtime (stat only; no dump/sparkline/chat history).
pub fn format_history_age_gateway() -> String {
    let path = crate::config::Config::history_file_path();
    if !path.exists() {
        return "**History:** no `history.json` yet · app writes it after metrics warm-up · `history path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**History:** last write **{age}** ago · CPU / metrics sparkline buffer · `history path` for the file · `history size` for on-disk bytes · open the CPU window for live charts."
            )
        }
        Err(e) => format!("**History** — could not stat `history.json`: {e}"),
    }
}


/// Zero-LLM history.json file size (stat only; no dump/sparkline/chat history).
pub fn format_history_size_gateway() -> String {
    let path = crate::config::Config::history_file_path();
    if !path.exists() {
        return "**History:** no `history.json` yet · app writes it after metrics warm-up · `history path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**History:** empty `history.json` · `history path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**History:** **{label}** on disk · CPU / metrics sparkline buffer · `history path` for the file · `history age` for mtime · open the CPU window for live charts."
            )
        }
        Err(e) => format!("**History** — could not stat `history.json`: {e}"),
    }
}


/// Zero-LLM history.json path (config only; no sparkline dump / chat history).
pub fn format_history_path_gateway() -> String {
    let path = crate::config::Config::history_file_path();
    let display = path.display().to_string();
    format!(
        "**History file:** `{display}` · CPU / metrics sparkline buffer on disk · `history size` / `history age` for bytes / mtime · open the CPU window for live charts · does not dump points or chat history."
    )
}


/// Zero-LLM disk_cleanup.json age from file mtime (stat only; no list/reclaim/clean-now).
pub fn format_disk_cleanup_age_gateway() -> String {
    let path = crate::config::Config::disk_cleanup_file_path();
    if !path.exists() {
        return "**Disk cleanup:** no `disk_cleanup.json` yet · app writes it after Disk Cleanup runs · `disk cleanup path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Disk cleanup:** last write **{age}** ago · scopes + last-run state · `disk cleanup path` for the file · `disk cleanup size` for on-disk bytes · `/disk` for live scopes."
            )
        }
        Err(e) => format!("**Disk cleanup** — could not stat `disk_cleanup.json`: {e}"),
    }
}


/// Zero-LLM disk_cleanup.json file size (stat only; no list/reclaim/clean-now).
pub fn format_disk_cleanup_size_gateway() -> String {
    let path = crate::config::Config::disk_cleanup_file_path();
    if !path.exists() {
        return "**Disk cleanup:** no `disk_cleanup.json` yet · app writes it after Disk Cleanup runs · `disk cleanup path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Disk cleanup:** empty `disk_cleanup.json` · `disk cleanup path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Disk cleanup:** **{label}** on disk · scopes + last-run state · `disk cleanup path` for the file · `disk cleanup age` for mtime · `/disk` for live scopes."
            )
        }
        Err(e) => format!("**Disk cleanup** — could not stat `disk_cleanup.json`: {e}"),
    }
}


/// Zero-LLM disk_cleanup.json path (config only; no list/reclaim/clean-now).
pub fn format_disk_cleanup_path_gateway() -> String {
    let path = crate::config::Config::disk_cleanup_file_path();
    let display = path.display().to_string();
    format!(
        "**Disk cleanup file:** `{display}` · Disk Cleanup scopes + last-run state · `disk cleanup size` / `disk cleanup age` for bytes / mtime · `/disk` for live scopes · does not reclaim or clean."
    )
}


/// Zero-LLM perplexity_last.json file size (stat only; no Top/Snippet dump / `/perplexity`).
pub fn format_perplexity_last_size_gateway() -> String {
    let path = crate::config::Config::perplexity_last_file_path();
    if !path.exists() {
        return "**Perplexity last:** no `perplexity_last.json` yet · app writes it after a successful Perplexity Search · `perplexity last path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Perplexity last:** empty `perplexity_last.json` · `perplexity last path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Perplexity last:** **{label}** on disk · last Perplexity Search cache · `perplexity last path` for the file · `perplexity last age` for last write · `/perplexity` for Top/Snippet."
            )
        }
        Err(e) => format!("**Perplexity last** — could not stat `perplexity_last.json`: {e}"),
    }
}


/// Zero-LLM perplexity_last.json age from file mtime (stat only; no Top/Snippet dump / `/perplexity`).
pub fn format_perplexity_last_age_gateway() -> String {
    let path = crate::config::Config::perplexity_last_file_path();
    if !path.exists() {
        return "**Perplexity last:** no `perplexity_last.json` yet · app writes it after a successful Perplexity Search · `perplexity last path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Perplexity last:** last write **{age}** ago · last Perplexity Search cache · `perplexity last path` for the file · `perplexity last size` for on-disk bytes · `/perplexity` for Top/Snippet."
            )
        }
        Err(e) => format!("**Perplexity last** — could not stat `perplexity_last.json`: {e}"),
    }
}


/// Zero-LLM perplexity_last.json path (config only; no Top/Snippet dump / new search).
pub fn format_perplexity_last_path_gateway() -> String {
    let path = crate::config::Config::perplexity_last_file_path();
    let display = path.display().to_string();
    format!(
        "**Perplexity last file:** `{display}` · last Perplexity Search cache on disk · `perplexity last size` / `perplexity last age` for bytes / mtime · `/perplexity` for Top/Snippet · does not search or dump results."
    )
}


/// Zero-LLM discord_channels.json file size (stat only; no list/edit / `/discord`).
pub fn format_discord_channels_size_gateway() -> String {
    let path = crate::config::Config::discord_channels_path();
    if !path.exists() {
        return "**Discord channels:** no `discord_channels.json` yet · app writes it after Discord channel config saves · `discord channels path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Discord channels:** empty `discord_channels.json` · `discord channels path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Discord channels:** **{label}** on disk · per-channel listen / having_fun config · `discord channels path` for the file · `discord channels age` for last write · `/discord` for gateway Ready."
            )
        }
        Err(e) => format!("**Discord channels** — could not stat `discord_channels.json`: {e}"),
    }
}


/// Zero-LLM discord_channels.json age from file mtime (stat only; no list/edit / `/discord`).
pub fn format_discord_channels_age_gateway() -> String {
    let path = crate::config::Config::discord_channels_path();
    if !path.exists() {
        return "**Discord channels:** no `discord_channels.json` yet · app writes it after Discord channel config saves · `discord channels path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Discord channels:** last write **{age}** ago · per-channel listen / having_fun config · `discord channels path` for the file · `discord channels size` for on-disk bytes · `/discord` for gateway Ready."
            )
        }
        Err(e) => format!("**Discord channels** — could not stat `discord_channels.json`: {e}"),
    }
}


/// Zero-LLM discord_channels.json path (config only; no list/edit / `/discord`).
pub fn format_discord_channels_path_gateway() -> String {
    let path = crate::config::Config::discord_channels_path();
    let display = path.display().to_string();
    format!(
        "**Discord channels file:** `{display}` · per-channel listen / having_fun config · `discord channels size` / `discord channels age` for bytes / mtime · `/discord` for gateway Ready · does not list or edit channels."
    )
}


/// Zero-LLM scheduler_delivery_awareness.json file size (stat only; no list / `last delivery` / `/schedules`).
pub fn format_scheduler_delivery_awareness_size_gateway() -> String {
    let path = crate::config::Config::scheduler_delivery_awareness_file_path();
    if !path.exists() {
        return "**Delivery awareness:** no `scheduler_delivery_awareness.json` yet · app writes it after scheduler Discord deliveries · `delivery awareness path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Delivery awareness:** empty `scheduler_delivery_awareness.json` · `delivery awareness path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Delivery awareness:** **{label}** on disk · scheduler Discord delivery log · `delivery awareness path` for the file · `delivery awareness age` for last write · `last delivery` for the newest entry."
            )
        }
        Err(e) => format!("**Delivery awareness** — could not stat `scheduler_delivery_awareness.json`: {e}"),
    }
}


/// Zero-LLM scheduler_delivery_awareness.json age from file mtime (stat only; no list / `last delivery` / `/schedules`).
pub fn format_scheduler_delivery_awareness_age_gateway() -> String {
    let path = crate::config::Config::scheduler_delivery_awareness_file_path();
    if !path.exists() {
        return "**Delivery awareness:** no `scheduler_delivery_awareness.json` yet · app writes it after scheduler Discord deliveries · `delivery awareness path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Delivery awareness:** last write **{age}** ago · scheduler Discord delivery log · `delivery awareness path` for the file · `delivery awareness size` for on-disk bytes · `last delivery` for the newest entry."
            )
        }
        Err(e) => format!("**Delivery awareness** — could not stat `scheduler_delivery_awareness.json`: {e}"),
    }
}


/// Zero-LLM scheduler_delivery_awareness.json path (config only; no list / last delivery).
pub fn format_scheduler_delivery_awareness_path_gateway() -> String {
    let path = crate::config::Config::scheduler_delivery_awareness_file_path();
    let display = path.display().to_string();
    format!(
        "**Delivery awareness file:** `{display}` · scheduler Discord delivery log for chat context · `delivery awareness size` / `delivery awareness age` for bytes / mtime · `last delivery` for the newest entry · does not list deliveries."
    )
}


/// Zero-LLM user-info.json file size (stat only; no dump / edit / who-am-i).
pub fn format_user_info_size_gateway() -> String {
    let path = crate::config::Config::user_info_file_path();
    if !path.exists() {
        return "**User info:** no `user-info.json` yet · app writes display names from Discord · `user info path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**User info:** empty `user-info.json` · `user info path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**User info:** **{label}** on disk · Discord display-name map · `user info path` for the file · `user info age` for last write."
            )
        }
        Err(e) => format!("**User info** — could not stat `user-info.json`: {e}"),
    }
}


/// Zero-LLM user-info.json age from file mtime (stat only; no dump / edit / who-am-i).
pub fn format_user_info_age_gateway() -> String {
    let path = crate::config::Config::user_info_file_path();
    if !path.exists() {
        return "**User info:** no `user-info.json` yet · app writes display names from Discord · `user info path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**User info:** last write **{age}** ago · Discord display-name map · `user info path` for the file · `user info size` for on-disk bytes."
            )
        }
        Err(e) => format!("**User info** — could not stat `user-info.json`: {e}"),
    }
}


/// Zero-LLM user-info.json path (config only; no dump / edit).
pub fn format_user_info_path_gateway() -> String {
    let path = crate::config::Config::user_info_file_path();
    let display = path.display().to_string();
    format!(
        "**User info file:** `{display}` · Discord display-name map on disk · `user info size` / `user info age` for bytes / mtime · does not list or edit users."
    )
}


/// Zero-LLM `.config.env` file size (stat only; never dumps keys).
pub fn format_config_env_size_gateway() -> String {
    let path = crate::config::Config::home_config_env_path();
    if !path.exists() {
        return "**Secrets env:** no `.config.env` yet · sync from repo with `./scripts/sync-home-config-env.sh` · `config.env path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Secrets env:** empty `.config.env` · `config.env path` for the file · `config.env age` for last write.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Secrets env:** **{label}** on disk · `.config.env` · `config.env path` for the file · `config.env age` for last write · does not read keys."
            )
        }
        Err(e) => format!("**Secrets env** — could not stat `.config.env`: {e}"),
    }
}


/// Zero-LLM `.config.env` age from file mtime (stat only; never dumps keys).
pub fn format_config_env_age_gateway() -> String {
    let path = crate::config::Config::home_config_env_path();
    if !path.exists() {
        return "**Secrets env:** no `.config.env` yet · sync from repo with `./scripts/sync-home-config-env.sh` · `config.env path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Secrets env:** last write **{age}** ago · `.config.env` · `config.env path` for the file · `config.env size` for on-disk bytes · does not read keys."
            )
        }
        Err(e) => format!("**Secrets env** — could not stat `.config.env`: {e}"),
    }
}


/// Zero-LLM `.config.env` path (path only; never dumps keys).
pub fn format_config_env_path_gateway() -> String {
    let path = crate::config::Config::home_config_env_path();
    let display = path.display().to_string();
    format!(
        "**Secrets env file:** `{display}` · path only · `config.env size` / `config.env age` for bytes / mtime · does not read or list keys · sync from repo with `./scripts/sync-home-config-env.sh` · Settings / Keychain for many credentials."
    )
}


/// Zero-LLM improvements directory size (recursive file bytes; no list dump).
pub fn format_improvements_size_gateway() -> String {
    let dir = crate::config::Config::improvements_dir();
    match dir_total_bytes(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Improvements dir:** not created yet · digester / overnight harness writes here · `improvements path` for the folder."
                .to_string()
        }
        Err(e) => format!("**Improvements dir** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Improvements dir:** empty · `improvements path` for the folder · digester writes `latest.md` here."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Improvements dir:** **{label}** on disk ({files} files) · digester · morning surprise · results.tsv · `improvements path` for the folder · `improvements age` for newest mtime · does not list names."
            )
        }
    }
}


/// Zero-LLM improvements directory path (config only; no list/open digest).
pub fn format_improvements_path_gateway() -> String {
    let dir = crate::config::Config::improvements_dir();
    let display = dir.display().to_string();
    format!(
        "**Improvements dir:** `{display}` · digester · morning surprise · results.tsv · Agent Ops → Digest · does not list files · `improvements size` for disk use · `improvements age` for newest mtime."
    )
}


/// Zero-LLM improvements directory age (newest file mtime; no list dump).
pub fn format_improvements_age_gateway() -> String {
    let dir = crate::config::Config::improvements_dir();
    match dir_newest_mtime_ms(&dir, 8_000) {
        Err(msg) if msg == "missing" => {
            "**Improvements dir:** not created yet · digester / overnight harness writes here · `improvements path` for the folder · `improvements size` for on-disk bytes."
                .to_string()
        }
        Err(e) => format!("**Improvements dir** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Improvements dir:** empty · `improvements path` for the folder · digester writes `latest.md` here · `improvements size` for on-disk bytes."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Improvements dir:** last write **{age}** ago · 1 file · digester · morning surprise · results.tsv · `improvements path` for the folder · `improvements size` for on-disk bytes · does not list names."
                )
            } else {
                format!(
                    "**Improvements dir:** newest write **{age}** ago · {files} files · digester · morning surprise · results.tsv · `improvements path` for the folder · `improvements size` for on-disk bytes · does not list names."
                )
            }
        }
        Ok((None, _)) => {
            "**Improvements dir** — could not read mtime · `improvements path` for the folder · `improvements size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM credential_accounts.json file size (stat only; no dump / list / Keychain secrets).
pub fn format_credential_accounts_size_gateway() -> String {
    let path = crate::config::Config::credential_accounts_file_path();
    if !path.exists() {
        return "**Credential accounts:** no `credential_accounts.json` yet · app writes Keychain account names · `credential accounts path` for the file.".to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Credential accounts:** empty `credential_accounts.json` · `credential accounts path` for the file.".to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Credential accounts:** **{label}** on disk · Keychain account-name list · `credential accounts path` for the file · `credential accounts age` for last write."
            )
        }
        Err(e) => format!("**Credential accounts** — could not stat `credential_accounts.json`: {e}"),
    }
}


/// Zero-LLM credential_accounts.json age from file mtime (stat only; no dump / list / Keychain secrets).
pub fn format_credential_accounts_age_gateway() -> String {
    let path = crate::config::Config::credential_accounts_file_path();
    if !path.exists() {
        return "**Credential accounts:** no `credential_accounts.json` yet · app writes Keychain account names · `credential accounts path` for the file.".to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Credential accounts:** last write **{age}** ago · Keychain account-name list · `credential accounts path` for the file · `credential accounts size` for on-disk bytes."
            )
        }
        Err(e) => format!("**Credential accounts** — could not stat `credential_accounts.json`: {e}"),
    }
}


/// Zero-LLM credential_accounts.json path (config only; no list/dump Keychain accounts).
pub fn format_credential_accounts_path_gateway() -> String {
    let path = crate::config::Config::credential_accounts_file_path();
    let display = path.display().to_string();
    format!(
        "**Credential accounts file:** `{display}` · Keychain account-name list on disk · `credential accounts size` / `credential accounts age` for bytes / mtime · does not list accounts or dump secrets · Settings → Credentials."
    )
}


/// Zero-LLM escalation_patterns.md file size (stat only; no dump/list phrases).
pub fn format_escalation_patterns_size_gateway() -> String {
    let path = crate::config::Config::escalation_patterns_path();
    if !path.exists() {
        return "**Escalation patterns:** no `escalation_patterns.md` yet · `escalation patterns path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Escalation patterns:** empty `escalation_patterns.md` · `escalation patterns path` for the file."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Escalation patterns:** **{label}** on disk · phrases that raise escalation mode · `escalation patterns path` for the file · `escalation patterns age` for last write · does not list phrases."
            )
        }
        Err(e) => format!("**Escalation patterns** — could not stat `escalation_patterns.md`: {e}"),
    }
}


/// Zero-LLM escalation_patterns.md age from file mtime (stat only; no dump/list phrases).
pub fn format_escalation_patterns_age_gateway() -> String {
    let path = crate::config::Config::escalation_patterns_path();
    if !path.exists() {
        return "**Escalation patterns:** no `escalation_patterns.md` yet · `escalation patterns path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Escalation patterns:** last write **{age}** ago · phrases that raise escalation mode · `escalation patterns path` for the file · `escalation patterns size` for on-disk bytes."
            )
        }
        Err(e) => {
            format!("**Escalation patterns** — could not stat `escalation_patterns.md`: {e}")
        }
    }
}


/// Zero-LLM escalation_patterns.md path (config only; no list/append phrases).
pub fn format_escalation_patterns_path_gateway() -> String {
    let path = crate::config::Config::escalation_patterns_path();
    let display = path.display().to_string();
    format!(
        "**Escalation patterns file:** `{display}` · phrases that raise escalation mode · path only · `escalation patterns size` for on-disk bytes · `escalation patterns age` for last write · does not list or edit phrases."
    )
}


/// Zero-LLM session_reset_phrases.md file size (stat only; no dump/list phrases).
pub fn format_session_reset_phrases_size_gateway() -> String {
    let path = crate::config::Config::session_reset_phrases_path();
    if !path.exists() {
        return "**Session reset phrases:** no `session_reset_phrases.md` yet · `session reset phrases path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Session reset phrases:** empty `session_reset_phrases.md` · `session reset phrases path` for the file."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Session reset phrases:** **{label}** on disk · phrases that clear a Discord session · `session reset phrases path` for the file · `session reset phrases age` for last write · does not list phrases."
            )
        }
        Err(e) => {
            format!("**Session reset phrases** — could not stat `session_reset_phrases.md`: {e}")
        }
    }
}


/// Zero-LLM session_reset_phrases.md age from file mtime (stat only; no dump/list phrases).
pub fn format_session_reset_phrases_age_gateway() -> String {
    let path = crate::config::Config::session_reset_phrases_path();
    if !path.exists() {
        return "**Session reset phrases:** no `session_reset_phrases.md` yet · `session reset phrases path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Session reset phrases:** last write **{age}** ago · phrases that clear a Discord session · `session reset phrases path` for the file · `session reset phrases size` for on-disk bytes."
            )
        }
        Err(e) => {
            format!("**Session reset phrases** — could not stat `session_reset_phrases.md`: {e}")
        }
    }
}


/// Zero-LLM session_reset_phrases.md path (config only; no list / no session clear).
pub fn format_session_reset_phrases_path_gateway() -> String {
    let path = crate::config::Config::session_reset_phrases_path();
    let display = path.display().to_string();
    format!(
        "**Session reset phrases file:** `{display}` · phrases that clear a Discord session · path only · `session reset phrases size` for on-disk bytes · `session reset phrases age` for last write · does not list or trigger a reset."
    )
}


/// Zero-LLM before-reset transcript file size (stat only; no dump / no hook / no session clear).
pub fn format_before_reset_transcript_size_gateway() -> String {
    use crate::config::Config;
    let path = if let Some(p) = Config::before_reset_transcript_path_resolved() {
        p
    } else {
        Config::default_before_reset_transcript_path()
    };
    if !path.exists() {
        return "**Before-reset transcript:** no file yet · `before reset transcript path` for the location · `before reset transcript age` for mtime · written when a before-reset hook runs · does not dump JSONL."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Before-reset transcript:** empty file · `before reset transcript path` for the location · `before reset transcript age` for mtime · does not dump JSONL."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Before-reset transcript:** **{label}** on disk · `before reset transcript path` for the location · `before reset transcript age` for mtime · does not dump JSONL or trigger reset."
            )
        }
        Err(_) => {
            "**Before-reset transcript:** could not read size · `before reset transcript path` for the location · `before reset transcript age` for mtime."
                .to_string()
        }
    }
}


/// Zero-LLM before-reset transcript age (mtime only; no dump / no hook / no session clear).
pub fn format_before_reset_transcript_age_gateway() -> String {
    use crate::config::Config;
    let path = if let Some(p) = Config::before_reset_transcript_path_resolved() {
        p
    } else {
        Config::default_before_reset_transcript_path()
    };
    if !path.exists() {
        return "**Before-reset transcript:** no file yet · `before reset transcript path` for the location · `before reset transcript size` for on-disk bytes · written when a before-reset hook runs · does not dump JSONL."
            .to_string();
    }
    match std::fs::metadata(&path) {
        Ok(meta) => {
            let ms = file_mtime_ms(&meta);
            if ms == 0 {
                return "**Before-reset transcript** — could not read mtime · `before reset transcript path` for the location · `before reset transcript size` for on-disk bytes."
                    .to_string();
            }
            let age = age_from_ms(ms);
            format!(
                "**Before-reset transcript:** last write **{age}** ago · `before reset transcript path` for the location · `before reset transcript size` for on-disk bytes · does not dump JSONL or trigger reset."
            )
        }
        Err(_) => {
            "**Before-reset transcript:** could not read mtime · `before reset transcript path` for the location · `before reset transcript size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM before-reset transcript path (config/env only; no dump / no hook / no session clear).
pub fn format_before_reset_transcript_path_gateway() -> String {
    use crate::config::Config;
    let display = Config::before_reset_transcript_path_display();
    let custom = !Config::before_reset_transcript_path_raw().trim().is_empty();
    let note = if custom {
        "configured path"
    } else {
        "default when a before-reset hook runs (set `beforeResetTranscriptPath` / `MAC_STATS_BEFORE_RESET_TRANSCRIPT_PATH` to override)"
    };
    format!(
        "**Before-reset transcript:** `{display}` · {note} · path only · `before reset transcript size` for on-disk bytes · `before reset transcript age` for mtime · does not dump JSONL or trigger reset · `session reset phrases path` for the phrases file · `before compaction transcript path` for compaction export."
    )
}


/// Zero-LLM before-compaction transcript file size (stat only; no dump / no hook / no compact).
pub fn format_before_compaction_transcript_size_gateway() -> String {
    use crate::config::Config;
    let path = if let Some(p) = Config::before_compaction_transcript_path_resolved() {
        p
    } else {
        Config::default_before_compaction_transcript_path()
    };
    if !path.exists() {
        return "**Before-compaction transcript:** no file yet · `before compaction transcript path` for the location · `before compaction transcript age` for mtime · written when a before-compaction hook runs · does not dump JSONL."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Before-compaction transcript:** empty file · `before compaction transcript path` for the location · `before compaction transcript age` for mtime · does not dump JSONL."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Before-compaction transcript:** **{label}** on disk · `before compaction transcript path` for the location · `before compaction transcript age` for mtime · does not dump JSONL or run compaction."
            )
        }
        Err(_) => {
            "**Before-compaction transcript:** could not read size · `before compaction transcript path` for the location · `before compaction transcript age` for mtime."
                .to_string()
        }
    }
}


/// Zero-LLM before-compaction transcript age (mtime only; no dump / no hook / no compact).
pub fn format_before_compaction_transcript_age_gateway() -> String {
    use crate::config::Config;
    let path = if let Some(p) = Config::before_compaction_transcript_path_resolved() {
        p
    } else {
        Config::default_before_compaction_transcript_path()
    };
    if !path.exists() {
        return "**Before-compaction transcript:** no file yet · `before compaction transcript path` for the location · `before compaction transcript size` for on-disk bytes · written when a before-compaction hook runs · does not dump JSONL."
            .to_string();
    }
    match std::fs::metadata(&path) {
        Ok(meta) => {
            let ms = file_mtime_ms(&meta);
            if ms == 0 {
                return "**Before-compaction transcript** — could not read mtime · `before compaction transcript path` for the location · `before compaction transcript size` for on-disk bytes."
                    .to_string();
            }
            let age = age_from_ms(ms);
            format!(
                "**Before-compaction transcript:** last write **{age}** ago · `before compaction transcript path` for the location · `before compaction transcript size` for on-disk bytes · does not dump JSONL or run compaction."
            )
        }
        Err(_) => {
            "**Before-compaction transcript:** could not read mtime · `before compaction transcript path` for the location · `before compaction transcript size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM before-compaction transcript path (config/env only; no dump / no hook / no compact).
pub fn format_before_compaction_transcript_path_gateway() -> String {
    use crate::config::Config;
    let display = Config::before_compaction_transcript_path_display();
    let custom = !Config::before_compaction_transcript_path_raw().trim().is_empty();
    let note = if custom {
        "configured path"
    } else {
        "default when a before-compaction hook runs (set `beforeCompactionTranscriptPath` / `MAC_STATS_BEFORE_COMPACTION_TRANSCRIPT_PATH` to override)"
    };
    format!(
        "**Before-compaction transcript:** `{display}` · {note} · path only · `before compaction transcript size` for on-disk bytes · `before compaction transcript age` for mtime · does not dump JSONL or run compaction · `before reset transcript path` for the reset export."
    )
}


/// Zero-LLM cookie_reject_patterns.md file size (stat only; no dump/list patterns).
pub fn format_cookie_reject_patterns_size_gateway() -> String {
    let path = crate::config::Config::cookie_reject_patterns_path();
    if !path.exists() {
        return "**Cookie reject patterns:** no `cookie_reject_patterns.md` yet · `cookie reject patterns path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Cookie reject patterns:** empty `cookie_reject_patterns.md` · `cookie reject patterns path` for the file."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Cookie reject patterns:** **{label}** on disk · browser cookie banner reject phrases · `cookie reject patterns path` for the file · `cookie reject patterns age` for last write · does not list patterns."
            )
        }
        Err(e) => {
            format!("**Cookie reject patterns** — could not stat `cookie_reject_patterns.md`: {e}")
        }
    }
}


/// Zero-LLM cookie_reject_patterns.md age from file mtime (stat only; no dump/list patterns).
pub fn format_cookie_reject_patterns_age_gateway() -> String {
    let path = crate::config::Config::cookie_reject_patterns_path();
    if !path.exists() {
        return "**Cookie reject patterns:** no `cookie_reject_patterns.md` yet · `cookie reject patterns path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Cookie reject patterns:** last write **{age}** ago · browser cookie banner reject phrases · `cookie reject patterns path` for the file · `cookie reject patterns size` for on-disk bytes."
            )
        }
        Err(e) => {
            format!("**Cookie reject patterns** — could not stat `cookie_reject_patterns.md`: {e}")
        }
    }
}


/// Zero-LLM cookie_reject_patterns.md path (config only; no list/edit patterns).
pub fn format_cookie_reject_patterns_path_gateway() -> String {
    let path = crate::config::Config::cookie_reject_patterns_path();
    let display = path.display().to_string();
    format!(
        "**Cookie reject patterns file:** `{display}` · browser cookie banner reject phrases · path only · `cookie reject patterns size` for on-disk bytes · `cookie reject patterns age` for last write · does not list or edit patterns."
    )
}


/// Zero-LLM downloads-organizer-rules.md file size (stat only; no dump/list rules).
pub fn format_downloads_organizer_rules_size_gateway() -> String {
    let path = crate::config::Config::downloads_organizer_rules_path();
    if !path.exists() {
        return "**Downloads organizer rules:** no `downloads-organizer-rules.md` yet · `downloads organizer rules path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Downloads organizer rules:** empty `downloads-organizer-rules.md` · `downloads organizer rules path` for the file."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Downloads organizer rules:** **{label}** on disk · folder sort rules · `downloads organizer rules path` for the file · `downloads organizer rules age` for last write · does not list rules or run organize-now."
            )
        }
        Err(e) => {
            format!(
                "**Downloads organizer rules** — could not stat `downloads-organizer-rules.md`: {e}"
            )
        }
    }
}


/// Zero-LLM downloads-organizer-rules.md age from file mtime (stat only; no dump/list rules).
pub fn format_downloads_organizer_rules_age_gateway() -> String {
    let path = crate::config::Config::downloads_organizer_rules_path();
    if !path.exists() {
        return "**Downloads organizer rules:** no `downloads-organizer-rules.md` yet · `downloads organizer rules path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Downloads organizer rules:** last write **{age}** ago · folder sort rules · `downloads organizer rules path` for the file · `downloads organizer rules size` for on-disk bytes."
            )
        }
        Err(e) => {
            format!(
                "**Downloads organizer rules** — could not stat `downloads-organizer-rules.md`: {e}"
            )
        }
    }
}


/// Zero-LLM downloads-organizer-rules.md path (config only; no list/edit/run).
pub fn format_downloads_organizer_rules_path_gateway() -> String {
    let path = crate::config::Config::downloads_organizer_rules_path();
    let display = path.display().to_string();
    format!(
        "**Downloads organizer rules file:** `{display}` · folder sort rules · path only · `downloads organizer rules size` for on-disk bytes · `downloads organizer rules age` for last write · does not list rules or run organize-now."
    )
}


/// Zero-LLM downloads-organizer-state.json file size (stat only; no dump/list JSON).
pub fn format_downloads_organizer_state_size_gateway() -> String {
    let path = crate::config::Config::downloads_organizer_state_path();
    if !path.exists() {
        return "**Downloads organizer state:** no `downloads-organizer-state.json` yet · `downloads organizer state path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Downloads organizer state:** empty `downloads-organizer-state.json` · `downloads organizer state path` for the file."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Downloads organizer state:** **{label}** on disk · last-run summary · `downloads organizer state path` for the file · `downloads organizer state age` for last write · does not dump JSON or run organize-now."
            )
        }
        Err(e) => {
            format!(
                "**Downloads organizer state** — could not stat `downloads-organizer-state.json`: {e}"
            )
        }
    }
}


/// Zero-LLM downloads-organizer-state.json age from file mtime (stat only; no dump/list JSON).
pub fn format_downloads_organizer_state_age_gateway() -> String {
    let path = crate::config::Config::downloads_organizer_state_path();
    if !path.exists() {
        return "**Downloads organizer state:** no `downloads-organizer-state.json` yet · `downloads organizer state path` for the file."
            .to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Downloads organizer state:** last write **{age}** ago · last-run summary · `downloads organizer state path` for the file · `downloads organizer state size` for on-disk bytes."
            )
        }
        Err(e) => {
            format!(
                "**Downloads organizer state** — could not stat `downloads-organizer-state.json`: {e}"
            )
        }
    }
}


/// Zero-LLM downloads-organizer-state.json path (config only; no dump/edit/run).
pub fn format_downloads_organizer_state_path_gateway() -> String {
    let path = crate::config::Config::downloads_organizer_state_path();
    let display = path.display().to_string();
    format!(
        "**Downloads organizer state file:** `{display}` · last-run summary · path only · `downloads organizer state size` for on-disk bytes · `downloads organizer state age` for last write · does not dump JSON or run organize-now."
    )
}


/// Zero-LLM per-agent testing.md path (config only; no dump/edit/run).
pub fn format_testing_md_path_gateway() -> String {
    let display = crate::config::Config::testing_file_path_display();
    format!(
        "**Testing file:** `{display}` · per-agent test prompts (`mac_stats agent test`) · path only · `testing.md size` / `testing.md age` for bytes / mtime · does not dump or edit testing text · Agent Ops → Agents for content."
    )
}


/// Zero-LLM per-agent testing.md file size (stat only; no dump/edit/run).
/// Sums on-disk bytes of every `agents/agent-*/testing.md` that exists.
pub fn format_testing_md_size_gateway() -> String {
    let agents = crate::config::Config::agents_dir();
    let mut total: u64 = 0;
    let mut count: usize = 0;
    if let Ok(entries) = std::fs::read_dir(&agents) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("agent-") {
                continue;
            }
            let testing = entry.path().join("testing.md");
            if let Ok(meta) = std::fs::metadata(&testing) {
                if meta.is_file() {
                    total = total.saturating_add(meta.len());
                    count = count.saturating_add(1);
                }
            }
        }
    }
    if count == 0 {
        return "**Testing file:** no `testing.md` yet · `testing.md path` for the file pattern · `testing.md age` for last write · Agent Ops → Agents for content · does not run tests."
            .to_string();
    }
    let label = crate::commands::disk_cleanup::format_bytes(total);
    if count == 1 {
        format!(
            "**Testing file:** **{label}** on disk · 1 per-agent `testing.md` · `testing.md path` for the file · `testing.md age` for last write · does not dump testing text or run tests."
        )
    } else {
        format!(
            "**Testing file:** **{label}** on disk · {count} per-agent `testing.md` files · `testing.md path` for the pattern · `testing.md age` for last write · does not dump testing text or run tests."
        )
    }
}


/// Zero-LLM per-agent testing.md age from newest file mtime (stat only; no dump/edit/run).
pub fn format_testing_md_age_gateway() -> String {
    let agents = crate::config::Config::agents_dir();
    let mut newest_ms: Option<u64> = None;
    let mut count: usize = 0;
    if let Ok(entries) = std::fs::read_dir(&agents) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("agent-") {
                continue;
            }
            let testing = entry.path().join("testing.md");
            if let Ok(meta) = std::fs::metadata(&testing) {
                if meta.is_file() {
                    count = count.saturating_add(1);
                    if let Ok(modified) = meta.modified() {
                        let ms = modified
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                        newest_ms = Some(match newest_ms {
                            Some(prev) => prev.max(ms),
                            None => ms,
                        });
                    }
                }
            }
        }
    }
    if count == 0 {
        return "**Testing file:** no `testing.md` yet · `testing.md path` for the file pattern · Agent Ops → Agents for content · does not run tests."
            .to_string();
    }
    let Some(ms) = newest_ms else {
        return "**Testing file** — could not read mtime on per-agent `testing.md`.".to_string();
    };
    let age = age_from_ms(ms);
    if count == 1 {
        format!(
            "**Testing file:** last write **{age}** ago · 1 per-agent `testing.md` · `testing.md path` for the file · `testing.md size` for on-disk bytes · does not dump testing text or run tests."
        )
    } else {
        format!(
            "**Testing file:** newest write **{age}** ago · {count} per-agent `testing.md` files · `testing.md path` for the pattern · `testing.md size` for on-disk bytes · does not dump testing text or run tests."
        )
    }
}


/// Zero-LLM planning_prompt.md file size (stat only; no dump/edit).
pub fn format_planning_prompt_size_gateway() -> String {
    let path = crate::config::Config::planning_prompt_path();
    if !path.exists() {
        return "**Planning prompt:** no `planning_prompt.md` yet · `planning_prompt.md path` for the file · `planning_prompt.md age` for last write · `prompts path` for the folder."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Planning prompt:** empty `planning_prompt.md` · `planning_prompt.md path` for the file · `planning_prompt.md age` for last write."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Planning prompt:** **{label}** on disk · agent-router planning step · `planning_prompt.md path` for the file · `planning_prompt.md age` for last write · does not dump prompt text."
            )
        }
        Err(e) => format!("**Planning prompt** — could not stat `planning_prompt.md`: {e}"),
    }
}


/// Zero-LLM planning_prompt.md age from file mtime (stat only; no dump/edit).
pub fn format_planning_prompt_age_gateway() -> String {
    let path = crate::config::Config::planning_prompt_path();
    if !path.exists() {
        return "**Planning prompt:** no `planning_prompt.md` yet · `planning_prompt.md path` for the file · `prompts path` for the folder."
            .to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Planning prompt:** last write **{age}** ago · agent-router planning step · `planning_prompt.md path` for the file · `planning_prompt.md size` for on-disk bytes · does not dump prompt text."
            )
        }
        Err(e) => format!("**Planning prompt** — could not stat `planning_prompt.md`: {e}"),
    }
}


/// Zero-LLM planning_prompt.md path (config only; no dump/edit).
pub fn format_planning_prompt_path_gateway() -> String {
    let display = crate::config::Config::planning_prompt_path().display().to_string();
    format!(
        "**Planning prompt:** `{display}` · agent-router planning step · path only · `planning_prompt.md size` / `planning_prompt.md age` for bytes / mtime · does not dump or edit prompt text · `prompts path` for the folder."
    )
}


/// Zero-LLM execution_prompt.md file size (stat only; no dump/edit).
pub fn format_execution_prompt_size_gateway() -> String {
    let path = crate::config::Config::execution_prompt_path();
    if !path.exists() {
        return "**Execution prompt:** no `execution_prompt.md` yet · `execution_prompt.md path` for the file · `execution_prompt.md age` for last write · `prompts path` for the folder."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Execution prompt:** empty `execution_prompt.md` · `execution_prompt.md path` for the file · `execution_prompt.md age` for last write."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Execution prompt:** **{label}** on disk · agent-router tools + conversation rules · `execution_prompt.md path` for the file · `execution_prompt.md age` for last write · does not dump prompt text."
            )
        }
        Err(e) => format!("**Execution prompt** — could not stat `execution_prompt.md`: {e}"),
    }
}


/// Zero-LLM execution_prompt.md age from file mtime (stat only; no dump/edit).
pub fn format_execution_prompt_age_gateway() -> String {
    let path = crate::config::Config::execution_prompt_path();
    if !path.exists() {
        return "**Execution prompt:** no `execution_prompt.md` yet · `execution_prompt.md path` for the file · `prompts path` for the folder."
            .to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Execution prompt:** last write **{age}** ago · agent-router tools + conversation rules · `execution_prompt.md path` for the file · `execution_prompt.md size` for on-disk bytes · does not dump prompt text."
            )
        }
        Err(e) => format!("**Execution prompt** — could not stat `execution_prompt.md`: {e}"),
    }
}


/// Zero-LLM execution_prompt.md path (config only; no dump/edit).
pub fn format_execution_prompt_path_gateway() -> String {
    let display = crate::config::Config::execution_prompt_path()
        .display()
        .to_string();
    format!(
        "**Execution prompt:** `{display}` · agent-router tools + conversation rules · path only · `execution_prompt.md size` / `execution_prompt.md age` for bytes / mtime · does not dump or edit prompt text · `prompts path` for the folder."
    )
}


/// Zero-LLM per-agent agent.json path (config only; no dump/edit).
pub fn format_agent_json_path_gateway() -> String {
    let display = crate::config::Config::agent_json_file_path_display();
    format!(
        "**Agent config:** `{display}` · per-agent name · model · enabled · tool limits · path only · `agent.json size` / `agent.json age` for bytes / mtime · does not dump or edit JSON · Agent Ops → Agents · `agents path` for the folder · `config path` for app config.json."
    )
}


/// Zero-LLM per-agent agent.json file size (stat only; no dump/edit).
/// Sums on-disk bytes of every `agents/agent-*/agent.json` that exists.
pub fn format_agent_json_size_gateway() -> String {
    let agents = crate::config::Config::agents_dir();
    let mut total: u64 = 0;
    let mut count: usize = 0;
    if let Ok(entries) = std::fs::read_dir(&agents) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("agent-") {
                continue;
            }
            let agent_json = entry.path().join("agent.json");
            if let Ok(meta) = std::fs::metadata(&agent_json) {
                if meta.is_file() {
                    total = total.saturating_add(meta.len());
                    count = count.saturating_add(1);
                }
            }
        }
    }
    if count == 0 {
        return "**Agent config:** no `agent.json` yet · `agent.json path` for the file pattern · `agent.json age` for last write · Agent Ops → Agents for content · does not dump JSON."
            .to_string();
    }
    let label = crate::commands::disk_cleanup::format_bytes(total);
    if count == 1 {
        format!(
            "**Agent config:** **{label}** on disk · 1 per-agent `agent.json` · `agent.json path` for the file · `agent.json age` for last write · does not dump JSON."
        )
    } else {
        format!(
            "**Agent config:** **{label}** on disk · {count} per-agent `agent.json` files · `agent.json path` for the pattern · `agent.json age` for last write · does not dump JSON."
        )
    }
}


/// Zero-LLM per-agent agent.json age from newest file mtime (stat only; no dump/edit).
pub fn format_agent_json_age_gateway() -> String {
    let agents = crate::config::Config::agents_dir();
    let mut newest_ms: Option<u64> = None;
    let mut count: usize = 0;
    if let Ok(entries) = std::fs::read_dir(&agents) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("agent-") {
                continue;
            }
            let agent_json = entry.path().join("agent.json");
            if let Ok(meta) = std::fs::metadata(&agent_json) {
                if meta.is_file() {
                    count = count.saturating_add(1);
                    if let Ok(modified) = meta.modified() {
                        let ms = modified
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                        newest_ms = Some(match newest_ms {
                            Some(prev) => prev.max(ms),
                            None => ms,
                        });
                    }
                }
            }
        }
    }
    if count == 0 {
        return "**Agent config:** no `agent.json` yet · `agent.json path` for the file pattern · Agent Ops → Agents for content · does not dump JSON."
            .to_string();
    }
    let Some(ms) = newest_ms else {
        return "**Agent config** — could not read mtime on per-agent `agent.json`.".to_string();
    };
    let age = age_from_ms(ms);
    if count == 1 {
        format!(
            "**Agent config:** last write **{age}** ago · 1 per-agent `agent.json` · `agent.json path` for the file · `agent.json size` for on-disk bytes · does not dump JSON."
        )
    } else {
        format!(
            "**Agent config:** newest write **{age}** ago · {count} per-agent `agent.json` files · `agent.json path` for the pattern · `agent.json size` for on-disk bytes · does not dump JSON."
        )
    }
}


/// Zero-LLM per-agent skill.md path (config only; no dump/edit).
pub fn format_skill_md_path_gateway() -> String {
    let display = crate::config::Config::skill_file_path_display();
    format!(
        "**Skill file:** `{display}` · per-agent skill (not the Hermes skills/ folder) · path only · `skill.md size` / `skill.md age` for bytes / mtime · does not dump or edit skill text · Agent Ops → Agents for content · `/skills` for catalog."
    )
}


/// Zero-LLM per-agent skill.md file size (stat only; no dump/edit).
/// Sums on-disk bytes of every `agents/agent-*/skill.md` that exists.
pub fn format_skill_md_size_gateway() -> String {
    let agents = crate::config::Config::agents_dir();
    let mut total: u64 = 0;
    let mut count: usize = 0;
    if let Ok(entries) = std::fs::read_dir(&agents) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("agent-") {
                continue;
            }
            let skill = entry.path().join("skill.md");
            if let Ok(meta) = std::fs::metadata(&skill) {
                if meta.is_file() {
                    total = total.saturating_add(meta.len());
                    count = count.saturating_add(1);
                }
            }
        }
    }
    if count == 0 {
        return "**Skill file:** no `skill.md` yet · `skill.md path` for the file pattern · `skill.md age` for last write · Agent Ops → Agents for content · `skills size` for the Hermes skills/ folder."
            .to_string();
    }
    let label = crate::commands::disk_cleanup::format_bytes(total);
    if count == 1 {
        format!(
            "**Skill file:** **{label}** on disk · 1 per-agent `skill.md` · `skill.md path` for the file · `skill.md age` for last write · does not dump skill text · `skills size` for Hermes skills/."
        )
    } else {
        format!(
            "**Skill file:** **{label}** on disk · {count} per-agent `skill.md` files · `skill.md path` for the pattern · `skill.md age` for last write · does not dump skill text · `skills size` for Hermes skills/."
        )
    }
}


/// Zero-LLM per-agent skill.md age from newest file mtime (stat only; no dump/edit).
pub fn format_skill_md_age_gateway() -> String {
    let agents = crate::config::Config::agents_dir();
    let mut newest_ms: Option<u64> = None;
    let mut count: usize = 0;
    if let Ok(entries) = std::fs::read_dir(&agents) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("agent-") {
                continue;
            }
            let skill = entry.path().join("skill.md");
            if let Ok(meta) = std::fs::metadata(&skill) {
                if meta.is_file() {
                    count = count.saturating_add(1);
                    if let Ok(modified) = meta.modified() {
                        let ms = modified
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                        newest_ms = Some(match newest_ms {
                            Some(prev) => prev.max(ms),
                            None => ms,
                        });
                    }
                }
            }
        }
    }
    if count == 0 {
        return "**Skill file:** no `skill.md` yet · `skill.md path` for the file pattern · Agent Ops → Agents for content · `skills size` for Hermes skills/."
            .to_string();
    }
    let Some(ms) = newest_ms else {
        return "**Skill file** — could not read mtime on per-agent `skill.md`.".to_string();
    };
    let age = age_from_ms(ms);
    if count == 1 {
        format!(
            "**Skill file:** last write **{age}** ago · 1 per-agent `skill.md` · `skill.md path` for the file · `skill.md size` for on-disk bytes · does not dump skill text."
        )
    } else {
        format!(
            "**Skill file:** newest write **{age}** ago · {count} per-agent `skill.md` files · `skill.md path` for the pattern · `skill.md size` for on-disk bytes · does not dump skill text."
        )
    }
}


/// Zero-LLM per-agent mood.md path (config only; no dump/edit).
pub fn format_mood_path_gateway() -> String {
    let display = crate::config::Config::mood_file_path_display();
    format!(
        "**Mood file:** `{display}` · per-agent mood (not shared like soul.md) · path only · `mood size` / `mood age` for bytes / mtime · does not dump or edit mood text · Agent Ops → Agents for content."
    )
}


/// Zero-LLM per-agent mood.md file size (stat only; no dump/edit).
/// Sums on-disk bytes of every `agents/agent-*/mood.md` that exists.
pub fn format_mood_size_gateway() -> String {
    let agents = crate::config::Config::agents_dir();
    let mut total: u64 = 0;
    let mut count: usize = 0;
    if let Ok(entries) = std::fs::read_dir(&agents) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("agent-") {
                continue;
            }
            let mood = entry.path().join("mood.md");
            if let Ok(meta) = std::fs::metadata(&mood) {
                if meta.is_file() {
                    total = total.saturating_add(meta.len());
                    count = count.saturating_add(1);
                }
            }
        }
    }
    if count == 0 {
        return "**Mood file:** no `mood.md` yet · `mood path` for the file pattern · `mood age` for last write · Agent Ops → Agents for content."
            .to_string();
    }
    let label = crate::commands::disk_cleanup::format_bytes(total);
    if count == 1 {
        format!(
            "**Mood file:** **{label}** on disk · 1 per-agent `mood.md` · `mood path` for the file · `mood age` for last write · does not dump mood text."
        )
    } else {
        format!(
            "**Mood file:** **{label}** on disk · {count} per-agent `mood.md` files · `mood path` for the pattern · `mood age` for last write · does not dump mood text."
        )
    }
}


/// Zero-LLM per-agent mood.md age from newest file mtime (stat only; no dump/edit).
pub fn format_mood_age_gateway() -> String {
    let agents = crate::config::Config::agents_dir();
    let mut newest_ms: Option<u64> = None;
    let mut count: usize = 0;
    if let Ok(entries) = std::fs::read_dir(&agents) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("agent-") {
                continue;
            }
            let mood = entry.path().join("mood.md");
            if let Ok(meta) = std::fs::metadata(&mood) {
                if meta.is_file() {
                    count = count.saturating_add(1);
                    if let Ok(modified) = meta.modified() {
                        let ms = modified
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                        newest_ms = Some(match newest_ms {
                            Some(prev) => prev.max(ms),
                            None => ms,
                        });
                    }
                }
            }
        }
    }
    if count == 0 {
        return "**Mood file:** no `mood.md` yet · `mood path` for the file pattern · Agent Ops → Agents for content."
            .to_string();
    }
    let Some(ms) = newest_ms else {
        return "**Mood file** — could not read mtime on per-agent `mood.md`.".to_string();
    };
    let age = age_from_ms(ms);
    if count == 1 {
        format!(
            "**Mood file:** last write **{age}** ago · 1 per-agent `mood.md` · `mood path` for the file · `mood size` for on-disk bytes · does not dump mood text."
        )
    } else {
        format!(
            "**Mood file:** newest write **{age}** ago · {count} per-agent `mood.md` files · `mood path` for the pattern · `mood size` for on-disk bytes · does not dump mood text."
        )
    }
}


/// Zero-LLM shared soul.md file size (stat only; no dump/edit).
pub fn format_soul_size_gateway() -> String {
    let path = crate::config::Config::soul_file_path();
    if !path.exists() {
        return "**Soul file:** no `soul.md` yet · `soul path` for the file · Agent Ops → Agents for content."
            .to_string();
    }
    match std::fs::metadata(&path).map(|m| m.len()) {
        Ok(0) => {
            "**Soul file:** empty `soul.md` · `soul path` for the file · `soul age` for last write."
                .to_string()
        }
        Ok(bytes) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Soul file:** **{label}** on disk · shared persona · `soul path` for the file · `soul age` for last write · does not dump soul text."
            )
        }
        Err(e) => format!("**Soul file** — could not stat `soul.md`: {e}"),
    }
}


/// Zero-LLM shared soul.md age from file mtime (stat only; no dump/edit).
pub fn format_soul_age_gateway() -> String {
    let path = crate::config::Config::soul_file_path();
    if !path.exists() {
        return "**Soul file:** no `soul.md` yet · `soul path` for the file · Agent Ops → Agents for content."
            .to_string();
    }
    match std::fs::metadata(&path).and_then(|m| m.modified()) {
        Ok(modified) => {
            let ms = modified
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let age = age_from_ms(ms);
            format!(
                "**Soul file:** last write **{age}** ago · shared persona · `soul path` for the file · `soul size` for on-disk bytes · does not dump soul text."
            )
        }
        Err(e) => format!("**Soul file** — could not stat `soul.md`: {e}"),
    }
}


/// Zero-LLM shared soul.md path (config only; no dump/edit).
pub fn format_soul_path_gateway() -> String {
    let path = crate::config::Config::soul_file_path();
    let display = path.display().to_string();
    format!(
        "**Soul file:** `{display}` · shared persona · path only · `soul size` / `soul age` for bytes / mtime · does not dump or edit soul text · Agent Ops → Agents for content."
    )
}


/// Zero-LLM Ori vault directory size (recursive file bytes; no list/MCP/Ready).
pub fn format_ori_vault_size_gateway() -> String {
    use crate::config::Config;
    let raw = Config::ori_vault_path_raw();
    if raw.trim().is_empty() {
        return "**Ori vault:** not set · set `ORI_VAULT` / `MAC_STATS_ORI_VAULT` (or Settings Product) · `ori vault path` for the location · `ori vault age` for newest mtime · `/ori` for Ready / Off · does not list vault."
            .to_string();
    }
    let Some(dir) = Config::expand_user_path_str(raw.trim()) else {
        return "**Ori vault:** path not usable · check `ORI_VAULT` / `MAC_STATS_ORI_VAULT` · `ori vault path` for the configured string · `ori vault age` for newest mtime · does not list vault."
            .to_string();
    };
    match dir_total_bytes(&dir, 12_000) {
        Err(msg) if msg == "missing" => {
            "**Ori vault:** folder missing · `ori vault path` for the location · `ori vault age` for newest mtime · must contain `.ori` · `/ori` for Ready / Off · does not list vault."
                .to_string()
        }
        Err(e) => format!("**Ori vault** — could not scan: {e}"),
        Ok((0, 0)) => {
            "**Ori vault:** empty · `ori vault path` for the location · `ori vault age` for newest mtime · `/ori` for Ready / Off · does not list names."
                .to_string()
        }
        Ok((bytes, files)) => {
            let label = crate::commands::disk_cleanup::format_bytes(bytes);
            format!(
                "**Ori vault:** **{label}** on disk ({files} files) · `ori vault path` for the location · `ori vault age` for newest mtime · `/ori` for Ready / Off · does not list names or call MCP."
            )
        }
    }
}


/// Zero-LLM Ori vault directory age (newest file mtime; no list/MCP/Ready).
pub fn format_ori_vault_age_gateway() -> String {
    use crate::config::Config;
    let raw = Config::ori_vault_path_raw();
    if raw.trim().is_empty() {
        return "**Ori vault:** not set · set `ORI_VAULT` / `MAC_STATS_ORI_VAULT` (or Settings Product) · `ori vault path` for the location · `ori vault size` for on-disk bytes · `/ori` for Ready / Off · does not list vault."
            .to_string();
    }
    let Some(dir) = Config::expand_user_path_str(raw.trim()) else {
        return "**Ori vault:** path not usable · check `ORI_VAULT` / `MAC_STATS_ORI_VAULT` · `ori vault path` for the configured string · `ori vault size` for on-disk bytes · does not list vault."
            .to_string();
    };
    match dir_newest_mtime_ms(&dir, 12_000) {
        Err(msg) if msg == "missing" => {
            "**Ori vault:** folder missing · `ori vault path` for the location · `ori vault size` for on-disk bytes · must contain `.ori` · `/ori` for Ready / Off · does not list vault."
                .to_string()
        }
        Err(e) => format!("**Ori vault** — could not scan: {e}"),
        Ok((_, 0)) => {
            "**Ori vault:** empty · `ori vault path` for the location · `ori vault size` for on-disk bytes · `/ori` for Ready / Off · does not list names."
                .to_string()
        }
        Ok((Some(ms), files)) => {
            let age = age_from_ms(ms);
            if files == 1 {
                format!(
                    "**Ori vault:** last write **{age}** ago · 1 file · `ori vault path` for the location · `ori vault size` for on-disk bytes · `/ori` for Ready / Off · does not list names or call MCP."
                )
            } else {
                format!(
                    "**Ori vault:** newest write **{age}** ago · {files} files · `ori vault path` for the location · `ori vault size` for on-disk bytes · `/ori` for Ready / Off · does not list names or call MCP."
                )
            }
        }
        Ok((None, _)) => {
            "**Ori vault** — could not read mtime under the vault · `ori vault path` for the location · `ori vault size` for on-disk bytes."
                .to_string()
        }
    }
}


/// Zero-LLM Ori vault path (config/env only; no vault list / MCP / Ready chip).
pub fn format_ori_vault_path_gateway() -> String {
    use crate::config::Config;
    let raw = Config::ori_vault_path_raw();
    if raw.trim().is_empty() {
        return "**Ori vault:** not set · set `ORI_VAULT` / `MAC_STATS_ORI_VAULT` (or Settings Product) · path only · `ori vault size` for on-disk bytes · `ori vault age` for newest mtime · does not list vault or call MCP · `/ori` for Ready / Off."
            .to_string();
    }
    let display = match Config::expand_user_path_str(raw.trim()) {
        Some(p) => p.display().to_string(),
        None => raw.trim().to_string(),
    };
    format!(
        "**Ori vault:** `{display}` · must contain `.ori` · path only · `ori vault size` for on-disk bytes · `ori vault age` for newest mtime · does not list vault or call MCP · `/ori` for Ready / Off · Settings Product."
    )
}

