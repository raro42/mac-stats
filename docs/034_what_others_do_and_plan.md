# What others are doing — research and proposed plan

Short review of comparable projects and a concrete plan for mac-stats. No commitment to implement everything; use this to pick next steps.

---

## 1. System monitors (menu bar / stats)

| Project | Stack | What they do | Gap vs mac-stats |
|--------|--------|---------------|-------------------|
| **Stats** (exelban/stats) | Swift, Xcode, SMC | CPU, GPU, RAM, disk, network, battery, sensors, fans, Bluetooth, widgets; 36k+ stars; Homebrew; multi-language; “disable Sensors/Bluetooth to cut CPU up to 50%” | No AI. No Discord. No browser automation. Strong on sensors/widgets and localization. |
| **iStat Menus** etc. | Native | Paid; rich widgets, notifications, fan control | Different product (commercial, no agent). |

**Takeaway:** mac-stats is differentiated by **local AI + Discord + tasks + browser**, not by out-featuring Stats on sensors. Keeping metrics lean and CPU low remains important; adding more sensor modules (like Stats) is optional and costly.

---

## 2. Browser automation for AI agents

| Project | Stack | What they do | Gap vs mac-stats |
|--------|--------|---------------|-------------------|
| **browser-use** | Python, CDP, Playwright-like | LLM-driven browser agent: navigate, click, type, extract, screenshot; custom tools; CLI; “tell computer what to do”; Cloud option (stealth, scale); Cursor/Claude skills; env timeouts for nav/click/type | Full actor API (Page, Element, Mouse); vision; structured output; fallback LLM; max_actions_per_step, max_failures. We have a smaller tool set (NAVIGATE, CLICK, INPUT, SCROLL, EXTRACT, SEARCH_PAGE, SCREENSHOT) and completion verification. |
| **Playwright + custom** | Various | Standard approach: launch browser, CDP or Playwright API, feed pages to LLM | We’re in the same family; we use CDP + headless by default for Discord/scheduler. |

**Takeaways:**

- **Headless by default for remote** (Discord, scheduler) is aligned with “no popups”; we did that and fixed retries.
- **Configurable timeouts** (nav, click, type) like browser-use’s env vars could reduce flakiness on slow SPAs without code churn; we have some ad-hoc timeouts.
- **Structured output** (e.g. extract “price + name” into a schema) is a possible future tool (e.g. BROWSER_EXTRACT_STRUCTURED) if we want to go that way.
- **Cloud/stealth** is out of scope for mac-stats (local-first).

---

## 3. Ollama ecosystem

| Area | What others do | mac-stats |
|------|----------------|-----------|
| **Integrations** | Open WebUI, LibreChat, Discord bots, Telegram, Slack, Raycast, code editors (Cline, Continue), OpenClaw (multi-platform assistant) | Discord bot + in-app chat; same pipeline for both; `discord run-ollama` for testing. |
| **API** | REST /api/chat, libraries (ollama-rs, etc.) | We use HTTP POST to /api/chat; session/connection reuse deferred (see 006). |
| **Tools** | Many projects add tools (RAG, run command, browse). We have FETCH_URL, BROWSER_*, RUN_CMD, SCHEDULE, TASK, MCP, etc. | Already rich; main differentiator is “all in one app” (monitor + Discord + scheduler + tasks + browser). |

**Takeaway:** We’re not missing a major Ollama integration pattern; we can improve reliability and docs (e.g. 007, 033) and optionally connection reuse later.

---

## 4. Tauri and system tray

- Tauri has **system tray** (icon, menu, events, update icon/menu at runtime). We use a **menu bar** (macOS NSStatusItem), which is the right primitive on macOS.
- **Prevent exit on window close** is documented (keep backend or frontend running); we already keep the app running when the CPU window is closed.

No change needed for tray vs menu bar.

---

## 5. Proposed plan (priorities)

### A. Harden and document (high value, low risk)

1. **Docs vs code** — Already audited in 033; keep 033 updated when behavior or intervals change.
2. **Testing pipeline** — `mac_stats discord run-ollama "<question>"` is the main way to test without Discord; document in 007 and 033 (done). Optionally add one or two more examples to 007 (e.g. “screenshot this URL” one-liner).
3. **Retry / headless** — Done: from_remote ⇒ headless unless user asks for visible; retries stay headless.
4. **Ollama connection reuse** — Deferred in 006; when touching that code path, consider reusing one reqwest client for chat to avoid repeated connection setup.

### B. Browser agent (medium effort, high impact)

1. **Timeouts** — Make navigation/click/type timeouts configurable (e.g. env or `~/.mac-stats/config.json`) so slow or heavy SPAs don’t require code changes. Align with browser-use-style env vars or a single “browser timeouts” section in config.
2. **Session recovery** — We already detect “new tab” and tell the model to re-navigate; keep refining messages so the model reliably recovers.
3. **Optional: BROWSER_EXTRACT with schema** — If we ever want “extract these fields” (e.g. price, title) we could add a structured variant; low priority until a concrete use case appears.

### C. Metrics and menu bar (optional)

1. **CPU/memory** — Keep current optimization (lazy window, 20s/30s for temp/freq, process cache). No need to match Stats’ sensor count.
2. **Themes and copy** — Already multiple themes; version from Cargo.toml (done). No new work unless we add a new theme or change copy.

### D. Integrations (only if needed)

1. **Mail / WhatsApp / Google Docs** — In roadmap (006) as future phases; not proposed for immediate work.
2. **Other messengers** — Same pattern as Discord (credentials, handler, optional tool protocol); add when there’s a real use case.

### E. Experiments (try and decide)

1. **Prompting** — Borrow from browser-use’s prompting guide: “be specific”, “name actions”, “error recovery” in our execution/planning prompts (e.g. in `~/.mac-stats/prompts/` or defaults). Could reduce tool misuse and retries.
2. **Fallback model** — browser-use has `fallback_llm` on rate limit/5xx; we could add “on Ollama 503/timeout, retry once with a smaller model” if we see real 503s in the wild.
3. **Cost / observability** — We’re local-first so “cost” is compute; optional “log token usage” for power users or debugging. No urgency.

---

## 6. Summary table

| Direction | Action | Effort | Impact |
|----------|--------|--------|--------|
| Docs | Keep 033 + 007 aligned; add 1–2 run-ollama examples | Low | Clarity, onboarding |
| Browser | Configurable timeouts (env or config) | Medium | Fewer SPA/timeout failures |
| Browser | Better prompting (specificity, actions, recovery) in prompts | Low | Fewer wrong tools, retries |
| Ollama | Connection reuse (single client for chat) | Low–medium | Latency/resource use |
| Metrics | No change | — | Stay lean |
| New integrations | Only when needed (Mail, etc.) | — | Roadmap only |

---

## 7. Next steps (concrete)

1. **Short term** — Nothing mandatory; iteration (review, code, test, document) is done for headless/retry and docs.
2. **When touching browser agent** — Add configurable timeouts and consider one prompt iteration (specificity + error recovery) in default execution prompt.
3. **When touching Ollama client** — Consider reusing one HTTP client for chat (see 006).
4. **Revisit this doc** — When adding a new big feature (e.g. Mail, new channel) or when someone asks “what should we do next?”.

---

*Sources: exelban/stats (README), browser-use (README, docs.browser-use.com), ollama/ollama (README), Tauri system tray guide. Doc created 2026-03-02.*
