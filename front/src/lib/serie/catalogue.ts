// Helpers of the catalogue screen (E09). Presentation only: names come from
// the messages, order and filters are the interface's.
import type { CultureView } from "../bindings";
import { m } from "../paraglide/messages.js";

/** « Solanacées » → `solanacees`: the key of the family in the messages. */
export function cleFamille(famille: string): string {
  return famille
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase();
}

const FAMILLES: Record<string, () => string> = {
  solanacees: m.famille_solanacees,
  asteracees: m.famille_asteracees,
  apiacees: m.famille_apiacees,
  fabacees: m.famille_fabacees,
  cucurbitacees: m.famille_cucurbitacees,
  amaryllidacees: m.famille_amaryllidacees,
};

/** The translated name of a botanical family; a family the messages do not
 *  know yet shows as the catalogue wrote it. */
export function nomFamille(famille: string): string {
  return FAMILLES[cleFamille(famille)]?.() ?? famille;
}

/** The crops sorted by their translated name. */
export function tri(cultures: readonly CultureView[], nom: (id: string) => string): CultureView[] {
  return [...cultures].sort((a, b) => nom(a.id).localeCompare(nom(b.id), "fr"));
}
