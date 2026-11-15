# Potagerie

Une appli installée pour planifier son potager : un seul utilisateur, sans compte ni serveur. Le cœur est en Rust, l'interface en Svelte 5 + TypeScript, le tout emballé par Tauri v2 (macOS et Windows au lot 1).

La documentation (specs, designs, prototypes) est dans [`docs/`](docs/POT-00_index.md). La référence technique est [S05](docs/specs/POT-S05_spec-technique_v0.11.md).

## Installer

1. Ouvrir la page [Releases](../../releases) du dépôt et télécharger l'installeur de la dernière version : le `.dmg` sur macOS (universel, Apple Silicon et Intel), le `.exe` ou le `.msi` sur Windows.
2. Installer : glisser Potagerie dans Applications sur macOS, lancer l'installeur sur Windows.

Les installeurs ne sont pas signés, le système demande donc une confirmation au premier lancement.

- **macOS** : clic droit sur Potagerie dans Applications, puis Ouvrir, puis Ouvrir de nouveau. Si macOS dit que l'appli est « endommagée », retirer la quarantaine : `xattr -dr com.apple.quarantine /Applications/Potagerie.app`.
- **Windows** : si SmartScreen bloque l'installeur, cliquer sur « Informations complémentaires », puis « Exécuter quand même ».

**Données.** Tout reste sur l'ordinateur, dans `potagerie.db` : `~/Library/Application Support/garden.potagerie/` sur macOS, `%APPDATA%\garden.potagerie\` sur Windows. Les réglages permettent d'exporter le potager en JSON.

**Désinstaller.** macOS : mettre Potagerie à la corbeille. Windows : Paramètres > Applications > Potagerie > Désinstaller. Les données ne sont pas supprimées : effacer le dossier ci-dessus pour repartir de zéro.

## Prérequis

- Rust stable (`rustup`), avec `rustfmt` et `clippy`.
- La CLI Tauri : `cargo install tauri-cli --version "^2" --locked`.
- Node.js 22 ou plus, et pnpm (la version est fixée dans `front/package.json`, `corepack enable` suffit).
- macOS : les outils en ligne de commande de Xcode. Windows : WebView2, présent par défaut sur Windows 10 et 11 à jour. Linux (CI seulement) : `libwebkit2gtk-4.1-dev` et les paquets listés dans `.github/workflows/ci.yml`.

## Commandes

| Commande | Rôle |
|---|---|
| `make dev` | Ouvre l'appli (`cargo tauri dev`) ; l'interface se recharge à chaud. Régénère `front/src/lib/bindings.ts` |
| `make build` | Compile `Potagerie.app` et le `.dmg` sur macOS, le `.exe` et le `.msi` sur Windows, dans `target/release/bundle/` |
| `make test` | `cargo test` sur tout le workspace, puis Vitest |
| `make check` | `cargo fmt --check`, clippy en `-D warnings`, `svelte-check` : ce que vérifie la CI |
| `make fmt` | Formate le code Rust |
| `make bindings` | Réécrit `bindings.ts` depuis les commandes Rust (un test échoue s'il est périmé) |

Pour travailler sur l'interface seule, `pnpm --dir front dev` l'ouvre dans un navigateur, sur `http://localhost:5173`, avec de fausses commandes en mémoire (`?exemple` pour partir d'un potager rempli, `?reglages` ou `?potager` pour ouvrir un écran).

Les données sont dans `potagerie.db`, dans le dossier de données de l'appli (`~/Library/Application Support/garden.potagerie/` sur macOS, `%APPDATA%\garden.potagerie\` sur Windows).

## Arborescence

```
Cargo.toml              workspace
crates/plan/            modèle, règles et calculs (S03), vue de l'année
crates/catalogue/       cultures.json (S02), embarqué
crates/store/           trait Store et implémentation SQLite (rusqlite)
crates/export/          JSON du potager, PNG et PDF de la frise (resvg, svg2pdf), polices embarquées
src-tauri/              appli Tauri : commandes, capacités, icônes
front/                  interface Svelte + TypeScript (Vite, pnpm)
  messages/fr.json      tous les textes de l'interface (Paraglide JS)
  src/lib/bindings.ts   généré par tauri-specta, jamais modifié à la main
  src/lib/atelier/      onglet Plan
  src/lib/diorama/      onglet Potager
  src/lib/reglages/     réglages, export et import
docs/                   specs, designs, prototypes
Makefile                dev, build, test, check, fmt
```

## Skills Claude

Les skills du projet sont dans `skills/<nom>/SKILL.md`, versionnées avec le dépôt.

Claude Code les lit dans `.claude/skills/`. Pour qu'il trouve celles du dépôt, crée une fois ce lien depuis la racine :

```
mkdir -p .claude && ln -s ../skills .claude/skills
```
