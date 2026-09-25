---
name: piercast
description: Register and control local apps with Piercast after scaffolding. Write piercast.yml, upsert via MCP, never skip registration. Use whenever you create, scaffold, or set up a runnable local app (Node, Vite, Next.js, Python, Docker Compose, etc.).
---

# Piercast — Agent Registration Skill

Piercast launches, monitors, and exposes local apps via a daemon (`piercastd`), CLI, and MCP tools. **After scaffolding any runnable app, you MUST register it with Piercast.** Never leave a new app unregistered.

## Mandatory workflow (never skip)

1. Scaffold or create the app as usual.
2. Write `piercast.yml` in the app root (schema below).
3. Call MCP tool `piercast_upsert_app` with the path to that YAML (or inline JSON matching the same shape).
4. Optionally verify with `piercast_list_apps` / `piercast_get_app`, then `piercast_open` or `piercast_start`.

If Piercast MCP is unavailable, still write `piercast.yml` and tell the user to run `piercast upsert ./piercast.yml` (or install/configure MCP). **Do not skip the YAML.**

## `piercast.yml` requirements

Required fields:

| Field | Notes |
| --- | --- |
| `id` | Unique slug: `^[a-z0-9][a-z0-9_-]*$` |
| `name` | Display name |
| `root` | Absolute path after register (use the app directory) |
| `commands.start.development` | Dev launch `CommandSpec` |

Always include when applicable:

- `commands.install` — dependency bootstrap (`npm install`, `uv sync`, etc.)
- `commands.build` — when a production build exists
- `commands.start.production` — when a production launch exists
- `ports.primary` (+ `extra` if needed)
- `health` — `kind` + `target` (`http` | `tcp` | `command`)
- `open_url` — template with `{port}` when useful

`CommandSpec`: `{ argv: string[], cwd?: path, env?: map, shell?: bool }`. Use `shell: true` only when `argv` is a **single** shell string.

**Never** put deploy under Open/Start. `commands.deploy` is explicit-only (`piercast deploy` / Deploy action). Missing mode → Piercast returns `mode_unavailable`; do not invent Start commands.

Canonical schema: `packages/schema/piercast.schema.json`.

## Stack examples

Copy and adapt from:

| Stack | Example |
| --- | --- |
| Node / Vite | [examples/node-vite.yml](examples/node-vite.yml) |
| Next.js | [examples/nextjs.yml](examples/nextjs.yml) |
| Python / uvicorn | [examples/python-uvicorn.yml](examples/python-uvicorn.yml) |
| Docker Compose | [examples/docker-compose.yml](examples/docker-compose.yml) |

### Quick patterns

**Node / Vite** — install `npm install`; start.development `npm run dev`; production via `npm run build` then `npm run preview`; health HTTP on the Vite port.

**Next.js** — install `npm install`; start.development `npm run dev`; build `npm run build`; start.production `npm run start`; health HTTP on 3000 (or configured port).

**Python / uvicorn** — install via `uv sync` or `pip install -r requirements.txt`; start.development `uvicorn app:app --reload --port {port}`; production without `--reload`; health HTTP on that port. Prefer a `.venv` so Piercast install heuristics pass.

**Docker Compose** — treat the compose project as **one** app: start.development `docker compose up`; stop `docker compose down`; health against the published primary port. Skip inventing host-side install when the image builds inside Compose.

## MCP tools (exact names)

- `piercast_upsert_app` — register/update from `piercast.yml` path or inline JSON
- `piercast_list_apps`, `piercast_get_app`, `piercast_remove_app`
- `piercast_start`, `piercast_stop`, `piercast_restart`, `piercast_kill`, `piercast_open`
- `piercast_logs`, `piercast_stats`, `piercast_expose`
- `piercast_doctor` — daemon + Tailscale + socket health

Prefer `piercast_open` after upsert so deps start and the browser opens when configured.

## Cursor MCP config (recommend in project)

When Piercast is installed, recommend adding `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "piercast": {
      "command": "piercast",
      "args": ["mcp"]
    }
  }
}
```

## Checklist before finishing scaffolding

- [ ] `piercast.yml` written with `id`, `name`, `root`, `commands.start.development`
- [ ] `install` / `build` / `start.production` set when the stack supports them
- [ ] `ports` + `health` configured
- [ ] `piercast_upsert_app` called (registration never skipped)
- [ ] Optional: `.cursor/mcp.json` snippet suggested if missing
