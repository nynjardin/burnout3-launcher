# Le contrat entre le lanceur et nos versions du jeu

Ce que le lanceur suppose de nos versions (et de celles de l'amont, qui ont la même forme).
Changer une de ces règles demande de changer **les deux** projets.

## Ce qui existe déjà dans le jeu — CONFIRMÉ (code du 2026-10-30)

- **Le paquet** : `tools/package.py` (jeu) fait `Burnout3-PC-<version>-windows-x64.zip`, un
  dossier `Burnout3-PC-<version>/` dedans. Le lanceur enlève ce premier dossier en extrayant.
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
- **Fichiers à ne jamais écraser** en mettant à jour : tout ce qui n'est pas dans le zip
  (`memcard/`, `pc_settings.ini`, caches, journaux, `captures/`, `custom_songs/`,
  `texture pack/`). Le lanceur n'écrit que les fichiers présents dans le zip.

## À ajouter au jeu plus tard — à faire

1. **La version dans le jeu** : un moyen pour le lanceur de lire la version d'un exe sans le
   lancer (ressource de version Windows `FileVersion`, à vérifier dans le build) ; aujourd'hui
   le lanceur retient l'étiquette qu'il a installée (`.lanceur-version`).
2. **Préparer les shaders seulement** : une option (`--warm-pipelines` par exemple) qui compile
   les pipelines, écrit le cache et quitte en donnant l'avancement (sur la sortie, ligne par
   ligne). Le lanceur l'appelle après une mise à jour et montre une barre de progression.
3. **L'indicateur dans le jeu** (barre de titre + texte par-dessus l'image) pendant la
   préparation des shaders : `SetWindowStatus` / `SetWindowOverlay` existent dans le moteur.
4. **La version en LAN** : le lobby refuse un jeu d'une autre version, avec un message clair.
