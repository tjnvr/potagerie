// Layout of the timeline (S01 F-VIS-2, S05 §4): pixels to days and back,
// scrolling, zoom, the bars and their sub-rows. Pure functions, tested.
//
// Nothing here is a rule of S03. The dates of a bar (planting out, harvest,
// end), the conflicts, the free slots and the suggestions all come from Rust
// in the year view; this file only decides where to draw them.
import type { CreneauView, PlancheView, SerieView, YearView } from "../bindings";
import { jourDe, premierJanvier } from "../dates";

/** The visible part of the timeline: from `debutJour` on, `pxParJour` pixels
 *  per day, `largeur` pixels wide. */
export type Echelle = { debutJour: number; pxParJour: number; largeur: number };

/** 12 months at first; from 3 months to 3 years by pinching or with the wheel (F-VIS-2). */
export const JOURS_PAR_DEFAUT = 365;
/** On a phone width the timeline opens on 6 months: 12 would not be readable (decision of S05 v0.9). */
export const JOURS_PAR_DEFAUT_ETROIT = 180;
export const JOURS_MIN = 90;
export const JOURS_MAX = 1095;
/** Pointer moves under this distance, in pixels, are a click, not a scroll. */
export const SEUIL_DEFILEMENT_PX = 4;

export function joursVisibles(e: Echelle): number {
  return e.largeur / e.pxParJour;
}

/** The pixel of a day, which may be fractional. */
export function xDe(e: Echelle, jour: number): number {
  return (jour - e.debutJour) * e.pxParJour;
}

/** The day under a pixel, which may be fractional. */
export function jourDeX(e: Echelle, x: number): number {
  return e.debutJour + x / e.pxParJour;
}

/** The civil day under a pixel: the pointer to a date. `x` is relative to the
 *  left of the drawing area, not to the window. */
export function jourSousLePointeur(e: Echelle, clientX: number, gauche: number): number {
  return Math.floor(jourDeX(e, clientX - gauche));
}

/** A scale showing `joursVisibles` days with `centreJour` in the middle. */
export function echelleCentree(largeur: number, centreJour: number, nombreDeJours = JOURS_PAR_DEFAUT): Echelle {
  const pxParJour = largeur / nombreDeJours;
  return { debutJour: centreJour - nombreDeJours / 2, pxParJour, largeur };
}

/** The finger or mouse moved by `dxPx` since the start of the scroll: the
 *  timeline follows it, so moving right shows earlier days. */
export function defiler(depart: Echelle, dxPx: number): Echelle {
  return { ...depart, debutJour: depart.debutJour - dxPx / depart.pxParJour };
}

/** Zoom by `facteur` (> 1 widens the view onto more days), keeping the day
 *  under `ancrageX` where it is. Clamped from 3 months to 3 years. */
export function zoomer(e: Echelle, facteur: number, ancrageX: number): Echelle {
  const nombre = Math.min(JOURS_MAX, Math.max(JOURS_MIN, joursVisibles(e) * facteur));
  const pxParJour = e.largeur / nombre;
  const ancre = jourDeX(e, ancrageX);
  return { largeur: e.largeur, pxParJour, debutJour: ancre - ancrageX / pxParJour };
}

/** The scale with `jour` in the middle, same zoom. */
export function centrerSur(e: Echelle, jour: number): Echelle {
  return { ...e, debutJour: jour - joursVisibles(e) / 2 };
}

/** Scrolls as little as needed to bring `jour` into view, with a margin of a
 *  tenth of the width. Unchanged if it is already visible. */
export function garderVisible(e: Echelle, jour: number): Echelle {
  const marge = joursVisibles(e) * 0.1;
  if (jour < e.debutJour + marge) return { ...e, debutJour: jour - marge };
  if (jour > e.debutJour + joursVisibles(e) - marge) return { ...e, debutJour: jour - joursVisibles(e) + marge };
  return e;
}

/** A bar dragged by `dxPx` moves by whole days (S03 §2: the shift is in days). */
export function decalageEnJours(dxPx: number, pxParJour: number): number {
  return Math.round(dxPx / pxParJour);
}

/** The years the visible days touch: the views to load. */
export function anneesVisibles(e: Echelle): number[] {
  const debut = new Date(Math.floor(e.debutJour) * 86_400_000).getUTCFullYear();
  const fin = new Date(Math.ceil(e.debutJour + joursVisibles(e)) * 86_400_000).getUTCFullYear();
  return Array.from({ length: fin - debut + 1 }, (_, i) => debut + i);
}

/** The first day of each month that touches the visible days, for the columns. */
export type Mois = { jour: number; annee: number; mois: number; jours: number };

export function moisVisibles(e: Echelle): Mois[] {
  const premier = new Date(Math.floor(e.debutJour) * 86_400_000);
  let annee = premier.getUTCFullYear();
  let mois = premier.getUTCMonth();
  const limite = e.debutJour + joursVisibles(e);
  const sortie: Mois[] = [];
  for (;;) {
    const jour = Math.round(Date.UTC(annee, mois, 1) / 86_400_000);
    if (jour >= limite) break;
    const suivant = mois === 11 ? [annee + 1, 0] : [annee, mois + 1];
    const jours = Math.round(Date.UTC(suivant[0]!, suivant[1]!, 1) / 86_400_000) - jour;
    sortie.push({ jour, annee, mois, jours });
    [annee, mois] = suivant as [number, number];
  }
  return sortie;
}

// ---------- Bars, slots and rows ----------

export type Phase = "godets" | "croissance" | "recolte";
export type Segment = { phase: Phase; debut: number; fin: number };

/** A serie as a bar: day numbers, `fin` not included. */
export type Barre = {
  id: string;
  cultureId: string;
  plancheId: string;
  mode: SerieView["mode"];
  debut: number;
  fin: number;
  segments: Segment[];
  troncons: [number, number];
  finManuelle: boolean;
  enConflit: boolean;
};

/** The phases of S03 §2, read from the dates Rust sends: pots from sowing to
 *  planting out (mode abri only), growth to the start of harvest, harvest. */
export function barreDe(s: SerieView, enConflit = false): Barre {
  const m = jourDe(s.planting_out);
  const r = jourDe(s.harvest_start);
  const f = jourDe(s.end);
  const semis = s.mode === "abri" && s.sowing ? jourDe(s.sowing) : m;
  const segments: Segment[] = [];
  if (semis < m) segments.push({ phase: "godets", debut: semis, fin: m });
  if (r > m) segments.push({ phase: "croissance", debut: m, fin: r });
  segments.push({ phase: "recolte", debut: r, fin: f });
  return {
    id: s.id,
    cultureId: s.culture_id,
    plancheId: s.planche_id,
    mode: s.mode,
    debut: Math.min(semis, m),
    fin: f,
    segments,
    troncons: [s.troncon_debut, s.troncon_fin],
    finManuelle: s.manual_end,
    enConflit,
  };
}

export type Creneau = {
  cle: string;
  plancheId: string;
  troncons: [number, number];
  debut: number;
  /** Day after the last free day. */
  fin: number;
  source: CreneauView;
};

export function creneauDe(c: CreneauView): Creneau {
  return {
    cle: `${c.planche_id}/${c.troncon_debut}-${c.troncon_fin}/${c.start}/${c.end}`,
    plancheId: c.planche_id,
    troncons: [c.troncon_debut, c.troncon_fin],
    debut: jourDe(c.start),
    fin: jourDe(c.end) + 1,
    source: c,
  };
}

/** Greedy sub-rows: items that overlap in time are stacked (F-VIS-2). Items
 *  already placed (`fixes`) keep their sub-row. Returns the sub-row of each id. */
export function empiler(
  elements: readonly { id: string; debut: number; fin: number }[],
  fixes: readonly { debut: number; fin: number; voie: number }[] = [],
): Map<string, number> {
  const occupees: { debut: number; fin: number; voie: number }[] = [...fixes];
  const voies = new Map<string, number>();
  for (const e of [...elements].sort((a, b) => a.debut - b.debut || a.fin - b.fin)) {
    let voie = 0;
    while (occupees.some((o) => o.voie === voie && o.debut < e.fin && e.debut < o.fin)) voie++;
    voies.set(e.id, voie);
    occupees.push({ debut: e.debut, fin: e.fin, voie });
  }
  return voies;
}

export type Ligne = {
  planche: PlancheView;
  barres: (Barre & { voie: number })[];
  creneaux: (Creneau & { voie: number })[];
  /** Day ranges where the planche does not exist yet or any more (greyed). */
  absences: { debut: number; fin: number }[];
  voies: number;
};

/**
 * One row per planche, from the year views loaded around the visible days.
 * The rows follow the position on the plan (top to bottom, then left to
 * right); a planche that does not exist in a loaded year is greyed for it.
 * `reference` is the year whose layout names the row.
 */
export function construireLignes(vues: readonly YearView[], reference: number): Ligne[] {
  const triees = [...vues].sort((a, b) => a.year - b.year);
  const planches = new Map<string, PlancheView>();
  for (const vue of triees) {
    for (const p of vue.planches) {
      if (vue.year <= reference || !planches.has(p.id)) planches.set(p.id, p);
    }
  }
  const series = new Map<string, SerieView>();
  const conflits = new Set<string>();
  const creneaux = new Map<string, Creneau>();
  for (const vue of triees) {
    for (const s of vue.series) series.set(s.id, s);
    for (const c of vue.conflicts) {
      conflits.add(c.a);
      conflits.add(c.b);
    }
    for (const c of vue.creneaux) {
      const creneau = creneauDe(c);
      creneaux.set(creneau.cle, creneau);
    }
  }

  const lignes: Ligne[] = [];
  for (const planche of planches.values()) {
    const barres = [...series.values()]
      .filter((s) => s.planche_id === planche.id)
      .map((s) => barreDe(s, conflits.has(s.id)));
    const voiesBarres = empiler(barres);
    const posees = barres.map((b) => ({ debut: b.debut, fin: b.fin, voie: voiesBarres.get(b.id) ?? 0 }));
    const libres = [...creneaux.values()].filter((c) => c.plancheId === planche.id);
    const voiesCreneaux = empiler(
      libres.map((c) => ({ id: c.cle, debut: c.debut, fin: c.fin })),
      posees,
    );
    const absences = triees
      .filter((vue) => !vue.planches.some((p) => p.id === planche.id))
      .map((vue) => ({ debut: premierJanvier(vue.year), fin: premierJanvier(vue.year + 1) }));
    const toutes = [...voiesBarres.values(), ...voiesCreneaux.values()];
    lignes.push({
      planche,
      barres: barres.map((b) => ({ ...b, voie: voiesBarres.get(b.id) ?? 0 })),
      creneaux: libres.map((c) => ({ ...c, voie: voiesCreneaux.get(c.cle) ?? 0 })),
      absences,
      voies: toutes.length ? Math.max(...toutes) + 1 : 1,
    });
  }
  return lignes.sort((a, b) => a.planche.y - b.planche.y || a.planche.x - b.planche.x);
}
