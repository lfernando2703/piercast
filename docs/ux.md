# Piercast UX Spec

Shared UX contract for all Piercast clients: macOS (SwiftUI), Windows (WinUI 3), Linux (GTK4/libadwaita), iOS, and Android. Shells are thin API clients of `piercastd`; this document defines information architecture, visual language, feature parity, accessibility, onboarding, and deep links so native implementations stay consistent.

Brand tokens live in [`brand/tokens.json`](../brand/tokens.json). Logo assets live in [`brand/logo/`](../brand/logo/).

---

## Principles

1. **Library first** — the primary job is find → inspect → act on a local app.
2. **Status at a glance** — health, CPU/mem, and expose state must be readable without opening inspector chrome.
3. **Native, not skeuomorphic** — follow platform HIG/Fluent/GNOME patterns; share IA and copy, not pixel-perfect clones.
4. **Local-first calm** — no telemetry banners by default; no dark-pattern upsells; empty states stay quiet.
5. **Deploy is explicit** — Open/Start never run `commands.deploy`; Deploy is a deliberate action only.

---

## Information architecture

### Desktop shells (macOS / Windows / Linux)

```
┌─────────────┬──────────────────────────────┬────────────────────┐
│  Sidebar    │  Main (grid | list)          │  Detail inspector  │
│             │                              │  (optional pane)   │
│  Library    │  App cards / rows            │  Status + actions  │
│  · All      │                              │  Sparkline / gauges│
│  · Running  │                              │  Usage / Logs      │
│  · Unhealthy│                              │  Expose / Depends  │
│  · Favorites│                              │                    │
│  Tags       │                              │                    │
└─────────────┴──────────────────────────────┴────────────────────┘
         Menu bar / system tray · Command palette (⌘K / Ctrl+K)
```

#### Sidebar — Library + Tags

| Section | Contents |
|---------|----------|
| **Library** | Filters: **All**, **Running**, **Unhealthy**, **Favorites**. Selection is single; count badges optional. |
| **Tags** | Dynamic list from app `tags[]`. Selecting a tag intersects with the active Library filter. |
| **Footer** (optional) | Settings gear, Import `piercast.yml`, Doctor shortcut. |

- Default selection on launch: **All**.
- Persisted prefs: last filter, last tag, grid vs list, inspector open/closed, sidebar width.
- Collapsible sidebar on narrow windows; minimum content width before collapse is platform-native.

#### Main — App grid / list

- User toggle: **grid** (default) or **list**.
- Each item shows: icon (or pier mark fallback), name, status pill, primary port (if set), favorite star.
- Sort: Favorites first within filter, then name A–Z (stable). Recents may surface in palette, not as a separate main sort unless Settings enables it.
- Multi-select: not required in v1; single selection drives the inspector.
- Empty filter → empty state (see Visual).

#### Detail inspector

Shown when an app is selected. Sections, top to bottom:

| Block | Content |
|-------|---------|
| **Header** | Name, id slug, status pill, primary action row |
| **Actions** | Start (mode menu: development / production when available), Open, Stop, Restart, Kill, Deploy (only if `commands.deploy` set) |
| **Health** | Status + sparkline of recent health samples |
| **Resources** | CPU gauge, memory gauge (warn thresholds from `resources.*`) |
| **Usage** | Launches, foreground opens, total runtime, last launched, last updated |
| **Logs** | Tail viewer with filter + copy; link to full log window if needed |
| **Expose** | Tailscale control: Off / Serve / Funnel; show resulting URL when active |
| **Depends-on** | Graph / list of `depends_on` edges (outgoing + reverse dependents) |

Missing mode for Start → surface daemon error `mode_unavailable` with configured modes listed; do not invent commands.

#### Command palette — ⌘K / Ctrl+K

- Global shortcut: **⌘K** (macOS), **Ctrl+K** (Windows / Linux).
- Fuzzy find apps by name, id, tags.
- Actions on selected result: Start, Open, Stop, Restart, Kill, Expose (serve / funnel / off), Show in Library, Copy id.
- Also surfaces: Settings, Import, Doctor, Pair Mobile, Quit.
- Results update live from WebSocket `/v1/events` when the palette is open.

#### Menu bar / system tray

| Item | Behavior |
|------|----------|
| Running count | Badge or subtitle: number of apps in Running |
| Open Piercast | Focus / restore main window |
| Quick list | Optional: up to N running apps with Open / Stop |
| Quit | Stop UI shell; daemon lifetime follows Settings (“Keep piercastd running”) |

#### Settings (desktop)

- Control plane port (`PIERCAST_PORT` / default `47923`)
- Launch at login
- Theme: **Dark / Light / System**
- Mobile Access (bind Tailscale IP when enabled; never `0.0.0.0`)
- Keep daemon running when UI quits
- Telemetry: **off** by default; optional anonymized crash reports opt-in
- Reduced motion (mirrors OS when available; also explicit toggle)

### Mobile companions (iOS / Android)

Simpler stack; same actions, no sidebar chrome.

| Screen | Purpose |
|--------|---------|
| **Home** | App list + status pills; pull-to-refresh; filter chips: All / Running / Unhealthy / Favorites |
| **App detail** | Actions (Open, Start, Stop, Restart, Kill), health summary, stats summary, expose URL if present |
| **Settings** | Paired hosts, re-pair, theme, reduced motion |

- Open prefers funnel/serve URL when present; else `open_url` / primary port.
- No cloud account. Re-pair if token rotated.
- Cleartext HTTP allowed only to RFC1918 + Tailscale CGNAT `100.x`.

---

## Visual language

### Inspiration

Apple HIG–inspired across platforms: large titles, clear hierarchy, generous touch/click targets, materials/vibrancy on macOS, Fluent acrylic/mica on Windows where appropriate, libadwaita Adw views on Linux. Prefer platform controls over custom widgets.

### Color

| Token | Hex | Use |
|-------|-----|-----|
| Primary | `#7B6CFF` | Accents, focus rings, links, brand mark |
| Ink | `#0B0B0F` | Text / surfaces in light mode |
| Fog | `#F5F5F7` | Light backgrounds / dark-mode text |
| Success | `#30D158` | Healthy / Running |
| Warn | `#FF9F0A` | Degraded / resource warn |
| Danger | `#FF453A` | Unhealthy / Kill emphasis |

Status pills map: Running→success, Starting/Stopping→warn or primary, Unhealthy→danger, Stopped→secondary/neutral, Expose active→primary.

### Typography

| Platform | Font |
|----------|------|
| macOS / iOS | SF Pro |
| Windows | Segoe UI Variable |
| Linux / web | Inter |

Large title for Library; body for lists; caption for metadata (ports, ids, timestamps).

### Spacing & layout

- **8pt grid** everywhere (padding, gaps, icon hit areas).
- Corner radii: follow platform defaults (do not invent a global “card radius” language).
- Avoid decorative card chrome in the hero/empty state; cards only when they group interactive content.

### Motion

- Spring / ease animations **≤ 200ms**.
- Purposeful only: inspector open/close, palette appear, status pill change, list insert/remove.
- Honor **Reduce Motion** / `prefers-reduced-motion`: crossfade or instant; no springs, no parallax.

### Theme

Support **Dark**, **Light**, and **System**. Brand primary remains `#7B6CFF` in both themes; adjust contrast for text/icons to meet WCAG AA against surfaces.

### Icons

- SF Symbols on Apple; Segoe Fluent Icons on Windows; symbolic icons on Linux; Material Icons on Android.
- Map semantic names (play, stop, restart, bolt/kill, antenna/expose, heart/favorite) — do not ship emoji as UI icons.

### Empty states

- Centered **pier mark** (`brand/logo/mark.svg`) + one short headline + one supporting sentence + one CTA.
- Examples:
  - No apps: “No apps yet” / “Import a `piercast.yml` or register via MCP.” → Import / Docs
  - Filter empty: “Nothing running” / “Start an app from the Library.”
  - Unhealthy empty: “All clear” / “No unhealthy apps.”

Zero clutter chrome; no illustration packs beyond the pier mark.

---

## Onboarding (desktop) — 4 screens

Shown once until dismissed/completed; re-openable from Settings → Setup.

| # | Screen | Goal | Primary CTA |
|---|--------|------|-------------|
| 1 | **Install daemon** | Confirm `piercastd` is running / bundled binary launched | Continue / Retry health check (`GET /v1/health`) |
| 2 | **Connect MCP** | Show Cursor/agent config snippet for `piercast mcp` | Copy config → Continue |
| 3 | **Register first app** | Import wizard or path to `piercast.yml` / upsert | Import / Skip |
| 4 | **Pair mobile** | Show QR from `POST /v1/pair/bootstrap` | Continue / Skip |

Rules:

- Non-blocking Skip on screens 3–4; screen 1 should not Skip if daemon is required for the shell.
- Progress indicator (1–4) with back navigation.
- After completion, land on Library **All** empty or populated state.

Mobile onboarding is pairing-first: scan QR → confirm host → Home.

---

## Deep links

| URI | Behavior |
|-----|----------|
| `piercast://open/{id}` | Focus app (auto-start daemon if needed); call Open for `{id}` (Start `last_mode` or `development`, wait healthy ≤60s, open URL) |
| `piercast://apps/{id}` | Select app in Library / open detail (no auto Open) |
| `piercast://pair` | Jump to Pair mobile flow |

Unknown id → toast/error `app_not_found`; do not create placeholder apps.

Platform registration: custom URL scheme `piercast` on macOS/iOS/Windows/Android; Linux `.desktop` MimeType / xdg handler where applicable.

---

## Accessibility

| Requirement | Detail |
|-------------|--------|
| **Screen readers** | VoiceOver (macOS/iOS), Narrator (Windows), Orca (Linux), TalkBack (Android). Every control has an accessible name; status pills expose textual status, not color alone. |
| **Focus** | Visible focus rings; palette and inspector fully keyboard operable. |
| **Contrast** | Text/icon contrast ≥ WCAG AA on fog/ink surfaces; status color never sole indicator. |
| **Reduced motion** | Respect OS setting; optional in-app override. Caps animation at 0–200ms; disables springs when reduced. |
| **Dynamic type** | Apple Dynamic Type / Android font scale; avoid truncating critical status. |
| **Hit targets** | ≥ 44×44 pt (Apple) / 48×48 dp (Android); desktop rows ≥ 32 pt height. |
| **Logs** | Selectable text; copy action labeled; filter field labeled. |

---

## Feature parity checklist

All desktop shells (**macOS**, **Windows**, **Linux**) MUST support:

| Feature | Notes |
|---------|-------|
| Library filters All / Running / Unhealthy / Favorites | Sidebar |
| Tags filter | From app tags |
| Grid + list toggle | Default grid |
| Detail inspector | Status, sparkline, CPU/mem, usage, logs, expose, depends-on |
| Start / Open / Stop / Restart / Kill | Mode-aware Start; `mode_unavailable` surfaced |
| Deploy action | Only when configured; never via Open/Start |
| Logs stream / tail + filter + copy | |
| Health badges | Library + inspector |
| Stats charts / gauges | From `/v1/apps/{id}/stats` + samples |
| Tailscale expose Off / Serve / Funnel | `tailscale_unavailable` → download link |
| Settings | Port, launch at login, theme, mobile access, reduced motion, telemetry opt-in |
| Import `piercast.yml` | Wizard may draft from package.json / Procfile / compose |
| Command palette ⌘K / Ctrl+K | |
| Menu bar / tray | Running count, Open Piercast, Quit |
| 4-screen onboarding | Daemon, MCP, first app, pair mobile |
| Deep link `piercast://open/{id}` | |
| Favorites + recents | Favorites in Library; recents in palette |
| Export/import registry JSON | Settings or menu |
| `piercast doctor` entry point | UI wrapper around doctor |
| Auto-start daemon on app launch | If not running |
| Dark / Light / System theme | |
| Accessibility | VoiceOver/Narrator/Orca + reduced motion |
| WebSocket live updates | `/v1/events` |

Mobile (**iOS**, **Android**) MUST support:

| Feature | Notes |
|---------|-------|
| Home list + status | Filters All / Running / Unhealthy / Favorites |
| App detail actions | Open, Start, Stop, Restart, Kill |
| Health + stats summary | |
| Pair via QR | Bootstrap + complete |
| Settings paired hosts | Re-pair on token rotate |
| Theme + reduced motion | |
| TalkBack / VoiceOver labels | |
| Prefer expose URL on Open | When serve/funnel active |

Out of parity by nature (document, don’t fake):

- Materials/vibrancy (macOS), Mica (Windows), Adw breakpoints (Linux)
- Sparkle / WinGet / Flatpak update channels
- MenuBarExtra vs WinUI tray vs StatusNotifier

---

## Copy & error surfacing

- Prefer short, sentence-case labels: “Start”, “Open”, “Expose”, “Kill”.
- Daemon errors shown as user-readable toasts or inline banners with `code` in detail disclosure:
  - `dependency_cycle`
  - `mode_unavailable`
  - `start_timeout`
  - `tailscale_unavailable`
  - `app_not_found`
- Kill is destructive: confirm on desktop when app is Running (optional hold on mobile long-press).

---

## Platform notes

| Shell | Stack | Distribution cues |
|-------|-------|-------------------|
| macOS | SwiftUI + MenuBarExtra; `piercastd` in `Contents/Resources` | Developer ID + notarized DMG; Sparkle 2; Homebrew cask |
| Windows | WinUI 3 (.NET 8) + tray | MSIX + MSI; HKCU Run for autostart |
| Linux | GTK4 + libadwaita (Relm4) | deb, rpm, AppImage; Flatpak when ready |
| iOS | SwiftUI; bundle `io.piercast.app` | Pair QR; ATS exceptions for RFC1918 + `100.x` |
| Android | Kotlin + Jetpack Compose; `io.piercast.android` | Same pairing; network security config mirrors ATS |

---

## Verification (UX)

Manual smoke across at least one desktop and one mobile shell:

1. Fresh install → 4 onboarding screens complete → Library empty state shows pier mark.
2. Import / upsert app → appears in All within 2s of WebSocket event.
3. ⌘K / Ctrl+K finds app → Open starts deps and opens URL.
4. Unhealthy filter shows only failing health; inspector sparkline updates.
5. Expose Serve returns URL; Off clears it; missing Tailscale shows `tailscale_unavailable` guidance.
6. Reduce Motion on → no spring animations.
7. VoiceOver/TalkBack: status and actions announced without relying on color.
8. `piercast://open/{id}` from browser/terminal focuses shell and Opens the app.
