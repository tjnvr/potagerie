# POT-P01 · Prototype rendu isométrique — v0.1

> Version : 0.1 (brouillon) · Date : 2026-11-14 · Auteur : Tim
> Page en ligne : https://claude.ai/artifact/PBrzt3qta6sHKisnhfZWMv

## Question

Le diorama (10 planches, 6 légumes de la S02) reste-t-il fluide pendant « Jouer la saison », sur ordinateur et sur téléphone ? Quel moteur retenir : **SVG** ou **Canvas** ?

## Contenu

| Fichier | Rôle |
|---|---|
| `index.html` | Page autonome : diorama, lecture de la saison en 15 s, choix du moteur et de la densité, mesure des images/s |

Ouvrir `index.html` dans un navigateur, ou la page en ligne.

## Protocole

Pour chaque appareil (ordinateur, téléphone) : jouer une saison complète pour chaque combinaison moteur × densité (Normale, × 2, × 4), puis « Copier les résultats ».

Critère : **moyenne ≥ 55 images/s** et **1 % des images les plus lentes ≥ 30 images/s** à densité Normale.

## Résultats

| Appareil | Rendu | Densité | Plantes | Moyenne | 1 % lentes | p95 | Verdict |
|---|---|---|---|---|---|---|---|
| Mac (écran 1512×982, 120 Hz) | SVG | Normale | 132 | 120 img/s | 106 img/s | 9,3 ms | Fluide |
| Mac (écran 1512×982, 120 Hz) | Canvas | Normale | 132 | 120 img/s | 106 img/s | 9,2 ms | Fluide |
| Téléphone | SVG | Normale | | | | | à mesurer |
| Téléphone | SVG | × 4 | | | | | à mesurer |

Sur le Mac, les deux moteurs plafonnent à la fréquence de l'écran : à 132 plantes, ils sont indiscernables.

## Décision

**SVG retenu, à titre provisoire.** À performance égale sur ordinateur, il simplifie l'atelier (toucher, glisser une planche), le zoom et l'export vectoriel.

À confirmer sur téléphone (SVG Normale et × 4). Repli si le SVG saccade : Canvas pour les plantes animées, couche SVG transparente par-dessus pour les planches qu'on touche.

Avec Go + htmx (S05), le serveur rend le SVG du diorama ; l'animation de la saison reste dans le navigateur. Aucune image en pixels n'est générée côté serveur.
