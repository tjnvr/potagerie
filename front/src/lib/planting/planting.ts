// The form of E07 (S01 F-PLA-1, F-PLA-2): what the user types, and how it
// becomes the input of the commands. No rule of S03 here: the dates, the
// quantities, the conflicts and the zones of the shortcuts all come back from
// `preview_planting` (Rust). This file only moves values between the form and the
// commands, and handles the clicks on the strip of segments.
import type {
  CropView,
  Mode,
  ShortcutsView,
  PlantingPreview,
  PlantingInput,
  PlantingView,
  SuggestionView,
  ZoneView,
} from "../bindings";
import { dayOf } from "../dates";

export type Zone = { start: number; end: number };

export type Form = {
  cropId: string;
  bedId: string;
  mode: Mode;
  /** The date the user types: sowing, planting, or transplanting if `transplanting`. */
  date: string;
  /** « I rather know the transplanting date » (S03 §2, mode indoor). */
  transplanting: boolean;
  /** Days in pots, typed, for a crop the catalogue does not start in pots. */
  indoorDays: string;
  /** End modified by hand, or `null` to follow the calculation. */
  manualEnd: string | null;
  zone: Zone;
};

/** Days in pots proposed for a crop without pots in the catalogue (S03 T9 uses 21). */
export const DAYS_POTS_BY_DEFAULT = "21";

export function versZone(z: ZoneView): Zone {
  return { start: z.segment_start, end: z.segment_end };
}

export function sameZone(a: Zone | null | undefined, b: Zone | null | undefined): boolean {
  return !!a && !!b && a.start === b.start && a.end === b.end;
}

/** A click on segment `i` of the strip (S01 F-PLA-1: contiguous segments): on
 *  a single segment it extends the zone to `i`, otherwise it starts again from `i`. */
export function zoneAfterClick(zone: Zone, i: number): Zone {
  if (zone.start === zone.end && i !== zone.start) {
    return { start: Math.min(zone.start, i), end: Math.max(zone.start, i) };
  }
  return { start: i, end: i };
}

export type KeyShortcut = "all" | "half" | "third" | "quarter";
export const SHORTCUTS: KeyShortcut[] = ["all", "half", "third", "quarter"];

/** The zone of a shortcut, as Rust computed it (`null`: greyed out). */
export function zoneFromShortcut(r: ShortcutsView, key: KeyShortcut): Zone | null {
  const z = r[key];
  return z ? versZone(z) : null;
}

/** The first shortcut Rust says is free, to start from when the wanted zone is not. */
export function firstShortcutFree(r: ShortcutsView): Zone | null {
  for (const key of SHORTCUTS) {
    const z = zoneFromShortcut(r, key);
    if (z) return z;
  }
  return null;
}

/** The mode a new planting starts with: pots when the catalogue has a time in
 *  pots for the crop, otherwise sowing in the ground. The user picks freely
 *  (S01 F-PLA-1): this is only the card that is checked at first. */
export function modeInitial(crop: CropView | undefined): Mode {
  return crop && crop.indoor_days > 0 ? "indoor" : "open_ground";
}

export function formEmpty(
  crop: CropView | undefined,
  bedId: string,
  segments: number,
  date: string,
): Form {
  return {
    cropId: crop?.id ?? "",
    bedId,
    mode: modeInitial(crop),
    date,
    transplanting: false,
    indoorDays: DAYS_POTS_BY_DEFAULT,
    manualEnd: null,
    zone: { start: 1, end: Math.max(1, segments) },
  };
}

/** A suggestion of a free slot fills the form: crop, mode, date, zone and the
 *  proposed end (S03 §9, which counts as an end modified by hand). */
export function applySuggestion(
  f: Form,
  s: SuggestionView,
  zone: Zone,
): Form {
  return {
    ...f,
    cropId: s.crop_id,
    mode: s.mode,
    date: s.date,
    transplanting: false,
    manualEnd: s.manual_end ?? null,
    zone,
  };
}

/** The form of a planting being edited: the stored date is the sowing or planting date. */
export function formSincePlanting(s: PlantingView): Form {
  return {
    cropId: s.crop_id,
    bedId: s.bed_id,
    mode: s.mode,
    date: s.date,
    transplanting: false,
    indoorDays: s.pot_days > 0 ? String(s.pot_days) : DAYS_POTS_BY_DEFAULT,
    manualEnd: s.manual_end ? s.end : null,
    zone: { start: s.segment_start, end: s.segment_end },
  };
}

/** The input of `create_planting`, `modify_planting` and `preview_planting`. */
export function toPlantingInput(f: Form): PlantingInput {
  const days = Number.parseInt(f.indoorDays, 10);
  return {
    crop_id: f.cropId,
    bed_id: f.bedId,
    segment_start: f.zone.start,
    segment_end: f.zone.end,
    mode: f.mode,
    date: f.date,
    indoor_days: f.mode === "indoor" && Number.isFinite(days) && days > 0 ? days : null,
    manual_end: f.manualEnd,
    date_is_transplanting: f.mode === "indoor" && f.transplanting,
  };
}

/** Toggling « I rather know the transplanting date »: the field shows the
 *  other date, as Rust computed it, so the planting does not move. */
export function toggleTransplanting(f: Form, preview: PlantingPreview | null): Form {
  const d = preview?.dates;
  if (!d) return { ...f, transplanting: !f.transplanting };
  return f.transplanting
    ? { ...f, transplanting: false, date: d.date }
    : { ...f, transplanting: true, date: d.planting_out };
}

/** Typing a date for the end: the one Rust would calculate is not « by hand ». */
export function endInput(date: string, endComputed: string): string | null {
  return date === "" || date === endComputed ? null : date;
}

export type Segment = { phase: "pots" | "growth" | "harvest"; days: number };

/** The proportions of the mini-timeline (F-PLA-1), from the dates Rust sent. */
export function segmentsMiniTimeline(d: NonNullable<PlantingPreview["dates"]>, mode: Mode): Segment[] {
  const m = dayOf(d.planting_out);
  const r = dayOf(d.harvest_start);
  const f = dayOf(d.end);
  const sowing = mode === "indoor" && d.sowing ? dayOf(d.sowing) : m;
  const segments: Segment[] = [];
  if (m > sowing) segments.push({ phase: "pots", days: m - sowing });
  if (r > m) segments.push({ phase: "growth", days: r - m });
  if (f > r) segments.push({ phase: "harvest", days: f - r });
  return segments;
}
