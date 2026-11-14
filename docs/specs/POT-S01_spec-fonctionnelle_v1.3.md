# Potagerie — Spécification fonctionnelle V1

> Version : 1.3 (brouillon) · Date : 14 novembre 2026 · Auteur : Tim · Domaine : potagerie.garden
> Remplace la v1.2, archivée dans `archives/`. Stack et lots : [S05](POT-S05_spec-technique_v0.6.md). Règles et calculs : [S03](POT-S03_regles-calculs_v0.5.md). Catalogue : [S02](POT-S02_catalogue-cultures_v0.2.md).
> v1.3 : trois précisions révélées par D02 (écrans complémentaires), voir §11.

---

## 1. Vision

Potagerie est une **appli installée sur ordinateur et téléphone** pour **concevoir, planifier et visualiser un potager**. Elle n'a ni serveur ni compte : un seul utilisateur, et ses données restent sur ses appareils.
On dessine son terrain, on y place ses planches, puis on organise ses cultures dans le temps. Le potager s'affiche comme un **petit diorama isométrique** qui pousse au fil de la saison.

Potagerie est un **outil de planification**, pas un jeu. Son rendu est doux, apaisant et satisfaisant à regarder.

### 1.1 Nom et domaine

- **Potagerie** : un nom à consonance française, facile à prononcer à l'international, plus distinctif que « Potager » seul, déjà utilisé par plusieurs applis et marques.
- Domaine visé : **potagerie.garden**. Sa disponibilité reste à vérifier avant l'achat. Il n'héberge pas l'appli.

### 1.2 Objectifs

- Concevoir le plan d'un **futur potager** à partir d'un terrain vierge.
- Planifier une saison complète de cultures, successions comprises.
- **Visualiser** le potager à n'importe quelle date et voir la saison défiler.
- Être aidé pour remplir les planches vides (créneaux libres et suggestions).
- Retrouver son potager sur tous ses appareils : par export et import avant le lot 3, par synchronisation au lot 3.

### 1.3 Utilisateur cible

Un jardinier amateur, seul, qui prépare un potager familial (quelques planches, de 10 à 100 m²). Il planifie surtout l'hiver sur ordinateur et consulte au printemps et en été sur téléphone (lot 2).

---

## 2. Périmètre

### 2.1 Dans la V1

| Domaine | Contenu |
|---|---|
| Données | Pas de compte, un seul utilisateur ; enregistrement local automatique ; export et import du potager pour passer d'un appareil à l'autre ; synchronisation entre appareils au lot 3 |
| Terrain | Dimensions, taille de case |
| Plan | Un seul plan par utilisateur : planches et carrés, allées |
| Catalogue | Les 6 légumes de S02, préremplis, non modifiables en V1 |
| Planification | Séries de cultures, successions, calcul automatique des dates et des quantités |
| Visualisation | Vue jardin isométrique datée, frise (Gantt) continue sur plusieurs années, curseur de temps, lecture de la saison |
| Aide | Créneaux libres et suggestions de cultures |
| Temps | Frise continue d'une année à l'autre, historique conservé, une disposition du plan par année |
| Export | Frise des cultures en PDF et en image PNG |

La V1 est livrée en trois lots (S05, §9) :

| Lot | Contenu |
|---|---|
| **1 · Bureau** | Appli sur ordinateur (macOS, Windows). La mise en page téléphone est déjà faite |
| **2 · Mobile** | Appli sur téléphone (iOS, Android) ; transfert entre appareils par export et import |
| **3 · Synchro** | Le même potager sur tous les appareils, sans serveur |

### 2.2 Hors V1

- Suivi du réel : dates effectives, « semé », « récolté », recalage automatique.
- Journal de récoltes, quantités, photos, notes.
- Météo en direct, notifications et rappels.
- Partage ou collaboration, potager public.
- Plusieurs potagers ou plusieurs plans (versions).
- Catalogue personnalisable : variétés, durées modifiées, légumes ajoutés.
- Toute mécanique de jeu : points, niveaux, quêtes, déblocages.
- **Ensoleillement** : orientation du terrain, latitude, obstacles qui font de l'ombre (maison, mur, haie, arbre, abri), carte d'ensoleillement. Le calcul est déjà décrit (S03, annexe A).
- **Décor** : composteur, récupérateur d'eau, serre, chemin, banc.
- **Climat** : dates de gel, périodes de gel dans la frise.
- **Alertes** : gel, période de mise en place, rotation, associations, exposition.
- **Catalogue élargi** : une trentaine de légumes, engrais verts compris, avec l'exposition et les bons et mauvais voisins.
- **Album des saisons** : une image figée du potager à son apogée, gardée chaque année.
- Export de la frise en CSV.
- Export du calendrier en iCal (.ics). La conception le prépare déjà (S03, §11).
- Plusieurs fenêtres de mise en place pour un même légume.

---

## 3. Concepts et vocabulaire

| Terme | Définition |
|---|---|
| **Terrain** | La surface totale disponible, rectangulaire en V1, découpée en grille. |
| **Case** | Unité de la grille, de 25, 50 ou 100 cm de côté. Elle est choisie à la création du terrain et ne change plus ensuite. |
| **Plan** | La disposition des planches sur le terrain. **Un seul plan par utilisateur**, qui évolue de saison en saison : chaque saison garde la disposition qu'elle avait. |
| **Planche** | Une zone cultivable rectangulaire : planche, carré, butte ou bac. |
| **Culture** | Une fiche du catalogue, par exemple « Tomate ». |
| **Série** | Une culture placée sur une zone d'une planche, sur une période. C'est l'objet central de la planification. |
| **Tronçon** | Une portion de la longueur d'une planche, de la taille d'une case. Une zone est une suite de tronçons contigus. |
| **Saison** | Une année civile. Elle ne découpe pas la frise, qui est continue : elle sert seulement à dater les dispositions du plan. |
| **Mode de démarrage** | La façon de démarrer une culture : semis à l'abri, achat de plants, ou semis en pleine terre. |
| **Mise en place** | Le jour où la culture arrive dans la planche : repiquage, plantation des plants achetés, ou semis en pleine terre. |
| **Créneau libre** | Une période pendant laquelle une planche n'a aucune série. |

---

## 4. Fonctionnalités

### 4.1 Données et synchronisation

**F-CPT-1 — Connexion** : supprimée. L'appli n'a pas de compte.

**F-CPT-2 — Enregistrement et synchronisation**
- Chaque modification est enregistrée automatiquement sur l'appareil, sans bouton « Sauvegarder ». L'appli fonctionne toujours sans réseau.
- Un indicateur discret affiche l'état : « Enregistré », « Enregistrement… », ou, si l'écriture échoue (disque plein, dossier de données inaccessible…), « Erreur d'enregistrement » avec un bouton « Réessayer ». La modification reste affichée et modifiable tant que l'enregistrement n'a pas réussi, pour qu'elle ne soit pas perdue (§8).
- **Lot 3** : les modifications faites sur plusieurs appareils sont fusionnées (S05, §6). Les ajouts faits en même temps sur deux appareils sont tous conservés ; sur un même champ, la dernière écriture gagne. Les incohérences qui en résultent sont traitées par la validation après fusion (S03, §13). Un indicateur affiche l'état de la synchro.

**F-CPT-3 — Export et import du potager**
- Export complet du potager dans un fichier, à la fois comme sauvegarde et pour passer d'un appareil à l'autre.
- L'import d'un export remplace le potager existant, après confirmation.
- Avant le lot 3, c'est le **transfert entre appareils**. L'import prévient si le potager reçu est plus ancien que celui de l'appareil (S05, §6).

**F-CPT-4 — Effacer les données de l'appli**
- Efface toutes les données du potager sur l'appareil, après double confirmation.

---

### 4.2 Terrain

**F-TER-1 — Création du terrain**
- Saisie de la longueur et de la largeur en mètres.
- Choix de la taille de case : 25, 50 ou 100 cm (50 cm par défaut). Un texte prévient qu'elle ne pourra plus être changée.
- Choix de la largeur d'allée minimale (F-PLN-2) : 30, 50 ou 80 cm (50 cm par défaut). Une valeur de départ est nécessaire pour l'atelier du plan (F-PLN-6), mais elle reste modifiable à tout moment dans les réglages (E10).

Obstacles (F-TER-2), carte d'ensoleillement (F-TER-3) et climat (F-TER-4) sont hors V1 (voir 2.2).

---

### 4.3 Plan

**F-PLN-1 — Planches**
- **10 planches au maximum** par saison. Un compteur « Planches 8 / 10 » est visible dans l'atelier, et les planches du bac d'éléments sont grisées à 10, avec un message qui l'explique.
- Types de planche : **planche** en pleine terre (1,2 × 3 m par défaut), **carré** (1 × 1 m), **butte** (1,2 m de large), **bac** surélevé (1,2 × 2 m). Le type ne change que l'apparence et la taille de départ : toutes se planifient de la même façon.
- **Dimensions** : la longueur est un nombre entier de cases, car elle est découpée en tronçons. La largeur est libre, au centimètre près, pour garder les 1,2 m classiques.
- Chaque planche a un **nom libre** (30 caractères au plus), par exemple « Les tomates » ou « Côté cabane ». À la création, elle reçoit un nom par défaut (« Planche A », « Carré B »…) que l'on peut changer à tout moment, directement dans le panneau planche.
- Le nom s'affiche partout où la planche apparaît : étiquette dans la vue jardin, ligne de la frise, panneau planche, création de série et exports. Deux planches peuvent porter le même nom, mais l'appli le signale.
- Chaque planche est déplaçable et redimensionnable (voir F-PLN-6).
- Deux planches ne peuvent pas se chevaucher. Une planche ne peut pas sortir du terrain.

**F-PLN-6 — Placer et dimensionner les planches**

Tout se fait dans l'**atelier du plan** (E3), en vue de dessus sur la grille. L'appli a **deux modes**, toujours accessibles par les onglets « Potager » et « Plan » en haut de l'écran : « Plan » ouvre l'atelier, « Potager » ramène à la vue jardin (le bouton « Voir le potager » de l'atelier fait de même).

L'atelier n'a **pas d'outils à choisir** : on manipule directement les planches.

- **Ajouter** : un **bac d'éléments** sous le plan propose Planche, Carré, Butte et Bac. On attrape un élément et on le lâche sur le terrain : il arrive à sa taille par défaut, reçoit un nom par défaut et reste sélectionné. Le compteur « Planches 8 / 10 » est affiché sur le bac, et ses éléments sont grisés à 10.
- **Sélectionner** : toucher ou cliquer une planche. Toucher l'herbe désélectionne.
- **Déplacer** : glisser la planche à la souris, ou au doigt après un appui long sur téléphone.
- **Redimensionner** : avec les 8 poignées de la planche sélectionnée, ou en saisissant longueur et largeur dans le panneau. La longueur avance case par case ; la largeur se règle au centimètre.
- **Tourner** d'un quart de tour, **dupliquer**, **supprimer** : depuis la barre qui apparaît sous la planche sélectionnée, ou depuis le panneau.
- **Panneau de droite** : il montre toujours la planche sélectionnée (nom, type, longueur, largeur) et, sans sélection, le bilan du plan (F-PLN-5).
- **Aides au placement** :
  - magnétisme sur la grille et sur les bords des planches voisines, avec un guide en pointillés ;
  - cotes en direct jusqu'aux voisines et aux bords du terrain ;
  - position d'origine en pointillés pendant le glisser.
- **Année** : l'atelier modifie le plan de l'année du curseur (voir F-PLN-4). L'année est rappelée en haut de l'atelier (« Plan 2027 »).
- **Positions refusées** (chevauchement, sortie du terrain) : la planche passe en rouge pâle et revient à sa place si on la lâche.
- Les séries et annotations **restent attachées** à la planche quand on la déplace. Si on la raccourcit, les séries qui ne tiennent plus sont signalées avant de valider.
- **Annuler / Rétablir** pour chaque action, avec les boutons de l'en-tête.

**F-PLN-2 — Allées**
- Largeur d'allée minimale réglable : 30, 50 ou 80 cm.
- Lors du placement, l'appli garde cette largeur entre deux planches. Si on lâche la planche plus près, une bulle propose « Garder ici » ou « Revenir ».

Le décor (F-PLN-3) est hors V1 (voir 2.2).

**F-PLN-4 — Un seul plan, qui évolue par saison**
- Chaque utilisateur a **un terrain et un plan**. Il n'y a ni versions ni plan actif à choisir.
- **Chaque année garde sa disposition.** L'atelier montre le plan de l'année où se trouve le curseur de temps. Les années passées ne changent jamais.
- **Report sur les années suivantes** : la disposition n'est enregistrée que pour les années où elle change. Une modification d'une planche faite sur l'année Y s'applique à Y et aux années suivantes, **jusqu'à la prochaine année où cette même planche a été modifiée**. Les autres planches ne sont pas touchées.
- **Saisons passées en lecture seule** dans l'atelier : un bandeau indique « Plan de 2026, en consultation ». Pour modifier, on ramène le curseur sur l'année en cours ou une année à venir (bouton « Aujourd'hui »).
- **Saison en cours** : on peut déplacer ou redimensionner une planche même si elle porte des cultures. La saison en cours prend directement la nouvelle disposition, et les cultures suivent leur planche.
- **Nouvelle saison** : elle reprend la disposition de la saison précédente.
- **Supprimer une planche** la retire de l'année affichée et des suivantes. Ses séries entièrement passées restent visibles dans leurs saisons. Ses séries qui touchent l'année affichée ou les suivantes, **y compris celles à cheval sur l'année précédente**, sont supprimées, après une confirmation qui indique leur nombre.
- Pour repartir de zéro : « Vider le plan », dans les réglages, après confirmation. Comme toute modification, cela s'applique à la saison en cours et aux suivantes.

**F-PLN-5 — Bilan du plan**
- Surface totale, surface cultivable, surface d'allées.
- Nombre de planches.

---

### 4.4 Catalogue des cultures

**F-CAT-1 — Contenu**
- **En V1, les 6 légumes de S02** : tomate, laitue, carotte, haricot vert, courgette, poireau.
- La liste sera allongée après la V1. Liste indicative : poivron, aubergine, pomme de terre, courge, concombre, melon, petit pois, fève, panais, betterave, radis, navet, oignon, ail, échalote, mâche, épinard, blette, chou, brocoli, basilic, persil, fraisier, engrais vert (phacélie).

**F-CAT-2 — Fiche d'une culture**

Le schéma est **figé pour la V1** ; il est détaillé dans S02.

| Champ | Exemple (tomate) | Usage |
|---|---|---|
| Nom | Tomate | Affichage |
| Famille botanique | Solanacées | Suggestions, rotation après la V1 |
| Espacement entre rangs | 80 cm | Calcul des quantités |
| Espacement sur le rang | 60 cm | Calcul des quantités |
| Durée sous abri | 8 semaines | Calcul de la date de repiquage |
| Durée jusqu'à la récolte | 75 jours après la mise en place | Phases |
| Durée de récolte | 70 jours | Phases |
| Fenêtre de mise en place | 1er mai – 15 juin | Période idéale, suggestions |
| Sensible au gel | Oui | Alertes de gel après la V1 |
| Illustrations | 4 stades | Vue jardin |

L'exposition et les bons et mauvais voisins viendront avec le catalogue élargi.

- Une culture a **une seule fenêtre** de mise en place.
- Une fenêtre peut chevaucher deux années (par exemple ail : 15 octobre – 15 mars).

**F-CAT-3 — Consultation**
- Liste filtrable par famille et par saison de mise en place (« de saison maintenant »).
- Fiche détaillée en lecture seule.

---

### 4.5 Planification

**F-PLA-1 — Créer une série**
- Depuis une planche (vue jardin ou frise) ou depuis un créneau libre suggéré.
- Trois questions, dans cet ordre :
  1. **Quel légume ?** Choisi dans le catalogue, avec les légumes de saison mis en avant.
  2. **Comment tu démarres ?** Trois cartes, toujours toutes disponibles, sans conseil propre à chaque légume : l'utilisateur choisit librement.
     - **Je sème à l'abri** : semis en godets, puis repiquage au jardin ;
     - **J'achète des plants** : plants de jardinerie, plantés directement ;
     - **Je sème en pleine terre** : semis directement dans la planche.
  3. **Quand ?** Une **seule date**, celle que l'utilisateur connaît, dont la question suit le mode : « Quand sèmes-tu ? » (abri ou pleine terre) ou « Quand plantes-tu ? » (plants). Pour un semis à l'abri, un lien « Je connais plutôt la date de repiquage » permet de saisir l'autre date : l'appli en déduit et enregistre la date de semis.
- La **planche** est connue d'avance (on part d'une planche ou d'un créneau libre) et reste modifiable.
  - **Zone de la planche** : la planche est découpée en **tronçons** de la taille d'une case sur sa longueur. On touche les tronçons occupés par la série, qui doivent être contigus. Des raccourcis sélectionnent toute la planche, ½, ⅓ ou ¼, en partant du premier tronçon libre. La zone choisie s'affiche aussi sur un petit schéma de la planche.
- Valeurs calculées et affichées immédiatement :
  - Une **mini-frise** des étapes avec leurs dates : godets (semis à l'abri seulement), en terre, récolte.
  - Date de **repiquage** pour un semis à l'abri (date de semis plus la durée sous abri).
  - **Début de récolte** (mise en place plus la durée jusqu'à la récolte).
  - **Fin de série** (début de récolte plus la durée de récolte), modifiable.
  - **Quantité estimée** : nombre de rangs × pieds par rang, d'après la longueur de la zone, la largeur de la planche et les espacements.
- Une série **garde les durées du catalogue** qu'elle avait à sa création : une mise à jour du catalogue ne décale jamais les séries existantes.

**F-PLA-2 — Modifier ou supprimer une série**
- Toutes les valeurs saisies restent modifiables.
- Si on modifie la date de mise en place, la fin se recalcule, sauf si elle avait été modifiée à la main.
- Sur la frise, on peut **glisser une barre** pour décaler la série. La durée est conservée. Si la nouvelle position crée un conflit, le décalage est refusé et la barre revient à sa place.
- La suppression demande une confirmation.

**F-PLA-3 — Successions**
- Plusieurs séries peuvent se suivre sur une même planche.
- Deux séries peuvent coexister sur une planche si leurs **zones ne se chevauchent pas** pendant la même période. Sinon, la validation est impossible et l'appli montre les tronçons et la période en conflit.

**F-PLA-4 — Un temps continu**
- Il n'y a **pas de sélecteur de saison** : la frise défile en continu d'une année à l'autre, et les séries à cheval sur deux années (ail, poireau d'hiver) apparaissent d'un seul tenant.
- Le diorama montre le plan **de l'année où se trouve le curseur** (F-PLN-4). Quand le curseur franchit le 1er janvier, il prend la disposition de la nouvelle année.
- Les cultures passées restent consultables et modifiables (le plan des années passées, lui, est figé). Elles alimentent le rappel des familles cultivées.

**F-PLA-5 — Calcul des quantités**
- Longueur de référence = la longueur de la zone (nombre de tronçons × taille d'un tronçon). Largeur = la largeur de la planche.
- Rangs = largeur ÷ espacement entre rangs, arrondi à l'entier inférieur, au minimum 1.
- Pieds par rang = longueur ÷ espacement sur le rang, arrondi à l'entier inférieur, au minimum 1.
- Le résultat est affiché comme une **estimation** (« ≈ 21 pieds, 1 rang × 21 »).

---

### 4.6 Visualisation

L'écran principal présente deux vues **synchronisées sur une même date**, le *curseur de temps*.

**F-VIS-1 — Vue jardin (isométrique)**
- Le potager, rendu comme un diorama isométrique posé sur un fond uni, avec la disposition du plan de l'année du curseur.
- Le terrain et les planches sont dessinés en volume simple.
- Chaque série en terre à la date du curseur est représentée par des plantes **sur sa zone** de la planche.
- **4 stades visuels** selon l'avancement :

  | Stade | Avancement depuis la mise en place |
  |---|---|
  | Semis | 0 – 15 % de la durée jusqu'à la récolte |
  | Jeune pousse | 15 – 50 % |
  | Adulte | 50 – 100 % |
  | Récolte | Au-delà de 100 % (fruits ou légumes visibles) |

- Une série semée à l'abri, pas encore repiquée, est signalée par une **petite étiquette** plantée sur la planche. Une série en plants achetés apparaît directement au stade « jeune pousse ».
- **Ambiance saisonnière** : la couleur de l'herbe et du ciel suit la date (givre en hiver, fleurs au printemps, tons dorés à l'automne).
- Navigation :
  - zoom par pincement (téléphone), molette ou pavé tactile (ordinateur), sans bouton ;
  - déplacement en faisant glisser le fond ;
  - bouton « Tourner la vue » pour faire pivoter le potager d'un quart de tour, et voir ce que cachent les grandes plantes.
- L'onglet « Plan », en haut de l'écran, ouvre l'atelier du plan (E3).
- Un toucher ou un clic sur une planche ouvre son panneau de détail.

**F-VIS-2 — Vue frise (Gantt)**
- **Frise continue** : on la fait glisser (souris ou doigt) pour remonter ou avancer dans le temps, sans coupure entre les années. L'année est rappelée au-dessus des mois, et le passage au 1er janvier est marqué d'un trait.
- **Échelle** : par défaut, 12 mois sont visibles. On peut resserrer jusqu'à 3 mois ou élargir jusqu'à 3 ans, par pincement ou à la molette.
- Une ligne par planche, triée selon sa position sur le plan. Une planche qui n'existe pas encore (ou plus) sur une période a sa ligne grisée sur cette période.
- Une barre par série, découpée en **phases** visuellement distinctes : godets (hachuré, semis à l'abri seulement), croissance (couleur atténuée), récolte (couleur pleine). Une série en plants achetés porte une petite icône « plant » au début de sa barre.
- Les séries qui se chevauchent sur une même planche sont empilées sur plusieurs sous-lignes.
- En fond : colonnes des mois et repère « aujourd'hui ».
- Un toucher ou un clic sur une barre ouvre l'édition de la série. Un glisser décale la série (F-PLA-2).
- Sur téléphone (lot 2), le doigt fait aussi défiler la frise : on décale une barre après un **appui long**, comme dans l'atelier.

**F-VIS-3 — Curseur de temps**
- Un repère vertical sur la frise et un sélecteur de date sous la vue jardin, les deux liés.
- On peut le faire glisser, toucher ou cliquer sur la frise, ou avancer d'une semaine avec les boutons ‹ et › placés de part et d'autre de la date.
- À l'ouverture, le curseur est sur **aujourd'hui**. Un bouton « Aujourd'hui », en haut de l'écran, y ramène à tout moment.

**F-VIS-4 — Lecture de la saison**
- Le bouton « ▶ Jouer la saison » fait avancer le curseur sur 12 mois à partir de sa position, en une quinzaine de secondes. La frise défile avec lui.
- Les plantes poussent et changent de stade avec une **animation douce** : elles sortent de terre et grandissent avec un léger rebond.
- On peut mettre en pause, reprendre, et choisir la vitesse (×1, ×2).
- Si le système demande de réduire les animations, la lecture avance par paliers, sans transition.

**F-VIS-5 — Annotations de la frise**
- On peut annoter la frise avec deux types d'annotations :
  - **Note** : un repère à une date, avec un texte court (80 caractères au plus), par exemple « Pailler les tomates ». Elle se rattache à **une planche** (sa ligne dans la frise) ou à **tout le potager**.
  - **Période** : une plage de dates nommée, par exemple « Vacances, 1er → 21 août » ou « Absent ». Elle couvre une planche ou tout le potager, et s'affiche en bande colorée derrière les barres.
- Création :
  - bouton « Annoter » dans l'en-tête de la frise ;
  - un double-clic (ordinateur) ou un appui long (téléphone) sur une ligne, à la date voulue ;
  - depuis le panneau planche (« Ajouter une note »).
- Une annotation se modifie, se déplace en la glissant, et se supprime.
- Les annotations d'une planche s'affichent aussi dans son panneau, triées par date.
- Une annotation peut être masquée de la frise sans être supprimée (case « Afficher les annotations » de la frise).
- Les annotations font partie de l'export de la frise (option « Annotations » cochée par défaut).

**F-VIS-6 — Disposition des écrans**
- **Navigation principale** : deux onglets en haut de l'écran, « Potager » (E5) et « Plan » (E3).
- **Ordinateur** (lot 1) : la vue jardin occupe le haut de l'écran, la frise est en dessous, dépliable. Un panneau latéral affiche le détail (planche, série).
- **Téléphone** (lot 2) : la vue jardin est en plein écran avec le curseur de temps en bas. La frise et le panneau de détail s'ouvrent en feuilles coulissantes.
- La mise en page s'adapte à la largeur de l'écran **dès le lot 1**, pour que le passage au téléphone au lot 2 ne demande pas de refaire l'interface.

**F-VIS-7 — Panneau planche**
- Nom (modifiable sur place), dimensions, surface.
- Notes de la planche, triées par date.
- Séries de l'année du curseur, avec leurs dates et quantités.
- Familles cultivées les 2 saisons précédentes, simple rappel pour la rotation.
- **Historique** : les séries des années passées sur cette planche (culture, dates, quantité), en lecture seule, repliable.
- Actions : « Planter ici », déplacer (raccourci vers l'atelier du plan, planche déjà sélectionnée), modifier, supprimer.

---

### 4.7 Créneaux libres et suggestions

Pas d'alertes en V1 (voir 2.2). L'appli aide seulement à remplir les planches vides.

**F-CRE-1 — Détection des créneaux libres**
- Pour chaque planche, l'appli repère les périodes **sans série de plus de 35 jours** entre le 1er mars et le 15 novembre de chaque année.
- Dans la frise, un créneau libre apparaît en pointillés. Dans la vue jardin, un panneau « + Libre » est planté sur la planche.

**F-CRE-2 — Suggestions**
- Pour chaque créneau, l'appli propose **jusqu'à 3 cultures** :
  - mise en place possible dans les 4 premières semaines du créneau, dans la fenêtre idéale de la culture ;
  - durée jusqu'à la récolte, plus un minimum de récolte, compatible avec la longueur du créneau.
- Le mode proposé est le semis à l'abri si la culture en prévoit un. Si la date de semis tombe déjà dans le passé, l'appli propose plutôt des plants.
- Classement : bon remplissage du créneau d'abord, puis une famille différente de la culture précédente sur la planche.
- Un toucher sur le créneau ouvre la création de série, avec les suggestions en tête de liste et les dates préremplies. Si la fin proposée est raccourcie pour tenir dans le créneau, elle compte comme une fin modifiée à la main : elle ne se recalcule pas.

---

### 4.8 Export de la frise

**F-EXP-1 — Export de la frise (Gantt)**
- Accessible depuis le bouton « Exporter la frise », dans l'en-tête de la frise.
- Formats :
  - **PDF**, pour imprimer et afficher dans l'abri de jardin : A4 ou A3, paysage, une ou plusieurs pages selon le nombre de planches.
  - **Image PNG**, pour partager ou garder une trace.
- Options :
  - **Période** : une plage de dates libre. Par défaut, l'année civile où se trouve le curseur.
  - **Planches** : toutes, ou une sélection.
  - **Contenu** : afficher ou non les phases (abri, croissance, récolte), le repère « aujourd'hui » et les annotations.
- Le rendu exporté reprend le style visuel de l'appli, adapté à l'impression : fond clair, couleurs lisibles en noir et blanc grâce aux motifs de phase.

**F-EXP-2 — Contenu du PDF et de l'image**
- En-tête : nom du potager, période exportée, date d'export.
- La frise : une ligne par planche, les mois en colonnes, une barre par série avec le nom de la culture.
- Légende : phases et annotations.
- **Miniature du plan** : la vue de dessus du potager, avec le nom des planches, pour retrouver chaque ligne sur le terrain.
- Option **« Calendrier des tâches »** : la liste mois par mois, déduite des séries, de ce qu'il faut faire : semer à l'abri, acheter des plants, semer en pleine terre, repiquer, commencer à récolter.

---

## 5. Direction visuelle

- **Isométrique doux, proche du low-poly** : formes simples et arrondies, pas de réalisme. Le potager ressemble à une petite île posée sur un fond uni.
- **Palette pastel et apaisante** : verts tendres, terre chaude, ciel dont la teinte évolue avec la saison.
- **Interface minimale** : le potager au centre, des commandes discrètes, de grandes zones tactiles, peu de texte à l'écran.
- **Souris et tactile d'abord** : tout se fait en glissant, touchant ou cliquant. Aucune action ne demande le clavier, qui ne sert qu'à saisir du texte ou des valeurs.
- **Animations de croissance** : les plantes sortent de terre et grandissent avec un léger rebond. Une seule animation forte, la lecture de la saison. Le reste reste sobre.
- **Illustrations par légume** : une silhouette reconnaissable à chacun des 4 stades. C'est la signature visuelle de l'appli.
- **Accessibilité** : les phases de la frise se distinguent par leur motif autant que par leur couleur, et l'option « réduire les animations » est respectée.

---

## 6. Écrans

| # | Écran | Contenu principal |
|---|---|---|
| E2 | Création du terrain | Dimensions, taille de case, largeur d'allée |
| E3 | Atelier du plan | Vue de dessus, bac de planches à glisser, compteur 10 planches, panneau de la planche sélectionnée, bilan |
| E5 | **Potager** (principal) | Vue jardin, curseur de temps, lecture de la saison, frise continue dépliable, bouton « Aujourd'hui » |
| E6 | Panneau planche | Détail, séries, historique, actions |
| E7 | Création / édition de série | Légume, mode de démarrage, date unique, mini-frise des étapes, zone de la planche |
| E9 | Catalogue | Liste filtrable, fiches des cultures |
| E10 | Réglages | Allées, export et import du potager, vider le plan, effacer les données |
| E11 | Export de la frise | Format, période, planches, options d'affichage, aperçu |

L'écran E1 (Connexion) est supprimé avec le compte.

### 6.1 Parcours principal (premier usage)

1. Création du terrain : dimensions et taille de case (E2).
2. Placement des planches en les glissant depuis le bac (E3).
3. Passage à la vue Potager ; on fait glisser la frise jusqu'à la saison à préparer (E5).
4. Ajout des séries, planche par planche (E6 → E7).
5. Remplissage des créneaux libres à partir des suggestions (E5 → E7).
6. Lecture de la saison pour visualiser le résultat (E5).

---

## 7. Modèle de données (vue fonctionnelle)

```
Potager (un seul par appareil, sans compte)
 └─ Terrain (1)
     ├─ dimensions, taille de case (fixe), largeur d'allée
     └─ Plan (un seul)
         ├─ Planches[] : nom
         │    └─ Dispositions[] : année, type, position, dimensions
         │       (seulement pour les années où la planche change ; 10 planches au maximum par saison)
         ├─ Séries[] : planche, culture, mode de démarrage (abri | plants | pleine terre),
         │             date de semis ou de plantation, zone (premier et dernier tronçon),
         │             durées copiées du catalogue, fin (si modifiée à la main)
         └─ Annotations[] : type (note | période), début, fin (période), planche ou tout le potager, texte, masquée (oui/non)

Catalogue (global, en lecture seule)
 └─ Cultures[] : voir F-CAT-2
```

Valeurs **calculées, non stockées** : date de repiquage, début de récolte, fin (si elle n'est pas modifiée à la main), quantités, créneaux libres, suggestions, calendrier des tâches.

---

## 8. Exigences non fonctionnelles

- **Plateformes** : appli installée sur ordinateur, macOS et Windows (lot 1), puis sur téléphone, iOS et Android (lot 2). Mise en page adaptée dès 360 px de large, préparée dès le lot 1.
- **Performance** : la vue jardin reste fluide (60 images par seconde visées) pendant la lecture de la saison, pour un potager de 20 planches et 60 séries.
- **Réactivité** : les valeurs calculées se mettent à jour instantanément pendant la saisie.
- **Données** : enregistrement automatique, et rien ne doit être perdu si l'on ferme l'appli.
- **Langue** : français uniquement en V1. Tous les textes, noms du catalogue compris, sont prévus pour être traduits : ajouter une langue doit revenir à ajouter un fichier de traductions, sans toucher au reste.
- **Confidentialité** : les données restent sur les appareils de l'utilisateur. Rien n'est envoyé à un serveur. L'export comme l'effacement sont accessibles à tout moment.

---

## 9. Questions ouvertes

Aucune pour l'instant.

---

## 10. Pistes au-delà de la V1

- Ensoleillement : orientation, latitude, obstacles, carte d'ensoleillement, avec peut-être un réglage manuel case par case pour corriger le calcul.
- Décor : composteur, récupérateur d'eau, serre, chemin, banc.
- Climat, dates et périodes de gel.
- Alertes : gel, période de mise en place, rotation, associations, exposition.
- Export du calendrier en iCal (.ics), puis rappels (« semer les tomates sous abri cette semaine »).
- Album des saisons.
- Suivi du réel et recalage automatique du plan.
- Journal de récoltes et bilan de production.
- Météo locale et ajustement des alertes de gel.
- Catalogue élargi (une trentaine de légumes, engrais verts compris), puis personnalisable, variétés.
- Partage du potager en lecture.
- Plusieurs potagers ou plusieurs plans (versions).

---

## 11. Décisions depuis la v1.2

Trois précisions révélées par la conception de D02 (écrans complémentaires) et tranchées sans attendre Tim :

| Sujet | Décision | Pourquoi |
|---|---|---|
| Largeur d'allée à la création du terrain | F-TER-1 demande aussi la largeur d'allée (30, 50 ou 80 cm, 50 cm par défaut), modifiable ensuite dans les réglages (E10) | F-PLN-2 et E10 (§6) rendent l'allée réglable, mais aucune valeur n'existait avant le premier passage dans l'atelier (E3, F-PLN-6) : E2 doit en fixer une dès la création |
| État « Erreur d'enregistrement » | F-CPT-2 ajoute ce troisième état à l'indicateur, avec un bouton « Réessayer » ; la modification reste affichée jusqu'au succès | S05 §7 prévoit déjà qu'une écriture (`exporter_potager`, et l'enregistrement courant) puisse renvoyer une erreur, mais F-CPT-2 ne décrivait que « Enregistré » et « Enregistrement… ». Sans cet état, l'exigence « rien ne doit être perdu » (§8) resterait sans écran pour la montrer |
| Historique du panneau planche | F-VIS-7 ajoute une section Historique (séries des années passées, lecture seule, repliable) et une action « Déplacer » | Le tableau des écrans (§6, E6) promettait déjà « historique » et des « actions » au plan de « détail, séries, actions », mais F-VIS-7 ne détaillait que le rappel des familles sur 2 saisons et trois actions sans déplacement |
