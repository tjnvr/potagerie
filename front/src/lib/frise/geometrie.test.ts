import { describe, expect, it, vi } from "vitest";
import type { CreneauView, PlancheView, SerieView, YearView } from "../bindings";
import { dateDe, jourDe } from "../dates";
import { ControleurGlisser, DELAI_APPUI_LONG_MS } from "../atelier/glisser";
import { decalageAffiche, deposerBarre } from "./barre";
import {
  JOURS_MAX,
  JOURS_MIN,
  anneesVisibles,
  barreDe,
  centrerSur,
  construireLignes,
  decalageEnJours,
  defiler,
  echelleCentree,
  empiler,
  garderVisible,
  jourDeX,
  jourSousLePointeur,
  joursVisibles,
  moisVisibles,
  xDe,
  zoomer,
} from "./geometrie";

const echelle = { debutJour: jourDe("2027-01-01"), pxParJour: 2, largeur: 730 };

describe("pointer to date", () => {
  it("maps a pixel to the civil day under it", () => {
    expect(dateDe(jourSousLePointeur(echelle, 100, 100))).toBe("2027-01-01");
    expect(dateDe(jourSousLePointeur(echelle, 100 + 2 * 31, 100))).toBe("2027-02-01");
    // The middle of a day stays on that day.
    expect(dateDe(jourSousLePointeur(echelle, 100 + 2 * 31 + 1, 100))).toBe("2027-02-01");
    expect(dateDe(jourSousLePointeur(echelle, 100 + 2 * 31 - 1, 100))).toBe("2027-01-31");
  });

  it("goes back and forth between days and pixels", () => {
    const jour = jourDe("2027-06-20");
    expect(jourDeX(echelle, xDe(echelle, jour))).toBe(jour);
  });

  it("reads the window left of the visible days", () => {
    expect(dateDe(jourSousLePointeur(echelle, 50, 100))).toBe("2026-12-07");
  });
});

describe("scrolling and zoom", () => {
  it("follows the pointer: moving right shows earlier days", () => {
    const apres = defiler(echelle, 200);
    expect(apres.debutJour).toBe(echelle.debutJour - 100);
    expect(defiler(echelle, -20).debutJour).toBe(echelle.debutJour + 10);
  });

  it("does not move for no distance", () => {
    expect(defiler(echelle, 0)).toEqual(echelle);
  });

  it("centres the timeline on a day", () => {
    const jour = jourDe("2027-06-20");
    const e = centrerSur(echelle, jour);
    expect(e.debutJour + joursVisibles(e) / 2).toBe(jour);
  });

  it("keeps a day in view with the least scroll", () => {
    const visible = jourDe("2027-06-01");
    expect(garderVisible(echelle, visible)).toBe(echelle);
    const loin = jourDe("2028-03-01");
    const e = garderVisible(echelle, loin);
    expect(jourDeX(e, e.largeur)).toBeGreaterThan(loin);
    expect(e.debutJour).toBeGreaterThan(echelle.debutJour);
    expect(e.pxParJour).toBe(echelle.pxParJour);
    const avant = garderVisible(echelle, jourDe("2026-06-01"));
    expect(avant.debutJour).toBeLessThan(jourDe("2026-06-01"));
  });

  it("zooms around the day under the anchor and stays between 3 months and 3 years", () => {
    const ancre = 300;
    const jourAvant = jourDeX(echelle, ancre);
    const e = zoomer(echelle, 0.5, ancre);
    expect(joursVisibles(e)).toBeCloseTo(joursVisibles(echelle) / 2);
    expect(jourDeX(e, ancre)).toBeCloseTo(jourAvant);
    expect(joursVisibles(zoomer(echelle, 0.0001, 0))).toBeCloseTo(JOURS_MIN);
    expect(joursVisibles(zoomer(echelle, 1000, 0))).toBeCloseTo(JOURS_MAX);
  });

  it("opens on 12 months", () => {
    const e = echelleCentree(1200, jourDe("2027-06-20"));
    expect(joursVisibles(e)).toBe(365);
  });

  it("lists the years to load and the month columns", () => {
    expect(anneesVisibles(echelle)).toEqual([2027, 2028]);
    expect(anneesVisibles({ ...echelle, debutJour: jourDe("2027-03-01") })).toEqual([2027, 2028]);
    expect(anneesVisibles({ ...echelle, debutJour: jourDe("2027-01-01"), largeur: 600 })).toEqual([2027]);
    const mois = moisVisibles({ ...echelle, largeur: 600 });
    expect(mois).toHaveLength(10);
    expect(mois[0]).toMatchObject({ annee: 2027, mois: 0, jours: 31 });
    expect(mois[1]).toMatchObject({ mois: 1, jours: 28 });
  });
});

describe("drag of a bar", () => {
  it("moves by whole days", () => {
    expect(decalageEnJours(10, 4)).toBe(3);
    expect(decalageEnJours(-5, 4)).toBe(-1);
    expect(decalageEnJours(1, 4)).toBe(0);
    expect(decalageAffiche(10, 4)).toBe(12);
  });

  it("sends nothing for a click", async () => {
    const envoyer = vi.fn();
    expect(await deposerBarre(1, 4, envoyer)).toEqual({ reponse: "inchange", jours: 0 });
    expect(envoyer).not.toHaveBeenCalled();
  });

  it("keeps the new place when Rust accepts", async () => {
    const envoyer = vi.fn().mockResolvedValue("ok");
    expect(await deposerBarre(-9, 3, envoyer)).toEqual({ reponse: "ok", jours: -3 });
    expect(envoyer).toHaveBeenCalledWith(-3);
  });

  it("brings the bar back when Rust refuses (S03 T12)", async () => {
    const envoyer = vi.fn().mockResolvedValue("refus");
    expect(await deposerBarre(-3, 3, envoyer)).toEqual({ reponse: "refus", jours: 0 });
    expect(await deposerBarre(-3, 3, async () => "annule")).toEqual({ reponse: "annule", jours: 0 });
  });
});

describe("long press on a bar", () => {
  function avecMinuterie() {
    let rappel: (() => void) | null = null;
    return {
      minuterie: {
        demarrer: (f: () => void) => ((rappel = f), 1),
        arreter: () => (rappel = null),
      },
      declencher: () => rappel?.(),
    };
  }

  it("arms a finger after the delay, then drags", () => {
    const { minuterie, declencher } = avecMinuterie();
    const arme = vi.fn();
    const c = new ControleurGlisser({ surArme: arme, minuterie, delai: DELAI_APPUI_LONG_MS });
    c.appui("touch", { x: 10, y: 10 });
    expect(c.deplacer({ x: 12, y: 10 })).toBe("rien");
    declencher();
    expect(arme).toHaveBeenCalledOnce();
    expect(c.deplacer({ x: 40, y: 10 })).toBe("debut");
    expect(c.deplacer({ x: 60, y: 10 })).toBe("glisse");
    expect(c.relacher()).toBe("fin");
  });

  it("lets a finger that moves early scroll the timeline instead", () => {
    const { minuterie } = avecMinuterie();
    const c = new ControleurGlisser({ minuterie });
    c.appui("touch", { x: 10, y: 10 });
    expect(c.deplacer({ x: 60, y: 10 })).toBe("abandon");
    expect(c.relacher()).toBe("rien");
  });

  it("starts a mouse drag after a few pixels, without waiting", () => {
    const c = new ControleurGlisser();
    c.appui("mouse", { x: 10, y: 10 });
    expect(c.deplacer({ x: 20, y: 10 })).toBe("debut");
  });
});

// ---------- Rows from the year views ----------

const PLANCHE: PlancheView = {
  id: "p1",
  nom: "Les tomates",
  planche_type: "planche",
  x: 50,
  y: 50,
  longueur_cm: 300,
  largeur_cm: 120,
  troncons: 6,
  series_supprimees: [],
};

function serie(id: string, culture: string, over: Partial<SerieView> = {}): SerieView {
  return {
    id,
    culture_id: culture,
    planche_id: "p1",
    troncon_debut: 1,
    troncon_fin: 3,
    mode: "abri",
    date: "2027-03-10",
    sowing: "2027-03-10",
    planting_out: "2027-05-05",
    harvest_start: "2027-07-19",
    end: "2027-09-27",
    manual_end: false,
    pot_days: 56,
    rangs: 1,
    pieds_par_rang: 2,
    appearance_today: { appearance: "nothing" },
    ...over,
  } as SerieView;
}

function vue(year: number, over: Partial<YearView> = {}): YearView {
  return {
    year,
    today: "2027-06-20",
    read_only: false,
    terrain: { longueur_cm: 1100, largeur_cm: 650, case_cm: 50, allee_cm: 50 },
    version: 3,
    updated_at: null,
    planches: [PLANCHE],
    series: [],
    conflicts: [],
    historique: [],
    creneaux: [],
    annotations: [],
    tasks: [],
    ...over,
  };
}

describe("bars and phases", () => {
  it("cuts a serie into pots, growth and harvest from the dates Rust sent", () => {
    const b = barreDe(serie("a", "tomate"));
    expect(b.segments.map((s) => s.phase)).toEqual(["godets", "croissance", "recolte"]);
    expect(dateDe(b.debut)).toBe("2027-03-10");
    expect(dateDe(b.segments[1]!.debut)).toBe("2027-05-05");
    expect(dateDe(b.segments[2]!.debut)).toBe("2027-07-19");
    expect(dateDe(b.fin)).toBe("2027-09-27");
  });

  it("has no pots for bought plants or sowing in the ground", () => {
    const plants = barreDe(serie("a", "laitue", { mode: "plants", sowing: null, planting_out: "2027-09-27", harvest_start: "2027-11-16", end: "2027-12-06", date: "2027-09-27" }));
    expect(plants.segments.map((s) => s.phase)).toEqual(["croissance", "recolte"]);
    expect(dateDe(plants.debut)).toBe("2027-09-27");
    const terre = barreDe(serie("a", "carotte", { mode: "pleine_terre", sowing: "2027-03-15", planting_out: "2027-03-15", harvest_start: "2027-06-23", end: "2027-08-02" }));
    expect(terre.segments.map((s) => s.phase)).toEqual(["croissance", "recolte"]);
  });
});

describe("stacking", () => {
  it("puts overlapping items on separate sub-rows and reuses a free one", () => {
    const voies = empiler([
      { id: "a", debut: 0, fin: 10 },
      { id: "b", debut: 5, fin: 15 },
      { id: "c", debut: 10, fin: 20 },
    ]);
    expect(voies.get("a")).toBe(0);
    expect(voies.get("b")).toBe(1);
    // c starts the day a ends: the half-open ranges do not overlap.
    expect(voies.get("c")).toBe(0);
  });

  it("avoids the sub-rows that are already taken", () => {
    const voies = empiler([{ id: "x", debut: 0, fin: 10 }], [{ debut: 5, fin: 8, voie: 0 }]);
    expect(voies.get("x")).toBe(1);
  });
});

describe("rows", () => {
  const creneau: CreneauView = {
    planche_id: "p1",
    troncon_debut: 1,
    troncon_fin: 6,
    start: "2027-09-27",
    end: "2027-11-15",
    suggestions: [],
  };

  it("makes one row per planche, with its bars and free slots", () => {
    const lignes = construireLignes(
      [vue(2027, { series: [serie("a", "tomate"), serie("b", "laitue", { troncon_debut: 4, troncon_fin: 6 })], creneaux: [creneau] })],
      2027,
    );
    expect(lignes).toHaveLength(1);
    const l = lignes[0]!;
    expect(l.barres).toHaveLength(2);
    // The two bars overlap in time: two sub-rows. The slot starts after both.
    expect(l.voies).toBe(2);
    expect(l.creneaux[0]!.voie).toBe(0);
    expect(dateDe(l.creneaux[0]!.fin)).toBe("2027-11-16");
  });

  it("shows a serie that straddles two years once, and greys a missing planche", () => {
    const poireau = serie("p", "poireau", { planting_out: "2027-07-10", end: "2028-02-05", harvest_start: "2027-11-07" });
    const lignes = construireLignes([vue(2027, { series: [poireau] }), vue(2028, { series: [poireau], planches: [] })], 2027);
    expect(lignes[0]!.barres).toHaveLength(1);
    expect(lignes[0]!.absences).toEqual([{ debut: jourDe("2028-01-01"), fin: jourDe("2029-01-01") }]);
  });

  it("orders the rows by position on the plan", () => {
    const b: PlancheView = { ...PLANCHE, id: "p2", nom: "B", x: 10, y: 50 };
    const c: PlancheView = { ...PLANCHE, id: "p3", nom: "C", x: 0, y: 10 };
    const lignes = construireLignes([vue(2027, { planches: [PLANCHE, b, c] })], 2027);
    expect(lignes.map((l) => l.planche.id)).toEqual(["p3", "p2", "p1"]);
  });

  it("marks the series Rust reports in conflict", () => {
    const lignes = construireLignes(
      [vue(2027, { series: [serie("a", "tomate")], conflicts: [{ a: "a", b: "z", troncon_debut: 1, troncon_fin: 1, start: "2027-06-01", end: "2027-06-02" }] })],
      2027,
    );
    expect(lignes[0]!.barres[0]!.enConflit).toBe(true);
  });
});
