// The series, slots and catalogue of the in-memory fake (see simulation.ts).
//
// Like the rest of the fake, it is NOT the source of truth: a rough copy of
// S02, S03 §2, §4, §5, §6, §8 and §9 so the timeline and the form of a serie
// can be laid out, looked at and driven in a plain browser. It does not copy
// the family tie-break of the suggestions. The real rules are Rust's
// (crates/plan), tested there. Dev only: main.ts never loads it in a build.
import type {
  Appearance,
  AppearanceChange,
  CreneauView,
  CultureView,
  DatesApercu,
  ErrorCode,
  Mode,
  PlancheView,
  ProblemeView,
  RaccourcisView,
  SerieApercu,
  SerieInput,
  SerieView,
  SuggestionView,
  TerrainView,
  ZoneView,
} from "./bindings";
import { dateDe, decalerDate, jourDe } from "./dates";

type Args = Record<string, unknown>;

/** S02 §3. */
export const CULTURES: CultureView[] = [
  { id: "tomate", famille: "Solanacées", abri_jours: 56, recolte_apres_jours: 75, recolte_duree_jours: 70, mise_en_place_debut: "05-01", mise_en_place_fin: "06-15", rang_cm: 80, pied_cm: 60 },
  { id: "laitue", famille: "Astéracées", abri_jours: 28, recolte_apres_jours: 50, recolte_duree_jours: 20, mise_en_place_debut: "03-01", mise_en_place_fin: "09-30", rang_cm: 30, pied_cm: 30 },
  { id: "carotte", famille: "Apiacées", abri_jours: 0, recolte_apres_jours: 100, recolte_duree_jours: 40, mise_en_place_debut: "03-01", mise_en_place_fin: "07-15", rang_cm: 25, pied_cm: 4 },
  { id: "haricot_vert", famille: "Fabacées", abri_jours: 0, recolte_apres_jours: 60, recolte_duree_jours: 30, mise_en_place_debut: "05-10", mise_en_place_fin: "07-31", rang_cm: 50, pied_cm: 10 },
  { id: "courgette", famille: "Cucurbitacées", abri_jours: 21, recolte_apres_jours: 50, recolte_duree_jours: 80, mise_en_place_debut: "05-01", mise_en_place_fin: "06-30", rang_cm: 100, pied_cm: 100 },
  { id: "poireau", famille: "Amaryllidacées", abri_jours: 70, recolte_apres_jours: 120, recolte_duree_jours: 90, mise_en_place_debut: "05-15", mise_en_place_fin: "07-31", rang_cm: 30, pied_cm: 12 },
];

export type Contexte = {
  terrain: () => TerrainView | null;
  planches: () => PlancheView[];
  series: () => SerieView[];
  setSeries: (series: SerieView[]) => void;
  aujourdhui: string;
  annee0: number;
  /** Bumps the version of the plan and returns the view of `annee`. */
  valider: (annee: number) => unknown;
  refus: (code: ErrorCode) => never;
};

export const NON_GERE = Symbol("non géré");

const culture = (id: string) => CULTURES.find((c) => c.id === id);
const jour = (iso: string) => jourDe(iso);
const iso = (n: number) => dateDe(n);

type Calcul = { sowing: string | null; m: string; r: string; f: string; fDefaut: string; stockee: string; godets: number };

/** S03 §2. Throws the code Rust would answer. */
function calculer(c: CultureView, entree: SerieInput, refus: (code: ErrorCode) => never): Calcul {
  const godets = c.abri_jours > 0 ? c.abri_jours : (entree.abri_jours ?? 0);
  if (entree.mode === "abri" && godets < 1) refus("pot_days_required");
  const A = entree.mode === "abri" ? godets : 0;
  const stockee = entree.mode === "abri" && entree.date_est_repiquage ? iso(jour(entree.date) - A) : entree.date;
  const s = jour(stockee);
  const m = entree.mode === "abri" ? s + A : s;
  const r = entree.mode === "pleine_terre" ? m + c.recolte_apres_jours + c.abri_jours : m + c.recolte_apres_jours;
  const fDefaut = r + c.recolte_duree_jours;
  const f = entree.fin_manuelle ? jour(entree.fin_manuelle) : fDefaut;
  if (entree.fin_manuelle && f <= r) refus("fin_manuelle_invalide");
  if (f <= m) refus("occupation_vide");
  return {
    sowing: entree.mode === "plants" ? null : stockee,
    m: iso(m),
    r: iso(r),
    f: iso(f),
    fDefaut: iso(fDefaut),
    stockee,
    godets: entree.mode === "abri" ? godets : c.abri_jours,
  };
}

function quantite(c: CultureView, p: PlancheView, terrain: TerrainView, k: number) {
  return {
    rangs: Math.max(1, Math.floor(p.largeur_cm / c.rang_cm)),
    pieds: Math.max(1, Math.floor((k * terrain.case_cm) / c.pied_cm)),
  };
}

/** What a serie looks like on a day, thresholds of S03 §3 (dev copy). */
function apparenceLe(d: string, sowing: string | null, m: string, r: string, f: string, mode: SerieView["mode"]): Appearance {
  if (d < m) return mode === "abri" && sowing !== null && d >= sowing ? { appearance: "label" } : { appearance: "nothing" };
  if (d >= f) return { appearance: "nothing" };
  const ecoule = (Date.parse(d) - Date.parse(m)) / 86_400_000;
  const duree = (Date.parse(r) - Date.parse(m)) / 86_400_000;
  const stage =
    duree <= 0 || ecoule >= duree ? "harvest" : ecoule * 100 < duree * 15 ? "sowing" : ecoule * 100 < duree * 50 ? "young_shoot" : "adult";
  return { appearance: "plant", stage: mode === "plants" && stage === "sowing" ? "young_shoot" : stage };
}

function changements(sowing: string | null, m: string, r: string, f: string, mode: SerieView["mode"]): AppearanceChange[] {
  const resultat: AppearanceChange[] = [];
  let precedent = "nothing:";
  for (let d = sowing && sowing < m ? sowing : m; d <= f; d = decalerDate(d, 1)) {
    const a = apparenceLe(d, sowing, m, r, f, mode);
    const cle = a.appearance === "plant" ? `plant:${a.stage}` : `${a.appearance}:`;
    if (cle !== precedent) {
      resultat.push({ from: d, appearance: a });
      precedent = cle;
    }
  }
  return resultat;
}

function enVue(id: string, e: SerieInput, c: CultureView, p: PlancheView, terrain: TerrainView, calcul: Calcul): SerieView {
  const q = quantite(c, p, terrain, e.troncon_fin - e.troncon_debut + 1);
  return {
    id,
    culture_id: e.culture_id,
    planche_id: e.planche_id,
    troncon_debut: e.troncon_debut,
    troncon_fin: e.troncon_fin,
    mode: e.mode,
    date: calcul.stockee,
    sowing: calcul.sowing,
    planting_out: calcul.m,
    harvest_start: calcul.r,
    end: calcul.f,
    manual_end: !!e.fin_manuelle,
    pot_days: calcul.godets,
    rangs: q.rangs,
    pieds_par_rang: q.pieds,
    appearance_today: { appearance: "nothing" },
    appearance_changes: changements(calcul.sowing, calcul.m, calcul.r, calcul.f, e.mode),
  };
}

type Occ = { z1: number; z2: number; m: number; f: number };

const occ = (s: Pick<SerieView, "troncon_debut" | "troncon_fin" | "planting_out" | "end">): Occ => ({
  z1: s.troncon_debut,
  z2: s.troncon_fin,
  m: jour(s.planting_out),
  f: jour(s.end),
});

/** Zones and occupations overlap (S03 §5). */
function enConflit(a: Occ, b: Occ): ZoneView & { start: string; end: string } | null {
  if (a.z1 > b.z2 || b.z1 > a.z2 || a.m >= b.f || b.m >= a.f) return null;
  return {
    troncon_debut: Math.max(a.z1, b.z1),
    troncon_fin: Math.min(a.z2, b.z2),
    start: iso(Math.max(a.m, b.m)),
    end: iso(Math.min(a.f, b.f)),
  };
}

function problemes(ctx: Contexte, ignore: string | null, e: SerieInput, calcul: Calcul): ProblemeView[] {
  const base = { avec: null, zone: null, start: null, end: null, annee: null, troncons: null };
  const planche = ctx.planches().find((p) => p.id === e.planche_id);
  if (!planche) return [{ ...base, code: "planche_inconnue" }];
  const liste: ProblemeView[] = [];
  if (e.troncon_fin > planche.troncons) {
    liste.push({
      ...base,
      code: "zone_hors_planche",
      zone: { troncon_debut: e.troncon_debut, troncon_fin: e.troncon_fin },
      annee: Number(calcul.m.slice(0, 4)),
      troncons: planche.troncons,
    });
  }
  const moi = { z1: e.troncon_debut, z2: e.troncon_fin, m: jour(calcul.m), f: jour(calcul.f) };
  for (const autre of ctx.series()) {
    if (autre.id === ignore || autre.planche_id !== e.planche_id) continue;
    const c = enConflit(moi, occ(autre));
    if (c) {
      liste.push({
        ...base,
        code: "conflict",
        avec: autre.id,
        zone: { troncon_debut: c.troncon_debut, troncon_fin: c.troncon_fin },
        start: c.start,
        end: c.end,
      });
    }
  }
  return liste;
}

/** The first run of `k` troncons free over the occupation (S03 §4). */
function raccourci(ctx: Contexte, planche: PlancheView, ignore: string | null, m: number, f: number, fraction: number): ZoneView | null {
  const n = planche.troncons;
  const k = Math.max(1, Math.floor(n * fraction + 0.5));
  const libre = (a: number, b: number) =>
    ctx
      .series()
      .filter((s) => s.id !== ignore && s.planche_id === planche.id)
      .every((s) => !(s.troncon_debut <= b && a <= s.troncon_fin && jour(s.planting_out) < f && m < jour(s.end)));
  for (let t1 = 1; t1 + k - 1 <= n; t1++) {
    if (libre(t1, t1 + k - 1)) return { troncon_debut: t1, troncon_fin: t1 + k - 1 };
  }
  return null;
}

function apercu(ctx: Contexte, ignore: string | null, e: SerieInput): SerieApercu {
  const vide: RaccourcisView = { toute: null, moitie: null, tiers: null, quart: null };
  const c = culture(e.culture_id);
  const planche = ctx.planches().find((p) => p.id === e.planche_id);
  const terrain = ctx.terrain();
  if (!c || !planche || !terrain) {
    return { dates: null, problemes: [{ code: "culture_inconnue", avec: null, zone: null, start: null, end: null, annee: null, troncons: null }], raccourcis: vide, occupes: [] };
  }
  let calcul: Calcul;
  try {
    calcul = calculer(c, e, ctx.refus);
  } catch (erreur) {
    const code = (erreur as { code: ErrorCode }).code;
    return { dates: null, problemes: [{ code, avec: null, zone: null, start: null, end: null, annee: null, troncons: null }], raccourcis: vide, occupes: [] };
  }
  const m = jour(calcul.m);
  const f = jour(calcul.f);
  const q = quantite(c, planche, terrain, e.troncon_fin - e.troncon_debut + 1);
  const dates: DatesApercu = {
    date: calcul.stockee,
    sowing: calcul.sowing,
    planting_out: calcul.m,
    harvest_start: calcul.r,
    end: calcul.f,
    default_end: calcul.fDefaut,
    manual_end: !!e.fin_manuelle,
    pot_days: calcul.godets,
    rangs: q.rangs,
    pieds_par_rang: q.pieds,
  };
  const occupes = ctx
    .series()
    .filter((s) => s.id !== ignore && s.planche_id === e.planche_id && jour(s.planting_out) < f && m < jour(s.end))
    .map((s) => ({ troncon_debut: s.troncon_debut, troncon_fin: s.troncon_fin }));
  return {
    dates,
    problemes: problemes(ctx, ignore, e, calcul),
    raccourcis: {
      toute: raccourci(ctx, planche, ignore, m, f, 1),
      moitie: raccourci(ctx, planche, ignore, m, f, 1 / 2),
      tiers: raccourci(ctx, planche, ignore, m, f, 1 / 3),
      quart: raccourci(ctx, planche, ignore, m, f, 1 / 4),
    },
    occupes,
  };
}

// ---------- Free slots and suggestions (S03 §8, §9) ----------

function suggestions(ctx: Contexte, debut: number, fin: number): SuggestionView[] {
  const N = fin - debut + 1;
  const aujourdhui = jour(ctx.aujourdhui);
  const annee = Number(iso(debut).slice(0, 4));
  const liste: (SuggestionView & { nom: string })[] = [];
  for (const c of CULTURES) {
    const [mo = 1, jo = 1] = c.mise_en_place_debut.split("-").map(Number);
    const [mf = 1, jf = 1] = c.mise_en_place_fin.split("-").map(Number);
    const fenetre = [annee - 1, annee].map((a) => [Date.UTC(a, mo - 1, jo) / 86_400_000, Date.UTC(a + (mo > mf ? 1 : 0), mf - 1, jf) / 86_400_000]);
    let m: number | null = null;
    for (let d = debut; d <= debut + 27 && m === null; d++) {
      if (fenetre.some(([a, b]) => a! <= d && d <= b!)) m = d;
    }
    if (m === null) continue;
    let mode: Mode = "pleine_terre";
    let semis = m;
    if (c.abri_jours > 0) {
      if (m - c.abri_jours >= aujourdhui) {
        mode = "abri";
        semis = m - c.abri_jours;
      } else {
        mode = "plants";
      }
    }
    const r = mode === "pleine_terre" ? m + c.recolte_apres_jours + c.abri_jours : m + c.recolte_apres_jours;
    if (r + Math.ceil(c.recolte_duree_jours / 2) > fin + 1) continue;
    const f = Math.min(fin + 1, r + c.recolte_duree_jours);
    liste.push({
      nom: c.id,
      culture_id: c.id,
      mode,
      date: iso(semis),
      planting_out: iso(m),
      harvest_start: iso(r),
      end: iso(f),
      fin_manuelle: f < r + c.recolte_duree_jours ? iso(f) : null,
      score_dixiemes: Math.round(((f - debut) / N) * 10),
    });
  }
  return liste
    .sort((a, b) => b.score_dixiemes - a.score_dixiemes || a.nom.localeCompare(b.nom))
    .slice(0, 3)
    .map(({ nom: _nom, ...s }) => s);
}

export function creneaux(ctx: Contexte, annee: number): CreneauView[] {
  const sortie: CreneauView[] = [];
  const debutFenetre = jour(`${annee}-03-01`);
  const finFenetre = jour(`${annee}-11-15`);
  for (const p of ctx.planches()) {
    const doubles = ctx.series().filter((s) => s.planche_id === p.id).map(occ);
    const coupes = new Set<number>([debutFenetre, finFenetre + 1]);
    for (const o of doubles) {
      for (const d of [o.m, o.f]) if (d > debutFenetre && d <= finFenetre) coupes.add(d);
    }
    const bornes = [...coupes].sort((a, b) => a - b);
    const segments = bornes.slice(0, -1).map((debut, i) => ({ debut, fin: bornes[i + 1]! }));
    const libres = (seg: { debut: number; fin: number }) =>
      Array.from({ length: p.troncons }, (_, i) => i + 1).map((t) => !doubles.some((o) => o.z1 <= t && t <= o.z2 && o.m < seg.fin && seg.debut < o.f));
    const candidats = new Map<string, { z1: number; z2: number; debut: number; fin: number }>();
    segments.forEach((seg, i) => {
      const l = libres(seg);
      for (let a = 0; a < l.length; a++) {
        if (!l[a] || (a > 0 && l[a - 1])) continue;
        let b = a;
        while (b + 1 < l.length && l[b + 1]) b++;
        const ok = (s: { debut: number; fin: number }) => libres(s).slice(a, b + 1).every(Boolean);
        let g = i;
        let d = i;
        while (g > 0 && ok(segments[g - 1]!)) g--;
        while (d + 1 < segments.length && ok(segments[d + 1]!)) d++;
        const c = { z1: a + 1, z2: b + 1, debut: segments[g]!.debut, fin: segments[d]!.fin };
        candidats.set(`${c.z1}-${c.z2}-${c.debut}-${c.fin}`, c);
      }
    });
    const gardes = [...candidats.values()]
      .filter((c) => c.fin - c.debut > 35)
      .filter((c, _i, tous) => !tous.some((o) => o !== c && o.z1 <= c.z1 && c.z2 <= o.z2 && o.debut <= c.debut && c.fin <= o.fin));
    for (const c of gardes) {
      sortie.push({
        planche_id: p.id,
        troncon_debut: c.z1,
        troncon_fin: c.z2,
        start: iso(c.debut),
        end: iso(c.fin - 1),
        suggestions: suggestions(ctx, c.debut, c.fin - 1),
      });
    }
  }
  return sortie;
}

/** A serie as the form of the seed describes it; checked like a real one. */
export function creerSerie(ctx: Contexte, entree: SerieInput): SerieView {
  const c = culture(entree.culture_id);
  const p = ctx.planches().find((x) => x.id === entree.planche_id);
  const terrain = ctx.terrain();
  if (!c) ctx.refus("culture_inconnue");
  if (!p || !terrain) ctx.refus("planche_inconnue");
  const calcul = calculer(c, entree, ctx.refus);
  const [premier] = problemes(ctx, null, entree, calcul);
  if (premier) ctx.refus(premier.code);
  return enVue(crypto.randomUUID(), entree, c, p, terrain, calcul);
}

/** The commands of the series, the slots and the catalogue. */
export function gerer(ctx: Contexte, cmd: string, args: Args): unknown {
  const annee = Number(args.annee ?? ctx.annee0);
  switch (cmd) {
    case "catalogue":
      return CULTURES;
    case "cultures_de_saison": {
      const d = jour(String(args.date));
      const an = Number(iso(d).slice(0, 4));
      return CULTURES.filter((c) => {
        const [mo = 1, jo = 1] = c.mise_en_place_debut.split("-").map(Number);
        const [mf = 1, jf = 1] = c.mise_en_place_fin.split("-").map(Number);
        return [an - 1, an].some((a) => Date.UTC(a, mo - 1, jo) / 86_400_000 <= d && d <= Date.UTC(a + (mo > mf ? 1 : 0), mf - 1, jf) / 86_400_000);
      }).map((c) => c.id);
    }
    case "apercu_serie":
      return apercu(ctx, (args.id as string | null) ?? null, args.serie as SerieInput);
    case "creer_serie": {
      const nouvelle = creerSerie(ctx, args.serie as SerieInput);
      ctx.setSeries([...ctx.series(), nouvelle]);
      return ctx.valider(annee);
    }
    case "modifier_serie": {
      const id = String(args.id);
      const entree = args.serie as SerieInput;
      const c = culture(entree.culture_id);
      const p = ctx.planches().find((x) => x.id === entree.planche_id);
      const terrain = ctx.terrain();
      if (!ctx.series().some((s) => s.id === id)) ctx.refus("serie_inconnue");
      if (!c) ctx.refus("culture_inconnue");
      if (!p || !terrain) ctx.refus("planche_inconnue");
      const calcul = calculer(c, entree, ctx.refus);
      const [premier] = problemes(ctx, id, entree, calcul);
      if (premier) ctx.refus(premier.code);
      ctx.setSeries(ctx.series().map((s) => (s.id === id ? enVue(id, entree, c, p, terrain, calcul) : s)));
      return ctx.valider(annee);
    }
    case "decaler_serie": {
      const id = String(args.id);
      const jours = Number(args.jours);
      const s = ctx.series().find((x) => x.id === id);
      if (!s) ctx.refus("serie_inconnue");
      const entree: SerieInput = {
        culture_id: s.culture_id,
        planche_id: s.planche_id,
        troncon_debut: s.troncon_debut,
        troncon_fin: s.troncon_fin,
        mode: s.mode,
        date: iso(jour_(s.date) + jours),
        abri_jours: s.pot_days,
        fin_manuelle: s.manual_end ? iso(jour_(s.end) + jours) : null,
      };
      const c = culture(s.culture_id)!;
      const p = ctx.planches().find((x) => x.id === s.planche_id)!;
      const calcul = calculer(c, entree, ctx.refus);
      const [premier] = problemes(ctx, id, entree, calcul);
      if (premier) ctx.refus(premier.code);
      ctx.setSeries(ctx.series().map((x) => (x.id === id ? enVue(id, entree, c, p, ctx.terrain()!, calcul) : x)));
      return ctx.valider(annee);
    }
    case "supprimer_serie": {
      if (!ctx.series().some((s) => s.id === args.id)) ctx.refus("serie_inconnue");
      ctx.setSeries(ctx.series().filter((s) => s.id !== args.id));
      return ctx.valider(annee);
    }
    default:
      return NON_GERE;
  }
}

const jour_ = jour;
