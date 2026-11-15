// Draws the plants of the diorama, frame after frame, without going through
// the reactive system: one <use> per plant (P01), created once, then only the
// href and the transform of the ones that change are written. A serie whose
// state did not change since the last frame costs one comparison.

import { etatAu, SYMBOLE_ETIQUETTE, type Etat } from "./animation";
import type { SerieScene } from "./scene";
import { VIEWBOX_SYMBOLE } from "./symboles";

const NS = "http://www.w3.org/2000/svg";
const [VX, VY, VL, VH] = VIEWBOX_SYMBOLE;

type SerieRendue = {
  scene: SerieScene;
  groupe: SVGGElement;
  plantes: SVGGElement;
  uses: SVGUseElement[];
  /** The symbol and scale each plant was last given. */
  dessines: (string | null)[];
  echelles: number[];
  etiquette: SVGUseElement;
  /** What the serie should show, and how many plants already show it. */
  cible: Etat;
  fait: number;
  /** Where the next slice starts. */
  curseur: number;
};

function nouvelUse(): SVGUseElement {
  const u = document.createElementNS(NS, "use");
  u.setAttribute("x", String(VX));
  u.setAttribute("y", String(VY));
  u.setAttribute("width", String(VL));
  u.setAttribute("height", String(VH));
  return u;
}

const RIEN: Etat = { symbole: null, echelle: 1 };

export class RenduPlantes {
  #series: SerieRendue[] = [];
  /** Plants currently visible, for the counter of the dev tools. */
  visibles = 0;
  #depart = 0;

  /** Builds the <use> of every serie inside `couche` (one planche), replacing what it held. */
  construire(couche: SVGGElement, series: readonly SerieScene[]): void {
    couche.replaceChildren();
    for (const scene of series) {
      const groupe = document.createElementNS(NS, "g");
      const plantes = document.createElementNS(NS, "g");
      const uses = scene.pieds.map(() => {
        const u = nouvelUse();
        plantes.appendChild(u);
        return u;
      });
      const etiquette = nouvelUse();
      etiquette.setAttribute("transform", `translate(${scene.centre.x.toFixed(1)} ${scene.centre.y.toFixed(1)})`);
      etiquette.setAttribute("href", `#${SYMBOLE_ETIQUETTE}`);
      groupe.append(plantes, etiquette);
      groupe.style.display = "none";
      couche.appendChild(groupe);
      this.#series.push({
        scene,
        groupe,
        plantes,
        uses,
        dessines: uses.map(() => null),
        echelles: uses.map(() => -1),
        etiquette,
        cible: RIEN,
        fait: uses.length,
        curseur: 0,
      });
    }
  }

  /** Forgets the series built so far (before a new `construire` of every planche). */
  vider(): void {
    this.#series = [];
    this.#depart = 0;
  }

  /**
   * Shows every serie as it is on day `jour`.
   *
   * `plafond` bounds the plants redrawn in one call. A serie that changes
   * stage, or grows, over more plants than that is brought up to date over
   * several calls, a slice of its plants at a time, so no frame has to repaint
   * thousands of plants at once (a few frames of lag, invisible). Without
   * `plafond`, everything is drawn at once.
   */
  appliquer(jour: number, animer: boolean, plafond = Number.POSITIVE_INFINITY): void {
    const n = this.#series.length;
    let reste = plafond;
    let reporte = -1;
    for (let k = 0; k < n; k++) {
      const index = (this.#depart + k) % n;
      const s = this.#series[index]!;
      const etat = etatAu(s.scene.chrono, s.scene.culture, jour, animer);
      const echelle = Math.round(etat.echelle * 100) / 100;
      if (etat.symbole !== s.cible.symbole || echelle !== s.cible.echelle) {
        s.cible = { symbole: etat.symbole, echelle };
        s.fait = 0;
        // Show or hide at once, the plants follow.
        s.groupe.style.display = etat.symbole === null ? "none" : "";
        const etiquette = etat.symbole === SYMBOLE_ETIQUETTE;
        s.plantes.style.display = etiquette ? "none" : "";
        s.etiquette.style.display = etiquette ? "" : "none";
      }
      if (s.fait >= s.uses.length) continue;
      if (reste <= 0) {
        if (reporte < 0) reporte = index;
        continue;
      }
      reste -= this.#mettreAJour(s, reste);
      if (s.fait < s.uses.length && reporte < 0) reporte = index;
    }
    this.#depart = reporte < 0 ? 0 : reporte;
    let visibles = 0;
    for (const s of this.#series) {
      const c = s.cible.symbole;
      if (c !== null && c !== SYMBOLE_ETIQUETTE) visibles += s.uses.length;
    }
    this.visibles = visibles;
  }

  /**
   * Brings up to `budget` more plants of `s` to its target, going round the
   * serie from where it stopped; returns how many were redrawn. A plant
   * already on the target costs only a comparison.
   */
  #mettreAJour(s: SerieRendue, budget: number): number {
    const symbole = s.cible.symbole;
    if (symbole === null || symbole === SYMBOLE_ETIQUETTE) {
      s.fait = s.uses.length;
      return 1;
    }
    const href = `#${symbole}`;
    const echelle = s.cible.echelle;
    const k = s.scene.echelle * echelle;
    const n = s.uses.length;
    let redessines = 0;
    let vus = 0;
    while (vus < n && redessines < Math.max(1, budget)) {
      const i = (s.curseur + vus) % n;
      vus++;
      if (s.dessines[i] === symbole && s.echelles[i] === echelle) continue;
      const u = s.uses[i]!;
      if (s.dessines[i] !== symbole) {
        u.setAttribute("href", href);
        s.dessines[i] = symbole;
      }
      const p = s.scene.pieds[i]!;
      u.setAttribute("transform", `translate(${p.x.toFixed(1)} ${p.y.toFixed(1)}) scale(${k.toFixed(3)})`);
      s.echelles[i] = echelle;
      redessines++;
    }
    s.curseur = (s.curseur + vus) % n;
    // Done once a whole round found nothing left to redraw.
    if (vus === n && redessines < Math.max(1, budget)) s.fait = n;
    else if (vus === n && s.dessines.every((d, i) => d === symbole && s.echelles[i] === echelle)) s.fait = n;
    return redessines;
  }
}
