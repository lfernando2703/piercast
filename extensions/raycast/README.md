# Piercast for Raycast

Control local apps managed by [Piercast](https://piercast.io) — search, open, start, stop, restart, kill, expose via Tailscale, and check daemon health — without leaving Raycast.

**Platforms:** macOS  
**License:** MIT  
**Author:** piercast  
> Store publish requires a Raycast account named `piercast` (or update `author` in `package.json`).

## Requirements

1. Install and run the Piercast daemon (`piercastd`) on this Mac.
2. Copy the bearer token from Piercast's `pairing.json` (Application Support data directory).
3. Set extension preferences (Raycast → Extensions → Piercast):
   - **API Base URL** — default `http://127.0.0.1:47923`
   - **Bearer Token** — required password field

## Commands

| Command | Description |
| --- | --- |
| **Search Apps** | Fuzzy-find registered apps; Open / Start / Stop / Restart / Kill / Expose from the action panel |
| **Open App** | Calls `POST /v1/apps/{id}/open` so the daemon auto-starts dependencies, then opens the app URL |
| **Start** | Start in development or production mode |
| **Stop** | Graceful stop |
| **Restart** | Stop then start with the same mode |
| **Kill** | Force kill |
| **Expose Via Tailscale** | `serve` (tailnet), `funnel` (public HTTPS), or `off` |
| **Daemon Status** | `GET /v1/health` plus registered app count |

## Setup

```bash
cd extensions/raycast
npm install
npm run dev
```

Or import the folder in Raycast → **Create Extension** / **Import Extension**.

### Preferences

| Name | Type | Default | Notes |
| --- | --- | --- | --- |
| `apiBaseUrl` | textfield | `http://127.0.0.1:47923` | No trailing slash |
| `token` | password | — | From `pairing.json`; sent as `Authorization: Bearer` |

## Development

```bash
npm run lint
npm run build
```

Icons: `assets/icon.png` and `assets/command-icon.png` (512×512, Raycast Store requirement) plus `assets/icon-16.png` (16×16 placeholder). Brand colors: ink `#0B0B0F`, primary periwinkle `#7B6CFF`.

## Privacy

Requests go only to your local Piercast control plane (loopback by default). No analytics or telemetry from this extension.
