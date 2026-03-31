//! Strip dangerous variables from the inherited environment before spawning agent-adjacent subprocesses.
//!
//! Aligns with OpenClaw `host-env-security-policy.json` **`blockedKeys` + `blockedPrefixes`**
//! (`isDangerousHostEnvVarName` in `host-env-security.ts`), plus **`BROWSER`**, **`GIT_EDITOR`**,
//! **`GIT_SEQUENCE_EDITOR`** (OpenClaw infra parity for editor / browser hijack via inherited env).

use std::process::Command as StdCommand;

/// Exact names (ASCII case-insensitive) blocked from the child environment, matching OpenClaw policy.
const BLOCKED_ENV_KEYS: &[&str] = &[
    "ANT_OPTS",
    "BASH_ENV",
    "BROWSER",
    "DOTNET_ADDITIONAL_DEPS",
    "DOTNET_STARTUP_HOOKS",
    "DYLD_FALLBACK_LIBRARY_PATH",
    "DYLD_FRAMEWORK_PATH",
    "DYLD_IMAGE_SUFFIX",
    "DYLD_INSERT_LIBRARIES",
    "DYLD_LIBRARY_PATH",
    "ENV",
    "GIT_EDITOR",
    "GIT_EXEC_PATH",
    "GIT_EXTERNAL_DIFF",
    "GIT_SEQUENCE_EDITOR",
    "GIT_TEMPLATE_DIR",
    "GLIBC_TUNABLES",
    "GCONV_PATH",
    "GRADLE_OPTS",
    "IFS",
    "JAVA_TOOL_OPTIONS",
    "JDK_JAVA_OPTIONS",
    "_JAVA_OPTIONS",
    "LD_LIBRARY_PATH",
    "LD_PRELOAD",
    "MAVEN_OPTS",
    "NODE_OPTIONS",
    "NODE_PATH",
    "PERL5LIB",
    "PERL5OPT",
    "PS4",
    "PYTHONBREAKPOINT",
    "PYTHONHOME",
    "PYTHONPATH",
    "RUBYLIB",
    "RUBYOPT",
    "SBT_OPTS",
    "SHELL",
    "SHELLOPTS",
    "SSLKEYLOGFILE",
];

const BLOCKED_ENV_PREFIXES: &[&str] = &["BASH_FUNC_", "DYLD_", "LD_"];

fn is_blocked_host_env_key(upper: &str) -> bool {
    if BLOCKED_ENV_KEYS
        .iter()
        .any(|blocked| *blocked == upper)
    {
        return true;
    }
    BLOCKED_ENV_PREFIXES
        .iter()
        .any(|prefix| upper.starts_with(prefix))
}

/// Remove dangerous host variables from the environment this [`StdCommand`] will pass to its child.
pub fn apply_host_exec_env_hardening(cmd: &mut StdCommand) {
    // Canonical spellings: strips vars added on this `Command` before hardening (e.g. tests, callers).
    for &key in BLOCKED_ENV_KEYS {
        cmd.env_remove(key);
    }
    // Case-sensitive keys from the parent: match OpenClaw rules in uppercase.
    for (k, _) in std::env::vars_os() {
        let Some(kstr) = k.to_str() else {
            continue;
        };
        let upper = kstr.to_ascii_uppercase();
        if is_blocked_host_env_key(&upper) {
            cmd.env_remove(k);
        }
    }
}

/// Same as [`apply_host_exec_env_hardening`] for `tokio::process::Command`.
pub fn apply_host_exec_env_hardening_tokio(cmd: &mut tokio::process::Command) {
    for &key in BLOCKED_ENV_KEYS {
        cmd.env_remove(key);
    }
    for (k, _) in std::env::vars_os() {
        let Some(kstr) = k.to_str() else {
            continue;
        };
        let upper = kstr.to_ascii_uppercase();
        if is_blocked_host_env_key(&upper) {
            cmd.env_remove(k);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardened_child_does_not_see_dyld_insert_libraries() {
        let mut cmd = StdCommand::new("sh");
        cmd.env_clear();
        cmd.env("DYLD_INSERT_LIBRARIES", "/tmp/should-not-appear.dylib");
        cmd.env("PATH", "/usr/bin:/bin");
        apply_host_exec_env_hardening(&mut cmd);
        cmd.args([
            "-c",
            "if [ -n \"${DYLD_INSERT_LIBRARIES+x}\" ]; then echo BAD; else echo OK; fi",
        ]);
        let out = cmd.output().expect("spawn sh");
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "OK");
    }

    #[test]
    fn hardened_child_does_not_see_pythonpath() {
        let mut cmd = StdCommand::new("sh");
        cmd.env_clear();
        cmd.env("PYTHONPATH", "/tmp/evil");
        cmd.env("PATH", "/usr/bin:/bin");
        apply_host_exec_env_hardening(&mut cmd);
        cmd.args([
            "-c",
            "if [ -n \"${PYTHONPATH+x}\" ]; then echo BAD; else echo OK; fi",
        ]);
        let out = cmd.output().expect("spawn sh");
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "OK");
    }

    #[test]
    fn hardened_child_does_not_see_ld_library_path_prefix() {
        let mut cmd = StdCommand::new("sh");
        cmd.env_clear();
        cmd.env("LD_LIBRARY_PATH", "/tmp");
        cmd.env("PATH", "/usr/bin:/bin");
        apply_host_exec_env_hardening(&mut cmd);
        cmd.args([
            "-c",
            "if [ -n \"${LD_LIBRARY_PATH+x}\" ]; then echo BAD; else echo OK; fi",
        ]);
        let out = cmd.output().expect("spawn sh");
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "OK");
    }
}
