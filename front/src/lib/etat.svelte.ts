import { commands, type AppError, type CreneauView, type SerieView, type SuggestionView, type YearView } from "./bindings";
import { estEchecEnregistrement, traduireErreur } from "./erreurs";


/** What a command resolves to (tauri-specta, ErrorHandlingMode::Result). */
export type Resultat<T> = { status: "ok"; data: T } | { status: "error"; error: AppError };

/** The save indicator of the header (F-CPT-2). */
export type Enregistrement = "enregistre" | "enregistrement" | "erreur";

export type Onglet = "potager" | "plan";

export type Message = { texte: string; ton: "info" | "erreur" };

/** What opens E07 (new serie and edition): from a planche (« Planter ici »),
 *  from a free slot of the timeline (with its suggestions), or from a bar. */
export type OuvertureSerie =
  | { type: "nouvelle"; plancheId: string; creneau?: CreneauView; suggestion?: SuggestionView }
  | { type: "edition"; serie: SerieView };

/**
 * State of the interface (S05 §4). It keeps the last year view returned by
 * Rust and replaces it entirely after each command. No business rule here.
 */
class Appli {
  annee = $state(new Date().getFullYear());
  onglet = $state<Onglet>("plan");
  reglagesOuverts = $state(false);
  /** E07, new serie and edition. */
  serieOuverte = $state<OuvertureSerie | null>(null);
  /** E09, catalogue. */
  catalogueOuvert = $state(false);
  /** The last refusal of a command that changes the plan (not a save failure),
   *  for the screens that word it themselves (a bar that goes back, S03 T12). */
  dernierRefus = $state<AppError | null>(null);
  vue = $state<YearView | null>(null);
  enregistrement = $state<Enregistrement>("enregistre");
  message = $state<Message | null>(null);
  #reessai: (() => Promise<void>) | null = null;

  /** Opens a year. */
  async charger(annee: number = this.annee): Promise<void> {
    this.annee = annee;
    const resultat = await appeler(() => commands.vueAnnee(annee));
    if (resultat.status === "ok") {
      this.vue = resultat.data;
    } else {
      this.signaler(resultat.error);
    }
  }

  /** Runs a command that changes the plan and shows the view it returns. */
  async modifier(action: (annee: number) => Promise<Resultat<YearView>>): Promise<boolean> {
    this.enregistrement = "enregistrement";
    const resultat = await appeler(() => action(this.annee));
    this.dernierRefus = null;
    if (resultat.status === "ok") {
      this.vue = resultat.data;
      this.enregistrement = "enregistre";
      this.#reessai = null;
      return true;
    }
    if (estEchecEnregistrement(resultat.error)) {
      this.enregistrement = "erreur";
      this.#reessai = async () => {
        await this.modifier(action);
      };
    } else {
      this.enregistrement = "enregistre";
      this.dernierRefus = resultat.error;
      this.signaler(resultat.error);
    }
    return false;
  }

  /** « Réessayer » of the save indicator. */
  async reessayer(): Promise<void> {
    await this.#reessai?.();
  }

  signaler(erreur: AppError): void {
    this.message = { texte: traduireErreur(erreur), ton: "erreur" };
  }

  informer(texte: string): void {
    this.message = { texte, ton: "info" };
  }
}

/** Calls a command; a JavaScript-level failure (IPC, permission) becomes an
 *  error too, so the interface never stays stuck. */
export async function appeler<T>(appel: () => Promise<Resultat<T>>): Promise<Resultat<T>> {
  try {
    return await appel();
  } catch (e) {
    console.error(e);
    return { status: "error", error: { code: "inattendue" as AppError["code"], detail: String(e) } };
  }
}

export const appli = new Appli();

/** A civil date `YYYY-MM-DD` or a UTC timestamp, as « 2 novembre 2026 ». */
export function dateLongue(date: string): string {
  const jour = /^\d{4}-\d{2}-\d{2}$/.test(date) ? new Date(`${date}T12:00:00`) : new Date(date);
  return jour.toLocaleDateString("fr-FR", { day: "numeric", month: "long", year: "numeric" });
}

