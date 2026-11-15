# Potagerie — Catalogue des cultures

> Version : 0.3 (brouillon) · Date : 15 novembre 2026 · Auteur : Tim
> v0.3 : le catalogue de la V1 passe à 67 légumes (section 3 bis). Les 6 légumes de la section 3 restent le catalogue de référence des cas de test de S03. Le schéma ne change pas.

---

## 1. Périmètre

- **Catalogue de référence : les 6 légumes de la section 3**, choisis pour couvrir les cas différents : culture longue, culture rapide, semis en pleine terre, grande plante, culture à cheval sur deux années. Les cas de test de S03 §12 sont écrits sur ce catalogue.
- **Catalogue de la V1 : 67 légumes annuels** (section 3 bis), qui contiennent les 6 de référence. Les vivaces (rhubarbe, asperge, fraisier, aromatiques vivaces) restent hors V1 : leur culture sur plusieurs années ne rentre pas dans le schéma.
- **Le schéma de la fiche (section 2) est figé pour la V1.** Il ne contient que les champs dont l'appli a besoin pour **calculer** : phases de la frise, quantités, créneaux libres et suggestions.
- Hors V1 : bons et mauvais voisins, exposition, engrais verts, textes descriptifs, variétés. Ils viendront avec l'élargissement du catalogue.
- Les valeurs sont **indicatives**, à affiner à l'usage.

---

## 2. Fiche type

| Champ | Type | Unité | Règle |
|---|---|---|---|
| `id` | texte | — | Identifiant stable, minuscules sans accents (`tomate`) |
| `nom` | texte | — | Nom affiché (« Tomate ») |
| `famille` | texte | — | Famille botanique, pour classer les suggestions |
| `abri_jours` | entier | jours | Durée en godets avant repiquage. 0 si on ne la sème pas à l'abri habituellement |
| `recolte_apres_jours` | entier | jours | Durée de la mise en place au début de la récolte |
| `recolte_duree_jours` | entier | jours | Durée de la période de récolte |
| `mise_en_place` | période | `MM-JJ` → `MM-JJ` | Fenêtre idéale de mise en place, une seule par culture. Peut chevaucher deux années (début > fin) |
| `rang_cm` | entier | cm | Espacement entre rangs |
| `pied_cm` | entier | cm | Espacement sur le rang |
| `sensible_gel` | booléen | — | Non utilisé en V1 ; servira aux alertes de gel |

Les **4 illustrations** (semis, jeune pousse, adulte, récolte) sont rattachées à l'`id` et ne font pas partie des données.

---

## 3. Les 6 légumes de référence

| Légume | Famille | Abri | Récolte après | Durée récolte | Mise en place | Rang × pied | Gel |
|---|---|---|---|---|---|---|---|
| Tomate | Solanacées | 56 j | 75 j | 70 j | 05-01 → 06-15 | 80 × 60 cm | sensible |
| Laitue | Astéracées | 28 j | 50 j | 20 j | 03-01 → 09-30 | 30 × 30 cm | résiste |
| Carotte | Apiacées | 0 | 100 j | 40 j | 03-01 → 07-15 | 25 × 4 cm | résiste |
| Haricot vert | Fabacées | 0 | 60 j | 30 j | 05-10 → 07-31 | 50 × 10 cm | sensible |
| Courgette | Cucurbitacées | 21 j | 50 j | 80 j | 05-01 → 06-30 | 100 × 100 cm | sensible |
| Poireau | Amaryllidacées | 70 j | 120 j | 90 j | 05-15 → 07-31 | 30 × 12 cm | résiste |

## 3 bis. Catalogue complet de la V1

Ce tableau est généré depuis `crates/catalogue/cultures.json`, qui fait foi. Pour les légumes semés en pleine terre (abri 0), « récolte après » compte depuis le semis.

| id | Nom | Famille | Abri (j) | Récolte après (j) | Durée récolte (j) | Mise en place | Rang × pied (cm) | Gel |
|---|---|---|---|---|---|---|---|---|
| `tomate` | Tomate | Solanacées | 56 | 75 | 70 | 05-01 → 06-15 | 80 × 60 | oui |
| `laitue` | Laitue | Astéracées | 28 | 50 | 20 | 03-01 → 09-30 | 30 × 30 | non |
| `carotte` | Carotte | Apiacées | 0 | 100 | 40 | 03-01 → 07-15 | 25 × 4 | non |
| `haricot-vert` | Haricot vert | Fabacées | 0 | 60 | 30 | 05-10 → 07-31 | 50 × 10 | oui |
| `courgette` | Courgette | Cucurbitacées | 21 | 50 | 80 | 05-01 → 06-30 | 100 × 100 | oui |
| `poireau` | Poireau | Amaryllidacées | 70 | 120 | 90 | 05-15 → 07-31 | 30 × 12 | non |
| `aubergine` | Aubergine | Solanacées | 70 | 75 | 70 | 05-15 → 06-15 | 70 × 50 | oui |
| `poivron` | Poivron | Solanacées | 70 | 80 | 70 | 05-15 → 06-15 | 60 × 45 | oui |
| `piment` | Piment | Solanacées | 70 | 80 | 70 | 05-15 → 06-15 | 50 × 40 | oui |
| `tomate-cerise` | Tomate cerise | Solanacées | 56 | 65 | 80 | 05-01 → 06-15 | 80 × 50 | oui |
| `pomme-de-terre` | Pomme de terre | Solanacées | 0 | 100 | 30 | 03-20 → 05-15 | 70 × 35 | oui |
| `physalis` | Physalis | Solanacées | 60 | 90 | 60 | 05-15 → 06-15 | 80 × 60 | oui |
| `laitue-romaine` | Laitue romaine | Astéracées | 28 | 60 | 20 | 03-15 → 08-31 | 35 × 30 | non |
| `chicoree-frisee` | Chicorée frisée | Astéracées | 30 | 70 | 30 | 06-01 → 08-15 | 35 × 35 | non |
| `scarole` | Scarole | Astéracées | 30 | 75 | 30 | 06-01 → 08-15 | 35 × 35 | non |
| `endive` | Endive (racines) | Astéracées | 0 | 150 | 30 | 05-15 → 06-30 | 30 × 10 | non |
| `artichaut` | Artichaut | Astéracées | 60 | 150 | 60 | 04-15 → 05-31 | 100 × 100 | oui |
| `salsifis` | Salsifis | Astéracées | 0 | 180 | 90 | 03-15 → 05-15 | 30 × 8 | non |
| `topinambour` | Topinambour | Astéracées | 0 | 200 | 90 | 03-01 → 04-30 | 80 × 40 | non |
| `panais` | Panais | Apiacées | 0 | 140 | 90 | 03-15 → 05-31 | 35 × 10 | non |
| `celeri-rave` | Céleri-rave | Apiacées | 70 | 140 | 60 | 05-15 → 06-15 | 50 × 40 | non |
| `celeri-branche` | Céleri branche | Apiacées | 70 | 110 | 60 | 05-15 → 06-30 | 50 × 35 | non |
| `fenouil` | Fenouil | Apiacées | 30 | 80 | 30 | 06-01 → 07-31 | 40 × 25 | non |
| `persil` | Persil | Apiacées | 0 | 75 | 150 | 03-15 → 07-31 | 25 × 10 | non |
| `cerfeuil` | Cerfeuil | Apiacées | 0 | 50 | 60 | 03-01 → 09-30 | 25 × 10 | non |
| `coriandre` | Coriandre | Apiacées | 0 | 45 | 30 | 04-01 → 08-31 | 25 × 10 | non |
| `aneth` | Aneth | Apiacées | 0 | 55 | 40 | 04-15 → 07-31 | 30 × 20 | non |
| `haricot-a-ecosser` | Haricot à écosser | Fabacées | 0 | 90 | 30 | 05-10 → 06-30 | 50 × 10 | oui |
| `haricot-a-rames` | Haricot à rames | Fabacées | 0 | 70 | 60 | 05-15 → 07-15 | 80 × 15 | oui |
| `petit-pois` | Petit pois | Fabacées | 0 | 80 | 30 | 02-15 → 04-30 | 50 × 5 | non |
| `pois-gourmand` | Pois gourmand | Fabacées | 0 | 75 | 30 | 02-15 → 04-30 | 50 × 5 | non |
| `feve` | Fève | Fabacées | 0 | 110 | 30 | 02-01 → 04-15 | 50 × 15 | non |
| `concombre` | Concombre | Cucurbitacées | 28 | 55 | 60 | 05-15 → 06-30 | 100 × 50 | oui |
| `cornichon` | Cornichon | Cucurbitacées | 28 | 55 | 60 | 05-15 → 06-30 | 100 × 50 | oui |
| `melon` | Melon | Cucurbitacées | 30 | 80 | 40 | 05-15 → 06-15 | 100 × 80 | oui |
| `pasteque` | Pastèque | Cucurbitacées | 30 | 90 | 30 | 05-20 → 06-15 | 150 × 100 | oui |
| `potiron` | Potiron | Cucurbitacées | 28 | 110 | 30 | 05-15 → 06-15 | 200 × 150 | oui |
| `potimarron` | Potimarron | Cucurbitacées | 28 | 100 | 30 | 05-15 → 06-15 | 150 × 100 | oui |
| `courge-butternut` | Courge butternut | Cucurbitacées | 28 | 110 | 30 | 05-15 → 06-15 | 150 × 100 | oui |
| `patisson` | Pâtisson | Cucurbitacées | 21 | 55 | 70 | 05-15 → 06-30 | 100 × 100 | oui |
| `oignon` | Oignon | Amaryllidacées | 0 | 120 | 30 | 03-01 → 04-15 | 25 × 10 | non |
| `oignon-blanc` | Oignon blanc | Amaryllidacées | 0 | 180 | 60 | 08-15 → 09-30 | 25 × 8 | non |
| `echalote` | Échalote | Amaryllidacées | 0 | 110 | 30 | 02-15 → 03-31 | 25 × 15 | non |
| `ail` | Ail | Amaryllidacées | 0 | 220 | 30 | 10-15 → 12-15 | 25 × 12 | non |
| `ciboulette` | Ciboulette | Amaryllidacées | 50 | 60 | 150 | 04-01 → 06-30 | 30 × 25 | non |
| `chou-pomme` | Chou pommé | Brassicacées | 35 | 90 | 40 | 05-01 → 07-15 | 60 × 50 | non |
| `chou-fleur` | Chou-fleur | Brassicacées | 35 | 90 | 20 | 05-15 → 07-15 | 60 × 60 | non |
| `brocoli` | Brocoli | Brassicacées | 35 | 75 | 30 | 05-01 → 07-15 | 60 × 50 | non |
| `chou-de-bruxelles` | Chou de Bruxelles | Brassicacées | 40 | 130 | 90 | 05-15 → 06-30 | 70 × 60 | non |
| `chou-kale` | Chou kale | Brassicacées | 35 | 70 | 120 | 05-01 → 07-31 | 60 × 50 | non |
| `chou-rave` | Chou-rave | Brassicacées | 30 | 55 | 20 | 04-01 → 08-15 | 35 × 25 | non |
| `chou-chinois` | Chou chinois | Brassicacées | 0 | 70 | 30 | 07-15 → 08-31 | 40 × 35 | non |
| `pak-choi` | Pak-choï | Brassicacées | 0 | 50 | 20 | 07-15 → 09-15 | 30 × 25 | non |
| `radis` | Radis | Brassicacées | 0 | 28 | 14 | 03-01 → 09-30 | 15 × 3 | non |
| `radis-noir` | Radis noir | Brassicacées | 0 | 90 | 60 | 07-01 → 08-15 | 30 × 10 | non |
| `navet` | Navet | Brassicacées | 0 | 60 | 30 | 03-01 → 09-15 | 25 × 10 | non |
| `rutabaga` | Rutabaga | Brassicacées | 0 | 110 | 60 | 06-01 → 07-15 | 40 × 20 | non |
| `roquette` | Roquette | Brassicacées | 0 | 35 | 30 | 03-15 → 09-30 | 20 × 5 | non |
| `betterave` | Betterave | Amaranthacées | 0 | 90 | 60 | 04-15 → 07-15 | 30 × 10 | non |
| `blette` | Blette | Amaranthacées | 0 | 70 | 120 | 04-15 → 07-15 | 40 × 30 | non |
| `epinard` | Épinard | Amaranthacées | 0 | 50 | 30 | 03-01 → 09-30 | 25 × 10 | non |
| `basilic` | Basilic | Lamiacées | 42 | 40 | 90 | 05-15 → 06-30 | 30 × 25 | oui |
| `mache` | Mâche | Valérianacées | 0 | 70 | 90 | 07-15 → 09-30 | 15 × 8 | non |
| `mais-doux` | Maïs doux | Poacées | 21 | 90 | 20 | 05-15 → 06-15 | 70 × 30 | oui |
| `patate-douce` | Patate douce | Convolvulacées | 0 | 120 | 20 | 05-20 → 06-15 | 80 × 40 | oui |
| `gombo` | Gombo | Malvacées | 42 | 70 | 60 | 05-25 → 06-20 | 70 × 40 | oui |
| `pourpier` | Pourpier | Portulacacées | 0 | 50 | 60 | 05-15 → 07-31 | 25 × 15 | oui |

Pour les légumes sans dessin propre, le diorama et les icônes reprennent le dessin d'un légume proche (`front/src/lib/apparences.ts`), et la frise leur donne une couleur à eux.

---

## 4. Format de données

Un fichier `cultures.json`, chargé tel quel par l'appli :

```json
[
  {
    "id": "tomate",
    "nom": "Tomate",
    "famille": "Solanacées",
    "abri_jours": 56,
    "recolte_apres_jours": 75,
    "recolte_duree_jours": 70,
    "mise_en_place": { "debut": "05-01", "fin": "06-15" },
    "rang_cm": 80,
    "pied_cm": 60,
    "sensible_gel": true
  }
]
```

---

## 5. Questions ouvertes

1. Quels légumes ajouter après la V1, et dans quel ordre ? À voir plus tard.

## 6. Décisions

| Sujet | Décision |
|---|---|
| Catalogue de la V1 | 67 légumes annuels (section 3 bis), dont les 6 de référence (section 3), sur lesquels portent les cas de S03 |
| Vivaces | Hors V1 |
| Familles | 14 familles botaniques ; Amaranthacées pour betterave, blette et épinard (classification actuelle) |
| Schéma de la fiche | Figé pour la V1 (section 2) |
| Plusieurs fenêtres de mise en place | Non : une seule fenêtre par culture |
