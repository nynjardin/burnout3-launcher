//! A zip of what helps understand a problem, to hand over by hand (nothing
//! is sent anywhere): the game's logs, its pc_settings.ini and a short
//! description of the PC, in journaux/ beside the launcher.

use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

pub struct Facts<'a> {
    pub launcher_version: &'a str,
    pub id: &'a str,
    pub name: &'a str,
    pub tag: Option<String>,
    pub exe: &'a str,
}

/// Writes the zip; its path.
pub fn export(launcher: &Path, game: &Path, facts: &Facts) -> Result<PathBuf, String> {
    let stem = Path::new(facts.exe).file_stem().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
    let mut files: Vec<PathBuf> = std::fs::read_dir(game)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
            p.is_file() && ((!stem.is_empty() && name.starts_with(&stem) && name.ends_with(".log")) || name == "pc_settings.ini")
        })
        .collect();
    files.sort();
    if !files.iter().any(|f| f.extension().is_some_and(|e| e == "log")) {
        return Err("Aucun journal du jeu dans son dossier : il n'a pas encore été lancé ?".into());
    }

    let folder = launcher.join("journaux");
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let out = folder.join(format!("journaux-{}-{}.zip", facts.id, chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")));
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&out).map_err(|e| e.to_string())?);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("infos.txt", options).map_err(|e| e.to_string())?;
    zip.write_all(info(facts).as_bytes()).map_err(|e| e.to_string())?;
    for file in &files {
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        let bytes = std::fs::read(file).map_err(|e| format!("{name} : {e}"))?;
        zip.start_file(name, options).map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(out)
}

fn info(facts: &Facts) -> String {
    format!(
        "Lanceur Burnout 3 v{}\nVersion du jeu : {} ({})\nInstallée : {}\nDate : {}\n\n{}\n",
        facts.launcher_version,
        facts.name,
        facts.id,
        facts.tag.as_deref().unwrap_or("inconnue"),
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S %:z"),
        crate::winfo::machine()
    )
}
