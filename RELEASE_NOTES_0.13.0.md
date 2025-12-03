# Release Notes - Version 0.13.0

## 🎯 Objectif

Amélioration de l'observabilité et de la gestion des ressources avec des logs étendus et une gestion RAII cohérente pour tous les fichiers temporaires.

---

## ✅ Fonctionnalités implémentées

### 1. **Logs étendus dans le pipeline audio** 🔍

Des logs structurés ont été ajoutés dans les modules critiques pour un meilleur suivi des opérations.

**Dans `audio.rs` :**
- Log du démarrage du découpage avec le nombre de chapitres
- Log des détails (fichier d'entrée, répertoire de sortie, artiste, album)
- Log de chaque piste en cours de découpage avec son numéro et titre
- Log du chemin de sortie pour chaque fichier créé

**Exemple de logs (avec `RUST_LOG=info`) :**
```
INFO  Starting audio download from: https://youtube.com/watch?v=...
INFO  Audio download successful with format selector #1
INFO  Starting audio splitting: 5 chapters
```

**Exemple de logs (avec `RUST_LOG=debug`) :**
```
DEBUG Input file: "/tmp/temp_audio.mp3"
DEBUG Output directory: "/home/user/Music/Artist - Album"
DEBUG Artist: Paradox, Album: Chemical Love Theory
DEBUG Splitting track #1: Light Years Apart
DEBUG Output path: "/home/user/Music/Artist - Album/01 - Light Years Apart.mp3"
```

### 2. **Logs dans le parsing de chapitres** 📖

Des logs ont été ajoutés dans `chapters_from_description.rs` pour suivre la détection de chapitres.

**Ce qui est loggé :**
- Tentative de parsing depuis la description
- Durée de la vidéo et longueur de la description
- Nombre de chapitres trouvés (succès)
- Warning si aucun chapitre n'est trouvé

**Exemple :**
```
INFO  Attempting to parse chapters from description
DEBUG Video duration: 1847.00s
DEBUG Description length: 542 characters
INFO  Successfully parsed 5 chapters from description
```

### 3. **Gestion RAII pour les fichiers de couverture** 🧹

Les fichiers de couverture utilisent maintenant `TempFile` pour un nettoyage automatique cohérent.

**Comportement :**
- ✅ Le fichier `cover.jpg` est créé avec `TempFile`
- ✅ Si `--no-cover` n'est **pas** utilisé : le fichier est conservé avec `.keep()`
- ✅ Si `--no-cover` est utilisé : le fichier est automatiquement supprimé à la fin
- ✅ En cas d'erreur : le fichier est toujours nettoyé (RAII)

**Avant (v0.12.0) :**
```rust
// Nettoyage manuel
if cover_downloaded && !keep_cover {
    std::fs::remove_file(&cover_file).ok();
}
```

**Maintenant (v0.13.0) :**
```rust
let mut temp_cover = TempFile::new(&cover_path);
if keep_cover {
    temp_cover.keep(); // Conserve le fichier
}
// Sinon, suppression automatique par RAII
```

### 4. **Documentation du debugging dans le README** 📚

Une nouvelle section "Debugging with Logs" a été ajoutée au README.

**Contenu :**
- Exemples d'utilisation de `RUST_LOG`
- Liste de ce qui est loggé
- Instructions pour sauvegarder les logs dans un fichier
- Exemples de commandes pour différents niveaux de verbosité

**Extrait du README :**
```bash
# Show debug logs (very verbose, includes all operations)
RUST_LOG=debug ytcs "https://youtube.com/..."

# Show info logs (important events only)
RUST_LOG=info ytcs "https://youtube.com/..."

# Save logs to file
RUST_LOG=debug ytcs "https://youtube.com/..." 2>&1 | tee debug.log
```

---

## 📊 Statistiques

### Tests
- **Total** : 74 tests (71 tests unitaires + 3 tests TempFile)
- **Doc-tests** : 16 tests de documentation
- **Résultat** : ✅ **100% de réussite**

### Code
- **Fichiers modifiés** : 9
- **Lignes ajoutées** : ~290
- **Logs ajoutés** : 10+ points de log

### Compilation
- ✅ **Build dev** : Succès
- ✅ **Build release** : Succès
- ✅ **rustfmt** : Appliqué
- ✅ **Clippy** : 1 warning mineur (faux positif)

---

## 📦 Statut du push

- ✅ **Commit créé** : `d2f0800`
- ✅ **Tag créé et poussé** : `v0.13.0`
- ✅ **Branche `master` mise à jour**
- ✅ **Tous les tests passent** (74/74)
- ✅ **rustfmt appliqué**
- ✅ **Clippy vérifié**
- ✅ **README mis à jour**

---

## 🔄 Comparaison avec v0.12.0

| Aspect | v0.12.0 | v0.13.0 |
|--------|---------|---------|
| **Logs audio** | ⚠️ Téléchargement uniquement | ✅ Téléchargement + découpage |
| **Logs chapitres** | ❌ Aucun | ✅ Parsing et résultats |
| **Cover RAII** | ⚠️ Nettoyage manuel | ✅ RAII automatique |
| **Documentation logging** | ⚠️ Basique | ✅ Section dédiée |
| **Cohérence RAII** | ⚠️ Audio uniquement | ✅ Audio + Cover |

---

## 💡 Utilisation pratique

### Debugging d'un problème de découpage

```bash
RUST_LOG=debug ytcs "https://youtube.com/watch?v=..." 2>&1 | tee debug.log
```

Dans `debug.log`, vous verrez :
- Quel sélecteur de format a été utilisé
- Combien de chapitres ont été détectés
- Le chemin de chaque fichier créé
- Les détails de chaque opération

### Voir uniquement les événements importants

```bash
RUST_LOG=info ytcs "https://youtube.com/watch?v=..."
```

Affiche uniquement :
- Début/fin du téléchargement
- Nombre de chapitres trouvés
- Succès du découpage

### Filtrer les logs par module

```bash
# Logs uniquement pour le module audio
RUST_LOG=youtube_chapter_splitter::audio=debug ytcs "..."

# Logs uniquement pour le parsing de chapitres
RUST_LOG=youtube_chapter_splitter::chapters_from_description=debug ytcs "..."
```

---

## 🎓 Détails techniques

### Points de log ajoutés

**audio.rs :**
- `log::info!("Starting audio splitting: {} chapters", chapters.len())`
- `log::debug!("Input file: {:?}", input_file)`
- `log::debug!("Splitting track #{}: {}", track_number, title)`
- `log::debug!("Output path: {:?}", output_path)`

**chapters_from_description.rs :**
- `log::info!("Attempting to parse chapters from description")`
- `log::debug!("Video duration: {:.2}s", video_duration)`
- `log::info!("Successfully parsed {} chapters", chapters.len())`
- `log::warn!("No valid chapters found in description")`

### Gestion RAII du cover

```rust
// Création du TempFile
let mut temp_cover = TempFile::new(&cover_path);

// Téléchargement
match download_thumbnail(...) {
    Ok(_) => {
        if keep_cover {
            temp_cover.keep(); // Conserve
        }
        // Sinon, suppression automatique
    }
    Err(_) => {
        // Suppression automatique en cas d'erreur
    }
}
```

---

## 🚀 Prochaine version (0.14.0)

Les améliorations suivantes sont planifiées :

1. **Implémentation du timeout complet** avec gestion de processus
2. **Refactoring de `process_single_url`** en fonctions modulaires
3. **Métriques de performance** (temps de téléchargement, taille des fichiers)
4. **Tests d'intégration** pour les logs
5. **Logs dans les modules restants** (downloader, playlist)

---

## 🎯 Résumé

La version **0.13.0** est une release d'**observabilité et cohérence** qui ajoute :
- ✅ **Logs étendus** dans audio et chapters pour un meilleur debugging
- ✅ **Gestion RAII cohérente** pour tous les fichiers temporaires
- ✅ **Documentation du logging** dans le README

Ces améliorations rendent le programme plus facile à déboguer et plus robuste face aux erreurs, tout en maintenant une gestion cohérente des ressources.

---

**La version 0.13.0 est maintenant disponible !** Faites `git pull` sur votre machine locale pour la récupérer.

**Note importante** : Pensez à supprimer le token GitHub temporaire sur https://github.com/settings/tokens.
