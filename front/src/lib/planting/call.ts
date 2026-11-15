import type { Result } from "../state.svelte";

/** A command that cannot be refused (`catalogue`, `crops_of_season`) as a
 *  result, to go through `call` like the others. */
export function isOk<T>(value: Promise<T>): Promise<Result<T>> {
  return value.then((data) => ({ status: "ok" as const, data }));
}
