import { expect, test } from "vitest";
import { estEchecEnregistrement, traduireErreur } from "./erreurs";

test("an error code is translated, the technical detail is not shown", () => {
  const texte = traduireErreur({ code: "annee_passee", detail: "year 2026 is in the past" });
  expect(texte).toBe("Les saisons passées ne se modifient plus.");
});

test("an unknown code falls back to a generic message", () => {
  // A code from a newer Rust than these bindings.
  const texte = traduireErreur({ code: "nouveau_code" as never, detail: "" });
  expect(texte).toBe("Une erreur inattendue est survenue.");
});

test("only write failures can be retried", () => {
  expect(estEchecEnregistrement({ code: "enregistrement", detail: "" })).toBe(true);
  expect(estEchecEnregistrement({ code: "conflict", detail: "" })).toBe(false);
});
