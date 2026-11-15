// Draft of a note or a period (S01 F-VIS-5), as the form edits it. The rules
// are Rust's (`plan::annotations`): this only builds the command input and
// decides when « Enregistrer » can be pressed, so a refusal from Rust stays
// the exception.
import type { AnnotationInput, AnnotationView, PlancheId, TypeAnnotation } from "../bindings";

/** Longest text, as in Rust (`MAX_ANNOTATION_CHARS`); only used for `maxlength`. */
export const TEXTE_MAX = 80;

export type Brouillon = {
  type: TypeAnnotation;
  texte: string;
  debut: string;
  /** Only read for a period. */
  fin: string;
  /** `null`: the whole potager. */
  planche_id: PlancheId | null;
  masquee: boolean;
};

/** A new note on `date`, on a planche or on the whole potager. */
export function brouillonVide(date: string, planche_id: PlancheId | null = null): Brouillon {
  return { type: "note", texte: "", debut: date, fin: date, planche_id, masquee: false };
}

/** The draft of an existing annotation. */
export function brouillonDe(a: AnnotationView): Brouillon {
  return {
    type: a.annotation_type,
    texte: a.texte,
    debut: a.date_debut,
    fin: a.date_fin ?? a.date_debut,
    planche_id: a.planche_id,
    masquee: a.masquee,
  };
}

/** Whether the draft can be sent: some text, a start, and an end for a period. */
export function envoyable(b: Brouillon): boolean {
  if (b.texte.trim() === "" || b.debut === "") return false;
  return b.type === "note" || (b.fin !== "" && b.fin >= b.debut);
}

/** The input of `creer_annotation` and `modifier_annotation`. */
export function versEntree(b: Brouillon): AnnotationInput {
  return {
    annotation_type: b.type,
    date_debut: b.debut,
    date_fin: b.type === "period" ? b.fin : null,
    planche_id: b.planche_id,
    texte: b.texte.trim(),
    masquee: b.masquee,
  };
}
