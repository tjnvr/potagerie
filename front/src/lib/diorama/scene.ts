// The geometry of the diorama: from the year view (beds, plantings) and the
// turn of the view, everything the SVG needs, in view box pixels. Pure, so it
// is testable and the component only draws.

import type { BedView, PlantingView, PlotView, TypeBed, YearView } from "../bindings";
import { chronology, type Chronology } from "./animation";
import { scalePlant, positionsPlants } from "./plants";
import {
  cadre,
  dimensions,
  inUnits,
  rotate,
  pivoterRect,
  depth,
  project,
  type Point,
  type Quarter,
  type Rect,
} from "./projection";

/** Height of the top of each kind of bed, and of the soil on it (D01). */
export const HAUTEURS: Record<TypeBed, { bord: number; ground: number }> = {
  bed: { bord: 7, ground: 7 },
  square: { bord: 9, ground: 8.5 },
  mound: { bord: 12, ground: 12 },
  planter: { bord: 14, ground: 13.5 },
};

export type Face = { points: string; class: string };
export type Sillon = { a: Point; b: Point };

export type SpacingScene = { x: number; y: number };

export type PlantingScene = {
  id: string;
  crop: string;
  chrono: Chronology;
  scale: number;
  /** Feet in view box pixels, back to front. */
  plants: SpacingScene[];
  /** Middle of the zone, for the label of a planting sown under cover. */
  centre: SpacingScene;
};

export type BedScene = {
  id: string;
  name: string;
  type: TypeBed;
  faces: Face[];
  sillons: Sillon[];
  contour: string;
  /** Top of the soil, where the plants stand. */
  ground: number;
  plantings: PlantingScene[];
  /** Left corner of the top, where the name sits. */
  coinName: Point;
  /** Middle of the top, for the free sign. */
  centre: Point;
  depth: number;
};

export type Tuft = { x: number; y: number; flower: boolean; tint: number };

export type Scene = {
  l: number;
  h: number;
  cadre: [number, number, number, number];
  socle: Face[];
  tufts: Tuft[];
  beds: BedScene[];
};

function pts(...ps: Point[]): string {
  return ps.map(([x, y]) => `${x.toFixed(1)},${y.toFixed(1)}`).join(" ");
}

/** The three visible faces of a box: front-left, right, top (gen.py, box()). */
function box(r: Rect, z0: number, z1: number, classes: [string, string, string]): Face[] {
  const P = project;
  const { x, y, l, h } = r;
  return [
    { points: pts(P(x, y + h, z1), P(x + l, y + h, z1), P(x + l, y + h, z0), P(x, y + h, z0)), class: classes[0] },
    { points: pts(P(x + l, y, z1), P(x + l, y + h, z1), P(x + l, y + h, z0), P(x + l, y, z0)), class: classes[1] },
    { points: pts(P(x, y, z1), P(x + l, y, z1), P(x + l, y + h, z1), P(x, y + h, z1)), class: classes[2] },
  ];
}

function top(r: Rect, z: number, faceClass: string): Face {
  const P = project;
  return { points: pts(P(r.x, r.y, z), P(r.x + r.l, r.y, z), P(r.x + r.l, r.y + r.h, z), P(r.x, r.y + r.h, z)), class: faceClass };
}

function shrink(r: Rect, e: number): Rect {
  return { x: r.x + e, y: r.y + e, l: r.l - 2 * e, h: r.h - 2 * e };
}

/** The faces and furrows of a bed of each type, on its rectangle `r`. */
function shapes(type: TypeBed, r: Rect): { faces: Face[]; sillons: Sillon[] } {
  const { bord, ground } = HAUTEURS[type];
  const P = project;
  switch (type) {
    case "bed": {
      const sillons: Sillon[] = [];
      for (let j = 1; j < r.h * 2; j++) {
        sillons.push({ a: P(r.x + 0.15, r.y + j / 2, bord), b: P(r.x + r.l - 0.15, r.y + j / 2, bord) });
      }
      return { faces: box(r, 0, bord, ["lit-gauche", "lit-right", "lit-top"]), sillons };
    }
    case "mound": {
      // A mound: the top is smaller than the foot, the sides slope.
      const i = Math.min(0.35, r.l / 4, r.h / 4);
      const t = shrink(r, i);
      return {
        faces: [
          {
            points: pts(P(r.x, r.y + r.h, 0), P(r.x + r.l, r.y + r.h, 0), P(t.x + t.l, t.y + t.h, bord), P(t.x, t.y + t.h, bord)),
            class: "lit-gauche",
          },
          {
            points: pts(P(r.x + r.l, r.y, 0), P(r.x + r.l, r.y + r.h, 0), P(t.x + t.l, t.y + t.h, bord), P(t.x + t.l, t.y, bord)),
            class: "lit-right",
          },
          top(t, bord, "lit-top"),
        ],
        sillons: [],
      };
    }
    case "square":
    case "planter": {
      const faces = box(r, 0, bord, ["bois-gauche", "bois-right", "bois-top"]);
      faces.push(top(shrink(r, type === "planter" ? 0.18 : 0.14), ground, "lit-top"));
      return { faces, sillons: [] };
    }
  }
}

/** Small deterministic generator, so the tufts stay where they are. */
function random(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function tufts(l: number, h: number, beds: Rect[], quarter: Quarter): Tuft[] {
  const rng = random(7);
  const result: Tuft[] = [];
  const count = Math.min(90, Math.round((l * h) / 3));
  for (let i = 0; i < count * 3 && result.length < count; i++) {
    const gx = 0.4 + rng() * (l - 0.8);
    const gy = 0.4 + rng() * (h - 0.8);
    const flower = rng() < 0.25;
    const tint = Math.floor(rng() * 3);
    if (beds.some((r) => gx > r.x - 0.3 && gx < r.x + r.l + 0.3 && gy > r.y - 0.3 && gy < r.y + r.h + 0.3)) continue;
    const [x, y] = project(...rotate([gx, gy], quarter, l, h));
    result.push({ x, y, flower, tint });
  }
  return result;
}

function rectOf(p: BedView): Rect {
  return { x: inUnits(p.x), y: inUnits(p.y), l: inUnits(p.length_cm), h: inUnits(p.width_cm) };
}

function plantingScene(p: BedView, s: PlantingView, plot: PlotView, quarter: Quarter, lt: number, ht: number, ground: number): PlantingScene {
  const brut = positionsPlants(p, s, plot.cell_cm);
  const plants = brut
    .map((b) => rotate([b.gx, b.gy], quarter, lt, ht))
    .sort((a, b) => depth(a[0], a[1]) - depth(b[0], b[1]))
    .map(([gx, gy]) => {
      const [x, y] = project(gx, gy, ground);
      return { x, y };
    });
  const start = p.x + (s.segment_start - 1) * plot.cell_cm;
  const length = (s.segment_end - s.segment_start + 1) * plot.cell_cm;
  const [cx, cy] = rotate([inUnits(start + length / 2), inUnits(p.y + p.width_cm / 2)], quarter, lt, ht);
  const [x, y] = project(cx, cy, ground);
  return {
    id: s.id,
    crop: s.crop_id,
    chrono: chronology(s.appearance_changes),
    scale: scalePlant(p, s, plot.cell_cm),
    plants,
    centre: { x, y },
  };
}

/** The whole scene for a year view whose plot exists, turned by `quarter`. */
export function buildScene(view: YearView, quarter: Quarter): Scene | null {
  const plot = view.plot;
  if (!plot) return null;
  const l0 = inUnits(plot.length_cm);
  const h0 = inUnits(plot.width_cm);
  const { l: lt, h: ht } = dimensions(quarter, l0, h0);
  const socle: Face[] = [
    ...box({ x: 0, y: 0, l: lt, h: ht }, -30, -8, ["ground-gauche", "ground-right", "grass-top"]),
    ...box({ x: 0, y: 0, l: lt, h: ht }, -8, 0, ["grass-gauche", "grass-right", "grass-top"]),
  ];
  const rects = view.beds.map((p) => rectOf(p));
  const beds = view.beds
    .map((p, i): BedScene => {
      const r = pivoterRect(rects[i]!, quarter, l0, h0);
      const { bord, ground } = HAUTEURS[p.bed_type];
      const { faces, sillons } = shapes(p.bed_type, r);
      const plantings = view.plantings
        .filter((s) => s.bed_id === p.id)
        .sort((a, b) => a.segment_start - b.segment_start)
        .map((s) => plantingScene(p, s, plot, quarter, lt, ht, ground));
      // Plantings further back go first: sort by the depth of their zone once turned.
      plantings.sort((a, b) => a.centre.y - b.centre.y);
      return {
        id: p.id,
        name: p.name,
        type: p.bed_type,
        faces,
        sillons,
        contour: pts(
          project(r.x, r.y, bord + 1),
          project(r.x + r.l, r.y, bord + 1),
          project(r.x + r.l, r.y + r.h, bord + 1),
          project(r.x, r.y + r.h, bord + 1),
        ),
        ground,
        plantings,
        coinName: project(r.x + 0.2, r.y + r.h - 0.2, bord + 4),
        centre: project(r.x + r.l / 2, r.y + r.h / 2, bord),
        depth: depth(r.x + r.l / 2, r.y + r.h / 2),
      };
    })
    // Back to front, so a front bed covers the feet of a plant behind it.
    .sort((a, b) => a.depth - b.depth);
  return {
    l: lt,
    h: ht,
    cadre: cadre(lt, ht),
    socle,
    tufts: tufts(l0, h0, rects, quarter),
    beds,
  };
}
