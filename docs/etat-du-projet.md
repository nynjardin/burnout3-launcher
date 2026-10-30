# État du projet — Lanceur Burnout 3

## Tableau de bord

| Sujet | État |
|---|---|
| Dossier, dépôt git local, doc | ✅ 2026-10-30 |
| Lanceur Tauri : catalogue, fiches, versions GitHub, installer/mettre à jour un zip, jouer | ✅ compile sans avertissement (exe 9,3 Mo) ; installation de l'amont par l'interface vérifiée (clic simulé + capture) ; rien écrit hors du dossier. **Jouer et l'allure pas encore vus par l'utilisateur** |
| Publication de nos versions (Releases du fork) | ⏸ décision de l'utilisateur |
| Dépôt GitHub du lanceur | ⏸ décision de l'utilisateur |
| Préparation des shaders depuis le lanceur | ⏳ demande d'abord une option dans le jeu ([contrat-jeu.md](contrat-jeu.md)) |

## Décisions de l'utilisateur

- Un projet à part, dossier propre, relié au jeu par la doc (2026-10-30).
- **Portable** : rien à installer.
- Plusieurs versions au choix (la nôtre, l'amont, takedawn, Burn3), chacune avec ce qu'elle
  apporte ; la nôtre suit nos mises à jour, les autres affichent les leurs.

## ▶ REPRENDRE ICI

1. **Faire essayer le lanceur à l'utilisateur** : `essai/Lanceur.exe` (après
   `cargo build --release` dans `src-tauri/`, recopier `target/release/Lanceur.exe` dans `essai/`).
   Dans `essai/` : l'amont `v0.7.2-alpha` est déjà installé (`versions/amont/`), le disque PS2
   réglé sur `I:\Projets\Recomp\Burnout 3\Burnout 3 - Takedown (USA).iso` (`lanceur.json`).
   À vérifier avec lui : l'allure de la fenêtre, **Jouer** (premier lancement long : shaders),
   ses remarques sur les fiches.
2. Quand l'utilisateur le décide : publier une version de notre jeu sur le fork, puis vérifier
   que le lanceur la voit.
3. Côté jeu : l'option « préparer les shaders seulement » et l'indicateur
   ([contrat-jeu.md](contrat-jeu.md)).
