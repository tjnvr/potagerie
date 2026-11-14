# POT-S05 · Spec technique — v0.6

> Version : 0.6 (brouillon) · Date : 2026-11-14 · Auteur : Tim
> Références : [S01](POT-S01_spec-fonctionnelle_v1.3.md) · [S02](POT-S02_catalogue-cultures_v0.2.md) · [S03](POT-S03_regles-calculs_v0.5.md) · [P01](../prototypes/POT-P01_rendu-iso_v0.1/README.md) · [D01](../design/POT-D01_ecrans-cles_v1.1/README.md)
> La v0.4 a introduit Rust + Tauri, la v0.5 a aligné le modèle sur S01 et S03. La v0.6 remplace htmx et le routeur axum par une interface Svelte + TypeScript qui appelle des commandes Tauri (voir 11).

## 1. Principes

- **Une appli installée, locale d'abord.** Pas d'hébergement ni de compte : un seul utilisateur, et ses données restent sur ses appareils.

  | Mode | Lancement | Stockage | Lot |
  |---|---|---|---|
  | **Développement** | `cargo tauri dev` : l'appli s'ouvre et l'interface se recharge à chaud (serveur Vite) | SQLite dans le dossier de données de l'appli | — |
  | **Bureau** | `Potagerie.app` sur macOS, `Potagerie.exe` sur Windows | SQLite dans le dossier de données de l'appli | 1 |
  | **Mobile** | Appli iOS et Android | SQLite dans le dossier de données de l'appli | 2 |
  | **Synchro** | Les appareils échangent leurs modifications, fusionnées sans conflit | Document CRDT | 3 |

  Tous les modes partagent la même interface et les mêmes commandes Rust. Passer de l'un à l'autre change seulement la cible de compilation.
- **Svelte + TypeScript pour l'interface, Rust pour le reste.** L'interface est une application Svelte compilée par Vite et embarquée dans l'appli. Elle appelle le cœur Rust par des commandes Tauri (`invoke`), dont les types TypeScript sont générés depuis Rust.
- **Le métier vit en Rust.** Les calculs de dates, les stades, les conflits, les créneaux libres, les suggestions (S03) et les règles de cohérence sont écrits en Rust et testés unitairement (`cargo test`). L'interface affiche ce que Rust renvoie et ne refait aucun de ces calculs : la seule logique côté interface est l'animation et le glisser-déposer.
- **Le modèle est prêt pour la fusion dès le lot 1** (voir 5), pour que le passage au CRDT du lot 3 ne demande pas de tout réécrire.

## 2. Architecture

```
webview ── interface Svelte + TypeScript (compilée par Vite, embarquée)
   │       diorama et frise en SVG, animation, glisser-déposer
   │       invoke() typé (bindings.ts généré par tauri-specta)
appli Tauri `potagerie` (src-tauri)
   ├── commandes  une fonction Rust par action, renvoie la vue de l'année
   ├── plan       modèle, règles, calculs de dates (S03), vue, validation
   ├── catalogue  cultures.json (S02), embarqué
   ├── export     PNG et PDF de la frise, JSON du potager
   └── store      trait ── sqlite (lots 1 et 2) │ crdt (lot 3)
```

- **Pas de serveur ni de port réseau** : la webview charge l'interface embarquée et parle au Rust uniquement par les commandes Tauri.
- **Tout est embarqué dans l'appli** : l'interface compilée (HTML, CSS, JS), les traductions et le catalogue.
- **Tout se compile avec la chaîne Rust standard** et Node.js pour l'interface. Aucun cgo, aucun sidecar.

## 3. Appli Tauri

| Sujet | Choix |
|---|---|
| Emballage | Tauri v2 : fenêtre native qui affiche l'interface dans le moteur web du système (WebKit sur macOS et iOS, WebView2 sur Windows, WebView sur Android) |
| Branchement | L'interface appelle les commandes déclarées dans `src-tauri` ; chaque commande renvoie un `Result` typé, et l'erreur porte un code traduit par l'interface |
| Sécurité | Les capacités Tauri n'autorisent que les commandes de l'appli et les fenêtres « Ouvrir » et « Enregistrer sous » du plugin dialog. CSP stricte : aucune ressource externe |
| Données | `potagerie.db` dans le dossier de données de l'appli fourni par Tauri (`app_data_dir`) |
| Exports | L'interface ouvre la fenêtre « Enregistrer sous » du plugin dialog, puis passe le chemin choisi à la commande d'export, qui écrit le fichier (PNG, PDF, JSON) |
| Compilation bureau | Version macOS universelle (Apple Silicon et Intel) sur un Mac ; version Windows sur un runner GitHub Actions Windows |
| Compilation mobile (lot 2) | iOS sur un Mac avec Xcode ; Android avec le SDK et le NDK Android |
| Prérequis | Windows 10 ou 11 avec le runtime WebView2, présent par défaut sur les versions à jour |
| Signature bureau | Aucune en V1 (usage personnel) : au premier lancement, clic droit puis Ouvrir sur macOS, « Exécuter quand même » sur Windows |
| Installation mobile (lot 2) | Android : APK installé à la main. iOS : voir 10 |
| Mises à jour | Manuelles en V1 : on remplace l'appli. Le plugin updater de Tauri reste possible plus tard |

Le diorama, l'animation et le glisser-déposer doivent être vérifiés sous WebKit (macOS et iOS), WebView2 et la WebView Android.

## 4. Interface

| Sujet | Choix |
|---|---|
| Framework | Svelte 5 + TypeScript, compilé par Vite en application d'une seule page (deux onglets, Potager et Plan, sans routeur) |
| Liaison avec Rust | `tauri-specta` génère `bindings.ts` (fonctions et types) à partir des commandes et des structures Rust : un changement de modèle casse la compilation TypeScript au lieu de casser l'appli |
| État | L'interface garde la dernière vue de l'année renvoyée par Rust. Chaque action appelle une commande, qui enregistre et renvoie la vue mise à jour ; l'interface la remplace entièrement |
| Diorama | Composants Svelte qui produisent du SVG : une forme `<symbol>` par légume et par stade, une balise `<use>` par plante (P01) |
| Jouer la saison | Composant Svelte animé par `requestAnimationFrame` : il fait avancer les stades à partir des dates fournies par la vue, sans appeler Rust pendant la lecture |
| Atelier du plan | Glisser en Pointer Events (souris et tactile), appui long pour saisir une barre sur téléphone (S01, F-VIS-2). L'élément suit le doigt pendant le geste ; au lâcher, la commande valide et enregistre la position. Si Rust refuse (conflit, S03 §2), l'élément revient à sa place et le motif s'affiche |
| Frise des cultures | Composant Svelte en SVG ; ajouter, déplacer ou annoter passe par les commandes |
| Export de la frise | L'interface sérialise le SVG de la frise ; Rust le convertit en PNG (resvg) ou en PDF (svg2pdf), avec les polices embarquées dans l'appli, puis l'écrit au chemin choisi |
| Traduction | Tous les textes de l'interface viennent de fichiers de messages (`front/messages/fr.json`), compilés et typés (Paraglide JS). Les noms de cultures et les codes d'erreur renvoyés par Rust sont des clés, traduites par l'interface |
| Mise en page | Ordinateur et téléphone selon D01. La mise en page s'adapte à la largeur dès le lot 1, pour que le lot 2 ne refasse pas l'interface |
| Qualité | `svelte-check` et `tsc` pour les types, Vitest pour les composants qui ont de la logique (animation, glisser) |

Renvoyer la vue complète de l'année après chaque action garde une seule source de vérité, Rust, et évite de recopier les règles de S03 en TypeScript. Une vue d'année reste petite (10 planches au plus).

## 5. Modèle de données

Un **plan** par utilisateur, stocké comme **un seul document**. Chaque action le charge, le modifie, puis l'enregistre.

```
Plan
├── id, version, mis_a_jour_le
├── terrain : longueur_cm, largeur_cm, case_cm (25|50|100, fixée à la création), allee_cm
├── planches{id} : nom
│   └── dispositions{annee} : absente?, type (planche|carre|butte|bac),
│                             x, y, longueur_cm, largeur_cm
│                             ← seulement les années où la planche change (S03, §7)
├── series{id} : culture_id, planche_id, troncon_debut, troncon_fin,
│                mode (abri|plants|pleine_terre), date (semis ou plantation, S03 §2),
│                abri_jours, apres_jours, duree_jours (copiés du catalogue à la création, S03 §2),
│                fin_manuelle?
└── annotations{id} : type (note|periode), date_debut, date_fin?,
                      planche_id? (vide = tout le potager), texte, masquee
```

- Au maximum 10 planches par année.
- La longueur d'une planche est un nombre entier de cases ; sa largeur est libre au centimètre. Un tronçon fait la taille d'une case (S03, §4).
- Les années passées sont en lecture seule.
- Une série référence une planche par son `id`, qui reste stable d'une année à l'autre.

L'appli vérifie toutes ces règles. Les dates calculées (repiquage, mise en place, récolte, fin non modifiée à la main) ne sont pas stockées : elles sont recalculées à chaque vue à partir des durées copiées dans la série. Une mise à jour du catalogue ne décale donc aucune série.

**Vue de l'année** : c'est ce que les commandes renvoient à l'interface. Elle est calculée par `plan` à partir du document et de la date du jour, et n'est jamais stockée. Elle contient les planches disposées, les séries avec leurs dates et stades calculés, les conflits, les créneaux libres et leurs suggestions (S03), les annotations et les tâches.

**Règles pour préparer la fusion (lot 3)** :
- Les collections sont des **maps indexées par `id`**, jamais des listes ordonnées : deux appareils peuvent ajouter chacun un élément sans se gêner.
- Les `id` sont générés sur l'appareil (UUID v7), jamais attribués par un compteur.
- Aucune donnée dérivée n'est stockée.
- Le champ `version` sert aux lots 1 et 2 seulement ; il disparaît avec le CRDT.

**Stockage** : aux lots 1 et 2, une ligne SQLite contient le document sérialisé en JSON (serde). Au lot 3, elle contient le document CRDT.

## 6. Synchro (lot 3)

**Avant le lot 3**, chaque appareil a ses propres données. Le transfert se fait par l'export JSON et son import, qui remplace le potager existant après confirmation (S01, F-CPT-3), en prévenant si le plan reçu est plus ancien que celui de l'appareil (`version`, `mis_a_jour_le`).

**Au lot 3**, le document devient un CRDT :
- Chaque appareil garde sa copie, fonctionne sans réseau, et échange ses modifications avec les autres.
- Les ajouts faits en même temps sur deux appareils sont tous conservés. Sur un même champ (position, nom), la dernière écriture gagne.
- **Validation après fusion** : un document fusionné converge, mais il peut enfreindre les règles du plan. Les cas sont spécifiés dans S03, §13 :

  | Cas | Exemple |
  |---|---|
  | Trop de planches | Une planche ajoutée hors ligne sur chaque appareil : 11 planches sur 10 |
  | Série orpheline | Planche supprimée sur un appareil, série ajoutée dessus sur l'autre |
  | Chevauchement | Deux séries sur les mêmes tronçons aux mêmes dates |
  | Année passée modifiée | Modification hors ligne d'une année devenue passée entre-temps |

**Transports**, cumulables :

| Transport | Fonctionnement |
|---|---|
| QR code sur le Wi-Fi | Un appareil ouvre un serveur temporaire sur le réseau local et affiche un QR code ; l'autre le scanne et les deux échangent leurs modifications. Jeton à usage unique, expiration après 5 minutes |
| Dossier cloud | Chaque appareil écrit uniquement son propre fichier de modifications (`<appareil>.changes`) dans un dossier partagé (iCloud Drive, Google Drive, Dropbox), et lit ceux des autres. Personne n'écrit dans le même fichier |

## 7. Commandes

| Commande | Rôle | Renvoie |
|---|---|---|
| `catalogue()` | Les cultures de S02 | Liste des cultures |
| `vue_annee(annee)` | Ouvre une année (onglets Potager et Plan) | Vue de l'année |
| `creer_terrain`, `modifier_terrain` | Dimensions du terrain et allées | Vue de l'année |
| `creer_planche`, `modifier_planche`, `deplacer_planche`, `supprimer_planche` | Atelier du plan (S03, §6 et §7) | Vue de l'année |
| `creer_serie`, `modifier_serie`, `supprimer_serie` | Frise des cultures | Vue de l'année |
| `creer_annotation`, `modifier_annotation`, `supprimer_annotation` | Notes et périodes | Vue de l'année |
| `exporter_frise(svg, format, chemin)` | Export PNG ou PDF de la frise | Rien, ou une erreur |
| `exporter_potager(chemin)` | Export complet en JSON (F-CPT-3) | Rien, ou une erreur |
| `lire_import(chemin)` | Lit un fichier d'import et le compare au plan de l'appareil, sans rien remplacer | Résumé (date, version, plus ancien ou non) |
| `importer_potager(chemin)` | Remplace le potager, après confirmation (F-CPT-3) | Vue de l'année |
| `effacer_donnees()` | Efface toutes les données de l'appli, après confirmation (F-CPT-4) | Rien |

Le lot 3 ajoute les commandes de synchro, à détailler avec le lot.

**Écritures** : aux lots 1 et 2, chaque appareil écrit seul dans sa base et chaque commande qui modifie le plan incrémente `version`. Au lot 3, les modifications sont fusionnées (6).

**Erreurs** : une commande refusée (conflit, planche de trop, année passée) renvoie un code d'erreur et ne modifie rien.

## 8. Arborescence du dépôt

```
Cargo.toml              workspace
crates/plan/            modèle, règles, calculs, vue de l'année, validation
crates/catalogue/       cultures.json + chargement
crates/export/          PNG et PDF de la frise (resvg, svg2pdf), JSON du potager
crates/store/           trait + sqlite (+ crdt au lot 3)
src-tauri/              appli Tauri : commandes, capacités, génération de bindings.ts
front/                  interface Svelte + TypeScript (Vite)
  src/lib/diorama/      diorama SVG et lecture de la saison
  src/lib/frise/        frise des cultures
  src/lib/atelier/      atelier du plan, glisser-déposer
  src/lib/bindings.ts   généré par tauri-specta, jamais modifié à la main
  messages/             traductions
docs/                   specs, designs, prototypes
Makefile                build, test, run
```

## 9. Lots

| Lot | Contenu | Sortie |
|---|---|---|
| **1 · Bureau** | Interface Svelte, diorama, frise, atelier, export de la frise (PDF, PNG), export et import du potager, SQLite, mise en page adaptable, appli Tauri bureau | `Potagerie.app` et `Potagerie.exe` utilisables au quotidien |
| **2 · Mobile** | Appli Tauri iOS et Android, mise en page téléphone (D01), transfert par export et import JSON | Le potager consultable et modifiable sur téléphone |
| **3 · Synchro** | Document CRDT, validation après fusion, transports QR code et dossier cloud | Le même potager sur tous les appareils, sans serveur |

## 10. Hypothèses à valider

| Sujet | Hypothèse par défaut | Alternative |
|---|---|---|
| Framework | Svelte 5 + Vite, sans SvelteKit | SvelteKit en mode statique (`adapter-static`), si un routeur devient utile |
| Types partagés | tauri-specta (encore en version candidate pour Tauri v2) | ts-rs pour les types et des appels `invoke` écrits à la main |
| Diorama | SVG produit par Svelte, fluide sous WebKit, WebView2 et Android pendant la lecture et le glisser : à valider par un prototype (P02) | Hybride Canvas + couche SVG, si le téléphone saccade (P01) |
| Export de la frise | resvg et svg2pdf côté Rust | Rendu dans la webview (canvas pour le PNG, bibliothèque JS pour le PDF) |
| Traduction | Paraglide JS | svelte-i18n |
| Gestionnaire de paquets | pnpm | npm |
| SQLite | rusqlite, SQLite embarqué | sqlx |
| Dates | chrono | jiff |
| CRDT (lot 3) | Automerge + autosurgeon (struct Rust adossée au document) | Loro |
| Après fusion (lot 3) | Rien n'est supprimé automatiquement : les incohérences s'affichent à résoudre | Réparation automatique (par exemple, suppression des séries orphelines) |
| Relais (lot 3) | Aucun : QR code et dossier cloud suffisent | Relais samod auto-hébergé (Raspberry Pi), si les deux transports ne suffisent pas |
| iOS (lot 2) | Compte Apple Developer (99 $ par an) pour installer durablement sur iPhone | Compte gratuit : réinstallation depuis Xcode tous les 7 jours ; ou Android seul |
| Signature (bureau) | Aucune en V1 | Compte Apple Developer et certificat de signature Windows, si l'appli est distribuée à d'autres |
| potagerie.garden | N'héberge plus l'appli | Page de présentation et téléchargements |

Le prototype P02 répond à une question : **une appli Tauri + Svelte affiche-t-elle et anime-t-elle le diorama SVG de D01 de façon fluide sur macOS et Windows, et l'export resvg de la frise est-il fidèle à l'écran ?**

## 11. Changements par rapport à la v0.5

- htmx, askama et le routeur axum sont retirés. L'interface est en Svelte + TypeScript et appelle des commandes Tauri typées (1, 2, 4).
- Le mode développement passe de `localhost:8080` à `cargo tauri dev`. Le crate `serve` et la route `/healthz` disparaissent.
- Les routes deviennent des commandes (7), qui renvoient la vue de l'année au lieu de fragments HTML. `lire_import` et `effacer_donnees` sont ajoutées pour F-CPT-3 et F-CPT-4.
- Le crate `rendu` disparaît : le SVG est produit par les composants Svelte, la vue de l'année est calculée par `plan` (5), et les exports passent dans un crate `export` (8).
- L'export PNG et PDF de la frise se fait en Rust à partir du SVG de l'interface (4).
- La traduction a désormais un mécanisme défini (4).
- Les références à D01 pointent vers la v1.1.
