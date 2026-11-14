# Potagerie — Index de la documentation

## Arborescence

```
docs/
├── POT-00_index.md                         ← ce fichier
├── specs/
│   ├── POT-S01_spec-fonctionnelle_v1.1.md   ← versions courantes
│   ├── POT-S02_catalogue-cultures_v0.1.md
│   ├── POT-S03_regles-calculs_v0.3.md
│   ├── POT-S05_spec-technique_v0.4.md
│   └── archives/                            ← versions précédentes
├── design/
│   ├── POT-D01_ecrans-cles_v1.0/            ← version courante (un dossier par canvas)
│   └── archives/                            ← versions précédentes
└── prototypes/
    ├── POT-P01_rendu-iso_v0.1/              ← un dossier par prototype
    └── archives/
```

## Convention de nommage (identique pour specs, designs et prototypes)

```
POT-<T><NN>_<sujet>_v<X.Y>      (.md pour une spec, dossier pour un design ou un prototype)
```

| Élément | Règle | Exemple |
|---|---|---|
| `POT` | Préfixe du projet Potagerie | `POT` |
| `<T>` | Type : `S` spec, `D` design, `P` prototype | `S`, `D`, `P` |
| `<NN>` | Numéro sur 2 chiffres, propre à chaque type, attribué une fois pour toutes, jamais réutilisé | `01` |
| `<sujet>` | Minuscules, sans accents, mots séparés par des tirets | `spec-fonctionnelle`, `ecrans-cles` |
| `v<X.Y>` | Version : `v0.Y` brouillon, `v1.0` validé. Les évolutions suivantes, même de périmètre, font avancer Y (`v1.1`). X ne passe à 2 qu'une fois une première version de l'appli sortie (lot 1 livré) | `v0.5`, `v1.1` |

Règles communes :
- **Une seule version courante** par document dans `specs/`, `design/` ou `prototypes/`. À chaque nouvelle version, la précédente est déplacée telle quelle dans le dossier `archives/` voisin.
- **En-tête** : chaque spec, et le `README.md` de chaque dossier de design ou de prototype, commence par `Version · Date · Auteur`, cohérents avec le nom.
- **Index** : ce fichier est mis à jour à chaque création, renommage ou nouvelle version.
- Le titre du canvas sur claude.ai reprend le code : « POT-D01 · Écrans clés V1 ».

Propre aux designs :
- Contenu d'un dossier : `README.md`, `project/` (copie exacte des fichiers du canvas), `exports/` (PNG, PDF), et un éventuel générateur.
- Planches : `<code-écran>_<ecran>_<support>.dc.html`, avec le code écran de la spec (`E05`, `E07`) ou `DV<NN>` pour la direction visuelle, et `desktop` ou `mobile` comme support. Exception : la planche d'entrée du canvas s'appelle toujours `Main.dc.html`, car l'outil l'impose.
- Les exports portent le nom de leur planche : `exports/E05_potager_mobile.png`.

Propre aux prototypes :
- Code jetable, autonome (s'ouvre sans installation), qui répond à **une question** précise.
- Le `README.md` contient : la question, le protocole, les résultats et la décision prise.

## Documents

| Code | Document | Version | Statut | Lien |
|---|---|---|---|---|
| S01 | Spec fonctionnelle V1 (appli installée, sans compte, lots bureau, mobile et synchro) | 1.1 | Brouillon | [specs/POT-S01_spec-fonctionnelle_v1.1.md](specs/POT-S01_spec-fonctionnelle_v1.1.md) |
| S02 | Catalogue des cultures (6 légumes de prototype) | 0.1 | Brouillon | [specs/POT-S02_catalogue-cultures_v0.1.md](specs/POT-S02_catalogue-cultures_v0.1.md) |
| S03 | Règles et calculs (dates, stades, conflits, quantités, dispositions, créneaux, suggestions, tâches, validation après fusion ; ensoleillement en annexe) | 0.3 | Brouillon | [specs/POT-S03_regles-calculs_v0.3.md](specs/POT-S03_regles-calculs_v0.3.md) |
| S05 | Spec technique (Rust + htmx, appli Tauri bureau puis mobile, synchro CRDT sans serveur) | 0.4 | Brouillon | [specs/POT-S05_spec-technique_v0.4.md](specs/POT-S05_spec-technique_v0.4.md) |
| P01 | Prototype rendu isométrique (SVG ou Canvas ?) | 0.1 | Brouillon | [prototypes/POT-P01_rendu-iso_v0.1/](prototypes/POT-P01_rendu-iso_v0.1/README.md) · [page](https://claude.ai/artifact/PBrzt3qta6sHKisnhfZWMv) |
| D01 | Écrans clés V1 (E03 atelier, E05 ordinateur et téléphone, E07, direction visuelle) | 1.0 | Validé | [design/POT-D01_ecrans-cles_v1.0/](design/POT-D01_ecrans-cles_v1.0/README.md) · [canvas](https://claude.ai/artifact/DoHRUtcujzwPF6bfjdQtSN) |

## Codes réservés

| Code | Document prévu |
|---|---|
| S04 | Direction visuelle |
| D02 | Atelier du plan et export |
