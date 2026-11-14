# POT-S03 · Règles et calculs — v0.1

> Version : 0.1 (brouillon) · Date : 2026-11-14 · Auteur : Tim
> Références : [S01](POT-S01_spec-fonctionnelle_v1.0.md) · [S02](POT-S02_catalogue-cultures_v0.1.md) · [S05](POT-S05_spec-technique_v0.2.md)

Cette spec fixe **chaque calcul** de l'appli, assez précisément pour être codé en Go (`internal/plan`) et testé. Les cas de la section 12 servent de premiers tests unitaires.

---

## 1. Conventions

- **Dates civiles**, sans heure ni fuseau horaire. Toutes les durées sont en jours.
- `d + n` est la date située n jours après d.
- Une date `MM-JJ` du catalogue est placée dans une année Y. Si on tombe sur `02-29` une année non bissextile, on prend `02-28`.
- **Occupation d'une planche** : intervalle **semi-ouvert** `[mise en place, fin[`. Une série peut donc démarrer le jour même où la précédente se termine.

---

## 2. Dates d'une série

**Saisies** : le mode de démarrage, une date `s`, et éventuellement une fin modifiée à la main. **Catalogue** : `abri`, `apres`, `duree` (S02).

| Mode | Semis | Mise en place `m` | Début de récolte `r` |
|---|---|---|---|
| Je sème à l'abri | `s` | `s + A` | `m + apres` |
| … avec « je connais la date de repiquage » | `s − A` | `s` | `m + apres` |
| J'achète des plants | — | `s` | `m + apres` |
| Je sème en pleine terre | `s` | `s` | `m + apres + abri` |

- `A` est la durée sous abri : `abri` s'il est supérieur à 0, sinon **21 jours** par défaut.
- **Semis en pleine terre** : la plante passe en terre la phase qu'elle aurait passée en godet. On ajoute donc `abri` à la durée jusqu'à la récolte, avec 0 pour la carotte ou le haricot.
- **Fin** `f = r + duree`, sauf si elle a été modifiée à la main. Une fin saisie doit être postérieure à `r`.
- Si on modifie `s` ou le mode, `m`, `r` et `f` se recalculent, sauf une fin modifiée à la main (S01, F-PLA-2).
- **Glisser une barre** dans la frise de n jours décale la date saisie `s` et la fin manuelle éventuelle de n jours.

Les **phases de la frise** sont :
- godets, de semis à `m`, en mode abri seulement ;
- croissance, de `m` à `r` ;
- récolte, de `r` à `f`.

Seules la croissance et la récolte occupent la planche.

---

## 3. Stades visuels (vue jardin)

À une date `d` :

| Situation | Affichage |
|---|---|
| `d < m`, mode abri et `d ≥ semis` | Étiquette plantée sur la zone |
| `d < m` sinon, ou `d ≥ f` | Rien |
| `m ≤ d < f` | Plante, au stade donné par `p = (d − m) / (r − m)` |

| `p` | Stade |
|---|---|
| `< 0,15` | Semis |
| `0,15 – 0,50` | Jeune pousse |
| `0,50 – 1` | Adulte |
| `≥ 1` | Récolte |

Une série en **plants achetés** ne descend jamais sous le stade « Jeune pousse ».

---

## 4. Tronçons et zone

- **Taille d'un tronçon** = taille de case du terrain (25, 50 ou 100 cm). Le nombre de tronçons vaut `n = longueur ÷ taille de case`, et c'est un entier, puisque les planches sont calées sur la grille.
- La **longueur** d'une planche est son plus grand côté, et sa **largeur** son plus petit côté. Les tronçons sont numérotés de 1 à n le long de la longueur.
- Une zone est un intervalle `[t1, t2]` avec `1 ≤ t1 ≤ t2 ≤ n`.
- **Raccourcis** ½, ⅓ et ¼ :
  - la taille visée est `k = max(1, arrondi(n × fraction))` ;
  - la zone retenue est la première suite de k tronçons libres sur toute l'occupation prévue de la série ;
  - s'il n'y en a pas, le raccourci est grisé.
- Si une planche est **raccourcie** à n′ tronçons, toute série dont `t2 > n′` et qui occupe l'année modifiée ou une année suivante est signalée avant validation (S01, F-PLN-6).

---

## 5. Conflits entre séries

Deux séries a et b de la même planche **sont en conflit** si leurs zones se chevauchent et que leurs occupations se chevauchent :

```
a.t1 ≤ b.t2  et  b.t1 ≤ a.t2      (zones)
a.m  < b.f   et  b.m  < a.f       (occupations semi-ouvertes)
```

En cas de conflit, la validation est refusée. L'appli montre les tronçons en commun et la période en commun, `[max(a.m, b.m), min(a.f, b.f)[`.

---

## 6. Quantités

Pour une planche de largeur `L`, une zone de `k` tronçons et une taille de case `c` :

```
rangs          = max(1, ⌊L ÷ rang_cm⌋)
pieds par rang = max(1, ⌊(k × c) ÷ pied_cm⌋)
quantité       = rangs × pieds par rang
```

Affichage : « ≈ 21 pieds, 1 rang × 21 ».

---

## 7. Créneaux libres

Le calcul se fait pour chaque planche et pour chaque année Y où elle existe dans la disposition de Y.

1. **Fenêtre de recherche** : `[Y-03-01, Y-11-15]`.
2. **Découpage** : on coupe la fenêtre aux dates `m` et `f` de toutes les séries de la planche. Sur chaque sous-période, l'ensemble des tronçons occupés est constant.
3. Un **créneau** est un triplet `(zone, début, fin)` qui vérifie quatre conditions :
   - tous les tronçons de la zone sont libres chaque jour de `[début, fin]` ;
   - `fin − début + 1 > 35` jours ;
   - il est **maximal dans le temps** : on ne peut avancer `début` ni reculer `fin` sans occuper un tronçon de la zone, ni sortir de la fenêtre ;
   - il est **maximal en largeur** : on ne peut pas élargir la zone sur toute la période.
4. On ne garde que les créneaux **qui ne sont pas contenus** dans un autre, en zone comme en période.

Affichage :
- dans la frise, un rectangle en pointillés sur les tronçons et la période du créneau ;
- dans la vue jardin, un panneau « + Libre » sur la planche si un créneau contient la date du curseur.

---

## 8. Suggestions

Pour un créneau de zone Z, du jour D au jour F, de longueur `N = F − D + 1`, chaque culture du catalogue est évaluée ainsi :

1. **Date de mise en place** : `m` est la plus petite date de `[D, D + 27]` qui tombe dans une fenêtre de mise en place de la culture (section 9). Sans date possible, la culture est écartée.
2. **Mode proposé** : semis à l'abri si `abri > 0` (alors semis = `m − abri`), sinon semis en pleine terre. On calcule `r` selon la section 2.
3. **Récolte minimale** : `min = ⌈duree ÷ 2⌉`. La culture est écartée si `r + min > F + 1`.
4. **Gel**, pour une culture sensible : elle est écartée si `m` tombe avant les dernières gelées de l'année, ou si `r + min` tombe après les premières gelées.
5. **Fin proposée** : `f = min(F + 1, r + duree)`.
6. **Score de remplissage** : `(f − D) ÷ N`, entre 0 et 1.

**Classement**, au plus 3 cultures :
1. score arrondi au dixième, du plus grand au plus petit ;
2. puis une famille différente de celle de la dernière série terminée avant D sur un tronçon de Z ;
3. puis par ordre alphabétique du nom.

Les engrais verts passent toujours en dernier.

Toucher une suggestion ouvre la création de série avec la culture, le mode, la date, la zone Z et la fin proposée déjà remplis.

---

## 9. Fenêtres de mise en place et gel

- **Fenêtre sur une même année** (`debut ≤ fin`) : `[Y-debut, Y-fin]` pour chaque année Y.
- **Fenêtre à cheval sur deux années** (`debut > fin`, ail par exemple) : `[Y-debut, (Y+1)-fin]`.
- **Périodes de gel** de la frise : pour chaque année Y, du 1er janvier aux dernières gelées, et des premières gelées au 31 décembre (F-TER-4).

---

## 10. Ensoleillement

Le calcul est simplifié (S01, F-TER-3). On compte le nombre d'**heures de soleil direct** au centre de chaque case, le **21 juin**.

| Heures de soleil le 21 juin | Niveau |
|---|---|
| ≥ 6 h | Plein soleil |
| 3 – 6 h | Mi-ombre |
| < 3 h | Ombre |

**Position du soleil**, calculée toutes les 15 minutes de 4 h à 20 h, en heure solaire :
- déclinaison `δ = +23,44°` le 21 juin, et `0°` à l'équinoxe ;
- angle horaire `H = 15° × (t − 12)`, avec t en heures ;
- hauteur `a` : `sin a = sin φ · sin δ + cos φ · cos δ · cos H`, où φ est la latitude (44,84° pour Bordeaux) ;
- azimut depuis le nord `Az` : `cos Az = (sin δ − sin a · sin φ) ÷ (cos a · cos φ)`. On garde `Az` le matin (H < 0) et on prend `360° − Az` l'après-midi ;
- on ne compte que les instants où `a > 5°`, car un soleil plus bas ne compte pas pour la culture.

**Ombre portée** : un obstacle est un pavé de hauteur h, posé sur son emprise au sol. La boussole du terrain (F-TER-1) sert à exprimer `Az` dans les axes de la grille. Le point P est **à l'ombre** à un instant donné si le segment au sol qui part de P vers le soleil, de longueur `h ÷ tan a`, coupe l'emprise d'un obstacle. Les arbres sont traités comme des pavés en V1. Les planches et le décor ne font pas d'ombre.

**Ensoleillement d'une planche** : c'est le niveau de la majorité de ses cases, et en cas d'égalité le niveau le plus ensoleillé.

---

## 11. Rappels et calendrier

- **Familles des 2 saisons précédentes** (panneau planche) : ce sont les familles des séries de la planche dont l'occupation touche les années Y−1 ou Y−2, où Y est l'année du curseur.
- **Calendrier des tâches** (export) : chaque série produit une ou deux tâches, regroupées par mois.

  | Mode | Tâches |
  |---|---|
  | Semis à l'abri | « Semer à l'abri » au semis, « Repiquer » à `m` |
  | Plants | « Acheter et planter » à `m` |
  | Pleine terre | « Semer en pleine terre » à `m` |
  | Tous | « Commencer à récolter » à `r` |

---

## 12. Cas de test

Les données viennent de la S02. La planche fait 1,2 m de large, avec des cases de 50 cm.

| # | Entrée | Attendu |
|---|---|---|
| T1 | Tomate, semis à l'abri le 2027-03-10 | `m` 2027-05-05 · `r` 2027-07-19 · `f` 2027-09-27 |
| T2 | Laitue, pleine terre le 2027-04-01 | `m` 2027-04-01 · `r` 2027-06-18 (50 + 28 j) · `f` 2027-07-08 |
| T3 | Carotte, pleine terre le 2027-03-15 | `r` 2027-06-23 · `f` 2027-08-02 |
| T4 | Carotte, zone de 2 tronçons | 4 rangs × 25 pieds = ≈ 100 |
| T5 | Tomate, zone de 3 tronçons | 1 rang × 2 pieds = ≈ 2 |
| T6 | Tomate T1 sur [1,3], laitue plants le 2027-09-27 sur [1,2] | Pas de conflit : la tomate se termine le jour où la laitue démarre |
| T7 | Planche de 6 tronçons, avec seulement la carotte T3 sur [1,6], année 2027 | Créneaux : [1,6] du 03-01 au 03-14 écarté (< 35 j) ; **[1,6] du 2027-08-02 au 2027-11-15** (106 j) |
| T8 | Suggestions pour le créneau de T7 | Laitue seule : semis à l'abri le 2027-07-05, `m` 2027-08-02, `r` 2027-09-21, `f` 2027-10-11 ; les 5 autres cultures sont hors fenêtre de mise en place |

---

## 13. Questions ouvertes

1. Durée sous abri par défaut, pour un légume qui n'est pas semé à l'abri d'habitude : faut-il 21 jours, ou faut-il interdire le mode ? S01 garde les trois modes toujours disponibles.
2. Ensoleillement : le 21 juin suffit-il, ou faut-il combiner avec l'équinoxe (moyenne des deux journées, par exemple) pour mieux refléter le printemps ?
3. Récolte minimale des suggestions : la moitié de la durée de récolte convient-elle ?
