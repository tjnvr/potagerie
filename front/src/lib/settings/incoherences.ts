// message text of the incoherences listed for an imported plan (S03 §13).
// Rust decides what is incoherent; this only writes it.
import type { IncoherenceView } from "../bindings";
import { m } from "../paraglide/messages.js";
import { nameCrop } from "../beds";

/** « Tomato » for a catalogue crop, the raw id for an unknown one. */
function crop(id: string | null): string {
  if (!id) return "?";
  return nameCrop(id);
}

export function textIncoherence(i: IncoherenceView): string {
  const bed = i.bed_name ?? "?";
  const other = i.other_bed_name ?? "?";
  const year = i.year ?? 0;
  const count = i.count ?? 0;
  switch (i.code) {
    case "too_many_beds":
      return m.import_i_too_many_beds({ year, count });
    case "planting_orphan":
      return i.year === null
        ? m.import_i_planting_orphan_unknown({ crop: crop(i.crop_id) })
        : m.import_i_planting_orphan({ crop: crop(i.crop_id), bed, year });
    case "overlap":
      return m.import_i_overlap({
        bed,
        crop: crop(i.crop_id),
        other: crop(i.other_crop_id),
      });
    case "zone_outside_bed":
      return m.import_i_zone_outside_bed({ crop: crop(i.crop_id), bed, year, count });
    case "crop_unknown":
      return m.import_i_crop_unknown({ crop: i.crop_id ?? "?" });
    case "bed_outside_plot":
      return m.import_i_bed_outside_plot({ bed, year });
    case "bed_overlaps":
      return m.import_i_bed_overlaps({ bed, other, year });
    case "aisle_too_narrow":
      return m.import_i_aisle_too_narrow({ bed, other, year });
    case "width_invalid":
      return m.import_i_width_invalid();
    case "note_bed_unknown":
      return m.import_i_note_bed_unknown();
    case "note_invalid":
      return m.import_i_note_invalid();
    default:
      return m.import_i_other();
  }
}
