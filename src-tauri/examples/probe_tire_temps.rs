//! Probe tire-temp channel variance in an IBT.
//! cargo run --example probe_tire_temps -- "path\to\file.ibt"

use std::collections::BTreeSet;
use std::env;
use std::path::PathBuf;

use pitwall::ibt::IbtReader;
use pitwall::{VarData, VariableInfo};

fn main() -> anyhow::Result<()> {
    let path: PathBuf = env::args()
        .nth(1)
        .expect("usage: probe_tire_temps <path.ibt>")
        .into();

    let mut reader = IbtReader::open(&path)?;
    let schema = reader.variables();

    let names = [
        "LFtempCL", "LFtempCM", "LFtempCR",
        "RFtempCL", "RFtempCM", "RFtempCR",
        "LRtempCL", "LRtempCM", "LRtempCR",
        "RRtempCL", "RRtempCM", "RRtempCR",
        "LFtempL", "LFtempM", "LFtempR",
        "RFtempL", "RFtempM", "RFtempR",
        "LRtempL", "LRtempM", "LRtempR",
        "RRtempL", "RRtempM", "RRtempR",
    ];

    let mut infos: Vec<(&str, Option<VariableInfo>)> = names
        .iter()
        .map(|n| (*n, schema.get_variable(n).cloned()))
        .collect();

    println!("Present channels:");
    for (n, info) in &infos {
        println!("  {n}: {}", if info.is_some() { "yes" } else { "no" });
    }

    // Sample every 60th frame (~1 Hz at 60 Hz) for unique rounded values.
    let mut uniques: Vec<(&str, BTreeSet<i32>)> = infos
        .iter()
        .filter(|(_, i)| i.is_some())
        .map(|(n, _)| (*n, BTreeSet::new()))
        .collect();
    let mut mins: Vec<(&str, f32)> = uniques.iter().map(|(n, _)| (*n, f32::INFINITY)).collect();
    let mut maxs: Vec<(&str, f32)> = uniques.iter().map(|(n, _)| (*n, f32::NEG_INFINITY)).collect();

    let mut idx = 0usize;
    while let Some((data, _, _)) = reader.read_next_frame()? {
        if idx % 60 == 0 {
            for (n, info) in &infos {
                let Some(info) = info else { continue };
                let v = f32::from_bytes(&data, info).unwrap_or(0.0);
                if let Some((_, set)) = uniques.iter_mut().find(|(nn, _)| nn == n) {
                    set.insert((v * 10.0).round() as i32); // 0.1Â° buckets
                }
                if let Some((_, m)) = mins.iter_mut().find(|(nn, _)| nn == n) {
                    *m = m.min(v);
                }
                if let Some((_, m)) = maxs.iter_mut().find(|(nn, _)| nn == n) {
                    *m = m.max(v);
                }
            }
        }
        idx += 1;
    }

    println!("\nSampled ~{} frames (every 60th of {}):", idx / 60, idx);
    println!("{:<12} {:>8} {:>8} {:>8}", "channel", "min", "max", "uniq0.1");
    for (n, set) in &uniques {
        let min = mins.iter().find(|(nn, _)| nn == n).map(|(_, v)| *v).unwrap_or(0.0);
        let max = maxs.iter().find(|(nn, _)| nn == n).map(|(_, v)| *v).unwrap_or(0.0);
        println!("{n:<12} {min:>8.2} {max:>8.2} {:>8}", set.len());
    }


    Ok(())
}
