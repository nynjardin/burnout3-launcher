// The launcher's page: the versions on the left, the chosen one on the right.
// It talks to the Rust side through window.__TAURI__ (withGlobalTauri).
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const $ = (id) => document.getElementById(id);
let state = null;              // { versions, settings, folder } from the Rust side
let chosen = null;             // the version id shown
const releases = {};           // id -> [release] once GitHub answered
const errors = {};             // id -> why it did not
let busy = null;               // the version id being installed

const escapeHtml = (s) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
const date = (iso) => (iso ? new Date(iso).toLocaleDateString("fr-FR", { day: "2-digit", month: "2-digit", year: "numeric" }) : "");
const version = (id) => state.versions.find((v) => v.id === id);
const latest = (id) => (releases[id] || [])[0];

// The release's asset the version installs (its regex), as the Rust side picks it.
function assetOf(v, release) {
  if (!v.installable || !release) return null;
  const pattern = new RegExp(v.fichier, "i");
  return release.assets.find((a) => pattern.test(a.name)) || null;
}

// What the version's card shows as its status, and how.
function statusOf(v) {
  const last = latest(v.id);
  if (!v.installable) return { text: "Page seulement", kind: "page" };
  if (v.installed && last && v.installed !== last.tag && assetOf(v, last)) return { text: "Mise à jour", kind: "update" };
  if (v.installed) return { text: "Installée", kind: "ok" };
  return { text: "Non installée", kind: "page" };
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
    button.className = "version" + (v.id === chosen ? " active" : "");
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

function renderDetail() {
  const v = version(chosen);
  if (!v) return;
  const last = latest(v.id);
  const asset = assetOf(v, last);

  $("title").textContent = v.nom;
  $("platform").textContent = v.plateforme;
  $("platform").className = "chip platform " + v.plateforme.toLowerCase();
  $("repo").textContent = "github.com/" + v.depot;
  $("installed").textContent = v.installed || "non";
  if (last) $("latest").textContent = `${last.tag}${last.prerelease ? " · préversion" : ""} · ${date(last.published)}`;
  else if (errors[v.id]) $("latest").textContent = "GitHub ne répond pas";
  else if (releases[v.id]) $("latest").textContent = "aucune version publiée";
  else $("latest").textContent = "…";

  const install = $("install");
  if (!v.installable) {
    install.textContent = "Ouvrir la page ↗";
    install.disabled = false;
    install.onclick = () => invoke("open_page", { id: v.id });
  } else {
    install.textContent = !v.installed ? "Installer" : last && v.installed === last.tag ? "À jour" : "Mettre à jour";
    install.disabled = !!busy || !asset || (last && v.installed === last.tag);
    install.onclick = () => installChosen();
  }
  $("play").disabled = !!busy || !v.installed;
  $("folder").disabled = !v.installable;

  $("features").innerHTML = v.apports.map((a) => `<li>${escapeHtml(a)}</li>`).join("");
  $("remark").hidden = !v.remarque;
  $("remark").textContent = v.remarque || "";

  const list = releases[v.id];
  if (errors[v.id]) $("notes").innerHTML = `<p class="empty">GitHub ne répond pas : ${escapeHtml(errors[v.id])}</p>`;
  else if (!list) $("notes").innerHTML = `<p class="empty">Chargement…</p>`;
  else if (!list.length) $("notes").innerHTML = `<p class="empty">Aucune version publiée.</p>`;
  else
    $("notes").innerHTML = list.slice(0, 3).map((r) => `
      <article class="release">
        <div class="release-head">
          <strong>${escapeHtml(r.tag)}</strong>
          <time>${date(r.published)}</time>
          ${r.prerelease ? '<span class="chip update">préversion</span>' : ""}
        </div>
        <div class="release-body">${r.notes.trim() ? markdown(r.notes) : '<p class="empty">Pas de notes.</p>'}</div>
      </article>`).join("");
}

function renderDiscs() {
  for (const kind of ["ps2_us", "xbox_us"]) {
    const path = state.settings.discs[kind];
    $("disc-" + kind).textContent = path || "aucun";
    $("disc-" + kind).title = path || "";
  }
}

function render() { renderList(); renderDetail(); renderDiscs(); }

function message(text, kind) {
  const box = $("message");
  box.hidden = !text;
  box.textContent = text || "";
  box.className = "message" + (kind ? " " + kind : "");
}

function choose(id) {
  chosen = id;
  message("");
  invoke("remember", { id });
  render();
}

async function installChosen() {
  const v = version(chosen);
  const last = latest(v.id);
  if (!last) return;
  busy = v.id;
  message("");
  $("progress").hidden = false;
  setProgress(0);
  render();
  try {
    v.installed = await invoke("install", { id: v.id, tag: last.tag });
    message(`${v.nom} ${last.tag} est installée.`, "info");
  } catch (e) {
    message("L'installation a échoué : " + e);
  } finally {
    busy = null;
    $("progress").hidden = true;
    render();
  }
}

function setProgress(percent) {
  $("bar").style.width = percent + "%";
  $("percent").textContent = percent + " %";
}

$("play").onclick = async () => {
  try {
    await invoke("play", { id: chosen });
    message("Le jeu démarre. Après une mise à jour, le premier lancement peut être long (préparation des shaders).", "info");
  } catch (e) {
    message(String(e));
  }
};
$("folder").onclick = () => invoke("open_folder", { id: chosen }).catch((e) => message(String(e)));
$("page").onclick = () => invoke("open_page", { id: chosen });
for (const button of document.querySelectorAll("[data-disc]"))
  button.onclick = async () => {
    const kind = button.dataset.disc;
    const path = await invoke("choose_disc", { kind }).catch((e) => { message(String(e)); return null; });
    if (path) { state.settings.discs[kind] = path; renderDiscs(); }
  };

listen("progression", (event) => { if (event.payload.id === busy) setProgress(event.payload.percent); });

async function start() {
  state = await invoke("state");
  chosen = state.settings.selected && version(state.settings.selected) ? state.settings.selected : state.versions[0]?.id;
  render();
  // Each version's releases; the page fills in as GitHub answers.
  await Promise.all(state.versions.map(async (v) => {
    try { releases[v.id] = await invoke("releases", { id: v.id }); }
    catch (e) { errors[v.id] = String(e); }
    render();
  }));
}

start();
