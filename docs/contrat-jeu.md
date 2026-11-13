# Le contrat entre le lanceur et nos versions du jeu

Ce que le lanceur suppose de nos versions (et de celles de l'amont, qui ont la même forme).
Changer une de ces règles demande de changer **les deux** projets.

## Ce qui existe déjà dans le jeu — CONFIRMÉ (code du 2026-10-30)

- **Le paquet** : `tools/package.py` (jeu) fait `Burnout3-PC-<version>-windows-x64.zip`. Chez
  l'amont, un dossier `Burnout3-PC-<version>/` dedans ; dans notre LAN edition (depuis
  `0.8.0-lan.1`, option `"flat": true` de `game.json`), les fichiers sont **à la racine** du zip.
  Le lanceur gère les deux : il enlève le premier dossier seulement si toutes les entrées sont
  dedans (`installation.rs`).
- **L'exe** : `burnout3.exe` ; sur un processeur sans AVX2, il lance lui-même
  `burnout3-sse4.exe`, à côté.
- **Sans arguments**, le jeu (`PlayArguments`, `engine/host/src/main.cpp`) :
  - lit `pc_settings.ini` à côté de l'exe ; la ligne `disc=<chemin>` donne l'image disque ;
  - sinon prend une image du jeu posée à côté de l'exe, sinon la demande ;
  - garde sa carte mémoire dans `memcard/`, à côté de l'exe ; ses pipelines Vulkan dans
    `gpu_pipeline_cache.bin` + `gpu_pipelines.txt` ; son journal dans `burnout3.log`
    (+ `burnout3.1.log`…`.5.log`).
- Donc le lanceur **écrit `disc=` dans `pc_settings.ini`** de la version (sans toucher aux autres
  lignes) : une seule copie de l'ISO pour toutes les versions PS2.
- **Ce que le lanceur écrit dans `pc_settings.ini`** (2026-11-13) : `disc=`, et les clés de
  PC SETTINGS que montre sa page (`resolution`, `aspect`, `fullscreen`, `vsync`,
  `frame_pacing`, `frame_rate`, `texture_pack`, `vrr`, `gpu_boost`, valeurs de
  `engine/host/src/pc_menu.cpp`) ; il enlève la ligne pour « par défaut ». Renommer une de ces
  clés ou de leurs valeurs dans le jeu demande de changer `ui/app.js` (`GAME_SETTINGS`).
- **Le disque reconnu** : le lanceur fait la même vérification que `IsGameDisc` (le `BOOT2` de
  `SYSTEM.CNF` = `SLUS_210.50`).
- **Le temps de jeu** suppose que le processus lancé dure toute la partie (vrai : la version
  SSE4 est attendue par `RunSibling`). Un jeu qui se relancerait lui-même fausserait le compte.
- **Fichiers à ne jamais écraser** en mettant à jour : tout ce qui n'est pas dans le zip
  (`memcard/`, `pc_settings.ini`, caches, journaux, `captures/`, `custom_songs/`,
  `texture pack/`). Le lanceur n'écrit que les fichiers présents dans le zip.

## À ajouter au jeu plus tard — à faire

1. ~~La version dans le jeu~~ : **existe déjà** — CONFIRMÉ (2026-11-13). `engine/host/CMakeLists.txt`
   écrit `FileVersion` = la `version` de `game.json` (`0.8.0-lan.1` ; un exe compilé avant ce
   numéro dit `0.8.0-alpha`, celui de l'amont). Le lanceur la lit (`winfo::file_version`) et ne
   prend `.lanceur-version` que si l'exe n'en a pas. **Monter `version` de `game.json` à chaque
   publication** : c'est ce numéro que le lanceur compare (semver) à l'étiquette publiée.
2. **Préparer les shaders seulement** : une option (`--warm-pipelines` par exemple) qui compile
   les pipelines, écrit le cache et quitte en donnant l'avancement (sur la sortie, ligne par
   ligne). Le lanceur l'appelle après une mise à jour et montre une barre de progression.
3. **L'indicateur dans le jeu** (barre de titre + texte par-dessus l'image) pendant la
   préparation des shaders : `SetWindowStatus` / `SetWindowOverlay` existent dans le moteur.
4. **La version en LAN** : le lobby refuse un jeu d'une autre version, avec un message clair.
