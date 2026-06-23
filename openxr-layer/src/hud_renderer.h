// Direct2D/DirectWrite renderer for the PitWall overlays.
//
// The layer composites quads, but the actual pixels are drawn here from the
// shared-memory snapshot. Keeping the draw code in the layer (rather than
// shipping pixels over SHM) avoids fragile cross-process GPU texture sharing;
// the HTML layouts in src-tauri/src/vr/hud_server.rs remain the browser preview
// and the visual reference this renderer mirrors.

#pragma once

#include <d2d1_1.h>
#include <d3d11.h>
#include <dwrite.h>
#include <wrl/client.h>

#include "pitwall_vr_shm.h"

class HudRenderer {
public:
    HudRenderer() = default;
    ~HudRenderer() = default;

    // Bind to the session's D3D11 device. Safe to call repeatedly; only the
    // first successful call allocates the device-independent resources.
    bool Initialize(ID3D11Device* device);

    // Draw one overlay into `target` (a BGRA swapchain texture) from `snapshot`.
    // `opacity` scales the whole overlay. Returns false on a hard device error.
    bool Render(ID3D11Texture2D* target, const PwOverlay& overlay,
                const PwSnapshot& snapshot, float opacity);

private:
    Microsoft::WRL::ComPtr<ID2D1Factory1> m_d2dFactory;
    Microsoft::WRL::ComPtr<IDWriteFactory> m_dwriteFactory;
    Microsoft::WRL::ComPtr<ID2D1Device> m_d2dDevice;
    Microsoft::WRL::ComPtr<ID2D1DeviceContext> m_d2dContext;

    Microsoft::WRL::ComPtr<IDWriteTextFormat> m_hero;     // lap time
    Microsoft::WRL::ComPtr<IDWriteTextFormat> m_label;    // small caps labels
    Microsoft::WRL::ComPtr<IDWriteTextFormat> m_value;    // delta / gap values
    Microsoft::WRL::ComPtr<IDWriteTextFormat> m_badge;    // flag / pack badge

    bool m_ready = false;

    void DrawCoach(const PwSnapshot& s, float w, float h);
    void DrawStandings(const PwSnapshot& s, float w, float h);
    void DrawRelative(const PwSnapshot& s, float w, float h);
    void DrawRadar(const PwSnapshot& s, float w, float h);

    void DrawText(const wchar_t* text, IDWriteTextFormat* fmt, D2D1_RECT_F rect,
                  D2D1_COLOR_F color);
    void DrawCornerBrackets(D2D1_RECT_F rect, D2D1_COLOR_F color);
};
