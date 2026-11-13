# Journal des sessions

Le récit chronologique, **le plus récent en haut**. Pas une référence : la vérité actuelle est
dans les autres fichiers de `docs/`.

---

suite 8 (lanceur v0.2.0 publié — 2026-11-13)

- L'utilisateur : la version sans PowerShell marche sur l'autre machine (l'antivirus la laisse) ;
  il demande de commiter et publier.
- `version` 0.2.0 (`Cargo.toml`, `tauri.conf.json`), compilé sans avertissement : `Lanceur.exe`
  9 713 664 octets, SHA-256 `E4E9E941…C075F3`, ressource de version 0.2.0 / © 2026 nynjardin.
- Commit poussé sur `main`, version **`v0.2.0`** publiée (non préversion) avec `Lanceur.exe`.

suite 7 (version lue dans l'exe du jeu ; plus de PowerShell — 2026-11-13)

- Retour de l'utilisateur : lanceur copié dans `I:\Projets\Recomp\Burnout 3\` (son dossier de
  jeu) → « Mettre à jour » proposé alors que son jeu lui semble plus récent. Cause : sans
  `.lanceur-version`, toute version publiée paraissait différente. Son `burnout3.exe` (compilé
  le 13/11 à 03:49, après le dernier commit de jeu de 03:46) dit `FileVersion` `0.8.0-alpha` :
  le même jeu que `v0.8.0-lan.1`, avant le changement de numéro.
- Correction : version lue dans l'exe (`GetFileVersionInfoW`), comparée en semver ; mise à jour
  proposée seulement si la publiée est plus haute ; « Plus récente » sinon ; « Installer vX » si
  inconnue. Essais : `essai/lan` → `0.8.0-lan.1`, « À jour » ; `essai/alpha` (copie de son exe)
  → `0.8.0-alpha`, « Mise à jour » (vrai au sens des numéros).
- Second retour : un antivirus d'une autre machine a supprimé le lanceur. Trouvé : la
  bibliothèque `open` lançait **PowerShell** pour « Dossier » / « Page » (depuis la v0.1.0), et
  « Journaux » aussi (suite 6). Remplacés par l'API Windows (`ShellExecuteW`, registre,
  `GlobalMemoryStatusEx`) ; `open` retiré ; le mot n'apparaît plus dans l'exe. Copyright et
  éditeur ajoutés à la ressource de version. Defender local : rien. Antivirus en cause : INCONNU.

suite 6 (disque vérifié, carte mémoire, retour arrière, réglages, temps de jeu, journaux — 2026-11-13)

- Idées proposées à l'utilisateur ; ses choix : rien sur le réseau entre PC (doublon avec
  LanCommander) ; la barre des shaders n'est pas utile (le jeu montre déjà son avancement) ;
  disque vérifié pour chaque version ; tout le confort validé ; habillage mis de côté.
  Question « envoyer les journaux où ? » : nulle part, un zip à passer à la main.
- Rust : `disc.rs` (BOOT2 de SYSTEM.CNF, .iso/.bin/.cue ; Xbox par son volume), `gameini.rs`,
  `saves.rs`, `journaux.rs` ; `installation.rs` garde les deux derniers zips (vérifiés par
  taille + SHA-256) et rend le processus du jeu ; `main.rs` : commandes nouvelles, temps de jeu
  compté dans un fil qui attend la fin du jeu, rien d'installé ni restauré pendant qu'il tourne.
  Catalogue : champ `reglages` (lan, amont). Dépendance `chrono` (heure locale).
- **Bogue trouvé à l'essai et corrigé** : `state` gardait le verrou des réglages pendant que
  `disc_states` le reprenait → page figée au démarrage.
- Essais dans `essai/lan/` par le port de débogage : vrai ISO reconnu (`SLUS_210.50`, « celui du
  jeu » depuis `disc=`), `burnout3.exe` refusé comme disque ; réglage écrit puis enlevé, ligne de
  commentaire gardée, clé `disc` et valeur avec retour à la ligne refusées ; copie, restauration
  (copie « avant restauration » faite d'abord), nom `../x` refusé ; installation depuis GitHub
  3,4 s puis depuis le zip gardé 0,6 s, une copie de la carte avant chacune ; Jouer → « En jeu »,
  jeu arrêté au bout de 16 s → temps compté, message du code -1 ; zip des journaux avec infos
  de la machine. Captures de la page vérifiées. Compilé sans avertissement (exe 9,8 Mo).
- Le bouton « Revenir à » n'a pas pu être vu : une seule version publiée.

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
- À sa demande : code poussé (`master` local → `main` du dépôt, remote `origin`) et version
  **`v0.1.0`** publiée avec `Lanceur.exe` (9 380 352 octets) ; l'API donne bien le fichier et
  son `digest` SHA-256, identique à l'exe local.

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
