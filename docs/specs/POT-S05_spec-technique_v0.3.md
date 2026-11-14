# POT-S05 · Spec technique — v0.3

> Version : 0.3 (brouillon) · Date : 2026-11-14 · Auteur : Tim
> Références : [S01](POT-S01_spec-fonctionnelle_v1.0.md) · [S02](POT-S02_catalogue-cultures_v0.1.md) · [P01](../prototypes/POT-P01_rendu-iso_v0.1/README.md)

## 1. Principes

- **Appli de bureau d'abord, déployable ensuite.** Le même code tourne en trois modes :

  | Mode | Lancement | Stockage | Compte |
  |---|---|---|---|
  | **Développement** | `potagerie serve` puis http://localhost:8080 | Fichier SQLite | Aucun |
  | **Bureau** (lot 1) | `Potagerie.app` sur macOS, `Potagerie.exe` sur Windows | Fichier SQLite dans le dossier de l'utilisateur | Aucun, un seul utilisateur |
  | **Déployé** (lot 2) | Sur AWS, sous potagerie.garden | DynamoDB | Lien magique ou Google |

  Les trois modes partagent le même `http.Handler`. Passer de l'un à l'autre change seulement le point d'entrée et les adaptateurs de stockage et d'authentification.
- **Go + htmx.** Le serveur rend le HTML et le SVG, et htmx remplace les fragments de page après chaque action. Il n'y a pas de framework front ni d'étape de build JavaScript.
- **Le métier vit en Go.** Les calculs de dates, les créneaux libres, les suggestions (S03) et les règles de cohérence sont écrits en Go et testés unitairement.

## 2. Architecture

```
navigateur ── HTML + SVG rendus par le serveur
     │        htmx (fragments) + petits modules JS (animation, glisser-déposer)
binaire Go `potagerie`
     ├── web        routes HTML, templates, fichiers statiques (embed.FS)
     ├── plan       modèle, règles, calculs de dates (S03)
     ├── catalogue  cultures.json (S02), embarqué
     ├── rendu      construction du diorama SVG et de la frise
     ├── store      interface ── sqlite (local) │ dynamo (déployé)
     └── auth       interface ── none (local)   │ session (déployé)
```

- **Un seul binaire par point d'entrée**, sans dépendance à installer : templates, CSS, JS et catalogue sont embarqués (`embed.FS`).
- **SQLite en pur Go** (`modernc.org/sqlite`, sans cgo) : seul l'emballage Wails demande cgo, le binaire de développement et celui du lot 2 se compilent partout.

## 3. Appli de bureau

| Sujet | Choix |
|---|---|
| Emballage | Wails v2 : fenêtre native qui affiche l'interface dans le moteur web du système (WebKit sur macOS, WebView2 sur Windows) |
| Branchement | Le `http.Handler` de l'appli est passé tel quel au serveur de ressources de Wails : routes et fragments htmx inchangés, sans port réseau ouvert |
| Données | `~/Library/Application Support/Potagerie/potagerie.db` sur macOS, `%APPDATA%\Potagerie\potagerie.db` sur Windows |
| Compilation | Wails demande cgo. La version macOS (universelle, Apple Silicon et Intel) se compile sur un Mac ; la version Windows sur un runner GitHub Actions Windows |
| Prérequis | Windows 10 ou 11 avec le runtime WebView2, présent par défaut sur les versions à jour |
| Signature | Aucune en V1 (usage personnel) : au premier lancement, clic droit puis Ouvrir sur macOS, « Exécuter quand même » sur Windows |
| Mises à jour | Manuelles en V1 : on remplace l'appli |

Le diorama et les modules JS doivent aussi être vérifiés sous WebKit (Safari), qui est le moteur de l'appli macOS.

## 4. Interface

| Sujet | Choix |
|---|---|
| Templates | `html/template` de la bibliothèque standard |
| Échanges | htmx 2 : `hx-get` et `hx-post` ; la réponse est un fragment HTML qui remplace la zone concernée |
| Diorama | SVG rendu par Go : une forme `<symbol>` par légume et par stade, une balise `<use>` par plante (P01) |
| Jouer la saison | Module JS d'environ 100 lignes : il anime les `<use>` dans le navigateur à partir des dates portées en attributs `data-*`, sans aller-retour serveur pendant la lecture |
| Atelier du plan | Module JS en Pointer Events (souris et tactile) pour le glisser ; au lâcher, un `hx-post` enregistre la position et le serveur renvoie le plan mis à jour |
| Frise des cultures | Rendue par Go (SVG ou grille HTML) ; ajouter, déplacer ou annoter passe par htmx |
| Export | PNG et PDF générés dans le navigateur à partir du SVG ; CSV généré par Go |

L'animation et le glisser-déposer restent dans le navigateur, parce qu'un aller-retour serveur à chaque image serait trop lent. Tout le reste passe par le serveur.

## 5. Modèle de données

Un **plan** par utilisateur, stocké comme **un seul document versionné**. Chaque action le charge, le modifie, puis l'enregistre.

```
Plan
├── id, version, mis_a_jour_le
├── dispositions[annee]            ← une disposition par année (S01, option B)
│   └── planches[] : id, nom, type (planche|bac), x, y, largeur, longueur
│                    tronçons de 50 cm déduits de la longueur
├── series[] : id, culture_id, planche_id, troncon_debut, troncon_fin,
│              mode (abri|plants|pleine_terre), date,
│              abri_jours? (saisi seulement si le catalogue n'en prévoit pas, S03),
│              fin_manuelle?
└── annotations[] : id, type (note|periode), date_debut, date_fin?, texte
```

- Au maximum 10 planches par disposition.
- Les années passées sont en lecture seule.
- Une série référence une planche par son `id`, qui reste stable d'une année à l'autre.

Le serveur vérifie toutes ces règles. Les dates calculées (levée, mise en place, récolte) ne sont pas stockées : elles sont recalculées à chaque rendu à partir du catalogue.

## 6. Routes

| Méthode | Route | Renvoie |
|---|---|---|
| GET | `/` | Page Potager (onglets Potager et Plan) |
| GET | `/potager?annee=` | Fragment : diorama + frise de l'année |
| POST | `/series`, `/series/{id}`, `/series/{id}/suppression` | Fragment : frise mise à jour |
| POST | `/planches`, `/planches/{id}/position`, `/planches/{id}/suppression` | Fragment : atelier mis à jour |
| POST | `/annotations`, `/annotations/{id}/suppression` | Fragment : frise mise à jour |
| GET | `/export/frise.csv` | Fichier CSV |
| GET | `/healthz` | État du service |

En mode déployé, l'appli ajoute les routes de compte : `/connexion`, `/auth/lien/{jeton}`, `/auth/google`, `/deconnexion`.

**Écritures concurrentes** : la dernière écriture gagne (S01). Chaque `POST` incrémente `version`.

## 7. Arborescence du dépôt

```
cmd/potagerie/          point d'entrée développement (serve)
cmd/potagerie-desktop/  point d'entrée bureau (Wails)
internal/web/           handlers, templates/, static/ (htmx, CSS, modules JS)
internal/plan/          modèle, règles, calculs
internal/catalogue/     cultures.json + chargement
internal/rendu/         diorama SVG, frise
internal/store/         interface + sqlite/ (+ dynamo/ au lot 2)
internal/auth/          interface + none/ (+ session/ au lot 2)
docs/                   specs, designs, prototypes
Makefile                build, test, run
```

## 8. Lots

| Lot | Contenu | Sortie |
|---|---|---|
| **1 · Bureau** | Pages et fragments htmx, diorama, frise, atelier, export, SQLite, sans compte, emballage Wails | `Potagerie.app` et `Potagerie.exe` utilisables au quotidien |
| **2 · Déployé** | Adaptateurs DynamoDB et session, lien magique (SES) et Google, infrastructure AWS | potagerie.garden en ligne |

## 9. Hypothèses à valider

| Sujet | Hypothèse par défaut | Alternative |
|---|---|---|
| Templates | `html/template` (bibliothèque standard) | `templ` (typé, compilé, mais une étape de génération) |
| Hébergement (lot 2) | Lambda + API Gateway, via un adaptateur HTTP | Conteneur sur Fargate ou App Runner, si le démarrage à froid gêne |
| Base déployée (lot 2) | DynamoDB, un élément par plan | PostgreSQL |
| Compte (lot 2) | Maison en Go : lien magique par SES, Google OpenID, cookie de session signé | Cognito |
| Rendu | SVG | Hybride Canvas + couche SVG, si le téléphone saccade (P01) |
| Signature (bureau) | Aucune en V1 | Compte Apple Developer (99 $ par an) et certificat de signature Windows, si l'appli est distribuée à d'autres |
