# Potagerie — Catalogue des cultures

> Version : 0.1 (brouillon) · Date : 14 novembre 2026 · Auteur : Tim

---

## 1. Périmètre

- **6 légumes de prototype**, choisis pour couvrir les cas différents : culture longue, culture rapide, semis en pleine terre, grande plante, culture à cheval sur deux années.
- Seulement les champs dont l'appli a besoin pour **calculer** : phases de la frise, quantités, créneaux libres et suggestions.
- Hors périmètre pour l'instant : bons et mauvais voisins, exposition, textes descriptifs, variétés. Ils viendront avec l'élargissement du catalogue.
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
| `mise_en_place` | période | `MM-JJ` → `MM-JJ` | Fenêtre idéale de mise en place. Peut chevaucher deux années (début > fin) |
| `rang_cm` | entier | cm | Espacement entre rangs |
| `pied_cm` | entier | cm | Espacement sur le rang |
| `sensible_gel` | booléen | — | Écarte la culture des suggestions hors saison |

Les **4 illustrations** (semis, jeune pousse, adulte, récolte) sont rattachées à l'`id` et ne font pas partie des données.

---

## 3. Les 6 légumes

| Légume | Famille | Abri | Récolte après | Durée récolte | Mise en place | Rang × pied | Gel |
|---|---|---|---|---|---|---|---|
| Tomate | Solanacées | 56 j | 75 j | 70 j | 05-01 → 06-15 | 80 × 60 cm | sensible |
| Laitue | Astéracées | 28 j | 50 j | 20 j | 03-01 → 09-30 | 30 × 30 cm | résiste |
| Carotte | Apiacées | 0 | 100 j | 40 j | 03-01 → 07-15 | 25 × 4 cm | résiste |
| Haricot vert | Fabacées | 0 | 60 j | 30 j | 05-10 → 07-31 | 50 × 10 cm | sensible |
| Courgette | Cucurbitacées | 21 j | 50 j | 80 j | 05-01 → 06-30 | 100 × 100 cm | sensible |
| Poireau | Amaryllidacées | 70 j | 120 j | 90 j | 05-15 → 07-31 | 30 × 12 cm | résiste |

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

1. Quels légumes ajouter après le prototype, et dans quel ordre ?
2. Faut-il plusieurs fenêtres de mise en place par légume (épinard au printemps et à l'automne) dès l'élargissement ?
