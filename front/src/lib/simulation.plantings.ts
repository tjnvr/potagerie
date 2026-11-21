// The plantings, slots and catalogue of the in-memory fake (see simulation.ts).
//
// Like the rest of the fake, it is NOT the source of truth: a rough copy of
// S02, S03 §2, §4, §5, §6, §8 and §9 so the timeline and the form of a planting
// can be laid out, looked at and driven in a plain browser. It does not copy
// the family tie-break of the suggestions. The real rules are Rust's
// (crates/plan), tested there. Dev only: main.ts never loads it in a build.
import type {
  Appearance,
  AppearanceChange,
  SlotView,
  CropView,
  DatesPreview,
  ErrorCode,
  HarvestView,
  Mode,
  BedView,
  ProblemView,
  ShortcutsView,
  PlantingPreview,
  PlantingInput,
  PlantingView,
  SuggestionView,
  PlotView,
  ZoneView,
} from "./bindings";
import { dateOf, shiftDate, dayOf } from "./dates";

import catalogueJson from "../../../crates/catalogue/crops.json";

type RawCrop = {
  id: string;
  family: string;
  indoor_days: number;
  harvest_after_days: number;
  harvest_duration_days: number;
  planting_out: { start: string; end: string };
  row_cm: number;
  spacing_cm: number;
  perennial?: { lifespan_years: number; harvest_from: string };
};

type Args = Record<string, unknown>;

/** The V1 catalogue (S02): the real file of the Rust core, so that the demo shows
 *  the 67 crops and the red fruits. */
export const CROPS: CropView[] = (catalogueJson as RawCrop[]).map((c) => ({
  id: c.id,
  family: c.family,
  indoor_days: c.indoor_days,
  harvest_after_days: c.harvest_after_days,
  harvest_duration_days: c.harvest_duration_days,
  planting_out_start: c.planting_out.start,
  planting_out_end: c.planting_out.end,
  row_cm: c.row_cm,
  spacing_cm: c.spacing_cm,
  lifespan_years: c.perennial?.lifespan_years ?? null,
  harvest_from: c.perennial?.harvest_from ?? null,
}));

export type Context = {
  plot: () => PlotView | null;
  beds: () => BedView[];
  plantings: () => PlantingView[];
  setPlantings: (plantings: PlantingView[]) => void;
  today: string;
  annee0: number;
  /** Bumps the version of the plan and returns the view of `year`. */
  validate: (year: number) => unknown;
  refusal: (code: ErrorCode) => never;
};

export const NON_HANDLES = Symbol("unhandled");

const crop = (id: string) => CROPS.find((c) => c.id === id);
const day = (iso: string) => dayOf(iso);
const iso = (n: number) => dateOf(n);

type Calculation = {
  sowing: string | null;
  m: string;
  r: string;
  f: string;
  fDefault: string;
  stored: string;
  pots: number;
  harvests: HarvestView[];
};

/** `MM-DD` placed in `year`, 02-29 giving 02-28 (S03 §1). */
function inYear(md: string, year: number): number {
  const [mo = 1, jo = 1] = md.split("-").map(Number);
  const d = Date.UTC(year, mo - 1, jo) / 86_400_000;
  return new Date(d * 86_400_000).getUTCMonth() === mo - 1 ? d : d - 1;
}

const yearOf = (n: number) => new Date(n * 86_400_000).getUTCFullYear();

/** The yearly harvests of a perennial, cut to `[m, f[` (S03 §2 bis). */
function perennialHarvests(from: string, duration: number, r: number, f: number): HarvestView[] {
  const list: HarvestView[] = [];
  for (let y = yearOf(r); inYear(from, y) < f; y++) {
    const start = inYear(from, y);
    const end = Math.min(start + duration, f);
    if (start >= r && end > start) list.push({ start: iso(start), end: iso(end) });
  }
  return list;
}

/** S03 §2 and §2 bis. Throws the code Rust would answer. */
function compute(c: CropView, entree: PlantingInput, refusal: (code: ErrorCode) => never): Calculation {
  // Rust (`pot_days_for`): the days entered only count for an indoor start of a crop without pots;
  // in every other mode the catalogue's time in pots is kept.
  const pots = c.indoor_days > 0 || entree.mode !== "indoor" ? c.indoor_days : (entree.indoor_days ?? 0);
  if (entree.mode === "indoor" && pots < 1) refusal("pot_days_required");
  const A = entree.mode === "indoor" ? pots : 0;
  const stored = entree.mode === "indoor" && entree.date_is_transplanting ? iso(day(entree.date) - A) : entree.date;
  const s = day(stored);
  const m = entree.mode === "indoor" ? s + A : s;
  let r = entree.mode === "open_ground" ? m + c.harvest_after_days + c.indoor_days : m + c.harvest_after_days;
  let fDefault = r + c.harvest_duration_days;
  if (c.harvest_from && c.lifespan_years) {
    const first = inYear(c.harvest_from, yearOf(r));
    r = first >= r ? first : inYear(c.harvest_from, yearOf(r) + 1);
    fDefault = inYear(iso(m).slice(5), yearOf(m) + c.lifespan_years);
  }
  const f = entree.manual_end ? day(entree.manual_end) : fDefault;
  if (entree.manual_end && f <= (c.harvest_from ? m : r)) refusal("manual_end_invalid");
  if (f <= m) refusal("occupation_empty");
  return {
    harvests: c.harvest_from ? perennialHarvests(c.harvest_from, c.harvest_duration_days, r, f) : [{ start: iso(r), end: iso(f) }],
    sowing: entree.mode === "plants" ? null : stored,
    m: iso(m),
    r: iso(r),
    f: iso(f),
    fDefault: iso(fDefault),
    stored,
    pots: entree.mode === "indoor" ? pots : c.indoor_days,
  };
}

function quantity(c: CropView, p: BedView, plot: PlotView, k: number) {
  return {
    rows: Math.max(1, Math.floor(p.width_cm / c.row_cm)),
    plants: Math.max(1, Math.floor((k * plot.cell_cm) / c.spacing_cm)),
  };
}

/** What a planting looks like on a day, thresholds of S03 §3 (dev copy). `harvests` is
 *  only given for a perennial. */
function apparenceLe(
  d: string,
  sowing: string | null,
  m: string,
  r: string,
  f: string,
  mode: PlantingView["mode"],
  harvests: HarvestView[] | null,
): Appearance {
  if (d < m) return mode === "indoor" && sowing !== null && d >= sowing ? { appearance: "label" } : { appearance: "nothing" };
  if (d >= f) return { appearance: "nothing" };
  if (harvests) {
    if (harvests.some((h) => h.start <= d && d < h.end)) return { appearance: "plant", stage: "harvest" };
    const md = d.slice(5);
    if (md < "03-01" || md >= "11-15") return { appearance: "plant", stage: "dormant" };
    if (d >= r) return { appearance: "plant", stage: "adult" };
  }
  const elapsed = (Date.parse(d) - Date.parse(m)) / 86_400_000;
  const duration = (Date.parse(r) - Date.parse(m)) / 86_400_000;
  const stage =
    duration <= 0 || elapsed >= duration ? "harvest" : elapsed * 100 < duration * 15 ? "sowing" : elapsed * 100 < duration * 50 ? "young_shoot" : "adult";
  return { appearance: "plant", stage: mode === "plants" && stage === "sowing" ? "young_shoot" : stage };
}

function changements(
  sowing: string | null,
  m: string,
  r: string,
  f: string,
  mode: PlantingView["mode"],
  harvests: HarvestView[] | null,
): AppearanceChange[] {
  const result: AppearanceChange[] = [];
  let previous = "nothing:";
  for (let d = sowing && sowing < m ? sowing : m; d <= f; d = shiftDate(d, 1)) {
    const a = apparenceLe(d, sowing, m, r, f, mode, harvests);
    const key = a.appearance === "plant" ? `plant:${a.stage}` : `${a.appearance}:`;
    if (key !== previous) {
      result.push({ from: d, appearance: a });
      previous = key;
    }
  }
  return result;
}

function enView(id: string, e: PlantingInput, c: CropView, p: BedView, plot: PlotView, calculation: Calculation): PlantingView {
  const q = quantity(c, p, plot, e.segment_end - e.segment_start + 1);
  return {
    id,
    crop_id: e.crop_id,
    bed_id: e.bed_id,
    segment_start: e.segment_start,
    segment_end: e.segment_end,
    mode: e.mode,
    date: calculation.stored,
    sowing: calculation.sowing,
    planting_out: calculation.m,
    harvest_start: calculation.r,
    end: calculation.f,
    manual_end: !!e.manual_end,
    pot_days: calculation.pots,
    rows: q.rows,
    plants_by_row: q.plants,
    appearance_today: { appearance: "nothing" },
    appearance_changes: changements(
      calculation.sowing,
      calculation.m,
      calculation.r,
      calculation.f,
      e.mode,
      c.harvest_from ? calculation.harvests : null,
    ),
    harvests: calculation.harvests,
  };
}

type Occ = { z1: number; z2: number; m: number; f: number };

const occ = (s: Pick<PlantingView, "segment_start" | "segment_end" | "planting_out" | "end">): Occ => ({
  z1: s.segment_start,
  z2: s.segment_end,
  m: day(s.planting_out),
  f: day(s.end),
});

/** Zones and occupations overlap (S03 §5). */
function enConflict(a: Occ, b: Occ): ZoneView & { start: string; end: string } | null {
  if (a.z1 > b.z2 || b.z1 > a.z2 || a.m >= b.f || b.m >= a.f) return null;
  return {
    segment_start: Math.max(a.z1, b.z1),
    segment_end: Math.min(a.z2, b.z2),
    start: iso(Math.max(a.m, b.m)),
    end: iso(Math.min(a.f, b.f)),
  };
}

function problems(ctx: Context, ignore: string | null, e: PlantingInput, calculation: Calculation): ProblemView[] {
  const base = { with: null, zone: null, start: null, end: null, year: null, segments: null };
  const bed = ctx.beds().find((p) => p.id === e.bed_id);
  if (!bed) return [{ ...base, code: "bed_unknown" }];
  const list: ProblemView[] = [];
  if (e.segment_end > bed.segments) {
    list.push({
      ...base,
      code: "zone_outside_bed",
      zone: { segment_start: e.segment_start, segment_end: e.segment_end },
      year: Number(calculation.m.slice(0, 4)),
      segments: bed.segments,
    });
  }
  const moi = { z1: e.segment_start, z2: e.segment_end, m: day(calculation.m), f: day(calculation.f) };
  for (const other of ctx.plantings()) {
    if (other.id === ignore || other.bed_id !== e.bed_id) continue;
    const c = enConflict(moi, occ(other));
    if (c) {
      list.push({
        ...base,
        code: "conflict",
        with: other.id,
        zone: { segment_start: c.segment_start, segment_end: c.segment_end },
        start: c.start,
        end: c.end,
      });
    }
  }
  return list;
}

/** The first run of `k` segments free over the occupation (S03 §4). */
function shortcut(ctx: Context, bed: BedView, ignore: string | null, m: number, f: number, fraction: number): ZoneView | null {
  const n = bed.segments;
  const k = Math.max(1, Math.floor(n * fraction + 0.5));
  const free = (a: number, b: number) =>
    ctx
      .plantings()
      .filter((s) => s.id !== ignore && s.bed_id === bed.id)
      .every((s) => !(s.segment_start <= b && a <= s.segment_end && day(s.planting_out) < f && m < day(s.end)));
  for (let t1 = 1; t1 + k - 1 <= n; t1++) {
    if (free(t1, t1 + k - 1)) return { segment_start: t1, segment_end: t1 + k - 1 };
  }
  return null;
}

function preview(ctx: Context, ignore: string | null, e: PlantingInput): PlantingPreview {
  const empty: ShortcutsView = { all: null, half: null, third: null, quarter: null };
  const c = crop(e.crop_id);
  const bed = ctx.beds().find((p) => p.id === e.bed_id);
  const plot = ctx.plot();
  if (!c || !bed || !plot) {
    return { dates: null, problems: [{ code: "crop_unknown", with: null, zone: null, start: null, end: null, year: null, segments: null }], shortcuts: empty, occupied: [] };
  }
  let calculation: Calculation;
  try {
    calculation = compute(c, e, ctx.refusal);
  } catch (error) {
    const code = (error as { code: ErrorCode }).code;
    return { dates: null, problems: [{ code, with: null, zone: null, start: null, end: null, year: null, segments: null }], shortcuts: empty, occupied: [] };
  }
  const m = day(calculation.m);
  const f = day(calculation.f);
  const q = quantity(c, bed, plot, e.segment_end - e.segment_start + 1);
  const dates: DatesPreview = {
    date: calculation.stored,
    sowing: calculation.sowing,
    planting_out: calculation.m,
    harvest_start: calculation.r,
    end: calculation.f,
    default_end: calculation.fDefault,
    manual_end: !!e.manual_end,
    pot_days: calculation.pots,
    rows: q.rows,
    plants_by_row: q.plants,
    harvests: calculation.harvests,
  };
  const occupied = ctx
    .plantings()
    .filter((s) => s.id !== ignore && s.bed_id === e.bed_id && day(s.planting_out) < f && m < day(s.end))
    .map((s) => ({ segment_start: s.segment_start, segment_end: s.segment_end }));
  return {
    dates,
    problems: problems(ctx, ignore, e, calculation),
    shortcuts: {
      all: shortcut(ctx, bed, ignore, m, f, 1),
      half: shortcut(ctx, bed, ignore, m, f, 1 / 2),
      third: shortcut(ctx, bed, ignore, m, f, 1 / 3),
      quarter: shortcut(ctx, bed, ignore, m, f, 1 / 4),
    },
    occupied,
  };
}

// ---------- Free slots and suggestions (S03 §8, §9) ----------

function suggestions(ctx: Context, start: number, end: number): SuggestionView[] {
  const N = end - start + 1;
  const today = day(ctx.today);
  const year = Number(iso(start).slice(0, 4));
  const list: (SuggestionView & { name: string })[] = [];
  for (const c of CROPS) {
    if (c.lifespan_years) continue; // a perennial is never suggested (S03 §9)
    const [mo = 1, jo = 1] = c.planting_out_start.split("-").map(Number);
    const [mf = 1, jf = 1] = c.planting_out_end.split("-").map(Number);
    const windows = [year - 1, year].map((a) => [Date.UTC(a, mo - 1, jo) / 86_400_000, Date.UTC(a + (mo > mf ? 1 : 0), mf - 1, jf) / 86_400_000]);
    let m: number | null = null;
    for (let d = start; d <= start + 27 && m === null; d++) {
      if (windows.some(([a, b]) => a! <= d && d <= b!)) m = d;
    }
    if (m === null) continue;
    let mode: Mode = "open_ground";
    let sowing = m;
    if (c.indoor_days > 0) {
      if (m - c.indoor_days >= today) {
        mode = "indoor";
        sowing = m - c.indoor_days;
      } else {
        mode = "plants";
      }
    }
    const r = mode === "open_ground" ? m + c.harvest_after_days + c.indoor_days : m + c.harvest_after_days;
    if (r + Math.ceil(c.harvest_duration_days / 2) > end + 1) continue;
    const f = Math.min(end + 1, r + c.harvest_duration_days);
    list.push({
      name: c.id,
      crop_id: c.id,
      mode,
      date: iso(sowing),
      planting_out: iso(m),
      harvest_start: iso(r),
      end: iso(f),
      manual_end: f < r + c.harvest_duration_days ? iso(f) : null,
      score_tenths: Math.round(((f - start) / N) * 10),
      recent_family: false,
    });
  }
  list.sort((a, b) => b.score_tenths - a.score_tenths || a.name.localeCompare(b.name));
  // Like Rust: the first 3, and the crops tied with the 3rd.
  const last = list[2]?.score_tenths;
  return list.filter((s, i) => i < 3 || s.score_tenths === last).map(({ name: _name, ...s }) => s);
}

export function slots(ctx: Context, year: number): SlotView[] {
  const sortie: SlotView[] = [];
  const startWindow = day(`${year}-03-01`);
  const endWindow = day(`${year}-11-15`);
  for (const p of ctx.beds()) {
    const doubles = ctx.plantings().filter((s) => s.bed_id === p.id).map(occ);
    const coupes = new Set<number>([startWindow, endWindow + 1]);
    for (const o of doubles) {
      for (const d of [o.m, o.f]) if (d > startWindow && d <= endWindow) coupes.add(d);
    }
    const bornes = [...coupes].sort((a, b) => a - b);
    const segments = bornes.slice(0, -1).map((start, i) => ({ start, end: bornes[i + 1]! }));
    const free = (seg: { start: number; end: number }) =>
      Array.from({ length: p.segments }, (_, i) => i + 1).map((t) => !doubles.some((o) => o.z1 <= t && t <= o.z2 && o.m < seg.end && seg.start < o.f));
    const candidates = new Map<string, { z1: number; z2: number; start: number; end: number }>();
    segments.forEach((seg, i) => {
      const l = free(seg);
      for (let a = 0; a < l.length; a++) {
        if (!l[a] || (a > 0 && l[a - 1])) continue;
        let b = a;
        while (b + 1 < l.length && l[b + 1]) b++;
        const ok = (s: { start: number; end: number }) => free(s).slice(a, b + 1).every(Boolean);
        let g = i;
        let d = i;
        while (g > 0 && ok(segments[g - 1]!)) g--;
        while (d + 1 < segments.length && ok(segments[d + 1]!)) d++;
        const c = { z1: a + 1, z2: b + 1, start: segments[g]!.start, end: segments[d]!.end };
        candidates.set(`${c.z1}-${c.z2}-${c.start}-${c.end}`, c);
      }
    });
    const kept = [...candidates.values()]
      .filter((c) => c.end - c.start > 35)
      .filter((c, _i, all) => !all.some((o) => o !== c && o.z1 <= c.z1 && c.z2 <= o.z2 && o.start <= c.start && c.end <= o.end));
    for (const c of kept) {
      sortie.push({
        bed_id: p.id,
        segment_start: c.z1,
        segment_end: c.z2,
        start: iso(c.start),
        end: iso(c.end - 1),
        suggestions: suggestions(ctx, c.start, c.end - 1),
      });
    }
  }
  return sortie;
}

/** A planting as the form of the seed describes it; checked like a real one. */
export function createPlanting(ctx: Context, entree: PlantingInput): PlantingView {
  const c = crop(entree.crop_id);
  const p = ctx.beds().find((x) => x.id === entree.bed_id);
  const plot = ctx.plot();
  if (!c) ctx.refusal("crop_unknown");
  if (!p || !plot) ctx.refusal("bed_unknown");
  const calculation = compute(c, entree, ctx.refusal);
  const [premier] = problems(ctx, null, entree, calculation);
  if (premier) ctx.refusal(premier.code);
  return enView(crypto.randomUUID(), entree, c, p, plot, calculation);
}

/** The commands of the plantings, the slots and the catalogue. */
export function gerer(ctx: Context, cmd: string, args: Args): unknown {
  const year = Number(args.year ?? ctx.annee0);
  switch (cmd) {
    case "catalogue":
      return CROPS;
    case "crops_of_season": {
      const d = day(String(args.date));
      const an = Number(iso(d).slice(0, 4));
      return CROPS.filter((c) => {
        const [mo = 1, jo = 1] = c.planting_out_start.split("-").map(Number);
        const [mf = 1, jf = 1] = c.planting_out_end.split("-").map(Number);
        return [an - 1, an].some((a) => Date.UTC(a, mo - 1, jo) / 86_400_000 <= d && d <= Date.UTC(a + (mo > mf ? 1 : 0), mf - 1, jf) / 86_400_000);
      }).map((c) => c.id);
    }
    case "preview_planting":
      return preview(ctx, (args.id as string | null) ?? null, args.planting as PlantingInput);
    case "create_planting": {
      const created = createPlanting(ctx, args.planting as PlantingInput);
      ctx.setPlantings([...ctx.plantings(), created]);
      return ctx.validate(year);
    }
    case "modify_planting": {
      const id = String(args.id);
      const entree = args.planting as PlantingInput;
      const c = crop(entree.crop_id);
      const p = ctx.beds().find((x) => x.id === entree.bed_id);
      const plot = ctx.plot();
      if (!ctx.plantings().some((s) => s.id === id)) ctx.refusal("planting_unknown");
      if (!c) ctx.refusal("crop_unknown");
      if (!p || !plot) ctx.refusal("bed_unknown");
      const calculation = compute(c, entree, ctx.refusal);
      const [premier] = problems(ctx, id, entree, calculation);
      if (premier) ctx.refusal(premier.code);
      ctx.setPlantings(ctx.plantings().map((s) => (s.id === id ? enView(id, entree, c, p, plot, calculation) : s)));
      return ctx.validate(year);
    }
    case "shift_planting": {
      const id = String(args.id);
      const days = Number(args.days);
      const s = ctx.plantings().find((x) => x.id === id);
      if (!s) ctx.refusal("planting_unknown");
      const entree: PlantingInput = {
        crop_id: s.crop_id,
        bed_id: s.bed_id,
        segment_start: s.segment_start,
        segment_end: s.segment_end,
        mode: s.mode,
        date: iso(day_(s.date) + days),
        indoor_days: s.pot_days,
        manual_end: s.manual_end ? iso(day_(s.end) + days) : null,
      };
      const c = crop(s.crop_id)!;
      const p = ctx.beds().find((x) => x.id === s.bed_id)!;
      const calculation = compute(c, entree, ctx.refusal);
      const [premier] = problems(ctx, id, entree, calculation);
      if (premier) ctx.refusal(premier.code);
      ctx.setPlantings(ctx.plantings().map((x) => (x.id === id ? enView(id, entree, c, p, ctx.plot()!, calculation) : x)));
      return ctx.validate(year);
    }
    case "delete_planting": {
      if (!ctx.plantings().some((s) => s.id === args.id)) ctx.refusal("planting_unknown");
      ctx.setPlantings(ctx.plantings().filter((s) => s.id !== args.id));
      return ctx.validate(year);
    }
    default:
      return NON_HANDLES;
  }
}

const day_ = day;
