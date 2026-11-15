// Draws the plants of the diorama, frame after frame, without going through
// the reactive system: one <use> per plant (P01), created once, then only the
// href and the transform of the ones that change are written. A planting whose
// state did not change since the last frame costs one comparison.

import { stateTo, SYMBOL_ETIQUETTE, type State } from "./animation";
import type { PlantingScene } from "./scene";
import { VIEWBOX_SYMBOL } from "./symboles";

const NS = "http://www.w3.org/2000/svg";
const [VX, VY, VL, VH] = VIEWBOX_SYMBOL;

type PlantingRendered = {
  scene: PlantingScene;
  group: SVGGElement;
  plants: SVGGElement;
  uses: SVGUseElement[];
  /** The symbol and scale each plant was last given. */
  drawn: (string | null)[];
  scales: number[];
  etiquette: SVGUseElement;
  /** What the planting should show, and how many plants already show it. */
  target: State;
  done: number;
  /** Where the next slice starts. */
  cursor: number;
};

function newUse(): SVGUseElement {
  const u = document.createElementNS(NS, "use");
  u.setAttribute("x", String(VX));
  u.setAttribute("y", String(VY));
  u.setAttribute("width", String(VL));
  u.setAttribute("height", String(VH));
  return u;
}

const NOTHING: State = { symbol: null, scale: 1 };

export class RenderingPlants {
  #plantings: PlantingRendered[] = [];
  /** Plants currently visible, for the counter of the dev tools. */
  visible = 0;
  #depart = 0;

  /** Builds the <use> of every planting inside `layer` (one bed), replacing what it held. */
  build(layer: SVGGElement, plantings: readonly PlantingScene[]): void {
    layer.replaceChildren();
    for (const scene of plantings) {
      const group = document.createElementNS(NS, "g");
      const plants = document.createElementNS(NS, "g");
      const uses = scene.plants.map(() => {
        const u = newUse();
        plants.appendChild(u);
        return u;
      });
      const etiquette = newUse();
      etiquette.setAttribute("transform", `translate(${scene.centre.x.toFixed(1)} ${scene.centre.y.toFixed(1)})`);
      etiquette.setAttribute("href", `#${SYMBOL_ETIQUETTE}`);
      group.append(plants, etiquette);
      group.style.display = "none";
      layer.appendChild(group);
      this.#plantings.push({
        scene,
        group,
        plants,
        uses,
        drawn: uses.map(() => null),
        scales: uses.map(() => -1),
        etiquette,
        target: NOTHING,
        done: uses.length,
        cursor: 0,
      });
    }
  }

  /** Forgets the plantings built so far (before a new `build` of every bed). */
  clear(): void {
    this.#plantings = [];
    this.#depart = 0;
  }

  /**
   * Shows every planting as it is on day `day`.
   *
   * `ceiling` bounds the plants redrawn in one call. A planting that changes
   * stage, or grows, over more plants than that is brought up to date over
   * several calls, a slice of its plants at a time, so no frame has to repaint
   * thousands of plants at once (a few frames of lag, invisible). Without
   * `ceiling`, everything is drawn at once.
   */
  apply(day: number, animate: boolean, ceiling = Number.POSITIVE_INFINITY): void {
    const n = this.#plantings.length;
    let rest = ceiling;
    let postponed = -1;
    for (let k = 0; k < n; k++) {
      const index = (this.#depart + k) % n;
      const s = this.#plantings[index]!;
      const state = stateTo(s.scene.chrono, s.scene.crop, day, animate);
      const scale = Math.round(state.scale * 100) / 100;
      if (state.symbol !== s.target.symbol || scale !== s.target.scale) {
        s.target = { symbol: state.symbol, scale };
        s.done = 0;
        // Show or hide at once, the plants follow.
        s.group.style.display = state.symbol === null ? "none" : "";
        const etiquette = state.symbol === SYMBOL_ETIQUETTE;
        s.plants.style.display = etiquette ? "none" : "";
        s.etiquette.style.display = etiquette ? "" : "none";
      }
      if (s.done >= s.uses.length) continue;
      if (rest <= 0) {
        if (postponed < 0) postponed = index;
        continue;
      }
      rest -= this.#putADay(s, rest);
      if (s.done < s.uses.length && postponed < 0) postponed = index;
    }
    this.#depart = postponed < 0 ? 0 : postponed;
    let visible = 0;
    for (const s of this.#plantings) {
      const c = s.target.symbol;
      if (c !== null && c !== SYMBOL_ETIQUETTE) visible += s.uses.length;
    }
    this.visible = visible;
  }

  /**
   * Brings up to `budget` more plants of `s` to its target, going round the
   * planting from where it stopped; returns how many were redrawn. A plant
   * already on the target costs only a comparison.
   */
  #putADay(s: PlantingRendered, budget: number): number {
    const symbol = s.target.symbol;
    if (symbol === null || symbol === SYMBOL_ETIQUETTE) {
      s.done = s.uses.length;
      return 1;
    }
    const href = `#${symbol}`;
    const scale = s.target.scale;
    const k = s.scene.scale * scale;
    const n = s.uses.length;
    let redrawn = 0;
    let vus = 0;
    while (vus < n && redrawn < Math.max(1, budget)) {
      const i = (s.cursor + vus) % n;
      vus++;
      if (s.drawn[i] === symbol && s.scales[i] === scale) continue;
      const u = s.uses[i]!;
      if (s.drawn[i] !== symbol) {
        u.setAttribute("href", href);
        s.drawn[i] = symbol;
      }
      const p = s.scene.plants[i]!;
      u.setAttribute("transform", `translate(${p.x.toFixed(1)} ${p.y.toFixed(1)}) scale(${k.toFixed(3)})`);
      s.scales[i] = scale;
      redrawn++;
    }
    s.cursor = (s.cursor + vus) % n;
    // Done once a whole round found nothing left to redraw.
    if (vus === n && redrawn < Math.max(1, budget)) s.done = n;
    else if (vus === n && s.drawn.every((d, i) => d === symbol && s.scales[i] === scale)) s.done = n;
    return redrawn;
  }
}
