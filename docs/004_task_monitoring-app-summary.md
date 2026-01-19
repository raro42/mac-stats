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
