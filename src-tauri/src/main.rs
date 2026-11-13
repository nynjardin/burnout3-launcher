//! Lanceur Burnout 3: a portable launcher for the PC versions of the game.
//! Everything lives beside its exe: settings, catalogue, versions/, and the
//! web view's own data (.lanceur-webview/), so nothing is written elsewhere.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod catalogue;
mod github;
mod installation;
mod self_update;
mod settings;

use catalogue::{Catalogue, GameVersion};
use installation::Installation;
use serde::Serialize;
use settings::Settings;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Emitter, State, WebviewUrl, WebviewWindowBuilder};

struct Launcher {
    folder: PathBuf,
    catalogue: Catalogue,
    settings: Mutex<Settings>,
    /// The launcher's own update found at start, if any.
    update: Mutex<Option<self_update::Update>>,
}

impl Launcher {
    fn version(&self, id: &str) -> Result<&GameVersion, String> {
        self.catalogue.find(id).ok_or_else(|| format!("Version inconnue : {id}"))
    }
}

#[derive(Serialize)]
struct VersionState {
    #[serde(flatten)]
    version: GameVersion,
    installable: bool,
    page: String,
    present: bool,
    installed: Option<String>,
}

#[derive(Serialize)]
struct LauncherState {
    versions: Vec<VersionState>,
    settings: Settings,
    folder: String,
    launcher_version: &'static str,
}

#[derive(Clone, Serialize)]
struct Progress {
    id: String,
    percent: u32,
}

#[tauri::command]
fn state(launcher: State<Launcher>) -> LauncherState {
    LauncherState {
        versions: launcher
            .catalogue
            .versions
            .iter()
            .map(|v| {
                let installation = Installation::new(&launcher.folder, v);
                VersionState {
                    installable: v.installable(),
                    page: v.page(),
                    present: installation.present(),
                    installed: installation.tag(),
                    version: v.clone(),
                }
            })
            .collect(),
        settings: launcher.settings.lock().unwrap().clone(),
        folder: launcher.folder.display().to_string(),
        launcher_version: self_update::CURRENT,
    }
}

/// A newer launcher on its own repository, or None.
#[tauri::command]
async fn launcher_update(launcher: State<'_, Launcher>) -> Result<Option<self_update::Update>, String> {
    let source = &launcher.catalogue.lanceur;
    let found = self_update::check(&source.depot, &source.fichier).await?;
    *launcher.update.lock().unwrap() = found.clone();
    Ok(found)
}

/// Replaces the launcher by the update found, starts the new one and quits;
/// download progress as "progression" events with the id "lanceur".
#[tauri::command]
async fn update_launcher(app: tauri::AppHandle, launcher: State<'_, Launcher>) -> Result<(), String> {
    let update = launcher.update.lock().unwrap().clone().ok_or("Aucune mise à jour du lanceur.")?;
    self_update::apply(&update, |percent| {
        let _ = app.emit("progression", Progress { id: "lanceur".into(), percent });
    })
    .await?;
    app.exit(0);
    Ok(())
}

#[tauri::command]
async fn releases(launcher: State<'_, Launcher>, id: String) -> Result<Vec<github::Release>, String> {
    let depot = launcher.version(&id)?.depot.clone();
    github::releases(&depot).await
}

/// Installs the release `tag` of the version, telling its progress as
/// "progression" events.
#[tauri::command]
async fn install(app: tauri::AppHandle, launcher: State<'_, Launcher>, id: String, tag: String) -> Result<String, String> {
    let version = launcher.version(&id)?;
    if !version.installable() {
        return Err("Cette version n'est pas proposée pour l'instant.".into());
    }
    let all = github::releases(&version.depot).await?;
    let release = all.iter().find(|r| r.tag == tag).ok_or("Cette version n'est plus sur GitHub.")?;
    let installation = Installation::new(&launcher.folder, version);
    installation
        .install(release, |percent| {
            let _ = app.emit("progression", Progress { id: id.clone(), percent });
        })
        .await?;
    Ok(release.tag.clone())
}

#[tauri::command]
fn play(launcher: State<Launcher>, id: String) -> Result<(), String> {
    let version = launcher.version(&id)?;
    if !version.active {
        return Err("Cette version n'est pas proposée pour l'instant.".into());
    }
    let installation = Installation::new(&launcher.folder, version);
    if !installation.present() {
        return Err(format!("{} est introuvable dans {}.", version.exe, installation.folder.display()));
    }
    // A disc chosen in the launcher wins; without one the game uses its own.
    let settings = launcher.settings.lock().unwrap();
    installation.play(settings.disc(&version.disque))
}

/// Asks for a disc image of that kind ("ps2_us", "xbox_us"); its path, or None.
#[tauri::command]
async fn choose_disc(launcher: State<'_, Launcher>, kind: String) -> Result<Option<String>, String> {
    let picked = rfd::AsyncFileDialog::new()
        .set_title("Image du disque")
        .add_filter("Images disque", &["iso", "bin", "cue"])
        .add_filter("Tous les fichiers", &["*"])
        .pick_file()
        .await;
    let Some(file) = picked else { return Ok(None) };
    let path = file.path().display().to_string();
    let mut settings = launcher.settings.lock().unwrap();
    settings.discs.insert(kind, path.clone());
    settings.save(&launcher.folder).map_err(|e| e.to_string())?;
    Ok(Some(path))
}

#[tauri::command]
fn remember(launcher: State<Launcher>, id: String) {
    let mut settings = launcher.settings.lock().unwrap();
    if settings.selected.as_deref() != Some(id.as_str()) {
        settings.selected = Some(id);
        let _ = settings.save(&launcher.folder);
    }
}

#[tauri::command]
fn open_folder(launcher: State<Launcher>, id: String) -> Result<(), String> {
    let folder = Installation::new(&launcher.folder, launcher.version(&id)?).folder;
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    open::that_detached(folder).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_page(launcher: State<Launcher>, id: String) -> Result<(), String> {
    open::that_detached(launcher.version(&id)?.page()).map_err(|e| e.to_string())
}

fn main() {
    let folder = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    let launcher = Launcher {
        catalogue: Catalogue::load(&folder),
        settings: Mutex::new(Settings::load(&folder)),
        update: Mutex::new(None),
        folder: folder.clone(),
    };
    self_update::clean_up();
    tauri::Builder::default()
        .manage(launcher)
        .invoke_handler(tauri::generate_handler![
            state, releases, install, play, choose_disc, remember, open_folder, open_page,
            launcher_update, update_launcher
        ])
        .setup(move |app| {
            // The window built here, not in tauri.conf.json: its web view keeps
            // its data beside the exe (portable), not in %LOCALAPPDATA%.
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Lanceur Burnout 3")
                .inner_size(1040.0, 680.0)
                .min_inner_size(820.0, 540.0)
                .data_directory(folder.join(".lanceur-webview"))
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("the launcher's window");
}
