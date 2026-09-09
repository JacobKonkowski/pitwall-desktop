# Native In-Headset VR

PitWall’s goal for VR is a self-contained **in-headset HUD**: coach, standings, relative, and radar composited into the OpenXR frame while you drive. The desktop app publishes live data; PitWall’s OpenXR API layer draws the panels inside the iRacing OpenXR process.

Historical design research: [VR_NATIVE_SPIKE.md](VR_NATIVE_SPIKE.md). This guide is the current setup and architecture.

## Architecture

```
PitWall desktop (Tauri/Rust)              iRacing (OpenXR app)
  LiveService -> LiveSnapshot               |
     |                                       v
  vr::shm::ShmWriter --> Local\PitWallVR --> pitwall-openxr-layer.dll
  (30 Hz, seqlock)        shared memory       hooks xrEndFrame,
                                              draws the HUD with Direct2D,
                                              appends XrCompositionLayerQuad
                                                |
                                                v
                                         OpenXR runtime (Quest Link, SteamVR OpenXR, VDXR, …)
```

- **Producer:** [`src-tauri/src/vr/shm.rs`](../src-tauri/src/vr/shm.rs) writes a
  compact mirror of `LiveSnapshot` plus per-overlay placement into the named
  shared-memory block `Local\PitWallVR` at ~30 Hz, guarded by a seqlock.
- **Contract:** [`openxr-layer/include/pitwall_vr_shm.h`](../openxr-layer/include/pitwall_vr_shm.h)
  is the canonical byte layout; the Rust structs mirror it field-for-field.
- **Consumer:** the [`openxr-layer/`](../openxr-layer/) C++ DLL hooks `xrEndFrame`,
  reads the block, draws each enabled overlay with Direct2D/DirectWrite, and
  appends an `XrCompositionLayerQuad`.

The HTTP HUD in [`hud_server.rs`](../src-tauri/src/vr/hud_server.rs) is the
browser preview and the visual reference the Direct2D renderer mirrors. The layer
draws from the shared snapshot (not pre-rendered GPU textures) for robustness.

## Why a separate native DLL

A Tauri/Rust process cannot composite over another OpenXR app from the outside.
Consumer runtimes do not support a portable overlay extension for this use case,
so PitWall installs an **implicit OpenXR API layer** that the loader injects into
the iRacing process. See [VR_NATIVE_SPIKE.md](VR_NATIVE_SPIKE.md) for the research trail.

## Build the layer

Requires **CMake 3.22+**, **Visual Studio 2022 Build Tools** (Desktop C++ workload),
and the **Windows SDK**. The OpenXR SDK headers are fetched automatically on first
configure.

From the repo root:

```powershell
cmake -S openxr-layer -B openxr-layer/build -A x64
cmake --build openxr-layer/build --config Release
```

Output: `openxr-layer/build/Release/pitwall-openxr-layer.dll`

**Stage for PitWall** (required before **Install VR layer** or `npm run tauri build`):

```powershell
copy openxr-layer\build\Release\pitwall-openxr-layer.dll  src-tauri\resources\openxr-layer\
copy openxr-layer\manifest\pitwall_openxr_layer.json      src-tauri\resources\openxr-layer\
```

The DLL is a local build artifact (gitignored). The manifest JSON is copied beside
it so the OpenXR loader can find the layer when PitWall registers it.

If the build fails with `Cannot open include file: 'openxr/loader_interfaces.h'`,
update to a current checkout — the layer uses `openxr_loader_negotiation.h`
(OpenXR SDK 1.0.33+).

### Full dev workflow (native VR)

```powershell
# 1. Build + stage the OpenXR layer (once per layer code change)
cmake -S openxr-layer -B openxr-layer/build -A x64
cmake --build openxr-layer/build --config Release
copy openxr-layer\build\Release\pitwall-openxr-layer.dll  src-tauri\resources\openxr-layer\
copy openxr-layer\manifest\pitwall_openxr_layer.json      src-tauri\resources\openxr-layer\

# 2. Run PitWall
npm run tauri dev

# 3. In the app: Start live monitor → Settings → VR mode Native → Install VR layer
# 4. Start in-headset HUD, restart iRacing (OpenXR), enable widgets under Overlay widgets
```

### Release installer

After staging the DLL + manifest, build the MSI:

```powershell
npm run tauri build
```

## Install and enable

In PitWall: start the live monitor, then **Start in-headset HUD** with VR mode
set to **Native** (Settings → VR mode). If the layer is not yet registered, the
panel shows an **Install VR layer** button, which registers the manifest under
`HKCU\Software\Khronos\OpenXR\1\ApiLayers\Implicit`
(see [`layer_install.rs`](../src-tauri/src/vr/layer_install.rs)). Restart iRacing
after installing so the loader picks up the layer.

Set `PITWALL_VR_DISABLE=1` to bypass the layer without unregistering it. The layer
loads automatically once registered (no extra environment variable required).

**Compatibility note:** only one OpenXR API layer should composite overlays for a
given session. If the headset stays blank or the sim fails to start, disable other
implicit API layers, confirm iRacing is in **OpenXR** (not OpenVR), then retry.

## Quest 3 + Meta Link setup

1. Connect the Quest 3 via Meta Link (or Air Link) and set iRacing to **OpenXR**.
2. In PitWall: Settings → **VR mode: Native**, install the VR layer, start the
   in-headset HUD, then launch iRacing and get on track.
3. Under Settings → **Overlay widgets**, enable the widgets you want and tune
   each one's **VR height / scale / opacity**. Widgets are head-locked.
4. Choose **Field pace (coach)** (session best, optimal, or both) for the FLD/OPT
   readout on the coach widget.

## Overlay widgets

PitWall ships one shared widget catalog. Enable flags and field-pace preference
drive the Live in-app preview, the native layer, and the web HUD at `:17342`.
VR placement (height / scale / opacity) is tuned under Settings → Overlay widgets.

The protocol carries four head-locked overlay slots; the slot index equals the
widget kind, so each keeps a stable, correctly-sized swapchain:

| Slot | Widget | VR placement |
|------|--------|--------------|
| 0 | Coach HUD | Centered upper windshield (wide-short) |
| 1 | Standings strip | Lower-left (tall list) |
| 2 | Relative board | Lower-right (square) |
| 3 | Proximity radar | Low-center (square) |

Disabled widgets are published with `enabled = 0` and skipped by the compositor.
The web preview renders the same four layouts
(`/vr?layout=ironman|standings|relative|radar`).

## In-app and web preview

Enable or disable VR widget slots and field pace from Live / settings (`overlayLayout`).
The Live page shows an in-app coach preview; the same slot config drives the native
layer and the browser HUD at `http://127.0.0.1:17342/vr`.

## Troubleshooting

| Symptom | Check |
|---------|-------|
| HUD not visible in VR | iRacing in OpenXR mode? Layer installed and iRacing restarted? Other API layers off? `PITWALL_VR_DISABLE` unset? |
| "VR layer not installed" persists | Run **Install VR layer** again; confirm the registry value under the Implicit ApiLayers key |
| Black screen / crash on launch | Disable other implicit OpenXR API layers and retry to isolate load-order conflicts |
| HUD shows but no data | Live monitor running? Diagnostics **write age** should stay low while HUD is started |
| Compositor always false | Layer does not write a heartbeat file; status uses producer write age + layer installed |
| Spotter pack line never shows | Requires on-track traffic and `CarLeftRight` mapping in `live/pack.rs` |
