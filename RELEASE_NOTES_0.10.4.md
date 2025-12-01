# YouTube Chapter Splitter - Version 0.10.4

## Résumé des modifications

La version 0.10.4 apporte deux améliorations majeures pour la détection des métadonnées musicales :

### 1. Détection du nom d'artiste depuis le nom de la chaîne

Lorsque le titre de la vidéo ne contient pas de séparateur artiste-album (comme `-` ou `|`), le programme utilise maintenant le nom de la chaîne YouTube comme nom d'artiste au lieu de "Unknown Artist".

**Exemple :**
- Vidéo : https://www.youtube.com/watch?v=ujK_oAKnHXg
- Nom de la chaîne : **HasvAlner**
- Titre de la vidéo : "Some Album Title"
- Résultat : Artiste = **Hasvalner** (nom de la chaîne capitalisé), Album = "Some Album Title"

### 2. Support du format de liste de pistes dans la description

Le programme détecte maintenant automatiquement les chapitres dans le format suivant :

```
1 - The Cornerstone of Some Dream (0:00)
2 - Architects of Inner Time (Part I) (4:24)
3 - The Ritual of the Octagonal Chamber (11:01)
4 - Colors at the Bottom of the Gesture (Instrumental) (17:52)
5 - The Ballad of the Hourglass Man (22:23)
6 - Mirror Against the Firmament (Suite in Three Parts) (26:43)
7 - The Navigation of Rational Ice (31:28)
8 - The Guardian of the Shadow Papyri (35:24)
9 - The Cycle of Chalk and Fine Sand (40:29)
10 - Song for the Submerged Mountains (44:11)
11 - The Filters of Chronos (48:35)
12 - Architects of Inner Time (Part II: The Awakening) (51:42)
```

Ce format est couramment utilisé dans les descriptions de vidéos musicales sur YouTube.

## Modifications techniques

### Fichiers modifiés

1. **src/utils.rs**
   - `parse_artist_album()` accepte maintenant un paramètre `uploader` (nom de la chaîne)
   - Utilise le nom de la chaîne comme fallback si l'artiste n'est pas détecté dans le titre

2. **src/chapters_from_description.rs**
   - Ajout d'une regex pour détecter le format `N - Title (MM:SS)`
   - Priorité au format avec numéro de piste, puis fallback sur le format classique

3. **src/main.rs**
   - Mise à jour des appels à `parse_artist_album()` pour passer le nom de la chaîne

4. **tests/**
   - Mise à jour de tous les tests pour passer le paramètre `uploader`
   - Ajout de tests pour le nouveau format de liste de pistes
   - Ajout de tests pour le fallback sur le nom de la chaîne

### Tests

Tous les tests passent avec succès :
- 25 tests unitaires dans la bibliothèque
- 28 tests dans les fichiers de test
- 2 tests d'intégration end-to-end
- Total : **55+ tests** passés

## Installation

La version 0.10.4 est maintenant disponible sur GitHub :
- Commit : `89da924`
- Tag : `v0.10.4`
- Branche : `master`

Pour mettre à jour :
```bash
git pull origin master
cargo build --release
```

## Changelog complet

Voir [CHANGELOG.md](CHANGELOG.md) pour le changelog complet.
