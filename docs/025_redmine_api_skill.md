# Redmine API — skill reference

Use **REDMINE_API** for Redmine. Reply with exactly one line: `REDMINE_API: <METHOD> <path> [body]`.

**When the user asks to update a ticket or add a comment:** use **PUT** with `notes`. If you first do a GET to see the issue, your **very next** reply must be `REDMINE_API: PUT /issues/<id>.json {"issue":{"notes":"..."}}` — do not reply with only a summary.

## GET (read)

- Full issue with comments and attachments:  
  `REDMINE_API: GET /issues/1234.json?include=journals,attachments`
- My open issues:  
  `REDMINE_API: GET /issues.json?assigned_to_id=me&status_id=open`
- Project issues:  
  `REDMINE_API: GET /issues.json?project_id=ID&status_id=open&limit=25`
- List projects:  
  `REDMINE_API: GET /projects.json`
- **Search issues by keyword** (subject, description, journals):  
  `REDMINE_API: GET /search.json?q=<keyword>&issues=1&limit=100`  
  Use when the user asks to search or list tickets by topic (e.g. "datadog", "monitoring"). Do **not** use `GET /issues.json?search=...` — the issues list API has no full-text search param; use `/search.json` for keyword search.

Always use the `.json` suffix.

## POST — create a new ticket

Use these **valid default values** for creating an issue (this Redmine instance):

| Field            | Default | Notes |
|------------------|---------|--------|
| `project_id`     | 2       | |
| `tracker_id`     | 1       | |
| `status_id`      | 1       | |
| `priority_id`    | 2       | |
| `is_private`     | false   | (0 in JSON) |
| `assigned_to_id` | 5       | optional |
| `subject`        | (required) | from user |
| `description`    | (optional) | from user or `""` |

**Create the issue** with:

`REDMINE_API: POST /issues.json {"issue":{"project_id":2,"tracker_id":1,"status_id":1,"priority_id":2,"is_private":false,"subject":"Your subject","description":""}}`

With description and assignee:

`REDMINE_API: POST /issues.json {"issue":{"project_id":2,"tracker_id":1,"status_id":1,"priority_id":2,"is_private":false,"subject":"Title","description":"Optional description.","assigned_to_id":5}}`

The API returns the created issue (with `id`) on success.

## PUT — add a comment to a ticket

To **add a comment** (journal note) to an issue, use **PUT** with a JSON body containing `notes`:

- One line, path then space then JSON (no newline inside the line):
  `REDMINE_API: PUT /issues/1234.json {"issue":{"notes":"Your comment text here."}}`

- Private note (only visible to users with permission):
  `REDMINE_API: PUT /issues/1234.json {"issue":{"notes":"Internal remark.","private_notes":true}}`

- Escape double quotes inside the comment if needed, or use single quotes for the outer string in your head; the body must be valid JSON, so the notes value is a JSON string.

Examples:

- Add a short status update:  
  `REDMINE_API: PUT /issues/5678.json {"issue":{"notes":"Checked with the team. We will ship next week."}}`
- Add a private note:  
  `REDMINE_API: PUT /issues/5678.json {"issue":{"notes":"Internal: need to verify with legal.","private_notes":true}}`

After a successful PUT, the API returns the updated issue. You can confirm the comment was added and relay that to the user.

## Summary

| Action           | Invocation |
|------------------|------------|
| Get issue        | `REDMINE_API: GET /issues/<id>.json?include=journals,attachments` |
| Search by keyword | `REDMINE_API: GET /search.json?q=<keyword>&issues=1&limit=100` |
| Create issue     | POST /issues.json with `{"issue":{"project_id":2,"tracker_id":1,"status_id":1,"priority_id":2,"is_private":false,"subject":"...","description":""}}` (optional: `assigned_to_id`:5) |
| Add comment      | `REDMINE_API: PUT /issues/<id>.json {"issue":{"notes":"..."}}` |
| Add private note | `REDMINE_API: PUT /issues/<id>.json {"issue":{"notes":"...","private_notes":true}}` |

Config: `REDMINE_URL` and `REDMINE_API_KEY` in env or `~/.mac-stats/.config.env`.

**Context injection:** When Redmine is configured, the app fetches projects, trackers, statuses, and priorities from your Redmine and injects them into the agent context (refreshed every 5 minutes). The LLM can then resolve "Create in AMVARA" or "project AMVARA" to the correct `project_id` from the list (e.g. "2=AMVARA (amvara)"). Use the names or ids from that context when building the POST body.
