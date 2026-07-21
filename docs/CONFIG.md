# Configuration

All runtime data stays on your Mac under **`~/.mac-stats/`**. There is **no cloud telemetry**.

## Layout

```
~/.mac-stats/
├── config.json            # Window, timeouts, harness mode, browser, …
├── .config.env            # Secrets — never commit
├── discord_channels.json
├── schedules.json
├── agents/                # soul.md, memory.md, skills, prompts
├── task/
├── session/
├── screenshots/
└── debug.log
```

After editing secrets for LaunchAgent / `/Applications` installs, run:

```bash
./scripts/sync-home-config-env.sh
```

(from a clone) so keys from `src-tauri/.config.env` merge into `~/.mac-stats/.config.env`.

## Secrets

| Variable | Purpose |
|----------|---------|
| `DISCORD_BOT_TOKEN` or `DISCORD-USER*-TOKEN` | Discord bot |
| `REDMINE_URL` / `REDMINE_API_KEY` | Redmine |
| `BRAVE_API_KEY` | Brave Search |
| `PERPLEXITY_API_KEY` | Perplexity (also Keychain via Settings) |
| `MASTODON_INSTANCE_URL` / `MASTODON_ACCESS_TOKEN` | Mastodon |

**Prefer Keychain** where the UI supports it (Discord, Perplexity). `.config.env` is plain text — do not commit it; keep it out of backups you share.

## Harness / agent

In `config.json`:

- `agentHarnessMode`: `"direct"` (default) or `"classic"`
- `agentNativeTools`: `true` (default)

Details: [039_werner_harness_parity.md](039_werner_harness_parity.md).

## Logs

```bash
tail -f ~/.mac-stats/debug.log
```

Filter console noise with `MAC_STATS_LOG` — [039_mac_stats_log_subsystems.md](039_mac_stats_log_subsystems.md).
