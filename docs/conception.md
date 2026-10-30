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
  catalogue.json        (facultatif) remplace le catalogue intégré à l'exe
  versions/
    lan/  amont/  ...   une version par dossier, avec ses sauvegardes et caches
```

Pas de registre, pas de droits d'administrateur, rien hors de ce dossier.

## Choix techniques

- **C# / .NET Framework 4.8 / WinForms** : livré avec Windows 10 et 11, donc un seul exe sans
  dépendance. Aucun paquet NuGet : JSON par `JavaScriptSerializer` (System.Web.Extensions),
  zip par `System.IO.Compression`, HTTP par `HttpClient` (TLS 1.2 forcé).
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
