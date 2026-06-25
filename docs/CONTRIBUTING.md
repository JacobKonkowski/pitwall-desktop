# Contributing

Thanks for helping improve PitWall Desktop.

---

## Prerequisites

Same as [SETUP.md](SETUP.md):

- Windows 10/11, Rust 1.89+, Node 18+
- `npm install` then `npm run tauri dev`

---

## Commands

| Command | Purpose |
|---------|---------|
| `npm run tauri dev` | Dev app + hot reload |
| `npm run build` | Frontend production build |
| `npm run tauri build` | Release installer |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Rust unit tests |
| `npm run docs:api` | Generate rustdoc + TypeDoc (output in `docs/.api-out/`, gitignored) |

OpenXR layer build: [NATIVE_VR.md](NATIVE_VR.md) and [openxr-layer/README.md](../openxr-layer/README.md).

---

## Code layout

| Area | Path |
|------|------|
| IPC / state | `src-tauri/src/commands/mod.rs` |
| Live telemetry | `src-tauri/src/live/` |
| Audio coach | `src-tauri/src/audio/` |
| IBT analysis | `src-tauri/src/analysis/`, `ingest/` |
| VR | `src-tauri/src/vr/`, `openxr-layer/` |
| Frontend | `src/`, `src/widgets/` |
| Docs hub | `docs/README.md` |

Start with [ARCHITECTURE.md](ARCHITECTURE.md) for the system map.

---

## Conventions

- Rust modules by domain; `#[tauri::command]` handlers in `commands/mod.rs`
- IPC JSON uses **camelCase** (`serde(rename_all = "camelCase")`)
- TypeScript types in `src/lib/types.ts` mirror Rust structs
- Do not edit `.cursor/plans/*.plan.md` in PRs unless explicitly asked

---

## Regenerating audio clips

See [AUDIO_COACH.md](AUDIO_COACH.md). Phrases in `scripts/audio-phrases.txt`; commit generated WAVs under `src-tauri/resources/audio/coach/default/`.

---

## CI

[`.github/workflows/ci.yml`](../.github/workflows/ci.yml) on PRs to `main`:

- `npm ci` → `npm run build`
- `cargo test`
- `npm run docs:api` (smoke — ensures rustdoc + TypeDoc config valid)

---

## Documentation maintenance

| Code change | Update |
|-------------|--------|
| New Tauri command / event | `commands/mod.rs` `///`, `api.ts` TSDoc, **API.md** |
| New IPC type | `types.ts`, rustdoc, **DATA_MODEL.md** if stored |
| New settings field | **DATA_MODEL.md**, SETUP/LivePanel if user-facing |
| New audio message | **AUDIO_COACH.md**, `audio-phrases.txt` |
| Live field | **LIVE_TELEMETRY.md**, **COMPARISON.md** if SDK-related |

Index: [docs/README.md](README.md).

---

## Suggested reading order

1. `src-tauri/src/lib.rs`
2. `src-tauri/src/commands/mod.rs`
3. `src-tauri/src/live/mod.rs`
4. `src-tauri/src/audio/coach.rs`
5. `src/App.tsx` + `src/lib/api.ts`
