// In-memory fake of the Rust commands, for `pnpm dev` in a plain browser only
// (main.ts never loads it in a build or inside Tauri). It has no rule: it only
// lets the interface be laid out and looked at. `?exemple` starts with a
// terrain and three planches.
import { mockIPC } from "@tauri-apps/api/mocks";
import type { PlancheView, TerrainView, TypePlanche, YearView } from "./bindings";

type Args = Record<string, unknown>;

export function simulerTauri(): void {
  const aujourdhui = new Date().toISOString().slice(0, 10);
  let terrain: TerrainView | null = null;
  let planches: PlancheView[] = [];
  let version = 0;

  if (new URLSearchParams(location.search).has("exemple")) {
    terrain = { longueur_cm: 1100, largeur_cm: 650, case_cm: 50, allee_cm: 50 };
    planches = [
      planche("Les tomates", "planche", 50, 50, 300, 120),
      planche("Côté cabane", "planche", 400, 50, 300, 120),
      planche("Carré salades", "carre", 750, 50, 100, 100),
    ];
    version = 4;
  }

  function planche(nom: string, type: TypePlanche, x: number, y: number, l: number, h: number): PlancheView {
    return { id: crypto.randomUUID(), nom, planche_type: type, x, y, longueur_cm: l, largeur_cm: h, troncons: l / 50 };
  }

  function vue(annee: number): YearView {
    return {
      year: annee,
      today: aujourdhui,
      read_only: annee < new Date().getFullYear(),
      terrain,
      version,
      updated_at: terrain ? new Date().toISOString() : null,
      planches,
      series: [],
      conflicts: [],
      creneaux: [],
      annotations: [],
      tasks: [],
    };
  }

  const tailles: Record<TypePlanche, [number, number]> = {
    planche: [300, 120],
    carre: [100, 100],
    butte: [300, 120],
    bac: [200, 120],
  };

  mockIPC((cmd, payload) => {
    const args = (payload ?? {}) as Args;
    const annee = Number(args.annee ?? new Date().getFullYear());
    switch (cmd) {
      case "vue_annee":
        return vue(annee);
      case "creer_terrain":
        terrain = args.terrain as TerrainView;
        version = 1;
        return vue(annee);
      case "modifier_terrain":
        terrain = { ...terrain!, ...(args.terrain as Partial<TerrainView>) };
        version++;
        return vue(annee);
      case "creer_planche": {
        const type = args.plancheType as TypePlanche;
        const [l, h] = tailles[type];
        planches = [...planches, planche(String(args.nom), type, Number(args.x), Number(args.y), l, h)];
        version++;
        return vue(annee);
      }
      case "effacer_donnees":
        terrain = null;
        planches = [];
        version = 0;
        return null;
      default:
        throw { code: "fichier", detail: `${cmd} is not simulated` };
    }
  });
}
