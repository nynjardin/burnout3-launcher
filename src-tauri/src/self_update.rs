//! The launcher updating itself from its own GitHub releases: a newer tag than
//! its version (Cargo.toml) carrying a Lanceur.exe. The running exe cannot be
//! overwritten but can be renamed on Windows: the new one is downloaded beside
//! it, the running one renamed to <exe>.ancien, the new one put in its place and
//! started; the next start removes the old one.

use crate::github::{self, Asset};
use regex::RegexBuilder;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::io::Read;
use std::path::{Path, PathBuf};

pub const CURRENT: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Serialize)]
pub struct Update {
    pub tag: String,
    pub notes: String,
    pub prerelease: bool,
    #[serde(skip)]
    pub asset: Asset,
}

/// The newest release above the running version with an asset matching
/// `pattern`; None when there is none or the repository does not exist yet.
pub async fn check(repository: &str, pattern: &str) -> Result<Option<Update>, String> {
    let pattern = RegexBuilder::new(pattern).case_insensitive(true).build().map_err(|e| e.to_string())?;
    let releases = match github::releases(repository).await {
        Ok(list) => list,
        Err(e) if e.contains("404") => return Ok(None),
        Err(e) => return Err(e),
    };
    let best = releases
        .into_iter()
        .filter(|r| newer(&r.tag, CURRENT) == Ordering::Greater)
        .filter_map(|r| {
            let asset = r.assets.iter().find(|a| pattern.is_match(&a.name))?.clone();
            Some(Update { tag: r.tag, notes: r.notes, prerelease: r.prerelease, asset })
        })
        .max_by(|a, b| newer(&a.tag, &b.tag));
    Ok(best)
}

/// Downloads the update, checks it, swaps the exe and starts the new one.
/// The caller then quits.
pub async fn apply(update: &Update, mut progress: impl FnMut(u32)) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let fresh = beside(&exe, "nouveau");
    let old = beside(&exe, "ancien");
    let result = async {
        crate::installation::download(&update.asset.url, &fresh, &mut progress).await?;
        verify(&fresh, &update.asset)?;
        let _ = std::fs::remove_file(&old);
        std::fs::rename(&exe, &old).map_err(|e| format!("Impossible de renommer le lanceur : {e}"))?;
        if let Err(e) = std::fs::rename(&fresh, &exe) {
            let _ = std::fs::rename(&old, &exe);
            return Err(format!("Impossible de poser le nouveau lanceur : {e}"));
        }
        Ok(())
    }
    .await;
    if result.is_err() {
        let _ = std::fs::remove_file(&fresh);
        return result;
    }
    let folder = exe.parent().map(Path::to_path_buf).unwrap_or_default();
    std::process::Command::new(&exe)
        .current_dir(folder)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Le nouveau lanceur ne démarre pas : {e}"))
}

/// Removes what an update left (the old exe, still running for a moment after
/// the new one starts; a download cut short), retrying for a few seconds.
pub fn clean_up() {
    let Ok(exe) = std::env::current_exe() else { return };
    std::thread::spawn(move || {
        for _ in 0..20 {
            let left: Vec<PathBuf> = ["ancien", "nouveau"].iter().map(|s| beside(&exe, s)).filter(|p| p.exists()).collect();
            if left.is_empty() {
                return;
            }
            for file in left {
                let _ = std::fs::remove_file(file);
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}

/// Lanceur.exe -> Lanceur.exe.<suffix>
fn beside(exe: &Path, suffix: &str) -> PathBuf {
    let mut name = exe.as_os_str().to_os_string();
    name.push(".");
    name.push(suffix);
    PathBuf::from(name)
}

/// The size GitHub gives, and its SHA-256 when it gives one ("sha256:<hex>").
fn verify(file: &Path, asset: &Asset) -> Result<(), String> {
    let size = std::fs::metadata(file).map_err(|e| e.to_string())?.len();
    if size != asset.size {
        return Err(format!("Téléchargement incomplet ({size} octets sur {}).", asset.size));
    }
    let Some(expected) = asset.digest.as_deref().and_then(|d| d.strip_prefix("sha256:")) else {
        return Ok(());
    };
    let mut hasher = Sha256::new();
    let mut input = std::fs::File::open(file).map_err(|e| e.to_string())?;
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let n = input.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let actual: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    if !actual.eq_ignore_ascii_case(expected) {
        return Err("Le fichier téléchargé est abîmé (empreinte différente).".into());
    }
    Ok(())
}

/// Compares two versions as semver ("v0.2.0", "0.2.0-beta.1"): a version with a
/// pre-release part is below the same one without.
pub fn newer(a: &str, b: &str) -> Ordering {
    let (a_core, a_pre) = split(a);
    let (b_core, b_pre) = split(b);
    a_core.cmp(&b_core).then_with(|| match (a_pre, b_pre) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(x), Some(y)) => compare_pre(x, y),
    })
}

fn split(version: &str) -> (Vec<u64>, Option<&str>) {
    let version = version.trim().trim_start_matches(['v', 'V']);
    let version = version.split('+').next().unwrap_or_default();
    let (core, pre) = match version.split_once('-') {
        Some((c, p)) => (c, Some(p)),
        None => (version, None),
    };
    let mut numbers: Vec<u64> = core.split('.').map(|n| n.parse().unwrap_or(0)).collect();
    numbers.resize(3, 0);
    (numbers, pre)
}

fn compare_pre(a: &str, b: &str) -> Ordering {
    let mut x = a.split('.');
    let mut y = b.split('.');
    loop {
        match (x.next(), y.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(p), Some(q)) => {
                let order = match (p.parse::<u64>(), q.parse::<u64>()) {
                    (Ok(m), Ok(n)) => m.cmp(&n),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => p.cmp(q),
                };
                if order != Ordering::Equal {
                    return order;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::newer;
    use std::cmp::Ordering::*;

    #[test]
    fn versions() {
        assert_eq!(newer("v0.2.0", "0.1.0"), Greater);
        assert_eq!(newer("v0.1.0", "0.1.0"), Equal);
        assert_eq!(newer("v0.10.0", "0.9.3"), Greater);
        assert_eq!(newer("v0.2.0-beta.1", "0.2.0"), Less);
        assert_eq!(newer("v0.2.0-beta.2", "0.2.0-beta.10"), Less);
        assert_eq!(newer("v0.2.0-beta.1", "0.1.9"), Greater);
        assert_eq!(newer("lanceur", "0.1.0"), Less);
    }
}
