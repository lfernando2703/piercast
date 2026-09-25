# Piercast

**Launch anything. From anywhere.**

Piercast is a native desktop app (macOS / Windows / Linux) that registers, launches, monitors, exposes, and controls local applications. Agents register apps via an embedded MCP server and an auto-registration skill. Companions include iOS/Android apps and a Raycast extension.

## Status

Monorepo under active build. Canonical product domain: [piercast.io](https://piercast.io) (purchase pending — see Domain note below).

**GitHub:** currently hosted at [`lfernando2703/piercast`](https://github.com/lfernando2703/piercast). Org `piercast` was free at planning time but creating it requires `admin:org` scope — migrate remote to `git@github.com:piercast/piercast.git` (or `piercast-hq/piercast` if the name collides) once the org exists.

## Architecture

| Layer | Path | Role |
| --- | --- | --- |
| Schema | `packages/schema/piercast.schema.json` | Canonical app config |
| Daemon / CLI | `core/` | Rust workspace: registry, supervisor, HTTP API, MCP, Tailscale |
| Desktop | `apps/macos`, `apps/windows`, `apps/linux` | Native thin clients of `piercastd` |
| Mobile | `apps/ios`, `apps/android` | Pair via QR; control over LAN/Tailscale |
| Raycast | `extensions/raycast` | Command palette actions |
| Skill | `skill/piercast` | Agent auto-registration |
| Website | `website/` | Marketing on piercast.io |

Control plane: `http://127.0.0.1:47923` (override with `PIERCAST_PORT`). Auth: bearer token in `pairing.json`.

## Quick start (daemon)

```bash
cd core && cargo build --release
./target/release/piercastd &
./target/release/piercast upsert ./path/to/app
./target/release/piercast open <app-id>
```

MCP (Cursor):

```json
{ "mcpServers": { "piercast": { "command": "piercast", "args": ["mcp"] } } }
```

## Data directories

- macOS: `~/Library/Application Support/Piercast/`
- Linux: `$XDG_DATA_HOME/piercast/` (default `~/.local/share/piercast/`)
- Windows: `%APPDATA%\\Piercast\\`

Files: `registry.sqlite`, `piercast.lock`, `daemon.log`, `pairing.json`, `piercast.sock` (Unix).

## Domain note

Primary: **piercast.io**. Also intended: piercast.sh, piercast.co; optional piercast.app / piercast.dev / usepiercast.com. Purchase requires registrar checkout (manual). If piercast.io is taken at purchase, fall back to piercast.sh as primary and usepiercast.com as marketing redirect. **piercast.com is taken** — do not rely on it.

## License

Apache-2.0. See [LICENSE](LICENSE), [CONTRIBUTING.md](CONTRIBUTING.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [SECURITY.md](SECURITY.md).

## Verification (local)

- `cargo test -p piercast-core` — 8/8 supervisor tests passing
- Fixture smoke: `piercast upsert` + start/open → HTTP 200 on `:8765`; `/v1/apps/fixture/stats` shows `launches >= 1` + CPU/mem sample; kill removes process

## Footprint targets

Idle `piercastd` RSS < 40MB (macOS); idle desktop shell < 80MB. Document measured numbers here after verification.
