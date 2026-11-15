# Potagerie — Couverture du lot 1 (bureau)

État des exigences de S01 v1.3 (§4) dans l'appli au lot 1, relevé dans le code le 15 novembre 2026. « Fait » : couvert par le code. « Partiel » : une partie de l'exigence manque. « Reporté » : pas dans le lot 1. Les commandes sont celles de `src-tauri/src/commands.rs`.

| Exigence | État | Où | Remarque |
|---|---|---|---|
| F-CPT-1 Connexion | Fait | sans objet | Supprimée : pas de compte |
| F-CPT-2 Enregistrement automatique | Fait | indicateur de l'en-tête, `Store` SQLite | Trois états dont « Erreur d'enregistrement » et « Réessayer ». La fusion multi-appareils est du lot 3 |
| F-CPT-3 Export et import | Fait | Réglages ; `exporter_potager`, `lire_import`, `importer_potager`, `incoherences_plan` | Avertissement « fichier plus ancien » et liste « à résoudre » |
| F-CPT-4 Effacer les données | Fait | Réglages ; `effacer_donnees` | Double confirmation |
| F-TER-1 Création du terrain | Fait | Plan, « Créer mon terrain » ; `creer_terrain`, `modifier_terrain` | Allée minimale réglable dans les réglages |
| F-PLN-1 Planches | Fait | Plan ; `creer_planche`, `modifier_planche`, `supprimer_planche` | Quatre modèles (planche, carré, butte, bac), limite de 10 planches |
| F-PLN-2 Allées | Partiel | Plan | La largeur est réglable et imposée, mais une position trop proche est refusée (message d'erreur) au lieu de la bulle « Garder ici » / « Revenir » |
| F-PLN-4 Un plan qui évolue par saison | Fait | Plan, bandeau « en consultation » ; `vue_annee`, `vider_plan` | Report sur les années suivantes, saisons passées en lecture seule |
| F-PLN-5 Bilan du plan | Fait | Plan, panneau de droite | Terrain, cultivable, allées, planches |
| F-PLN-6 Placer et dimensionner | Partiel | Plan ; `deplacer_planche` | Glisser, poignées et ajout au toucher présents ; pas d'Annuler / Rétablir par action |
| F-CAT-1 Contenu | Fait | `catalogue` | Les 6 légumes de S02 |
| F-CAT-2 Fiche d'une culture | Fait | Catalogue, « Fiche » | Schéma de S02 |
| F-CAT-3 Consultation | Fait | Catalogue ; `cultures_de_saison` | Filtre par famille et « de saison maintenant » |
| F-PLA-1 Créer une série | Fait | Panneau planche, frise ; `creer_serie`, `apercu_serie` | Dates calculées, aperçu avant validation |
| F-PLA-2 Modifier ou supprimer | Fait | Frise (clic, glisser) ; `modifier_serie`, `decaler_serie`, `supprimer_serie` | |
| F-PLA-3 Successions | Fait | Frise, création de série | Conflit de tronçons signalé avec la période |
| F-PLA-4 Un temps continu | Fait | Potager, frise | Le diorama prend la disposition de l'année du curseur |
| F-PLA-5 Calcul des quantités | Fait | Création de série (« ≈ n pieds, rangs × par rang ») | Calcul dans `crates/plan` |
| F-VIS-1 Vue jardin isométrique | Fait | Potager | Rotation, étiquettes, panneaux « Libre » |
| F-VIS-2 Vue frise | Fait | Potager, sous le diorama | Fusionnée avec le diorama ; défilement continu entre années |
| F-VIS-3 Curseur de temps | Fait | Potager | |
| F-VIS-4 Lecture de la saison | Fait | Potager, ×1 et ×2 | Mesurée à 60 images/s (10 planches, densité normale) |
| F-VIS-5 Annotations de la frise | Fait | Frise, « Annoter » ; `creer_annotation`, `modifier_annotation`, `supprimer_annotation` | Notes et périodes |
| F-VIS-6 Disposition des écrans | Fait | Coquille de l'appli | Vérifiée à 360 et 1440 px |
| F-VIS-7 Panneau planche | Partiel | Potager et Plan | Historique en lecture seule présent ; l'action « Déplacer » du panneau est absente (le déplacement se fait dans l'atelier) |
| F-CRE-1 Créneaux libres | Fait | Frise (pointillés), diorama (« Libre ») | |
| F-CRE-2 Suggestions | Fait | Clic sur un créneau | Jusqu'à 3 cultures, dates préremplies |
| F-EXP-1 Export de la frise | Fait | Écran d'export ; `exporter_frise` | PDF A4 ou A3, PNG, période et planches au choix |
| F-EXP-2 Contenu du PDF et de l'image | Fait | Aperçu de l'export | Phases, repère « aujourd'hui », annotations optionnelles |

| État | Nombre |
|---|---|
| Fait | 26 |
| Partiel | 3 |
| Reporté | 0 |

Les exigences propres au lot 2 (mobile) et au lot 3 (synchronisation multi-appareils) ne figurent pas dans le tableau.
