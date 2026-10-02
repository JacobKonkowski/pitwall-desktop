//! Racing-radio phrasing for live numbers, tuned for the Piper voice: its espeak
//! phonemizer reads "1 29.5" as "one twenty-nine point five".

/// Lap / sector time cadence rounded to tenths: 89.45 s -> "1 29.5",
/// 65.3 s -> "1 oh 5.3", 42.3 s -> "42.3".
pub fn format_lap_time(ms: f64) -> String {
    let tenths = (ms / 100.0).round().max(0.0) as i64;
    let (min, sec_tenths) = (tenths / 600, tenths % 600);
    let sec = format!("{}.{}", sec_tenths / 10, sec_tenths % 10);
    match min {
        0 => sec,
        _ if sec_tenths < 100 => format!("{min} oh {sec}"),
        _ => format!("{min} {sec}"),
    }
}

/// "1 second", "2 seconds", "1.4 seconds" (rounded to tenths).
pub fn format_seconds(seconds: f64) -> String {
    let tenths = (seconds.abs() * 10.0).round() as i64;
    if tenths % 10 == 0 {
        let whole = tenths / 10;
        format!("{whole} second{}", if whole == 1 { "" } else { "s" })
    } else {
        format!("{}.{} seconds", tenths / 10, tenths % 10)
    }
}

/// Tenths/seconds faster or slower; for use after `pace_off_pb_intro` clip.
pub fn format_delta_tts(delta_ms: f64) -> String {
    let dir = if delta_ms > 0.0 { "slower" } else { "faster" };
    let tenths = (delta_ms.abs() / 100.0).round().max(1.0) as i64;
    if tenths < 10 {
        format!(
            "{tenths} tenth{} {dir}.",
            if tenths == 1 { "" } else { "s" }
        )
    } else {
        format!("{} {dir}.", format_seconds(delta_ms / 1000.0))
    }
}

pub fn format_gap_seconds(gap_s: f32) -> String {
    format_seconds(f64::from(gap_s))
}

/// Follows the `lap` clip: "12, 1 29.5".
pub fn lap_time_tts(lap_num: i32, lap_ms: f64) -> String {
    format!("{lap_num}, {}", format_lap_time(lap_ms))
}

/// Follows the `sector` clip: "2, 31.4".
pub fn sector_time_tts(sector_num: i32, ms: f64) -> String {
    format!("{sector_num}, {}", format_lap_time(ms))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_tts_tenths() {
        assert_eq!(format_delta_tts(350.0), "4 tenths slower.");
        assert_eq!(format_delta_tts(-280.0), "3 tenths faster.");
    }

    #[test]
    fn delta_tts_never_says_zero_tenths() {
        assert_eq!(format_delta_tts(60.0), "1 tenth slower.");
        assert_eq!(format_delta_tts(-120.0), "1 tenth faster.");
    }

    #[test]
    fn delta_tts_switches_to_seconds() {
        assert_eq!(format_delta_tts(960.0), "1 second slower.");
        assert_eq!(format_delta_tts(-1_440.0), "1.4 seconds faster.");
    }

    #[test]
    fn lap_time_cadence() {
        assert_eq!(format_lap_time(89_452.0), "1 29.5");
        assert_eq!(format_lap_time(65_300.0), "1 oh 5.3");
        assert_eq!(format_lap_time(42_340.0), "42.3");
        // Rounding carries into the minute instead of saying "1 60.0".
        assert_eq!(format_lap_time(119_970.0), "2 oh 0.0");
        assert_eq!(lap_time_tts(12, 89_452.0), "12, 1 29.5");
        assert_eq!(sector_time_tts(2, 31_420.0), "2, 31.4");
    }

    #[test]
    fn gaps_drop_trailing_zero() {
        assert_eq!(format_gap_seconds(1.04), "1 second");
        assert_eq!(format_gap_seconds(3.0), "3 seconds");
        assert_eq!(format_gap_seconds(2.36), "2.4 seconds");
    }
}
