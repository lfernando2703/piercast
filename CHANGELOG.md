# Changelog

## v0.1.0 — 2026-09-25

### Added
- Monorepo bootstrap: Apache-2.0, CoC, CONTRIBUTING, SECURITY, README, brand tokens/logos
- Canonical schema `packages/schema/piercast.schema.json`
- Rust workspace `core/`: piercast-schema, piercast-core (supervisor/graph/health/SQLite), piercast-api, piercast-mcp, piercast-tailscale, `piercastd`, `piercast` CLI
- Agent skill `skill/piercast` + examples (Vite, Next.js, uvicorn, Docker Compose)
- Native shells: macOS SwiftUI, Windows WinUI 3, Linux Relm4/GTK4
- Mobile companions: iOS SwiftUI, Android Compose + pairing protocol
- Raycast extension (macOS) with Search/Open/Start/Stop/Restart/Kill/Expose/Daemon Status
- Marketing website (Next.js) pages: Home, Features, Download, Docs, Open Source, Changelog, Privacy, Terms
- CI + Release workflows, Homebrew cask draft, winget manifest draft
- Import wizard (`piercast-core::import_wizard`) for package.json / Procfile / compose
- UX spec `docs/ux.md`

### Notes
- GitHub remote interim: `lfernando2703/piercast` (org `piercast` needs `admin:org`)
- Domains piercast.io/.sh/.co pending manual registrar checkout
