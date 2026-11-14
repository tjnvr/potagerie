# POT-S05 · Spec technique — v0.5

> Version : 0.5 (brouillon) · Date : 2026-11-14 · Auteur : Tim
> Références : [S01](POT-S01_spec-fonctionnelle_v1.2.md) · [S02](POT-S02_catalogue-cultures_v0.2.md) · [S03](POT-S03_regles-calculs_v0.5.md) · [P01](../prototypes/POT-P01_rendu-iso_v0.1/README.md) · [D01](../design/POT-D01_ecrans-cles_v1.0/README.md)
> La v0.4 a introduit Rust + Tauri. La v0.5 aligne le modèle sur S01 v1.1 et S03 v0.3, et retire l'export CSV.

## 1. Principes

- **Une appli installée, locale d'abord.** Pas d'hébergement ni de compte : un seul utilisateur, et ses données restent sur ses appareils.

  | Mode | Lancement | Stockage | Lot |
  |---|---|---|---|
  | **Développement** | `cargo run -p potagerie-serve` puis http://localhost:8080 | Fichier SQLite | — |
  | **Bureau** | `Potagerie.app` sur macOS, `Potagerie.exe` sur Windows | SQLite dans le dossier de données de l'appli | 1 |
  | **Mobile** | Appli iOS et Android | SQLite dans le dossier de données de l'appli | 2 |
  | **Synchro** | Les appareils échangent leurs modifications, fusionnées sans conflit | Document CRDT | 3 |

  Tous les modes partagent le même routeur axum. Passer de l'un à l'autre change seulement le point d'entrée.
- **Rust + htmx.** Le routeur rend le HTML et le SVG, et htmx remplace les fragments de page après chaque action. Il n'y a pas de framework front ni d'étape de build JavaScript.
- **Le métier vit en Rust.** Les calculs de dates, les créneaux libres, les suggestions (S03) et les règles de cohérence sont écrits en Rust et testés unitairement (`cargo test`).
- **Le modèle est prêt pour la fusion dès le lot 1** (voir 5), pour que le passage au CRDT du lot 3 ne demande pas de tout réécrire.

## 2. Architecture

```
webview ── HTML + SVG rendus par le routeur
   │       htmx (fragments) + petits modules JS (animation, glisser-déposer)
   │       requêtes vers le protocole personnalisé de Tauri (sans port réseau)
appli Tauri `potagerie`
   ├── web        routeur axum, templates, fichiers statiques embarqués
   ├── plan       modèle, règles, calculs de dates (S03), validation
   ├── catalogue  cultures.json (S02), embarqué
   ├── rendu      construction du diorama SVG et de la frise
   └── store      trait ── sqlite (lots 1 et 2) │ crdt (lot 3)
```

- **Pas de sidecar ni de serveur local** : le routeur axum est un `tower::Service`, appelé directement par le gestionnaire du protocole personnalisé de Tauri.
- **Tout est embarqué dans l'appli** : templates (compilés), CSS, JS et catalogue (`rust-embed` ou `include_dir`).
- **Plus de cgo** : tout se compile avec la chaîne Rust standard.

## 3. Appli Tauri

| Sujet | Choix |
|---|---|
| Emballage | Tauri v2 : fenêtre native qui affiche l'interface dans le moteur web du système (WebKit sur macOS et iOS, WebView2 sur Windows, WebView sur Android) |
| Branchement | Chaque requête de la webview vers le protocole personnalisé, htmx compris, est transmise au routeur axum : routes et fragments identiques au mode développement |
| Données | `potagerie.db` dans le dossier de données de l'appli fourni par Tauri (`app_data_dir`) |
| Exports | Les fichiers (PNG, PDF, JSON) passent par la fenêtre « Enregistrer sous » du plugin dialog de Tauri : le téléchargement d'un navigateur n'existe pas tel quel dans une webview |
| Compilation bureau | Version macOS universelle (Apple Silicon et Intel) sur un Mac ; version Windows sur un runner GitHub Actions Windows |
| Compilation mobile (lot 2) | iOS sur un Mac avec Xcode ; Android avec le SDK et le NDK Android |
| Prérequis | Windows 10 ou 11 avec le runtime WebView2, présent par défaut sur les versions à jour |
| Signature bureau | Aucune en V1 (usage personnel) : au premier lancement, clic droit puis Ouvrir sur macOS, « Exécuter quand même » sur Windows |
| Installation mobile (lot 2) | Android : APK installé à la main. iOS : voir 10 |
| Mises à jour | Manuelles en V1 : on remplace l'appli. Le plugin updater de Tauri reste possible plus tard |

Le diorama et les modules JS doivent être vérifiés sous WebKit (macOS et iOS), WebView2 et la WebView Android.

## 4. Interface

| Sujet | Choix |
|---|---|
| Templates | askama : templates compilés et typés |
| Échanges | htmx 2 : `hx-get` et `hx-post` ; la réponse est un fragment HTML qui remplace la zone concernée |
| Diorama | SVG rendu par Rust : une forme `<symbol>` par légume et par stade, une balise `<use>` par plante (P01) |
| Jouer la saison | Module JS d'environ 100 lignes : il anime les `<use>` dans la webview à partir des dates portées en attributs `data-*`, sans aller-retour vers le routeur pendant la lecture |
| Atelier du plan | Module JS en Pointer Events (souris et tactile) pour le glisser ; au lâcher, un `hx-post` enregistre la position et le routeur renvoie le plan mis à jour |
| Frise des cultures | Rendue par Rust (SVG ou grille HTML) ; ajouter, déplacer ou annoter passe par htmx |
| Export | PNG et PDF de la frise générés dans la webview à partir du SVG, puis enregistrés via le plugin dialog ; export du potager (JSON) généré par Rust |
| Mise en page | Ordinateur et téléphone selon D01. La mise en page s'adapte à la largeur dès le lot 1, pour que le lot 2 ne refasse pas l'interface |

L'animation et le glisser-déposer restent dans la webview, parce qu'un aller-retour à chaque image serait trop lent. Tout le reste passe par le routeur.

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

L'appli vérifie toutes ces règles. Les dates calculées (repiquage, mise en place, récolte, fin non modifiée à la main) ne sont pas stockées : elles sont recalculées à chaque rendu à partir des durées copiées dans la série. Une mise à jour du catalogue ne décale donc aucune série.

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
- **Validation après fusion** : un document fusionné converge, mais il peut enfreindre les règles du plan. Cas à traiter, à spécifier dans S03 :

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

## 7. Routes

| Méthode | Route | Renvoie |
|---|---|---|
| GET | `/` | Page Potager (onglets Potager et Plan) |
| GET | `/potager?annee=` | Fragment : diorama + frise de l'année |
| POST | `/series`, `/series/{id}`, `/series/{id}/suppression` | Fragment : frise mise à jour |
| POST | `/planches`, `/planches/{id}/position`, `/planches/{id}/suppression` | Fragment : atelier mis à jour |
| POST | `/annotations`, `/annotations/{id}/suppression` | Fragment : frise mise à jour |
| GET | `/export/potager.json` | Export complet (F-CPT-3) |
| POST | `/import` | Remplace le potager après confirmation (F-CPT-3) |
| GET | `/healthz` | État du service (mode développement) |

Le lot 3 ajoute les routes de synchro, à détailler avec le lot.

**Écritures** : aux lots 1 et 2, chaque appareil écrit seul dans sa base et chaque `POST` incrémente `version`. Au lot 3, les modifications sont fusionnées (6).

## 8. Arborescence du dépôt

```
Cargo.toml              workspace
crates/plan/            modèle, règles, calculs, validation
crates/catalogue/       cultures.json + chargement
crates/rendu/           diorama SVG, frise
crates/web/             routeur axum, templates/, static/ (htmx, CSS, modules JS)
crates/store/           trait + sqlite (+ crdt au lot 3)
crates/serve/           point d'entrée développement (localhost:8080)
src-tauri/              appli Tauri : bureau (lot 1) et mobile (lot 2)
docs/                   specs, designs, prototypes
Makefile                build, test, run
```

## 9. Lots

| Lot | Contenu | Sortie |
|---|---|---|
| **1 · Bureau** | Pages et fragments htmx, diorama, frise, atelier, export de la frise (PDF, PNG), export et import du potager, SQLite, mise en page adaptable, appli Tauri bureau | `Potagerie.app` et `Potagerie.exe` utilisables au quotidien |
| **2 · Mobile** | Appli Tauri iOS et Android, mise en page téléphone (D01), transfert par export et import JSON | Le potager consultable et modifiable sur téléphone |
| **3 · Synchro** | Document CRDT, validation après fusion, transports QR code et dossier cloud | Le même potager sur tous les appareils, sans serveur |

## 10. Hypothèses à valider

| Sujet | Hypothèse par défaut | Alternative |
|---|---|---|
| Branchement htmx | Protocole personnalisé de Tauri, à valider par un prototype sous WebKit, WebView2 et Android | Serveur axum sur localhost, port aléatoire et jeton |
| Templates | askama | maud (HTML écrit en macros Rust) |
| SQLite | rusqlite, SQLite embarqué | sqlx |
| Dates | chrono | jiff |
| CRDT (lot 3) | Automerge + autosurgeon (struct Rust adossée au document) | Loro |
| Après fusion (lot 3) | Rien n'est supprimé automatiquement : les incohérences s'affichent à résoudre | Réparation automatique (par exemple, suppression des séries orphelines) |
| Relais (lot 3) | Aucun : QR code et dossier cloud suffisent | Relais samod auto-hébergé (Raspberry Pi), si les deux transports ne suffisent pas |
| iOS (lot 2) | Compte Apple Developer (99 $ par an) pour installer durablement sur iPhone | Compte gratuit : réinstallation depuis Xcode tous les 7 jours ; ou Android seul |
| Rendu | SVG | Hybride Canvas + couche SVG, si le téléphone saccade (P01) |
| Signature (bureau) | Aucune en V1 | Compte Apple Developer et certificat de signature Windows, si l'appli est distribuée à d'autres |
| potagerie.garden | N'héberge plus l'appli | Page de présentation et téléchargements |

## 11. Écarts avec S01

Les écarts avec S01 v1.0 relevés dans la v0.4 (appli installée, pas de compte, enregistrement local, transfert par export et import, synchro au lot 3) sont reportés dans S01 v1.1.
