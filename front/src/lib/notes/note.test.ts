import { expect, test } from "vitest";
import type { AnnotationView } from "../bindings";
import { brouillonDe, brouillonVide, envoyable, versEntree } from "./note";

const PLANCHE = "0190c5a0-0000-7000-8000-000000000001";

test("a new draft is an empty note on the given day", () => {
  const b = brouillonVide("2026-12-01", PLANCHE);
  expect(b).toMatchObject({ type: "note", texte: "", debut: "2026-12-01", planche_id: PLANCHE, masquee: false });
  expect(envoyable(b)).toBe(false);
});

test("a note is sent with trimmed text and no end date", () => {
  const b = { ...brouillonVide("2026-12-01"), texte: "  Pailler les tomates ", fin: "2030-01-01" };
  expect(envoyable(b)).toBe(true);
  expect(versEntree(b)).toEqual({
    annotation_type: "note",
    date_debut: "2026-12-01",
    date_fin: null,
    planche_id: null,
    texte: "Pailler les tomates",
    masquee: false,
  });
});

test("a period needs an end on or after its start", () => {
  const b = { ...brouillonVide("2027-08-01"), type: "period" as const, texte: "Vacances", fin: "2027-08-21" };
  expect(envoyable(b)).toBe(true);
  expect(versEntree(b).date_fin).toBe("2027-08-21");
  expect(envoyable({ ...b, fin: "2027-08-01" })).toBe(true);
  expect(envoyable({ ...b, fin: "2027-07-31" })).toBe(false);
  expect(envoyable({ ...b, fin: "" })).toBe(false);
});

test("a blank text cannot be sent", () => {
  expect(envoyable({ ...brouillonVide("2026-12-01"), texte: "   " })).toBe(false);
});

test("the draft of an existing annotation keeps its hidden flag and planche", () => {
  const a: AnnotationView = {
    id: "0190c5a0-0000-7000-8000-000000000002",
    annotation_type: "period",
    date_debut: "2027-08-01",
    date_fin: "2027-08-21",
    planche_id: PLANCHE,
    texte: "Absent",
    masquee: true,
  };
  const b = brouillonDe(a);
  expect(b).toEqual({ type: "period", texte: "Absent", debut: "2027-08-01", fin: "2027-08-21", planche_id: PLANCHE, masquee: true });
  expect(versEntree(b)).toMatchObject({ masquee: true, planche_id: PLANCHE });
});
