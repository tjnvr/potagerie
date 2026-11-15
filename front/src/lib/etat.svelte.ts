import { commands, type AppError, type YearView } from "./bindings";
import { estEchecEnregistrement, traduireErreur } from "./erreurs";


/** What a command resolves to (tauri-specta, ErrorHandlingMode::Result). */
export type Resultat<T> = { status: "ok"; data: T } | { status: "error"; error: AppError };

/** The save indicator of the header (F-CPT-2). */
export type Enregistrement = "enregistre" | "enregistrement" | "erreur";

export type Onglet = "potager" | "plan";

export type Message = { texte: string; ton: "info" | "erreur" };

/**
 * State of the interface (S05 §4). It keeps the last year view returned by
 * Rust and replaces it entirely after each command. No business rule here.
 */
class Appli {
  annee = $state(new Date().getFullYear());
  onglet = $state<Onglet>("plan");
  reglagesOuverts = $state(false);
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

