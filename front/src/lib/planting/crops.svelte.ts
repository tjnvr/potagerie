import { commands, type CropView } from "../bindings";
import { call } from "../state.svelte";
import { isOk } from "./call";

/** The crops of S02, loaded once from Rust (`catalogue()`): the form of a
 *  planting and the catalogue screen read them here. Names are not in the data:
 *  the interface translates `crop_<id>` (S05 §4). */
class Catalogue {
  crops = $state<CropView[]>([]);
  #request = false;

  async load(): Promise<void> {
    if (this.#request) return;
    this.#request = true;
    const result = await call(() => isOk(commands.catalogue()));
    if (result.status === "ok") this.crops = result.data;
    else this.#request = false;
  }

  crop(id: string): CropView | undefined {
    return this.crops.find((c) => c.id === id);
  }
}

export const catalogue = new Catalogue();
