/// Racing-radio phrasing helpers for times, deltas, and gaps.

pub fn format_duration_short(ms: f64) -> String {
    let total = ms / 1000.0;
    if total >= 60.0 {
        let min = (total / 60.0).floor() as i32;
        let sec = total - (min as f64 * 60.0);
        format!(
            "{min} minute{} {:.1} seconds",
            if min == 1 { "" } else { "s" },
            sec
        )
    } else {
        format!("{:.1} seconds", total)
    }
}

pub fn format_duration_long(ms: f64) -> String {
    format_duration_short(ms)
}

/// Tenths/seconds faster or slower — for use after `pace_off_pb_intro` clip.
pub fn format_delta_tts(delta_ms: f64) -> String {
    let abs = delta_ms.abs();
    let dir = if delta_ms > 0.0 { "slower" } else { "faster" };
    if abs < 1000.0 {
        format!("{:.0} tenths {dir}.", abs / 100.0)
    } else {
        format!("{:.1} seconds {dir}.", abs / 1000.0)
    }
}

pub fn format_gap_seconds(gap_s: f32) -> String {
    format!("{:.1} seconds", gap_s.abs())
}

/// WinRT-friendly lap time cadence: "3, 1 minute 42.3"
pub fn lap_time_tts(lap_num: i32, lap_ms: f64) -> String {
    let total = lap_ms / 1000.0;
    if total >= 60.0 {
        let min = (total / 60.0).floor() as i32;
        let sec = total - (min as f64 * 60.0);
        format!("{lap_num}, {min} minute {sec:.1}")
    } else {
        format!("{lap_num}, {total:.1}")
    }
}

pub fn sector_time_tts(sector_num: i32, ms: f64) -> String {
    let total = ms / 1000.0;
    format!("{sector_num}, {total:.1}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_tts_tenths() {
        assert!(format_delta_tts(350.0).contains("tenths slower"));
        assert!(format_delta_tts(-280.0).contains("faster"));
    }
}
