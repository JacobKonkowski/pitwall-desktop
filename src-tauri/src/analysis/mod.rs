pub mod coach;
pub mod fuel_tire;
pub mod lap_kind;
pub mod lap_segmenter;
pub mod pipeline;
pub mod sector_splitter;
pub mod trace_coach;
pub mod types;

pub use coach::{build_coach_report, CoachInsight, CoachReport, SessionCoachStats};
pub use lap_kind::classify_lap_kind;
pub use pipeline::analyze_session;
pub use sector_splitter::compute_sector_times;
pub use types::*;
