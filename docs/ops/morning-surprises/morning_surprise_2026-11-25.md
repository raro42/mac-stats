# Morning surprise — 2026-11-25

Overnight autoresearch (Track B) — Ralf.

## Shipped
- **v0.1.3093** — Scheduler delivery awareness: skip MS_UNTRUSTED refusal replies and scrub legacy refusal rows on load so Last delivery / CPU chat continuity stay clean after #29 (live file dropped 5 refusal rows).
- **v0.1.3092** — Discord/operator `/powerstate` · `/iokit-drivers` · `/power-state` (+ view/open NL): summarized IOKit driver power states from `pmset -g powerstate`.
- **v0.1.3091** — Memory scrub: leftover HTML `<thead>` / `<tbody>` scaffolding + session-compactor / `**DROP:**` / “hallucinated or unverified claims” instruction lines.
- **v0.1.3090** — Memory scrub: MS_UNTRUSTED `<th>` / `<tr>` / `\text{…}` wrappers and related spam.
- **v0.1.3089** — Schedules / tasks / CLI: stop wrapping operator-local text in `MS_UNTRUSTED` (#29).
- **v0.1.3088–3082** — Keyboard-hint polish train earlier in the window.

## Tried / closed
- Digester open empty all night; design-review surfaces in grace (TCC blocks capture).
- Layout daily: data-poster marked; **Dark** waits for next calendar day.
- SoftDisabled…pmhistory soft-state closed; `pmset -g disk` == `custom`; `uuidlog` blocks; tip train closed after 3088.
- #29 wrap-stop shipped as 3089; residual delivery-awareness refusal poison closed as 3093.

## Next fuel
- Dark layout-daily (next calendar quota).
- Remaining non-blocking `pmset -g` only if clearly distinct — not keep-pNN clones.
