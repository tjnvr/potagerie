// Draft of a note or a period (S01 F-VIS-5), as the form edits it. The rules
// are Rust's (`plan::annotations`): this only builds the command input and
// decides when « Save » can be pressed, so a refusal from Rust stays
// the exception.
import type { AnnotationInput, AnnotationView, BedId, TypeAnnotation } from "../bindings";

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

/** The input of `create_annotation` and `modify_annotation`. */
export function versEntree(b: Draft): AnnotationInput {
  return {
    annotation_type: b.type,
    date_start: b.start,
    date_end: b.type === "period" ? b.end : null,
    bed_id: b.bed_id,
    text: b.text.trim(),
    hidden: b.hidden,
  };
}
