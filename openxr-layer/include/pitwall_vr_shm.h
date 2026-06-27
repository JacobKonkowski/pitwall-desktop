// PitWall VR shared-memory contract.
//
// This header is the single source of truth for the binary layout exchanged
// between the PitWall desktop process (producer, written in Rust) and the
// pitwall-openxr-layer DLL (consumer, this C++ project). The Rust mirror lives
// in `src-tauri/src/vr/shm.rs` and MUST stay byte-for-byte identical.
//
// Layout rules that keep both sides in sync without compiler-specific packing:
//   * Every field is 4 bytes (i32 / u32 / f32) or a char array whose length is
//     a multiple of 4, so natural alignment never inserts hidden padding.
//   * 64-bit values are split into lo/hi u32 pairs for the same reason.
//   * "Absent" optional floats are encoded as NaN; absent positions use 0;
//     absent lap counts use -1.
//
// Concurrency: a seqlock on `seq`. The writer sets `seq` odd before mutating
// the block and even (incremented) after. A reader copies the block, then
// rechecks that `seq` is unchanged and even; otherwise it retries.

#ifndef PITWALL_VR_SHM_H
#define PITWALL_VR_SHM_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// "PWVR" as little-endian bytes ('P'=0x50, 'W'=0x57, 'V'=0x56, 'R'=0x52).
#define PITWALL_VR_MAGIC 0x52565750u
#define PITWALL_VR_VERSION 2u

#define PITWALL_VR_SHM_NAME "Local\\PitWallVR"

#define PITWALL_VR_MAX_OVERLAYS 4
#define PITWALL_VR_MAX_COMPETITORS 64
#define PITWALL_VR_MAX_SECTORS 8
#define PITWALL_VR_NUM_LEN 8
#define PITWALL_VR_NAME_LEN 40
#define PITWALL_VR_TRACK_LEN 64
#define PITWALL_VR_SESSION_LEN 32

// Overlay slot kind. v1 ships kind 0 (coach HUD). Slots 1..3 are reserved for
// the RaceLab-replacement widgets so the protocol does not break when they land.
enum PwOverlayKind {
    PW_OVERLAY_COACH = 0,
    PW_OVERLAY_STANDINGS = 1,
    PW_OVERLAY_RELATIVE = 2,
    PW_OVERLAY_RADAR = 3,
};

// Reference space the overlay quad is locked to.
enum PwLockSpace {
    PW_LOCK_VIEW = 0,   // head-locked (XR_REFERENCE_SPACE_TYPE_VIEW)
    PW_LOCK_LOCAL = 1,  // world-locked (XR_REFERENCE_SPACE_TYPE_LOCAL)
};

// One overlay's placement + visibility. 13 x 4 = 52 bytes.
typedef struct PwOverlay {
    uint32_t enabled;     // 0 / 1
    uint32_t kind;        // PwOverlayKind
    uint32_t lock_space;  // PwLockSpace
    float opacity;        // 0..1
    float pos_x;          // meters, relative to the locked space
    float pos_y;
    float pos_z;
    float rot_x;          // orientation quaternion (xyzw)
    float rot_y;
    float rot_z;
    float rot_w;
    float size_w;         // quad size in meters
    float size_h;
} PwOverlay;

// One competitor row for the standings / relative / radar overlays. 80 bytes.
typedef struct PwCompetitor {
    int32_t position;        // overall position, 0 = none
    int32_t class_position;  // class position, 0 = none
    int32_t class_id;
    float best_lap_ms;       // NaN = none
    float last_lap_ms;       // NaN = none
    float lap_dist_pct;      // 0..1 around the lap
    float gap_to_player_s;   // signed seconds; + = ahead of player
    uint32_t flags;          // bit0 = is_player, bit1 = on_pit_road
    char number[PITWALL_VR_NUM_LEN];
    char name[PITWALL_VR_NAME_LEN];
} PwCompetitor;

#define PW_COMPETITOR_IS_PLAYER 0x1u
#define PW_COMPETITOR_ON_PIT_ROAD 0x2u

// Field-pace display preference, mirrors AppSettings.vr_field_pace_mode.
enum PwFieldPaceMode {
    PW_FIELD_PACE_BEST = 0,
    PW_FIELD_PACE_OPTIMAL = 1,
    PW_FIELD_PACE_BOTH = 2,
};

// Compact mirror of LiveSnapshot, the data every overlay renders from.
typedef struct PwSnapshot {
    int32_t lap;
    float lap_time_ms;
    float last_lap_ms;             // NaN = none
    float best_lap_ms;             // NaN = none
    float delta_best_ms;           // NaN = none
    float delta_last_ms;           // NaN = none
    float delta_field_best_ms;     // NaN = none
    float delta_field_optimal_ms;  // NaN = none
    int32_t player_position;       // 0 = none
    int32_t player_class_position; // 0 = none
    float gap_ahead_s;             // NaN = none
    float gap_behind_s;            // NaN = none
    float fuel_level;
    float speed;
    float lap_dist_pct;
    int32_t current_sector;
    uint32_t pack_state;           // mirrors PackState ordinal
    uint32_t session_flags;        // raw iRacing SessionFlags bitfield
    int32_t incident_count;
    int32_t session_laps_remain;   // -1 = none
    float session_time_remain_s;   // NaN = none
    uint32_t on_track;             // 0 / 1
    uint32_t field_pace_mode;      // PwFieldPaceMode
    uint32_t sector_count;         // active sectors in sector_pct / sector_done
    float sector_pct[PITWALL_VR_MAX_SECTORS];   // 0..1 progress per sector
    uint32_t sector_done[PITWALL_VR_MAX_SECTORS]; // 0 / 1 completed flag per sector
    uint32_t competitor_count;
    char track[PITWALL_VR_TRACK_LEN];
    char session_type[PITWALL_VR_SESSION_LEN];
    PwCompetitor competitors[PITWALL_VR_MAX_COMPETITORS];
} PwSnapshot;

// PackState ordinals, matching src-tauri/src/live/pack.rs.
enum PwPackState {
    PW_PACK_OFF = 0,
    PW_PACK_CLEAR = 1,
    PW_PACK_CAR_LEFT = 2,
    PW_PACK_CAR_RIGHT = 3,
    PW_PACK_THREE_WIDE = 4,
    PW_PACK_TWO_LEFT = 5,
    PW_PACK_TWO_RIGHT = 6,
};

// Top-level shared block. Producer maps it writable; layer maps it read-only.
typedef struct PwSharedBlock {
    uint32_t magic;         // PITWALL_VR_MAGIC
    uint32_t version;       // PITWALL_VR_VERSION
    uint32_t seq;           // seqlock; odd while writing
    uint32_t overlay_count; // active overlays in `overlays`
    uint32_t write_ms_lo;   // low 32 bits of last write time (ms since epoch)
    uint32_t write_ms_hi;   // high 32 bits
    PwOverlay overlays[PITWALL_VR_MAX_OVERLAYS];
    PwSnapshot snapshot;
} PwSharedBlock;

#ifdef __cplusplus
}  // extern "C"
#endif

#endif  // PITWALL_VR_SHM_H
