# Morning surprise — 2026-10-06

Overnight Track B (20:00–06:00 local).

## Shipped

| Version | What |
| --- | --- |
| **v0.1.1381** | Monitors detail keyboard hint ships with Check now · Remove (same paint; hidden until both buttons are on screen). |
| **v0.1.1382** | Perplexity setup keyboard hint sits in the theme HTML under key · Save key. It no longer pops in after JavaScript. Hidden until the setup panel is on screen. |

## Tried / context

- Digester open stayed empty; design review still in grace.
- Fuel from loop_backlog: perplexity-setup kb hint JS pop-in.
- Debug.log: Having-fun idle Ollama Connection refused / circuit open (best-effort; not product panic).

## For Ralf

Open Perplexity without a key configured (or clear the key) and expand the section: the setup toolbar keyboard tip should already be in the DOM under key · Save key, not appear only after JS injects it.
