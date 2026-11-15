// Helpers of the catalogue screen (E09). Presentation only: names come from
// the messages, order and filters are the interface's.
import type { CropView } from "../bindings";
import { m } from "../paraglide/messages.js";

/** « Solanaceae » → `solanaceae`: the key of the family in the messages. */
export function keyFamily(family: string): string {
  return family
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase();
}


/** The translated name of a botanical family; a family the messages do not
 *  know yet shows as the catalogue wrote it. */
export function nameFamily(family: string): string {
  const name = (m as unknown as Record<string, (() => string) | undefined>)[`family_${keyFamily(family)}`];
  return name ? name() : family;
}

/** The crops sorted by their translated name. */
export function tri(crops: readonly CropView[], name: (id: string) => string): CropView[] {
  return [...crops].sort((a, b) => name(a.id).localeCompare(name(b.id), "fr"));
}
