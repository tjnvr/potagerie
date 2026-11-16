// In-memory fake of the Rust commands, for `pnpm dev` in a plain browser only
// (main.ts never loads it in a build or inside Tauri). It is NOT the source of
// truth: it only lets the interface be laid out, looked at and driven. It
// copies just enough of the placement refusals of S03 §15 to see the
// « position refused » path; the real rules are Rust's, tested there.
// `?example` starts with a plot, 8 beds and a few plantings. For the
// settings: `?import=old|incoherent|unreadable|recent|file` picks what
// « Import » finds in the file, `?failure` makes the first save fail
// (« Error d'saving »), `?export=failure` makes the export fail.
import { mockIPC } from "@tauri-apps/api/mocks";
import type { AnnotationInput, AnnotationView, ErrorCode, IncoherenceView, BedView, PlantingInput, PlantingView, PlotView, TypeBed, YearView } from "./bindings";
import { dayOf, premierJanuary } from "./dates";
import { NON_HANDLES, createPlanting, slots, gerer, type Context } from "./simulation.plantings";
import { getLocale } from "./paraglide/runtime.js";

/** Example data in the language of the interface (English is the base). */
function tr(en: string, fr: string): string {
  return getLocale() === "fr" ? fr : en;
}

type Args = Record<string, unknown>;
type Geometry = { bed_type: TypeBed; x: number; y: number; length_cm: number; width_cm: number };

const SIZES: Record<TypeBed, [number, number]> = {
  bed: [300, 120],
  square: [100, 100],
  mound: [300, 120],
  planter: [200, 120],
};

function refusal(code: ErrorCode): never {
  throw { code, detail: "simulated" };
}

export function simulerTauri(): void {
  const parameters = new URLSearchParams(location.search);
  // `?date=2027-06-20` pretends today is that day (screenshots, D01 shows 20 june 2027).
  const today = parameters.get("date") ?? new Date().toISOString().slice(0, 10);
  const annee0 = Number(today.slice(0, 4));
  let plot: PlotView | null = null;
  let beds: BedView[] = [];
  let plantings: PlantingView[] = [];
  let history: PlantingView[] = [];
  let version = 0;
  let annotations: AnnotationView[] = [];
  let nextAnnotation = 1;
  /** What `?import=incoherent` leaves to resolve, until the demo plan is emptied. */
  let toResolve: IncoherenceView[] = [];

  const context: Context = {
    plot: () => plot,
    beds: () => beds,
    plantings: () => plantings,
    setPlantings: (list) => (plantings = list),
    today,
    annee0,
    validate: (year) => {
      version++;
      return view(year);
    },
    refusal,
  };

  function bed(name: string, type: TypeBed, x: number, y: number, l: number, h: number): BedView {
    return {
      id: crypto.randomUUID(),
      name,
      bed_type: type,
      x,
      y,
      length_cm: l,
      width_cm: h,
      segments: l / 50,
      plantings_deleted: [],
    };
  }

  if (new URLSearchParams(location.search).has("banc")) {
    // The bench of P01 for the frame rate: 10 beds, one or two crops each.
    plot = { length_cm: 1200, width_cm: 650, cell_cm: 50, aisle_cm: 50 };
    const Y = annee0;
    beds = [50, 200, 350, 500].flatMap((y) => [
      bed(`Lit ${y}`, "bed", 50, y, 400, 100),
      bed(`Lit ${y} bis`, "bed", 550, y, 400, 100),
    ]);
    beds.push(bed("Planter A", "planter", 1050, 50, 100, 100), bed("Planter B", "planter", 1050, 350, 100, 100));
    const plan: [string, PlantingInput["mode"], string][][] = [
      [["lettuce", "open_ground", "03-01"], ["green_bean", "open_ground", "07-20"]],
      [["carrot", "open_ground", "03-15"], ["lettuce", "open_ground", "09-01"]],
      [["tomato", "indoor", "03-10"]],
      [["zucchini", "indoor", "04-15"]],
      [["leek", "indoor", "03-01"]],
      [["lettuce", "open_ground", "03-10"], ["leek", "plants", "09-01"]],
      [["green_bean", "open_ground", "05-10"], ["lettuce", "open_ground", "09-10"]],
      [["carrot", "open_ground", "04-01"]],
      [["tomato", "plants", "05-05"]],
      [["lettuce", "open_ground", "03-05"], ["lettuce", "open_ground", "07-15"]],
    ];
    beds.forEach((p, i) => {
      for (const [crop_id, mode, date] of plan[i] ?? []) {
        plantings.push(
          createPlanting(context, { crop_id, bed_id: p.id, segment_start: 1, segment_end: p.segments, mode, date: `${Y}-${date}`, indoor_days: null, manual_end: null }),
        );
      }
    });
    version = 1;
  } else if (new URLSearchParams(location.search).has("example")) {
    plot = { length_cm: 1100, width_cm: 650, cell_cm: 50, aisle_cm: 50 };
    beds = [
      bed(tr("Back of the garden", "Fond du potager"), "bed", 50, 50, 300, 120),
      bed(tr("Roots", "Racines"), "bed", 400, 50, 300, 120),
      bed(tr("The tomatoes", "Les tomates"), "bed", 50, 220, 400, 100),
      bed(tr("Shed side", "Côté cabane"), "bed", 500, 220, 300, 120),
      bed(tr("The zucchinis", "Les courgettes"), "bed", 50, 370, 300, 120),
      bed(tr("Leek square", "Carré poireaux"), "square", 400, 370, 100, 100),
      bed(tr("Salad square", "Carré salades"), "square", 550, 390, 100, 100),
      bed(tr("End of the aisle", "Bout de l’allée"), "mound", 750, 50, 300, 120),
    ];
    const Y = annee0;
    const [fond, tomatoes, shed, zucchinis, roots, leeks, salads] = [
      beds[0]!,
      beds[2]!,
      beds[3]!,
      beds[4]!,
      beds[1]!,
      beds[5]!,
      beds[6]!,
    ];
    const entree = (
      p: BedView,
      crop_id: string,
      segment_start: number,
      segment_end: number,
      mode: PlantingInput["mode"],
      date: string,
    ): PlantingInput => ({ crop_id, bed_id: p.id, segment_start, segment_end, mode, date, indoor_days: null, manual_end: null });
    for (const e of [
      entree(fond, "lettuce", 1, 6, "indoor", `${Y}-03-01`),
      entree(tomatoes, "tomato", 1, 4, "indoor", `${Y}-03-10`),
      entree(tomatoes, "lettuce", 5, 8, "indoor", `${Y}-04-12`),
      entree(zucchinis, "zucchini", 1, 6, "indoor", `${Y}-04-27`),
      entree(roots, "carrot", 1, 3, "open_ground", `${Y}-03-15`),
      entree(roots, "lettuce", 1, 3, "plants", `${Y}-08-05`),
      entree(roots, "leek", 4, 6, "indoor", `${Y}-05-01`),
      entree(shed, "lettuce", 1, 6, "indoor", `${Y}-03-02`),
      entree(shed, "green_bean", 1, 6, "open_ground", `${Y}-06-20`),
      entree(leeks, "leek", 1, 2, "indoor", `${Y}-03-25`),
      entree(salads, "lettuce", 1, 2, "plants", `${Y}-04-20`),
      entree(salads, "lettuce", 1, 2, "indoor", `${Y}-07-12`),
      entree(fond, "tomato", 1, 6, "plants", `${Y + 1}-05-10`),
      entree(roots, "carrot", 1, 3, "open_ground", `${Y + 1}-03-20`),
    ]) {
      plantings.push(createPlanting(context, e));
    }
    const past = createPlanting(context, entree(roots, "carrot", 1, 3, "open_ground", `${Y - 1}-03-15`));
    plantings = plantings.filter((x) => x.id !== past.id);
    history = [past];
    annotations = [
      { id: "n0", annotation_type: "period", date_start: `${Y}-08-01`, date_end: `${Y}-08-21`, bed_id: null, text: "Holidays", hidden: false },
      { id: "n1", annotation_type: "period", date_start: `${Y}-05-05`, date_end: `${Y}-06-10`, bed_id: tomatoes.id, text: tr("Mulching", "Paillage"), hidden: false },
      { id: "n2", annotation_type: "note", date_start: `${Y}-06-02`, date_end: null, bed_id: tomatoes.id, text: tr("Stake the tomatoes", "Tuteurer les tomates"), hidden: false },
      { id: "n3", annotation_type: "note", date_start: `${Y}-04-10`, date_end: null, bed_id: null, text: tr("Last frosts?", "Dernières gelées ?"), hidden: false },
    ];
    version = 4;
  }


  /** Overlap, aisle and plot edges: a rough copy of S03 §15, for the demo. */
  function verifier(id: string, g: Geometry): void {
    const t = plot;
    if (!t) refusal("plot_absent");
    if (g.x + g.length_cm > t.length_cm || g.y + g.width_cm > t.width_cm) {
      refusal("bed_outside_plot");
    }
    for (const other of beds) {
      if (other.id === id) continue;
      const dx = Math.max(other.x - (g.x + g.length_cm), g.x - (other.x + other.length_cm));
      const dy = Math.max(other.y - (g.y + g.width_cm), g.y - (other.y + other.width_cm));
      if (dx < 0 && dy < 0) refusal("bed_overlaps");
      if (dx < t.aisle_cm && dy < t.aisle_cm) refusal("aisle_too_narrow");
    }
  }

  function view(year: number): YearView {
    const start = premierJanuary(year);
    const key = (x: PlantingView) =>
      dayOf(x.sowing && x.mode === "indoor" ? x.sowing : x.planting_out) < premierJanuary(year + 1) && dayOf(x.end) > start;
    const old = [...history, ...plantings].filter((x) => dayOf(x.end) <= start);
    return {
      year: year,
      today: today,
      read_only: year < annee0,
      plot,
      version,
      updated_at: plot ? new Date().toISOString() : null,
      beds: beds.map((p) => ({
        ...p,
        plantings_deleted: plantings.filter((x) => x.bed_id === p.id && dayOf(x.end) > start).map((x) => x.id),
      })),
      plantings: plantings.filter(key),
      history: old,
      conflicts: [],
      slots: plot && year >= annee0 ? slots(context, year) : [],
      annotations: annotations.filter(
        (a) => a.date_start < `${year + 1}-01-01` && (a.date_end ?? a.date_start) >= `${year}-01-01`,
      ),
      tasks: [],
    };
  }

  function modifiable(year: number): void {
    if (year < annee0) refusal("year_past");
  }

  function verifierAnnotation(year: number, a: AnnotationInput): void {
    modifiable(year);
    if (a.text.trim() === "" || a.text.trim().length > 80) refusal("text_invalid");
    if (a.annotation_type === "period" && (!a.date_end || a.date_end < a.date_start)) refusal("annotation_dates_invalid");
    if (a.bed_id && !beds.some((p) => p.id === a.bed_id)) refusal("bed_unknown");
  }

  const params = new URLSearchParams(location.search);
  let failureSaving = params.has("failure");

  mockIPC((cmd, payload) => {
    const args = (payload ?? {}) as Args;
    const year = Number(args.year ?? annee0);
    if (failureSaving && cmd !== "view_year" && cmd !== "catalogue" && !cmd.startsWith("plugin:")) {
      failureSaving = false;
      refusal("saving");
    }
    const incoherence = (code: string, extra: Partial<Record<string, unknown>>) => ({
      code,
      year: null,
      bed_name: null,
      other_bed_name: null,
      crop_id: null,
      other_crop_id: null,
      count: null,
      ...extra,
    });
    const readIncoherences = () => [
      incoherence("too_many_beds", { year: annee0, count: 11 }),
      incoherence("planting_orphan", { year: annee0, bed_name: tr("Roots", "Racines"), crop_id: "carrot" }),
      incoherence("overlap", { bed_name: tr("The tomatoes", "Les tomates"), crop_id: "tomato", other_crop_id: "lettuce" }),
      incoherence("aisle_too_narrow", { year: annee0, bed_name: tr("Leek square", "Carré poireaux"), other_bed_name: tr("Salad square", "Carré salades") }),
    ];
    switch (cmd) {
      case "plugin:dialog|save":
        return "/tmp/garden-export.json";
      case "plugin:dialog|open":
        return "/tmp/garden-import.json";
      case "export_timeline":
        if (params.get("export") === "failure") refusal("file");
        return null;
      case "create_annotation": {
        const a = args.annotation as AnnotationInput;
        verifierAnnotation(year, a);
        annotations.push({ ...a, date_end: a.annotation_type === "period" ? a.date_end : null, id: `n${100 + nextAnnotation++}` });
        version++;
        return view(year);
      }
      case "modify_annotation": {
        const a = args.annotation as AnnotationInput;
        if (!annotations.some((x) => x.id === args.id)) refusal("annotation_unknown");
        verifierAnnotation(year, a);
        annotations = annotations.map((x) =>
          x.id === args.id ? { ...a, date_end: a.annotation_type === "period" ? a.date_end : null, id: x.id } : x,
        );
        version++;
        return view(year);
      }
      case "delete_annotation":
        modifiable(year);
        annotations = annotations.filter((x) => x.id !== args.id);
        version++;
        return view(year);
      case "incoherences_plan":
        return toResolve;
      case "export_garden":
        if (params.get("export") === "failure") refusal("file");
        return null;
      case "read_import": {
        const cas = params.get("import") ?? "old";
        if (cas === "unreadable") refusal("import_invalid");
        if (cas === "recent") refusal("import_too_recent");
        if (cas === "file") refusal("file");
        return {
          updated_at: cas === "old" ? "2026-11-02T10:00:00Z" : new Date().toISOString(),
          version: 7,
          bed_count: cas === "incoherent" ? 11 : 8,
          planting_count: 12,
          older: cas === "old",
          replaces: plot !== null,
          incoherences: cas === "incoherent" ? readIncoherences() : [],
        };
      }
      case "import_garden":
        toResolve = params.get("import") === "incoherent" ? readIncoherences() : [];
        version++;
        return view(year);
      case "clear_plan":
        modifiable(year);
        beds = [];
        plantings = [];
        toResolve = [];
        version++;
        return view(year);
      case "view_year":
        return view(year);
      case "create_plot": {
        const t = args.plot as PlotView;
        if (t.length_cm < 100 || t.width_cm < 100 || t.length_cm > 10000 || t.width_cm > 10000) {
          refusal("plot_invalid");
        }
        plot = t;
        version = 1;
        return view(year);
      }
      case "modify_plot":
        plot = { ...plot!, ...(args.plot as Partial<PlotView>) };
        version++;
        return view(year);
      case "create_bed": {
        modifiable(year);
        const type = args.bedType as TypeBed;
        const [l, h] = SIZES[type];
        const created = bed(String(args.name), type, Number(args.x), Number(args.y), l, h);
        verifier(created.id, created);
        beds = [...beds, created];
        version++;
        return view(year);
      }
      case "move_bed": {
        modifiable(year);
        const target = beds.find((p) => p.id === args.id)!;
        verifier(target.id, { ...target, x: Number(args.x), y: Number(args.y) });
        beds = beds.map((p) => (p === target ? { ...p, x: Number(args.x), y: Number(args.y) } : p));
        version++;
        return view(year);
      }
      case "modify_bed": {
        modifiable(year);
        const target = beds.find((p) => p.id === args.id)!;
        const g = args.geometry as Geometry | null;
        if (g) {
          if (g.length_cm % plot!.cell_cm !== 0) refusal("length_invalid");
          verifier(target.id, g);
        }
        beds = beds.map((p) =>
          p === target
            ? {
                ...p,
                name: (args.name as string | null) ?? p.name,
                ...(g ? { ...g, segments: g.length_cm / plot!.cell_cm } : {}),
              }
            : p,
        );
        version++;
        return view(year);
      }
      case "plantings_exceeding": {
        const n = Number(args.lengthCm) / plot!.cell_cm;
        return plantings.filter((s) => s.bed_id === args.id && s.segment_end > n).map((s) => s.id);
      }
      case "delete_bed":
        modifiable(year);
        beds = beds.filter((p) => p.id !== args.id);
        plantings = plantings.filter((s) => s.bed_id !== args.id);
        version++;
        return view(year);
      case "erase_data":
        plot = null;
        beds = [];
        plantings = [];
        history = [];
        version = 0;
        return null;
      default: {
        const response = gerer(context, cmd, args);
        if (response !== NON_HANDLES) return response;
        throw { code: "file", detail: `${cmd} is not simulated` };
      }
    }
  });
}
