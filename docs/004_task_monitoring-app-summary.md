# Monitoring App – Design & Architecture Summary

## Core Vision
A calm, premium macOS monitoring app focused on *signal over noise*.  
At-a-glance system health with optional, expandable insights and alerts.

> “A calm, high-fidelity system and service monitor that tells you when something matters — and stays quiet otherwise.”

---

## Core UI Principles
- **Minimal & Glass-like** macOS aesthetic (dark, translucent, polished)
- **Calm by default**, details on demand
- Strong visual hierarchy, no visual competition
- Everything optional and collapsible

---

## Main Dashboard (Always Visible)
### Three Core Gauges (Hero Metrics)
These define the identity of the app and should never expand beyond three:

1. **Temperature**
2. **CPU Usage**
3. **CPU Frequency**

Rules:
- No 4th gauge (power, battery, externals do not belong here)
- Simple visuals (bars, subtle motion)
- No heavy charts in the main view

---

## Battery & Power
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

---

## External Monitoring (Websites, APIs, Social)
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

---

## Social Monitoring (Mastodon / X)
- Treated as external monitors
- Collapsed summary:
  - “2 new mentions”
- Expand to show:
  - Last few mentions
  - Links to original posts

Notes:
- Mastodon API is straightforward
- X (Twitter) API is more restricted and fragile

---

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

---

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

---

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

---

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

---

## Visualization Strategy
- **SVG / CSS** for core UI and gauges
- **D3.js only in detail views**
  - History timelines
  - Heatmaps
  - Distributions
- Avoid heavy visuals in the main dashboard

---

## What NOT to Add (For Now)
To avoid bloat and loss of focus:
- Productivity scoring
- App time tracking
- Finance / health dashboards
- Social analytics
- AI-driven “life dashboards”

---

## Long-Term Direction
- Modular system (like Home Assistant / Grafana / Uptime Kuma)
- Calm overview + powerful drill-down
- Plugin ecosystem for advanced users
- Clear separation:
  - Metrics
  - Monitors
  - Alerts
  - UI

---

## Final Guiding Rule
If a feature is:
- Not glanceable
- Not actionable
- Not calm

→ It goes behind a click or into a plugin.
