# Potagerie

Appli de planification de potager, installée, sans serveur ni compte : Rust + Tauri v2, interface Svelte + TypeScript. Les specs sont dans `docs/specs/` (index : `docs/POT-00_index.md`).

## Installation (développement)

Ce qu'il faut à ce stade : la chaîne Rust. Tauri, Node.js et pnpm viendront avec l'appli et l'interface (S05, sections 3 et 4) ; rien à installer pour eux aujourd'hui.

1. **Installer rustup**, le gestionnaire de la chaîne Rust (une seule fois). Sur macOS :

   ```
   brew install rustup
   export PATH="$(brew --prefix rustup)/bin:$PATH"   # à ajouter aussi à ~/.zshrc
   ```

   Ou, sans Homebrew : `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`, puis rouvrir le terminal.

2. **Installer la chaîne stable et ses deux outils** (Rust 1.85 ou plus récent, édition 2024) :

   ```
   rustup default stable
   rustup component add clippy rustfmt
   ```

3. **Vérifier** :

   ```
   cargo --version
   cargo clippy --version
   cargo fmt --version
   ```

   La première compilation télécharge les dépendances (chrono, serde, uuid, thiserror) : il faut une connexion Internet une fois.

## Commandes

Depuis la racine du dépôt :

| Commande | Rôle |
|---|---|
| `cargo test` | tous les tests, dont T1 à T13 de S03 §12 (`crates/plan/tests/s03_cas.rs`) |
| `cargo test -p plan --test s03_cas` | seulement T1 à T13 |
| `cargo clippy --all-targets -- -D warnings` | lint, sans aucun avertissement toléré |
| `cargo fmt --check` | vérifie le formatage (`cargo fmt` pour l'appliquer) |

## Crates

| Crate | Contenu | Spec |
|---|---|---|
| `crates/catalogue` | `cultures.json` (les 6 cultures de la V1), embarqué et désérialisé | S02 |
| `crates/plan` | modèle du plan et tous les calculs : dates, stades, zones, conflits, créneaux, suggestions, tâches | S03, S05 §5 |

Les autres crates de S05 §8 (`export`, `store`), l'appli Tauri (`src-tauri`) et l'interface (`front`) ne sont pas encore créés.

Le crate `plan` ne lit jamais l'horloge : « aujourd'hui » est toujours un paramètre, et les dates du plan sont des dates civiles (`chrono::NaiveDate`).

## Skills Claude

Les skills du projet sont dans `skills/<nom>/SKILL.md`, versionnées avec le dépôt.

Claude Code les lit dans `.claude/skills/`. Pour qu'il trouve celles du dépôt, crée une fois ce lien depuis la racine :

```
mkdir -p .claude && ln -s ../skills .claude/skills
```
