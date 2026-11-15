// Logic of the drag and drop of the atelier (S05 §4): pointer to centimetres,
// snapping to cases, long press on touch, resizing with handles, and the
// return to the starting place when Rust refuses. No rule of S03 here: a
// position is only a proposal, Rust judges at the drop.

export type Point = { x: number; y: number };
export type Rect = { left: number; top: number; width: number; height: number };
/** The `viewBox` of the plan, in centimetres. */
export type Boite = { x: number; y: number; largeur: number; hauteur: number };
/** Where a planche is and how big, in centimetres. */
export type Geo = { x: number; y: number; longueur_cm: number; largeur_cm: number };
export type Poignee = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

/** Pointer moves under this distance, in pixels, are a click, not a drag. */
export const SEUIL_SOURIS_PX = 4;
/** On touch, the finger may drift this far while waiting for the long press. */
export const TOLERANCE_APPUI_LONG_PX = 8;
export const DELAI_APPUI_LONG_MS = 450;
/** The narrowest width the handles can give: the same minimum as Rust (S03 §15),
 *  so a handle never proposes a width Rust would call invalid. */
export const LARGEUR_MIN_CM = 10;

/** Pixels of the screen to centimetres of the plan. */
export function pixelsVersCm(p: Point, rect: Rect, boite: Boite): Point {
  return {
    x: boite.x + ((p.x - rect.left) / rect.width) * boite.largeur,
    y: boite.y + ((p.y - rect.top) / rect.height) * boite.hauteur,
  };
}

/** The nearest multiple of the case size. */
export function arrondiCase(cm: number, pas: number): number {
  return Math.round(cm / pas) * pas;
}

/** Top left of a planche carried by `decalage` (where the pointer holds it),
 *  snapped to the grid of cases. */
export function positionDeposee(pointeur: Point, decalage: Point, pas: number): Point {
  return { x: arrondiCase(pointeur.x - decalage.x, pas), y: arrondiCase(pointeur.y - decalage.y, pas) };
}

/** A planche dropped from the tray is held by its centre. */
export function positionDepuisBac(
  pointeur: Point,
  taille: { longueur_cm: number; largeur_cm: number },
  pas: number,
): Point {
  return positionDeposee(pointeur, { x: taille.longueur_cm / 2, y: taille.largeur_cm / 2 }, pas);
}

/**
 * The geometry proposed while a handle is dragged. The length moves case by
 * case (a whole number of cases, at least one); the width is free to the
 * centimetre (S01 F-PLN-1).
 */
export function redimensionner(g: Geo, poignee: Poignee, pointeur: Point, pas: number): Geo {
  const droite = g.x + g.longueur_cm;
  const bas = g.y + g.largeur_cm;
  let { x, y, longueur_cm, largeur_cm } = g;
  if (poignee.includes("e")) {
    longueur_cm = Math.max(pas, arrondiCase(pointeur.x - g.x, pas));
  }
  if (poignee.includes("w")) {
    longueur_cm = Math.max(pas, arrondiCase(droite - pointeur.x, pas));
    x = droite - longueur_cm;
  }
  if (poignee.includes("s")) {
    largeur_cm = Math.max(LARGEUR_MIN_CM, Math.round(pointeur.y - g.y));
  }
  if (poignee.includes("n")) {
    largeur_cm = Math.max(LARGEUR_MIN_CM, Math.round(bas - pointeur.y));
    y = bas - largeur_cm;
  }
  return { x, y, longueur_cm, largeur_cm };
}

/** Rust takes unsigned positions: a negative one is left of or above the
 *  terrain, so it is refused as outside without a call. */
export function estAvantLeTerrain(g: Pick<Geo, "x" | "y">): boolean {
  return g.x < 0 || g.y < 0;
}

export type TypePointeur = "mouse" | "touch" | "pen";
export type EtatGeste = "repos" | "attente" | "arme" | "glisse";

type Minuterie = {
  demarrer: (rappel: () => void, ms: number) => unknown;
  arreter: (id: unknown) => void;
};

const minuterieReelle: Minuterie = {
  demarrer: (rappel, ms) => setTimeout(rappel, ms),
  arreter: (id) => clearTimeout(id as ReturnType<typeof setTimeout>),
};

/**
 * When does a press become a drag? With a mouse (or a pen), as soon as the
 * pointer moves a few pixels. With a finger, only after a long press: a finger
 * that moves before that is scrolling the page, and the gesture is dropped
 * (S01 F-VIS-2, S05 §4).
 */
export class ControleurGlisser {
  etat: EtatGeste = "repos";
  #depart: Point = { x: 0, y: 0 };
  #type: TypePointeur = "mouse";
  #minuteur: unknown = null;
  readonly #surArme: () => void;
  readonly #delai: number;
  readonly #minuterie: Minuterie;

  constructor(options: { surArme?: () => void; delai?: number; minuterie?: Minuterie } = {}) {
    this.#surArme = options.surArme ?? (() => {});
    this.#delai = options.delai ?? DELAI_APPUI_LONG_MS;
    this.#minuterie = options.minuterie ?? minuterieReelle;
  }

  appui(type: TypePointeur, p: Point): void {
    this.annuler();
    this.#type = type;
    this.#depart = p;
    this.etat = "attente";
    if (type === "touch") {
      this.#minuteur = this.#minuterie.demarrer(() => {
        this.#minuteur = null;
        if (this.etat === "attente") {
          this.etat = "arme";
          this.#surArme();
        }
      }, this.#delai);
    }
  }

  /** `debut` once, when the drag starts; `glisse` while it goes on; `abandon`
   *  when a finger scrolls instead. */
  deplacer(p: Point): "rien" | "debut" | "glisse" | "abandon" {
    const distance = Math.hypot(p.x - this.#depart.x, p.y - this.#depart.y);
    switch (this.etat) {
      case "attente":
        if (this.#type === "touch") {
          if (distance > TOLERANCE_APPUI_LONG_PX) {
            this.annuler();
            return "abandon";
          }
          return "rien";
        }
        if (distance > SEUIL_SOURIS_PX) {
          this.etat = "glisse";
          return "debut";
        }
        return "rien";
      case "arme":
        this.etat = "glisse";
        return "debut";
      case "glisse":
        return "glisse";
      default:
        return "rien";
    }
  }

  /** `fin` after a drag, `clic` for a press without drag, `rien` otherwise. */
  relacher(): "rien" | "clic" | "fin" {
    const avant = this.etat;
    this.annuler();
    if (avant === "glisse") return "fin";
    if (avant === "attente" || avant === "arme") return "clic";
    return "rien";
  }

  annuler(): void {
    if (this.#minuteur !== null) {
      this.#minuterie.arreter(this.#minuteur);
      this.#minuteur = null;
    }
    this.etat = "repos";
  }
}

/** What the command answered. A refusal brings the planche back. */
export type Reponse = "ok" | "refus" | "annule";

export type IssueDepot = { position: Geo; reponse: Reponse | "inchange" | "avant-terrain" };

/**
 * End of a drag: where the planche stays. Nothing is sent if it did not
 * move; a position before the terrain is refused without a call (Rust takes
 * unsigned numbers); otherwise the command judges, and a refusal, or a
 * cancelled confirmation, returns the planche to `origine`.
 */
export async function deposer(
  origine: Geo,
  cible: Geo,
  envoyer: (g: Geo) => Promise<Reponse>,
): Promise<IssueDepot> {
  if (
    cible.x === origine.x &&
    cible.y === origine.y &&
    cible.longueur_cm === origine.longueur_cm &&
    cible.largeur_cm === origine.largeur_cm
  ) {
    return { position: origine, reponse: "inchange" };
  }
  if (estAvantLeTerrain(cible)) {
    return { position: origine, reponse: "avant-terrain" };
  }
  const reponse = await envoyer(cible);
  return { position: reponse === "ok" ? cible : origine, reponse };
}
