# Contributing to Piercast

Thanks for helping build Piercast. This monorepo uses a shared Rust daemon (`piercastd`) with thin native clients.

## Prerequisites

- Rust stable 1.81+
- Node 22+ (website, Raycast)
- Xcode 16+ (macOS / iOS)
- JDK 17 (Android)
- Optional: Tailscale CLI for expose tests

## Layout

- `core/` — Cargo workspace (schema, core, api, mcp, tailscale, piercastd, piercast CLI)
- `packages/schema/` — canonical JSON Schema for `piercast.yml`
- `apps/` — native shells
- `skill/piercast/` — agent skill
- `extensions/raycast/` — Raycast extension
- `website/` — Next.js marketing site
- `brand/` — tokens and logo assets

## Development

```bash
# Core
cd core && cargo test && cargo build

# Website
cd website && pnpm install && pnpm dev

# Raycast
cd extensions/raycast && npm install && npm run build
```

## Conventions

1. All app configs validate against `packages/schema/piercast.schema.json`.
2. UIs are API clients of `piercastd` — do not reimplement the supervisor.
3. `commands.deploy` must never run from Open/Start — only explicit Deploy.
4. Default telemetry is off.
5. Prefer boring, tested code over abstraction.

## Pull requests

- One feature or bug fix per PR.
- Include tests for supervisor/API behavior changes.
- Update `CHANGELOG.md` for user-visible changes.
- Keep PRs focused; link issues when applicable.

## Security

See [SECURITY.md](SECURITY.md). Never commit tokens, pairing secrets, or notarization credentials.
