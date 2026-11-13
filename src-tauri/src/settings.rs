//! The launcher's own settings, in lanceur.json beside it.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    /// "ps2_us", "xbox_us" -> disc image path.
    #[serde(default)]
    pub discs: HashMap<String, String>,
    /// The version shown last.
    #[serde(default)]
    pub selected: Option<String>,
    /// Version id -> its time played.
    #[serde(default)]
    pub jeu: HashMap<String, PlayStats>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct PlayStats {
    /// Seconds played, every session added.
    #[serde(default)]
    pub secondes: u64,
    #[serde(default)]
    pub parties: u32,
    /// The last session's start, Unix seconds.
    #[serde(default)]
    pub derniere: Option<i64>,
}

impl Settings {
    pub fn load(folder: &Path) -> Settings {
        std::fs::read_to_string(folder.join("lanceur.json"))
            .ok()
            .and_then(|text| serde_json::from_str(text.trim_start_matches('\u{feff}')).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, folder: &Path) -> std::io::Result<()> {
        let text = serde_json::to_string_pretty(self).expect("settings as JSON");
        std::fs::write(folder.join("lanceur.json"), text)
    }

    pub fn disc(&self, kind: &str) -> Option<&String> {
        self.discs.get(kind)
    }
}
