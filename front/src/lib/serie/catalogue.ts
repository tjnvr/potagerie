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


/** The translated name of a botanical family; a family the messages do not
 *  know yet shows as the catalogue wrote it. */
export function nomFamille(famille: string): string {
  const nom = (m as unknown as Record<string, (() => string) | undefined>)[`famille_${cleFamille(famille)}`];
  return nom ? nom() : famille;
}

/** The crops sorted by their translated name. */
export function tri(cultures: readonly CultureView[], nom: (id: string) => string): CultureView[] {
  return [...cultures].sort((a, b) => nom(a.id).localeCompare(nom(b.id), "fr"));
}
