import { formatCivil } from "./dates";
import { localeTag } from "./locale";
import { commands, type AnnotationView, type AppError, type SlotView, type IncoherenceView, type BedId, type GeometryInput, type PlantingView, type SuggestionView, type YearView } from "./bindings";
import { isFailureSaving, translateError } from "./errors";


/** What a command resolves to (tauri-specta, ErrorHandlingMode::Result). */
export type Result<T> = { status: "ok"; data: T } | { status: "error"; error: AppError };

/** The save indicator of the header (F-CPT-2). */
export type Saving = "saved" | "saving" | "error";

export type Tab = "garden" | "plan";

export type Message = { text: string; tone: "info" | "error" };

/** What opens E07 (new planting and edition): from a bed (« Plant here »),
 *  from a free slot of the timeline (with its suggestions), or from a bar. */
export type OpeningPlanting =
  | { type: "new"; bedId: string; slot?: SlotView; suggestion?: SuggestionView }
  | { type: "edition"; planting: PlantingView };

/** What opens the note form: a new note on a day (on a bed or the whole
 *  garden), or an existing annotation. */
export type OpeningNote =
  | { type: "new"; date: string; bedId: BedId | null }
  | { type: "edition"; annotation: AnnotationView };

/**
 * State of the interface (S05 §4). It keeps the last year view returned by
 * Rust and replaces it entirely after each command. No business rule here.
 */
class App {
  year = $state(new Date().getFullYear());
  tab = $state<Tab>("plan");
  settingsOpen = $state(false);
  /** E07, new planting and edition. */
  plantingOpen = $state<OpeningPlanting | null>(null);
  /** The note form (F-VIS-5). */
  noteOpen = $state<OpeningNote | null>(null);
  /** E11, export of the timeline. */
  exportOpen = $state(false);
  /** What is left to resolve in the plan (S03 §13), computed by Rust. */
  toResolve = $state<IncoherenceView[]>([]);
  /** E09, catalogue. */
  catalogueOpen = $state(false);
  /** The last refusal of a command that changes the plan (not a save failure),
   *  for the screens that word it themselves (a bar that goes back, S03 T12). */
  lastRefusal = $state<AppError | null>(null);
  /** The « Keep here » / « Go back » bubble (S01 F-PLN-2), open while a bed
   *  closer than the aisle waits for an answer. */
  aisleQuestion = $state<{ answer: (keep: boolean) => void } | null>(null);
  view = $state<YearView | null>(null);
  /** Why the first load failed (a damaged data file): the shell says so instead
   *  of an endless « Loading… ». */
  loadFailure = $state<AppError | null>(null);
  saving = $state<Saving>("saved");
  message = $state<Message | null>(null);
  /** The bed copied with Cmd+C (S05 §20): its type, place and size in the
   *  year it was copied in, kept until the app closes, whatever the year. */
  copiedBed = $state<GeometryInput | null>(null);
  #retry: (() => Promise<void>) | null = null;
  /** The bed the editor selects when it opens (« Move » of the bed panel). */
  #bedToSelect: BedId | null = null;

  /** « Move » of the bed panel (F-VIS-7): the plan editor, in the same year,
   *  with this bed already selected. */
  moveBed(id: BedId): void {
    this.#bedToSelect = id;
    this.tab = "plan";
  }

  /** The bed the editor opens with, given once. */
  takeBedToSelect(): BedId | null {
    const id = this.#bedToSelect;
    this.#bedToSelect = null;
    return id;
  }

  /** Opens a year. */
  async load(year: number = this.year): Promise<void> {
    this.year = year;
    const result = await call(() => commands.viewYear(year));
    if (result.status === "ok") {
      this.view = result.data;
      this.loadFailure = null;
      await this.refreshToResolve();
    } else if (!this.view) {
      // Nothing to show yet: the shell explains and offers a way out.
      this.loadFailure = result.error;
    } else {
      this.report(result.error);
    }
  }

  /** Asks Rust what is left to resolve; the list stays until each case is settled. */
  async refreshToResolve(): Promise<void> {
    const result = await call(() => commands.incoherencesPlan());
    if (result.status === "ok") this.toResolve = result.data;
  }

  /** Runs a command that changes the plan and shows the view it returns.
   *  A refusal is reported, unless `asked` takes it as a question. */
  async modify(
    action: (year: number) => Promise<Result<YearView>>,
    asked: (error: AppError) => boolean = () => false,
  ): Promise<boolean> {
    this.saving = "saving";
    const result = await call(() => action(this.year));
    this.lastRefusal = null;
    if (result.status === "ok") {
      this.view = result.data;
      void this.refreshToResolve();
      this.saving = "saved";
      this.#retry = null;
      return true;
    }
    if (isFailureSaving(result.error)) {
      this.saving = "error";
      this.#retry = async () => {
        await this.modify(action);
      };
    } else {
      this.saving = "saved";
      this.lastRefusal = result.error;
      if (!asked(result.error)) this.report(result.error);
    }
    return false;
  }

  /** Places, moves or resizes a bed (S03 §15). Rust refuses a bed closer than
   *  the aisle with `aisle_too_narrow`: the bubble asks (S01 F-PLN-2), « Keep
   *  here » sends the same change with `keepClose`, « Go back » sends nothing. */
  async place(
    action: (year: number, keepClose: boolean) => Promise<Result<YearView>>,
  ): Promise<"ok" | "refusal" | "cancelled"> {
    let narrow = false;
    const ok = await this.modify(
      (year) => action(year, false),
      (error) => (narrow = error.code === "aisle_too_narrow"),
    );
    if (ok) return "ok";
    if (!narrow) return "refusal";
    this.aisleQuestion?.answer(false);
    const keep = await new Promise<boolean>((answer) => {
      this.aisleQuestion = { answer };
    });
    this.aisleQuestion = null;
    if (!keep) return "cancelled";
    return (await this.modify((year) => action(year, true))) ? "ok" : "refusal";
  }

  /** Undo / Redo of the editor (S01 F-PLN-6): Rust puts back the plan. With
   *  nothing left (a shortcut pressed anyway), nothing is said. */
  async undo(): Promise<void> {
    await this.modify((year) => commands.undo(year), (error) => error.code === "history_empty");
  }

  async redo(): Promise<void> {
    await this.modify((year) => commands.redo(year), (error) => error.code === "history_empty");
  }

  /** « Retry » of the save indicator. */
  async retry(): Promise<void> {
    await this.#retry?.();
  }

  report(error: AppError): void {
    this.message = { text: translateError(error), tone: "error" };
  }

  informer(text: string): void {
    this.message = { text, tone: "info" };
  }
}

/** Calls a command; a JavaScript-level failure (IPC, permission) becomes an
 *  error too, so the interface never stays stuck. */
export async function call<T>(call: () => Promise<Result<T>>): Promise<Result<T>> {
  try {
    return await call();
  } catch (e) {
    console.error(e);
    return { status: "error", error: { code: "unexpected" as AppError["code"], detail: String(e) } };
  }
}

export const app = new App();

/** A civil date `YYYY-MM-DD` or a UTC timestamp, as “2 November 2026”. */
export function dateLong(date: string): string {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) {
    return new Date(date).toLocaleDateString(localeTag(), { day: "numeric", month: "long", year: "numeric" });
  }
  return formatCivil(date, { day: "numeric", month: "long", year: "numeric" });
}

