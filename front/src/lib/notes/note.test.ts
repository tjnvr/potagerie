import { expect, test } from "vitest";
import type { AnnotationView } from "../bindings";
import { draftOf, draftEmpty, isMove, movedInput, sendable, toInput } from "./note";

const BED = "0190c5a0-0000-7000-8000-000000000001";

test("a new draft is an empty note on the given day", () => {
  const b = draftEmpty("2026-12-01", BED);
  expect(b).toMatchObject({ type: "note", text: "", start: "2026-12-01", bed_id: BED, hidden: false });
  expect(sendable(b)).toBe(false);
});

test("a note is sent with trimmed text and no end date", () => {
  const b = { ...draftEmpty("2026-12-01"), text: "  Mulch the tomatoes ", end: "2030-01-01" };
  expect(sendable(b)).toBe(true);
  expect(toInput(b)).toEqual({
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
  expect(toInput(b).date_end).toBe("2027-08-21");
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
  expect(toInput(b)).toMatchObject({ hidden: true, bed_id: BED });
});

const OTHER = "0190c5a0-0000-7000-8000-000000000002";
const note: AnnotationView = { id: "n", annotation_type: "note", date_start: "2026-12-30", date_end: null, bed_id: BED, text: "Mulch", hidden: false };
const period: AnnotationView = { id: "p", annotation_type: "period", date_start: "2026-08-01", date_end: "2026-08-21", bed_id: BED, text: "Away", hidden: true };

test("a dragged note changes its date, across the year, and the bed of the row it is dropped on", () => {
  expect(movedInput(note, 3, OTHER)).toEqual({
    annotation_type: "note",
    date_start: "2027-01-02",
    date_end: null,
    bed_id: OTHER,
    text: "Mulch",
    hidden: false,
  });
});

test("a note dropped outside any row keeps its bed", () => {
  expect(movedInput(note, -1, null)).toMatchObject({ date_start: "2026-12-29", bed_id: BED });
});

test("a dragged period shifts both dates and keeps its bed and its hidden state", () => {
  expect(movedInput(period, -10, OTHER)).toEqual({
    annotation_type: "period",
    date_start: "2026-07-22",
    date_end: "2026-08-11",
    bed_id: BED,
    text: "Away",
    hidden: true,
  });
});

test("a note of the whole garden stays on the whole garden", () => {
  expect(movedInput({ ...note, bed_id: null }, 7, OTHER)).toMatchObject({ date_start: "2027-01-06", bed_id: null });
});

test("a drag that changes nothing is not a move", () => {
  expect(isMove(note, 0, BED)).toBe(false);
  expect(isMove(note, 0, null)).toBe(false);
  expect(isMove(note, 0, OTHER)).toBe(true);
  expect(isMove(period, 0, OTHER)).toBe(false);
  expect(isMove(period, 1, BED)).toBe(true);
});
