# Updating ~/.mac-stats from repo defaults

When asked to **update MD files (or agents) inside .mac-stats** or to **sync defaults into .mac-stats**:

- **Do not overwrite** existing files in `~/.mac-stats/` with repo content.
- **Merge** instead:
  - **New files / new agents**: Create missing files or agent directories and write default content (e.g. new `agent-005/` or new sections).
  - **Existing files**: Merge default content into the existing file — e.g. add missing sections (by heading), add new bullets to lists, or append new blocks at the end. Preserve the user’s existing content and customizations.
  - **agent.json**: Prefer merging or “write only if missing”; do not overwrite user edits (e.g. model, enabled).

So: **merge defaults into .mac-stats; do not simply overwrite.**

## Automatic prompt merge

At startup, `ensure_defaults()` runs. For **prompt files** (`~/.mac-stats/prompts/planning_prompt.md` and `execution_prompt.md`):

- If the file **does not exist**, the default is written (unchanged).
- If the file **exists**, it is **merged** with the bundled default: the file is split into paragraphs (by `\n\n`); each default paragraph is identified by its first-line key (trimmed, up to 80 chars). Any default paragraph whose key is not already present in the file is **appended**. User content is never overwritten; new sections from repo defaults are added automatically.

So after an app update, new prompt paragraphs (e.g. a new "Search → screenshot → Discord" rule in the planning prompt) appear in your existing `~/.mac-stats/prompts/` files on the next run without manual copy or overwrite.
