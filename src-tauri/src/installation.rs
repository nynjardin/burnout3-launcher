//! A version's folder (versions/<id>/ beside the launcher, or the launcher's own
//! folder for a version marked `ici`): what is installed there, putting a
//! release in, starting it. The zips installed last are kept in
//! .lanceur-archives/<id>/ beside the launcher: going back needs no internet.

use crate::catalogue::GameVersion;
use crate::github::{self, Asset, Release};
use regex::RegexBuilder;
use std::io::Write;
use std::path::{Path, PathBuf};

const TAG_FILE: &str = ".lanceur-version";
/// The zips kept: the one installed and the one before.
const ARCHIVES_KEPT: usize = 2;

pub struct Installation<'a> {
    version: &'a GameVersion,
    pub folder: PathBuf,
    archives: PathBuf,
}

impl<'a> Installation<'a> {
    pub fn new(launcher_folder: &Path, version: &'a GameVersion) -> Self {
        let folder = if version.ici {
            launcher_folder.to_path_buf()
        } else {
            launcher_folder.join("versions").join(&version.id)
        };
        let archives = launcher_folder.join(".lanceur-archives").join(&version.id);
        Installation { version, folder, archives }
    }

    /// Its exe is there (installed by the launcher or put there by hand).
    pub fn present(&self) -> bool {
        !self.version.exe.is_empty() && self.folder.join(&self.version.exe).is_file()
    }

    /// The version there: the exe's own (its FileVersion, which the game takes
    /// from its game.json), else the tag the launcher installed last; None when
    /// not there or unknown.
    pub fn tag(&self) -> Option<String> {
        if !self.present() {
            return None;
        }
        crate::winfo::file_version(&self.folder.join(&self.version.exe))
            .or_else(|| std::fs::read_to_string(self.folder.join(TAG_FILE)).ok().map(|t| t.trim().to_string()))
            .filter(|t| !t.is_empty())
    }

    pub fn asset_of<'r>(&self, release: &'r Release) -> Option<&'r Asset> {
        if !self.version.installable() {
            return None;
        }
        let pattern = RegexBuilder::new(&self.version.fichier).case_insensitive(true).build().ok()?;
        release.assets.iter().find(|a| pattern.is_match(&a.name))
    }

    /// Downloads the release's zip (or takes the one kept) and writes its files
    /// over the folder's: only the files it carries (saves, settings, caches,
    /// texture packs stay). The tag is written last, so a broken update is
    /// offered again.
    pub async fn install(&self, release: &Release, mut progress: impl FnMut(u32)) -> Result<(), String> {
        let asset = self.asset_of(release).ok_or("Cette version n'a pas de fichier à installer.")?;
        let zip = self.archive_of(&release.tag)?;
        if crate::self_update::verify(&zip, asset).is_err() {
            std::fs::create_dir_all(&self.archives).map_err(|e| e.to_string())?;
            let part = zip.with_extension("part");
            let fetched = async {
                download(&asset.url, &part, &mut progress).await?;
                crate::self_update::verify(&part, asset)?;
                std::fs::rename(&part, &zip).map_err(|e| e.to_string())
            }
            .await;
            if fetched.is_err() {
                let _ = std::fs::remove_file(&part);
            }
            fetched?;
        }
        progress(100);
        self.install_zip(&zip, &release.tag)
    }

    /// Installs a zip kept from before (no internet needed).
    pub fn install_archived(&self, tag: &str) -> Result<(), String> {
        let zip = self.archive_of(tag)?;
        if !zip.is_file() {
            return Err(format!("Le zip de {tag} n'est plus gardé."));
        }
        self.install_zip(&zip, tag)
    }

    /// The tags whose zip is kept, the last used first.
    pub fn archived(&self) -> Vec<String> {
        let mut kept: Vec<(std::time::SystemTime, String)> = std::fs::read_dir(&self.archives)
            .map(|entries| {
                entries
                    .flatten()
                    .filter_map(|e| {
                        let path = e.path();
                        if path.extension()? != "zip" {
                            return None;
                        }
                        let when = e.metadata().and_then(|m| m.modified()).ok()?;
                        Some((when, path.file_stem()?.to_string_lossy().to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        kept.sort_by(|a, b| b.0.cmp(&a.0));
        kept.into_iter().map(|(_, tag)| tag).collect()
    }

    fn archive_of(&self, tag: &str) -> Result<PathBuf, String> {
        let safe = !tag.is_empty()
            && !tag.starts_with('.')
            && tag.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'));
        if !safe {
            return Err(format!("Étiquette de version inattendue : {tag}"));
        }
        Ok(self.archives.join(format!("{tag}.zip")))
    }

    fn install_zip(&self, zip: &Path, tag: &str) -> Result<(), String> {
        std::fs::create_dir_all(&self.folder).map_err(|e| e.to_string())?;
        extract(zip, &self.folder)?;
        std::fs::write(self.folder.join(TAG_FILE), tag).map_err(|e| e.to_string())?;
        // The zip used last is kept; the oldest go.
        if let Ok(file) = std::fs::File::options().write(true).open(zip) {
            let _ = file.set_modified(std::time::SystemTime::now());
        }
        for old in self.archived().into_iter().skip(ARCHIVES_KEPT) {
            let _ = std::fs::remove_file(self.archives.join(format!("{old}.zip")));
        }
        Ok(())
    }

    /// Starts the version, its disc given in its pc_settings.ini (disc=), the
    /// file's other lines kept. No disc: the game finds its own (its ini, or an
    /// image beside it). The game's process, to wait for its end.
    pub fn play(&self, disc: Option<&String>) -> Result<std::process::Child, String> {
        if let Some(disc) = disc {
            crate::gameini::set(&self.folder, "disc", Some(disc)).map_err(|e| e.to_string())?;
        }
        std::process::Command::new(self.folder.join(&self.version.exe))
            .current_dir(&self.folder)
            .spawn()
            .map_err(|e| e.to_string())
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
