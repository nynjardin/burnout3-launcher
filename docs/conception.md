# Conception du lanceur

## Ce qu'il fait

- Liste les versions du jeu (le **catalogue**) avec, pour chacune, une **fiche** : plateforme
  d'origine, disque à fournir, ce qu'elle apporte.
- Pour chacune, demande à GitHub ses versions publiées : affiche la version installée, la
  dernière, ses notes ; propose **Installer** / **Mettre à jour** quand un fichier joint
  portable existe, sinon **Ouvrir la page du projet**.
- **Jouer** : si un disque est choisi dans le lanceur, écrit son chemin dans les réglages de la
  version (`disc=`) ; sinon le jeu prend le sien. Lance son exe.
- Télécharge **seulement** le zip de la version : ni disque, ni sauvegardes, ni pack de textures.
- **Version présente** : lue dans l'exe du jeu (`FileVersion`), sinon dans `.lanceur-version`.
  « Mettre à jour » seulement si la dernière publiée est **plus haute** (semver, `v` ignoré) ;
  plus basse : « Plus récente », rien proposé ; inconnue : « Installer vX ».
- **Revenir en arrière** : chaque version publiée des notes a un bouton « Installer celle-ci »
  / « Revenir à celle-ci » ; les deux derniers zips installés sont gardés, et « ↩ Revenir à vX »
  réinstalle l'avant-dernier **sans internet**. Les fichiers ajoutés par la version plus récente
  restent (seuls ceux du zip sont écrits).
- **Carte mémoire** (`memcard/` de la version) : copiée avant chaque installation et avant
  chaque restauration, ou à la main ; « Restaurer » (deux clics) la remet. Les 15 copies les plus
  récentes sont gardées. Refusé pendant que le jeu tourne.
- **Réglages du jeu** (versions marquées `reglages` dans le catalogue : moteur du portage PS2) :
  les lignes de PC SETTINGS du jeu (résolution, format, plein écran, images/s, synchro,
  régularité, VRR, pack de textures, GPU à fond) écrites dans son `pc_settings.ini` ; « Par
  défaut du jeu » enlève la ligne. Les autres lignes du fichier restent telles quelles.
- **Disque vérifié** : pour chaque sorte de disque d'une version proposée, le lanceur montre
  l'image utilisée (celle choisie dans le lanceur, sinon celle du jeu : `disc=` de son
  `pc_settings.ini` ou une image posée à côté) et ce qu'elle est : PS2 US = `BOOT2` de
  `SYSTEM.CNF` égal à `SLUS_210.50` (la vérification du jeu lui-même) ; Xbox = volume
  `MICROSOFT*XBOX*MEDIA` (titre non vérifié). `.iso`, `.bin` brut (2352 octets, mode 1/2), `.cue`.
- **Temps de jeu** : le lanceur attend la fin du jeu qu'il a lancé (le jeu attend lui-même sa
  version SSE4) et additionne la durée, le nombre de parties et la date de la dernière dans
  `lanceur.json`. Une fin avec un code autre que 0 affiche un message qui renvoie à « Journaux ».
- **Journaux** : un zip (`burnout3*.log`, `pc_settings.ini`, `infos.txt` : versions, Windows,
  processeur, mémoire, cartes graphiques et pilotes) dans `journaux/`, montré dans
  l'Explorateur. **Rien n'est envoyé** : le joueur passe le fichier lui-même (Discord…).

## Le dossier portable — CONFIRMÉ (choix de l'utilisateur : rien à installer)

```
<le dossier du jeu, partagé entre amis>/
  burnout3.exe, ...     NOTRE version : le jeu lui-même (champ « ici » du catalogue)
  Lanceur.exe
  lanceur.json          réglages : chemins des disques (facultatifs), dernière version choisie,
                        temps de jeu par version
  .lanceur-version      étiquette de notre version installée par le lanceur
  .lanceur-archives/<id>/<étiquette>.zip   les deux derniers zips installés (retour sans internet)
  sauvegardes/<id>/<date>_<motif>/         copies de la carte mémoire
  journaux/             zips de journaux préparés par le bouton « Journaux »
  .lanceur-webview/     données du moteur web de la fenêtre
  catalogue.json        (facultatif) remplace le catalogue intégré à l'exe
  versions/
    amont/  ...         les autres versions, une par dossier (grisées pour l'instant)
```

CONFIRMÉ (choix de l'utilisateur, 2026-11-13) : le lanceur sert surtout à jouer entre amis ;
on le pose **dans le dossier du jeu** et il y détecte `burnout3.exe`. Les autres versions sont
**grisées** (champ `active: false`) : affichées, ni installables ni lançables. Une mise à jour
de notre version écrit les fichiers du zip dans ce dossier, jamais `Lanceur*` / `.lanceur*`.

Pas de registre, pas de droits d'administrateur, rien hors de ce dossier.

## Choix techniques

- **Tauri 2 (Rust + HTML/CSS/JS)** — choix de l'utilisateur (2026-10-30), à la place d'un
  premier squelette C# WinForms : une interface soignée en HTML, un seul exe de quelques Mo,
  le même code pour Linux et Mac plus tard.
  - Rust (`src-tauri/`) : `reqwest` (rustls) pour GitHub et les téléchargements, `zip`,
    `rfd` (choix d'un fichier), `open` (dossiers, pages).
  - Page (`ui/`) : HTML/CSS/JS sans outil de construction ; appelle les commandes Rust
    (`state`, `releases`, `install`, `install_archived`, `play`, `choose_disc`, `forget_disc`,
    `discs`, `remember`, `open_folder`, `open_page`, `launcher_update`, `update_launcher`,
    `game_settings`, `set_game_setting`, `saves`, `backup_now`, `restore_save`, `open_saves`,
    `export_logs`) ; l'avancement d'une installation arrive en événements `progression`, la
    fin d'une partie en événement `partie`.
  - Modules : `winfo.rs` (API Windows : version d'un exe, infos du PC par le registre, ouvrir
    un dossier ou une page par `ShellExecuteW`), `disc.rs` (identifier une image), `gameini.rs` (`pc_settings.ini`), `saves.rs`
    (carte mémoire), `journaux.rs` (zip des journaux), `installation.rs` (zips gardés).
  - **Portable** : la fenêtre est créée dans le code avec `data_directory` =
    `.lanceur-webview/` à côté de l'exe (sinon WebView2 écrirait dans `%LOCALAPPDATA%`).
  - Windows : il faut le moteur WebView2 (présent sur Windows 11 et les Windows 10 à jour).
    Linux : WebKitGTK du système (ou un AppImage). Mac : « Ouvrir quand même » sans signature.
- **GitHub sans compte** : `GET /repos/<dépôt>/releases` (60 requêtes/heure par connexion :
  une par version à l'ouverture suffit). En-tête `User-Agent` obligatoire.
- **Catalogue** : un `catalogue.json` intégré à l'exe ; un `catalogue.json` posé à côté le
  remplace (pour corriger une fiche sans republier). Plus tard : le relire depuis le dépôt
  du lanceur.
- **Mise à jour sûre** : téléchargement dans `.lanceur-archives/<id>/<étiquette>.part`, taille
  et SHA-256 (`digest` de GitHub) vérifiés, puis renommé en `.zip` (gardé) ; extraction fichier
  par fichier, étiquette installée écrite à la fin (`.lanceur-version`). Une mise à jour ratée
  laisse l'étiquette d'avant : le lanceur la reproposera.

## Antivirus — HYPOTHÈSE

- 2026-11-13 : un antivirus d'une autre machine a supprimé le lanceur (lequel, et quel nom de
  menace : INCONNU). Defender de la machine de développement ne trouve rien. **La version sans
  PowerShell passe sur cette autre machine** (CONFIRMÉ par l'utilisateur, publiée en `v0.2.0`).
- Un exe **non signé**, peu téléchargé, qui télécharge des fichiers, se remplace lui-même et
  lance d'autres programmes ressemble à un « dropper » pour les détections heuristiques.
- Fait : **plus aucun appel à PowerShell** (la bibliothèque `open` en lançait un à chaque
  « Dossier » / « Page » ; « Journaux » aussi) — tout passe par l'API Windows ; copyright et
  éditeur dans la ressource de version de l'exe.
- Restent possibles : signaler un faux positif à l'éditeur de l'antivirus ; signer l'exe
  (certificat de signature de code : payant, ou SignPath Foundation, gratuit pour un projet
  libre).

## Le lanceur se met à jour lui-même — CONFIRMÉ (essai local 2026-11-13)

- Au démarrage, il lit les versions publiées de **son propre dépôt** (champ `lanceur` du
  catalogue : `depot`, par défaut `nynjardin/burnout3-launcher` — dépôt créé le 2026-11-13, `v0.1.0` publiée,
  décision de l'utilisateur ; `fichier`, par défaut `^Lanceur\.exe$`). Il retient la plus
  haute étiquette **au-dessus de sa propre version** (`version` de `Cargo.toml`, comparaison
  semver : `v0.2.0-beta.1` < `v0.2.0`), préversions comprises, qui porte ce fichier.
- Si elle existe : un bandeau en haut à droite, « Lanceur vX disponible » + « Mettre à jour le
  lanceur ». Rien d'automatique sans clic. Dépôt absent ou GitHub muet : rien ne s'affiche.
- Le clic : téléchargement dans `Lanceur.exe.nouveau`, vérification de la taille et de
  l'empreinte SHA-256 que donne GitHub (`digest` du fichier joint), puis `Lanceur.exe` (en
  marche) renommé `Lanceur.exe.ancien`, le nouveau mis à sa place et lancé, l'ancien quitte.
  Le nouveau supprime `.ancien` / `.nouveau` à son démarrage. Un échec remet l'ancien en place.
- Ne marche que si le dossier est inscriptible (c'est le cas d'un dossier portable).
- **Publier une version du lanceur** : monter `version` dans `src-tauri/Cargo.toml` (et
  `tauri.conf.json`), compiler, publier sur le dépôt du lanceur une version d'étiquette
  `v<cette version>` avec le fichier joint **`Lanceur.exe`** (l'exe seul, pas de zip).
  Une étiquette sans rapport avec la version de `Cargo.toml` serait proposée en boucle.
- Pour essayer sans rien publier : la variable `LANCEUR_GITHUB_API` remplace
  `https://api.github.com` (un petit serveur local qui imite `/repos/<dépôt>/releases`).

## Plus tard

- Barre de progression des shaders : l'utilisateur la voit déjà dans le jeu au lancement
  (2026-11-13) ; pas besoin côté lanceur pour l'instant.
- De côté (décision de l'utilisateur, 2026-11-13) : tout ce qui touche au réseau entre PC (fait
  doublon avec LanCommander) et l'habillage (images de fond, mode compact).
- Takedawn quand son mode portable existera (annoncé en 0.2.2).
