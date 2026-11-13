//! A version's folder (versions/<id>/ beside the launcher, or the launcher's own
//! folder for a version marked `ici`): what is installed there, putting a
//! release in, starting it.

use crate::catalogue::GameVersion;
use crate::github::{self, Asset, Release};
use regex::RegexBuilder;
use std::io::Write;
use std::path::{Path, PathBuf};

const TAG_FILE: &str = ".lanceur-version";

pub struct Installation<'a> {
    version: &'a GameVersion,
    pub folder: PathBuf,
}

impl<'a> Installation<'a> {
    pub fn new(launcher_folder: &Path, version: &'a GameVersion) -> Self {
        let folder = if version.ici {
            launcher_folder.to_path_buf()
        } else {
            launcher_folder.join("versions").join(&version.id)
        };
        Installation { version, folder }
    }

    /// Its exe is there (installed by the launcher or put there by hand).
    pub fn present(&self) -> bool {
        !self.version.exe.is_empty() && self.folder.join(&self.version.exe).is_file()
    }

    /// The release tag installed by the launcher, or None (not there, or there
    /// without the launcher knowing which release).
    pub fn tag(&self) -> Option<String> {
        if !self.present() {
            return None;
        }
        std::fs::read_to_string(self.folder.join(TAG_FILE)).ok().map(|t| t.trim().to_string())
    }

    pub fn asset_of<'r>(&self, release: &'r Release) -> Option<&'r Asset> {
        if !self.version.installable() {
            return None;
        }
        let pattern = RegexBuilder::new(&self.version.fichier).case_insensitive(true).build().ok()?;
        release.assets.iter().find(|a| pattern.is_match(&a.name))
    }

    /// Downloads the release's zip and writes its files over the folder's: only
    /// the files it carries (saves, settings, caches, texture packs stay). The
    /// tag is written last, so a broken update is offered again.
    pub async fn install(&self, release: &Release, mut progress: impl FnMut(u32)) -> Result<(), String> {
        let asset = self.asset_of(release).ok_or("Cette version n'a pas de fichier à installer.")?;
        std::fs::create_dir_all(&self.folder).map_err(|e| e.to_string())?;
        let zip = self.folder.join(".telechargement.zip");
        let result = async {
            download(&asset.url, &zip, &mut progress).await?;
            extract(&zip, &self.folder)?;
            std::fs::write(self.folder.join(TAG_FILE), &release.tag).map_err(|e| e.to_string())
        }
        .await;
        let _ = std::fs::remove_file(&zip);
        result
    }

    /// Starts the version, its disc given in its pc_settings.ini (disc=), the
    /// file's other lines kept. No disc: the game finds its own (its ini, or an
    /// image beside it).
    pub fn play(&self, disc: Option<&String>) -> Result<(), String> {
        if let Some(disc) = disc {
            self.set_disc(disc).map_err(|e| e.to_string())?;
        }
        std::process::Command::new(self.folder.join(&self.version.exe))
            .current_dir(&self.folder)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn set_disc(&self, disc: &str) -> std::io::Result<()> {
        let ini = self.folder.join("pc_settings.ini");
        let mut lines: Vec<String> = match std::fs::read_to_string(&ini) {
            Ok(text) => text.lines().map(str::to_string).collect(),
            Err(_) => vec!["# The PC port's settings (PC SETTINGS in the main menu).".into()],
        };
        let line = format!("disc={disc}");
        match lines.iter().position(|l| l.starts_with("disc=")) {
            Some(at) => lines[at] = line,
            None => lines.push(line),
        }
        std::fs::write(&ini, lines.join("\r\n") + "\r\n")
    }
}

pub async fn download(url: &str, file: &Path, progress: &mut impl FnMut(u32)) -> Result<(), String> {
    let mut response = github::client().get(url).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Le téléchargement a répondu {}", response.status()));
    }
    let total = response.content_length().unwrap_or(0);
    let mut output = std::fs::File::create(file).map_err(|e| e.to_string())?;
    let (mut done, mut last) = (0u64, u32::MAX);
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        output.write_all(&chunk).map_err(|e| e.to_string())?;
        done += chunk.len() as u64;
        let percent = if total > 0 { (done * 100 / total) as u32 } else { 0 };
        if percent != last {
            last = percent;
            progress(percent);
        }
    }
    Ok(())
}

/// The release's zip holds one folder (Burnout3-PC-<version>/): its content goes in.
fn extract(zip: &Path, folder: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let names: Vec<String> = archive.file_names().map(str::to_string).collect();
    let top = names
        .first()
        .and_then(|n| n.find('/').map(|i| n[..=i].to_string()))
        .filter(|t| names.iter().all(|n| n.starts_with(t.as_str())))
        .unwrap_or_default();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        // No path out of the folder (enclosed_name refuses "..", roots).
        let Some(inside) = entry.enclosed_name() else { continue };
        let Ok(name) = inside.strip_prefix(top.trim_end_matches('/')) else { continue };
        if name.as_os_str().is_empty() {
            continue;
        }
        // Never the launcher's own files (Lanceur.exe, lanceur.json, .lanceur-*),
        // as it may share the game's folder.
        let first = name.components().next().map(|c| c.as_os_str().to_string_lossy().to_lowercase()).unwrap_or_default();
        if first.starts_with("lanceur") || first.starts_with(".lanceur") {
            continue;
        }
        let target = folder.join(name);
        if entry.is_dir() {
            std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        // Written beside, then put in place: a file half written never replaces one.
        let mut fresh = target.clone().into_os_string();
        fresh.push(".nouveau");
        let fresh = PathBuf::from(fresh);
        let mut out = std::fs::File::create(&fresh).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
        drop(out);
        let _ = std::fs::remove_file(&target);
        std::fs::rename(&fresh, &target).map_err(|e| e.to_string())?;
    }
    Ok(())
}
