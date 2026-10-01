# Intellectual property hygiene

This document describes how PitWall Desktop handles **copyright**, **third-party
licenses**, **trademarks**, and **patents**. It is an engineering checklist, not
legal advice. Patent freedom-to-operate and trademark clearance require counsel.

## Copyright (our code)

- Original PitWall Desktop source and docs are licensed under the MIT License
  ([LICENSE](../LICENSE)).
- Do **not** paste proprietary SDK samples, game assets, branded audio, track
  maps from commercial products, or substantial third-party source into this
  tree without a compatible license and attribution in [NOTICE](../NOTICE).
- Prefer clean-room implementations against published telemetry **interfaces**
  (channel names, shared-memory layout docs, OpenXR headers) over copying
  another vendor’s client.

## Third-party software licenses

| Surface | Enforcement |
|---------|-------------|
| Rust crates | [`deny.toml`](../deny.toml) + `cargo deny check licenses bans sources` in CI |
| npm packages | `npm run check:licenses` (allowlist in [`scripts/check-npm-licenses.mjs`](../scripts/check-npm-licenses.mjs)) |
| Human-readable inventory | [NOTICE](../NOTICE); refresh with `npm run gen:notice` before tagged releases |

Allowed SPDX sets are intentionally permissive (MIT/Apache/BSD/ISC/MPL-2.0 and
similar). Copyleft that would force this app under GPL/AGPL is denied for Rust
and blocked by the npm allowlist.

### Key redistributed / build-time dependencies

| Component | Role | License (typical) |
|-----------|------|-------------------|
| [`pitwall`](https://crates.io/crates/pitwall) (We Race) | Live SHM + IBT reader | MIT |
| Tauri / wry / webview stack | Desktop shell | Apache-2.0 / MIT |
| OpenXR-SDK headers (Khronos) | Fetched at OpenXR layer build | Apache-2.0 |
| React, Vite, Recharts, etc. | Frontend / build | MIT / ISC / Apache-2.0 / CC-BY-* (data) |

Exact texts live in each package’s LICENSE file on crates.io / npm.

## Trademarks and product names

PitWall Desktop is an independent project. Unless a written license says
otherwise:

- **iRacing** and related marks are trademarks of iRacing.com Motorsport
  Simulations, LLC. Use them only to describe compatibility (nominative use).
  Do not imply endorsement, sponsorship, or official partnership.
- **OpenXR** and **Khronos** are trademarks of the Khronos Group.
- **OpenKneeboard**, **RaceLab**, headset vendors, and other third-party tools
  are mentioned for interoperability only.
- The crates.io name **`pitwall`** and the “We Race” / Pitwall telemetry stack
  are third-party branding. This app **depends on** that MIT-licensed crate; it
  is not the We Race product. Avoid implying ownership of their marks. If you
  commercialize or widen distribution, get trademark advice on the “PitWall”
  product name relative to We Race and iRacing.

UI and marketing copy should prefer “works with iRacing” / “reads iRacing
telemetry” over “official iRacing …” wording.

## iRacing telemetry interface

PitWall reads data iRacing exposes for third-party tools:

- Live: Windows shared memory (`Local\IRSDKMemMapFileName`) when enabled in
  `app.ini`
- Disk: user-recorded `.ibt` files under the user’s telemetry folder

Engineering constraints we follow:

1. **Read-only** telemetry consumption — do not write into the IRSDK map or
   claim to mutate sim state through the SDK.
2. **No cheats / unauthorized experience mods** — coaching and HUDs that display
   information already available to the driver are the intent; do not add
   features that circumvent anti-cheat or alter the sim client.
3. **Local-only by default** — see [PRIVACY.md](PRIVACY.md). Do not upload IBT or
   live session data without an explicit, documented user action and policy.
4. Revisit iRacing’s current Terms of Use / EULA and any Commercial Software
   Terms before **commercial** distribution or cloud features.

iRacing’s published terms can change; treat this section as a maintainer
checklist, not a compliance certificate.

## OpenXR layer

The `openxr-layer/` DLL is original PitWall code. Structure follows the common
implicit-layer pattern documented by Khronos and illustrated by community
templates (e.g. Ybalrid/OpenXR-API-Layer-Template, MIT). We do not vendor that
template’s source tree; if you copy substantial template code later, keep its
copyright notice and MIT terms in NOTICE.

OpenXR headers are pulled via CMake `FetchContent` from Khronos OpenXR-SDK
(Apache-2.0). Attribute Khronos in NOTICE; do not claim Khronos endorsement.

## Audio assets

Coach WAV clips under `src-tauri/resources/audio/coach/default/` are generated
locally via Windows speech APIs (`scripts/generate-audio-clips.ps1`). Do not
commit voice packs, spotter packs, or music from commercial products. If you
replace the default voice with a licensed third-party voice, record the license
in NOTICE.

## Patents

**No repository process can prove patent non-infringement.** This section is an
informal engineering landscape note (US-focused web search, April–October 2026
context). It is **not** a freedom-to-operate (FTO) opinion, claim chart, or legal
advice. Claim construction, doctrine of equivalents, and jurisdiction-specific
risk require counsel.

### PitWall surfaces that attract patent search

| Feature area | What PitWall does today |
|--------------|-------------------------|
| Audio coach | Rule-based callouts from live IRSDK (flags, pack/spotter, fuel, gaps, sector/lap times, race clock) via WAV + WinRT TTS. **No** live “brake earlier / later” speech. |
| Live field UI | Leaderboard, gaps, pack state from shared memory |
| Analyze | IBT import, lap cleanup, two-lap compare, corner technique, **brake-point deltas** (“X m earlier/later” vs a reference lap) and brake-point consistency scatter (`corners.rs`, `consistency.rs`) |
| Native VR HUD | Implicit OpenXR API layer appends `XrCompositionLayerQuad`s; desktop writes SHM |

### Notable US patents / families (watch list)

These were the closest public hits when searching for racing voice guidance,
path-based coaching, and VR/game overlays. “Overlap” below means **thematic /
feature adjacency**, not a finding of infringement or non-infringement.

| Patent / family | Owner (as listed publicly) | Claim focus (high level) | Overlap vs PitWall (engineering view) |
|-----------------|----------------------------|--------------------------|----------------------------------------|
| [US 11,151,900 B2](https://patents.google.com/patent/US11151900B2) | RaceVoice LLC → Finger Lakes Consulting Group Inc. | Pre-race UI to select **track points** + guidance options; while racing, detect location at a selected point; annunciate vehicle parameter via audio actuator in a **driver’s race helmet** | **Highest thematic risk for voice coaching.** RaceVoice also markets sim products and offers licensing. PitWall’s current coach is mostly **event/edge driven** (flag/pack/sector/lap), not a “pick map points → announce speed at GPS corner” product. That difference may or may not matter under claim construction — counsel must decide. Active; 4th-year maintenance fee recorded (2025). |
| [US 11,830,375 B2](https://patents.google.com/patent/US11830375B2) and continuations (e.g. [US 12,606,023 B2](https://patents.google.com/patent/US12606023B2)) | Garmin | Build an **optimal path of travel** from multiple geolocated laps; audible/visual coaching; some claims tie to brake-pedal sensors / camera | Lower overlap with current PitWall: no removable brake sensor, no GNSS coach device, no “stitch best segments into optimal line” coaching pipeline. Analyze track maps from IBT GPS are visualization, not this claimed coaching method. Still a watch if you add turn-by-turn “brake earlier / later” from path reconstruction. |
| Broader VR / game overlay art (e.g. cloud-gaming VRAM overlays, HMD compositors) | Various | Injecting / blending overlays into rendered frames | OpenXR **API layers** that append composition quads are a **published Khronos / community pattern** (not PitWall-specific). No patent was found in this scan that clearly claims “OpenXR API layer injects HUD quads into another app’s `xrEndFrame`.” Broader overlay patents still exist; using the loader-supported layer path is not itself a clearance. |
| Telemetry+video sync (e.g. [US 10,016,689 B2](https://patents.google.com/patent/US10016689B2)) | Various | Associate gameplay video timestamps with telemetry events | Low overlap today (PitWall does not ship synchronized video↔telemetry replay as a core feature). |

### Practical risk ranking for *this* codebase

1. **Audio coach** — watch RaceVoice / US 11,151,900 before expanding into
   location-triggered corner speed / G-force callouts, map-point configuration
   UIs, or marketing that mirrors their patented framing. Prefer documenting that
   PitWall announces **sim session events and timing edges**, not a race-vehicle
   helmet VGS at preselected track geolocations — but do not treat that as a
   legal safe harbor.
2. **Path / “optimal line” coaching** — watch Garmin family before shipping
   geolocation-stitched ideal lines with live audible brake/throttle instructions.
3. **VR HUD** — lower specific-patent signal in this scan; rely on OpenXR public
   APIs and avoid copying proprietary overlay implementations. Still get FTO if
   commercializing widely.
4. **IBT analyze / lap compare / fuel panels** — common analytics patterns; no
   standout blocking patent surfaced in this pass (absence of evidence ≠ evidence
   of absence).

### What to do next (counsel / product)

- Before **commercial sale, paid distribution, or fundraising**, commission a
  real FTO from a patent attorney covering at least US (and any launch markets),
  with claim charts against US 11,151,900 and the Garmin racing-coach family.
- If expanding the coach toward **corner entry/min/exit speeds at track
  locations**, treat RaceVoice licensing outreach (`patent@racevoice.com` per
  their site) as a business option to evaluate with counsel — do not DIY a
  “design around” without advice.
- Apache-2.0 dependencies grant a limited patent license **for those
  contributions only**; that does not cover RaceVoice, Garmin, iRacing, or
  unrelated third-party patents.
- If you knowingly implement a patented method, document the patent number and
  license status in NOTICE — or do not ship it.

## Maintainer checklist (each release)

1. `cargo deny check licenses bans sources`
2. `npm run check:licenses`
3. `npm run gen:notice` and merge meaningful changes into [NOTICE](../NOTICE)
4. Skim new files for pasted proprietary samples or unlicensed media
5. Confirm README trademark disclaimer still accurate
6. For commercial plans: counsel review of iRacing terms + trademarks + patents

## Related

- [LICENSE](../LICENSE) — MIT for this repository’s original code
- [NOTICE](../NOTICE) — third-party attributions
- [PRIVACY.md](PRIVACY.md) — local data handling
- [SECURITY.md](../SECURITY.md) — vulnerability reporting
- [RELEASING.md](RELEASING.md) — tag / installer process
