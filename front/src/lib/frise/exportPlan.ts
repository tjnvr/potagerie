// What the export screen (E11) decides before drawing: the default period,
// whether the chosen one can be exported, and the scale of the drawing.
// The drawing itself comes from `FriseSvg`, the business from Rust.
import { anneeDe, dateDe, jourDe } from "../dates";
import { JOURS_MAX, type Echelle } from "./geometrie";

/** Pixels of the exported drawing for a year: wide enough to read once printed on A3. */
const LARGEUR_CIBLE_PX = 1800;
const PX_PAR_JOUR_MIN = 1.5;
const PX_PAR_JOUR_MAX = 8;

/** The civil year of the cursor, the default of the period (S01 F-EXP-1). */
export function periodeParDefaut(curseur: string): { du: string; au: string } {
  const annee = anneeDe(curseur);
  return { du: `${annee}-01-01`, au: `${annee}-12-31` };
}

/** The period the timeline shows on screen: every day at least partly
 *  visible, at most the 3 years an export allows (S01 F-EXP-1). */
export function periodeVisible(fenetre: { debutJour: number; jours: number }): { du: string; au: string } {
  const debut = Math.floor(fenetre.debutJour);
  const fin = Math.min(Math.ceil(fenetre.debutJour + fenetre.jours) - 1, debut + JOURS_MAX - 1);
  return { du: dateDe(debut), au: dateDe(Math.max(fin, debut)) };
}

export type ProblemePeriode = "vide" | "inversee" | "trop_longue";

/** Why a period cannot be exported, or `null`. At most 3 years, as the timeline. */
export function problemePeriode(du: string, au: string): ProblemePeriode | null {
  if (!du || !au) return "vide";
  if (au < du) return "inversee";
  if (jourDe(au) - jourDe(du) + 1 > JOURS_MAX) return "trop_longue";
  return null;
}

/** The scale that draws `du` to `au` (included) in a fixed width. */
export function echellePourPeriode(du: string, au: string): Echelle {
  const jours = jourDe(au) - jourDe(du) + 1;
  const pxParJour = Math.min(PX_PAR_JOUR_MAX, Math.max(PX_PAR_JOUR_MIN, LARGEUR_CIBLE_PX / jours));
  return { debutJour: jourDe(du), pxParJour, largeur: jours * pxParJour };
}

/** The years a period touches, to load their views. */
export function anneesDePeriode(du: string, au: string): number[] {
  const debut = anneeDe(du);
  return Array.from({ length: anneeDe(au) - debut + 1 }, (_, i) => debut + i);
}
