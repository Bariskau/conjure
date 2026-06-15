# Conjure

![Conjure tools UI](docs/tools.png)

Conjure turns local PowerShell or sh scripts into typed MCP tools. The installed
`conjure` command runs the Vue UI, the local API, and the stdio MCP server.

GitHub Pages: https://bariskau.github.io/conjure/

Detailed product docs and screenshots live in [`docs/index.html`](docs/index.html).

## Install

macOS / Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/Bariskau/conjure/main/scripts/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/Bariskau/conjure/main/scripts/install.ps1 | iex
```

Run the app:

```bash
conjure
```

The UI and API run at `http://127.0.0.1:5174`.

Release packages do not include a database. On first launch, Conjure creates a
fresh local SQLite database and seeds two disabled debate templates:
`claude_debate` and `codex_debate`.
Those seeded tools use the platform default script shell: PowerShell on Windows
and `sh` on macOS / Linux. Script-body tools follow the same rule, so Windows
scripts run through PowerShell instead of `cmd.exe`.
By default, the database lives in the user's platform data directory, such as
`~/Library/Application Support/Conjure/conjure.db` on macOS. Set
`CONJURE_DATABASE_URL` to override it.

## Connect as MCP

Conjure connects to MCP clients over stdio:

```json
{
  "mcpServers": {
    "conjure": {
      "command": "conjure",
      "args": ["--mcp"]
    }
  }
}
```

For clients that use TOML:

```toml
[mcp_servers.conjure]
command = "conjure"
args = ["--mcp"]
```

Long-running tools may also need a client-side MCP timeout. This is separate
from the timeout set inside Conjure.

Claude Code uses milliseconds:

```bash
claude mcp add-json conjure '{"type":"stdio","command":"conjure","args":["--mcp"],"timeout":1800000}'
```

Codex uses seconds:

```toml
[mcp_servers.conjure]
command = "conjure"
args = ["--mcp"]
startup_timeout_sec = 30
tool_timeout_sec = 1800
```

## Develop

```bash
cd frontend
npm install
npm run build
cd ..
cargo run -p backend
```

Use `cargo run -p backend -- --mcp` to run the MCP server from source.
