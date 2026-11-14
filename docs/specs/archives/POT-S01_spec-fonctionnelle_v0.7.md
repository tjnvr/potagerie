# Potagerie — Spécification fonctionnelle V1

> Version : 0.7 (brouillon) · Date : 14 novembre 2026 · Auteur : Tim · Domaine : potagerie.garden

---

## 1. Vision

Potagerie est une application web pour **concevoir, planifier et visualiser un potager**.
On dessine son terrain, on y place ses planches, puis on organise ses cultures dans le temps. Le potager s'affiche comme un **petit diorama isométrique** qui pousse au fil de la saison.

Potagerie est un **outil de planification**, pas un jeu. Son style visuel s'inspire de l'application Forest : rendu doux, apaisant, satisfaisant à regarder.

### 1.1 Nom et domaine

- **Potagerie** : un nom à consonance française, facile à prononcer à l'international, plus distinctif que « Potager » seul, déjà utilisé par plusieurs applis et marques.
- Domaine visé : **potagerie.garden**. Sa disponibilité reste à vérifier avant l'achat.

### 1.2 Objectifs

- Concevoir le plan d'un **futur potager** à partir d'un terrain vierge.
- Planifier une saison complète de cultures, successions comprises.
- **Visualiser** le potager à n'importe quelle date et voir la saison défiler.
- Être aidé pour remplir les planches vides (créneaux libres et suggestions).
- Retrouver son potager sur n'importe quel appareil.

### 1.3 Utilisateur cible

Un jardinier amateur, seul, qui prépare un potager familial (quelques planches, de 10 à 100 m²). Il planifie surtout l'hiver sur ordinateur et consulte au printemps et en été sur téléphone.

---

## 2. Périmètre

### 2.1 Dans la V1

| Domaine | Contenu |
|---|---|
| Compte | Inscription et connexion, synchronisation entre appareils |
| Terrain | Dimensions, orientation, obstacles, carte d'ensoleillement |
| Plan | Un seul plan par utilisateur : planches et carrés, obstacles, décor, allées |
| Catalogue | ~30 légumes courants préremplis, non modifiables en V1 |
| Planification | Séries de cultures, successions, calcul automatique des dates et des quantités |
| Visualisation | Vue jardin isométrique datée, frise (Gantt), curseur de temps, lecture de la saison |
| Aide | Créneaux libres et suggestions de cultures |
| Saisons | Une saison planifiée à la fois, historique conservé pour la rotation |
| Export | Frise des cultures en PDF, PNG et CSV |

### 2.2 Hors V1

- Suivi du réel : dates effectives, « semé », « récolté », recalage automatique.
- Journal de récoltes, quantités, photos, notes.
- Météo en direct, notifications et rappels.
- Partage ou collaboration, potager public.
- Plusieurs potagers ou plusieurs plans (versions) par compte.
- Catalogue personnalisable : variétés, durées modifiées, légumes ajoutés.
- Mode hors ligne, prévu en V1.1.
- Toute mécanique de jeu : points, niveaux, quêtes, déblocages.
- **Alertes** : gel, période de mise en place, rotation, associations, exposition. Les données du catalogue nécessaires sont tout de même saisies dès la V1, pour les activer plus tard.

### 2.3 À confirmer

- **Album des saisons** : une image figée du potager à son apogée, gardée chaque année. Il n'est pas encore décidé si on l'inclut en V1.

---

## 3. Concepts et vocabulaire

| Terme | Définition |
|---|---|
| **Terrain** | La surface totale disponible, rectangulaire en V1, découpée en grille. |
| **Case** | Unité de la grille, de 25, 50 ou 100 cm de côté, réglée sur le terrain. |
| **Plan** | La disposition des planches, des obstacles et du décor sur le terrain. **Un seul plan par utilisateur.** |
| **Planche** | Une zone cultivable rectangulaire : planche, carré, butte ou bac. |
| **Obstacle** | Un élément qui projette de l'ombre : maison, haie, arbre, abri, mur. |
| **Décor** | Un élément sans effet sur les règles : composteur, récupérateur d'eau, chemin, banc. |
| **Culture** | Une fiche du catalogue, par exemple « Tomate ». |
| **Série** | Une culture placée sur une zone d'une planche, sur une période. C'est l'objet central de la planification. |
| **Tronçon** | Une portion de la longueur d'une planche, de la taille d'une case (50 cm par défaut). Une zone est une suite de tronçons contigus. |
| **Saison** | Une année civile de planification. |
| **Créneau libre** | Une période pendant laquelle une planche n'a aucune série. |

---

## 4. Fonctionnalités

### 4.1 Compte et synchronisation

**F-CPT-1 — Connexion**
- Connexion par lien magique envoyé par e-mail, ou par compte Google.
- Pas de mot de passe à gérer.

**F-CPT-2 — Synchronisation**
- Chaque modification est enregistrée automatiquement en ligne, sans bouton « Sauvegarder ».
- En cas de modification sur deux appareils, la **dernière modification enregistrée l'emporte**.
- Un indicateur discret affiche l'état : « Enregistré », « Enregistrement… » ou « Hors connexion, modifications en attente ».

**F-CPT-3 — Export**
- Export complet du potager en fichier JSON, à la fois comme sauvegarde et pour la portabilité des données.
- L'import d'un export remplace le potager existant, après confirmation.

**F-CPT-4 — Suppression de compte**
- Supprime le compte et toutes ses données, après double confirmation.

---

### 4.2 Terrain

**F-TER-1 — Création du terrain**
- Saisie de la longueur et de la largeur en mètres.
- Choix de la taille de case : 25, 50 ou 100 cm (50 cm par défaut).
- Saisie de l'**orientation** : une boussole à tourner pour indiquer où est le nord.

**F-TER-2 — Obstacles**
- On les ajoute en les glissant depuis le bac d'éléments de l'atelier (voir F-PLN-6), puis on les déplace, redimensionne ou supprime comme une planche.
- Chaque obstacle a une **hauteur** (par exemple maison 6 m, haie 2 m, abri 2,5 m) qui sert au calcul de l'ombre.
- Obstacles proposés : maison, mur, haie, arbre, abri ou cabane.

**F-TER-3 — Carte d'ensoleillement**
- Calculée à partir de l'orientation, des obstacles et de la latitude, que l'utilisateur peut régler (Bordeaux par défaut).
- Trois niveaux par case : **plein soleil** (6 h et plus de soleil direct par jour en été), **mi-ombre** (de 3 à 6 h), **ombre** (moins de 3 h).
- Toujours affichée en surimpression dans l'atelier du plan (E3), avec sa légende. Elle n'apparaît pas dans la vue jardin.
- Le calcul est simplifié : la journée du 21 juin et l'équinoxe servent de références.

**F-TER-4 — Climat**
- Date moyenne des **dernières gelées** (15 avril par défaut) et des **premières gelées** (10 novembre par défaut).
- Les deux dates sont modifiables. Un texte précise que ce sont des moyennes, à ajuster selon son coin.

---

### 4.3 Plan

**F-PLN-1 — Planches**
- **10 planches au maximum**. Un compteur « Planches 8 / 10 » est visible dans l'atelier, et les planches du bac d'éléments sont grisées à 10, avec un message qui l'explique.
- Types de planche : **planche** en pleine terre (1,2 × 3 m par défaut), **carré** (1 × 1 m), **butte** (1,2 m de large), **bac** surélevé (1,2 × 2 m). Le type ne change que l'apparence et la taille de départ : toutes se planifient de la même façon.
- Chaque planche a un **nom libre** (30 caractères au plus), par exemple « Les tomates » ou « Côté cabane ». À la création, elle reçoit un nom par défaut (« Planche A », « Carré B »…) que l'on peut changer à tout moment, directement dans le panneau planche.
- Le nom s'affiche partout où la planche apparaît : étiquette dans la vue jardin, ligne de la frise, panneau planche, création de série et exports. Deux planches peuvent porter le même nom, mais l'appli le signale.
- Chaque planche est déplaçable et redimensionnable (voir F-PLN-6).
- Deux planches ne peuvent pas se chevaucher. Une planche ne peut pas sortir du terrain ni empiéter sur un obstacle.
- La suppression d'une planche supprime aussi ses séries, après confirmation indiquant combien de séries sont concernées.

**F-PLN-6 — Placer et dimensionner les éléments**

Tout se fait dans l'**atelier du plan** (E3), en vue de dessus sur la grille. L'appli a **deux modes**, toujours accessibles par les onglets « Potager » et « Plan » en haut de l'écran : « Plan » ouvre l'atelier, « Potager » ramène à la vue jardin (le bouton « Voir le potager » de l'atelier fait de même).

L'atelier n'a **pas d'outils à choisir** : on manipule directement les éléments.

- **Ajouter** : un **bac d'éléments** sous le plan propose Planche, Carré, Bac, Arbre, Haie, Abri, Composteur et Récupérateur d'eau. On attrape un élément et on le lâche sur le terrain : il arrive à sa taille par défaut et reste sélectionné. Une planche reçoit un nom par défaut. Le compteur « Planches 8 / 10 » est affiché sur le bac, et les éléments de type planche sont grisés à 10.
- **Sélectionner** : toucher ou cliquer un élément. Toucher l'herbe désélectionne.
- **Déplacer** : glisser l'élément à la souris, ou au doigt après un appui long sur téléphone.
- **Redimensionner** : avec les 8 poignées de l'élément sélectionné, ou en saisissant longueur et largeur dans le panneau.
- **Tourner** d'un quart de tour, **dupliquer**, **supprimer** : depuis la barre qui apparaît sous l'élément sélectionné, ou depuis le panneau.
- **Panneau de droite** : il montre toujours l'élément sélectionné.
  - Pour une planche : nom, type, longueur, largeur, exposition.
  - Pour un obstacle : type et hauteur.
  - Sans sélection : le bilan du plan (F-PLN-5).
- **Aides au placement** :
  - magnétisme sur la grille et sur les bords des éléments voisins, avec un guide en pointillés ;
  - cotes en direct jusqu'aux voisins et aux bords du terrain ;
  - position d'origine en pointillés pendant le glisser.
- **Positions refusées** (chevauchement, sortie du terrain) : l'élément passe en rouge pâle et revient à sa place si on le lâche.
- Les séries et annotations **restent attachées** à la planche quand on la déplace. Si on la raccourcit, les séries qui ne tiennent plus sont signalées avant de valider.
- **Annuler / Rétablir** pour chaque action, avec les boutons de l'en-tête.

**F-PLN-2 — Allées**
- Largeur d'allée minimale réglable : 30, 50 ou 80 cm.
- Lors du placement, l'appli garde cette largeur entre deux planches. Si on lâche la planche plus près, une bulle propose « Garder ici » ou « Revenir ».

**F-PLN-3 — Décor**
- Placement libre d'éléments de décor (composteur, récupérateur d'eau, chemin, banc, serre).
- Le décor n'a aucun effet sur les règles en V1.

**F-PLN-4 — Un seul plan**
- Chaque compte a **un terrain et un plan**. Il n'y a ni versions ni plan actif à choisir.
- Pour repartir de zéro : « Vider le plan », dans les réglages, après confirmation. Les séries des planches supprimées sont supprimées aussi.

**F-PLN-5 — Bilan du plan**
- Surface totale, surface cultivable, surface d'allées.
- Répartition de la surface cultivable par ensoleillement.
- Nombre de planches.

---

### 4.4 Catalogue des cultures

**F-CAT-1 — Contenu**
- Environ 30 légumes courants. Liste indicative : tomate, poivron, aubergine, pomme de terre, courgette, courge, concombre, melon, haricot, petit pois, fève, carotte, panais, betterave, radis, navet, poireau, oignon, ail, échalote, laitue, mâche, épinard, blette, chou, brocoli, basilic, persil, fraisier, engrais vert (phacélie).

**F-CAT-2 — Fiche d'une culture**

| Champ | Exemple (tomate) | Usage |
|---|---|---|
| Nom | Tomate | Affichage |
| Famille botanique | Solanacées | Suggestions, rotation après la V1 |
| Espacement entre rangs | 80 cm | Calcul des quantités |
| Espacement sur le rang | 60 cm | Calcul des quantités |
| Durée de semis sous abri | 8 semaines (0 si semis direct) | Calcul de la date de semis |
| Durée jusqu'à la récolte | 75 jours après mise en place | Phases |
| Durée de récolte | 70 jours | Phases |
| Fenêtre(s) de mise en place | 1er mai – 15 juin | Période idéale |
| Sensible au gel | Oui | Suggestions, alertes de gel après la V1 |
| Exposition | Plein soleil | Filtre du catalogue (alertes après la V1) |
| Bons voisins | Basilic, carotte, laitue, persil | Fiche, alertes après la V1 |
| Mauvais voisins | Chou, brocoli, pomme de terre | Fiche, alertes après la V1 |
| Illustrations | 4 stades | Vue jardin |

- Une culture peut avoir **plusieurs fenêtres** de mise en place (par exemple épinard au printemps et à la fin de l'été).
- Une fenêtre peut chevaucher deux années (par exemple ail : 15 octobre – 15 mars).

**F-CAT-3 — Consultation**
- Liste filtrable par famille, par saison de mise en place (« de saison maintenant ») et par exposition.
- Fiche détaillée en lecture seule.

---

### 4.5 Planification

**F-PLA-1 — Créer une série**
- Depuis une planche (vue jardin ou frise) ou depuis un créneau libre suggéré.
- Saisie :
  - **Culture** : choisie dans le catalogue, avec les cultures « de saison » à la date choisie mises en avant.
  - **Planche**.
  - **Date de mise en place** : plantation, ou semis en pleine terre.
  - **Zone de la planche** : la planche est découpée en **tronçons** de 50 cm sur sa longueur. On touche les tronçons occupés par la série, qui doivent être contigus. Des raccourcis sélectionnent toute la planche, ½, ⅓ ou ¼, en partant du premier tronçon libre. La zone choisie s'affiche aussi sur un petit schéma de la planche.
- Valeurs calculées et affichées immédiatement :
  - Date de **semis sous abri** si la culture en a un (mise en place moins la durée sous abri).
  - **Début de récolte** (mise en place plus la durée jusqu'à la récolte).
  - **Fin de série** (début de récolte plus la durée de récolte), modifiable.
  - **Quantité estimée** : nombre de rangs × pieds par rang, d'après la longueur de la zone, la largeur de la planche et les espacements.

**F-PLA-2 — Modifier ou supprimer une série**
- Toutes les valeurs saisies restent modifiables.
- Si on modifie la date de mise en place, la fin se recalcule, sauf si elle avait été modifiée à la main.
- Sur la frise, on peut **glisser une barre** pour décaler la série. La durée est conservée.
- La suppression demande une confirmation.

**F-PLA-3 — Successions**
- Plusieurs séries peuvent se suivre sur une même planche.
- Deux séries peuvent coexister sur une planche si leurs **zones ne se chevauchent pas** pendant la même période. Sinon, la validation est impossible et l'appli montre les tronçons et la période en conflit.

**F-PLA-4 — Saisons**
- Sélecteur de saison : année précédente, année en cours, année suivante.
- Les séries à cheval sur deux années (par exemple ail, poireau d'hiver) apparaissent dans les deux saisons.
- Les saisons passées restent consultables et modifiables. Elles alimentent la règle de rotation.

**F-PLA-5 — Calcul des quantités**
- Longueur de référence = la longueur de la zone (nombre de tronçons × taille d'un tronçon). Largeur = le plus petit côté de la planche.
- Rangs = largeur ÷ espacement entre rangs, arrondi à l'entier inférieur, au minimum 1.
- Pieds par rang = longueur ÷ espacement sur le rang, arrondi à l'entier inférieur, au minimum 1.
- Le résultat est affiché comme une **estimation** (« ≈ 21 pieds, 1 rang × 21 »).

---

### 4.6 Visualisation

L'écran principal présente deux vues **synchronisées sur une même date**, le *curseur de temps*.

**F-VIS-1 — Vue jardin (isométrique)**
- Le potager, rendu comme un diorama isométrique posé sur un fond uni.
- Le terrain, les planches, les obstacles et le décor sont dessinés en volume simple.
- Chaque série en terre à la date du curseur est représentée par des plantes **sur sa zone** de la planche.
- **4 stades visuels** selon l'avancement :

  | Stade | Avancement depuis la mise en place |
  |---|---|
  | Semis | 0 – 15 % de la durée jusqu'à la récolte |
  | Jeune pousse | 15 – 50 % |
  | Adulte | 50 – 100 % |
  | Récolte | Au-delà de 100 % (fruits ou légumes visibles) |

- Une série en phase de semis sous abri est signalée par une **petite étiquette** plantée sur la planche.
- **Ambiance saisonnière** : la couleur de l'herbe et du ciel suit la date (givre en hiver, fleurs au printemps, tons dorés à l'automne).
- Navigation :
  - zoom par pincement (téléphone), molette ou pavé tactile (ordinateur), sans bouton ;
  - déplacement en faisant glisser le fond ;
  - bouton « Tourner la vue » pour faire pivoter le potager d'un quart de tour, et voir ce que cachent les grandes plantes.
- L'onglet « Plan », en haut de l'écran, ouvre l'atelier du plan (E3).
- Un toucher ou un clic sur une planche ouvre son panneau de détail.

**F-VIS-2 — Vue frise (Gantt)**
- Une ligne par planche, triée selon sa position sur le plan.
- Une barre par série, découpée en **3 phases** visuellement distinctes : semis sous abri (hachuré), croissance (couleur atténuée), récolte (couleur pleine).
- Les séries qui se chevauchent sur une même planche sont empilées sur plusieurs sous-lignes.
- En fond :
  - colonnes des mois ;
  - **périodes de gel** hachurées (avant les dernières gelées, après les premières) ;
  - repère « aujourd'hui ».
- Un toucher ou un clic sur une barre ouvre l'édition de la série. Un glisser décale la série (F-PLA-2).

**F-VIS-3 — Curseur de temps**
- Un repère vertical sur la frise et un sélecteur de date sous la vue jardin, les deux liés.
- On peut le faire glisser, toucher ou cliquer sur la frise, ou avancer d'une semaine avec les boutons ‹ et › placés de part et d'autre de la date.
- À l'ouverture, le curseur est sur **aujourd'hui** si la saison affichée est l'année en cours, sinon sur le 15 juin.

**F-VIS-4 — Lecture de la saison**
- Le bouton « ▶ Jouer la saison » fait avancer le curseur du 1er janvier au 31 décembre en une quinzaine de secondes.
- Les plantes poussent et changent de stade avec une **animation douce**, à la façon des arbres qui poussent dans Forest.
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
- Les annotations font partie de l'export (option « Annotations » cochée par défaut), et ont leur propre type de ligne dans le CSV.

**F-VIS-6 — Disposition des écrans**
- **Navigation principale** : deux onglets en haut de l'écran, « Potager » (E5) et « Plan » (E3).
- **Ordinateur** : la vue jardin occupe le haut de l'écran, la frise est en dessous, dépliable. Un panneau latéral affiche le détail (planche, série).
- **Téléphone** : la vue jardin est en plein écran avec le curseur de temps en bas. La frise et le panneau de détail s'ouvrent en feuilles coulissantes.

**F-VIS-7 — Panneau planche**
- Nom (modifiable sur place), dimensions, surface, ensoleillement majoritaire.
- Notes de la planche, triées par date.
- Séries de la saison, avec leurs dates et quantités.
- Familles cultivées les 2 saisons précédentes, simple rappel pour la rotation.
- Actions : « Planter ici », modifier, supprimer.

---

### 4.7 Créneaux libres et suggestions

Pas d'alertes en V1 (voir 2.2). L'appli aide seulement à remplir les planches vides.

**F-CRE-1 — Détection des créneaux libres**
- Pour chaque planche, l'appli repère les périodes **sans série de plus de 35 jours** entre le 1er mars et le 15 novembre.
- Dans la frise, un créneau libre apparaît en pointillés. Dans la vue jardin, un panneau « + Libre » est planté sur la planche.

**F-CRE-2 — Suggestions**
- Pour chaque créneau, l'appli propose **jusqu'à 3 cultures** :
  - mise en place possible dans les 4 premières semaines du créneau, dans une fenêtre idéale de la culture ;
  - durée jusqu'à la récolte, plus un minimum de récolte, compatible avec la longueur du créneau ;
  - cultures sensibles au gel écartées si elles devraient être mises en place avant les dernières gelées ou rester en terre après les premières.
- Classement : bon remplissage du créneau d'abord, puis une famille différente de la culture précédente sur la planche. Les engrais verts passent en dernier recours.
- Un toucher sur le créneau ouvre la création de série, avec les suggestions en tête de liste et les dates préremplies.

---

### 4.8 Export du plan de cultures

**F-EXP-1 — Export de la frise (Gantt)**
- Accessible depuis le bouton « Exporter la frise », dans l'en-tête de la frise.
- Formats :
  - **PDF**, pour imprimer et afficher dans l'abri de jardin : A4 ou A3, paysage, une ou plusieurs pages selon le nombre de planches.
  - **Image PNG**, pour partager ou garder une trace.
  - **CSV**, une ligne par série, pour les tableurs.
- Options :
  - **Période** : la saison entière (par défaut) ou une plage de mois.
  - **Planches** : toutes, ou une sélection.
  - **Contenu** : afficher ou non les phases (abri, croissance, récolte), les périodes de gel, le repère « aujourd'hui » et les annotations.
- Le rendu exporté reprend le style visuel de l'appli, adapté à l'impression : fond clair, couleurs lisibles en noir et blanc grâce aux motifs de phase.

**F-EXP-2 — Contenu du PDF et de l'image**
- En-tête : nom du potager, saison, date d'export.
- La frise : une ligne par planche, les mois en colonnes, une barre par série avec le nom de la culture.
- Légende : phases, périodes de gel et annotations.
- **Miniature du plan** : la vue de dessus du potager, avec le nom des planches, pour retrouver chaque ligne sur le terrain.
- Option **« Calendrier des tâches »** : la liste mois par mois de ce qu'il faut semer sous abri, mettre en place et commencer à récolter, déduite des séries.

**F-EXP-3 — Colonnes du CSV**
- Saison, planche, culture, famille, zone (tronçons et longueur), semis sous abri, mise en place, début de récolte, fin, quantité estimée. Les annotations sont exportées sur des lignes de type « note » ou « période » (date de début, date de fin, planche ou « tout le potager », texte).
- Les dates sont au format AAAA-MM-JJ et le séparateur est le point-virgule, pour Excel en français.

---

## 5. Direction visuelle

Inspiration : **Forest** pour le rendu, sans la mécanique de jeu.

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
| E1 | Connexion | E-mail (lien magique), Google |
| E2 | Création du terrain | Dimensions, taille de case, orientation, localisation et dates de gel |
| E3 | Atelier du plan | Vue de dessus, bac d'éléments à glisser, compteur 10 planches, carte d'ensoleillement, panneau de l'élément sélectionné, bilan |
| E5 | **Potager** (principal) | Vue jardin, curseur de temps, lecture de la saison, frise dépliable |
| E6 | Panneau planche | Détail, séries, historique, actions |
| E7 | Création / édition de série | Choix de la culture, dates, zone de la planche, valeurs calculées |
| E9 | Catalogue | Liste filtrable, fiches des cultures |
| E10 | Réglages | Climat, allées, export/import, vider le plan, compte |
| E11 | Export de la frise | Format, période, planches, options d'affichage, aperçu |

### 6.1 Parcours principal (premier usage)

1. Connexion (E1).
2. Création du terrain : dimensions, orientation, obstacles (E2 → E3).
3. Placement des planches en les glissant depuis le bac, à l'aide de la carte d'ensoleillement (E3).
4. Passage à la vue Potager, choix de la saison suivante (E5).
5. Ajout des séries, planche par planche (E6 → E7).
6. Remplissage des créneaux libres à partir des suggestions (E5 → E7).
7. Lecture de la saison pour visualiser le résultat (E5).

---

## 7. Modèle de données (vue fonctionnelle)

```
Compte
 └─ Terrain (1 en V1)
     ├─ dimensions, taille de case, orientation, latitude
     ├─ climat : dernières gelées, premières gelées
     ├─ Obstacles[] : type, position, taille, hauteur
     └─ Plan (un seul)
         ├─ Planches[] (10 au maximum) : nom, type, position, dimensions
         ├─ Décor[] : type, position
         ├─ Séries[] : planche, culture, mise en place, fin, zone (premier et dernier tronçon), fin modifiée à la main (oui/non)
         └─ Annotations[] : type (note | période), début, fin (période), planche ou tout le potager, texte, masquée (oui/non)

Catalogue (global, en lecture seule)
 └─ Cultures[] : voir F-CAT-2
```

Valeurs **calculées, non stockées** : date de semis sous abri, début de récolte, quantités, ensoleillement par case, créneaux libres, suggestions.

---

## 8. Exigences non fonctionnelles

- **Plateformes** : navigateurs récents sur ordinateur et téléphone. Mise en page adaptée dès 360 px de large.
- **Performance** : la vue jardin reste fluide (60 images par seconde visées) pendant la lecture de la saison, pour un potager de 20 planches et 60 séries.
- **Réactivité** : les valeurs calculées se mettent à jour instantanément pendant la saisie.
- **Données** : sauvegarde automatique, et rien ne doit être perdu en cas de fermeture de l'onglet.
- **Langue** : français uniquement en V1, en prévoyant les textes pour une traduction ultérieure.
- **Confidentialité** : seules l'adresse e-mail et les données du potager sont stockées, et l'export comme la suppression sont accessibles à tout moment.

---

## 9. Questions ouvertes

1. **Album des saisons** : inclus en V1 ou reporté ?
2. **Ensoleillement** : le calcul simplifié (21 juin et équinoxe) suffit-il, ou faut-il un réglage manuel case par case pour corriger ?
3. **Catalogue** : la liste des ~30 légumes est à valider, avec les données de chaque fiche à sourcer et relire.
4. **Hors ligne** : le report en V1.1 est-il confirmé ?
5. **Glisser des barres sur téléphone** : à garder ou à réserver à l'ordinateur ?
6. **Export calendrier** : faut-il aussi un export iCal (.ics), pour voir les dates de semis et de mise en place dans un agenda ? Ce serait un premier pas vers les rappels.
7. **Nom et domaine** : vérifier que potagerie.garden est libre et que le nom Potagerie n'est pas déjà déposé (INPI, EUIPO).

---

## 10. Pistes au-delà de la V1

- Suivi du réel et recalage automatique du plan.
- Journal de récoltes et bilan de production.
- Rappels (« semer les tomates sous abri cette semaine »).
- Alertes : gel, période de mise en place, rotation, associations, exposition.
- Météo locale et ajustement des alertes de gel.
- Catalogue personnalisable, variétés.
- Partage du potager en lecture.
- Plusieurs potagers ou plusieurs plans (versions) par compte.
