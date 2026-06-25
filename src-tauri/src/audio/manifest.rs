use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

    pub fn path(&self, key: &str) -> Option<PathBuf> {
        self.clips
            .get(key)
            .map(|rel| self.base_dir.join(rel))
            .filter(|p| p.is_file())
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}
