//! Lanceur Burnout 3: a portable launcher for the PC versions of the game.
//! Everything lives beside its exe: settings, catalogue, versions/, and the
//! web view's own data (.lanceur-webview/), so nothing is written elsewhere.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod catalogue;
mod disc;
mod gameini;
mod github;
mod installation;
mod journaux;
mod saves;
mod self_update;
mod settings;
mod winfo;

use catalogue::{Catalogue, GameVersion};
use installation::Installation;
use serde::Serialize;
use settings::{PlayStats, Settings};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

struct Launcher {
    folder: PathBuf,
    catalogue: Catalogue,
    settings: Mutex<Settings>,
    /// The launcher's own update found at start, if any.
    update: Mutex<Option<self_update::Update>>,
    /// The versions whose game runs, started from here.
    running: Mutex<HashSet<String>>,
}

impl Launcher {
    fn version(&self, id: &str) -> Result<&GameVersion, String> {
        self.catalogue.find(id).ok_or_else(|| format!("Version inconnue : {id}"))
    }

    fn installation<'a>(&'a self, id: &str) -> Result<(&'a GameVersion, Installation<'a>), String> {
        let version = self.version(id)?;
        Ok((version, Installation::new(&self.folder, version)))
    }

    fn is_running(&self, id: &str) -> bool {
        self.running.lock().unwrap().contains(id)
    }

    fn not_while_running(&self, id: &str) -> Result<(), String> {
        if self.is_running(id) {
            return Err("Le jeu tourne : ferme-le d'abord.".into());
        }
        Ok(())
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
    running: bool,
    /// Tags whose zip is kept (install_archived), the last used first.
    archived: Vec<String>,
}

/// A kind of disc: the image used and what it is.
#[derive(Serialize)]
struct DiscState {
    path: Option<String>,
    /// "choisi" (in the launcher), "jeu" (the game's own: its ini or beside it), "".
    origin: &'static str,
    check: Option<disc::DiscCheck>,
}

#[derive(Serialize)]
struct LauncherState {
    versions: Vec<VersionState>,
    settings: Settings,
    folder: String,
    launcher_version: &'static str,
    discs: HashMap<String, DiscState>,
}

#[derive(Clone, Serialize)]
struct SessionEnd {
    id: String,
    seconds: u64,
    code: Option<i32>,
    stats: PlayStats,
}

#[derive(Clone, Serialize)]
struct Progress {
    id: String,
    percent: u32,
}

#[tauri::command]
fn state(launcher: State<Launcher>) -> LauncherState {
    // Taken apart: a guard in the struct expression would live to its end,
    // and disc_states locks the settings too.
    let settings = launcher.settings.lock().unwrap().clone();
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
                    running: launcher.is_running(&v.id),
                    archived: installation.archived(),
                    version: v.clone(),
                }
            })
            .collect(),
        settings,
        folder: launcher.folder.display().to_string(),
        launcher_version: self_update::CURRENT,
        discs: disc_states(&launcher),
    }
}

/// Each kind of disc an offered version plays: the image chosen in the
/// launcher, else the one the game would take; checked.
#[tauri::command]
fn discs(launcher: State<Launcher>) -> HashMap<String, DiscState> {
    disc_states(&launcher)
}

fn disc_states(launcher: &Launcher) -> HashMap<String, DiscState> {
    let chosen = launcher.settings.lock().unwrap().discs.clone();
    let mut out = HashMap::new();
    for v in launcher.catalogue.versions.iter().filter(|v| v.active) {
        if out.contains_key(&v.disque) {
            continue;
        }
        let (path, origin) = match chosen.get(&v.disque) {
            Some(p) => (Some(PathBuf::from(p)), "choisi"),
            None => {
                let installation = Installation::new(&launcher.folder, v);
                match installation.present().then(|| disc::own_disc(&v.disque, &installation.folder)).flatten() {
                    Some(p) => (Some(p), "jeu"),
                    None => (None, ""),
                }
            }
        };
        let check = path.as_ref().map(|p| disc::check(&v.disque, p));
        out.insert(v.disque.clone(), DiscState { path: path.map(|p| p.display().to_string()), origin, check });
    }
    out
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
    launcher.not_while_running(&id)?;
    let all = github::releases(&version.depot).await?;
    let release = all.iter().find(|r| r.tag == tag).ok_or("Cette version n'est plus sur GitHub.")?;
    let installation = Installation::new(&launcher.folder, version);
    saves::backup(&launcher.folder, &id, &installation.folder, &format!("avant {tag}"))?;
    installation
        .install(release, |percent| {
            let _ = app.emit("progression", Progress { id: id.clone(), percent });
        })
        .await?;
    Ok(release.tag.clone())
}

/// Installs a zip kept from an earlier install (going back without internet).
#[tauri::command]
async fn install_archived(launcher: State<'_, Launcher>, id: String, tag: String) -> Result<String, String> {
    let (version, installation) = launcher.installation(&id)?;
    if !version.installable() {
        return Err("Cette version n'est pas proposée pour l'instant.".into());
    }
    launcher.not_while_running(&id)?;
    saves::backup(&launcher.folder, &id, &installation.folder, &format!("avant {tag}"))?;
    installation.install_archived(&tag)?;
    Ok(tag)
}

/// Starts the version; when its game ends, its time is added and a "partie"
/// event tells the page.
#[tauri::command]
fn play(app: tauri::AppHandle, launcher: State<Launcher>, id: String) -> Result<(), String> {
    let (version, installation) = launcher.installation(&id)?;
    if !version.active {
        return Err("Cette version n'est pas proposée pour l'instant.".into());
    }
    if !installation.present() {
        return Err(format!("{} est introuvable dans {}.", version.exe, installation.folder.display()));
    }
    launcher.not_while_running(&id)?;
    // A disc chosen in the launcher wins; without one the game uses its own.
    let disc = launcher.settings.lock().unwrap().disc(&version.disque).cloned();
    let mut child = installation.play(disc.as_ref())?;
    launcher.running.lock().unwrap().insert(id.clone());
    let started = chrono::Local::now().timestamp();
    std::thread::spawn(move || {
        let clock = std::time::Instant::now();
        let code = child.wait().ok().and_then(|s| s.code());
        let seconds = clock.elapsed().as_secs();
        let launcher = app.state::<Launcher>();
        launcher.running.lock().unwrap().remove(&id);
        let stats = {
            let mut settings = launcher.settings.lock().unwrap();
            let entry = settings.jeu.entry(id.clone()).or_default();
            entry.secondes += seconds;
            entry.parties += 1;
            entry.derniere = Some(started);
            let stats = entry.clone();
            let _ = settings.save(&launcher.folder);
            stats
        };
        let _ = app.emit("partie", SessionEnd { id, seconds, code, stats });
    });
    Ok(())
}

/// The version's game settings (its pc_settings.ini).
#[tauri::command]
fn game_settings(launcher: State<Launcher>, id: String) -> Result<BTreeMap<String, String>, String> {
    let (_, installation) = launcher.installation(&id)?;
    Ok(gameini::all(&installation.folder))
}

/// Sets one of the game's settings; None: back to the game's default.
#[tauri::command]
fn set_game_setting(launcher: State<Launcher>, id: String, key: String, value: Option<String>) -> Result<(), String> {
    let (version, installation) = launcher.installation(&id)?;
    if !version.reglages || !installation.present() {
        return Err("Pas de réglages pour cette version.".into());
    }
    if !gameini::EDITABLE.contains(&key.as_str()) {
        return Err(format!("Réglage inconnu : {key}"));
    }
    if let Some(v) = &value {
        if v.is_empty() || v.len() > 16 || !v.chars().all(|c| c.is_ascii_alphanumeric() || c == ':') {
            return Err(format!("Valeur inattendue : {v}"));
        }
    }
    gameini::set(&installation.folder, &key, value.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn saves(launcher: State<Launcher>, id: String) -> Vec<saves::Backup> {
    saves::list(&launcher.folder, &id)
}

/// Copies the memory card now; the copy's name.
#[tauri::command]
fn backup_now(launcher: State<Launcher>, id: String) -> Result<String, String> {
    let (_, installation) = launcher.installation(&id)?;
    saves::backup(&launcher.folder, &id, &installation.folder, "manuelle")?
        .ok_or_else(|| "La carte mémoire est vide : rien à sauvegarder.".into())
}

#[tauri::command]
fn restore_save(launcher: State<Launcher>, id: String, name: String) -> Result<(), String> {
    launcher.not_while_running(&id)?;
    let (_, installation) = launcher.installation(&id)?;
    saves::restore(&launcher.folder, &id, &installation.folder, &name)
}

#[tauri::command]
fn open_saves(launcher: State<Launcher>, id: String) -> Result<(), String> {
    let folder = launcher.folder.join("sauvegardes").join(&id);
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    winfo::open(folder.as_os_str())
}

/// Zips the game's logs and the PC's description in journaux/, and shows it
/// in the Explorer; its path.
#[tauri::command]
async fn export_logs(launcher: State<'_, Launcher>, id: String) -> Result<String, String> {
    let (version, installation) = launcher.installation(&id)?;
    let (launcher_folder, game) = (launcher.folder.clone(), installation.folder.clone());
    let (name, exe, tag) = (version.nom.clone(), version.exe.clone(), installation.tag());
    let zip = tauri::async_runtime::spawn_blocking(move || {
        let facts = journaux::Facts { launcher_version: self_update::CURRENT, id: &id, name: &name, tag, exe: &exe };
        journaux::export(&launcher_folder, &game, &facts)
    })
    .await
    .map_err(|e| e.to_string())??;
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("explorer").arg(format!("/select,{}", zip.display())).spawn();
    }
    Ok(zip.display().to_string())
}

/// The launcher forgets the disc of that kind: the game takes its own.
#[tauri::command]
fn forget_disc(launcher: State<Launcher>, kind: String) -> Result<(), String> {
    let mut settings = launcher.settings.lock().unwrap();
    settings.discs.remove(&kind);
    settings.save(&launcher.folder).map_err(|e| e.to_string())
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
    winfo::open(folder.as_os_str())
}

#[tauri::command]
fn open_page(launcher: State<Launcher>, id: String) -> Result<(), String> {
    winfo::open(std::ffi::OsStr::new(&launcher.version(&id)?.page()))
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
        running: Mutex::new(HashSet::new()),
        folder: folder.clone(),
    };
    self_update::clean_up();
    tauri::Builder::default()
        .manage(launcher)
        .invoke_handler(tauri::generate_handler![
            state, releases, install, play, choose_disc, remember, open_folder, open_page,
            launcher_update, update_launcher, install_archived, discs, game_settings, set_game_setting,
            saves, backup_now, restore_save, open_saves, export_logs, forget_disc
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
