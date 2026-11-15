import { expect, test } from "vitest";
import type { AnnotationView } from "../bindings";
import { draftOf, draftEmpty, sendable, versEntree } from "./note";

const BED = "0190c5a0-0000-7000-8000-000000000001";

test("a new draft is an empty note on the given day", () => {
  const b = draftEmpty("2026-12-01", BED);
  expect(b).toMatchObject({ type: "note", text: "", start: "2026-12-01", bed_id: BED, hidden: false });
  expect(sendable(b)).toBe(false);
});

test("a note is sent with trimmed text and no end date", () => {
  const b = { ...draftEmpty("2026-12-01"), text: "  Mulch the tomatoes ", end: "2030-01-01" };
  expect(sendable(b)).toBe(true);
  expect(versEntree(b)).toEqual({
    annotation_type: "note",
    date_start: "2026-12-01",
    date_end: null,
    bed_id: null,
    text: "Mulch the tomatoes",
    hidden: false,
  });
});

test("a period needs an end on or after its start", () => {
  const b = { ...draftEmpty("2027-08-01"), type: "period" as const, text: "Holidays", end: "2027-08-21" };
  expect(sendable(b)).toBe(true);
  expect(versEntree(b).date_end).toBe("2027-08-21");
  expect(sendable({ ...b, end: "2027-08-01" })).toBe(true);
  expect(sendable({ ...b, end: "2027-07-31" })).toBe(false);
  expect(sendable({ ...b, end: "" })).toBe(false);
});

test("a blank text cannot be sent", () => {
  expect(sendable({ ...draftEmpty("2026-12-01"), text: "   " })).toBe(false);
});

test("the draft of an existing annotation keeps its hidden flag and bed", () => {
  const a: AnnotationView = {
    id: "0190c5a0-0000-7000-8000-000000000002",
    annotation_type: "period",
    date_start: "2027-08-01",
    date_end: "2027-08-21",
    bed_id: BED,
    text: "Absent",
    hidden: true,
  };
  const b = draftOf(a);
  expect(b).toEqual({ type: "period", text: "Absent", start: "2027-08-01", end: "2027-08-21", bed_id: BED, hidden: true });
  expect(versEntree(b)).toMatchObject({ hidden: true, bed_id: BED });
});
