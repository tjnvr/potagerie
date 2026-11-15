import { commands, type CultureView } from "../bindings";
import { appeler } from "../etat.svelte";
import { enOk } from "./appel";

/** The crops of S02, loaded once from Rust (`catalogue()`): the form of a
 *  serie and the catalogue screen read them here. Names are not in the data:
 *  the interface translates `culture_<id>` (S05 §4). */
class Catalogue {
  cultures = $state<CultureView[]>([]);
  #demande = false;

  async charger(): Promise<void> {
    if (this.#demande) return;
    this.#demande = true;
    const resultat = await appeler(() => enOk(commands.catalogue()));
    if (resultat.status === "ok") this.cultures = resultat.data;
    else this.#demande = false;
  }

  culture(id: string): CultureView | undefined {
    return this.cultures.find((c) => c.id === id);
  }
}

export const catalogue = new Catalogue();
