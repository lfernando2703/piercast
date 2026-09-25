# Piercast Linux

GTK4 + libadwaita via Relm4. Thin client of `piercastd`.

## Build

```bash
cargo build --release
./target/release/piercast-gui
```

## Packaging

- `.deb` / `.rpm` / AppImage via release workflow
- Flatpak manifest: `io.piercast.Piercast.yml` (publish when Flathub credentials exist)
