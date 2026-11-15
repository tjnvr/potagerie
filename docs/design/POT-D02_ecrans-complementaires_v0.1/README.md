# POT-D02 · Écrans complémentaires V1 — v0.1

> Version : 0.1 (brouillon) · Date : 2026-11-14 · Auteur : Tim
> Specs : [S01 v1.3](../../specs/POT-S01_spec-fonctionnelle_v1.3.md) · [S02 v0.2](../../specs/POT-S02_catalogue-cultures_v0.2.md) · [S03 v0.7](../../specs/POT-S03_regles-calculs_v0.5.md)
> Fait suite à [D01 v1.1](../POT-D01_ecrans-cles_v1.1/README.md), qui couvre déjà E03, E05 et E07 (téléphone), et la direction visuelle (DV01).

Tim n'étant pas disponible pendant ce lot, les choix ci-dessous ont été tranchés seul ; ils sont listés dans « Décisions » plus bas.

## Contenu

Tous les écrans du lot 1 que D01 ne couvre pas encore, en version ordinateur, plus une planche téléphone pour montrer l'adaptation de mise en page (E10) :

| Fichier | Écran |
|---|---|
| `project/canvas.json` | Index du canvas (positions, titres, notes) |
| `project/Main.dc.html` | E02 · Création du terrain — ordinateur (planche d'entrée du canvas) |
| `project/E06_panneau-planche_desktop.dc.html` | E06 · Panneau planche — ordinateur |
| `project/E07_serie_desktop.dc.html` | E07 · Nouvelle série — ordinateur |
| `project/E09_catalogue_desktop.dc.html` | E09 · Catalogue — ordinateur |
| `project/E10_reglages_desktop.dc.html` | E10 · Réglages — ordinateur (panneau) |
| `project/E10_reglages_mobile.dc.html` | E10 · Réglages — téléphone (plein écran) |
| `project/E11_export-frise_desktop.dc.html` | E11 · Export de la frise — ordinateur |
| `project/ET01_etats_desktop.dc.html` | ET01 · États transverses — ordinateur (4 états sur une planche) |
| `gen.py` | Générateur des planches (`python3 gen.py`) |
| `plants2.py` | Dessins des légumes — **copié depuis D01**, voir « Réutilisation » |
| `exports/` | Un PNG par planche, capturé en Chrome headless (voir « Vérification ») |

Le canvas en ligne est la référence une fois publié ; ce dossier en est la copie versionnée. `gen.py` régénère tout depuis zéro et écrase les retouches faites à la main dans le canvas.

## Réutilisation de D01

`plants2.py` (dessins low-poly des légumes à leurs 4 stades, utilisés par `crop_icon()`) est **copié tel quel** depuis `POT-D01_ecrans-cles_v1.1/plants2.py`, octet pour octet — il n'a pas été réécrit à la main. La palette, les icônes, les polices (Fredoka/Figtree) et les petits composants d'interface (puce, bouton, carte…) suivent la même direction visuelle que D01 (DV01), mais sont redéfinis dans le `gen.py` de D02 : ce sont de petites fonctions propres à chaque générateur, pas une bibliothèque commune à factoriser pour l'instant.

Un seul fichier de D01 a été modifié, a minima, pour fermer la boucle du parcours : le bouton « Modifier » du panneau simplifié de l'écran Potager (`POT-D01.../Main.dc.html`) devient un lien vers le panneau planche complet de D02 (voir « Parcours »).

## Parcours

Les planches sont liées entre elles (bouton Play), y compris vers D01. Le parcours 6.1 complet (S01, §6.1) :

| De | Vers | Déclencheur |
|---|---|---|
| ① E02 Création du terrain (D02) | E03 Atelier du plan (D01) | « Créer le terrain » |
| E03 Atelier du plan (D01) | E05 Potager (D01) | « Voir le potager » (déjà dans D01) |
| E05 Potager (D01) | ④ E06 Panneau planche (D02) | « Modifier », dans le panneau simplifié (lien ajouté, voir « Réutilisation ») |
| ④ E06 Panneau planche (D02) | ⑤ E07 Nouvelle série (D02) | « Planter ici » |
| ④ E06 Panneau planche (D02) | E03 Atelier du plan (D01) | « Déplacer » |
| ⑤ E07 Nouvelle série (D02) | E05 Potager (D01) | « Planter … » ou la croix |

Écrans annexes, atteignables depuis l'icône réglages ou catalogue de l'en-tête (D02) :

| De | Vers | Déclencheur |
|---|---|---|
| N'importe quelle planche D02 | E09 Catalogue | icône livre de l'en-tête |
| N'importe quelle planche D02 | E10 Réglages | icône réglages de l'en-tête |
| E10 Réglages (ordinateur ou téléphone) | E05 Potager / E05 téléphone (D01) | croix ou flèche retour |

ET01 (états transverses) ne fait pas partie du parcours : comme DV01 dans D01, c'est une planche de référence, sans lien sortant.

## Données d'exemple

Le potager reprend celui de D01 (mêmes noms, mêmes 6 légumes de S02), étendu à **8 planches sur 10** pour refléter le compteur « Planches 8 / 10 » déjà affiché dans l'atelier de D01 (voir « Décisions »). Saison 2027, aujourd'hui réel = 14 novembre 2026.

Toutes les dates et quantités affichées ont été **calculées par un script Python** qui réimplémente les règles de S03 (§2 dates, §4 et §6 tronçons/quantités, §9 suggestions), puis vérifiées contre les cas de test de S03 §12 (T1, T3, T4, T5, T7, T8, T9, T11 reproduits à l'identique) avant d'être recopiées dans `gen.py`. Ce script est un script de vérification, hors du dossier versionné (comme les cas de test de S03, qui restent la seule source de vérité) :

- **E06** (panneau de « Racines ») et son avertissement de suppression reproduisent **T11** telle quelle (carotte gardée, poireau supprimé car à cheval sur 2028).
- **E07** (nouvelle série sur « Au bout de l'allée ») reproduit **T13** telle quelle (créneau de 65 j, fin proposée raccourcie au 6 oct. au lieu du 11 oct., enregistrée comme modifiée à la main).
- Le créneau libre de « Fond du jardin » (mentionné dans le contexte de E06/E05) reprend **T7**.
- Les autres séries (tomate/laitue de « Les tomates », courgette, haricot, poireaux des bacs) ont été calculées avec les mêmes formules et vérifiées par le même script, mais ne correspondent pas à un cas de test nommé.

## Vérification visuelle

Chaque planche a été capturée avec Chrome headless, à la taille de son `$preview` (`chrome --headless --disable-gpu --screenshot=... --window-size=W,H fichier.dc.html`), pour vérifier qu'aucun texte ne déborde ni ne se chevauche et que tout reste lisible. Ces captures sont les PNG de `exports/`. Une erreur de superposition a été trouvée ainsi sur E06 (la boîte de confirmation de suppression débordait sur le panneau voisin) et corrigée avant l'export final.

## Décisions

En l'absence de Tim, les choix suivants ont été tranchés et sont à confirmer avec lui :

| Sujet | Décision | Pourquoi |
|---|---|---|
| Largeur d'allée à la création du terrain (E02) | Ajoutée à F-TER-1 de S01 (v1.3) : 30/50/80 cm, 50 cm par défaut, modifiable ensuite | Le brief demandait l'allée sur E02, mais S01 v1.2 ne la plaçait que dans les réglages (E10) sans valeur de départ ; voir S01 §11 |
| État « Erreur d'enregistrement » (ET01) | Ajouté à F-CPT-2 de S01 (v1.3), avec un bouton « Réessayer » | S05 §7 prévoit déjà qu'une écriture puisse échouer, mais aucun état ne le montrait ; voir S01 §11 |
| Historique dans le panneau planche (E06) | Ajouté à F-VIS-7 de S01 (v1.3) : séries des années passées, lecture seule, repliable, plus l'action « Déplacer » | Le tableau des écrans (S01 §6) promettait déjà « historique » et des « actions », plus riches que ce que F-VIS-7 détaillait ; voir S01 §11 |
| 8ᵉ planche de l'exemple (« Au bout de l'allée », butte) | Ajoutée au potager d'exemple de D01 pour atteindre 8 / 10 | Le bac d'éléments de l'atelier (D01, E03) affiche déjà « Planches 8 / 10 », alors que l'exemple n'avait que 7 planches ; D02 corrige cette incohérence en ajoutant la 8ᵉ. Elle sert aussi de planche vierge pour la démonstration de E07 (créneau libre, T13) |
| Mise en avant de la fin raccourcie (E07) | Un badge « Modifiée à la main » à côté de la date de fin, plus un lien « Recalculer » vers la fin normale | S01 F-PLA-2 dit seulement que la valeur reste modifiable ; aucune maquette n'en montrait l'affichage |
| Message de conflit de déplacement (ET01) | « Position refusée. « X » chevaucherait « Y ». La planche revient à sa place. » | F-PLN-6 décrit le comportement (rouge pâle, retour à la place) mais pas le texte affiché à l'utilisateur |
| Panneau desktop vs plein écran mobile (E10) | Même contenu, affiché en panneau sur ordinateur (recouvrant un fond grisé) et en page plein écran sur téléphone (flèche retour au lieu d'une croix) | Démontre F-VIS-6 (mise en page adaptable dès le lot 1) sur un écran concret, comme demandé |
| Catalogue (E09) : légumes « de saison » | Démonstration datée au 20 avril 2027 (laitue et carotte de saison ce jour-là) plutôt qu'à la date réelle | Au 14 novembre 2026 (date réelle), aucun des 6 légumes de S02 n'est dans sa fenêtre de mise en place : la démonstration aurait été vide |
| Aperçu de l'export (E11) | Frise simplifiée (sans phase « semis à l'abri », sous-lignes seulement si plusieurs séries par planche) plutôt qu'une reproduction complète de la frise de E05 | Il s'agit d'un aperçu dans un panneau de réglages, pas de l'écran principal ; la frise complète est déjà celle de D01 (E05) |
| Lien D01 → D02 | Le bouton « Modifier » du panneau simplifié de D01 (`Main.dc.html`) devient un lien vers E06 (D02) ; aucun autre fichier de D01 n'est modifié | Nécessaire pour fermer la boucle du parcours 6.1 demandée (E02 → E05), sans retoucher le contenu déjà validé de D01 |
| Réutilisation de D01 | `plants2.py` copié tel quel ; palette/icônes redéfinies dans le `gen.py` de D02 | Évite de redessiner les légumes à la main (demande explicite) ; la palette est trop petite et trop couplée à chaque écran pour justifier une bibliothèque partagée dès maintenant |
