# Changelog des Améliorations

## Version Améliorée (Novembre 2025)

### ✨ Nouvelles Fonctionnalités

- **Barres de progression** : Ajout de barres de progression visuelles pour le téléchargement audio et le découpage des pistes (via `indicatif`)
- **Retry automatique** : Mécanisme de nouvelle tentative (3 essais) pour le téléchargement des miniatures
- **Timeout réseau** : Configuration d'un timeout de 30 secondes pour les requêtes HTTP

### 🚀 Performance

- **Optimisation regex** : Les expressions régulières sont maintenant compilées une seule fois au démarrage (via `once_cell`)
- Réduction significative de la surcharge CPU lors du traitement de multiples fichiers

### 📚 Documentation

- **Documentation rustdoc complète** : Tous les modules publics sont maintenant documentés
- Exemples d'utilisation ajoutés dans `lib.rs`
- Documentation des fonctions avec arguments, retours et erreurs possibles

### 🧪 Tests

- **Nouveaux tests unitaires** :
  - `tests/test_chapters.rs` : Tests pour les chapitres et timestamps
  - `tests/test_downloader.rs` : Tests pour l'extraction d'ID vidéo
  - `tests/test_error.rs` : Tests pour la gestion d'erreurs
- Amélioration de la couverture de code

### 🔧 CI/CD

- **GitHub Actions CI** :
  - Tests automatiques sur Linux, macOS et Windows
  - Vérification du formatage (`rustfmt`)
  - Analyse statique (`clippy`)
  - Build multi-plateforme
  - Génération de documentation

- **GitHub Actions Release** :
  - Création automatique de releases
  - Compilation de binaires pour toutes les plateformes
  - Publication automatique sur crates.io

### 🛠️ Améliorations Techniques

- Ajout de `once_cell = "1.19"` pour l'optimisation des regex
- Ajout de `indicatif = "0.17"` pour les barres de progression
- Configuration de `ureq` avec la feature `json`
- Refactorisation de `sanitize_title` dans `utils.rs`

### 📝 Fichiers Modifiés

- `Cargo.toml` : Nouvelles dépendances
- `src/utils.rs` : Regex optimisées + documentation
- `src/chapters.rs` : Documentation + refactorisation
- `src/downloader.rs` : Barre de progression + retry + timeout + documentation
- `src/audio.rs` : Barre de progression
- `src/error.rs` : Documentation complète
- `src/lib.rs` : Documentation du module

### 📁 Fichiers Créés

- `.github/workflows/ci.yml` : Pipeline CI
- `.github/workflows/release.yml` : Pipeline de release
- `tests/test_chapters.rs` : Tests des chapitres
- `tests/test_downloader.rs` : Tests du downloader
- `tests/test_error.rs` : Tests des erreurs
- `CHANGELOG_IMPROVEMENTS.md` : Ce fichier

---

**Note** : Ces améliorations sont rétrocompatibles et ne cassent pas l'API existante.
