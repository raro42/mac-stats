# Morning surprise — 2026-10-06

Overnight Track B put the Changelog body keyboard hint in the theme HTML.

## Shipped

| Version | What |
| --- | --- |
| **v0.1.1385** | Changelog body keyboard hint sits in the theme HTML at the top of the changelog body. It no longer pops in after JavaScript loads. It stays hidden until two versions are on screen. The line says how to move across those version headings. Keyboard tips stay out of the layout. |
| **v0.1.1384** | Agent Ops row-selection Tips keyboard hint sits in the theme HTML under the tab-bar hint. It no longer pops in after JavaScript loads. The line says how to move across tabs, filters, and list rows. Keyboard tips stay out of the layout. |
| **v0.1.1383** | AI Chat message-list keyboard hint sits in the theme HTML above the message list. It no longer pops in after JavaScript loads. It stays hidden until turns exist. The line says how to move across those messages. Keyboard tips stay out of the layout. |
| **v0.1.1382** | Perplexity setup keyboard hint sits in the theme HTML under key · Save key. It no longer pops in after JavaScript. Hidden until the setup panel is on screen. |
| **v0.1.1381** | Monitors detail keyboard hint ships with Check now · Remove (same paint; hidden until both buttons are on screen). |

## Tried / context

- Digester open stayed empty; design review still in grace.
- Fuel from loop_backlog: changelog-body-toolbar keyboard hint still appeared after JS.
- Debug.log: Having-fun idle Ollama Connection refused / circuit open (best-effort; not product panic).

## For Ralf

Open Changelog (footer version chip): with two or more versions loaded, the move-between-versions hint should already be in the DOM at the top of the body, not appear only after JS injects it. While only the loading line is on screen, it stays hidden.

## Tick notes

- ~21:50 tick: Changelog body keyboard hint first paint in theme HTML (**v0.1.1385**); sync-dist + ratchet keep.
- ~21:26 tick: Agent Ops row Tips keyboard hint first paint in theme HTML (**v0.1.1384**); sync-dist + ratchet keep.
- ~21:02 tick: AI Chat message-list keyboard hint first paint in theme HTML (**v0.1.1383**); sync-dist + ratchet keep.
- ~20:40 tick: Perplexity setup keyboard hint first paint in theme HTML (**v0.1.1382**); sync-dist + ratchet keep.
- ~20:10 tick: Monitors detail keyboard hint with Check now · Remove (**v0.1.1381**); sync-dist + ratchet keep.
