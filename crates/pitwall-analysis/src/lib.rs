//! Pure post-session analysis.
//!
//! Depends only on [`pitwall_telemetry`]. No storage, no Tauri, no I/O. The
//! pipeline turns raw frames into an [`AnalyzedSession`]; [`compare`] measures one
//! lap against another. Cleanup drops phantom reset buckets and sticky
//! `LapLastLapTime` copies; pace eligibility still requires the sim's `_OK`
//! flags plus near-full lap distance coverage.

pub mod aggregates;
pub mod cleanup;
pub mod compare;
pub mod pipeline;
pub mod sectors;
pub mod segment;
pub mod types;

pub use cleanup::{
    clear_sticky_times_in_place, finalize_analyzed_laps, has_full_coverage, is_phantom_lap,
    pace_eligible_from, FULL_LAP_PCT,
};
pub use compare::{compare_laps, AlignedPoint, CompareInput, LapComparison, SectorDelta};
pub use pipeline::analyze_session;
pub use types::{
    AnalyzedLap, AnalyzedSession, LapFrames, RawFrame, SectorBoundary, SessionMeta, TracePoint,
};
