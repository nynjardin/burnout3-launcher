# Burnout 3 Launcher — orientation

Goal: one **portable** Windows launcher (a single exe in a folder, nothing installed) to pick a
version of *Burnout 3: Takedown* for PC, see what each brings, install it and keep it up to date
by downloading **only that version's release** — never the disc images, saves or texture packs.
Project opened 2026-10-30.

Versions it knows (details and current facts: [`docs/versions.md`](docs/versions.md)):
- **ours**: the PS2 port with LAN, built in `I:\Projets\Recomp\Burnout 3\burnout3-pc` (branch
  `lan`), to be published as Releases of the user's fork `nynjardin/burnout3-pc`;
- **upstream**: `danielgomesvieira2000/burnout3-pc`;
- **takedawn**: `xenophobentx/takedawn` (the Xbox version);
- **Burn3**: `UwUCammie/Burn3` (PS2).

## How the user wants you to work

- **Do everything you can without asking**; keep `docs/journal.md` up to date as you go.
- When stuck, switch tasks; go as far as possible on your own.
- **Portability above all**: no installer, no registry, no admin rights; the launcher's
  settings and every version live beside the launcher's exe.
- The user reads French: replies, the journal and everything under `docs/` are in French.
  Source comments, UI strings' code and this file stay in English; **the UI itself is French**.

## Context carried over from the game project (2026-10-30)

- The user runs LAN evenings on several PCs and used LanCommander, which re-downloads the whole
  game for a changed exe: that is why this launcher exists.
- Our game builds are published as prereleases of `nynjardin/burnout3-pc` (first: `v0.8.0-lan.1`,
  2026-11-13; zip with its files at the root, no enclosing folder).
- Upstream's first launch after an update compiles Vulkan pipelines for minutes with no sign
  of it (black screen, stalls): players think it crashed. See `docs/contrat-jeu.md`.
- Windows: never kill every `node.exe` (it kills Claude Code); stop only the launcher's or the
  game's own processes (`Lanceur.exe`, `burnout3.exe`).

## Read this before searching

[`docs/INDEX.md`](docs/INDEX.md) is the entry point; read its row, open only the file you need.
Resuming: the « REPRENDRE ICI » section of [`docs/etat-du-projet.md`](docs/etat-du-projet.md).

## Hard constraints

- **Never publish anything** (push, release, repo creation) without the user saying so.
- **Never redistribute other projects' builds**: the launcher downloads them from their own
  GitHub Releases. Never ship or download game data (ISOs, BIOS, extracted files).
- The game side (what the launcher expects from our builds) is a contract:
  [`docs/contrat-jeu.md`](docs/contrat-jeu.md). Changing it means changing the game repo too.
- The game project's own rules apply to anything done there (`I:\Projets\Recomp\Burnout 3\CLAUDE.md`).

## Build

**Tauri 2**: Rust in `src-tauri/` (GitHub, downloads, zips, starting a version), the page in
`ui/` (plain HTML/CSS/JS, no bundler, `window.__TAURI__` via `withGlobalTauri`). No Tauri CLI
needed. Rust is installed but not on the shell's PATH: use `%USERPROFILE%\.cargoin\cargo.exe`.

```
cd src-tauri
cargo build --release
```

Output: `src-tauri/target/release/Lanceur.exe` (one file, the page embedded). Try it in a
scratch folder (`essai/`, ignored by git), never in the game project's folder. Windows needs the
WebView2 runtime (present on Windows 11 and updated Windows 10).

## Keeping the docs alive

Same rules as the game project: a spec states the current truth; the narrative goes to the
journal (a `suite N` block at the top); mark confidence (CONFIRMÉ / HYPOTHÈSE / INCONNU);
one subject per file, plus its row in `docs/INDEX.md`.
