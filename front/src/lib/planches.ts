import type { PlancheView, TerrainView, TypePlanche } from "./bindings";
import { m } from "./paraglide/messages.js";

/** At most this many planches in a year (S01 F-PLN-1). Rust enforces it. */
export const MAX_PLANCHES = 10;

/** Size of a new planche of each type (S01 F-PLN-1), as Rust creates it:
 *  longueur along x, largeur along y. Only used to choose where to drop it. */
export const TAILLE_PAR_DEFAUT: Record<TypePlanche, { longueur_cm: number; largeur_cm: number }> = {
  planche: { longueur_cm: 300, largeur_cm: 120 },
  carre: { longueur_cm: 100, largeur_cm: 100 },
  butte: { longueur_cm: 300, largeur_cm: 120 },
  bac: { longueur_cm: 200, largeur_cm: 120 },
};

const NOMS_DE_TYPE: Record<TypePlanche, () => string> = {
  planche: m.type_planche,
  carre: m.type_carre,
  butte: m.type_butte,
  bac: m.type_bac,
};

export function nomDeType(type: TypePlanche): string {
  return NOMS_DE_TYPE[type]();
}

/** Default name of a new planche: its type and the first letter no other
 *  planche ends with, « Planche A », « Carré B »… (S01 F-PLN-1). */
export function nomParDefaut(type: TypePlanche, planches: readonly PlancheView[]): string {
  let i = 0;
  while (planches.some((p) => p.nom.endsWith(` ${lettreDe(i)}`))) {
    i++;
  }
  return m.nom_par_defaut({ type: nomDeType(type), lettre: lettreDe(i) });
}

/** A, B, … Z, AA, AB… */
export function lettreDe(index: number): string {
  let n = index;
  let lettres = "";
  do {
    lettres = String.fromCharCode(65 + (n % 26)) + lettres;
    n = Math.floor(n / 26) - 1;
  } while (n >= 0);
  return lettres;
}

type Rect = { x: number; y: number; l: number; h: number };

function chevauche(a: Rect, b: Rect, marge: number): boolean {
  return (
    a.x < b.x + b.l + marge &&
    b.x < a.x + a.l + marge &&
    a.y < b.y + b.h + marge &&
    b.y < a.y + a.h + marge
  );
}

/**
 * Where the demo button « Ajouter une planche » drops a new planche: the first
 * spot, on the case grid, read row by row, that keeps the allée width from the
 * edges and from the other planches. `null` if there is no room.
 *
 * This only replaces the drag and drop of the atelier (F-PLN-6) for now: Rust
 * remains the judge of what is valid.
 */
export function positionLibre(
  terrain: TerrainView,
  planches: readonly PlancheView[],
  type: TypePlanche,
): { x: number; y: number } | null {
  const { longueur_cm, largeur_cm } = TAILLE_PAR_DEFAUT[type];
  const pas = terrain.case_cm;
  const marge = terrain.allee_cm;
  const debut = Math.ceil(marge / pas) * pas;
  const autres: Rect[] = planches.map((p) => ({
    x: p.x,
    y: p.y,
    l: p.longueur_cm,
    h: p.largeur_cm,
  }));
  for (let y = debut; y + largeur_cm + marge <= terrain.largeur_cm; y += pas) {
    for (let x = debut; x + longueur_cm + marge <= terrain.longueur_cm; x += pas) {
      const ici = { x, y, l: longueur_cm, h: largeur_cm };
      if (!autres.some((autre) => chevauche(ici, autre, marge))) {
        return { x, y };
      }
    }
  }
  return null;
}

/** The years offered by the season selector: last year to two years ahead. */
export function anneesProposees(anneeEnCours: number): number[] {
  return [anneeEnCours - 1, anneeEnCours, anneeEnCours + 1, anneeEnCours + 2];
}

/** Centimetres as « 1,2 » metres, the way the plan reads. */
export function enMetres(cm: number): string {
  return (cm / 100).toLocaleString("fr-FR", { maximumFractionDigits: 2 });
}

/** Square centimetres as « 71,5 » m². */
export function enM2(cm2: number): string {
  return (cm2 / 10_000).toLocaleString("fr-FR", { maximumFractionDigits: 1 });
}

/** The balance of the plan (S01 F-PLN-5): surfaces and number of planches. */
export function bilan(terrain: TerrainView, planches: readonly PlancheView[]) {
  const total = terrain.longueur_cm * terrain.largeur_cm;
  const cultivable = planches.reduce((somme, p) => somme + p.longueur_cm * p.largeur_cm, 0);
  return { total, cultivable, allees: total - cultivable, nombre: planches.length };
}

/** A civil date `YYYY-MM-DD` as « 15 mars », with the year when it is not `annee`. */
export function dateCourte(date: string, annee?: number): string {
  const jour = new Date(`${date}T12:00:00`);
  const options: Intl.DateTimeFormatOptions =
    jour.getFullYear() === annee ? { day: "numeric", month: "short" } : { day: "numeric", month: "short", year: "numeric" };
  return jour.toLocaleDateString("fr-FR", options);
}

/** The French name of a crop of the catalogue (`catalogue()` sends none):
 *  the message `culture_<id>`, hyphens of the id written as underscores. */
export function nomCulture(id: string): string {
  const nom = (m as unknown as Record<string, (() => string) | undefined>)[`culture_${id.replaceAll("-", "_")}`];
  return nom ? nom() : id;
}
