import { localeTag } from "./locale";
import { commands, type AnnotationView, type AppError, type SlotView, type IncoherenceView, type BedId, type PlantingView, type SuggestionView, type YearView } from "./bindings";
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
  view = $state<YearView | null>(null);
  saving = $state<Saving>("saved");
  message = $state<Message | null>(null);
  #retry: (() => Promise<void>) | null = null;

  /** Opens a year. */
  async load(year: number = this.year): Promise<void> {
    this.year = year;
    const result = await call(() => commands.viewYear(year));
    if (result.status === "ok") {
      this.view = result.data;
      await this.refreshToResolve();
    } else {
      this.report(result.error);
    }
  }

  /** Asks Rust what is left to resolve; the list stays until each case is settled. */
  async refreshToResolve(): Promise<void> {
    const result = await call(() => commands.incoherencesPlan());
    if (result.status === "ok") this.toResolve = result.data;
  }

  /** Runs a command that changes the plan and shows the view it returns. */
  async modify(action: (year: number) => Promise<Result<YearView>>): Promise<boolean> {
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
      this.report(result.error);
    }
    return false;
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
  const day = /^\d{4}-\d{2}-\d{2}$/.test(date) ? new Date(`${date}T12:00:00`) : new Date(date);
  return day.toLocaleDateString(localeTag(), { day: "numeric", month: "long", year: "numeric" });
}

