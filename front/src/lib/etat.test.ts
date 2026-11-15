import { expect, test } from "vitest";
import type { YearView } from "./bindings";
import { appli, type Resultat } from "./etat.svelte";

const vue = { year: 2026, version: 2 } as YearView;

test("a write failure keeps the change retryable, then « Réessayer » runs it again", async () => {
  let appels = 0;
  const action = async (): Promise<Resultat<YearView>> => {
    appels++;
    return appels === 1
      ? { status: "error", error: { code: "enregistrement", detail: "disk full" } }
      : { status: "ok", data: vue };
  };
  expect(await appli.modifier(action)).toBe(false);
  expect(appli.enregistrement).toBe("erreur");
  expect(appli.message).toBeNull();

  await appli.reessayer();
  expect(appels).toBe(2);
  expect(appli.enregistrement).toBe("enregistre");
  expect(appli.vue).toEqual(vue);
});

test("other refusals leave the indicator alone and show a message", async () => {
  const refus = async (): Promise<Resultat<YearView>> => ({
    status: "error",
    error: { code: "annee_passee", detail: "" },
  });
  expect(await appli.modifier(refus)).toBe(false);
  expect(appli.enregistrement).toBe("enregistre");
  expect(appli.message?.ton).toBe("erreur");
  expect(appli.message?.texte).toBe("Les saisons passées ne se modifient plus.");
});
