# Journal des sessions

Le récit chronologique, **le plus récent en haut**. Pas une référence : la vérité actuelle est
dans les autres fichiers de `docs/`.

---

suite 5 (le lanceur se met à jour lui-même — 2026-11-13)

- Demande de l'utilisateur : un « autoupdater » pour le lanceur.
- Choix (sans dépôt du lanceur pour l'instant) : champ `lanceur` du catalogue (`depot`, par
  défaut `nynjardin/burnout3-launcher` ; `fichier`, `^Lanceur\.exe$`) ; la plus haute étiquette
  au-dessus de la version de `Cargo.toml` (semver) avec ce fichier ; **bandeau + un clic**
  plutôt qu'une mise à jour silencieuse. Dépôt absent (404) : rien n'est affiché.
- Rust : `self_update.rs` (recherche, comparaison semver + test, téléchargement vérifié taille
  + SHA-256 `digest` de GitHub, échange `Lanceur.exe` ↔ `.ancien`, relance, ménage au
  démarrage) ; `github.rs` lit `digest` et accepte `LANCEUR_GITHUB_API` pour les essais ;
  commandes `launcher_update`, `update_launcher` ; la version du lanceur affichée sous son nom.
- Essai réel dans `essai/maj/` avec un faux serveur local (v0.1.1 = le même exe) : bandeau
  affiché, clic simulé (port de débogage WebView2) → en moins de 2 s l'ancien processus quitte,
  le nouvel exe démarre, `.ancien` supprimé. Compilé sans avertissement.
- Nom du dépôt choisi par l'utilisateur : **`burnout3-launcher`** (catalogue, défaut Rust, doc) ;
  dépôt public créé à sa demande (public obligatoire : le lanceur lit GitHub sans compte).

suite 4 (notre première version publiée, vue par le lanceur — 2026-11-13)

- L'utilisateur a publié la préversion `v0.8.0-lan.1` sur `nynjardin/burnout3-pc` ; son zip
  `Burnout3-PC-0.8.0-lan.1-windows-x64.zip` correspond au motif du catalogue, sans changement.
- Le zip n'a pas de dossier englobant (fichiers à la racine) : `extract` le gère déjà.
- Essai réel : `Lanceur.exe` seul dans `essai/lan/`, clic « Installer » simulé par le port de
  débogage de WebView2 (`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9333`,
  `Runtime.evaluate`). Avant : « Introuvable », dernière `v0.8.0-lan.1 · préversion`. Après :
  les six fichiers posés à côté du lanceur, `.lanceur-version` = `v0.8.0-lan.1`, bouton « À jour ».
- Remarque du catalogue mise à jour (plus « aucune version publiée »).

suite 3 (notre version dans le dossier du lanceur, les autres grisées — 2026-11-13)

- Demande de l'utilisateur : le lanceur sert surtout aux soirées entre amis ; **notre version =
  le `burnout3.exe` posé à côté du lanceur**, plus besoin d'installer l'amont en plus.
  **Les autres versions grisées** pour l'instant.
- Catalogue : deux champs. `ici` (la version vit dans le dossier du lanceur, pas dans
  `versions/<id>/`) mis sur `lan` ; `active: false` sur amont, takedawn, Burn3.
- Rust : `present` (l'exe est là, avec ou sans `.lanceur-version`) ; « Jouer » marche dès que
  l'exe est présent ; le disque choisi dans le lanceur devient **facultatif** (sans lui, le jeu
  prend celui de son `pc_settings.ini` ou l'image posée à côté) ; une version inactive ne
  s'installe ni ne se lance ; l'extraction n'écrit jamais `Lanceur*` / `.lanceur*` (le lanceur
  partage maintenant le dossier du jeu).
- Page : cartes grisées non cliquables (« Bientôt »), pas de requête GitHub pour elles ; « Dans
  ce dossier : oui (version inconnue) » quand l'exe est là sans étiquette ; la ligne du disque
  Xbox cachée (aucune version Xbox proposée).
- Compilé sans avertissement ; `node --check` sur `app.js`. **Pas encore vu à l'écran.**
- Notre version renommée **« LAN edition »** (nom affiché, catalogue).

suite 2 (bascule en Tauri — 2026-10-30)

- L'utilisateur trouve WinForms laid et Windows seulement : bascule vers **Tauri 2** (Rust +
  HTML/JS). Rust 1.99.1 déjà installé (hors PATH du shell), WebView2 154 présent.
- Code C# supprimé (il reste dans l'historique git, `8dbd6c4`) ; `catalogue.json` repris tel quel.
- Rust : catalogue, réglages, GitHub, installation (même logique : dossier de tête enlevé,
  fichiers du zip seulement, étiquette en dernier), commandes. Page : thème sombre, versions à
  gauche, fiche à droite, notes de version (petit Markdown), disques en bas.
- Première compilation 5 min (dépendances), ensuite ≈ 2,5 min ; exe 9,3 Mo, sans avertissement.
- Vérifié par captures de la fenêtre : affichage, versions GitHub lues ; installation de l'amont
  par un clic simulé sur « Installer » (étiquette réécrite, message, bouton « À jour ») ;
  `.lanceur-webview/` créé à côté de l'exe, rien dans `%LOCALAPPDATA%` / `%APPDATA%`.
- Retouches : boutons secondaires sur une ligne, « À jour » désactivé en gris.

suite 1 (création — 2026-10-30)

- Projet ouvert à la demande de l'utilisateur : lanceur portable, plusieurs versions du jeu,
  mises à jour sans tout retélécharger (LanCommander retéléchargeait tout le jeu).
- Relevé des dépôts (API GitHub) : l'amont publie des préversions (zip 42 Mo) ; le fork rien ;
  takedawn un installateur Windows ; Burn3 des versions sans fichier. Détails : `versions.md`.
- Contrat avec le jeu relevé dans son code (`pc_settings.ini`, `disc=`, zip de `package.py`).
- Choix : C# .NET Framework 4.8, WinForms, sans NuGet.
- Squelette écrit (`src/Lanceur/`) : catalogue intégré (4 versions), réglages `lanceur.json`,
  versions GitHub (préversions comprises), installation d'un zip (dossier de tête enlevé, seuls
  les fichiers du zip écrits, étiquette `.lanceur-version` en dernier), Jouer (`disc=` dans
  `pc_settings.ini`). Compile sans avertissement.
- Essai sans l'interface (Windows PowerShell 5.1 chargeant l'exe) : lan 0 version, amont 10
  (fichier trouvé), takedawn 4 et burn3 6 (aucun fichier portable) ; installation de l'amont
  `v0.7.2-alpha` dans `essai/versions/amont/` réussie. La fenêtre s'ouvre.
