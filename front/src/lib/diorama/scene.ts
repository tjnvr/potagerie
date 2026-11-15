// The geometry of the diorama: from the year view (planches, series) and the
// turn of the view, everything the SVG needs, in view box pixels. Pure, so it
// is testable and the component only draws.

import type { PlancheView, SerieView, TerrainView, TypePlanche, YearView } from "../bindings";
import { chronologie, type Chronologie } from "./animation";
import { echellePlante, positionsPlantes } from "./plantes";
import {
  cadre,
  dimensions,
  enUnites,
  pivoter,
  pivoterRect,
  profondeur,
  projeter,
  type Point,
  type Quart,
  type Rect,
} from "./projection";

/** Height of the top of each kind of planche, and of the soil on it (D01). */
export const HAUTEURS: Record<TypePlanche, { bord: number; terre: number }> = {
  planche: { bord: 7, terre: 7 },
  carre: { bord: 9, terre: 8.5 },
  butte: { bord: 12, terre: 12 },
  bac: { bord: 14, terre: 13.5 },
};

export type Face = { points: string; classe: string };
export type Sillon = { a: Point; b: Point };

export type PiedScene = { x: number; y: number };

export type SerieScene = {
  id: string;
  culture: string;
  chrono: Chronologie;
  echelle: number;
  /** Feet in view box pixels, back to front. */
  pieds: PiedScene[];
  /** Middle of the zone, for the label of a serie sown under cover. */
  centre: PiedScene;
};

export type PlancheScene = {
  id: string;
  nom: string;
  type: TypePlanche;
  faces: Face[];
  sillons: Sillon[];
  contour: string;
  /** Top of the soil, where the plants stand. */
  terre: number;
  series: SerieScene[];
  /** Left corner of the top, where the name sits. */
  coinNom: Point;
  /** Middle of the top, for the free sign. */
  centre: Point;
  profondeur: number;
};

export type Touffe = { x: number; y: number; fleur: boolean; teinte: number };

export type Scene = {
  l: number;
  h: number;
  cadre: [number, number, number, number];
  socle: Face[];
  touffes: Touffe[];
  planches: PlancheScene[];
};

function pts(...ps: Point[]): string {
  return ps.map(([x, y]) => `${x.toFixed(1)},${y.toFixed(1)}`).join(" ");
}

/** The three visible faces of a box: front-left, right, top (gen.py, box()). */
function boite(r: Rect, z0: number, z1: number, classes: [string, string, string]): Face[] {
  const P = projeter;
  const { x, y, l, h } = r;
  return [
    { points: pts(P(x, y + h, z1), P(x + l, y + h, z1), P(x + l, y + h, z0), P(x, y + h, z0)), classe: classes[0] },
    { points: pts(P(x + l, y, z1), P(x + l, y + h, z1), P(x + l, y + h, z0), P(x + l, y, z0)), classe: classes[1] },
    { points: pts(P(x, y, z1), P(x + l, y, z1), P(x + l, y + h, z1), P(x, y + h, z1)), classe: classes[2] },
  ];
}

function dessus(r: Rect, z: number, classe: string): Face {
  const P = projeter;
  return { points: pts(P(r.x, r.y, z), P(r.x + r.l, r.y, z), P(r.x + r.l, r.y + r.h, z), P(r.x, r.y + r.h, z)), classe };
}

function retrecir(r: Rect, e: number): Rect {
  return { x: r.x + e, y: r.y + e, l: r.l - 2 * e, h: r.h - 2 * e };
}

/** The faces and furrows of a planche of each type, on its rectangle `r`. */
function formes(type: TypePlanche, r: Rect): { faces: Face[]; sillons: Sillon[] } {
  const { bord, terre } = HAUTEURS[type];
  const P = projeter;
  switch (type) {
    case "planche": {
      const sillons: Sillon[] = [];
      for (let j = 1; j < r.h * 2; j++) {
        sillons.push({ a: P(r.x + 0.15, r.y + j / 2, bord), b: P(r.x + r.l - 0.15, r.y + j / 2, bord) });
      }
      return { faces: boite(r, 0, bord, ["lit-gauche", "lit-droite", "lit-haut"]), sillons };
    }
    case "butte": {
      // A mound: the top is smaller than the foot, the sides slope.
      const i = Math.min(0.35, r.l / 4, r.h / 4);
      const t = retrecir(r, i);
      return {
        faces: [
          {
            points: pts(P(r.x, r.y + r.h, 0), P(r.x + r.l, r.y + r.h, 0), P(t.x + t.l, t.y + t.h, bord), P(t.x, t.y + t.h, bord)),
            classe: "lit-gauche",
          },
          {
            points: pts(P(r.x + r.l, r.y, 0), P(r.x + r.l, r.y + r.h, 0), P(t.x + t.l, t.y + t.h, bord), P(t.x + t.l, t.y, bord)),
            classe: "lit-droite",
          },
          dessus(t, bord, "lit-haut"),
        ],
        sillons: [],
      };
    }
    case "carre":
    case "bac": {
      const faces = boite(r, 0, bord, ["bois-gauche", "bois-droite", "bois-haut"]);
      faces.push(dessus(retrecir(r, type === "bac" ? 0.18 : 0.14), terre, "lit-haut"));
      return { faces, sillons: [] };
    }
  }
}

/** Small deterministic generator, so the tufts stay where they are. */
function aleatoire(graine: number): () => number {
  let a = graine >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function touffes(l: number, h: number, planches: Rect[], quart: Quart): Touffe[] {
  const hasard = aleatoire(7);
  const resultat: Touffe[] = [];
  const nombre = Math.min(90, Math.round((l * h) / 3));
  for (let i = 0; i < nombre * 3 && resultat.length < nombre; i++) {
    const gx = 0.4 + hasard() * (l - 0.8);
    const gy = 0.4 + hasard() * (h - 0.8);
    const fleur = hasard() < 0.25;
    const teinte = Math.floor(hasard() * 3);
    if (planches.some((r) => gx > r.x - 0.3 && gx < r.x + r.l + 0.3 && gy > r.y - 0.3 && gy < r.y + r.h + 0.3)) continue;
    const [x, y] = projeter(...pivoter([gx, gy], quart, l, h));
    resultat.push({ x, y, fleur, teinte });
  }
  return resultat;
}

function rectDe(p: PlancheView): Rect {
  return { x: enUnites(p.x), y: enUnites(p.y), l: enUnites(p.longueur_cm), h: enUnites(p.largeur_cm) };
}

function serieScene(p: PlancheView, s: SerieView, terrain: TerrainView, quart: Quart, lt: number, ht: number, terre: number): SerieScene {
  const brut = positionsPlantes(p, s, terrain.case_cm);
  const pieds = brut
    .map((b) => pivoter([b.gx, b.gy], quart, lt, ht))
    .sort((a, b) => profondeur(a[0], a[1]) - profondeur(b[0], b[1]))
    .map(([gx, gy]) => {
      const [x, y] = projeter(gx, gy, terre);
      return { x, y };
    });
  const debut = p.x + (s.troncon_debut - 1) * terrain.case_cm;
  const longueur = (s.troncon_fin - s.troncon_debut + 1) * terrain.case_cm;
  const [cx, cy] = pivoter([enUnites(debut + longueur / 2), enUnites(p.y + p.largeur_cm / 2)], quart, lt, ht);
  const [x, y] = projeter(cx, cy, terre);
  return {
    id: s.id,
    culture: s.culture_id,
    chrono: chronologie(s.appearance_changes),
    echelle: echellePlante(p, s, terrain.case_cm),
    pieds,
    centre: { x, y },
  };
}

/** The whole scene for a year view whose terrain exists, turned by `quart`. */
export function construireScene(vue: YearView, quart: Quart): Scene | null {
  const terrain = vue.terrain;
  if (!terrain) return null;
  const l0 = enUnites(terrain.longueur_cm);
  const h0 = enUnites(terrain.largeur_cm);
  const { l: lt, h: ht } = dimensions(quart, l0, h0);
  const socle: Face[] = [
    ...boite({ x: 0, y: 0, l: lt, h: ht }, -30, -8, ["terre-gauche", "terre-droite", "herbe-haut"]),
    ...boite({ x: 0, y: 0, l: lt, h: ht }, -8, 0, ["herbe-gauche", "herbe-droite", "herbe-haut"]),
  ];
  const rects = vue.planches.map((p) => rectDe(p));
  const planches = vue.planches
    .map((p, i): PlancheScene => {
      const r = pivoterRect(rects[i]!, quart, l0, h0);
      const { bord, terre } = HAUTEURS[p.planche_type];
      const { faces, sillons } = formes(p.planche_type, r);
      const series = vue.series
        .filter((s) => s.planche_id === p.id)
        .sort((a, b) => a.troncon_debut - b.troncon_debut)
        .map((s) => serieScene(p, s, terrain, quart, lt, ht, terre));
      // Series further back go first: sort by the depth of their zone once turned.
      series.sort((a, b) => a.centre.y - b.centre.y);
      return {
        id: p.id,
        nom: p.nom,
        type: p.planche_type,
        faces,
        sillons,
        contour: pts(
          projeter(r.x, r.y, bord + 1),
          projeter(r.x + r.l, r.y, bord + 1),
          projeter(r.x + r.l, r.y + r.h, bord + 1),
          projeter(r.x, r.y + r.h, bord + 1),
        ),
        terre,
        series,
        coinNom: projeter(r.x + 0.2, r.y + r.h - 0.2, bord + 4),
        centre: projeter(r.x + r.l / 2, r.y + r.h / 2, bord),
        profondeur: profondeur(r.x + r.l / 2, r.y + r.h / 2),
      };
    })
    // Back to front, so a front planche covers the feet of a plant behind it.
    .sort((a, b) => a.profondeur - b.profondeur);
  return {
    l: lt,
    h: ht,
    cadre: cadre(lt, ht),
    socle,
    touffes: touffes(l0, h0, rects, quart),
    planches,
  };
}
