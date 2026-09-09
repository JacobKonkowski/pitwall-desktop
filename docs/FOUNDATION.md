# PitWall foundation

This document is the onboarding map for contributors and AI assistants.

## Goals

PitWall is a **Windows** desktop app for iRacing:

- **Analyze** — import IBT files, review laps, compare traces
- **Live** — shared-memory telemetry, voice coach, dual-surface widgets

Surfaces for live widgets: **monitor** (always-on-top windows) and **VR** (OpenXR API layer). One widget catalog; enable once, place twice.

## Workspace crates

| Crate | Role | May depend on |
|-------|------|----------------|
| `pitwall-telemetry` | Raw frame / sector types | — |
| `pitwall-analysis` | Pure IBT pipeline + cleanup | telemetry |
| `pitwall-settings` | JSON settings | — |
| `pitwall-storage` | SQLite | analysis |
| `pitwall-ingest` | IBT parse/import/watcher | analysis, storage, telemetry |
| `pitwall-live` | LiveSnapshot producer | telemetry, settings |
| `pitwall-audio` | Path B coach | live, settings |
| `pitwall-monitor` | Desktop monitor widget host | live, settings |
| `pitwall-vr` | SHM + OpenXR install + web HUD | live, settings |
| `pitwall-desktop` (`src-tauri`) | Tauri commands / composition | all |

**Forbidden:** domain crates depending on `commands` / desktop; `live` → `audio`; `monitor` ↔ `vr`; `analysis` → `storage`/`tauri`.

## Import path policy

`import_ibt` only accepts `.ibt` paths under the default telemetry directory (see `pitwall_ingest::validate_import_path`). Prefer the file dialog or folder watcher.

## Schema migrations

`PRAGMA user_version`: pre-v2 DBs are wiped once (incompatible). From v2 onward, upgrades are incremental. Full wipe only via debug `clear_database`.

## AI / PR playbook

1. Name one vertical slice (one coach rule, one widget, one cleanup change).
2. Touch one crate first; IPC/UI last.
3. Add/extend unit tests in that crate.
4. Do not rename crates and change behavior in the same PR.
5. Prompt boundary example: “Only modify `crates/pitwall-audio/src/engine/rules/pace.rs` and its tests.”

## Good first slices

- New coach rule under `pitwall-audio` `engine/rules/`
- Widget presentational tweak in `src/widgets/`
- Docs-only clarification in `docs/`

## Related docs

- [SETUP.md](SETUP.md) — race-night checklist
- [ARCHITECTURE.md](ARCHITECTURE.md) — module map
- [PRIVACY.md](PRIVACY.md) — local-only data
- [FIXTURES.md](FIXTURES.md) — tests without personal IBTs
- [PLUGINS.md](PLUGINS.md) — extension points
- [SECURITY.md](../SECURITY.md) — vulnerability reporting
