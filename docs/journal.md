# Journal des sessions

Le récit chronologique, **le plus récent en haut**. Pas une référence : la vérité actuelle est
dans les autres fichiers de `docs/`.

---

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
