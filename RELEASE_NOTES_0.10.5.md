# YouTube Chapter Splitter - Version 0.10.5

## Résumé des modifications

La version 0.10.5 apporte des améliorations majeures à l'interface utilisateur avec des barres de progression en temps réel et une expérience utilisateur simplifiée.

### 1. Barres de progression en temps réel

**Téléchargement vidéo :**
- Affichage d'une barre de progression pendant le téléchargement
- Message "Making an album out of the video" avec spinner

**Conversion audio :**
- Barre de progression remplace "Audio downloaded"
- Message final "✓ Audio ready"

**Découpage des pistes :**
- Barre de progression pour chaque piste individuellement
- Affichage du nom formaté de la piste (ex: "01 - Artist - Title")
- Indication de la durée pour chaque piste

### 2. Utilisation directe sans commande

**Avant :**
```bash
ytcs download "https://www.youtube.com/watch?v=..."
```

**Maintenant :**
```bash
ytcs "https://www.youtube.com/watch?v=..."
```

La commande `download` est maintenant optionnelle !

### 3. Validation des URLs YouTube

Le programme vérifie maintenant que l'URL fournie est bien une URL YouTube valide et affiche un message d'erreur clair si ce n'est pas le cas.

### 4. Affichage cohérent du titre

Le titre affiché au début correspond maintenant exactement au nom du dossier de sortie :
- **Avant :** "ECLIPSERA - Circle of the Endless Earth (1971)"
- **Maintenant :** "Eclipsera - Circle Of The Endless Earth"

### 5. Affichage intelligent du nombre de pistes

- **Chapitres YouTube détectés :** "5 tracks"
- **Vérification de la description :** "checking description..."
- **Détection de silence :** "? tracks → silence detection mode"

### 6. Messages UI améliorés

**Avant :**
```
Making an album

  Splitting tracks...

  ✓ 01 Whispers of the Twilight Path (6m 46s)
  ✓ 02 Chase of the Crimson Beast (6m 32s)
  ...
```

**Maintenant :**
```
Making the album...

  01 - Ethereal Compass - Whispers of the Twilight Path (6m 46s) [barre de progression]
  ✓ 01 - Ethereal Compass - Whispers of the Twilight Path (6m 46s)
  02 - Ethereal Compass - Chase of the Crimson Beast (6m 32s) [barre de progression]
  ✓ 02 - Ethereal Compass - Chase of the Crimson Beast (6m 32s)
  ...
```

## Modifications techniques

### Nouveaux fichiers

1. **src/progress.rs** - Module pour gérer les barres de progression
   - `create_download_progress()` - Barre pour le téléchargement
   - `create_audio_progress()` - Barre pour la conversion audio
   - `create_track_progress()` - Barre pour le découpage de piste

2. **src/audio.rs** - Nouvelle fonction
   - `split_single_track()` - Découpe une seule piste (permet l'affichage progressif)
   - `load_cover_image()` - Maintenant publique pour être utilisée dans main.rs

### Fichiers modifiés

1. **src/ui.rs**
   - `print_video_info()` - Accepte maintenant des paramètres pour l'affichage intelligent
   - `clean_title()` - Utilise `clean_folder_name()` pour la cohérence
   - `print_cover_status()` - Remplace `print_download_status()`
   - Suppression de `print_splitting_start()`
   - `print_track()` - Affiche le nom formaté complet avec artiste

2. **src/main.rs**
   - Support des URLs directes sans commande `download`
   - Validation des URLs YouTube
   - Boucle de découpage piste par piste avec barres de progression
   - Affichage du titre nettoyé correspondant au dossier

3. **src/downloader.rs**
   - `download_audio()` - Accepte maintenant une `ProgressBar` optionnelle

4. **tests/**
   - Mise à jour de tous les tests pour la nouvelle signature de `download_audio()`

### Tests

✅ Tous les tests passent avec succès :
- 25 tests unitaires dans la bibliothèque
- 28 tests dans les fichiers de test
- 2 tests d'intégration end-to-end
- 9 tests de documentation
- **Total : 64+ tests** passés

## Installation

La version 0.10.5 est maintenant disponible sur GitHub :
- Commit : `80e5d47`
- Tag : `v0.10.5`
- Branche : `master`

Pour mettre à jour :
```bash
cd ~/RustroverProjects/youtube-chapter-splitter
git pull origin master
cargo build --release
```

## Changelog complet

Voir [CHANGELOG.md](CHANGELOG.md) pour le changelog complet.

## Note de sécurité

⚠️ **Important :** Pensez à supprimer le token GitHub temporaire que vous avez créé pour ce push sur https://github.com/settings/tokens
