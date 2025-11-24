> **ytcs**: Téléchargez des albums YouTube complets, proprement découpés en pistes MP3 avec métadonnées et pochette, le tout via une simple ligne de commande.

[![Version](https://img.shields.io/badge/version-0.10.1-blue.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/releases) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/) [![CI](https://github.com/all3f0r1/youtube-chapter-splitter/workflows/CI/badge.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/actions/workflows/ci.yml)

---

`youtube-chapter-splitter` (ou `ytcs`) est un outil CLI puissant et pragmatique conçu pour une seule chose : archiver parfaitement la musique de YouTube. Il télécharge la vidéo, extrait l'audio en MP3, récupère la pochette, nettoie les titres, et découpe l'audio en pistes impeccables basées sur les chapitres, le tout en une seule commande.

## Philosophie

- **Pragmatique**: Pas de fioritures, juste ce qui compte.
- **Direct**: Info claire sans détours.
- **Classe**: Élégant sans être tape-à-l'œil.

```
ytcs v0.10.1

→ Marigold - Oblivion Gate
  29m 29s • 5 tracks

  ✓ Cover downloaded
  ✓ Audio downloaded

  Splitting tracks...
  ✓ 01 Oblivion Gate (5m 54s)
  ✓ 02 Obsidian Throne (5m 35s)
  ✓ 03 Crimson Citadel (5m 47s)
  ✓ 04 Silver Spire (6m 30s)
  ✓ 05 Eternal Pyre (5m 43s)

✓ Done → ~/Music/Marigold - Oblivion Gate
```

## Fonctionnalités

- **Téléchargement MP3**: Audio de haute qualité (192 kbps par défaut).
- **Pochette Automatique**: Pochette d'album intégrée aux métadonnées MP3.
- **Découpage par Chapitres**: Détection automatique des chapitres YouTube.
- **Détection de Silence**: Plan B si la vidéo n'a pas de chapitres.
- **Métadonnées Complètes**: Titre, artiste, album, numéro de piste, pochette.
- **Configuration Persistante**: Fichier `config.toml` pour vos préférences.
- **Formatage Personnalisable**: Noms de fichiers (`%n`, `%t`) et de dossiers (`%a`, `%A`).
- **Nettoyage Intelligent**: Supprime `[Full Album]`, `(Official Audio)`, etc.
- **Support des Playlists**: Gestion interactive des playlists.
- **Vérification des Dépendances**: `yt-dlp` et `ffmpeg` sont vérifiés au démarrage.

## Installation

### 1. Binaires Pré-compilés (Recommandé)

Récupérez la dernière version pour votre système sur la [page des Releases](https://github.com/all3f0r1/youtube-chapter-splitter/releases).

**Linux/macOS:**
```bash
# Téléchargez, extrayez, et installez
wget https://github.com/all3f0r1/youtube-chapter-splitter/releases/latest/download/ytcs-x86_64-unknown-linux-gnu.tar.gz
tar xzf ytcs-x86_64-unknown-linux-gnu.tar.gz
sudo mv ytcs /usr/local/bin/

# Vérifiez l'installation
ytcs --version
```

**Windows:**
1. Téléchargez `ytcs-x86_64-pc-windows-msvc.zip`.
2. Extrayez `ytcs.exe`.
3. Placez-le dans un dossier inclus dans votre `PATH`.

### 2. Via `cargo`

```bash
cargo install youtube_chapter_splitter
```

## Utilisation

`ytcs` fonctionne avec des commandes claires et directes.

### Télécharger une Vidéo

La commande par défaut est `download`. Vous pouvez l'omettre pour un usage rapide.

```bash
# Syntaxe complète
ytcs download "https://www.youtube.com/watch?v=..."

# Syntaxe rapide (recommandée)
ytcs "https://www.youtube.com/watch?v=..."
```

**Options de téléchargement:**
- `-o, --output <DIR>`: Spécifie un dossier de sortie.
- `-a, --artist <ARTIST>`: Force le nom de l'artiste.
- `-A, --album <ALBUM>`: Force le nom de l'album.
- `--no-cover`: Désactive le téléchargement de la pochette.

### Gérer la Configuration

`ytcs` utilise un fichier de configuration simple (`~/.config/ytcs/config.toml`).

```bash
# Afficher la configuration actuelle
ytcs config

# Modifier une valeur
ytcs set audio_quality 128
ytcs set playlist_behavior video_only

# Réinitialiser la configuration par défaut
ytcs reset
```

## Configuration

Personnalisez `ytcs` selon vos besoins. Modifiez directement le fichier `config.toml` ou utilisez `ytcs set`.

| Clé                  | Défaut                   | Description                                                                 |
|----------------------|--------------------------|-----------------------------------------------------------------------------|
| `default_output_dir` | `~/Music`                | Dossier de sortie par défaut.                                               |
| `download_cover`     | `true`                   | Télécharger la pochette de l'album.                                         |
| `filename_format`    | `"%n - %t"`              | Format du nom de fichier (`%n`: numéro, `%t`: titre, `%a`: artiste, `%A`: album). |
| `directory_format`   | `"%a - %A"`              | Format du dossier (`%a`: artiste, `%A`: album).                             |
| `audio_quality`      | `192`                    | Qualité audio en kbps (ex: `128`, `192`).                                   |
| `overwrite_existing` | `false`                  | Ré-télécharger et écraser les fichiers existants.                           |
| `max_retries`        | `3`                      | Nombre de tentatives en cas d'échec de téléchargement.                      |
| `create_playlist`    | `false`                  | Créer un fichier playlist `.m3u` pour les playlists YouTube.                |
| `playlist_behavior`  | `ask`                    | Comportement pour les URLs de playlist: `ask`, `video_only`, `playlist_only`. |

## Changelog

### [0.10.1] - 2025-11-24
- **Fixed:** Les sous-commandes CLI (`config`, `set`, `reset`) fonctionnent maintenant correctement.
- **Technical:** La structure CLI a été refactorisée pour gérer proprement les sous-commandes avec `clap`.

### [0.10.0] - 2025-11-24
- **Changed:** Refonte complète de l'UI pour un design minimaliste, propre et pragmatique.
- **Removed:** Suppression des bordures, boîtes et bruit visuel inutiles.
- **Improved:** Nettoyage automatique des titres de vidéos (supprime `[Full Album]`, `[Official Audio]`, etc.).

### [0.9.3] - 2025-11-24
- **Fixed:** Correction d'une erreur de parsing de configuration pour les utilisateurs migrant depuis d'anciennes versions.

---

## Licence

Ce projet est sous licence MIT. Voir le fichier [LICENSE](LICENSE) pour plus de détails.
