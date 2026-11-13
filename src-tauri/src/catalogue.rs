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
    /// The game lives in the launcher's own folder (beside Lanceur.exe), not in
    /// versions/<id>/: the launcher is dropped into the game folder shared with friends.
    #[serde(default)]
    pub ici: bool,
    /// False: shown greyed out, not offered yet.
    #[serde(default = "yes")]
    pub active: bool,
    #[serde(default)]
    pub apports: Vec<String>,
    #[serde(default)]
    pub remarque: String,
}

fn yes() -> bool {
    true
}

impl GameVersion {
    pub fn installable(&self) -> bool {
        self.active && !self.fichier.is_empty() && !self.exe.is_empty()
    }

    pub fn page(&self) -> String {
        format!("https://github.com/{}", self.depot)
    }
}

/// Where the launcher finds its own releases.
#[derive(Clone, Serialize, Deserialize)]
pub struct LauncherSource {
    /// GitHub "owner/name".
    pub depot: String,
    /// Regex of the release asset: the new exe.
    pub fichier: String,
}

impl Default for LauncherSource {
    fn default() -> Self {
        LauncherSource { depot: "nynjardin/burnout3-launcher".into(), fichier: r"^Lanceur\.exe$".into() }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Catalogue {
    #[serde(default)]
    pub lanceur: LauncherSource,
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
