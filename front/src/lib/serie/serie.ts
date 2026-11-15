// The form of E07 (S01 F-PLA-1, F-PLA-2): what the user types, and how it
// becomes the input of the commands. No rule of S03 here: the dates, the
// quantities, the conflicts and the zones of the shortcuts all come back from
// `apercu_serie` (Rust). This file only moves values between the form and the
// commands, and handles the clicks on the strip of troncons.
import type {
  CultureView,
  Mode,
  RaccourcisView,
  SerieApercu,
  SerieInput,
  SerieView,
  SuggestionView,
  ZoneView,
} from "../bindings";
import { jourDe } from "../dates";

export type Zone = { debut: number; fin: number };

export type Formulaire = {
  cultureId: string;
  plancheId: string;
  mode: Mode;
  /** The date the user types: sowing, planting, or transplanting if `repiquage`. */
  date: string;
  /** « Je connais plutôt la date de repiquage » (S03 §2, mode abri). */
  repiquage: boolean;
  /** Days in pots, typed, for a crop the catalogue does not start in pots. */
  abriJours: string;
  /** End modified by hand, or `null` to follow the calculation. */
  finManuelle: string | null;
  zone: Zone;
};

/** Days in pots proposed for a crop without pots in the catalogue (S03 T9 uses 21). */
export const JOURS_GODETS_PAR_DEFAUT = "21";

export function versZone(z: ZoneView): Zone {
  return { debut: z.troncon_debut, fin: z.troncon_fin };
}

export function memeZone(a: Zone | null | undefined, b: Zone | null | undefined): boolean {
  return !!a && !!b && a.debut === b.debut && a.fin === b.fin;
}

/** A click on troncon `i` of the strip (S01 F-PLA-1: contiguous troncons): on
 *  a single troncon it extends the zone to `i`, otherwise it starts again from `i`. */
export function zoneApresClic(zone: Zone, i: number): Zone {
  if (zone.debut === zone.fin && i !== zone.debut) {
    return { debut: Math.min(zone.debut, i), fin: Math.max(zone.debut, i) };
  }
  return { debut: i, fin: i };
}

export type CleRaccourci = "toute" | "moitie" | "tiers" | "quart";
export const RACCOURCIS: CleRaccourci[] = ["toute", "moitie", "tiers", "quart"];

/** The zone of a shortcut, as Rust computed it (`null`: greyed out). */
export function zoneDuRaccourci(r: RaccourcisView, cle: CleRaccourci): Zone | null {
  const z = r[cle];
  return z ? versZone(z) : null;
}

/** The first shortcut Rust says is free, to start from when the wanted zone is not. */
export function premierRaccourciLibre(r: RaccourcisView): Zone | null {
  for (const cle of RACCOURCIS) {
    const z = zoneDuRaccourci(r, cle);
    if (z) return z;
  }
  return null;
}

/** The mode a new serie starts with: pots when the catalogue has a time in
 *  pots for the crop, otherwise sowing in the ground. The user picks freely
 *  (S01 F-PLA-1): this is only the card that is checked at first. */
export function modeInitial(culture: CultureView | undefined): Mode {
  return culture && culture.abri_jours > 0 ? "abri" : "pleine_terre";
}

export function formulaireVide(
  culture: CultureView | undefined,
  plancheId: string,
  troncons: number,
  date: string,
): Formulaire {
  return {
    cultureId: culture?.id ?? "",
    plancheId,
    mode: modeInitial(culture),
    date,
    repiquage: false,
    abriJours: JOURS_GODETS_PAR_DEFAUT,
    finManuelle: null,
    zone: { debut: 1, fin: Math.max(1, troncons) },
  };
}

/** A suggestion of a free slot fills the form: crop, mode, date, zone and the
 *  proposed end (S03 §9, which counts as an end modified by hand). */
export function appliquerSuggestion(
  f: Formulaire,
  s: SuggestionView,
  zone: Zone,
): Formulaire {
  return {
    ...f,
    cultureId: s.culture_id,
    mode: s.mode,
    date: s.date,
    repiquage: false,
    finManuelle: s.fin_manuelle ?? null,
    zone,
  };
}

/** The form of a serie being edited: the stored date is the sowing or planting date. */
export function formulaireDepuisSerie(s: SerieView): Formulaire {
  return {
    cultureId: s.culture_id,
    plancheId: s.planche_id,
    mode: s.mode,
    date: s.date,
    repiquage: false,
    abriJours: s.pot_days > 0 ? String(s.pot_days) : JOURS_GODETS_PAR_DEFAUT,
    finManuelle: s.manual_end ? s.end : null,
    zone: { debut: s.troncon_debut, fin: s.troncon_fin },
  };
}

/** The input of `creer_serie`, `modifier_serie` and `apercu_serie`. */
export function versSerieInput(f: Formulaire): SerieInput {
  const jours = Number.parseInt(f.abriJours, 10);
  return {
    culture_id: f.cultureId,
    planche_id: f.plancheId,
    troncon_debut: f.zone.debut,
    troncon_fin: f.zone.fin,
    mode: f.mode,
    date: f.date,
    abri_jours: f.mode === "abri" && Number.isFinite(jours) && jours > 0 ? jours : null,
    fin_manuelle: f.finManuelle,
    date_est_repiquage: f.mode === "abri" && f.repiquage,
  };
}

/** Toggling « Je connais plutôt la date de repiquage »: the field shows the
 *  other date, as Rust computed it, so the serie does not move. */
export function basculerRepiquage(f: Formulaire, apercu: SerieApercu | null): Formulaire {
  const d = apercu?.dates;
  if (!d) return { ...f, repiquage: !f.repiquage };
  return f.repiquage
    ? { ...f, repiquage: false, date: d.date }
    : { ...f, repiquage: true, date: d.planting_out };
}

/** Typing a date for the end: the one Rust would calculate is not « by hand ». */
export function finSaisie(date: string, finCalculee: string): string | null {
  return date === "" || date === finCalculee ? null : date;
}

export type Segment = { phase: "godets" | "croissance" | "recolte"; jours: number };

/** The proportions of the mini-frise (F-PLA-1), from the dates Rust sent. */
export function segmentsMiniFrise(d: NonNullable<SerieApercu["dates"]>, mode: Mode): Segment[] {
  const m = jourDe(d.planting_out);
  const r = jourDe(d.harvest_start);
  const f = jourDe(d.end);
  const semis = mode === "abri" && d.sowing ? jourDe(d.sowing) : m;
  const segments: Segment[] = [];
  if (m > semis) segments.push({ phase: "godets", jours: m - semis });
  if (r > m) segments.push({ phase: "croissance", jours: r - m });
  if (f > r) segments.push({ phase: "recolte", jours: f - r });
  return segments;
}
