# Morning surprise — 2026-11-22

Overnight window 2026-11-21 20:00 → 2026-11-22 06:00 (local).

## Shipped

- **v0.1.2943** — kernel release `/kernel` · `/kernel-release` · `/kernel-version` · `/uname-r` · `/kern` + view/open NL. Bare `uname` / `/uname` stay with the agent; `/arch` stays machine arch.
- **v0.1.2942** — system locale `/locale` · `/lang` · `/language` + view/open NL. Bare `lang` stays with the agent; `/tz` stays timezone.
- **v0.1.2941** — Instant machine arch `/arch` · `/architecture` · `/uname-m` · `/machine-arch` · `/cpu-arch` · `/hw-arch` + view/open NL (`view arch`, `what architecture is this`, `cpu architecture`, `am i on arm64`). Cached `uname -m` (fallback `consts::ARCH`). Bare `architect` / theme Architect stay out; `/chip` stays Apple chip.
- **v0.1.2940** — Instant timezone `/timezone` · `/tz` · `/time-zone` · `/local-timezone` · `/local-tz` + view/open NL. Bare `/time` / `what time` / `view time` stay with the agent.
- **v0.1.2939** — Instant hardware serial `/serial` · `/serial-number` · `/hw-serial` · `/hardware-serial` · `/device-serial` · `/sn` + view/open NL.
- **v0.1.2938** — Instant login username `/username` · `/whoami` · `/login-name` · `/account-name` + view/open NL.
- **v0.1.2937** — Instant Mac model `/mac-model` · `/hardware-model` · `/machine-model` · `/model-id` · `/hw-model` + view/open NL (`what mac is this`).
- **v0.1.2936** — Instant hostname `/hostname` · `/computer-name` + Ori vault Ready fix (substantive-noun gate).
- **v0.1.2935** — Instant macOS version `/macos` · `/os` · `/sw-vers` · `/build` + view/open NL.
- **v0.1.2934** — SSD capacity `/ssd-size` + view/open NL.
- **v0.1.2933** — App process uptime `/app-uptime` + view/open NL.
- **v0.1.2932** — Apple chip identity `/chip` + view/open NL.
- Earlier in-window: RAM/SSD free/storage, ring slash NL, Load/Power/P-core chips, soft-state washes, idle Discord WARN gate.

## Tried / context
- Digester empty all ticks; design-review grace (Mac TCC); layout-daily data-poster marked tonight (Dark next calendar day).
- Cream/Swiss Send fill white-on-blue left intentional.
- Soft-state `.chat-input-container:focus-within` CRT remaps already present; skipped as stale fuel.

## Fitness
Operator can ask kernel release (`uname -r`) without hitting Ollama, bare `uname`, machine arch, or macOS ProductVersion — another identity chip with slash + view/open parity.
