// Isometric projection of P01 and D01 (gen.py, Iso): a 50 cm tile is a
// diamond 48 px wide and 24 px high. Plot coordinates are in centimetres;
// the scene works in tiles ("units") and in pixels of the SVG view box.

export const UNIT_CM = 50;
export const TW = 48;
export const TH = 24;

/** Depth of the soil under the grass, and of the grass layer (D01). */
export const THICKNESS_GROUND = 30;
export const THICKNESS_GRASS = 8;
/** Room left above the back row for the tallest plant (tomato, 48 px). */
export const MARGE_TOP = 72;
export const MARGE_COTE = 28;

export type Quarter = 0 | 1 | 2 | 3;
export type Point = [number, number];

/** Plot position in cm to tiles. */
export function enUnites(cm: number): number {
  return cm / UNIT_CM;
}

/** Tile coordinates and height `z` (px, up) to view box pixels. */
export function project(gx: number, gy: number, z = 0): Point {
  return [((gx - gy) * TW) / 2, ((gx + gy) * TH) / 2 - z];
}

/** Size of the plot, in tiles, once turned by `quarter` quarter turns. */
export function dimensions(quarter: Quarter, l: number, h: number): { l: number; h: number } {
  return quarter % 2 === 0 ? { l, h } : { l: h, h: l };
}

/** A point of the plot (tiles) turned by `quarter` quarter turns, clockwise. */
export function rotate(p: Point, quarter: Quarter, l: number, h: number): Point {
  const [x, y] = p;
  switch (quarter) {
    case 0:
      return [x, y];
    case 1:
      return [h - y, x];
    case 2:
      return [l - x, h - y];
    case 3:
      return [y, l - x];
  }
}

export type Rect = { x: number; y: number; l: number; h: number };

/** A rectangle of the plot (tiles) turned with the plot. */
export function pivoterRect(r: Rect, quarter: Quarter, l: number, h: number): Rect {
  const a = rotate([r.x, r.y], quarter, l, h);
  const b = rotate([r.x + r.l, r.y + r.h], quarter, l, h);
  return {
    x: Math.min(a[0], b[0]),
    y: Math.min(a[1], b[1]),
    l: Math.abs(a[0] - b[0]),
    h: Math.abs(a[1] - b[1]),
  };
}

/** The SVG view box `[x, y, width, height]` that holds the whole diorama. */
export function cadre(l: number, h: number): [number, number, number, number] {
  const gauche = project(0, h)[0] - MARGE_COTE;
  const right = project(l, 0)[0] + MARGE_COTE;
  const top = project(0, 0)[1] - MARGE_TOP;
  const low = project(l, h, -THICKNESS_GROUND)[1] + 12;
  return [gauche, top, right - gauche, low - top];
}

/** The rank of a point for painting: further from the viewer first. */
export function depth(gx: number, gy: number): number {
  return gx + gy;
}
