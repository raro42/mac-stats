# Updating ~/.mac-stats from repo defaults

When asked to **update MD files (or agents) inside .mac-stats** or to **sync defaults into .mac-stats**:

- **Do not overwrite** existing files in `~/.mac-stats/` with repo content.
- **Merge** instead:
  - **New files / new agents**: Create missing files or agent directories and write default content (e.g. new `agent-005/` or new sections).
  - **Existing files**: Merge default content into the existing file — e.g. add missing sections (by heading), add new bullets to lists, or append new blocks at the end. Preserve the user’s existing content and customizations.
  - **agent.json**: Prefer merging or “write only if missing”; do not overwrite user edits (e.g. model, enabled).

So: **merge defaults into .mac-stats; do not simply overwrite.**
