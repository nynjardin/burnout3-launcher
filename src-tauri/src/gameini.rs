//! A version's pc_settings.ini (the game's own settings, beside its exe):
//! `key=value` lines. The launcher changes only the keys it is asked to and
//! keeps every other line as it was.

use std::collections::BTreeMap;
use std::path::Path;

const FILE: &str = "pc_settings.ini";

/// The keys the launcher may change from its page (the game's PC SETTINGS
/// rows; `engine/host/src/pc_menu.cpp` in the game).
pub const EDITABLE: &[&str] =
    &["resolution", "aspect", "fullscreen", "vsync", "frame_pacing", "frame_rate", "texture_pack", "vrr", "gpu_boost"];

fn lines(folder: &Path) -> Vec<String> {
    match std::fs::read_to_string(folder.join(FILE)) {
        Ok(text) => text.lines().map(str::to_string).collect(),
        Err(_) => vec!["# The PC port's settings (PC SETTINGS in the main menu).".into()],
    }
}

fn split(line: &str) -> Option<(&str, &str)> {
    if line.starts_with('#') {
        return None;
    }
    line.split_once('=').map(|(k, v)| (k.trim(), v.trim()))
}

/// Every key=value of the file (the game's reading: trimmed, comments left out).
pub fn all(folder: &Path) -> BTreeMap<String, String> {
    lines(folder).iter().filter_map(|l| split(l)).map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

pub fn get(folder: &Path, key: &str) -> Option<String> {
    all(folder).remove(key).filter(|v| !v.is_empty())
}

/// Sets `key` (None: removes it, the game then takes its own default).
pub fn set(folder: &Path, key: &str, value: Option<&str>) -> std::io::Result<()> {
    let mut lines = lines(folder);
    let at = lines.iter().position(|l| split(l).is_some_and(|(k, _)| k == key));
    match (at, value) {
        (Some(at), Some(value)) => lines[at] = format!("{key}={value}"),
        (None, Some(value)) => lines.push(format!("{key}={value}")),
        (Some(at), None) => {
            lines.remove(at);
        }
        (None, None) => return Ok(()),
    }
    std::fs::write(folder.join(FILE), lines.join("\r\n") + "\r\n")
}
