# Potagerie

Une appli installée pour planifier son potager : un seul utilisateur, sans compte ni serveur. Le cœur est en Rust, l'interface en Svelte 5 + TypeScript, le tout emballé par Tauri v2 (macOS et Windows au lot 1).

La documentation (specs, designs, prototypes) est dans [`docs/`](docs/POT-00_index.md). La référence technique est [S05](docs/specs/POT-S05_spec-technique_v0.9.md).

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

Les compilations ne sont pas signées : au premier lancement, clic droit puis Ouvrir sur macOS, « Exécuter quand même » sur Windows.

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
