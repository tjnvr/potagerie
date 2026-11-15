import type { Resultat } from "../etat.svelte";

/** A command that cannot be refused (`catalogue`, `cultures_de_saison`) as a
 *  result, to go through `appeler` like the others. */
export function enOk<T>(valeur: Promise<T>): Promise<Resultat<T>> {
  return valeur.then((data) => ({ status: "ok" as const, data }));
}
