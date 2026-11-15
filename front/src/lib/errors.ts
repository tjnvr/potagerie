import type { AppError, ErrorCode } from "./bindings";
import { m } from "./paraglide/messages.js";

/** One message per error code returned by Rust. Typed as a full record, so a
 *  new code in `bindings.ts` breaks `pnpm check` until it is translated. */
const messages: Record<ErrorCode, () => string> = {
  year_past: m.error_year_past,
  too_many_beds: m.error_too_many_beds,
  bed_unknown: m.error_bed_unknown,
  planting_unknown: m.error_planting_unknown,
  bed_existing: m.error_bed_existing,
  planting_existing: m.error_planting_existing,
  crop_unknown: m.error_crop_unknown,
  zone_invalid: m.error_zone_invalid,
  bed_absent: m.error_bed_absent,
  zone_outside_bed: m.error_zone_outside_bed,
  conflict: m.error_conflict,
  pot_days_required: m.error_pot_days_required,
  manual_end_invalid: m.error_manual_end_invalid,
  occupation_empty: m.error_occupation_empty,
  length_invalid: m.error_length_invalid,
  plot_invalid: m.error_plot_invalid,
  aisle_invalid: m.error_aisle_invalid,
  width_invalid: m.error_width_invalid,
  name_invalid: m.error_name_invalid,
  bed_outside_plot: m.error_bed_outside_plot,
  bed_overlaps: m.error_bed_overlaps,
  aisle_too_narrow: m.error_aisle_too_narrow,
  plot_absent: m.error_plot_absent,
  plot_existing: m.error_plot_existing,
  case_invalid: m.error_cell_invalid,
  annotation_unknown: m.error_annotation_unknown,
  text_invalid: m.note_error_text,
  annotation_dates_invalid: m.note_error_dates,
  saving: m.error_saving,
  version_stale: m.error_version_stale,
  base_too_recent: m.error_base_too_recent,
  file: m.error_file,
  import_invalid: m.error_import_invalid,
  import_too_recent: m.error_import_too_recent,
  svg_invalid: m.error_svg_invalid,
  rendering_impossible: m.error_rendering_impossible,
};

/** The French text of an error returned by a command. */
export function translateError(error: AppError): string {
  const message = messages[error.code];
  return message ? message() : m.error_unexpected();
}

/** Write failures: the change is kept on screen and can be retried (F-CPT-2). */
export function estFailureSaving(error: AppError): boolean {
  return error.code === "saving";
}
