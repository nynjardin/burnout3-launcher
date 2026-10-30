# Journal des sessions

Le récit chronologique, **le plus récent en haut**. Pas une référence : la vérité actuelle est
dans les autres fichiers de `docs/`.

---

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
