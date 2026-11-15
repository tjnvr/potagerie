import { expect, test } from "vitest";
import type { IncoherenceView } from "../bindings";
import { texteIncoherence } from "./incoherences";

const base: IncoherenceView = {
  code: "",
  annee: null,
  planche_nom: null,
  autre_planche_nom: null,
  culture_id: null,
  autre_culture_id: null,
  nombre: null,
};

test("too many planches names the year and the count", () => {
  const t = texteIncoherence({ ...base, code: "trop_de_planches", annee: 2027, nombre: 11 });
  expect(t).toBe("2027 compte 11 planches, pour 10 au plus.");
});

test("an orphan serie names its crop, with or without a known planche", () => {
  const avec = texteIncoherence({
    ...base,
    code: "serie_orpheline",
    annee: 2027,
    planche_nom: "Les tomates",
    culture_id: "tomate",
  });
  expect(avec).toBe("La série de Tomate n’a plus sa planche « Les tomates » en 2027.");
  const sans = texteIncoherence({ ...base, code: "serie_orpheline", culture_id: "laitue" });
  expect(sans).toBe("La série de Laitue est rattachée à une planche inconnue.");
});

test("an overlap names both crops and the planche", () => {
  const t = texteIncoherence({
    ...base,
    code: "chevauchement",
    planche_nom: "Racines",
    culture_id: "carotte",
    autre_culture_id: "poireau",
  });
  expect(t).toBe("Sur « Racines », Carotte et Poireau occupent les mêmes tronçons aux mêmes dates.");
});

test("a placement problem names both planches", () => {
  const t = texteIncoherence({
    ...base,
    code: "planche_chevauche",
    annee: 2026,
    planche_nom: "A",
    autre_planche_nom: "B",
  });
  expect(t).toBe("« A » chevauche « B » en 2026.");
});

test("an unknown code falls back to a generic line, an unknown crop keeps its id", () => {
  expect(texteIncoherence({ ...base, code: "nouveau" })).toBe("Une autre incohérence a été trouvée dans le fichier.");
  expect(texteIncoherence({ ...base, code: "culture_inconnue", culture_id: "mangue" })).toBe(
    "Le légume « mangue » n’est pas dans le catalogue.",
  );
});
