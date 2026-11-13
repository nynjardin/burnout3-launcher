// The launcher's page: the versions on the left, the chosen one on the right.
// It talks to the Rust side through window.__TAURI__ (withGlobalTauri).
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const $ = (id) => document.getElementById(id);
let state = null;              // { versions, settings, folder, discs } from the Rust side
let chosen = null;             // the version id shown
const releases = {};           // id -> [release] once GitHub answered
const errors = {};             // id -> why it did not
let busy = null;               // the version id being installed
let launcherUpdate = null;     // { tag, notes, prerelease } when a newer launcher exists
let updatingLauncher = false;
let gameSettings = {};         // the chosen version's pc_settings.ini
let saves = [];                // the chosen version's memory card copies

// The game's PC SETTINGS rows the launcher shows (keys of pc_settings.ini;
// the game's engine/host/src/pc_menu.cpp). "" = the game's own default.
const ON_OFF = [["on", "Oui"], ["off", "Non"]];
const GAME_SETTINGS = [
  { key: "resolution", label: "Résolution", help: "La résolution à laquelle le jeu est dessiné.",
    options: [["original", "Originale (640×448)"], ["2x", "2×"], ["3x", "3×"], ["4x", "4×"], ["display", "Celle de l'écran"]] },
  { key: "aspect", label: "Format d'image", help: "4:3, 16:9 ou la forme de la fenêtre.",
    options: [["4:3", "4:3"], ["16:9", "16:9"], ["auto", "Remplir la fenêtre"]] },
  { key: "fullscreen", label: "Plein écran", help: "Jouer dans une fenêtre ou sur tout l'écran.", options: ON_OFF },
  { key: "frame_rate", label: "Images par seconde", help: "Le jeu avance toujours à son propre rythme.",
    options: [["30", "30"], ["60", "60"], ["120", "120"], ["display", "Celle de l'écran"]] },
  { key: "vsync", label: "Synchro verticale", help: "Attendre le rafraîchissement de l'écran : pas de déchirure.", options: ON_OFF },
  { key: "frame_pacing", label: "Régularité des images", help: "Les images montrées à un rythme égal.", options: ON_OFF },
  { key: "vrr", label: "Écran G-Sync / FreeSync", help: "Le rafraîchissement de l'écran suit les images.", options: ON_OFF },
  { key: "texture_pack", label: "Pack de textures", help: "Les textures du pack, ou celles du jeu.", options: ON_OFF },
  { key: "gpu_boost", label: "Carte graphique à fond", help: "Pas d'à-coup à une nouvelle scène. Non : économise la batterie.", options: ON_OFF },
];

const escapeHtml = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
const date = (iso) => (iso ? new Date(iso).toLocaleDateString("fr-FR", { day: "2-digit", month: "2-digit", year: "numeric" }) : "");
const version = (id) => state.versions.find((v) => v.id === id);
const latest = (id) => (releases[id] || [])[0];

function duration(seconds) {
  const minutes = Math.round(seconds / 60);
  if (minutes < 1) return "moins d'une minute";
  if (minutes < 60) return `${minutes} min`;
  const h = Math.floor(minutes / 60), m = minutes % 60;
  return m ? `${h} h ${String(m).padStart(2, "0")}` : `${h} h`;
}

function bytes(n) {
  if (n < 1024) return `${n} o`;
  if (n < 1024 * 1024) return `${Math.round(n / 1024)} Ko`;
  return `${(n / 1024 / 1024).toFixed(1)} Mo`;
}

// Versions compared as semver, as the Rust side does: "v0.8.0-lan.1" is
// "0.8.0-lan.1"; a pre-release part is below the same version without one.
function splitVersion(text) {
  const s = String(text).trim().replace(/^v/i, "").split("+")[0];
  const dash = s.indexOf("-");
  const core = (dash < 0 ? s : s.slice(0, dash)).split(".").map((n) => parseInt(n, 10) || 0);
  while (core.length < 3) core.push(0);
  return [core, dash < 0 ? null : s.slice(dash + 1)];
}
function compareVersions(a, b) {
  const [ac, ap] = splitVersion(a), [bc, bp] = splitVersion(b);
  for (let k = 0; k < Math.max(ac.length, bc.length); k++) {
    const d = (ac[k] || 0) - (bc[k] || 0);
    if (d) return Math.sign(d);
  }
  if (ap === bp) return 0;
  if (ap === null) return 1;
  if (bp === null) return -1;
  const x = ap.split("."), y = bp.split(".");
  for (let k = 0; ; k++) {
    if (k >= x.length && k >= y.length) return 0;
    if (k >= x.length) return -1;
    if (k >= y.length) return 1;
    const xn = /^\d+$/.test(x[k]), yn = /^\d+$/.test(y[k]);
    const d = xn && yn ? Number(x[k]) - Number(y[k]) : xn ? -1 : yn ? 1 : x[k] < y[k] ? -1 : x[k] > y[k] ? 1 : 0;
    if (d) return Math.sign(d);
  }
}
const sameVersion = (a, b) => !!a && !!b && compareVersions(a, b) === 0;

// The release's asset the version installs (its regex), as the Rust side picks it.
function assetOf(v, release) {
  if (!v.installable || !release) return null;
  const pattern = new RegExp(v.fichier, "i");
  return release.assets.find((a) => pattern.test(a.name)) || null;
}

// What the version's card shows as its status, and how.
function statusOf(v) {
  const last = latest(v.id);
  if (!v.active) return { text: "Bientôt", kind: "page" };
  if (v.running) return { text: "En jeu", kind: "ok" };
  // An update only when the release is above the version there (known by its exe).
  if (v.present && last && v.installed && compareVersions(last.tag, v.installed) > 0 && assetOf(v, last))
    return { text: "Mise à jour", kind: "update" };
  if (v.present && last && v.installed && compareVersions(v.installed, last.tag) > 0) return { text: "Plus récente", kind: "ok" };
  if (v.present) return { text: v.ici ? "Présente" : "Installée", kind: "ok" };
  if (!v.installable) return { text: "Page seulement", kind: "page" };
  return { text: v.ici ? "Introuvable" : "Non installée", kind: "page" };
}

// A small Markdown: headings, lists, bold, code, links (as text).
function markdown(text) {
  const lines = escapeHtml(text.replace(/\r\n/g, "\n")).split("\n");
  let html = "", list = false;
  const inline = (s) => s
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1");
  for (const raw of lines) {
    const line = raw.trimEnd();
    const item = line.match(/^\s*[-*]\s+(.*)/);
    if (item) {
      if (!list) { html += "<ul>"; list = true; }
      html += `<li>${inline(item[1])}</li>`;
      continue;
    }
    if (list) { html += "</ul>"; list = false; }
    const heading = line.match(/^#{1,6}\s+(.*)/);
    if (heading) html += `<h4>${inline(heading[1])}</h4>`;
    else if (line.trim()) html += `<p>${inline(line)}</p>`;
  }
  return html + (list ? "</ul>" : "");
}

function renderList() {
  const nav = $("versions");
  nav.innerHTML = "";
  for (const v of state.versions) {
    const s = statusOf(v);
    const button = document.createElement("button");
    button.className = "version" + (v.id === chosen ? " active" : "") + (v.active ? "" : " off");
    button.disabled = !v.active;
    if (!v.active) button.title = "Pas proposée pour l'instant";
    button.innerHTML = `
      <span class="name">${escapeHtml(v.nom)}</span>
      <span class="row">
        <span class="chip platform ${v.plateforme.toLowerCase()}">${escapeHtml(v.plateforme)}</span>
        <span class="chip ${s.kind}">${s.text}</span>
      </span>`;
    button.onclick = () => choose(v.id);
    nav.appendChild(button);
  }
}

// A button that asks a second click before acting.
function confirmClick(button, label, action) {
  if (button.dataset.armed) {
    delete button.dataset.armed;
    action();
    return;
  }
  const before = button.textContent;
  button.dataset.armed = "1";
  button.textContent = label;
  button.classList.add("armed");
  setTimeout(() => {
    if (!button.dataset.armed) return;
    delete button.dataset.armed;
    button.textContent = before;
    button.classList.remove("armed");
  }, 4000);
}

function renderDetail() {
  const v = version(chosen);
  if (!v) return;
  const last = latest(v.id);
  const asset = assetOf(v, last);
  const locked = !!busy || updatingLauncher || v.running;

  $("title").textContent = v.nom;
  $("platform").textContent = v.plateforme;
  $("platform").className = "chip platform " + v.plateforme.toLowerCase();
  $("repo").textContent = "github.com/" + v.depot;
  $("installed-label").textContent = v.ici ? "Dans ce dossier" : "Installée";
  const ahead = v.installed && last && compareVersions(v.installed, last.tag) > 0;
  $("installed").textContent = (v.installed || (v.present ? "oui (version inconnue)" : "non")) + (ahead ? " · plus récente" : "");
  if (last) $("latest").textContent = `${last.tag}${last.prerelease ? " · préversion" : ""} · ${date(last.published)}`;
  else if (errors[v.id]) $("latest").textContent = "GitHub ne répond pas";
  else if (releases[v.id]) $("latest").textContent = "aucune version publiée";
  else $("latest").textContent = "…";

  const stats = state.settings.jeu?.[v.id];
  $("playtime-box").hidden = !stats || !stats.parties;
  if (stats && stats.parties) {
    const when = stats.derniere ? new Date(stats.derniere * 1000).toLocaleDateString("fr-FR", { day: "numeric", month: "long" }) : "";
    $("playtime").textContent = `${duration(stats.secondes)} · ${stats.parties} partie${stats.parties > 1 ? "s" : ""}`;
    $("playtime-box").title = when ? `Dernière partie le ${when}` : "";
  }

  const install = $("install");
  if (!v.installable) {
    install.textContent = "Ouvrir la page ↗";
    install.disabled = false;
    install.onclick = () => invoke("open_page", { id: v.id });
  } else {
    const order = last && v.installed ? compareVersions(last.tag, v.installed) : null;
    const upToDate = order !== null && order <= 0;
    if (!v.present) install.textContent = "Installer";
    else if (!v.installed) install.textContent = last ? `Installer ${last.tag}` : "Installer";
    else install.textContent = upToDate ? "À jour" : "Mettre à jour";
    install.title = order < 0 ? `Ta version (${v.installed}) est plus récente que la dernière publiée (${last.tag}).`
      : v.present && !v.installed ? "La version du jeu présent est inconnue." : "";
    install.disabled = locked || !asset || upToDate;
    install.onclick = () => installTag(last.tag);
  }

  // Going back to the zip kept from before: no internet needed.
  const previous = v.present && v.installable ? v.archived.find((tag) => !sameVersion(tag, v.installed)) : null;
  $("rollback").hidden = !previous;
  if (previous) {
    $("rollback").textContent = `↩ Revenir à ${previous}`;
    $("rollback").title = "Réinstalle la version d'avant, gardée par le lanceur (sans internet). La carte mémoire est sauvegardée d'abord.";
    $("rollback").disabled = locked;
    $("rollback").onclick = () => installTag(previous, true);
  }

  $("play").disabled = locked || !v.present || !v.active;
  $("play").textContent = v.running ? "En jeu…" : "▶ Jouer";
  $("folder").disabled = !v.installable;
  $("logs").disabled = !v.present;

  $("features").innerHTML = v.apports.map((a) => `<li>${escapeHtml(a)}</li>`).join("");
  $("remark").hidden = !v.remarque;
  $("remark").textContent = v.remarque || "";

  renderSettings(v);
  renderSaves(v, locked);
  renderNotes(v, locked);
}

function renderSettings(v) {
  $("settings-panel").hidden = !(v.reglages && v.present);
  if ($("settings-panel").hidden) return;
  $("settings-hint").textContent = v.running ? "Le jeu tourne : il pourrait remplacer ces réglages en quittant." : "Pris en compte au prochain lancement.";
  const box = $("settings");
  box.innerHTML = "";
  for (const s of GAME_SETTINGS) {
    const current = gameSettings[s.key] ?? "";
    const options = [["", "Par défaut du jeu"], ...s.options];
    if (current && !options.some(([value]) => value === current)) options.push([current, current]);
    const row = document.createElement("label");
    row.className = "setting";
    row.title = s.help;
    row.innerHTML = `<span>${escapeHtml(s.label)}</span>
      <select>${options.map(([value, text]) =>
        `<option value="${escapeHtml(value)}"${value === current ? " selected" : ""}>${escapeHtml(text)}</option>`).join("")}</select>`;
    row.querySelector("select").onchange = async (event) => {
      const value = event.target.value || null;
      try {
        await invoke("set_game_setting", { id: v.id, key: s.key, value });
        if (value) gameSettings[s.key] = value; else delete gameSettings[s.key];
      } catch (e) {
        message(String(e));
        event.target.value = current;
      }
    };
    box.appendChild(row);
  }
}

function renderSaves(v, locked) {
  $("saves-panel").hidden = !(v.installable && v.present);
  if ($("saves-panel").hidden) return;
  $("backup-now").disabled = !!busy;
  const box = $("saves");
  if (!saves.length) {
    box.innerHTML = `<p class="empty">Aucune copie pour l'instant.</p>`;
    return;
  }
  box.innerHTML = "";
  const shown = saves.slice(0, 6);
  for (const s of shown) {
    const row = document.createElement("div");
    row.className = "save";
    row.innerHTML = `<span class="save-date">${escapeHtml(s.date)}</span>
      <span class="save-why">${escapeHtml(s.why)}</span>
      <span class="save-size">${bytes(s.bytes)}</span>
      <button class="ghost small">Restaurer</button>`;
    const button = row.querySelector("button");
    button.disabled = locked;
    button.title = "Remet cette copie à la place de la carte mémoire (la carte actuelle est copiée d'abord)";
    button.onclick = () => confirmClick(button, "Confirmer ?", () => restore(v, s));
    box.appendChild(row);
  }
  if (saves.length > shown.length) {
    const more = document.createElement("p");
    more.className = "hint";
    more.textContent = `et ${saves.length - shown.length} plus anciennes (bouton « Dossier »)`;
    box.appendChild(more);
  }
}

function renderNotes(v, locked) {
  const list = releases[v.id];
  if (errors[v.id]) $("notes").innerHTML = `<p class="empty">GitHub ne répond pas : ${escapeHtml(errors[v.id])}</p>`;
  else if (!list) $("notes").innerHTML = `<p class="empty">Chargement…</p>`;
  else if (!list.length) $("notes").innerHTML = `<p class="empty">Aucune version publiée.</p>`;
  else {
    $("notes").innerHTML = list.slice(0, 5).map((r) => {
      const here = sameVersion(r.tag, v.installed);
      // Any other release can be installed: going back (older) as well as forward.
      const older = !!v.installed && compareVersions(r.tag, v.installed) < 0;
      const action = v.installable && !here && assetOf(v, r)
        ? `<button class="ghost small" data-tag="${escapeHtml(r.tag)}"${locked ? " disabled" : ""}>${older ? "Revenir à celle-ci" : "Installer celle-ci"}</button>`
        : "";
      return `
      <article class="release">
        <div class="release-head">
          <strong>${escapeHtml(r.tag)}</strong>
          <time>${date(r.published)}</time>
          ${r.prerelease ? '<span class="chip update">préversion</span>' : ""}
          ${here ? '<span class="chip ok">installée</span>' : ""}
          ${action}
        </div>
        <div class="release-body">${r.notes.trim() ? markdown(r.notes) : '<p class="empty">Pas de notes.</p>'}</div>
      </article>`;
    }).join("");
    for (const button of $("notes").querySelectorAll("[data-tag]"))
      button.onclick = () => confirmClick(button, "Confirmer ?", () => installTag(button.dataset.tag));
  }
}

// Only the discs an offered version plays, and what each image is.
function renderDiscs() {
  for (const kind of ["ps2_us", "xbox_us"]) {
    const d = state.discs[kind];
    const row = $("disc-" + kind).closest(".disc");
    row.hidden = !d;
    if (!d) continue;
    const path = d.path || "";
    $("disc-" + kind).textContent = !path ? "aucun : le jeu le demandera" : d.origin === "jeu" ? `${path} (celui du jeu)` : path;
    $("disc-" + kind).title = path;
    const check = $("check-" + kind);
    check.hidden = !d.check;
    if (d.check) {
      check.textContent = (d.check.good ? "✓ " : "⚠ ") + d.check.text;
      check.className = "check " + (d.check.good ? "good" : "bad");
    }
    row.querySelector("[data-forget]").hidden = d.origin !== "choisi";
  }
}

// The banner offering the launcher's own update.
function renderSelfUpdate() {
  const box = $("self-update");
  box.hidden = !launcherUpdate;
  if (!launcherUpdate) return;
  const go = $("self-update-go");
  go.disabled = !!busy || updatingLauncher;
  if (!updatingLauncher)
    $("self-update-text").textContent = `Lanceur ${launcherUpdate.tag}${launcherUpdate.prerelease ? " (préversion)" : ""} disponible`;
  box.title = launcherUpdate.notes || "";
}

function render() { renderList(); renderDetail(); renderDiscs(); renderSelfUpdate(); }

function message(text, kind) {
  const box = $("message");
  box.hidden = !text;
  box.textContent = text || "";
  box.className = "message" + (kind ? " " + kind : "");
}

// The chosen version's own files: its game settings, its memory card copies.
async function loadVersionFiles() {
  const v = version(chosen);
  if (!v) return;
  const [settings, list] = await Promise.all([
    v.reglages && v.present ? invoke("game_settings", { id: v.id }).catch(() => ({})) : {},
    v.installable && v.present ? invoke("saves", { id: v.id }).catch(() => []) : [],
  ]);
  if (v.id !== chosen) return;
  gameSettings = settings;
  saves = list;
  renderDetail();
}

async function refreshState() {
  const fresh = await invoke("state");
  for (const v of fresh.versions) {
    const old = version(v.id);
    if (old) Object.assign(old, v);
  }
  state.settings = fresh.settings;
  state.discs = fresh.discs;
}

function choose(id) {
  chosen = id;
  message("");
  gameSettings = {};
  saves = [];
  invoke("remember", { id });
  render();
  loadVersionFiles();
}

// Installs a release: from GitHub, or from the zip kept (`archived`).
async function installTag(tag, archived = false) {
  const v = version(chosen);
  busy = v.id;
  message("");
  $("progress").hidden = archived;
  setProgress(0);
  render();
  try {
    v.installed = await invoke(archived ? "install_archived" : "install", { id: v.id, tag });
    message(`${v.nom} ${tag} est installée.`, "info");
  } catch (e) {
    message("L'installation a échoué : " + e);
  } finally {
    busy = null;
    $("progress").hidden = true;
    await refreshState().catch(() => {});
    render();
    loadVersionFiles();
  }
}

async function restore(v, s) {
  try {
    await invoke("restore_save", { id: v.id, name: s.name });
    message(`Carte mémoire remise comme le ${s.date}.`, "info");
  } catch (e) {
    message(String(e));
  }
  loadVersionFiles();
}

function setProgress(percent) {
  $("bar").style.width = percent + "%";
  $("percent").textContent = percent + " %";
}

$("play").onclick = async () => {
  const v = version(chosen);
  try {
    await invoke("play", { id: v.id });
    v.running = true;
    message("Le jeu démarre. Après une mise à jour, le premier lancement peut être long (préparation des shaders).", "info");
  } catch (e) {
    message(String(e));
  }
  render();
};
$("folder").onclick = () => invoke("open_folder", { id: chosen }).catch((e) => message(String(e)));
$("page").onclick = () => invoke("open_page", { id: chosen });
$("logs").onclick = async () => {
  $("logs").disabled = true;
  try {
    const zip = await invoke("export_logs", { id: chosen });
    message(`Journaux prêts : ${zip}. Envoie ce fichier (Discord, mail…) à qui t'aide ; rien n'est parti tout seul.`, "info");
  } catch (e) {
    message(String(e));
  }
  $("logs").disabled = false;
};
$("backup-now").onclick = async () => {
  try {
    await invoke("backup_now", { id: chosen });
    message("Carte mémoire sauvegardée.", "info");
  } catch (e) {
    message(String(e));
  }
  loadVersionFiles();
};
$("saves-folder").onclick = () => invoke("open_saves", { id: chosen }).catch((e) => message(String(e)));

for (const button of document.querySelectorAll("[data-disc]"))
  button.onclick = async () => {
    const kind = button.dataset.disc;
    const path = await invoke("choose_disc", { kind }).catch((e) => { message(String(e)); return null; });
    if (!path) return;
    state.settings.discs[kind] = path;
    state.discs = await invoke("discs");
    renderDiscs();
    const d = state.discs[kind];
    if (d?.check && !d.check.good) message(`Ce disque ne convient pas : ${d.check.text}.`);
  };
for (const button of document.querySelectorAll("[data-forget]"))
  button.onclick = async () => {
    const kind = button.dataset.forget;
    await invoke("forget_disc", { kind }).catch((e) => message(String(e)));
    delete state.settings.discs[kind];
    state.discs = await invoke("discs");
    renderDiscs();
  };

$("self-update-go").onclick = async () => {
  updatingLauncher = true;
  $("self-update-text").textContent = "Téléchargement du lanceur… 0 %";
  render();
  try {
    await invoke("update_launcher");   // the launcher restarts itself
    $("self-update-text").textContent = "Redémarrage…";
  } catch (e) {
    updatingLauncher = false;
    message("La mise à jour du lanceur a échoué : " + e);
    render();
  }
};

listen("progression", (event) => {
  const { id, percent } = event.payload;
  if (id === "lanceur") $("self-update-text").textContent = `Téléchargement du lanceur… ${percent} %`;
  else if (id === busy) setProgress(percent);
});

// A game started from here has ended: its time, and a word if it stopped badly.
listen("partie", (event) => {
  const { id, seconds, code, stats } = event.payload;
  const v = version(id);
  if (!v) return;
  v.running = false;
  state.settings.jeu = state.settings.jeu || {};
  state.settings.jeu[id] = stats;
  if (code !== null && code !== 0)
    message(`Le jeu s'est arrêté avec le code ${code} après ${duration(seconds)}. S'il a planté, le bouton « Journaux » prépare un zip à envoyer.`);
  else
    message(`Partie terminée : ${duration(seconds)}.`, "info");
  render();
  if (id === chosen) loadVersionFiles();
  // The game may have changed its disc or settings.
  invoke("discs").then((d) => { state.discs = d; renderDiscs(); }).catch(() => {});
});

// Back to the launcher: what changed beside it (a disc, the game's settings, a save).
window.addEventListener("focus", () => {
  if (!state || busy) return;
  invoke("discs").then((d) => { state.discs = d; renderDiscs(); }).catch(() => {});
  loadVersionFiles();
});

async function start() {
  state = await invoke("state");
  $("launcher-version").textContent = "· v" + state.launcher_version;
  // The launcher's own update: quiet when there is none or GitHub does not answer.
  invoke("launcher_update").then((u) => { launcherUpdate = u; render(); }).catch(() => {});
  const offered = state.versions.filter((v) => v.active);
  chosen = offered.some((v) => v.id === state.settings.selected) ? state.settings.selected : offered[0]?.id;
  render();
  loadVersionFiles();
  // Each offered version's releases; the page fills in as GitHub answers.
  await Promise.all(offered.map(async (v) => {
    try { releases[v.id] = await invoke("releases", { id: v.id }); }
    catch (e) { errors[v.id] = String(e); }
    render();
  }));
}

start();
