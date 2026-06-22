use super::types::RawFrame;

pub fn fuel_stats(frames: &[RawFrame]) -> (Option<f64>, Option<f64>) {
    if frames.is_empty() {
        return (None, None);
    }
    let start = frames.first().map(|f| f.fuel_level as f64);
    let end = frames.last().map(|f| f.fuel_level as f64);
    let used = match (start, end) {
        (Some(s), Some(e)) if s >= e => Some(s - e),
        _ => None,
    };
    (start, used)
}

pub fn tire_averages(frames: &[RawFrame]) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if frames.is_empty() {
        return (None, None, None, None);
    }
    let n = frames.len() as f64;
    let lf: f64 = frames.iter().map(|f| f.lf_temp as f64).sum::<f64>() / n;
    let rf: f64 = frames.iter().map(|f| f.rf_temp as f64).sum::<f64>() / n;
    let lr: f64 = frames.iter().map(|f| f.lr_temp as f64).sum::<f64>() / n;
    let rr: f64 = frames.iter().map(|f| f.rr_temp as f64).sum::<f64>() / n;
    (Some(lf), Some(rf), Some(lr), Some(rr))
}
