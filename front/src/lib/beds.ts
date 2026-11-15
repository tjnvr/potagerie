import { localeTag } from "./locale";
import type { BedView, PlotView, TypeBed } from "./bindings";
import { m } from "./paraglide/messages.js";

/** At most this many beds in a year (S01 F-PLN-1). Rust enforces it. */
export const MAX_BEDS = 10;

/** Size of a new bed of each type (S01 F-PLN-1), as Rust creates it:
 *  length along x, width along y. Only used to choose where to drop it. */
export const SIZE_BY_DEFAULT: Record<TypeBed, { length_cm: number; width_cm: number }> = {
  bed: { length_cm: 300, width_cm: 120 },
  square: { length_cm: 100, width_cm: 100 },
  mound: { length_cm: 300, width_cm: 120 },
  planter: { length_cm: 200, width_cm: 120 },
};

const NAMES_OF_TYPE: Record<TypeBed, () => string> = {
  bed: m.type_bed,
  square: m.type_square,
  mound: m.type_mound,
  planter: m.type_planter,
};

export function nameOfType(type: TypeBed): string {
  return NAMES_OF_TYPE[type]();
}

/** Default name of a new bed: its type and the first letter no other
 *  bed ends with, « Bed A », « Square B »… (S01 F-PLN-1). */
export function nameByDefault(type: TypeBed, beds: readonly BedView[]): string {
  let i = 0;
  while (beds.some((p) => p.name.endsWith(` ${letterOf(i)}`))) {
    i++;
  }
  return m.name_by_default({ type: nameOfType(type), letter: letterOf(i) });
}

/** A, B, … Z, AA, AB… */
export function letterOf(index: number): string {
  let n = index;
  let letters = "";
  do {
    letters = String.fromCharCode(65 + (n % 26)) + letters;
    n = Math.floor(n / 26) - 1;
  } while (n >= 0);
  return letters;
}

type Rect = { x: number; y: number; l: number; h: number };

function overlaps(a: Rect, b: Rect, marge: number): boolean {
  return (
    a.x < b.x + b.l + marge &&
    b.x < a.x + a.l + marge &&
    a.y < b.y + b.h + marge &&
    b.y < a.y + a.h + marge
  );
}

/**
 * Where the demo button « Add a bed » drops a new bed: the first
 * spot, on the case grid, read row by row, that keeps the aisle width from the
 * edges and from the other beds. `null` if there is no room.
 *
 * This only replaces the drag and drop of the editor (F-PLN-6) for now: Rust
 * remains the judge of what is valid.
 */
export function positionFree(
  plot: PlotView,
  beds: readonly BedView[],
  type: TypeBed,
): { x: number; y: number } | null {
  const { length_cm, width_cm } = SIZE_BY_DEFAULT[type];
  const step = plot.cell_cm;
  const marge = plot.aisle_cm;
  const start = Math.ceil(marge / step) * step;
  const others: Rect[] = beds.map((p) => ({
    x: p.x,
    y: p.y,
    l: p.length_cm,
    h: p.width_cm,
  }));
  for (let y = start; y + width_cm + marge <= plot.width_cm; y += step) {
    for (let x = start; x + length_cm + marge <= plot.length_cm; x += step) {
      const here = { x, y, l: length_cm, h: width_cm };
      if (!others.some((other) => overlaps(here, other, marge))) {
        return { x, y };
      }
    }
  }
  return null;
}

/** The years offered by the season selector: last year to two years ahead. */
export function yearsProposees(yearEnProgress: number): number[] {
  return [yearEnProgress - 1, yearEnProgress, yearEnProgress + 1, yearEnProgress + 2];
}

/** Centimetres as « 1,2 » meters, the way the plan reads. */
export function enMeters(cm: number): string {
  return (cm / 100).toLocaleString(localeTag(), { maximumFractionDigits: 2 });
}

/** Square centimetres as « 71,5 » m². */
export function enM2(cm2: number): string {
  return (cm2 / 10_000).toLocaleString(localeTag(), { maximumFractionDigits: 1 });
}

/** The balance of the plan (S01 F-PLN-5): surfaces and number of beds. */
export function summary(plot: PlotView, beds: readonly BedView[]) {
  const total = plot.length_cm * plot.width_cm;
  const cultivable = beds.reduce((sum, p) => sum + p.length_cm * p.width_cm, 0);
  return { total, cultivable, aisles: total - cultivable, count: beds.length };
}

/** A civil date `YYYY-MM-DD` as « 15 March », with the year when it is not `year`. */
export function dateShort(date: string, year?: number): string {
  const day = new Date(`${date}T12:00:00`);
  const options: Intl.DateTimeFormatOptions =
    day.getFullYear() === year ? { day: "numeric", month: "short" } : { day: "numeric", month: "short", year: "numeric" };
  return day.toLocaleDateString(localeTag(), options);
}

/** The French name of a crop of the catalogue (`catalogue()` sends none):
 *  the message `crop_<id>`, hyphens of the id written as underscores. */
export function nameCrop(id: string): string {
  const name = (m as unknown as Record<string, (() => string) | undefined>)[`crop_${id.replaceAll("-", "_")}`];
  return name ? name() : id;
}
