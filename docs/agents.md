# Memorius for agents

Agents read the user's activity and propose worklogs; the user approves them
in the app, and Memorius logs the approved ones in the tracker itself.

- Skill with the full workflow: `skills/memorius-worklogs/SKILL.md`
- MCP server: `memorius mcp` (stdio). Tools: `memorius_status`, `memorius_day`, `memorius_week`,
  `memorius_unlogged`, `memorius_proposals`, `memorius_propose_worklog`.
- Command line with the same operations: `memorius --help`. Every command
  prints JSON with `"version": 9`; the shape changes only with a new version.
- `memorius status` / `memorius_status` names the build (Git commit) that
  answers. When `memorius` is updated while an MCP server runs, every tool
  result starts with a warning to restart the agent session.

## Install

The CLI ships inside the app. **Settings → Setup → Command line** links it as
the `memorius` command and registers it with Claude Code
(`claude mcp add --scope user memorius -- <app>/memorius mcp`) or Codex
(`codex mcp add memorius -- <app>/memorius mcp`), and copies the
`memorius-worklogs` skill into their `skills` folders. From source:
`cargo install --path crates/memorius-cli --locked`.

## Database access

The CLI opens the desktop app's database (`MEMORIUS_DB` or `--db` to
override). Reads are read-only. Writes change only proposals, and the CLI
never migrates the schema: when the app and the CLI disagree on the schema
version, the CLI stops with an error that says which one to update.
