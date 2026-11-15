// What a serie looks like at a date, and how the season plays.
//
// The stages and their dates come from the Rust view (`appearance_changes`,
// S03 §3). Nothing here knows a threshold: the only calculation is the
// interpolation of the animation (scale and bounce after a change) and the
// advance of the playback.

import type { Appearance, AppearanceChange } from "../bindings";
import { jourDepuis } from "./jours";

/** The changes of one serie with their dates as day numbers. */
export type Chronologie = { jours: number[]; apparences: Appearance[] };

export function chronologie(changes: readonly AppearanceChange[]): Chronologie {
  return {
    jours: changes.map((c) => jourDepuis(c.from)),
    apparences: changes.map((c) => c.appearance),
  };
}

/** Index of the last change that has happened on day `jour` (a fraction is
 *  dropped), -1 before the first. */
export function indexAu(chrono: Chronologie, jour: number): number {
  const j = Math.floor(jour);
  let bas = 0;
  let haut = chrono.jours.length;
  while (bas < haut) {
    const milieu = (bas + haut) >> 1;
    if ((chrono.jours[milieu] ?? 0) <= j) bas = milieu + 1;
    else haut = milieu;
  }
  return bas - 1;
}

export function apparenceAu(chrono: Chronologie, jour: number): Appearance {
  const i = indexAu(chrono, jour);
  return chrono.apparences[i] ?? { appearance: "nothing" };
}

/** The symbol and size of a serie on a day. `symbole` is null when nothing is drawn. */
export type Etat = { symbole: string | null; echelle: number };

export const SYMBOLE_ETIQUETTE = "etiquette";
const RIEN: Etat = { symbole: null, echelle: 1 };

/** Id of the symbol of a crop at a stage: `plante-tomate-adult`. */
export function symbolePlante(culture: string, stade: string): string {
  return `plante-${culture.replaceAll("_", "-")}-${stade}`;
}

/** Days a plant takes to settle at its new size after a change. */
export const DUREE_CROISSANCE_JOURS = 10;

/** Ease out with a slight overshoot: the little bounce of the growth. */
export function rebond(t: number): number {
  if (t <= 0) return 0;
  if (t >= 1) return 1;
  const c1 = 1.70158;
  const c3 = c1 + 1;
  return 1 + c3 * Math.pow(t - 1, 3) + c1 * Math.pow(t - 1, 2);
}

/**
 * What to draw for `culture` on day `jour`. With `animer`, a plant that has
 * just come out of the ground or changed stage grows to its size with a bounce;
 * without, it is at its size at once (scrubbing, reduced motion).
 */
export function etatAu(chrono: Chronologie, culture: string, jour: number, animer: boolean): Etat {
  const i = indexAu(chrono, jour);
  if (i < 0) return RIEN;
  const apparence = chrono.apparences[i];
  if (!apparence || apparence.appearance === "nothing") return RIEN;
  if (apparence.appearance === "label") return { symbole: SYMBOLE_ETIQUETTE, echelle: 1 };
  const symbole = symbolePlante(culture, apparence.stage);
  if (!animer) return { symbole, echelle: 1 };
  const t = (jour - (chrono.jours[i] ?? 0)) / DUREE_CROISSANCE_JOURS;
  if (t >= 1) return { symbole, echelle: 1 };
  const avant = chrono.apparences[i - 1]?.appearance ?? "nothing";
  // Out of the ground (or off the label) from small; from a stage to the next, a smaller step.
  const depart = avant === "plant" ? 0.75 : 0.2;
  return { symbole, echelle: depart + (1 - depart) * rebond(t) };
}

/** « Jouer la saison »: about 15 s for a year (S01 F-VIS-4). */
export const DUREE_SAISON_S = 15;
export const JOURS_PAR_SECONDE = 365 / DUREE_SAISON_S;
/** With reduced motion the season advances by steps of a week, no transition. */
export const PALIER_JOURS = 7;

export type EntreeLecture = {
  /** Day the playback (re)started from. */
  debut: number;
  /** Time since then, in ms. */
  ecouleMs: number;
  /** 1 or 2. */
  vitesse: number;
  /** Last day of the season: the playback stops there. */
  fin: number;
  reduit: boolean;
};

/** The day of the playback after `ecouleMs`, and whether it has reached the end. */
export function jourDeLecture(e: EntreeLecture): { jour: number; termine: boolean } {
  let avance = (e.ecouleMs / 1000) * JOURS_PAR_SECONDE * e.vitesse;
  if (e.reduit) avance = Math.floor(avance / PALIER_JOURS) * PALIER_JOURS;
  const jour = e.debut + avance;
  return jour >= e.fin ? { jour: e.fin, termine: true } : { jour, termine: false };
}
