// @vitest-environment happy-dom
import { mount, unmount } from "svelte";
import { afterEach, describe, expect, it } from "vitest";
import type { PlancheView, SerieView } from "../bindings";
import { jourDe } from "../dates";
import { serialiserFrise } from "./exportSvg";
import FriseSvg from "./FriseSvg.svelte";
import { barreDe, creneauDe, type Ligne } from "./geometrie";

const planche: PlancheView = {
  id: "p1",
  nom: "Les tomates",
  planche_type: "planche",
  x: 0,
  y: 0,
  longueur_cm: 300,
  largeur_cm: 120,
  troncons: 6,
  series_supprimees: [],
};

const tomate = {
  id: "s1",
  culture_id: "tomate",
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
} as SerieView;

const lignes: Ligne[] = [
  {
    planche,
    barres: [{ ...barreDe(tomate), voie: 0 }],
    creneaux: [
      {
        ...creneauDe({
          planche_id: "p1",
          troncon_debut: 1,
          troncon_fin: 6,
          start: "2027-09-27",
          end: "2027-11-15",
          suggestions: [],
        }),
        voie: 0,
      },
    ],
    absences: [],
    voies: 1,
  },
];

let instance: ReturnType<typeof mount> | null = null;
afterEach(() => {
  if (instance) unmount(instance);
  instance = null;
  document.body.innerHTML = "";
});

function dessiner(): SVGSVGElement {
  instance = mount(FriseSvg, {
    target: document.body,
    props: {
      lignes,
      echelle: { debutJour: jourDe("2027-01-01"), pxParJour: 2.5, largeur: 912 },
      aujourdhui: "2027-06-20",
      curseur: "2027-06-20",
      interactif: true,
      anneeAffichee: 2027,
    },
  });
  return document.querySelector("svg[data-frise]") as SVGSVGElement;
}

describe("timeline export", () => {
  it("is a standalone SVG: no CSS, no style, no custom property, no external resource", () => {
    const texte = serialiserFrise(dessiner());
    expect(texte.startsWith('<?xml version="1.0" encoding="UTF-8"?>')).toBe(true);
    expect(texte).toContain('xmlns="http://www.w3.org/2000/svg"');
    expect(texte).not.toMatch(/<style/i);
    expect(texte).not.toMatch(/\sstyle=/);
    expect(texte).not.toMatch(/\sclass=/);
    expect(texte).not.toContain("var(--");
    expect(texte).not.toMatch(/https?:\/\/(?!www\.w3\.org)/);
    expect(texte).not.toContain("<link");
    expect(texte).not.toContain("@import");
    expect(texte).not.toMatch(/href=/);
  });

  it("names the embedded fonts and draws the three phases with distinct patterns", () => {
    const texte = serialiserFrise(dessiner());
    expect(texte).toContain('font-family="Figtree');
    expect(texte).toContain("Fredoka");
    expect(texte).toContain('id="frise-godets-tomate"');
    expect(texte).toContain('id="frise-croissance-tomate"');
    expect(texte).toContain('fill="url(#frise-godets-tomate)"');
    expect(texte).toContain('fill="url(#frise-croissance-tomate)"');
    expect(texte).toContain('fill="#E5533D"');
    expect(texte).toContain("Les tomates");
  });

  it("drops the cursor and the screen-only attributes, unless asked", () => {
    const svg = dessiner();
    expect(serialiserFrise(svg)).not.toContain("data-");
    expect(svg.querySelector("[data-curseur]")).not.toBeNull();
    const sans = serialiserFrise(svg);
    const avec = serialiserFrise(svg, { curseur: true });
    expect(avec.length).toBeGreaterThan(sans.length);
    // The live drawing is left untouched.
    expect(svg.querySelector("[data-barre]")).not.toBeNull();
  });

  it("parses back as XML", () => {
    const texte = serialiserFrise(dessiner());
    const doc = new DOMParser().parseFromString(texte, "image/svg+xml");
    expect(doc.documentElement.nodeName).toBe("svg");
    expect(doc.querySelectorAll("parsererror")).toHaveLength(0);
  });
});
