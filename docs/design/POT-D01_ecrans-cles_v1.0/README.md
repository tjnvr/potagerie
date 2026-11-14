# POT-D01 · Écrans clés V1 — v1.0

> Version : 1.0 (validée) · Date : 2026-11-14 · Auteur : Tim
> Canvas : https://claude.ai/artifact/DoHRUtcujzwPF6bfjdQtSN

| Fichier | Écran |
|---|---|
| `project/canvas.json` | Index du canvas (positions, titres, notes) |
| `project/Main.dc.html` | E05 · Potager — ordinateur (planche d'entrée du canvas) |
| `project/E05_potager_mobile.dc.html` | E05 · Potager — téléphone |
| `project/E07_serie_mobile.dc.html` | E07 · Nouvelle série — téléphone |
| `project/DV01_direction-visuelle.dc.html` | DV01 · Direction visuelle |
| `project/E03_atelier-plan_desktop.dc.html` | E03 · Atelier du plan — ordinateur |
| `gen.py`, `plants2.py` | Générateur des planches et dessins des légumes (`python3 gen.py`) |
| `exports/` | PNG et PDF exportés depuis le canvas |

Le canvas en ligne est la référence. Ce dossier en est la copie versionnée : après des retouches dans le canvas, on resynchronise puis on commit.
`gen.py` régénère tout depuis zéro et écrase les retouches faites à la main dans le canvas.

## Parcours

Les feuilles sont liées entre elles : le bouton Play d'une feuille permet de suivre les liens.

| De | Vers | Déclencheur |
|---|---|---|
| ① E03 Atelier du plan | ② E05 Potager (ordinateur) | Onglet « Potager » ou bouton « Voir le potager » |
| ② E05 Potager (ordinateur) | ① E03 Atelier du plan | Onglet « Plan » |
| ② E05 Potager (ordinateur) | ③ E07 Nouvelle série | « Planter ici », ou clic sur un créneau libre de la frise |
| ② E05 Potager (téléphone) | ③ E07 Nouvelle série | Bouton « + » de la planche |
| ③ E07 Nouvelle série | ② E05 Potager (téléphone) | « Planter » ou la croix |
