# Guide de démarrage rapide

## Installation rapide

```bash
# 1. Installer les dépendances système
sudo apt install ffmpeg
pip install yt-dlp

# 2. Compiler le projet
cargo build --release

# 3. (Optionnel) Installer globalement
sudo cp target/release/ytcs /usr/local/bin/
```

## Utilisation basique

### Télécharger et diviser une vidéo YouTube

```bash
# Exemple avec la vidéo MARIGOLD - Oblivion Gate
ytcs download --url "https://www.youtube.com/watch?v=28vf7QxgCzA"
```

Cette commande va :
1. Télécharger l'audio de la vidéo en MP3
2. Extraire les chapitres (5 pistes dans cet exemple)
3. Créer un fichier MP3 séparé pour chaque piste
4. Sauvegarder dans `./output/MARIGOLD - Oblivion Gate [Full Album]/`

### Résultat attendu

```
output/
└── MARIGOLD - Oblivion Gate [Full Album]/
    ├── 01 - Oblivion Gate.mp3
    ├── 02 - Obsidian Throne.mp3
    ├── 03 - Crimson Citadel.mp3
    ├── 04 - Silver Spire.mp3
    └── 05 - Eternal Pyre.mp3
```

## Cas d'usage courants

### 1. Vidéo avec chapitres (automatique)

```bash
ytcs download --url "URL_YOUTUBE"
```

### 2. Vidéo sans chapitres (détection automatique)

```bash
ytcs download --url "URL_YOUTUBE" --detect-silence true
```

### 3. Diviser un fichier audio local

```bash
ytcs split --input mon_album.mp3 --output ./pistes --detect-silence
```

### 4. Voir les informations avant téléchargement

```bash
ytcs info --url "URL_YOUTUBE"
```

## Ajustement des paramètres

Si la détection automatique ne fonctionne pas bien :

```bash
# Silences plus subtils
ytcs download --url "URL" --silence-threshold -40

# Ignorer les courtes pauses
ytcs download --url "URL" --min-silence-duration 3.0

# Combiner les deux
ytcs download --url "URL" --silence-threshold -35 --min-silence-duration 2.5
```

## Aide

Pour voir toutes les options disponibles :

```bash
ytcs --help
ytcs download --help
ytcs split --help
ytcs info --help
```
