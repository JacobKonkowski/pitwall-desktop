use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;

#[derive(Debug, Clone)]
pub struct ClipManifest {
    base_dir: PathBuf,
    clips: HashMap<String, String>,
}

impl ClipManifest {
    pub fn load(base_dir: PathBuf) -> anyhow::Result<Self> {
        let manifest_path = base_dir.join("manifest.json");
        let raw = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("read {}", manifest_path.display()))?;
        let clips: HashMap<String, String> = serde_json::from_str(&raw)?;
        Ok(Self { base_dir, clips })
    }

    /// A manifest with no clips; every clip lookup misses and playback skips it.
    pub fn empty(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            clips: HashMap::new(),
        }
    }

    pub fn path(&self, key: &str) -> Option<PathBuf> {
        self.clips
            .get(key)
            .map(|rel| self.base_dir.join(rel))
            .filter(|p| p.is_file())
    }
}
