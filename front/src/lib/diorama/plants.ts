// Where the plants of a planting stand on its bed (S03 §6): the rows and the
// plants per row come from the view (Rust), the zone along the length from its
// tronçons. Rows run along the length of the bed, evenly across its width;
// the plants of a row are evenly spread along the zone.

import { enUnites } from "./projection";

export type GeoBed = { x: number; y: number; length_cm: number; width_cm: number };
export type ZonePlanting = {
  segment_start: number;
  segment_end: number;
  rows: number;
  plants_by_row: number;
};

/** A plant foot, in tiles of the plot. */
export type Spacing = { gx: number; gy: number };

/** Two drawn plants are never closer than this, in cm: closer, they only
 *  pile up into a blob and cost frames (carrots 4 cm apart). */
export const GAP_DRAWING_MIN_CM = 15;

let multiplierBench = 1;

/** Dev only (`?density=4`): draws N times as many plants, to stress the
 *  renderer with the bench of P01. Never set in a build. */
export function setDensityBench(n: number): void {
  multiplierBench = Math.max(1, Math.round(Math.sqrt(n)));
}

/** Rows and plants per row actually drawn: those of the view, thinned where
 *  the drawings would be closer than `GAP_DRAWING_MIN_CM`. */
export function quantitiesDrawn(
  bed: GeoBed,
  planting: ZonePlanting,
  cellCm: number,
): { rows: number; plants: number } {
  const lengthCm = (planting.segment_end - planting.segment_start + 1) * cellCm;
  return {
    rows:
      multiplierBench *
      Math.min(Math.max(1, planting.rows), Math.max(1, Math.floor(bed.width_cm / GAP_DRAWING_MIN_CM))),
    plants:
      multiplierBench *
      Math.min(Math.max(1, planting.plants_by_row), Math.max(1, Math.floor(lengthCm / GAP_DRAWING_MIN_CM))),
  };
}

/** The feet of the plants of a planting, row by row, in tiles of the plot. The
 *  caller sorts them by depth once the view is turned. */
export function positionsPlants(bed: GeoBed, planting: ZonePlanting, cellCm: number): Spacing[] {
  const { rows, plants } = quantitiesDrawn(bed, planting, cellCm);
  const startCm = bed.x + (planting.segment_start - 1) * cellCm;
  const lengthCm = (planting.segment_end - planting.segment_start + 1) * cellCm;
  const result: Spacing[] = [];
  for (let r = 0; r < rows; r++) {
    const gy = enUnites(bed.y + ((r + 0.5) * bed.width_cm) / rows);
    for (let p = 0; p < plants; p++) {
      result.push({ gx: enUnites(startCm + ((p + 0.5) * lengthCm) / plants), gy });
    }
  }
  return result;
}

/** Gap between two drawn plants along the row and between two rows, in cm. */
export function gaps(bed: GeoBed, planting: ZonePlanting, cellCm: number): { spacing: number; row: number } {
  const lengthCm = (planting.segment_end - planting.segment_start + 1) * cellCm;
  const { rows, plants } = quantitiesDrawn(bed, planting, cellCm);
  return { spacing: lengthCm / plants, row: bed.width_cm / rows };
}

const GAPS_OF_REFERENCE_CM = 35;
const SCALE_MIN = 0.45;
const SCALE_MAX = 1.1;

/**
 * Size of the drawing of a plant, from the room it has: the mean of the two
 * gaps against a reference one. Carrots 4 cm apart are drawn small, a
 * zucchini alone on a metre at full size.
 */
export function scalePlant(bed: GeoBed, planting: ZonePlanting, cellCm: number): number {
  const { spacing, row } = gaps(bed, planting, cellCm);
  const k = Math.sqrt(spacing * row) / GAPS_OF_REFERENCE_CM;
  return Math.min(SCALE_MAX, Math.max(SCALE_MIN, k));
}
