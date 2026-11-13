# État du projet — Lanceur Burnout 3

## Tableau de bord

| Sujet | État |
|---|---|
| Dossier, dépôt git local, doc | ✅ 2026-10-30 |
| Lanceur Tauri : catalogue, fiches, versions GitHub, installer/mettre à jour un zip, jouer | ✅ compile sans avertissement (exe 9,3 Mo) ; installation de l'amont par l'interface vérifiée (clic simulé + capture) ; rien écrit hors du dossier. **Jouer et l'allure pas encore vus par l'utilisateur** |
| Publication de nos versions (Releases du fork) | ✅ `v0.8.0-lan.1` (préversion, 2026-11-13) ; le lanceur la voit et l'installe (vérifié dans `essai/lan/`) |
| Mise à jour du lanceur par lui-même | ✅ bandeau + remplacement de l'exe + redémarrage, vérifié en local (faux serveur) ; attend le dépôt du lanceur |
| Dépôt GitHub du lanceur | ✅ `nynjardin/burnout3-launcher` créé (public, vide) le 2026-11-13 ; code pas poussé, aucune version publiée |
| Préparation des shaders depuis le lanceur | ⏳ demande d'abord une option dans le jeu ([contrat-jeu.md](contrat-jeu.md)) |

## Décisions de l'utilisateur

- Un projet à part, dossier propre, relié au jeu par la doc (2026-10-30).
- **Portable** : rien à installer.
- Plusieurs versions au choix (la nôtre, l'amont, takedawn, Burn3), chacune avec ce qu'elle
  apporte ; la nôtre suit nos mises à jour, les autres affichent les leurs.
- (2026-11-13) Le lanceur sert surtout entre amis : **notre version = le jeu posé à côté du
  lanceur** (pas de copie dans `versions/`) ; **les autres versions grisées** pour l'instant.

## ▶ REPRENDRE ICI

1. **Faire essayer le lanceur à l'utilisateur** : copier `src-tauri/target/release/Lanceur.exe`
   **dans un dossier du jeu** (à côté d'un `burnout3.exe`) ; notre version doit s'afficher
   « Présente », les trois autres grisées. À vérifier avec lui : l'allure, **Jouer** (premier
   lancement long : shaders), ses remarques. (`essai/` n'a pas de `burnout3.exe` à sa racine :
   la carte y dit « Introuvable ».)
2. ~~Publier notre version et vérifier que le lanceur la voit~~ : fait (`v0.8.0-lan.1`).
   Pour un ami sans le jeu : `Lanceur.exe` seul dans un dossier vide → « Installer ».
3. **Dépôt du lanceur** `nynjardin/burnout3-launcher` (créé, vide) : y pousser le code et
   publier `v0.1.0` avec `Lanceur.exe`, **quand l'utilisateur le dira**.
   Les lanceurs déjà distribués ne voient une mise à jour qu'à partir de là. Procédure :
   [conception.md](conception.md), « Le lanceur se met à jour lui-même ».
4. Côté jeu : l'option « préparer les shaders seulement » et l'indicateur
   ([contrat-jeu.md](contrat-jeu.md)).
