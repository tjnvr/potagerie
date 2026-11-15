// Isometric projection of P01 and D01 (gen.py, Iso): a 50 cm tile is a
// diamond 48 px wide and 24 px high. Terrain coordinates are in centimetres;
// the scene works in tiles ("unités") and in pixels of the SVG view box.

export const UNITE_CM = 50;
export const TW = 48;
export const TH = 24;

/** Depth of the soil under the grass, and of the grass layer (D01). */
export const EPAISSEUR_TERRE = 30;
export const EPAISSEUR_HERBE = 8;
/** Room left above the back row for the tallest plant (tomato, 48 px). */
export const MARGE_HAUT = 72;
export const MARGE_COTE = 28;

export type Quart = 0 | 1 | 2 | 3;
export type Point = [number, number];

/** Terrain position in cm to tiles. */
export function enUnites(cm: number): number {
  return cm / UNITE_CM;
}

/** Tile coordinates and height `z` (px, up) to view box pixels. */
export function projeter(gx: number, gy: number, z = 0): Point {
  return [((gx - gy) * TW) / 2, ((gx + gy) * TH) / 2 - z];
}

/** Size of the terrain, in tiles, once turned by `quart` quarter turns. */
export function dimensions(quart: Quart, l: number, h: number): { l: number; h: number } {
  return quart % 2 === 0 ? { l, h } : { l: h, h: l };
}

/** A point of the terrain (tiles) turned by `quart` quarter turns, clockwise. */
export function pivoter(p: Point, quart: Quart, l: number, h: number): Point {
  const [x, y] = p;
  switch (quart) {
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

/** A rectangle of the terrain (tiles) turned with the terrain. */
export function pivoterRect(r: Rect, quart: Quart, l: number, h: number): Rect {
  const a = pivoter([r.x, r.y], quart, l, h);
  const b = pivoter([r.x + r.l, r.y + r.h], quart, l, h);
  return {
    x: Math.min(a[0], b[0]),
    y: Math.min(a[1], b[1]),
    l: Math.abs(a[0] - b[0]),
    h: Math.abs(a[1] - b[1]),
  };
}

/** The SVG view box `[x, y, largeur, hauteur]` that holds the whole diorama. */
export function cadre(l: number, h: number): [number, number, number, number] {
  const gauche = projeter(0, h)[0] - MARGE_COTE;
  const droite = projeter(l, 0)[0] + MARGE_COTE;
  const haut = projeter(0, 0)[1] - MARGE_HAUT;
  const bas = projeter(l, h, -EPAISSEUR_TERRE)[1] + 12;
  return [gauche, haut, droite - gauche, bas - haut];
}

/** The rank of a point for painting: further from the viewer first. */
export function profondeur(gx: number, gy: number): number {
  return gx + gy;
}
