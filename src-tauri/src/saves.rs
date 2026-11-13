//! Copies of a version's memory card (memcard/ beside its exe), in
//! sauvegardes/<id>/<date>_<why>/ beside the launcher: made before every
//! install and before a restore, or by hand; put back on demand.

use serde::Serialize;
use std::path::{Path, PathBuf};

const KEEP: usize = 15;

#[derive(Serialize)]
pub struct Backup {
    /// The folder's name: what restore takes.
    pub name: String,
    /// "2026-11-13 21:05:33", local time.
    pub date: String,
    pub why: String,
    pub bytes: u64,
}

fn root(launcher: &Path, id: &str) -> PathBuf {
    launcher.join("sauvegardes").join(id)
}

fn card(game: &Path) -> PathBuf {
    game.join("memcard")
}

fn size(dir: &Path) -> u64 {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| {
                    let p = e.path();
                    if p.is_dir() { size(&p) } else { e.metadata().map(|m| m.len()).unwrap_or(0) }
                })
                .sum()
        })
        .unwrap_or(0)
}

fn copy(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Copies the card now; None when there is none or it is empty (nothing to keep).
pub fn backup(launcher: &Path, id: &str, game: &Path, why: &str) -> Result<Option<String>, String> {
    let made = snapshot(launcher, id, game, why)?;
    prune(launcher, id);
    Ok(made)
}

fn snapshot(launcher: &Path, id: &str, game: &Path, why: &str) -> Result<Option<String>, String> {
    let card = card(game);
    if size(&card) == 0 {
        return Ok(None);
    }
    let slug: String = why
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+' | ' ') { c } else { '-' })
        .collect();
    let name = format!("{}_{slug}", chrono::Local::now().format("%Y-%m-%d_%H-%M-%S"));
    let target = root(launcher, id).join(&name);
    copy(&card, &target).map_err(|e| format!("Copie de la carte mémoire impossible : {e}"))?;
    Ok(Some(name))
}

/// The copies, newest first.
pub fn list(launcher: &Path, id: &str) -> Vec<Backup> {
    let mut names: Vec<String> = std::fs::read_dir(root(launcher, id))
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default();
    names.sort_by(|a, b| b.cmp(a));
    names
        .into_iter()
        .map(|name| {
            let (stamp, why) = name.split_once('_').map(|(d, rest)| {
                // "2026-11-13_21-05-33_why": the date's second half is in `rest`.
                match rest.split_once('_') {
                    Some((time, why)) => (format!("{d} {}", time.replace('-', ":")), why.to_string()),
                    None => (d.to_string(), rest.to_string()),
                }
            }).unwrap_or((name.clone(), String::new()));
            let bytes = size(&root(launcher, id).join(&name));
            Backup { name, date: stamp, why, bytes }
        })
        .collect()
}

/// Puts a copy back in place of the card, the card as it was kept first.
pub fn restore(launcher: &Path, id: &str, game: &Path, name: &str) -> Result<(), String> {
    if name.is_empty() || name.contains(['/', '\\']) || name.contains("..") {
        return Err("Sauvegarde inconnue.".into());
    }
    let from = root(launcher, id).join(name);
    if !from.is_dir() {
        return Err("Cette sauvegarde n'existe plus.".into());
    }
    // Pruned after the copy back: the one restored may be the oldest kept.
    snapshot(launcher, id, game, "avant restauration")?;
    let card = card(game);
    if card.exists() {
        std::fs::remove_dir_all(&card).map_err(|e| format!("La carte mémoire est utilisée ? {e}"))?;
    }
    copy(&from, &card).map_err(|e| format!("Restauration impossible : {e}"))?;
    prune(launcher, id);
    Ok(())
}

/// Only the KEEP newest copies stay.
fn prune(launcher: &Path, id: &str) {
    for old in list(launcher, id).into_iter().skip(KEEP) {
        let _ = std::fs::remove_dir_all(root(launcher, id).join(old.name));
    }
}
