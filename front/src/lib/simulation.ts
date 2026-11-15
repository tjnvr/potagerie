// In-memory fake of the Rust commands, for `pnpm dev` in a plain browser only
// (main.ts never loads it in a build or inside Tauri). It is NOT the source of
// truth: it only lets the interface be laid out, looked at and driven. It
// copies just enough of the placement refusals of S03 §15 to see the
// « position refused » path; the real rules are Rust's, tested there.
// `?exemple` starts with a terrain, 8 planches and a few series.
import { mockIPC } from "@tauri-apps/api/mocks";
import type { ErrorCode, PlancheView, SerieInput, SerieView, TerrainView, TypePlanche, YearView } from "./bindings";
import { jourDe, premierJanvier } from "./dates";
import { NON_GERE, creerSerie, creneaux, gerer, type Contexte } from "./simulation.series";

type Args = Record<string, unknown>;
type Geometrie = { planche_type: TypePlanche; x: number; y: number; longueur_cm: number; largeur_cm: number };

const TAILLES: Record<TypePlanche, [number, number]> = {
  planche: [300, 120],
  carre: [100, 100],
  butte: [300, 120],
  bac: [200, 120],
};

function refus(code: ErrorCode): never {
  throw { code, detail: "simulated" };
}

export function simulerTauri(): void {
  const parametres = new URLSearchParams(location.search);
  // `?date=2027-06-20` pretends today is that day (screenshots, D01 shows 20 juin 2027).
  const aujourdhui = parametres.get("date") ?? new Date().toISOString().slice(0, 10);
  const annee0 = Number(aujourdhui.slice(0, 4));
  let terrain: TerrainView | null = null;
  let planches: PlancheView[] = [];
  let series: SerieView[] = [];
  let historique: SerieView[] = [];
  let version = 0;

  const contexte: Contexte = {
    terrain: () => terrain,
    planches: () => planches,
    series: () => series,
    setSeries: (nouvelles) => (series = nouvelles),
    aujourdhui,
    annee0,
    valider: (annee) => {
      version++;
      return vue(annee);
    },
    refus,
  };

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

  if (new URLSearchParams(location.search).has("banc")) {
    // The bench of P01 for the frame rate: 10 planches, one or two crops each.
    terrain = { longueur_cm: 1200, largeur_cm: 650, case_cm: 50, allee_cm: 50 };
    const Y = annee0;
    planches = [50, 200, 350, 500].flatMap((y) => [
      planche(`Lit ${y}`, "planche", 50, y, 400, 100),
      planche(`Lit ${y} bis`, "planche", 550, y, 400, 100),
    ]);
    planches.push(planche("Bac A", "bac", 1050, 50, 100, 100), planche("Bac B", "bac", 1050, 350, 100, 100));
    const plan: [string, SerieInput["mode"], string][][] = [
      [["laitue", "pleine_terre", "03-01"], ["haricot_vert", "pleine_terre", "07-20"]],
      [["carotte", "pleine_terre", "03-15"], ["laitue", "pleine_terre", "09-01"]],
      [["tomate", "abri", "03-10"]],
      [["courgette", "abri", "04-15"]],
      [["poireau", "abri", "03-01"]],
      [["laitue", "pleine_terre", "03-10"], ["poireau", "plants", "09-01"]],
      [["haricot_vert", "pleine_terre", "05-10"], ["laitue", "pleine_terre", "09-10"]],
      [["carotte", "pleine_terre", "04-01"]],
      [["tomate", "plants", "05-05"]],
      [["laitue", "pleine_terre", "03-05"], ["laitue", "pleine_terre", "07-15"]],
    ];
    planches.forEach((p, i) => {
      for (const [culture_id, mode, date] of plan[i] ?? []) {
        series.push(
          creerSerie(contexte, { culture_id, planche_id: p.id, troncon_debut: 1, troncon_fin: p.troncons, mode, date: `${Y}-${date}`, abri_jours: null, fin_manuelle: null }),
        );
      }
    });
    version = 1;
  } else if (new URLSearchParams(location.search).has("exemple")) {
    terrain = { longueur_cm: 1100, largeur_cm: 650, case_cm: 50, allee_cm: 50 };
    planches = [
      planche("Fond du jardin", "planche", 50, 50, 300, 120),
      planche("Racines", "planche", 400, 50, 300, 120),
      planche("Les tomates", "planche", 50, 220, 400, 100),
      planche("Côté cabane", "planche", 500, 220, 300, 120),
      planche("Les courgettes", "planche", 50, 370, 300, 120),
      planche("Carré poireaux", "carre", 400, 370, 100, 100),
      planche("Carré salades", "carre", 550, 390, 100, 100),
      planche("Au bout de l’allée", "butte", 750, 50, 300, 120),
    ];
    const Y = annee0;
    const [fond, tomates, cabane, courgettes, racines, poireaux, salades] = [
      planches[0]!,
      planches[2]!,
      planches[3]!,
      planches[4]!,
      planches[1]!,
      planches[5]!,
      planches[6]!,
    ];
    const entree = (
      p: PlancheView,
      culture_id: string,
      troncon_debut: number,
      troncon_fin: number,
      mode: SerieInput["mode"],
      date: string,
    ): SerieInput => ({ culture_id, planche_id: p.id, troncon_debut, troncon_fin, mode, date, abri_jours: null, fin_manuelle: null });
    for (const e of [
      entree(fond, "laitue", 1, 6, "abri", `${Y}-03-01`),
      entree(tomates, "tomate", 1, 4, "abri", `${Y}-03-10`),
      entree(tomates, "laitue", 5, 8, "abri", `${Y}-04-12`),
      entree(courgettes, "courgette", 1, 6, "abri", `${Y}-04-27`),
      entree(racines, "carotte", 1, 3, "pleine_terre", `${Y}-03-15`),
      entree(racines, "laitue", 1, 3, "plants", `${Y}-08-05`),
      entree(racines, "poireau", 4, 6, "abri", `${Y}-05-01`),
      entree(cabane, "laitue", 1, 6, "abri", `${Y}-03-02`),
      entree(cabane, "haricot_vert", 1, 6, "pleine_terre", `${Y}-06-20`),
      entree(poireaux, "poireau", 1, 2, "abri", `${Y}-03-25`),
      entree(salades, "laitue", 1, 2, "plants", `${Y}-04-20`),
      entree(salades, "laitue", 1, 2, "abri", `${Y}-07-12`),
      entree(fond, "tomate", 1, 6, "plants", `${Y + 1}-05-10`),
      entree(racines, "carotte", 1, 3, "pleine_terre", `${Y + 1}-03-20`),
    ]) {
      series.push(creerSerie(contexte, e));
    }
    const passee = creerSerie(contexte, entree(racines, "carotte", 1, 3, "pleine_terre", `${Y - 1}-03-15`));
    series = series.filter((x) => x.id !== passee.id);
    historique = [passee];
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
    const debut = premierJanvier(annee);
    const touche = (x: SerieView) =>
      jourDe(x.sowing && x.mode === "abri" ? x.sowing : x.planting_out) < premierJanvier(annee + 1) && jourDe(x.end) > debut;
    const anciennes = [...historique, ...series].filter((x) => jourDe(x.end) <= debut);
    return {
      year: annee,
      today: aujourdhui,
      read_only: annee < annee0,
      terrain,
      version,
      updated_at: terrain ? new Date().toISOString() : null,
      planches: planches.map((p) => ({
        ...p,
        series_supprimees: series.filter((x) => x.planche_id === p.id && jourDe(x.end) > debut).map((x) => x.id),
      })),
      series: series.filter(touche),
      historique: anciennes,
      conflicts: [],
      creneaux: terrain && annee >= annee0 ? creneaux(contexte, annee) : [],
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
      default: {
        const reponse = gerer(contexte, cmd, args);
        if (reponse !== NON_GERE) return reponse;
        throw { code: "fichier", detail: `${cmd} is not simulated` };
      }
    }
  });
}
