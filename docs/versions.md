# Les versions du jeu que connaît le lanceur

Faits relevés le **2026-10-30** (API GitHub publique et pages des projets). Ces projets bougent
vite : revérifier avant de s'y fier.

## Notre version (PS2, LAN) — `nynjardin/burnout3-pc`

- Recompilation statique de la PS2 NTSC-U (`SLUS-21050`), l'amont plus notre LAN sans serveur
  (salon, découverte, colonne PING…). Disque : l'ISO PS2 US.
- **Aucune version publiée** sur le fork (CONFIRMÉ). Publier = décision de l'utilisateur.
- Paquet prévu : le même que l'amont (voir [contrat-jeu.md](contrat-jeu.md)).

## L'amont (PS2) — `danielgomesvieira2000/burnout3-pc`

- Même base que nous, sans le LAN... jusqu'à ce qu'il le fusionne (notre PR #4 y est déjà).
- Publie des **préversions** seulement (CONFIRMÉ) : `v0.7.2-alpha` (2026-10-28), `v0.7.1-alpha`,
  `v0.7.0-alpha`. ⚠️ Le point d'accès GitHub `releases/latest` ignore les préversions : lire
  `releases` et prendre la plus récente.
- Fichier joint : `Burnout3-PC-<version>-windows-x64.zip` (≈ 42 Mo). Dedans, un dossier
  `Burnout3-PC-<version>/` : `burnout3.exe`, `burnout3-sse4.exe`, `README.txt`,
  `THIRD-PARTY-NOTICES.txt`, `texture pack/README.txt`, `custom_songs/README.txt`.
- Apporte : 60 images/s, 16:9, pack de textures, PC SETTINGS, mods (sans trafic, crash
  breaker, miroir…), série EXTRA, caméra orbitale, écran partagé.

## takedawn (Xbox) — `xenophobentx/takedawn`

- Recompilation statique de la version **Xbox** NTSC-U (`default.xbe`) : il faut **le disque
  Xbox US**, pas l'ISO PS2. Refuse PAL et Japon (PAL prévu en 0.2.2).
- Versions : `v0.2.1` (2026-10-22), `v0.2.0`, `v0.1.1`, `v0.1.0` (13-10). Fichiers joints :
  Windows **`Takedawn-<v>-windows-x64-setup.exe` (un installateur)**, AppImage Linux, dmg Mac,
  apk Android, sources, `SHA256SUMS`.
- **Pas portable sous Windows** aujourd'hui : un mode portable est annoncé pour 0.2.2.
  Le lanceur se contente de l'afficher et d'ouvrir sa page (CONFIRMÉ : choix de départ).
- Apporte : 60 images/s (et au-delà, expérimental), rendu jusqu'à 4x, 16:9 et ultra-large,
  écran partagé, HDR, Windows/Linux/Mac/Android. Prévu en 0.2.2 : en ligne et LAN, PAL avec
  langues, mods.

## Burn3 (PS2) — `UwUCammie/Burn3`

- Interpréteur R5900 + code traduit à l'avance, rendu Vulkan (paraLLEl-GS, jusqu'à 6x), FSR
  3.1, options PC (F10), jeu en ligne par un serveur central réservé à ses versions officielles.
- Versions `v0.1.3` à `v0.1.5` (2026-10-29) **sans aucun fichier joint** (CONFIRMÉ) : rien à
  télécharger par le lanceur. Sa source ne se compile pas (fichiers manquants).
- Le lanceur l'affiche et ouvre sa page.
