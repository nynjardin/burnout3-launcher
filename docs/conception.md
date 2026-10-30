# Conception du lanceur

## Ce qu'il fait

- Liste les versions du jeu (le **catalogue**) avec, pour chacune, une **fiche** : plateforme
  d'origine, disque à fournir, ce qu'elle apporte.
- Pour chacune, demande à GitHub ses versions publiées : affiche la version installée, la
  dernière, ses notes ; propose **Installer** / **Mettre à jour** quand un fichier joint
  portable existe, sinon **Ouvrir la page du projet**.
- **Jouer** : écrit le chemin du disque dans les réglages de la version, lance son exe.
- Télécharge **seulement** le zip de la version : ni disque, ni sauvegardes, ni pack de textures.

## Le dossier portable — CONFIRMÉ (choix de l'utilisateur : rien à installer)

```
<n'importe quel dossier>/
  Lanceur.exe
  lanceur.json          réglages : chemins des disques, dernière version choisie
  .lanceur-webview/     données du moteur web de la fenêtre
  catalogue.json        (facultatif) remplace le catalogue intégré à l'exe
  versions/
    lan/  amont/  ...   une version par dossier, avec ses sauvegardes et caches
```

Pas de registre, pas de droits d'administrateur, rien hors de ce dossier.

## Choix techniques

- **Tauri 2 (Rust + HTML/CSS/JS)** — choix de l'utilisateur (2026-10-30), à la place d'un
  premier squelette C# WinForms : une interface soignée en HTML, un seul exe de quelques Mo,
  le même code pour Linux et Mac plus tard.
  - Rust (`src-tauri/`) : `reqwest` (rustls) pour GitHub et les téléchargements, `zip`,
    `rfd` (choix d'un fichier), `open` (dossiers, pages).
  - Page (`ui/`) : HTML/CSS/JS sans outil de construction ; appelle les commandes Rust
    (`state`, `releases`, `install`, `play`, `choose_disc`, `remember`, `open_folder`,
    `open_page`) ; l'avancement d'une installation arrive en événements `progression`.
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

## Plus tard

- Le lanceur se met à jour lui-même (même mécanisme, sur son propre dépôt).
- Barre de progression des shaders après une mise à jour (voir [contrat-jeu.md](contrat-jeu.md)).
- Takedawn quand son mode portable existera (annoncé en 0.2.2).
