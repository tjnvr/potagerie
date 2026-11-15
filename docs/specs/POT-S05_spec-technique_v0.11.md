# POT-S05 · Spec technique — v0.11

> Version : 0.11 (brouillon) · Date : 2026-11-15 · Auteur : Tim
> Références : [S01](POT-S01_spec-fonctionnelle_v1.4.md) · [S02](POT-S02_catalogue-cultures_v0.3.md) · [S03](POT-S03_regles-calculs_v0.10.md) · [P01](../prototypes/POT-P01_rendu-iso_v0.1/README.md) · [D01](../design/POT-D01_ecrans-cles_v1.1/README.md) · [D02](../design/POT-D02_ecrans-complementaires_v0.1/README.md)
> La v0.4 a introduit Rust + Tauri, la v0.5 a aligné le modèle sur S01 et S03. La v0.6 a remplacé htmx par une interface Svelte + TypeScript qui appelle des commandes Tauri. La v0.7 reportait ce que le squelette de l'appli a tranché : liaison typée, erreurs, sécurité, polices, format d'export. La v0.8 reporte ce que l'atelier a tranché : commande `series_depassant`, champs de la vue, codes d'erreur de placement (voir 11). La v0.9 reporte ce que la frise, le formulaire de série et le catalogue ont tranché : aperçu de série, décalage d'une barre, cultures de saison, SVG autonome de la frise (voir 13). La v0.10 reporte ce que les réglages ont tranché : notes et périodes, validation d'un plan importé, `vider_plan`, états d'erreur de fichier (voir 14). La v0.11 reporte ce que l'export de la frise et les annotations ont tranché : écran E11, rendu des notes, commande `incoherences_plan` (voir 15).

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
| Sécurité | Les commandes de l'appli sont déclarées dans le manifeste de l'appli (`build.rs`) : comme celles des plugins, chacune doit être autorisée par une capacité. La seule capacité, `main`, accorde à la fenêtre principale les commandes de l'appli et les fenêtres « Ouvrir » et « Enregistrer sous » du plugin dialog, rien d'autre ; un test vérifie que chaque commande est enregistrée et autorisée. CSP stricte : `'self'` partout, `ipc:` pour les commandes, aucune ressource externe, aucun style en ligne. Une CSP de développement (`devCsp`) ajoute seulement le rechargement à chaud de Vite |
| Données | `potagerie.db` dans le dossier de données de l'appli fourni par Tauri (`app_data_dir`), avec l'identifiant `garden.potagerie` |
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
| Liaison avec Rust | `tauri-specta` génère `bindings.ts` (fonctions et types) à partir des commandes et des structures Rust : un changement de modèle casse la compilation TypeScript au lieu de casser l'appli. Le fichier est écrit au lancement en mode debug et par `make bindings` ; il est commité, et un test Rust échoue s'il ne correspond plus au code. Chaque commande renvoie `{ status: "ok", data }` ou `{ status: "error", error }` (mode `Result` de tauri-specta), avec un seul type TypeScript par type Rust. Les entiers exposés tiennent dans un nombre JavaScript (`u32` au plus) |
| État | L'interface garde la dernière vue de l'année renvoyée par Rust. Chaque action appelle une commande, qui enregistre et renvoie la vue mise à jour ; l'interface la remplace entièrement |
| Diorama | Composants Svelte qui produisent du SVG : une forme `<symbol>` par légume et par stade, une balise `<use>` par plante (P01) |
| Jouer la saison | Composant Svelte animé par `requestAnimationFrame` : il fait avancer les stades à partir des dates fournies par la vue, sans appeler Rust pendant la lecture |
| Atelier du plan | Glisser en Pointer Events (souris et tactile), appui long pour saisir une barre sur téléphone (S01, F-VIS-2). L'élément suit le doigt pendant le geste ; au lâcher, la commande valide et enregistre la position. Si Rust refuse (conflit, S03 §2), l'élément revient à sa place et le motif s'affiche |
| Frise des cultures | Composant Svelte en SVG (`FriseSvg`), une seule image : mois, une ligne par planche, barres par phases (godets hachurés, croissance pointillée, récolte pleine : le motif distingue autant que la couleur), créneaux en pointillés. Elle charge la vue des années visibles (`vue_annee`) et fusionne les séries. Défilement et zoom (molette avec Ctrl, pincement) ; ouvre sur 12 mois, 6 mois sous 560 px. La position dans le temps est le store `front/src/lib/temps.ts` (date du curseur), partagé avec le diorama |
| Export de la frise | L'interface sérialise le SVG de la frise (`serialiserFrise`, `front/src/lib/frise/exportSvg.ts`) : un SVG autonome, sans CSS, sans style, sans variable ni ressource externe, avec les noms de polices `Figtree` et `Fredoka`, validé par un test de `crates/export` ; Rust le convertit en PNG (resvg, à deux fois la taille) ou en PDF (svg2pdf), avec les polices embarquées dans l'appli, puis l'écrit au chemin choisi |
| Polices | Figtree et Fredoka (D01), en TTF sous licence OFL, dans `crates/export/fonts/`. Le même fichier sert à l'export (embarqué dans le binaire) et à l'interface (`@font-face`, copié par Vite) : la CSP interdit les polices en ligne, et l'export ressemble ainsi à l'écran |
| Traduction | Tous les textes de l'interface viennent de fichiers de messages (`front/messages/fr.json`), compilés et typés (Paraglide JS, projet `front/project.inlang`, plugin chargé depuis `node_modules` pour compiler sans réseau). Les noms de cultures (`culture_<id>`), les codes d'erreur (`erreur_<code>`) et les noms par défaut des planches (« Planche A ») sont produits par l'interface ; `catalogue()` n'envoie pas de nom |
| Mise en page | Ordinateur et téléphone selon D01. La mise en page s'adapte à la largeur dès le lot 1, pour que le lot 2 ne refasse pas l'interface |
| Qualité | `svelte-check` (en échec sur avertissement) pour les types, Vitest pour la logique de l'interface (animation, glisser, aides de placement) |
| Enregistrement | L'indicateur de l'en-tête (F-CPT-2) passe à « Enregistrement… » pendant une commande. Une erreur `enregistrement` le met à « Erreur d'enregistrement » avec « Réessayer », qui relance la même commande ; les autres refus s'affichent dans un message, sans toucher à l'indicateur |

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

**Vue de l'année** : c'est ce que les commandes renvoient à l'interface. Elle est calculée par `plan` (`YearView::build`) à partir du document, du catalogue et de la date du jour, et n'est jamais stockée. Elle contient :
- l'année, la date du jour et l'indication « lecture seule » pour une année passée ;
- le terrain, ou rien tant qu'il n'est pas créé (premier lancement, E2) ;
- `version` et `mis_a_jour_le` du plan ;
- les planches qui existent cette année-là, avec leur disposition de l'année et leur nombre de tronçons ;
- les séries dont la période, semis à l'abri compris, touche l'année, avec leurs dates et leur stade du jour calculés (S03 §2, §3) ;
- les conflits entre ces séries, les créneaux libres de l'année et leurs suggestions (S03 §5, §8, §9) ;
- pour chaque série, ses jours en godets (`pot_days`) ;
- pour chaque planche, les séries supprimées si on la supprime à partir de cette année (`series_supprimees`) ; pour chaque série, ses rangs et pieds par rang (S03 §6) ;
- l'historique : les séries terminées au plus tard le 1er janvier de l'année (panneau planche, lecture seule) ;
- les annotations qui touchent l'année, et les tâches datées de l'année (S03 §11).

La vue n'ajoute aucune règle : elle assemble les calculs de `plan`.

**Règles pour préparer la fusion (lot 3)** :
- Les collections sont des **maps indexées par `id`**, jamais des listes ordonnées : deux appareils peuvent ajouter chacun un élément sans se gêner.
- Les `id` sont générés sur l'appareil (UUID v7), jamais attribués par un compteur.
- Aucune donnée dérivée n'est stockée.
- Le champ `version` sert aux lots 1 et 2 seulement ; il disparaît avec le CRDT.

**Stockage** : aux lots 1 et 2, une ligne SQLite contient le document sérialisé en JSON (serde). Au lot 3, elle contient le document CRDT. Le crate `store` expose un trait `Store` (`load`, `save`, `replace`, `clear`) :
- `save` incrémente `version`, met à jour `mis_a_jour_le` et refuse d'écrire un plan dont la `version` n'est pas celle de la base (code `version_perimee`) : une copie périmée n'écrase jamais une plus récente ;
- le premier enregistrement, à la création du terrain, insère le plan en version 1 ;
- `replace` (import) écrit le plan reçu tel quel, avec sa `version` et sa date ;
- le schéma est versionné par `PRAGMA user_version` et des migrations numérotées, jamais modifiées une fois publiées ; une base écrite par une version plus récente de l'appli est refusée (`base_trop_recente`).

## 6. Synchro (lot 3)

**Avant le lot 3**, chaque appareil a ses propres données. Le transfert se fait par l'export JSON et son import, qui remplace le potager existant après confirmation (S01, F-CPT-3), en prévenant si le plan reçu est plus ancien que celui de l'appareil. « Plus ancien » compare `mis_a_jour_le` : `version` compte les écritures d'un appareil, et ne se compare pas d'un appareil à l'autre.

Le fichier d'export est une enveloppe JSON `{ "format": "potagerie", "format_version": 1, "plan": { … } }`. Un fichier d'un autre format est refusé (`import_invalide`), un `format_version` plus récent que l'appli aussi (`import_trop_recent`). Le plan reçu est ensuite vérifié (S03 §13) : refus (`import_invalide`) si les calculs ne peuvent pas le lire ; sinon il est accepté tel quel, avec la liste des incohérences à résoudre.

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

Les commandes qui modifient le plan prennent `annee`, l'année affichée, et renvoient sa vue. Dans `bindings.ts`, leurs noms passent en camelCase (`vueAnnee`).

| Commande | Rôle | Renvoie |
|---|---|---|
| `catalogue()` | Les cultures de S02, sans le nom (traduit par l'interface) | Liste des cultures |
| `vue_annee(annee)` | Ouvre une année (onglets Potager et Plan) | Vue de l'année |
| `creer_terrain(annee, terrain)` | Crée le terrain, et avec lui le plan (E2) : dimensions, case, allée | Vue de l'année |
| `modifier_terrain(annee, terrain)` | Dimensions et allée ; la case ne change plus | Vue de l'année |
| `creer_planche(annee, type, nom, x, y)` | Lâche une planche du bac à sa taille par défaut (S01 F-PLN-1) ; le nom par défaut vient de l'interface | Vue de l'année |
| `series_depassant(annee, id, longueur_cm)` | Série qui ne tiendraient plus si la planche était raccourcie (S03 §4) : lecture seule, appelée avant `modifier_planche` pour avertir | Liste d'identifiants de séries |
| `modifier_planche(annee, id, nom?, geometrie?)`, `deplacer_planche(annee, id, x, y)`, `supprimer_planche(annee, id)` | Atelier du plan (S03, §6 et §7) | Vue de l'année |
| `creer_serie(annee, serie)`, `modifier_serie(annee, id, serie)`, `supprimer_serie(annee, id)` | Frise des cultures ; les durées ne sont recopiées du catalogue qu'à la création ou si la culture ou le mode change (S03 §2). Refus : `conflict`, `zone_hors_planche`, `planche_absente`, `fin_manuelle_invalide`, `pot_days_required`, `occupation_vide`. `serie.date_est_repiquage` : la date saisie est un repiquage, Rust enregistre le semis `s = x − A` | Vue de l'année |
| `decaler_serie(annee, id, jours)` | Glisser une barre de `jours` jours : la date saisie et la fin manuelle se décalent (S03 §2, T12). Refusée avec `conflict` si la série décalée n'est plus valide | Vue de l'année |
| `apercu_serie(id?, serie)` | Lecture seule, appelée à chaque saisie du formulaire E07 : dates et quantités (`dates`, absentes si la série ne peut pas être construite), problèmes à afficher (code, tronçons et période en commun), zones des raccourcis Toute, ½, ⅓, ¼ (`null` : grisé) et tronçons pris pendant la période. L'interface ne calcule rien | Aperçu |
| `cultures_de_saison(date)` | Cultures dont la fenêtre de mise en place contient la date (S03 §10) : « de saison maintenant » | Liste d'identifiants |
| `creer_annotation(annee, annotation)`, `modifier_annotation(annee, id, annotation)`, `supprimer_annotation(annee, id)` | Notes et périodes (S03 §16) : année passée refusée, planche inconnue, texte de 1 à 80 caractères, fin d'une période après son début. Masquer de la frise est `modifier_annotation` avec `masquee: true` | Vue de l'année |
| `exporter_frise(svg, format, chemin)` | Export PNG ou PDF de la frise | Rien, ou une erreur |
| `exporter_potager(chemin)` | Export complet en JSON (F-CPT-3) | Rien, ou une erreur |
| `lire_import(chemin)` | Lit un fichier d'import et le compare au plan de l'appareil, sans rien remplacer | Résumé : date, version, nombre de planches et de séries, plus ancien ou non, remplace un potager ou non, et `incoherences` : la liste de ce qui enfreint les règles (S03 §13). Un fichier illisible ou invalide est refusé (`import_invalide`, `import_trop_recent`, `fichier`) |
| `incoherences_plan()` | Recalcule ce qui enfreint les règles du plan de l'appareil (S03 §13), pour la liste « à résoudre » qui reste après un import | La liste `incoherences`, vide sans plan ou quand tout est réglé |
| `importer_potager(annee, chemin)` | Remplace le potager, après confirmation (F-CPT-3). Mêmes vérifications que `lire_import` ; les incohérences sont gardées, rien n'est supprimé ni corrigé | Vue de l'année |
| `vider_plan(annee)` | « Vider le plan » (F-PLN-4) : supprime de `annee` et des suivantes toutes les planches de l'année, avec leurs séries (comme `supprimer_planche`) et leurs annotations qui se terminent à partir du 1er janvier. Garde le terrain, les années passées et les annotations de tout le potager. Refusée sur une année passée ; sans planche, ne fait rien | Vue de l'année |
| `effacer_donnees()` | Efface toutes les données de l'appli, après double confirmation (F-CPT-4) | Rien |

Le lot 3 ajoute les commandes de synchro, à détailler avec le lot.

**Manques connus**, à spécifier avec l'atelier :
- « Annuler » et « Rétablir » (F-PLN-6) n'ont pas de commande : il reste à choisir entre un historique côté Rust (les documents précédents) et des opérations inverses.

**Écritures** : aux lots 1 et 2, chaque appareil écrit seul dans sa base et chaque commande qui modifie le plan incrémente `version` (5). Au lot 3, les modifications sont fusionnées (6).

**Erreurs** : une commande refusée (conflit, planche de trop, année passée) renvoie une erreur et ne modifie rien. L'erreur est `{ code, detail }` : `code` est une valeur fixe, traduite par l'interface (`erreur_<code>` dans `fr.json`) ; `detail` est un message technique, pour le journal, jamais affiché. Les codes viennent :
- de `plan` (`PlanError::code`) : `annee_passee`, `too_many_planches`, `conflict`, `zone_hors_planche`… ;
- de `plan` pour le placement (S03 §15) : `terrain_invalide`, `allee_invalide`, `largeur_invalide`, `nom_invalide`, `planche_hors_terrain`, `planche_chevauche`, `allee_trop_etroite` ;
- de l'appli : `terrain_absent`, `terrain_existant`, `case_invalide` ;
- de `plan` pour les annotations (S03 §16) : `annotation_inconnue`, `texte_invalide`, `annotation_dates_invalides` (avec `annee_passee` et `planche_inconnue`) ;
- du stockage : `enregistrement` (lecture ou écriture impossible : seul cas où l'interface propose « Réessayer »), `version_perimee`, `base_trop_recente` ;
- des fichiers : `fichier`, `import_invalide`, `import_trop_recent`, `svg_invalide`, `rendu_impossible`.

La liste complète est le type `ErrorCode` de `bindings.ts` ; un code sans traduction casse `pnpm check`.

## 8. Arborescence du dépôt

```
Cargo.toml              workspace
crates/plan/            modèle, règles, calculs, vue de l'année, validation
crates/catalogue/       cultures.json + chargement
crates/export/          PNG et PDF de la frise (resvg, svg2pdf), JSON du potager
  fonts/                polices de l'export et de l'interface (OFL)
crates/store/           trait + sqlite (+ crdt au lot 3)
src-tauri/              appli Tauri : commandes, erreurs, génération de bindings.ts
  capabilities/         capacité `main` de la fenêtre principale
  src/command_names.rs  liste des commandes, partagée avec build.rs (permissions)
front/                  interface Svelte + TypeScript (Vite, pnpm)
  src/lib/coquille/     en-tête, onglets, sélecteur d'année, boîtes de confirmation
  src/lib/diorama/      onglet Potager : diorama SVG et lecture de la saison
  src/lib/frise/        frise des cultures, SVG autonome, glisser de barre
  src/lib/serie/        formulaire de série (E07), catalogue (E09)
  src/lib/temps.ts      position dans le temps, partagée (curseur)
  src/lib/atelier/      onglet Plan : atelier du plan, glisser-déposer
  src/lib/reglages/     réglages, export et import, vider le plan, liste des incohérences (E10)
  src/lib/notes/        formulaire de note ou de période, autonome (frise à venir)
  src/lib/bindings.ts   généré par tauri-specta, jamais modifié à la main
  messages/             traductions
  project.inlang/       configuration de Paraglide
docs/                   specs, designs, prototypes
Makefile                dev, build, test, check, fmt, bindings
.github/workflows/      CI : vérifications sur Ubuntu, compilation macOS et Windows
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
| Framework | **Validé** (P02, squelette) : Svelte 5 + Vite, sans SvelteKit | SvelteKit en mode statique (`adapter-static`), si un routeur devient utile |
| Types partagés | **Validé** (P02, squelette) : tauri-specta `=2.0.0-rc.29` et specta `=2.0.0-rc.28`, versions exactes tant qu'ils sont en version candidate | ts-rs pour les types et des appels `invoke` écrits à la main |
| Diorama | SVG produit par Svelte, fluide sous WebKit, WebView2 et Android pendant la lecture et le glisser : à valider par un prototype (P02) | Hybride Canvas + couche SVG, si le téléphone saccade (P01) |
| Export de la frise | **Validé** (P02, squelette) : resvg et svg2pdf côté Rust. resvg reste en 0.45 tant que svg2pdf 0.13 dépend de usvg 0.45 : les deux doivent partager le même arbre SVG | Rendu dans la webview (canvas pour le PNG, bibliothèque JS pour le PDF) |
| Traduction | **Validé** (squelette) : Paraglide JS 3 | svelte-i18n |
| Gestionnaire de paquets | **Validé** : pnpm, version fixée par `packageManager` dans `front/package.json` | npm |
| SQLite | **Validé** (squelette) : rusqlite, SQLite embarqué (`bundled`) | sqlx |
| Dates | chrono | jiff |
| CRDT (lot 3) | Automerge + autosurgeon (struct Rust adossée au document) | Loro |
| Après fusion (lot 3) | Rien n'est supprimé automatiquement : les incohérences s'affichent à résoudre | Réparation automatique (par exemple, suppression des séries orphelines) |
| Relais (lot 3) | Aucun : QR code et dossier cloud suffisent | Relais samod auto-hébergé (Raspberry Pi), si les deux transports ne suffisent pas |
| iOS (lot 2) | Compte Apple Developer (99 $ par an) pour installer durablement sur iPhone | Compte gratuit : réinstallation depuis Xcode tous les 7 jours ; ou Android seul |
| Signature (bureau) | Aucune en V1 | Compte Apple Developer et certificat de signature Windows, si l'appli est distribuée à d'autres |
| potagerie.garden | N'héberge plus l'appli | Page de présentation et téléchargements |

Le prototype P02 répond à une question : **une appli Tauri + Svelte affiche-t-elle et anime-t-elle le diorama SVG de D01 de façon fluide sur macOS et Windows, et l'export resvg de la frise est-il fidèle à l'écran ?**

## 11. Changements par rapport à la v0.6

Le squelette de l'appli (branche `feat/app-skeleton`) a tranché les points suivants :
- **Liaison** (4) : versions exactes de tauri-specta et specta, mode `Result` (`{ status, data | error }`), un type TypeScript par type Rust, entiers limités à `u32` dans ce qui est exposé. `bindings.ts` est commité et un test vérifie qu'il est à jour.
- **Erreurs** (7) : forme `{ code, detail }`, origine et liste des codes, et `enregistrement` comme seul cas réessayable (F-CPT-2).
- **Sécurité** (3) : les commandes de l'appli passent elles aussi par les permissions, avec une seule capacité `main` ; détail de la CSP et CSP de développement.
- **Polices** (4) : Figtree et Fredoka sont embarquées dans `crates/export/fonts/`, et partagées entre l'export et l'interface, puisque la CSP interdit Google Fonts.
- **Vue de l'année** (5) : contenu détaillé ; `YearView::build` dans `plan`.
- **Stockage** (5) : trait `Store`, refus d'une copie périmée (`version_perimee`), schéma versionné par `user_version`.
- **Export et import** (6) : enveloppe `format` / `format_version` ; « plus ancien » compare `mis_a_jour_le`, pas `version`.
- **Commandes** (7) : paramètres précisés, `annee` sur les commandes qui renvoient la vue, nom par défaut des planches fourni par l'interface. « Vider le plan » et « Annuler / Rétablir » sont signalés comme manques.
- **Arborescence** (8) : `capabilities/`, `command_names.rs`, `fonts/`, `coquille/`, `reglages/`, `project.inlang/`, CI.
- **Hypothèses** (10) : framework, types partagés, export, traduction, pnpm et SQLite sont validés ; resvg est fixé en 0.45 pour rester compatible avec svg2pdf.
- Les références pointent vers S01 v1.3, S03 v0.6 et D02 v0.1.

## 12. Changements par rapport à la v0.7

L'atelier du plan (branche `feat/front-atelier`) a tranché :
- **Commandes** (7) : `series_depassant` (lecture seule) ; `creer_terrain`, `modifier_terrain`, `creer_planche`, `modifier_planche` et `deplacer_planche` appliquent les règles de S03 §15.
- **Erreurs** (7) : sept codes de placement ; l'erreur ne porte que `code` et `detail`, donc l'interface affiche un motif générique et ne nomme pas la planche en cause (ET01 montre les deux noms : écart assumé, à reprendre avec une erreur structurée).
- **Vue de l'année** (5) : `series_supprimees`, `rangs` et `pieds_par_rang`, `historique`.
- **Atelier** (4) : pendant le glisser, l'interface ne calcule que la position en cases ; Rust juge au lâcher. Les aides visuelles de refus (rouge pâle) sont celles du retour de Rust.
- Les références pointent vers S03 v0.7.

## 13. Changements par rapport à la v0.8

La frise, le formulaire de série et le catalogue (branche `feat/front-frise`) ont tranché :
- **Commandes** (7) : `decaler_serie`, `apercu_serie`, `cultures_de_saison` ; `creer_serie` et `modifier_serie` acceptent une date de repiquage (`date_est_repiquage`). Un changement de culture ou de mode ne bute plus sur l'ancienne fin modifiée à la main : les champs saisis sont appliqués avant la copie des durées.
- **Vue de l'année** (5) : `SerieView.pot_days`.
- **Frise** (4) : une seule image SVG ; l'interface charge la vue de chaque année visible et ne recalcule rien. Elle ouvre sur 12 mois, et sur 6 mois sous 560 px de large (12 mois y seraient illisibles). Le glisser d'une barre est un décalage en jours entiers ; Rust juge, et sur refus la barre revient (sans animation si « réduire les animations » est demandé) et le texte du refus s'affiche, sans nommer la série en conflit (même écart que l'atelier). Au clavier, les flèches décalent la barre d'un jour (Maj : une semaine).
- **Export de la frise** (4) : SVG autonome, validé par un test Rust sur un SVG produit par l'interface.
- **Formulaire de série** (E07) : valeurs par défaut décidées sans Tim : mode « à l'abri » si le catalogue a une durée en godets, sinon pleine terre ; 21 jours en godets proposés pour un légume sans godets (exemple de S03 T9) ; une saisie de fin égale à la fin calculée n'est pas une fin à la main ; les tronçons se choisissent par clics (un clic, puis un second étend la zone).
- **Catalogue** (E09) : l'interrupteur « de saison maintenant » filtre la liste ; les badges « De saison » sont toujours affichés. Les illustrations des quatre stades sont des dessins simplifiés, à remplacer par ceux du diorama.
- **Hors de cette version** : annotations et export de la frise (écran), « Annoter », « vider_plan », annuler et rétablir.

## 14. Changements par rapport à la v0.9

Les réglages et les données (branche `feat/data-reglages`) ont tranché :
- **Notes et périodes** (7) : les trois commandes appliquent les règles de `plan` (S03 §16). `annee_passee` joue sur l'année affichée et sur la fin de l'annotation. Le texte d'une période est limité à 80 caractères comme celui d'une note. Une note n'a pas de fin : une fin reçue est ignorée. La vue de l'année contient toujours les annotations qui touchent l'année, masquées comprises. Le formulaire `NoteForm.svelte` (`src/lib/notes/`) est autonome : il rend l'entrée de la commande et ne l'appelle pas ; la frise l'intégrera.
- **Validation d'un plan importé** (6, 7) : deux niveaux (S03 §13). Refus pour ce que les calculs ne peuvent pas lire ; le reste est accepté **avec la liste des incohérences à résoudre**, rien n'est supprimé ni corrigé, par cohérence avec le comportement par défaut de la validation après fusion. `lire_import` renvoie la liste (`IncoherenceView` : code, année, noms des planches et des légumes), l'interface la montre dans la carte de confirmation, sous « Remplacer quand même ». Il n'y a pas encore d'écran qui garde cette liste après l'import : les conflits et les dépassements restent visibles dans la frise et l'atelier ; l'écran des « à résoudre » reste à concevoir (S03 §13, questions 1 à 4).
- **Vider le plan** (7) : `vider_plan(annee)`, selon S01 F-PLN-4 : les planches de l'année affichée, leurs séries et leurs notes à partir de cette année. L'interface demande confirmation, avec le nombre de planches et de séries.
- **Erreurs** (7) : deux codes ajoutés, `texte_invalide`, `annotation_dates_invalides` ; `annotation_inconnue` vient maintenant de `plan`. L'interface rattache les deux premiers à des textes `note_erreur_*`.
- **États** (4) : une erreur d'enregistrement affiche, en plus de l'indicateur de l'en-tête, la carte de ET01 (« Erreur d'enregistrement », « Rien n'est perdu », « Réessayer »). Un fichier d'import illisible ou d'un format inconnu, ou une erreur de fichier, s'affiche dans le panneau des réglages (pas dans un message qui disparaît) et précise que le potager de l'appareil n'a pas changé.
- **Capacité** : `vider_plan` est ajoutée à la capacité `main`.
- Les références pointent vers S03 v0.8.

## 15. Changements par rapport à la v0.10

L'export de la frise et les annotations (branche `feat/front-export`) ont tranché :
- **Export** (4, 7) : l'écran E11 monte `FriseSvg` hors écran sur la période choisie (échelle fixe, 1 800 px de large pour une année), les planches cochées et les options (phases, repère « aujourd'hui », annotations), le sérialise avec `serialiserFrise`, et montre ce texte comme aperçu. « Exporter » ouvre « Enregistrer sous » (plugin dialog), puis `exporter_frise(svg, format, chemin)`. Une erreur `fichier` affiche la carte de ET01 avec « Réessayer ». La période est limitée à 3 ans, comme la frise. Le PDF n'ajoute pas encore l'en-tête, la miniature du plan ni le calendrier des tâches (S01 F-EXP-2).
- **Annotations** (4) : une note est un repère, une période une bande derrière les barres, sur une planche ou sur tout le potager. Un clic ouvre le formulaire (modifier, masquer, supprimer). Une annotation masquée reste visible en fantôme à l'écran, pour pouvoir la retrouver, et n'est jamais exportée. La case « Afficher les annotations » filtre l'écran et l'export (par défaut cochée). Le panneau planche (E06) liste les notes de la planche et propose « Ajouter une note ». Pas de glisser d'une note, pas de double-clic sur une ligne.
- **À résoudre** (7) : `incoherences_plan()` est ajoutée, et à la capacité `main`.
- Les références pointent vers S03 v0.9.
