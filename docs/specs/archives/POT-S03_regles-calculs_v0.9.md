# POT-S03 · Règles et calculs — v0.9

> Version : 0.9 (brouillon) · Date : 2026-11-15 · Auteur : Tim
> Références : [S01](POT-S01_spec-fonctionnelle_v1.3.md) · [S02](POT-S02_catalogue-cultures_v0.2.md) · [S05](POT-S05_spec-technique_v0.11.md)
> La v0.6 précisait, à l'écriture de `crates/plan`, les cas que la v0.5 laissait ouverts. La v0.7 ajoute, à l'écriture de l'atelier, les règles de placement des planches et de validation du terrain (section 15). Aucun calcul de T1 à T13 ne change. La v0.8 ajoute, à l'écriture des réglages, la validation d'un plan importé (section 13) et les règles des annotations (section 16). La v0.9 ajoute, à l'écriture de l'export de la frise, la liste « à résoudre » qui reste après un import (section 13, questions 1 et 7 tranchées pour le lot 1).

Cette spec fixe **chaque calcul** de l'appli, assez précisément pour être codé en Rust (`crates/plan`) et testé. Les cas de la section 12 servent de premiers tests unitaires.

---

## 1. Conventions

- **Dates du plan = dates civiles**, sans heure ni fuseau horaire : un semis « le 10 mars » est le 10 mars partout. On ne les convertit jamais en UTC, sinon un 10 mars à 00:00 UTC deviendrait le 9 mars au soir à l'ouest de Greenwich.
- **Horodatages techniques en UTC** : dernière modification d'un document ou d'une donnée, par exemple.
- Toutes les durées sont en jours.
- **Aujourd'hui** est un paramètre des calculs, pas une lecture directe de l'horloge, pour que les tests soient reproductibles. C'est la date locale de l'appareil.
- `d + n` est la date située n jours après d.
- Une date `MM-JJ` du catalogue est placée dans une année Y. Si on tombe sur `02-29` une année non bissextile, on prend `02-28`.
- **Occupation d'une planche** : intervalle **semi-ouvert** `[mise en place, fin[`. Une série peut donc démarrer le jour même où la précédente se termine.

---

## 2. Dates d'une série

**Saisies** : le mode de démarrage, une date `s`, et éventuellement une fin modifiée à la main.

**Durées** : `abri`, `apres` et `duree` sont **copiées du catalogue (S02) dans la série à sa création**, et recopiées si l'on change de culture. Une mise à jour du catalogue ne décale donc jamais une série existante.

| Mode | `s` enregistrée | Mise en place `m` | Début de récolte `r` |
|---|---|---|---|
| Je sème à l'abri | date de semis | `s + A` | `m + apres` |
| J'achète des plants | date de plantation | `s` | `m + apres` |
| Je sème en pleine terre | date de semis | `s` | `m + apres + abri` |

- **« Je connais plutôt la date de repiquage »** ne change que la saisie : si l'utilisateur saisit un repiquage `x`, l'appli enregistre la date de semis `s = x − A`.
- `A` est la durée sous abri : `abri` s'il est supérieur à 0. Pour un légume que le catalogue ne prévoit pas en godets (`abri = 0`, carotte ou haricot), le mode abri fait apparaître le champ « Combien de jours en godets ? » : la valeur saisie, d'au moins 1 jour, est enregistrée avec la série et sert de `A`.
- **Changer le mode** d'une série recopie la durée sous abri `A` depuis le catalogue : une carotte passée de « à l'abri » (21 jours saisis) à « pleine terre » n'ajoute plus rien, comme dans T3.
- **Semis en pleine terre** : la plante passe en terre la phase qu'elle aurait passée en godet. On ajoute donc `abri` à la durée jusqu'à la récolte, avec 0 pour la carotte ou le haricot.
- **Fin** `f = r + duree`, sauf si elle a été modifiée à la main. Une fin saisie doit être postérieure à `r`. Une série dont la fin `f` n'est pas postérieure à `m` n'occupe aucun jour et est refusée.
- Si on modifie `s` ou le mode, `m`, `r` et `f` se recalculent, sauf une fin modifiée à la main (S01, F-PLA-2).
- **Glisser une barre** dans la frise de n jours décale la date saisie `s` et la fin manuelle éventuelle de n jours. Si la série décalée n'est plus valide (section 5), le décalage est refusé et la barre revient à sa place.

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

Chaque tranche inclut sa borne basse : `p = 0,15` est « Jeune pousse », `p = 0,50` est « Adulte ». Si `r = m` (aucun délai avant la récolte), la plante est directement au stade « Récolte ».

Une série en **plants achetés** ne descend jamais sous le stade « Jeune pousse ».

---

## 4. Tronçons et zone

- **Taille d'un tronçon** = taille de case du terrain (25, 50 ou 100 cm). Elle est fixée à la création du terrain et ne change plus, pour que les zones restent valables.
- La **longueur** d'une planche est le côté découpé en tronçons. C'est un nombre entier de cases, et le nombre de tronçons vaut `n = longueur ÷ taille de case`. La **largeur** est libre, au centimètre près.
- Les tronçons sont numérotés de 1 à n le long de la longueur.
- Une zone est un intervalle `[t1, t2]` avec `1 ≤ t1 ≤ t2 ≤ n`.
- **Raccourcis** ½, ⅓ et ¼ :
  - la taille visée est `k = max(1, arrondi(n × fraction))`, l'arrondi se faisant à l'entier supérieur pour une moitié (n = 6, ¼ : 1,5 → 2) ;
  - la zone retenue est la première suite de k tronçons libres sur toute l'occupation prévue de la série, et qui tient dans la planche chaque année que touche cette occupation ;
  - s'il n'y en a pas, le raccourci est grisé.
- Si une planche est **raccourcie** à n′ tronçons, toute série dont `t2 > n′` et qui occupe l'année modifiée ou une année suivante est signalée avant validation (S01, F-PLN-6).

---

## 5. Validité d'une série et conflits

Une série est **valide** si sa planche existe dans la disposition de chaque année que touche son occupation `[m, f[` (une série qui se termine un 1er janvier ne touche pas la nouvelle année), si sa zone tient dans la planche (`t2 ≤ n`, avec le `n` de chacune de ces années), et si elle n'est en conflit avec aucune autre série de la planche.

Deux séries a et b de la même planche **sont en conflit** si leurs zones se chevauchent et que leurs occupations se chevauchent :

```
a.t1 ≤ b.t2  et  b.t1 ≤ a.t2      (zones)
a.m  < b.f   et  b.m  < a.f       (occupations semi-ouvertes)
```

En cas de conflit, la validation est refusée. L'appli montre les tronçons en commun et la période en commun, `[max(a.m, b.m), min(a.f, b.f)[`.

**Suppression d'une planche à partir de l'année Y** (S01, F-PLN-4) : on supprime toutes ses séries dont la fin `f` est postérieure au 1er janvier de Y, y compris celles qui ont commencé avant Y. Les séries qui se terminent au plus tard le 1er janvier de Y sont gardées. Les dispositions de cette planche enregistrées pour une année postérieure à Y sont effacées avec elle.

---

## 6. Quantités

Pour une planche de largeur `L` (en cm), une zone de `k` tronçons et une taille de case `c` :

```
rangs          = max(1, ⌊L ÷ rang_cm⌋)
pieds par rang = max(1, ⌊(k × c) ÷ pied_cm⌋)
quantité       = rangs × pieds par rang
```

Affichage : « ≈ 21 pieds, 1 rang × 21 ».

---

## 7. Dispositions du plan

- La disposition d'une planche n'est enregistrée que pour les **années où elle change**.
- La disposition d'une planche en année Y est sa dernière disposition enregistrée pour une année ≤ Y. Si elle n'en a aucune, la planche n'existe pas encore en Y.
- Une suppression à partir de Y est une disposition « absente » enregistrée en Y.
- Une modification d'une planche en année Y remplace sa disposition de Y. Elle vaut ensuite pour les années suivantes **jusqu'à la prochaine disposition enregistrée de cette même planche**, qui reste telle quelle. Les autres planches ne sont pas touchées.
- On ne peut pas modifier une année passée (Y < année d'aujourd'hui).
- Une année ne compte jamais plus de 10 planches (S01, F-PLN-1) : l'ajout d'une 11e, ou le retour d'une planche absente qui porterait une année à 11, est refusé.
- La longueur d'une disposition est un nombre entier de cases, d'au moins une.

---

## 8. Créneaux libres

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

## 9. Suggestions

Pour un créneau de zone Z, du jour D au jour F, de longueur `N = F − D + 1`, chaque culture du catalogue est évaluée ainsi :

1. **Date de mise en place** : `m` est la plus petite date de `[D, D + 27]` qui tombe dans la fenêtre de mise en place de la culture (section 10). Sans date possible, la culture est écartée.
2. **Mode proposé** :
   - si `abri > 0` et `m − abri ≥ aujourd'hui` : semis à l'abri, avec semis = `m − abri` ;
   - si `abri > 0` et que ce semis tomberait avant aujourd'hui : plants, plantés à `m` ;
   - si `abri = 0` : semis en pleine terre à `m`.

   On calcule `r` selon la section 2.
3. **Récolte minimale** : `min = ⌈duree ÷ 2⌉`. La culture est écartée si `r + min > F + 1`.
4. **Fin proposée** : `f = min(F + 1, r + duree)`. Si `f < r + duree`, la série est créée avec cette fin comme **fin modifiée à la main**, pour qu'elle ne se recalcule pas et reste dans le créneau.
5. **Score de remplissage** : `(f − D) ÷ N`, entre 0 et 1.

**Classement**, au plus 3 cultures :
1. score arrondi au dixième, du plus grand au plus petit ;
2. puis une famille différente de celle de la dernière série terminée avant D sur un tronçon de Z (une série terminée avant D a une fin `f ≤ D` ; si plusieurs finissent le même jour, toutes comptent) ;
3. puis par ordre alphabétique du nom.

Le catalogue de la V1 n'a pas d'engrais vert (S02). Quand il en aura, ils passeront toujours en dernier.

Toucher une suggestion ouvre la création de série avec la culture, le mode, la date, la zone Z et la fin proposée déjà remplis.

Le gel n'est pas pris en compte en V1 (S01, 2.2) : les fenêtres de mise en place du catalogue en tiennent déjà compte.

---

## 10. Fenêtre de mise en place

Chaque culture a **une seule fenêtre** (S01, F-CAT-2).

- **Fenêtre sur une même année** (`debut ≤ fin`) : `[Y-debut, Y-fin]` pour chaque année Y.
- **Fenêtre à cheval sur deux années** (`debut > fin`, ail par exemple) : `[Y-debut, (Y+1)-fin]`.

---

## 11. Rappels et tâches

- **Familles des 2 saisons précédentes** (panneau planche) : ce sont les familles des séries de la planche dont l'occupation touche les années Y−1 ou Y−2, où Y est l'année du curseur.
- **Tâches** : chaque série produit une à trois tâches, chacune avec un type, une date et un **identifiant stable** `<id de la série>/<type>`, qui ne change pas quand la date bouge.

  | Mode | Tâches (type · date) |
  |---|---|
  | Semis à l'abri | `semer-abri` · semis, `repiquer` · `m` |
  | Plants | `planter` · `m` (« Acheter et planter ») |
  | Pleine terre | `semer-terre` · `m` |
  | Tous | `recolter` · `r` (« Commencer à récolter ») |

- **Calendrier des tâches** (export PDF) : les tâches regroupées par mois.
- **Après la V1**, un export iCal (.ics) en fera un événement par tâche, avec l'identifiant stable comme `UID`. Un agenda pourra ainsi mettre à jour ses événements au lieu de les créer en double.

---

## 12. Cas de test

Les données viennent de la S02. T10 et T13 évaluent une culture pour un créneau (§9, étapes 1 à 5), sans passer par le classement. La planche fait 1,2 m de large, avec des cases de 50 cm. Sauf mention contraire, aujourd'hui = 2026-11-14.

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
| T9 | Carotte, semis à l'abri le 2027-03-01, 21 jours en godets saisis | `m` 2027-03-22 · `r` 2027-06-30 · `f` 2027-08-09 |
| T10 | Créneau [1,6] du 2027-05-01 au 2027-11-15, aujourd'hui = 2027-04-20 | La tomate est proposée **en plants** (son semis à l'abri tomberait le 2027-03-06, dans le passé) : `m` 2027-05-01 · `r` 2027-07-15 · `f` 2027-09-23 |
| T11 | Planche de 6 tronçons : carotte T3 sur [1,3], poireau semé à l'abri le 2027-05-01 sur [4,6] ; planche supprimée à partir de 2028 | Poireau : `m` 2027-07-10 · `r` 2027-11-07 · `f` 2028-02-05 → **supprimé** (à cheval sur 2028). Carotte gardée |
| T12 | Séries de T6 ; on glisse la laitue de −1 jour | Refusé : la laitue démarrerait le 2027-09-26, en conflit avec la tomate sur [1,2] |
| T13 | Créneau [1,6] du 2027-08-02 au 2027-10-05 (65 j) | Laitue : `m` 2027-08-02 · `r` 2027-09-21 · `f` proposée 2027-10-06 au lieu du 2027-10-11, enregistrée comme **fin modifiée à la main** |

Avec un potager vide, le classement de T10 donne poireau (score 1,0), puis carotte et courgette (0,7) : la tomate, au même score arrondi, vient après elles par ordre alphabétique et n'est pas dans les 3 proposées.

---

## 13. Validation après fusion (lot 3)

Au lot 3, les modifications de plusieurs appareils sont fusionnées (S05, §6). Le document fusionné converge, mais il peut enfreindre les règles du plan. Après chaque fusion, l'appli vérifie le document fusionné.

**Comportement par défaut, pour tous les cas** (S05, §10) : **rien n'est supprimé ni corrigé automatiquement**. Chaque incohérence trouvée s'affiche comme « à résoudre », et l'utilisateur la règle lui-même.

| Cas | Règle de détection | Comportement par défaut |
|---|---|---|
| **Trop de planches** | Pour une année Y, la disposition de Y (section 7) contient plus de 10 planches (S01, F-PLN-1) | Toutes les planches de Y sont gardées. L'année Y est signalée « à résoudre », avec le nombre de planches en trop |
| **Série orpheline** | La planche de la série n'existe pas dans la disposition d'au moins une année que touche son occupation `[m, f[` | La série est gardée. Elle est signalée « à résoudre » |
| **Chevauchement** | Deux séries de la même planche sont en conflit selon la règle de la section 5 | Les deux séries sont gardées. La paire est signalée « à résoudre », avec les tronçons et la période en commun |
| **Année passée modifiée** | La fusion apporte une modification de la disposition d'une année Y, alors que Y est passée à la date de la fusion (Y < année d'aujourd'hui) | La modification est gardée. L'année Y est signalée « à résoudre » |

Les trois premiers cas se lisent sur l'état du document fusionné. Le quatrième dépend de ce que la fusion a apporté, pas seulement du résultat (voir question 5).

### Après un import (lot 1)

La question 7 ci-dessus est tranchée pour l'import, seul cas de plan reçu avant le lot 3. `lire_import` et `importer_potager` vérifient le document reçu, en deux niveaux :

| Niveau | Cas | Comportement |
|---|---|---|
| **Refus** (`import_invalide`) | Fichier qui n'est pas du JSON, enveloppe ou schéma du plan qui ne se lit pas, terrain invalide (dimensions, allée, case ≠ 25, 50, 100), longueur de planche nulle ou qui n'est pas un nombre entier de cases, série invalide seule (zone, godets, fin à la main, occupation vide) | Rien n'est lu ni remplacé. Un `format_version` plus récent que l'appli est refusé à part (`import_trop_recent`) |
| **Accepté, à résoudre** | Plus de 10 planches une année ; série orpheline (planche inconnue, ou absente d'une année occupée) ; zone qui dépasse la planche ; chevauchement de deux séries ; planche hors du terrain, qui en chevauche une autre ou trop proche (section 15) ; série d'un légume absent du catalogue ; note rattachée à une planche inconnue ou dont le texte ou les dates sont invalides | Le comportement par défaut ci-dessus s'applique : **rien n'est supprimé ni corrigé**. `lire_import` renvoie la liste des incohérences, l'interface la montre avant la confirmation (« Remplacer quand même »), et le plan est importé tel quel |

Le critère du refus : ce que les calculs de `plan` ne peuvent pas traverser sans risque (division par une longueur nulle, zone inversée). Tout le reste se lit sur l'état du document et se règle ensuite avec les actions habituelles. Les années passées sont vérifiées comme les autres : un fichier importé n'est pas une modification d'une année passée (cas « Année passée modifiée », propre à la fusion). **La liste reste visible après l'import** (v0.9). `lire_import` la montre à la confirmation ; ensuite, `incoherences_plan` la recalcule sur le plan de l'appareil, après le chargement de l'appli et après chaque modification. L'interface l'affiche dans un bandeau « À résoudre : N » au-dessus de l'écran, repliable, tant que la liste n'est pas vide. Elle se vide toute seule quand chaque point est réglé avec les actions habituelles : aucun bouton « Résoudre », rien n'est mémorisé (S05 §5 interdit de stocker des données dérivées).

### Questions ouvertes

1. **Affichage** : tranchée pour le lot 1 (import) : un bandeau repliable, section 13 « Après un import ». Reste ouverte pour la fusion (lot 3) : repère sur la frise et la vue jardin.
2. **Résolution** : quelles actions propose-t-on pour régler chaque cas (supprimer, déplacer, choisir une version…) ? Ou bien l'utilisateur corrige-t-il seulement avec les actions habituelles ?
3. **Blocage** : tant qu'une incohérence existe, les autres actions restent-elles possibles ? Par exemple, peut-on ajouter une série sur la 11e planche, ou modifier une planche en chevauchement ?
4. **Série orpheline** : où la montrer, puisque sa planche n'apparaît ni dans la vue jardin ni dans la frise pour les années concernées ?
5. **Année passée modifiée** :
   - Une modification faite quand l'année était encore en cours est-elle légitime, ou doit-elle être signalée quand même ?
   - Quelle date fait foi : celle de la modification sur l'appareil d'origine (son horloge peut être fausse), ou celle de la fusion ?
   - Ce cas ne se déduit pas de l'état du document. Il demande l'historique des modifications du CRDT, ou de stocker une trace, alors que S05 §5 interdit de stocker des données dérivées.
   - Comment résoudre ce cas, puisque l'atelier ne permet pas de modifier une année passée (S01, F-PLN-4) ?
6. **Autres cas possibles** : faut-il ajouter les cas suivants à la liste ?
   - Une planche est raccourcie sur un appareil pendant que l'autre y ajoute une série sur les derniers tronçons. La zone dépasse alors la planche (`t2 > n`, section 4).
   - Deux planches qui se chevauchent sur le plan, ou une planche qui sort du terrain, après deux déplacements faits en même temps (S01, F-PLN-1).
7. **Moment de la vérification** : tranchée pour le lot 1 : au chargement et après chaque modification, pas seulement à l'import (section 13 « Après un import »). Reste ouverte pour la fusion.

---

## 14. Décisions

| Sujet | Décision |
|---|---|
| Dates | Dates civiles pour le plan, UTC pour les horodatages techniques (section 1) |
| Mode abri pour un légume sans phase en godets | L'utilisateur saisit la durée en godets (section 2) |
| Date enregistrée en mode abri | Toujours la date de semis (section 2) |
| Durées d'une série | Copiées du catalogue à la création (section 2) |
| Glisser une barre en conflit | Refusé, la barre revient à sa place (section 2) |
| Taille d'un tronçon | La taille de case, fixée à la création du terrain (section 4) |
| Largeur des planches | Libre au centimètre ; seule la longueur suit la grille (section 4) |
| Suppression d'une planche | Les séries à cheval sur l'année de suppression sont supprimées (section 5) |
| Report des modifications du plan | Jusqu'à la prochaine disposition enregistrée de la même planche (section 7) |
| Récolte minimale des suggestions | La moitié de la durée de récolte (section 9) |
| Plan importé qui enfreint les règles | Refus si les calculs ne peuvent pas le lire ; sinon accepté tel quel, avec la liste des incohérences à résoudre (section 13) |
| Texte d'une période | 80 caractères au plus, comme une note (section 16) |
| Annotation « dans le passé » | Elle se termine avant l'année d'aujourd'hui (section 16) |
| « Vider le plan » | Les planches de l'année affichée, leurs séries et leurs notes à partir de cette année ; le terrain et les notes du potager entier restent (section 16) |
| Semis proposé dans le passé | On propose des plants (section 9) |
| Fin raccourcie par une suggestion | Elle compte comme une fin modifiée à la main (section 9) |
| Changer le mode d'une série | Recopie la durée sous abri depuis le catalogue (section 2) |
| Bornes des stades | Chaque tranche inclut sa borne basse (section 3) |
| Arrondi des raccourcis | À l'entier supérieur pour une moitié (section 4) |
| Série « terminée avant D » | `f ≤ D` (section 9) |
| Suppression d'une planche | Efface aussi ses dispositions des années suivantes (section 5) |
| Gel et ensoleillement | Hors V1 ; le calcul d'ensoleillement est gardé en annexe A |
| Dimensions du terrain | 1 à 100 m par côté, au moins une case (section 15) |
| Allées | 30, 50 ou 80 cm ; un écart égal à l'allée est accepté, mesuré sur un axe (section 15) |
| Planche au bord du terrain | Autorisée : l'allée n'est exigée qu'entre planches (section 15) |
| Allée trop étroite | Refusée comme un chevauchement en v1, sans bulle « Garder ici » (section 15) |
| Coordonnées | Libres en cm, la grille est une aide de l'interface (section 15) |
| Contrôle sur plusieurs années | De l'année modifiée à la dernière disposition enregistrée (section 15) |
| Nom, largeur minimale | 1 à 30 caractères, 10 cm (section 15) |
| Liste « à résoudre » après un import | Bandeau repliable, recalculé par `incoherences_plan` au chargement et après chaque modification, vide quand tout est réglé (section 13) |


---

## 15. Terrain et placement des planches

Tim n'étant pas disponible, ces règles ont été tranchées pendant l'écriture de l'atelier (voir les Décisions). Elles complètent S01 F-TER-1, F-PLN-1 et F-PLN-2.

**Terrain**
- Chaque côté fait **au moins 1 m et au plus 100 m**, et au moins une case.
- La largeur d'allée vaut **30, 50 ou 80 cm**.
- Un terrain modifié (dimensions ou allée) est refusé si, pour l'année en cours ou une année suivante, une planche sort du terrain, en recouvre une autre ou ne respecte plus l'allée.

**Planche**
- Le nom compte **1 à 30 caractères** (espaces de début et de fin ignorés).
- La largeur est d'**au moins 10 cm** ; la longueur reste un nombre entier de cases (section 7).
- Les coordonnées `(x, y)` sont celles du coin haut gauche, en cm depuis le coin haut gauche du terrain. Aucune grille n'est imposée par Rust : le magnétisme est une aide de l'interface.
- Une planche **tient dans le terrain** si `x + longueur ≤ longueur du terrain` et `y + largeur ≤ largeur du terrain`. Elle peut toucher le bord : l'allée n'est exigée qu'entre deux planches.
- Pour deux planches a et b, on calcule l'écart sur chaque axe, `dx = max(b.x − (a.x + a.l), a.x − (b.x + b.l))` et `dy` de même sur y.
  - **Chevauchement** : `dx < 0` et `dy < 0`.
  - **Allée trop étroite** : pas de chevauchement, mais `dx < allée` et `dy < allée`. Un écart exactement égal à l'allée est accepté. L'écart est mesuré sur un axe, pas en diagonale.
- Ces règles sont vérifiées **pour chaque année**, de l'année modifiée à la dernière année qui a une disposition enregistrée (une disposition vaut jusqu'à la suivante, section 7).
- Un refus ne modifie rien. Codes d'erreur : `terrain_invalide`, `allee_invalide`, `largeur_invalide`, `nom_invalide`, `planche_hors_terrain`, `planche_chevauche`, `allee_trop_etroite`.
- Position refusée : en v1 l'allée trop étroite est refusée comme un chevauchement. La bulle « Garder ici » de S01 F-PLN-2 est hors lot 1.


---

## Annexe A — Ensoleillement (après la V1)

Ce calcul n'est pas fait en V1 (S01, 2.2) : il demande l'orientation du terrain, la latitude et des obstacles, qui viendront avec lui.

On compte le nombre d'**heures de soleil direct** au centre de chaque case, le **21 juin** et le **20 mars**, puis on fait la **moyenne** des deux journées. Le 20 mars, plus bas, rend compte des ombres longues du printemps.

| Moyenne des heures de soleil | Niveau |
|---|---|
| ≥ 6 h | Plein soleil |
| 3 – 6 h | Mi-ombre |
| < 3 h | Ombre |

**Position du soleil**, calculée toutes les 15 minutes de 4 h à 20 h, en heure solaire :
- déclinaison `δ = +23,44°` le 21 juin, et `0°` le 20 mars ;
- angle horaire `H = 15° × (t − 12)`, avec t en heures ;
- hauteur `a` : `sin a = sin φ · sin δ + cos φ · cos δ · cos H`, où φ est la latitude (44,84° pour Bordeaux) ;
- azimut depuis le nord `Az` : `cos Az = (sin δ − sin a · sin φ) ÷ (cos a · cos φ)`. On garde `Az` le matin (H < 0) et on prend `360° − Az` l'après-midi ;
- on ne compte que les instants où `a > 5°`, car un soleil plus bas ne compte pas pour la culture.

**Ombre portée** : un obstacle est un pavé de hauteur h, posé sur son emprise au sol. L'orientation du terrain sert à exprimer `Az` dans les axes de la grille. Le point P est **à l'ombre** à un instant donné si le segment au sol qui part de P vers le soleil, de longueur `h ÷ tan a`, coupe l'emprise d'un obstacle. Les arbres sont traités comme des pavés. Les planches ne font pas d'ombre.

**Ensoleillement d'une planche** : c'est le niveau de la majorité de ses cases, et en cas d'égalité le niveau le plus ensoleillé.

---

## 16. Notes et périodes

Règles vérifiées par `crates/plan` avant tout enregistrement (S01 F-VIS-5) :

- **Année passée en lecture seule** : refusée (`annee_passee`) si l'année affichée est avant l'année d'aujourd'hui, ou si l'annotation, à créer ou à modifier, se termine avant l'année d'aujourd'hui (une période qui commence l'an dernier et finit cette année est permise). Modifier, masquer ou supprimer une annotation déjà passée est refusé de la même façon.
- **Planche** : si elle est donnée, elle doit exister dans le plan (`planche_inconnue`) ; elle peut ne plus exister l'année affichée.
- **Texte** : 1 à 80 caractères une fois les espaces retirés, pour une note comme pour une période (`texte_invalide`).
- **Dates** : une période a une fin, le même jour que son début ou après (`annotation_dates_invalides`). Une note n'a pas de fin : si elle en reçoit une, elle est ignorée.
- **Masquer** est une modification (`masquee`), soumise aux mêmes règles.
- **Vider le plan** (S01 F-PLN-4) : à partir de l'année affichée (pas une année passée), supprime toutes les planches qui existent cette année-là, avec leurs séries comme `supprimer_planche` (section 5), et les annotations rattachées à ces planches qui se terminent le 1er janvier de l'année ou après. Le terrain, les années passées, les séries déjà terminées et les annotations de tout le potager sont gardés.
