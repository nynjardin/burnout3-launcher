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

## Le dossier portable — CONFIRMÉ (choix de l'utilisateur : rien à installer)

```
<le dossier du jeu, partagé entre amis>/
  burnout3.exe, ...     NOTRE version : le jeu lui-même (champ « ici » du catalogue)
  Lanceur.exe
  lanceur.json          réglages : chemins des disques (facultatifs), dernière version choisie
  .lanceur-version      étiquette de notre version installée par le lanceur
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
    (`state`, `releases`, `install`, `play`, `choose_disc`, `remember`, `open_folder`,
    `open_page`, `launcher_update`, `update_launcher`) ; l'avancement d'une installation arrive en événements `progression`.
  - **Portable** : la fenêtre est créée dans le code avec `data_directory` =
    `.lanceur-webview/` à côté de l'exe (sinon WebView2 écrirait dans `%LOCALAPPDATA%`).
  - Windows : il faut le moteur WebView2 (présent sur Windows 11 et les Windows 10 à jour).
    Linux : WebKitGTK du système (ou un AppImage). Mac : « Ouvrir quand même » sans signature.
- **GitHub sans compte** : `GET /repos/<dépôt>/releases` (60 requêtes/heure par connexion :
  une par version à l'ouverture suffit). En-tête `User-Agent` obligatoire.
- **Catalogue** : un `catalogue.json` intégré à l'exe ; un `catalogue.json` posé à côté le
  remplace (pour corriger une fiche sans republier). Plus tard : le relire depuis le dépôt
  du lanceur.
- **Mise à jour sûre** : téléchargement dans un fichier temporaire, extraction fichier par
  fichier, étiquette installée écrite à la fin (`.lanceur-version`). Une mise à jour ratée
  laisse l'étiquette d'avant : le lanceur la reproposera.

## Le lanceur se met à jour lui-même — CONFIRMÉ (essai local 2026-11-13)

- Au démarrage, il lit les versions publiées de **son propre dépôt** (champ `lanceur` du
  catalogue : `depot`, par défaut `nynjardin/burnout3-launcher` — **dépôt créé le 2026-11-13, vide**,
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

- Barre de progression des shaders après une mise à jour (voir [contrat-jeu.md](contrat-jeu.md)).
- Takedawn quand son mode portable existera (annoncé en 0.2.2).
