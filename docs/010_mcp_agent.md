# MCP Agent (Model Context Protocol)

The MCP agent lets Ollama use tools from any MCP server on the internet. You select which MCP server to use; the app connects, fetches the tool list, and adds it to the options Ollama sees. Ollama invokes MCP tools by replying with a line like `MCP: <tool_name> <arguments>`.

## Overview

- **Agent name**: MCP  
- **Invocation**: Ollama replies with one line: `MCP: <tool_name> <arguments>` (e.g. `MCP: fetch_url https://example.com` or `MCP: get_time {"timezone": "UTC"}`).  
- The app calls the configured MCP server via the Model Context Protocol (JSON-RPC), runs the tool, and injects the result back into the conversation so Ollama can summarize or answer.

When MCP is not configured, the agent is omitted from the list Ollama sees (or shown as "not configured") so the model does not try to use it.

## Setup

1. **Choose an MCP server**  
   Use **HTTP + SSE** for remote servers (URL) or **stdio** for local servers that run as a subprocess (e.g. Node/npx). See [List of MCP servers](#list-of-mcp-servers) and [Airbnb MCP server](#airbnb-mcp-server) below.

2. **Provide the server config** (first match wins):
   - **stdio (local subprocess)** — **MCP_SERVER_STDIO** env or in `.config.env`:
     ```bash
     # Pipe-separated: command|arg1|arg2 (no spaces around |)
     MCP_SERVER_STDIO=npx|-y|@openbnb/mcp-server-airbnb
     ```
     Used for servers that run via a command (e.g. `npx -y @openbnb/mcp-server-airbnb`). Requires Node.js if using npx.
   - **HTTP/SSE (remote)** — **MCP_SERVER_URL** env or in `.config.env`:
     ```bash
     MCP_SERVER_URL=https://your-mcp-server.example.com/sse
     # or (in .config.env only):
     MCP-SERVER-URL=https://your-mcp-server.example.com/sse
     ```
   - **.config.env** is read from: current directory, `src-tauri/`, or `~/.mac-stats/.config.env`.

If neither is set, MCP is not offered to Ollama.

## Behaviour

- When MCP is configured, the app connects to the MCP server (HTTP/SSE or stdio), lists available tools, and appends them to the agent descriptions sent to Ollama in both the **planning** and **execution** steps.
- Ollama can then reply with `MCP: <tool_name> <arguments>`. The app parses this, calls the MCP server’s `tools/call` (JSON-RPC), and injects the result into the conversation before the next Ollama call.
- The tool loop (Discord, scheduler, and when wired CPU-window flow) supports up to the same maximum tool iterations as other agents; each MCP call counts as one.

## Transport

- **HTTP + SSE**: Used for remote MCP servers. The client connects to the server’s SSE endpoint and sends JSON-RPC over HTTP as per the [MCP specification](https://modelcontextprotocol.io/specification/latest). Set **MCP_SERVER_URL** to the SSE URL.
- **stdio**: For local MCP servers that run as a subprocess. The app spawns the command (e.g. `npx -y @openbnb/mcp-server-airbnb`), sends JSON-RPC messages newline-delimited on stdin, and reads responses from stdout. Set **MCP_SERVER_STDIO** to `command|arg1|arg2` (pipe-separated, no spaces).

## Security

- The MCP server URL is read only from the environment or `.config.env` (and later optionally from Settings); it is not logged in full in production (e.g. mask in logs).
- Do not commit `.config.env` or expose the URL in client-side code or public repos.
- When connecting to remote MCP servers, use HTTPS and trust only servers you control or reputable providers.

## Where it’s used

- **Discord bot**: When MCP is configured, Ollama can output `MCP: <tool_name> <args>`. The app runs the tool and gives the result back to Ollama for the reply.
- **Scheduler**: Same pipeline; scheduled tasks that go through Ollama can use MCP tools.
- **CPU window chat**: When the CPU-window flow uses the same tool loop, MCP tools will be available there too.

## List of MCP servers

### Directories (discover more servers)

- **[MCPServersList](https://mcpserverslist.com/)** — Large directory of MCP servers by category.  
- **[Find My MCP](https://findmymcp.com/)** — Categorized directory (databases, cloud, finance, etc.).  
- **[MCP Central](https://mcpcentral.io/servers)** — Searchable directory; data also available as JSON API.  
- **[Official GitHub: modelcontextprotocol/servers](https://github.com/modelcontextprotocol/servers)** — Reference implementations and official server list.

### Curated reference servers (official)

From the [modelcontextprotocol/servers](https://github.com/modelcontextprotocol/servers) repo; many run locally via stdio; some may expose HTTP/SSE:

| Server    | Purpose                          | Notes                    |
|----------|-----------------------------------|--------------------------|
| **Fetch**| Fetch web content                 | Similar to FETCH_URL     |
| **Filesystem** | Read/write files           | Local paths               |
| **Git**  | Git operations                    | Repo state, diff, etc.    |
| **Memory** | Persistent memory/kv            | Cross-session context     |
| **Time** | Time and timezone                 | Current time, conversion  |
| **Sequential Thinking** | Step-by-step reasoning | Extended reasoning        |
| **Everything** | Local search (Everything app) | Windows-focused           |

Use the directories above to find HTTP/SSE endpoints for remote use; the official repo often documents stdio first.

### Airbnb MCP server (stdio — search listings and check availability)

The [Airbnb MCP server](https://mcpservers.org/servers/openbnb-org/mcp-server-airbnb) lets Ollama (and the Discord bot) search Airbnb listings and get details. It uses **stdio** (runs via Node/npx).

**Setup:**

1. Ensure Node.js is installed (for `npx`).
2. In `~/.mac-stats/.config.env` (or `.config.env` in cwd / `src-tauri/`):
   ```bash
   MCP_SERVER_STDIO=npx|-y|@openbnb/mcp-server-airbnb
   ```
3. Restart mac-stats. On the next Discord message (or CPU-window chat) that uses the agent loop, the app will spawn the server, list its tools, and add them to the agent list Ollama sees.

**Tools exposed to Ollama:**

- **`airbnb_search`** — Search listings by location, dates, guests, price. Parameters: `location` (required), `checkin`, `checkout`, `adults`, `children`, `minPrice`, `maxPrice`, etc.
- **`airbnb_listing_details`** — Get full details for a listing by ID (amenities, rules, link).

**Asking via Discord for availability:**

Once MCP is configured, you can ask the bot in natural language; Ollama will choose the MCP tool and arguments. Examples:

- *“Check Airbnb availability in Paris for 2 adults, check-in March 15 and check-out March 18.”*
- *“Search Airbnb in San Francisco for next weekend, max 150 per night.”*
- *“Get details for Airbnb listing ID 12345678.”*

Ollama will reply with something like `MCP: airbnb_search {"location":"Paris, France","checkin":"2025-03-15","checkout":"2025-03-18","adults":2}`; the app runs the tool and injects the results so the bot can summarize availability and links.

**Note:** The server is not affiliated with Airbnb, Inc.; it uses publicly available data. Respect rate limits and robots.txt (see server docs).

### Other community / remote-capable examples

- **Firecrawl**, **Browserbase**, **Exa**, **Context7**, **Cloudflare** — Listed on MCPServersList / Find My MCP; check each server’s docs for HTTP/SSE URL and auth.

When adding a new server, ensure it supports the transport you use (HTTP/SSE for internet, stdio for local).

## Implementation

- **Module**: `src-tauri/src/mcp/mod.rs` — MCP client for both transports: `list_tools(server_config)`, `call_tool(server_config, tool_name, arguments)`. If config starts with `stdio:` (from **MCP_SERVER_STDIO**), spawns the process and talks JSON-RPC over stdin/stdout; otherwise uses HTTP/SSE with the URL.
- **Config**: `get_mcp_server_url()` — env **MCP_SERVER_STDIO** (then `stdio:...`) or **MCP_SERVER_URL**, then `.config.env` (cwd, `src-tauri/`, `~/.mac-stats/.config.env`).
- **Ollama**: `src-tauri/src/commands/ollama.rs` — `build_agent_descriptions()` fetches MCP tools when configured and appends them to the agent list; planning and execution use this dynamic list; `parse_tool_from_response` recognizes `MCP:`; tool loop handles `MCP` with `send_status("Calling MCP tool…")` (Discord) and logs, then calls `mcp::call_tool`.

## References

- **All agents:** `docs/100_all_agents.md`
- **MCP specification:** [modelcontextprotocol.io](https://modelcontextprotocol.io/specification/latest)
- **Transports (HTTP/SSE):** [MCP Transports](https://modelcontextprotocol.io/specification/latest/basic/transports)
