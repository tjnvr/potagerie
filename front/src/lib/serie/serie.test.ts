import { describe, expect, it } from "vitest";
import type { CultureView, SerieApercu, SerieView, SuggestionView } from "../bindings";
import {
  appliquerSuggestion,
  basculerRepiquage,
  finSaisie,
  formulaireDepuisSerie,
  formulaireVide,
  memeZone,
  modeInitial,
  premierRaccourciLibre,
  segmentsMiniFrise,
  versSerieInput,
  zoneApresClic,
} from "./serie";

const tomate = { id: "tomate", abri_jours: 56 } as CultureView;
const carotte = { id: "carotte", abri_jours: 0 } as CultureView;

describe("strip of troncons", () => {
  it("extends a single troncon to the one clicked, either side", () => {
    expect(zoneApresClic({ debut: 3, fin: 3 }, 5)).toEqual({ debut: 3, fin: 5 });
    expect(zoneApresClic({ debut: 3, fin: 3 }, 1)).toEqual({ debut: 1, fin: 3 });
  });

  it("starts again from a click when the zone has several troncons or is clicked again", () => {
    expect(zoneApresClic({ debut: 2, fin: 4 }, 6)).toEqual({ debut: 6, fin: 6 });
    expect(zoneApresClic({ debut: 3, fin: 3 }, 3)).toEqual({ debut: 3, fin: 3 });
  });

  it("compares zones", () => {
    expect(memeZone({ debut: 1, fin: 3 }, { debut: 1, fin: 3 })).toBe(true);
    expect(memeZone({ debut: 1, fin: 3 }, null)).toBe(false);
  });

  it("takes the first free shortcut, in order", () => {
    const z = { troncon_debut: 1, troncon_fin: 3 };
    expect(premierRaccourciLibre({ toute: null, moitie: z, tiers: null, quart: null })).toEqual({ debut: 1, fin: 3 });
    expect(premierRaccourciLibre({ toute: null, moitie: null, tiers: null, quart: null })).toBeNull();
  });
});

describe("form to command input", () => {
  it("starts in pots when the catalogue has a time in pots", () => {
    expect(modeInitial(tomate)).toBe("abri");
    expect(modeInitial(carotte)).toBe("pleine_terre");
    const f = formulaireVide(tomate, "p1", 6, "2027-03-10");
    expect(f.zone).toEqual({ debut: 1, fin: 6 });
    expect(versSerieInput(f)).toMatchObject({
      culture_id: "tomate",
      planche_id: "p1",
      troncon_debut: 1,
      troncon_fin: 6,
      mode: "abri",
      date: "2027-03-10",
      fin_manuelle: null,
      date_est_repiquage: false,
    });
  });

  it("sends the days in pots only in the mode abri", () => {
    const f = { ...formulaireVide(carotte, "p1", 6, "2027-03-01"), mode: "abri" as const, abriJours: "21" };
    expect(versSerieInput(f).abri_jours).toBe(21);
    expect(versSerieInput({ ...f, mode: "plants" }).abri_jours).toBeNull();
    expect(versSerieInput({ ...f, abriJours: "" }).abri_jours).toBeNull();
    expect(versSerieInput({ ...f, repiquage: true }).date_est_repiquage).toBe(true);
    expect(versSerieInput({ ...f, mode: "plants", repiquage: true }).date_est_repiquage).toBe(false);
  });

  it("fills the form from a suggestion, with its end as an end modified by hand", () => {
    const s: SuggestionView = {
      culture_id: "laitue",
      mode: "plants",
      date: "2027-08-02",
      planting_out: "2027-08-02",
      harvest_start: "2027-09-21",
      end: "2027-10-06",
      fin_manuelle: "2027-10-06",
      score_dixiemes: 10,
    };
    const f = appliquerSuggestion(formulaireVide(tomate, "p1", 6, "2027-01-01"), s, { debut: 1, fin: 6 });
    expect(f).toMatchObject({ cultureId: "laitue", mode: "plants", date: "2027-08-02", finManuelle: "2027-10-06" });
  });

  it("reads a serie back for editing", () => {
    const serie = {
      culture_id: "tomate",
      planche_id: "p1",
      troncon_debut: 2,
      troncon_fin: 4,
      mode: "abri",
      date: "2027-03-10",
      end: "2027-09-27",
      manual_end: false,
      pot_days: 56,
    } as SerieView;
    expect(formulaireDepuisSerie(serie)).toMatchObject({ zone: { debut: 2, fin: 4 }, finManuelle: null, abriJours: "56" });
    expect(formulaireDepuisSerie({ ...serie, manual_end: true, end: "2027-09-01" }).finManuelle).toBe("2027-09-01");
  });

  it("swaps the sowing and transplanting dates with what Rust calculated", () => {
    const apercu = { dates: { date: "2027-03-01", planting_out: "2027-03-22" } } as SerieApercu;
    const f = { ...formulaireVide(tomate, "p1", 6, "2027-03-01"), mode: "abri" as const };
    const r = basculerRepiquage(f, apercu);
    expect(r).toMatchObject({ repiquage: true, date: "2027-03-22" });
    expect(basculerRepiquage(r, apercu)).toMatchObject({ repiquage: false, date: "2027-03-01" });
    // Without a preview the date stays.
    expect(basculerRepiquage(f, null)).toMatchObject({ repiquage: true, date: "2027-03-01" });
  });

  it("does not call the end « by hand » when it is the calculated one", () => {
    expect(finSaisie("2027-10-11", "2027-10-11")).toBeNull();
    expect(finSaisie("2027-10-06", "2027-10-11")).toBe("2027-10-06");
    expect(finSaisie("", "2027-10-11")).toBeNull();
  });

  it("cuts the mini-frise from the dates of Rust", () => {
    const dates = {
      sowing: "2027-03-10",
      planting_out: "2027-05-05",
      harvest_start: "2027-07-19",
      end: "2027-09-27",
    } as NonNullable<SerieApercu["dates"]>;
    expect(segmentsMiniFrise(dates, "abri").map((s) => s.phase)).toEqual(["godets", "croissance", "recolte"]);
    expect(segmentsMiniFrise(dates, "abri").map((s) => s.jours)).toEqual([56, 75, 70]);
    expect(segmentsMiniFrise(dates, "plants").map((s) => s.phase)).toEqual(["croissance", "recolte"]);
  });
});
