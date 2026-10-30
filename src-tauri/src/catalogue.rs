//! The versions of the game the launcher knows, and their cards.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Serialize, Deserialize)]
pub struct GameVersion {
    pub id: String,
    pub nom: String,
    /// GitHub "owner/name".
    pub depot: String,
    pub plateforme: String,
    /// Which disc image it plays: a key of the settings' `discs`.
    pub disque: String,
    /// Regex of the release asset to install; empty: nothing portable to install.
    #[serde(default)]
    pub fichier: String,
    /// The program in the version's folder.
    #[serde(default)]
    pub exe: String,
    #[serde(default)]
    pub apports: Vec<String>,
    #[serde(default)]
    pub remarque: String,
}

impl GameVersion {
    pub fn installable(&self) -> bool {
        !self.fichier.is_empty() && !self.exe.is_empty()
    }

    pub fn page(&self) -> String {
        format!("https://github.com/{}", self.depot)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Catalogue {
    pub versions: Vec<GameVersion>,
}

const BUILT_IN: &str = include_str!("../catalogue.json");

impl Catalogue {
    /// A catalogue.json beside the launcher wins over the one built in
    /// (a card corrected without a new launcher).
    pub fn load(folder: &Path) -> Catalogue {
        std::fs::read_to_string(folder.join("catalogue.json"))
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_else(|| serde_json::from_str(BUILT_IN).expect("the built-in catalogue"))
    }

    pub fn find(&self, id: &str) -> Option<&GameVersion> {
        self.versions.iter().find(|v| v.id == id)
    }
}
