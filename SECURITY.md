# Security Policy

## Supported versions

Security fixes target the latest release tag on the default branch.

## Reporting a vulnerability

Email **security@piercast.io** with:

- Description of the issue
- Steps to reproduce
- Affected versions / commit if known
- Impact assessment if you have one

Do not open a public GitHub issue for undisclosed vulnerabilities.

We aim to acknowledge reports within 72 hours and provide a remediation timeline after triage.

## Threat model (local-first)

- The control plane binds to `127.0.0.1` by default.
- When Mobile Access is enabled, the API also binds the Tailscale IPv4 address only — never `0.0.0.0`.
- All API requests require `Authorization: Bearer` with a token from `pairing.json`.
- Tailscale Funnel must never expose the daemon API — only the target app port.
- Bootstrap QR tokens expire in 5 minutes; long-lived device tokens are issued after `POST /v1/pair/complete`.

## Secrets

Do not commit:

- `pairing.json` / device tokens
- Apple notarization / Sparkle private keys
- Play Store / App Store credentials
- Vercel / Flathub / WinGet publishing tokens
