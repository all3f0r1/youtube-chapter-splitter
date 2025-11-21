# GitHub Actions Workflows

Ce répertoire contient les workflows GitHub Actions pour l'intégration continue (CI) et la livraison continue (CD) du projet YouTube Chapter Splitter.

## Workflows disponibles

### 1. CI (Continuous Integration) - `ci.yml`

**Déclenchement** : À chaque push ou pull request sur les branches `master` ou `main`

**Objectifs** :
- Tester le code sur Linux, Windows et macOS
- Vérifier le formatage du code avec `rustfmt`
- Analyser le code avec `clippy`
- Compiler le projet en mode debug et release
- Générer un rapport de couverture de code

**Jobs** :
- **test** : Exécute tous les tests unitaires et d'intégration sur les 3 plateformes
- **fmt** : Vérifie que le code est correctement formaté
- **clippy** : Analyse statique du code pour détecter les problèmes potentiels
- **build** : Compile le projet en mode debug et release
- **coverage** : Génère un rapport de couverture de code avec `cargo-tarpaulin`

### 2. Release - `release.yml`

**Déclenchement** : Lors de la création d'un tag de version (format `v*.*.*`)

**Objectifs** :
- Compiler des binaires pour toutes les plateformes supportées
- Créer une release GitHub avec les binaires
- Générer des checksums SHA256 pour vérifier l'intégrité des fichiers

**Plateformes supportées** :
- **Linux x86_64 GNU** : `ytcs-x86_64-unknown-linux-gnu.tar.gz`
- **Linux x86_64 MUSL** : `ytcs-x86_64-unknown-linux-musl.tar.gz` (binaire statique)
- **Windows x86_64** : `ytcs-x86_64-pc-windows-msvc.zip`
- **macOS x86_64 (Intel)** : `ytcs-x86_64-apple-darwin.tar.gz`
- **macOS ARM64 (Apple Silicon)** : `ytcs-aarch64-apple-darwin.tar.gz`

**Fichiers générés** :
- Archives binaires pour chaque plateforme
- Fichiers `.sha256` individuels pour chaque archive
- `SHA256SUMS.txt` : Fichier combiné contenant tous les checksums

## Comment créer une release

### Méthode 1 : Via la ligne de commande

```bash
# 1. S'assurer que tous les changements sont commités
git status

# 2. Mettre à jour la version dans Cargo.toml
# Éditer manuellement ou avec sed :
sed -i 's/version = "0.8.4"/version = "0.8.5"/' Cargo.toml

# 3. Commiter le changement de version
git add Cargo.toml
git commit -m "Bump version to 0.8.5"

# 4. Créer un tag annoté
git tag -a v0.8.5 -m "Release v0.8.5: Description des changements"

# 5. Pousser le commit et le tag
git push origin master
git push origin v0.8.5
```

### Méthode 2 : Via GitHub CLI

```bash
# 1. Mettre à jour la version et commiter
sed -i 's/version = "0.8.4"/version = "0.8.5"/' Cargo.toml
git add Cargo.toml
git commit -m "Bump version to 0.8.5"
git push origin master

# 2. Créer une release avec gh
gh release create v0.8.5 \
  --title "Release v0.8.5" \
  --notes "Description des changements de cette version"
```

### Méthode 3 : Via l'interface GitHub

1. Aller sur https://github.com/all3f0r1/youtube-chapter-splitter/releases
2. Cliquer sur "Draft a new release"
3. Choisir un tag (créer `v0.8.5` si nécessaire)
4. Remplir le titre et la description
5. Publier la release

**Note** : Dans tous les cas, le workflow `release.yml` se déclenche automatiquement dès qu'un tag `v*.*.*` est créé.

## Suivi de l'exécution des workflows

### Via l'interface GitHub

1. Aller sur https://github.com/all3f0r1/youtube-chapter-splitter/actions
2. Sélectionner le workflow (CI ou Release)
3. Cliquer sur une exécution pour voir les détails

### Via GitHub CLI

```bash
# Lister les workflows
gh workflow list

# Voir les exécutions récentes du workflow CI
gh run list --workflow=ci.yml

# Voir les détails d'une exécution
gh run view <run-id>

# Voir les logs d'une exécution
gh run view <run-id> --log
```

## Vérification des binaires téléchargés

Après avoir téléchargé un binaire depuis une release, vérifiez son intégrité :

### Linux / macOS

```bash
# Télécharger l'archive et son checksum
wget https://github.com/all3f0r1/youtube-chapter-splitter/releases/download/v0.8.4/ytcs-x86_64-unknown-linux-gnu.tar.gz
wget https://github.com/all3f0r1/youtube-chapter-splitter/releases/download/v0.8.4/ytcs-x86_64-unknown-linux-gnu.tar.gz.sha256

# Vérifier le checksum
echo "$(cat ytcs-x86_64-unknown-linux-gnu.tar.gz.sha256)  ytcs-x86_64-unknown-linux-gnu.tar.gz" | shasum -a 256 -c

# Si OK, extraire et installer
tar xzf ytcs-x86_64-unknown-linux-gnu.tar.gz
sudo mv ytcs /usr/local/bin/
```

### Windows (PowerShell)

```powershell
# Télécharger l'archive et son checksum
Invoke-WebRequest -Uri "https://github.com/all3f0r1/youtube-chapter-splitter/releases/download/v0.8.4/ytcs-x86_64-pc-windows-msvc.zip" -OutFile "ytcs.zip"
Invoke-WebRequest -Uri "https://github.com/all3f0r1/youtube-chapter-splitter/releases/download/v0.8.4/ytcs-x86_64-pc-windows-msvc.zip.sha256" -OutFile "ytcs.zip.sha256"

# Vérifier le checksum
$expectedHash = Get-Content ytcs.zip.sha256
$actualHash = (Get-FileHash ytcs.zip -Algorithm SHA256).Hash.ToLower()
if ($expectedHash -eq $actualHash) {
    Write-Host "Checksum OK"
    Expand-Archive ytcs.zip -DestinationPath .
} else {
    Write-Host "Checksum FAILED"
}
```

## Badges de statut

Ajoutez ces badges dans le README.md principal pour afficher le statut des workflows :

```markdown
[![CI](https://github.com/all3f0r1/youtube-chapter-splitter/workflows/CI/badge.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/actions/workflows/ci.yml)
[![Release](https://github.com/all3f0r1/youtube-chapter-splitter/workflows/Release/badge.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/all3f0r1/youtube-chapter-splitter/branch/master/graph/badge.svg)](https://codecov.io/gh/all3f0r1/youtube-chapter-splitter)
```

## Dépannage

### Le workflow de release échoue

**Problème** : Erreur de compilation sur une plateforme spécifique

**Solution** :
1. Vérifier les logs du workflow dans l'onglet Actions
2. Tester localement la compilation pour cette cible :
   ```bash
   rustup target add x86_64-pc-windows-msvc
   cargo build --release --target x86_64-pc-windows-msvc
   ```
3. Corriger les erreurs et créer un nouveau tag

### Les checksums ne correspondent pas

**Problème** : Le fichier téléchargé a un checksum différent

**Solution** :
1. Re-télécharger le fichier (corruption possible)
2. Vérifier que vous avez téléchargé la bonne version
3. Signaler le problème si le problème persiste

### Le workflow CI échoue sur une plateforme

**Problème** : Les tests passent localement mais échouent sur GitHub Actions

**Solution** :
1. Vérifier les différences d'environnement (chemins, permissions, etc.)
2. Ajouter des logs de debug dans les tests
3. Utiliser `act` pour tester localement les workflows GitHub Actions :
   ```bash
   # Installer act
   brew install act  # macOS
   # ou
   curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash  # Linux
   
   # Exécuter le workflow CI localement
   act -j test
   ```

## Ressources

- [Documentation GitHub Actions](https://docs.github.com/en/actions)
- [Rust GitHub Actions](https://github.com/actions-rs)
- [cargo-dist](https://github.com/axodotdev/cargo-dist) - Alternative pour la distribution
- [release-plz](https://github.com/MarcoIeni/release-plz) - Automatisation des releases
