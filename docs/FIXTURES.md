# Synthetic fixtures & bring-your-own IBT

PitWall unit tests do **not** require a real iRacing recording. Analysis and
cleanup tests build `RawFrame` values in memory (see `pitwall-analysis` /
`src-tauri` analysis tests).

## Bring your own IBT

For manual / integration checks:

1. Enable disk telemetry in `app.ini` (`irsdkEnableDisk=1`).
2. Record with **Alt+L** → `Documents\iRacing\telemetry\*.ibt`.
3. Import via Analyze → Import, or drop into the watched telemetry folder.

Do **not** commit large personal `.ibt` files. If a tiny synthetic binary fixture
is added later, document its size and generation script here.

## Generator (dev)

Use `cargo test -p pitwall-analysis` (or `cargo test --lib` in the desktop
package) to exercise segmentation, sectors, and A/B/C cleanup without files.
