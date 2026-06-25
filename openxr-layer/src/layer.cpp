// PitWall OpenXR API layer.
//
// Inserted by the OpenXR loader between iRacing and the active runtime (Meta,
// SteamVR, VDXR, ...). It hooks xrEndFrame and appends one composition-layer
// quad per enabled PitWall overlay, drawing the pixels with Direct2D from the
// shared-memory snapshot produced by the PitWall desktop process.
//
// Structure follows the standard implicit-layer pattern from
// Ybalrid/OpenXR-API-Layer-Template: negotiate -> create-instance shim ->
// per-function dispatch via xrGetInstanceProcAddr.

#include <d3d11.h>
#include <windows.h>

#include <chrono>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include <openxr/openxr.h>
#include <openxr/openxr_platform.h>
#include <openxr/openxr_loader_negotiation.h>

#include "hud_renderer.h"
#include "pitwall_vr_shm.h"
#include "shm_reader.h"

namespace {

constexpr int64_t kSwapchainFormatBgra = DXGI_FORMAT_B8G8R8A8_UNORM;
constexpr uint64_t kMaxSnapshotAgeMs = 2000;

uint64_t NowMs() {
    using namespace std::chrono;
    return duration_cast<milliseconds>(system_clock::now().time_since_epoch()).count();
}

// #region agent log
std::string LocalPitwallDir() {
    const char* localAppData = std::getenv("LOCALAPPDATA");
    std::string base = localAppData ? localAppData : "C:\\Users\\Public";
    return base + "\\pitwall-desktop";
}

void AppendAgentLogLine(const std::string& path, const std::string& line) {
    std::ofstream f(path, std::ios::app);
    if (f) {
        f << line;
    }
}

void AgentDebug(const char* hyp, const char* loc, const char* msg, const std::string& dataJson) {
    const auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
                        std::chrono::system_clock::now().time_since_epoch())
                        .count();
    const std::string line = std::string("{\"sessionId\":\"68355e\",\"hypothesisId\":\"") + hyp +
                             "\",\"location\":\"" + loc + "\",\"message\":\"" + msg +
                             "\",\"data\":" + dataJson + ",\"timestamp\":" + std::to_string(ms) +
                             ",\"runId\":\"pre-fix\",\"source\":\"layer\"}\n";
    const std::string dir = LocalPitwallDir();
    CreateDirectoryA(dir.c_str(), nullptr);
    AppendAgentLogLine(dir + "\\debug-68355e.log", line);
    AppendAgentLogLine(R"(c:\Users\jrkon\Projects\pitwall-desktop\debug-68355e.log)", line);
}

void WriteLayerHeartbeat() {
    const std::string dir = LocalPitwallDir();
    CreateDirectoryA(dir.c_str(), nullptr);
    std::ofstream f(dir + "\\layer-heartbeat", std::ios::trunc);
    if (f) {
        f << NowMs();
    }
}
// #endregion

// Per-kind swapchain dimensions.
void HudDimensions(uint32_t kind, uint32_t& width, uint32_t& height) {
    switch (kind) {
        case PW_OVERLAY_STANDINGS: width = 512; height = 640; break;  // tall list
        case PW_OVERLAY_RELATIVE:  width = 512; height = 512; break;  // square board
        case PW_OVERLAY_RADAR:     width = 512; height = 512; break;  // square dish
        case PW_OVERLAY_COACH:
        default:                   width = 1024; height = 288; break; // wide-short
    }
}

// One overlay slot's GPU resources.
struct OverlaySwapchain {
    XrSwapchain swapchain = XR_NULL_HANDLE;
    std::vector<ID3D11Texture2D*> images;
    uint32_t width = 0;
    uint32_t height = 0;
};

// Next-layer dispatch pointers we need.
struct Dispatch {
    PFN_xrGetInstanceProcAddr getInstanceProcAddr = nullptr;
    PFN_xrDestroyInstance destroyInstance = nullptr;
    PFN_xrCreateSession createSession = nullptr;
    PFN_xrDestroySession destroySession = nullptr;
    PFN_xrCreateReferenceSpace createReferenceSpace = nullptr;
    PFN_xrDestroySpace destroySpace = nullptr;
    PFN_xrEndFrame endFrame = nullptr;
    PFN_xrCreateSwapchain createSwapchain = nullptr;
    PFN_xrDestroySwapchain destroySwapchain = nullptr;
    PFN_xrEnumerateSwapchainImages enumerateSwapchainImages = nullptr;
    PFN_xrAcquireSwapchainImage acquireSwapchainImage = nullptr;
    PFN_xrWaitSwapchainImage waitSwapchainImage = nullptr;
    PFN_xrReleaseSwapchainImage releaseSwapchainImage = nullptr;
};

struct LayerContext {
    XrInstance instance = XR_NULL_HANDLE;
    Dispatch dispatch;

    XrSession session = XR_NULL_HANDLE;
    ID3D11Device* device = nullptr;
    XrSpace viewSpace = XR_NULL_HANDLE;
    XrSpace localSpace = XR_NULL_HANDLE;

    OverlaySwapchain overlays[PITWALL_VR_MAX_OVERLAYS];
    HudRenderer renderer;
    ShmReader shm;
    bool sessionReady = false;
};

LayerContext g_ctx;

// ---------------------------------------------------------------------------
// Swapchain helpers
// ---------------------------------------------------------------------------

bool EnsureOverlaySwapchain(int slot, uint32_t kind) {
    if (slot < 0 || slot >= PITWALL_VR_MAX_OVERLAYS) {
        return false;
    }
    OverlaySwapchain& ov = g_ctx.overlays[slot];
    if (ov.swapchain != XR_NULL_HANDLE) {
        return true;
    }

    HudDimensions(kind, ov.width, ov.height);

    XrSwapchainCreateInfo info{XR_TYPE_SWAPCHAIN_CREATE_INFO};
    info.usageFlags = XR_SWAPCHAIN_USAGE_COLOR_ATTACHMENT_BIT;
    info.format = kSwapchainFormatBgra;
    info.sampleCount = 1;
    info.width = ov.width;
    info.height = ov.height;
    info.faceCount = 1;
    info.arraySize = 1;
    info.mipCount = 1;

    if (XR_FAILED(g_ctx.dispatch.createSwapchain(g_ctx.session, &info, &ov.swapchain))) {
        ov.swapchain = XR_NULL_HANDLE;
        return false;
    }

    uint32_t count = 0;
    if (XR_FAILED(g_ctx.dispatch.enumerateSwapchainImages(ov.swapchain, 0, &count, nullptr)) ||
        count == 0) {
        return false;
    }
    std::vector<XrSwapchainImageD3D11KHR> images(
        count, {XR_TYPE_SWAPCHAIN_IMAGE_D3D11_KHR});
    if (XR_FAILED(g_ctx.dispatch.enumerateSwapchainImages(
            ov.swapchain, count, &count,
            reinterpret_cast<XrSwapchainImageBaseHeader*>(images.data())))) {
        return false;
    }
    ov.images.clear();
    for (auto& img : images) {
        ov.images.push_back(img.texture);
    }
    return true;
}

// Acquire/wait/render/release one overlay's swapchain image. Returns true if the
// image is ready to be referenced by a composition layer this frame.
bool RenderOverlay(int slot, const PwOverlay& overlay, const PwSnapshot& snapshot) {
    if (!EnsureOverlaySwapchain(slot, overlay.kind)) {
        return false;
    }
    OverlaySwapchain& ov = g_ctx.overlays[slot];

    uint32_t index = 0;
    XrSwapchainImageAcquireInfo acquire{XR_TYPE_SWAPCHAIN_IMAGE_ACQUIRE_INFO};
    if (XR_FAILED(g_ctx.dispatch.acquireSwapchainImage(ov.swapchain, &acquire, &index))) {
        return false;
    }
    XrSwapchainImageWaitInfo wait{XR_TYPE_SWAPCHAIN_IMAGE_WAIT_INFO};
    wait.timeout = XR_INFINITE_DURATION;
    if (XR_FAILED(g_ctx.dispatch.waitSwapchainImage(ov.swapchain, &wait))) {
        return false;
    }

    bool ok = false;
    if (index < ov.images.size()) {
        ok = g_ctx.renderer.Render(ov.images[index], overlay, snapshot, overlay.opacity);
    }

    XrSwapchainImageReleaseInfo release{XR_TYPE_SWAPCHAIN_IMAGE_RELEASE_INFO};
    g_ctx.dispatch.releaseSwapchainImage(ov.swapchain, &release);
    return ok;
}

XrSpace SpaceFor(uint32_t lockSpace) {
    return lockSpace == PW_LOCK_LOCAL ? g_ctx.localSpace : g_ctx.viewSpace;
}

// ---------------------------------------------------------------------------
// Hooked entry points
// ---------------------------------------------------------------------------

XRAPI_ATTR XrResult XRAPI_CALL PitWall_xrEndFrame(XrSession session,
                                                  const XrFrameEndInfo* frameEndInfo) {
    static uint64_t frameCounter = 0;
    frameCounter++;

    if (session != g_ctx.session || !g_ctx.sessionReady || frameEndInfo == nullptr) {
        // #region agent log
        if (frameCounter % 90 == 0) {
            std::ostringstream data;
            data << "{\"sessionMatch\":" << (session == g_ctx.session ? "true" : "false")
                 << ",\"sessionReady\":" << (g_ctx.sessionReady ? "true" : "false")
                 << ",\"hasFrameInfo\":" << (frameEndInfo != nullptr ? "true" : "false") << "}";
            AgentDebug("C", "layer.cpp:xrEndFrame", "early exit before composite", data.str());
        }
        // #endregion
        return g_ctx.dispatch.endFrame(session, frameEndInfo);
    }

    WriteLayerHeartbeat();

    PwSharedBlock block;
    if (!g_ctx.shm.Read(block, NowMs(), kMaxSnapshotAgeMs)) {
        // #region agent log
        if (frameCounter % 90 == 0) {
            AgentDebug("D", "layer.cpp:xrEndFrame", "shm read failed", "{}");
        }
        // #endregion
        return g_ctx.dispatch.endFrame(session, frameEndInfo);
    }

    std::vector<const XrCompositionLayerBaseHeader*> layers(
        frameEndInfo->layers, frameEndInfo->layers + frameEndInfo->layerCount);

    // Quad structs must outlive the endFrame call below.
    std::vector<XrCompositionLayerQuad> quads;
    quads.reserve(PITWALL_VR_MAX_OVERLAYS);

    const uint32_t overlayCount =
        block.overlay_count < PITWALL_VR_MAX_OVERLAYS ? block.overlay_count
                                                      : PITWALL_VR_MAX_OVERLAYS;
    uint32_t enabledCount = 0;
    uint32_t renderedCount = 0;
    uint32_t renderFailCount = 0;
    for (uint32_t i = 0; i < overlayCount; ++i) {
        const PwOverlay& ov = block.overlays[i];
        if (!ov.enabled) {
            continue;
        }
        enabledCount++;
        if (!RenderOverlay(static_cast<int>(i), ov, block.snapshot)) {
            renderFailCount++;
            continue;
        }
        renderedCount++;

        XrCompositionLayerQuad quad{XR_TYPE_COMPOSITION_LAYER_QUAD};
        quad.layerFlags = XR_COMPOSITION_LAYER_BLEND_TEXTURE_SOURCE_ALPHA_BIT;
        quad.space = SpaceFor(ov.lock_space);
        quad.eyeVisibility = XR_EYE_VISIBILITY_BOTH;
        quad.subImage.swapchain = g_ctx.overlays[i].swapchain;
        quad.subImage.imageArrayIndex = 0;
        quad.subImage.imageRect = {{0, 0},
                                   {static_cast<int32_t>(g_ctx.overlays[i].width),
                                    static_cast<int32_t>(g_ctx.overlays[i].height)}};
        quad.pose.orientation = {ov.rot_x, ov.rot_y, ov.rot_z, ov.rot_w};
        quad.pose.position = {ov.pos_x, ov.pos_y, ov.pos_z};
        quad.size = {ov.size_w, ov.size_h};
        quads.push_back(quad);
    }

    // #region agent log
    if (frameCounter % 90 == 0) {
        std::ostringstream data;
        data << "{\"enabledCount\":" << enabledCount << ",\"renderedCount\":" << renderedCount
             << ",\"renderFailCount\":" << renderFailCount << ",\"quadsAppended\":" << quads.size()
             << ",\"lap\":" << block.snapshot.lap << "}";
        AgentDebug("E", "layer.cpp:xrEndFrame", "composite frame", data.str());
    }
    // #endregion

    for (auto& quad : quads) {
        layers.push_back(reinterpret_cast<const XrCompositionLayerBaseHeader*>(&quad));
    }

    XrFrameEndInfo modified = *frameEndInfo;
    modified.layerCount = static_cast<uint32_t>(layers.size());
    modified.layers = layers.data();
    return g_ctx.dispatch.endFrame(session, &modified);
}

XRAPI_ATTR XrResult XRAPI_CALL PitWall_xrCreateSession(XrInstance instance,
                                                       const XrSessionCreateInfo* createInfo,
                                                       XrSession* session) {
    const XrResult res = g_ctx.dispatch.createSession(instance, createInfo, session);
    if (XR_FAILED(res)) {
        return res;
    }

    // Pull the D3D11 device from the graphics binding chain.
    const XrBaseInStructure* next = static_cast<const XrBaseInStructure*>(createInfo->next);
    while (next) {
        if (next->type == XR_TYPE_GRAPHICS_BINDING_D3D11_KHR) {
            g_ctx.device = reinterpret_cast<const XrGraphicsBindingD3D11KHR*>(next)->device;
            break;
        }
        next = next->next;
    }

    if (!g_ctx.device || !g_ctx.renderer.Initialize(g_ctx.device)) {
        // #region agent log
        std::ostringstream data;
        data << "{\"hasDevice\":" << (g_ctx.device ? "true" : "false") << "}";
        AgentDebug("C", "layer.cpp:xrCreateSession", "d3d/renderer init failed", data.str());
        // #endregion
        return res;  // session is valid; we simply will not composite
    }

    g_ctx.session = *session;

    XrReferenceSpaceCreateInfo viewInfo{XR_TYPE_REFERENCE_SPACE_CREATE_INFO};
    viewInfo.referenceSpaceType = XR_REFERENCE_SPACE_TYPE_VIEW;
    viewInfo.poseInReferenceSpace.orientation.w = 1.0f;
    g_ctx.dispatch.createReferenceSpace(*session, &viewInfo, &g_ctx.viewSpace);

    XrReferenceSpaceCreateInfo localInfo{XR_TYPE_REFERENCE_SPACE_CREATE_INFO};
    localInfo.referenceSpaceType = XR_REFERENCE_SPACE_TYPE_LOCAL;
    localInfo.poseInReferenceSpace.orientation.w = 1.0f;
    g_ctx.dispatch.createReferenceSpace(*session, &localInfo, &g_ctx.localSpace);

    g_ctx.sessionReady = (g_ctx.viewSpace != XR_NULL_HANDLE);
    // #region agent log
    AgentDebug("B", "layer.cpp:xrCreateSession", "session created",
               std::string("{\"sessionReady\":") + (g_ctx.sessionReady ? "true" : "false") + "}");
    // #endregion
    return res;
}

XRAPI_ATTR XrResult XRAPI_CALL PitWall_xrDestroySession(XrSession session) {
    if (session == g_ctx.session) {
        for (auto& ov : g_ctx.overlays) {
            if (ov.swapchain != XR_NULL_HANDLE) {
                g_ctx.dispatch.destroySwapchain(ov.swapchain);
                ov.swapchain = XR_NULL_HANDLE;
                ov.images.clear();
            }
        }
        if (g_ctx.viewSpace) g_ctx.dispatch.destroySpace(g_ctx.viewSpace);
        if (g_ctx.localSpace) g_ctx.dispatch.destroySpace(g_ctx.localSpace);
        g_ctx.viewSpace = g_ctx.localSpace = XR_NULL_HANDLE;
        g_ctx.session = XR_NULL_HANDLE;
        g_ctx.sessionReady = false;
    }
    return g_ctx.dispatch.destroySession(session);
}

XRAPI_ATTR XrResult XRAPI_CALL PitWall_xrDestroyInstance(XrInstance instance) {
    PFN_xrDestroyInstance down = g_ctx.dispatch.destroyInstance;
    g_ctx = LayerContext{};
    return down ? down(instance) : XR_SUCCESS;
}

XRAPI_ATTR XrResult XRAPI_CALL PitWall_xrGetInstanceProcAddr(XrInstance instance,
                                                             const char* name,
                                                             PFN_xrVoidFunction* function) {
    const auto bind = [&](PFN_xrVoidFunction fn) {
        *function = fn;
        return XR_SUCCESS;
    };
    if (std::strcmp(name, "xrEndFrame") == 0)
        return bind(reinterpret_cast<PFN_xrVoidFunction>(PitWall_xrEndFrame));
    if (std::strcmp(name, "xrCreateSession") == 0)
        return bind(reinterpret_cast<PFN_xrVoidFunction>(PitWall_xrCreateSession));
    if (std::strcmp(name, "xrDestroySession") == 0)
        return bind(reinterpret_cast<PFN_xrVoidFunction>(PitWall_xrDestroySession));
    if (std::strcmp(name, "xrDestroyInstance") == 0)
        return bind(reinterpret_cast<PFN_xrVoidFunction>(PitWall_xrDestroyInstance));
    return g_ctx.dispatch.getInstanceProcAddr(instance, name, function);
}

template <typename T>
void Load(const char* name, T& slot) {
    PFN_xrVoidFunction fn = nullptr;
    if (XR_SUCCEEDED(g_ctx.dispatch.getInstanceProcAddr(g_ctx.instance, name, &fn))) {
        slot = reinterpret_cast<T>(fn);
    }
}

XRAPI_ATTR XrResult XRAPI_CALL PitWall_xrCreateApiLayerInstance(
    const XrInstanceCreateInfo* info, const XrApiLayerCreateInfo* apiLayerInfo,
    XrInstance* instance) {
    XrApiLayerNextInfo* nextInfo = apiLayerInfo->nextInfo;
    g_ctx.dispatch.getInstanceProcAddr = nextInfo->nextGetInstanceProcAddr;

    XrApiLayerCreateInfo nextLayerInfo = *apiLayerInfo;
    nextLayerInfo.nextInfo = nextInfo->next;
    const XrResult res =
        nextInfo->nextCreateApiLayerInstance(info, &nextLayerInfo, instance);
    if (XR_FAILED(res)) {
        return res;
    }

    g_ctx.instance = *instance;
    // #region agent log
    AgentDebug("B", "layer.cpp:xrCreateApiLayerInstance", "layer instance created", "{}");
    // #endregion
    Load("xrDestroyInstance", g_ctx.dispatch.destroyInstance);
    Load("xrCreateSession", g_ctx.dispatch.createSession);
    Load("xrDestroySession", g_ctx.dispatch.destroySession);
    Load("xrCreateReferenceSpace", g_ctx.dispatch.createReferenceSpace);
    Load("xrDestroySpace", g_ctx.dispatch.destroySpace);
    Load("xrEndFrame", g_ctx.dispatch.endFrame);
    Load("xrCreateSwapchain", g_ctx.dispatch.createSwapchain);
    Load("xrDestroySwapchain", g_ctx.dispatch.destroySwapchain);
    Load("xrEnumerateSwapchainImages", g_ctx.dispatch.enumerateSwapchainImages);
    Load("xrAcquireSwapchainImage", g_ctx.dispatch.acquireSwapchainImage);
    Load("xrWaitSwapchainImage", g_ctx.dispatch.waitSwapchainImage);
    Load("xrReleaseSwapchainImage", g_ctx.dispatch.releaseSwapchainImage);
    return res;
}

}  // namespace

extern "C" {

// Entry point the OpenXR loader looks up by name (see the manifest's
// negotiation contract). Advertises our two shim functions.
XRAPI_ATTR XrResult XRAPI_CALL xrNegotiateLoaderApiLayerInterface(
    const XrNegotiateLoaderInfo* loaderInfo, const char* /*layerName*/,
    XrNegotiateApiLayerRequest* apiLayerRequest) {
    // #region agent log
    AgentDebug("B", "layer.cpp:xrNegotiateLoaderApiLayerInterface", "negotiate called", "{}");
    // #endregion
    if (!loaderInfo || !apiLayerRequest ||
        loaderInfo->structType != XR_LOADER_INTERFACE_STRUCT_LOADER_INFO ||
        apiLayerRequest->structType != XR_LOADER_INTERFACE_STRUCT_API_LAYER_REQUEST) {
        return XR_ERROR_INITIALIZATION_FAILED;
    }
    apiLayerRequest->layerInterfaceVersion = XR_CURRENT_LOADER_API_LAYER_VERSION;
    apiLayerRequest->layerApiVersion = XR_CURRENT_API_VERSION;
    apiLayerRequest->getInstanceProcAddr = PitWall_xrGetInstanceProcAddr;
    apiLayerRequest->createApiLayerInstance = PitWall_xrCreateApiLayerInstance;
    return XR_SUCCESS;
}

BOOL WINAPI DllMain(HINSTANCE, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        // #region agent log
        OutputDebugStringA("PitWallVR: pitwall-openxr-layer.dll attached\n");
        AgentDebug("B", "layer.cpp:DllMain", "dll attached", "{}");
        // #endregion
    }
    return TRUE;
}

}  // extern "C"
