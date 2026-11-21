// Draft of a note or a period (S01 F-VIS-5), as the form edits it. The rules
// are Rust's (`plan::annotations`): this only builds the command input and
// decides when « Save » can be pressed, so a refusal from Rust stays
// the exception.
import type { AnnotationInput, AnnotationView, BedId, TypeAnnotation } from "../bindings";
import { shiftDate } from "../dates";

/** Longest text, as in Rust (`MAX_ANNOTATION_CHARS`); only used for `maxlength`. */
export const TEXT_MAX = 80;

export type Draft = {
  type: TypeAnnotation;
  text: string;
  start: string;
  /** Only read for a period. */
  end: string;
  /** `null`: the whole garden. */
  bed_id: BedId | null;
  hidden: boolean;
};

/** A new note on `date`, on a bed or on the whole garden. */
export function draftEmpty(date: string, bed_id: BedId | null = null): Draft {
  return { type: "note", text: "", start: date, end: date, bed_id, hidden: false };
}

/** The draft of an existing annotation. */
export function draftOf(a: AnnotationView): Draft {
  return {
    type: a.annotation_type,
    text: a.text,
    start: a.date_start,
    end: a.date_end ?? a.date_start,
    bed_id: a.bed_id,
    hidden: a.hidden,
  };
}

/** Whether the draft can be sent: some text, a start, and an end for a period. */
export function sendable(b: Draft): boolean {
  if (b.text.trim() === "" || b.start === "") return false;
  return b.type === "note" || (b.end !== "" && b.end >= b.start);
}

/** The bed a dragged annotation lands on: a note of a bed follows the row it
 *  is dropped on (`null`: no row, it keeps its bed); a period, or a note of
 *  the whole garden, keeps where it was. */
function bedDropped(a: AnnotationView, row: BedId | null): BedId | null {
  return a.annotation_type === "note" && a.bed_id !== null && row !== null ? row : a.bed_id;
}

/** Whether a drag of `days` dropped on `row` changes the annotation at all. */
export function isMove(a: AnnotationView, days: number, row: BedId | null): boolean {
  return days !== 0 || bedDropped(a, row) !== a.bed_id;
}

/** The input of `modify_annotation` for an annotation dragged by `days`
 *  (both dates of a period move together) and dropped on `row`. */
export function movedInput(a: AnnotationView, days: number, row: BedId | null): AnnotationInput {
  return {
    annotation_type: a.annotation_type,
    date_start: shiftDate(a.date_start, days),
    date_end: a.annotation_type === "period" && a.date_end ? shiftDate(a.date_end, days) : null,
    bed_id: bedDropped(a, row),
    text: a.text,
    hidden: a.hidden,
  };
}

/** The input of `create_annotation` and `modify_annotation`. */
export function toInput(b: Draft): AnnotationInput {
  return {
    annotation_type: b.type,
    date_start: b.start,
    date_end: b.type === "period" ? b.end : null,
    bed_id: b.bed_id,
    text: b.text.trim(),
    hidden: b.hidden,
  };
}
