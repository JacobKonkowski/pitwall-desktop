use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Parse `scripts/audio-phrases.txt` (`key=spoken text`, `#` comments).
pub fn load_phrases_file(path: &Path) -> anyhow::Result<HashMap<String, String>> {
    let raw = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("read {}: {e}", path.display()))?;
    let mut out = HashMap::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, text)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let text = text.trim();
        if key.is_empty() || text.is_empty() {
            continue;
        }
        out.insert(key.to_string(), text.to_string());
    }
    if out.is_empty() {
        anyhow::bail!("no phrases in {}", path.display());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_phrase_lines() {
        let dir = std::env::temp_dir().join("pitwall_phrases_test.txt");
        fs::write(&dir, "# comment\nflag_red=Red flag.\nlap=Lap.\n").unwrap();
        let map = load_phrases_file(&dir).unwrap();
        assert_eq!(map.get("flag_red").map(String::as_str), Some("Red flag."));
        assert_eq!(map.len(), 2);
        let _ = fs::remove_file(dir);
    }
}
