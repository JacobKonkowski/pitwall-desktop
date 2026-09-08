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

pub fn track_temp_average(frames: &[RawFrame]) -> Option<f64> {
    if frames.is_empty() { return None; }
    let n = frames.len() as f64;
    Some(frames.iter().map(|f| f.track_temp as f64).sum::<f64>() / n)
}

pub fn track_wetn_average(frames: &[RawFrame]) -> Option<f64> {
    frames.last().map(|f| f.track_wetn as f64)
}

pub fn rel_humid_average(frames: &[RawFrame]) -> Option<f64> {
    if frames.is_empty() { return None; }
    let n = frames.len() as f64;
    Some(frames.iter().map(|f| f.rel_humid as f64).sum::<f64>() / n)
}

pub fn air_averages(frames: &[RawFrame]) -> (Option<f64>, Option<f64>, Option<f64>) {
    if frames.is_empty() { return (None, None, None); }
    let n = frames.len() as f64;
    let temp = frames.iter().map(|f| f.air_temp as f64).sum::<f64>() / n;
    let pres = frames.iter().map(|f| f.air_pres as f64).sum::<f64>() / n;
    let dens = frames.iter().map(|f| f.air_dens as f64).sum::<f64>() / n;
    (Some(temp), Some(pres), Some(dens))
}

pub fn wind_averages(frames: &[RawFrame]) -> (Option<f64>, Option<f64>) {
    if frames.is_empty() { return (None, None); }
    let n = frames.len() as f64;
    let dir = frames.iter().map(|f| f.wind_dir as f64).sum::<f64>() / n;
    let vel = frames.iter().map(|f| f.wind_vel as f64).sum::<f64>() / n;
    (Some(dir), Some(vel))
}

pub fn skies_averages(frames: &[RawFrame]) -> Option<f64> {
    if frames.is_empty() { return None; }
    let n = frames.len() as f64;
    Some(frames.iter().map(|f| f.skies as f64).sum::<f64>() / n)
}