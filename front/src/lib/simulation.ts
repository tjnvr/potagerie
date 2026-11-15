// In-memory fake of the Rust commands, for `pnpm dev` in a plain browser only
// (main.ts never loads it in a build or inside Tauri). It is NOT the source of
// truth: it only lets the interface be laid out, looked at and driven. It
// copies just enough of the placement refusals of S03 §15 to see the
// « position refused » path; the real rules are Rust's, tested there.
// `?exemple` starts with a terrain, 8 planches and a few series.
import { mockIPC } from "@tauri-apps/api/mocks";
import type { PlancheView, SerieView, TerrainView, TypePlanche, YearView } from "./bindings";

type Args = Record<string, unknown>;
type Geometrie = { planche_type: TypePlanche; x: number; y: number; longueur_cm: number; largeur_cm: number };

const TAILLES: Record<TypePlanche, [number, number]> = {
  planche: [300, 120],
  carre: [100, 100],
  butte: [300, 120],
  bac: [200, 120],
};

function refus(code: string): never {
  throw { code, detail: "simulated" };
}

export function simulerTauri(): void {
  const maintenant = new Date();
  const annee0 = maintenant.getFullYear();
  const aujourdhui = maintenant.toISOString().slice(0, 10);
  let terrain: TerrainView | null = null;
  let planches: PlancheView[] = [];
  let series: SerieView[] = [];
  let historique: SerieView[] = [];
  let version = 0;

  function planche(nom: string, type: TypePlanche, x: number, y: number, l: number, h: number): PlancheView {
    return {
      id: crypto.randomUUID(),
      nom,
      planche_type: type,
      x,
      y,
      longueur_cm: l,
      largeur_cm: h,
      troncons: l / 50,
      series_supprimees: [],
    };
  }

  function serie(
    p: PlancheView,
    culture: string,
    t1: number,
    t2: number,
    mode: SerieView["mode"],
    date: string,
    fin: string,
    rangs: number,
    pieds: number,
  ): SerieView {
    return {
      id: crypto.randomUUID(),
      culture_id: culture,
      planche_id: p.id,
      troncon_debut: t1,
      troncon_fin: t2,
      mode,
      date,
      sowing: date,
      planting_out: date,
      harvest_start: date,
      end: fin,
      manual_end: false,
      rangs,
      pieds_par_rang: pieds,
      appearance_today: { appearance: "nothing" },
    };
  }

  if (new URLSearchParams(location.search).has("exemple")) {
    terrain = { longueur_cm: 1100, largeur_cm: 650, case_cm: 50, allee_cm: 50 };
    const racines = planche("Racines", "planche", 400, 50, 300, 120);
    planches = [
      planche("Fond du jardin", "planche", 50, 50, 300, 120),
      racines,
      planche("Les tomates", "planche", 50, 220, 400, 100),
      planche("Côté cabane", "planche", 500, 220, 300, 120),
      planche("Les courgettes", "planche", 50, 370, 300, 120),
      planche("Carré poireaux", "carre", 400, 370, 100, 100),
      planche("Carré salades", "carre", 550, 390, 100, 100),
      planche("Au bout de l’allée", "butte", 750, 50, 300, 120),
    ];
    const Y = annee0;
    const carotte = serie(racines, "carotte", 1, 3, "pleine_terre", `${Y}-03-15`, `${Y}-08-02`, 4, 37);
    const poireau = serie(racines, "poireau", 4, 6, "abri", `${Y}-05-01`, `${Y + 1}-02-05`, 4, 12);
    series = [carotte, poireau];
    historique = [serie(racines, "carotte", 1, 3, "pleine_terre", `${Y - 1}-03-15`, `${Y - 1}-08-02`, 4, 37)];
    racines.series_supprimees = [poireau.id];
    version = 4;
  }

  /** Overlap, allée and terrain edges: a rough copy of S03 §15, for the demo. */
  function verifier(id: string, g: Geometrie): void {
    const t = terrain;
    if (!t) refus("terrain_absent");
    if (g.x + g.longueur_cm > t.longueur_cm || g.y + g.largeur_cm > t.largeur_cm) {
      refus("planche_hors_terrain");
    }
    for (const autre of planches) {
      if (autre.id === id) continue;
      const dx = Math.max(autre.x - (g.x + g.longueur_cm), g.x - (autre.x + autre.longueur_cm));
      const dy = Math.max(autre.y - (g.y + g.largeur_cm), g.y - (autre.y + autre.largeur_cm));
      if (dx < 0 && dy < 0) refus("planche_chevauche");
      if (dx < t.allee_cm && dy < t.allee_cm) refus("allee_trop_etroite");
    }
  }

  function vue(annee: number): YearView {
    return {
      year: annee,
      today: aujourdhui,
      read_only: annee < annee0,
      terrain,
      version,
      updated_at: terrain ? new Date().toISOString() : null,
      planches,
      series,
      historique: annee > annee0 ? [...historique, ...series.filter((s) => s.end < `${annee}-01-02`)] : historique,
      conflicts: [],
      creneaux: [],
      annotations: [],
      tasks: [],
    };
  }

  function modifiable(annee: number): void {
    if (annee < annee0) refus("annee_passee");
  }

  mockIPC((cmd, payload) => {
    const args = (payload ?? {}) as Args;
    const annee = Number(args.annee ?? annee0);
    switch (cmd) {
      case "vue_annee":
        return vue(annee);
      case "creer_terrain": {
        const t = args.terrain as TerrainView;
        if (t.longueur_cm < 100 || t.largeur_cm < 100 || t.longueur_cm > 10000 || t.largeur_cm > 10000) {
          refus("terrain_invalide");
        }
        terrain = t;
        version = 1;
        return vue(annee);
      }
      case "modifier_terrain":
        terrain = { ...terrain!, ...(args.terrain as Partial<TerrainView>) };
        version++;
        return vue(annee);
      case "creer_planche": {
        modifiable(annee);
        const type = args.plancheType as TypePlanche;
        const [l, h] = TAILLES[type];
        const nouvelle = planche(String(args.nom), type, Number(args.x), Number(args.y), l, h);
        verifier(nouvelle.id, nouvelle);
        planches = [...planches, nouvelle];
        version++;
        return vue(annee);
      }
      case "deplacer_planche": {
        modifiable(annee);
        const cible = planches.find((p) => p.id === args.id)!;
        verifier(cible.id, { ...cible, x: Number(args.x), y: Number(args.y) });
        planches = planches.map((p) => (p === cible ? { ...p, x: Number(args.x), y: Number(args.y) } : p));
        version++;
        return vue(annee);
      }
      case "modifier_planche": {
        modifiable(annee);
        const cible = planches.find((p) => p.id === args.id)!;
        const g = args.geometrie as Geometrie | null;
        if (g) {
          if (g.longueur_cm % terrain!.case_cm !== 0) refus("longueur_invalide");
          verifier(cible.id, g);
        }
        planches = planches.map((p) =>
          p === cible
            ? {
                ...p,
                nom: (args.nom as string | null) ?? p.nom,
                ...(g ? { ...g, troncons: g.longueur_cm / terrain!.case_cm } : {}),
              }
            : p,
        );
        version++;
        return vue(annee);
      }
      case "series_depassant": {
        const n = Number(args.longueurCm) / terrain!.case_cm;
        return series.filter((s) => s.planche_id === args.id && s.troncon_fin > n).map((s) => s.id);
      }
      case "supprimer_planche":
        modifiable(annee);
        planches = planches.filter((p) => p.id !== args.id);
        series = series.filter((s) => s.planche_id !== args.id);
        version++;
        return vue(annee);
      case "effacer_donnees":
        terrain = null;
        planches = [];
        series = [];
        historique = [];
        version = 0;
        return null;
      default:
        throw { code: "fichier", detail: `${cmd} is not simulated` };
    }
  });
}
