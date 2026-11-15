// Where the plants of a serie stand on its planche (S03 §6): the rows and the
// plants per row come from the view (Rust), the zone along the length from its
// tronçons. Rows run along the length of the planche, evenly across its width;
// the plants of a row are evenly spread along the zone.

import { enUnites } from "./projection";

export type GeoPlanche = { x: number; y: number; longueur_cm: number; largeur_cm: number };
export type ZoneSerie = {
  troncon_debut: number;
  troncon_fin: number;
  rangs: number;
  pieds_par_rang: number;
};

/** A plant foot, in tiles of the terrain. */
export type Pied = { gx: number; gy: number };

/** Two drawn plants are never closer than this, in cm: closer, they only
 *  pile up into a blob and cost frames (carrots 4 cm apart). */
export const ECART_DESSIN_MIN_CM = 15;

let multiplicateurDeBanc = 1;

/** Dev only (`?densite=4`): draws N times as many plants, to stress the
 *  renderer with the bench of P01. Never set in a build. */
export function reglerDensiteDeBanc(n: number): void {
  multiplicateurDeBanc = Math.max(1, Math.round(Math.sqrt(n)));
}

/** Rows and plants per row actually drawn: those of the view, thinned where
 *  the drawings would be closer than `ECART_DESSIN_MIN_CM`. */
export function quantitesDessinees(
  planche: GeoPlanche,
  serie: ZoneSerie,
  caseCm: number,
): { rangs: number; pieds: number } {
  const longueurCm = (serie.troncon_fin - serie.troncon_debut + 1) * caseCm;
  return {
    rangs:
      multiplicateurDeBanc *
      Math.min(Math.max(1, serie.rangs), Math.max(1, Math.floor(planche.largeur_cm / ECART_DESSIN_MIN_CM))),
    pieds:
      multiplicateurDeBanc *
      Math.min(Math.max(1, serie.pieds_par_rang), Math.max(1, Math.floor(longueurCm / ECART_DESSIN_MIN_CM))),
  };
}

/** The feet of the plants of a serie, row by row, in tiles of the terrain. The
 *  caller sorts them by depth once the view is turned. */
export function positionsPlantes(planche: GeoPlanche, serie: ZoneSerie, caseCm: number): Pied[] {
  const { rangs, pieds } = quantitesDessinees(planche, serie, caseCm);
  const debutCm = planche.x + (serie.troncon_debut - 1) * caseCm;
  const longueurCm = (serie.troncon_fin - serie.troncon_debut + 1) * caseCm;
  const resultat: Pied[] = [];
  for (let r = 0; r < rangs; r++) {
    const gy = enUnites(planche.y + ((r + 0.5) * planche.largeur_cm) / rangs);
    for (let p = 0; p < pieds; p++) {
      resultat.push({ gx: enUnites(debutCm + ((p + 0.5) * longueurCm) / pieds), gy });
    }
  }
  return resultat;
}

/** Gap between two drawn plants along the row and between two rows, in cm. */
export function ecarts(planche: GeoPlanche, serie: ZoneSerie, caseCm: number): { pied: number; rang: number } {
  const longueurCm = (serie.troncon_fin - serie.troncon_debut + 1) * caseCm;
  const { rangs, pieds } = quantitesDessinees(planche, serie, caseCm);
  return { pied: longueurCm / pieds, rang: planche.largeur_cm / rangs };
}

const ECARTS_DE_REFERENCE_CM = 35;
const ECHELLE_MIN = 0.45;
const ECHELLE_MAX = 1.1;

/**
 * Size of the drawing of a plant, from the room it has: the mean of the two
 * gaps against a reference one. Carrots 4 cm apart are drawn small, a
 * courgette alone on a metre at full size.
 */
export function echellePlante(planche: GeoPlanche, serie: ZoneSerie, caseCm: number): number {
  const { pied, rang } = ecarts(planche, serie, caseCm);
  const k = Math.sqrt(pied * rang) / ECARTS_DE_REFERENCE_CM;
  return Math.min(ECHELLE_MAX, Math.max(ECHELLE_MIN, k));
}
