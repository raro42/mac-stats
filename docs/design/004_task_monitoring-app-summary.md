## Global Context
### mac-stats
A local AI agent for macOS: Ollama chat, Discord bot, task runner, scheduler, and MCP—all on your Mac. No cloud, no telemetry. Lives in your menu bar—CPU, GPU, RAM, disk at a glance. Real-time, minimal, there when you look. Built with Rust and Tauri.

### Installation
#### DMG (recommended)
[Download latest release](https://github.com/raro42/mac-stats/releases/latest) → drag to Applications.

#### Build from source
```bash
git clone https://github.com/raro42/mac-stats.git && cd mac-stats && ./run
```
Or one-liner: `curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run`

#### If macOS blocks the app:
Gatekeeper may show "damaged" or block the unsigned app—the file is fine. Right-click the DMG → **Open**, then confirm. Or after install: `xattr -rd com.apple.quarantine /Applications/mac-stats.app`

## At a Glance
### Menu Bar
- CPU, GPU, RAM, disk at a glance; click to open the details window.

### AI Chat
- Ollama in the app or via Discord; FETCH_URL, BRAVE_SEARCH, PERPLEXITY_SEARCH, RUN_CMD, code execution, MCP.

### Discord Bot
- [Discord Bot Documentation](discord-bot.md)

## Tool Agents
Whenever Ollama is asked to decide which agent to use (planning step in Discord and scheduler flow), the app sends the **complete list of active agents**: the invocable tools below plus the **SCHEDULER** (informational; Ollama can recommend it for recurring or delayed tasks but cannot invoke it with a tool line). Ollama invokes tools by replying with exactly one line in the form `TOOL_NAME: <argument>`.

| Agent | Invocation | Purpose | Implementation |
|-------|------------|---------|----------------|
| **FETCH_URL** | `FETCH_URL: <full URL>` | Fetch a web page’s body as text (server-side, no CORS). | `commands/browser.rs` → `fetch_page_content()` (reqwest blocking client, 15s timeout). Used by Discord pipeline and by CPU-window chat (`ollama_chat_with_execution`). |
| **BRAVE_SEARCH** | `BRAVE_SEARCH: <search query>` | Web search via Brave Search API; results (titles, URLs, snippets) are injected back for Ollama to summarize. | `commands/brave.rs` → `brave_web_search()`. Requires `BRAVE_API_KEY` (env or `.config.env`). Used by Discord and (when wired) CPU-window agent flow. |
| **RUN_JS** | `RUN_JS: <JavaScript code>` | Execute JavaScript (e.g. in CPU window). | In **CPU window**: executed in

## Monitoring App
### Core Vision
A calm, premium macOS monitoring app focused on *signal over noise*.  
At-a-glance system health with optional, expandable insights and alerts.

### Core UI Principles
- **Minimal & Glass-like** macOS aesthetic (dark, translucent, polished)
- **Calm by default**, details on demand
- Strong visual hierarchy, no visual competition
- Everything optional and collapsible

### Main Dashboard (Always Visible)
#### Three Core Gauges (Hero Metrics)
These define the identity of the app and should never expand beyond three:

1. **Temperature**
2. **CPU Usage**
3. **CPU Frequency**

Rules:
- No 4th gauge (power, battery, externals do not belong here)
- Simple visuals (bars, subtle motion)
- No heavy charts in the main view

### Battery & Power
- Shown as a **horizontal status strip below the gauges**
- Not a gauge, not a card

Displays:
- Battery percentage
- Charging / discharging state
- Current system power draw (W)
- Estimated time remaining

Purpose:
- Contextual information
- Always visible
- Never dominant

### External Monitoring (Websites, APIs, Social)
External monitoring represents a **different mental mode** than hardware.

### Placement
- Collapsed **“External / Monitors” section**
- Or separate view/tab
- Never competes visually with hardware gauges

### Website Monitoring
- Start aggregated:
  - “9 / 10 sites up · Avg 240 ms”
- Expand only on click or error
- Per-site details only in expanded view

Metrics:
- Uptime
- Response time
- SSL / HTTP errors
- Downtime history (optional)

### Social Monitoring (Mastodon / X)
- Treated as external monitors
- Collapsed summary:
  - “2 new mentions”
- Expand to show:
  - Last few mentions
  - Links to original posts

Notes:
- Mastodon API is straightforward
- X (Twitter) API is more restricted and fragile

## Local AI Integration (Ollama)
Support for locally running Ollama instance as an endpoint.

### Chat Interface
- Optional chat interface to interact with local AI LLM
- Accessible via collapsed section or separate view
- Uses local Ollama API endpoint (default: `http://localhost:11434`)
- Configurable endpoint URL in settings

### Use Cases
- Ask questions about system metrics
- Get insights on monitoring data
- Natural language queries about alerts and status
- All processing stays local (privacy-focused)

### Configuration
- Ollama endpoint URL (defaults to localhost:11434)
- Model selection
- Optional API key if using remote Ollama instance
- Connection status indicator

## Alerts & Notifications
Alerts are essential but **invisible unless configured**.

### Principles
- Silent by default
- Rule-based
- Channel-agnostic core

### Supported Channels (Plugin-based)
- Telegram
- Slack
- Signal
- Mastodon
- WhatsApp (optional later)

### Alert Rules Examples
- Site down ≥ 5 minutes
- New mention ≥ 1 in 1 hour
- Battery < 5%
- Sustained high temperature

## Plugin System (Key to Scalability)
### Recommended Model: Script-Based Plugins
Plugins are executable scripts (bash / python) that output JSON.

Benefits:
- Extremely low barrier for developers
- No SDK required
- Easy to sandbox and validate

### Plugin Contract
- Executable script
- Outputs a strict JSON schema to stdout
- Optional sidecar config (TOML)

Core responsibilities:
- Scheduling
- Timeouts
- Parsing
- Caching
- Alerts

## Security & Credentials Management

### Critical Security Requirements
**Never expose API keys, tokens, or credentials in any form.**

### Secure Storage
- **macOS Keychain** for all secrets (API keys, tokens, passwords)
  - Use Keychain Services API (`Security.framework`)
  - Never store credentials in plaintext files
  - Never log credentials (mask in logs if necessary)
- Encrypt sensitive data at rest
- Secure token refresh flows for OAuth-based services

### Credential Scope
The following features require secure credential storage:
- **Alert Channels**: Telegram, Slack, Signal, Mastodon API tokens
- **Social Monitoring**: Mastodon/X OAuth tokens or API keys
- **Website Monitoring**: API keys for authenticated endpoints
- **Plugin Scripts**: Credentials passed securely to plugins (never in command line args)
- **Ollama Integration**: API keys if using remote instances

### Implementation Guidelines
- Validate and sanitize all plugin inputs/outputs to prevent credential leakage
- Use secure inter-process communication for plugins
- Never include credentials in URLs, logs, UI displays, or error messages
- Implement proper keychain access controls (user-only, app-specific)
- Provide secure credential entry UI (password fields, masked inputs)
- Support credential rotation and revocation

### Security Audit Points
- All external API calls must use secure credential retrieval
- Plugin execution must not expose credentials in process lists
- Logging must mask or exclude sensitive data
- Network traffic should use HTTPS/TLS for all external communications

## Visualization Strategy
- **SVG / CSS** for core UI and gauges
- **D3.js only in detail views**
  - History timelines
  - Heatmaps
  - Distributions
- Avoid heavy visuals in the main dashboard

## What NOT to Add (For Now)
To avoid bloat and loss of focus:
- Productivity scoring
- App time tracking
- Finance / health dashboards
- Social analytics
- AI-driven “life dashboards”

## Long-Term Direction
- Modular system (like Home Assistant / Grafana / Uptime Kuma)
- Calm overview + powerful drill-down
- Plugin ecosystem for advanced users
- Clear separation:
  - Metrics
  - Monitors
  - Alerts
  - UI

## Final Guiding Rule
If a feature is:
- Not glanceable
- Not actionable
- Not calm

→ It goes behind a click or into a plugin.

## Open tasks:

See **006-feature-coder/FEATURE-CODER.md** for the current FEAT backlog.

- ~~Re-evaluate plugin system for better performance and security~~ Deferred: future/backlog (plugin timeout and error messages already improved; further work tracked in FEATURE-CODER backlog when scoped).
- ~~Implement a more robust and user-friendly settings interface~~ **Done:** Settings modal with Monitors, Alert Channels, Schedules, Skills, and Ollama tabs.
- ~~Enhance the plugin ecosystem to support more advanced features~~ Deferred: future/backlog (see docs/016_skill_agent.md "Future/backlog").
- ~~Review and refine the alert system for better accuracy and customization options~~ **Done:** Background alert evaluation (60s interval), channel registration commands, cooldown mechanism.
- ~~Investigate and fix the issue with Gatekeeper blocking the app~~ **Done:** Documented in README (xattr -rd workaround) and docs/006_roadmap_ai_tasks.md (Installation section).