# pitwall-openxr-layer

A standalone Windows OpenXR **API layer** that composites the PitWall HUD inside
the headset, the same mechanism RaceLab VR and OpenKneeboard use. It is built
outside the Tauri/Cargo tree because the OpenXR loader injects it into the
**iRacing** process, not into PitWall.

```
iRacing (OpenXR app)
   -> pitwall-openxr-layer.dll   (hooks xrEndFrame, appends quad layers)
   -> Meta / SteamVR / VDXR runtime
```

The DLL reads a shared-memory block (`Local\PitWallVR`, see
[`include/pitwall_vr_shm.h`](include/pitwall_vr_shm.h)) that the PitWall desktop
process writes from the live `LiveSnapshot`, then draws each enabled overlay with
Direct2D/DirectWrite and appends it as an `XrCompositionLayerQuad`.

## Build

Requires CMake 3.22+, a C++17 MSVC toolchain, and the Windows SDK (D3D11, D2D1,
DirectWrite). The OpenXR SDK headers are fetched automatically.

```powershell
cmake -S . -B build -A x64
cmake --build build --config Release
```

Output: `build/Release/pitwall-openxr-layer.dll` and a copy of
`pitwall_openxr_layer.json` beside it.

## Install (developer / manual)

Register the layer as an implicit API layer for the current user:

```powershell
reg add "HKCU\Software\Khronos\OpenXR\1\ApiLayers\Implicit" `
  /v "<full-path>\pitwall_openxr_layer.json" /t REG_DWORD /d 0 /f
```

A value of `0` means enabled. PitWall performs this registration through the
`install_vr_layer` command and the MSI installer; the manual command is for
local layer development. Set `PITWALL_VR_DISABLE=1` to bypass the layer without
unregistering it.

## Phase A POC (go/no-go gate)

The first milestone is a **static quad** in iRacing VR on Meta Quest Link:

1. Build and register the layer.
2. Temporarily hardcode one overlay (`enabled = 1`, `kind = COACH`, a fixed pose
   ~1.2 m forward) and skip the SHM read, or run PitWall so the block exists.
3. Launch iRacing in OpenXR mode on Quest Link and confirm:
   - the PitWall panel is visible and stable,
   - no black screen with iRacing alone,
   - no measurable FPS loss.

If the static quad does not render on Quest Link, stop and document — do not
invest further in the rendering pipeline. See
[`../docs/NATIVE_VR.md`](../docs/NATIVE_VR.md) and
[`../docs/VR_NATIVE_SPIKE.md`](../docs/VR_NATIVE_SPIKE.md).

## Files

| File | Role |
|------|------|
| `include/pitwall_vr_shm.h` | Shared-memory contract (mirrored by `src-tauri/src/vr/shm.rs`) |
| `src/layer.cpp` | Loader negotiation, dispatch, `xrEndFrame` quad injection |
| `src/shm_reader.h` | Seqlock reader for the producer's block |
| `src/hud_renderer.{h,cpp}` | Direct2D/DirectWrite overlay drawing |
| `manifest/pitwall_openxr_layer.json` | OpenXR API layer manifest |
