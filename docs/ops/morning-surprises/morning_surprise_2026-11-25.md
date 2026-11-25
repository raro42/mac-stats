# Morning surprise — 2026-11-25

Overnight autoresearch (Track B) — Ralf.

## Shipped
- **v0.1.3092** — Discord/operator `/powerstate` · `/iokit-drivers` · `/power-state` (+ view/open NL): summarized IOKit driver power states from `pmset -g powerstate` (driver count · root · top descs · below-max). Bare `powerstate` stays with the agent; `/systemstate` and `/userclients` stay put.
- **v0.1.3091** — Memory scrub: leftover HTML `<thead>` / `<tbody>` scaffolding + session-compactor / `**DROP:**` / “hallucinated or unverified claims” instruction lines.
- **v0.1.3090** — Memory scrub: MS_UNTRUSTED `<th>` / `<tr>` / `\text{…}` wrappers, “Treat everything until…”, “untrusted data points”, and “unverified claims” spam (Discord memory was re-injecting these after the old `<td>`-only filter; startup scrubbed 12 lines in one file).
- **v0.1.3089** — Schedules / tasks / CLI: stop wrapping operator-local text in `MS_UNTRUSTED` (#29).
- **v0.1.3088–3082** — Keyboard-hint polish train (filter chips · Disk Cleanup lists · Settings bodies · header/footer · Credentials) earlier in the window.

## Tried / closed
- Digester open empty all night; design-review surfaces in grace (TCC blocks capture).
- Layout daily: data-poster marked; **Dark** waits for next calendar day.
- SoftDisabled…pmhistory soft-state closed; `pmset -g disk` == `custom`; `uuidlog` blocks.
- Tip train closed for the night after 3088; then memory hygiene 3090–3091; then `/powerstate` summary 3092.

## Next fuel
- Dark layout-daily (next calendar quota).
- Remaining non-blocking `pmset -g` only if clearly distinct — not keep-pNN clones.
