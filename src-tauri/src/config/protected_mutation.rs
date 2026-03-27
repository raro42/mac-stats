//! **Protected subtrees of `~/.mac-stats/config.json` for automated / programmatic mutation.**
//!
//! Any code path that merges into or overwrites the config file (Tauri commands, future MCP
//! `config.apply`-style APIs, import hooks, etc.) must call [`reject_if_protected_config_json_changed`]
//! on the JSON **before** write: current on-disk snapshot vs the would-be next document. If any
//! listed top-level key’s value differs (deep `serde_json::Value` equality), the write is refused
//! with a clear error. Operators can still edit `config.json` directly or use a future dedicated
//! bypass (out of scope for the agent layer).
//!
//! ## Inventory (security-sensitive)
//!
//! Execution surface and tool gates: `runJsEnabled`, `browserToolsEnabled`.
//!
//! Network / SSRF policy: `ssrfAllowedHosts`, `strictSsrfRejectWhenProxyEnv`, `browserAllowedDomains`,
//! `browserBlockedDomains`.
//!
//! Outbound attachments: `extraAttachmentRoots`.
//!
//! Browser/CDP attack surface: `browserCdpPort`, `browserChromiumExecutable`,
//! `browserChromiumUserDataDir`, `browserCdpProxyUsername`, `browserCdpProxyPassword`,
//! `browserCdpGrantPermissions`, `browserArtifactMaxBytes`, CDP trace toggles.
//!
//! Compaction / reset hooks (arbitrary commands): `beforeResetTranscriptPath`, `beforeResetHook`,
//! `beforeCompactionTranscriptPath`, `beforeCompactionHook`, `afterCompactionHook`.
//!
//! Tool-loop and context limits: `toolLoopDetection`, `maxConsecutiveToolFailures`,
//! `agentRouterMaxToolIterationsDiscord`, `agentRouterMaxToolIterationsUi`,
//! `agentRouterMaxToolIterationsRemote`, `agentRouterTurnTimeoutSecsDiscord`,
//! `agentRouterTurnTimeoutSecsUi`, `agentRouterTurnTimeoutSecsRemote`,
//! `contextOverflowTruncateEnabled`, `contextOverflowMaxResultChars`,
//! `normalizeUntrustedHomoglyphs`, `proactiveToolResultContextBudgetEnabled`,
//! `proactiveContextBudgetHeadroomRatio`, `proactiveContextMaxResultChars`.
//!
//! Heartbeat can post to Discord / run checklist: `heartbeat`.

use serde_json::Value;

/// Top-level keys treated as immutable for programmatic config writes (see module doc).
const PROTECTED_TOP_LEVEL_KEYS: &[&str] = &[
    "runJsEnabled",
    "browserToolsEnabled",
    "ssrfAllowedHosts",
    "strictSsrfRejectWhenProxyEnv",
    "extraAttachmentRoots",
    "browserAllowedDomains",
    "browserBlockedDomains",
    "browserCdpPort",
    "browserChromiumExecutable",
    "browserChromiumUserDataDir",
    "browserCdpProxyUsername",
    "browserCdpProxyPassword",
    "browserCdpGrantPermissions",
    "browserArtifactMaxBytes",
    "browserCdpTraceEnabled",
    "browserCdpTraceWallClockMinutes",
    "browserCdpTraceMaxFileBytes",
    "browserCdpTraceMaxRetainedFiles",
    "beforeResetTranscriptPath",
    "beforeResetHook",
    "beforeCompactionTranscriptPath",
    "beforeCompactionHook",
    "afterCompactionHook",
    "toolLoopDetection",
    "maxConsecutiveToolFailures",
    "agentRouterMaxToolIterationsDiscord",
    "agentRouterMaxToolIterationsUi",
    "agentRouterMaxToolIterationsRemote",
    "agentRouterTurnTimeoutSecsDiscord",
    "agentRouterTurnTimeoutSecsUi",
    "agentRouterTurnTimeoutSecsRemote",
    "contextOverflowTruncateEnabled",
    "contextOverflowMaxResultChars",
    "normalizeUntrustedHomoglyphs",
    "proactiveToolResultContextBudgetEnabled",
    "proactiveContextBudgetHeadroomRatio",
    "proactiveContextMaxResultChars",
    "heartbeat",
];

/// Refuse the mutation when any protected top-level key differs between `before` and `after`
/// (including `null` vs missing — both are explicit `Value` states in the merged document).
pub fn reject_if_protected_config_json_changed(
    before: &Value,
    after: &Value,
) -> Result<(), String> {
    for key in PROTECTED_TOP_LEVEL_KEYS {
        let b = before.get(*key);
        let a = after.get(*key);
        if b != a {
            tracing::warn!(
                "config: protected-mutation guard rejected write (protected key `{}` would change)",
                key
            );
            return Err(format!(
                "config mutation refused: security-sensitive key `{}` would change; edit ~/.mac-stats/config.json directly or use operator tools outside the agent/automation path",
                key
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn benign_non_protected_change_allowed() {
        let before = json!({"runJsEnabled": false, "windowDecorations": true});
        let after = json!({"runJsEnabled": false, "windowDecorations": false});
        assert!(reject_if_protected_config_json_changed(&before, &after).is_ok());
    }

    #[test]
    fn protected_toggle_rejected() {
        let before = json!({"runJsEnabled": false});
        let after = json!({"runJsEnabled": true});
        let err = reject_if_protected_config_json_changed(&before, &after).unwrap_err();
        assert!(err.contains("runJsEnabled"), "{}", err);
    }

    #[test]
    fn full_document_same_protected_ok() {
        let before = json!({
            "runJsEnabled": false,
            "browserToolsEnabled": true,
            "windowDecorations": true
        });
        let mut after = before.clone();
        after["windowDecorations"] = json!(false);
        assert!(reject_if_protected_config_json_changed(&before, &after).is_ok());
    }

    #[test]
    fn nested_tool_loop_detection_change_rejected() {
        let before = json!({
            "toolLoopDetection": {"enabled": true, "historySize": 12}
        });
        let after = json!({
            "toolLoopDetection": {"enabled": true, "historySize": 99}
        });
        assert!(reject_if_protected_config_json_changed(&before, &after).is_err());
    }
}
