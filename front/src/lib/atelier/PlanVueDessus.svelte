<script lang="ts">
  // Top view of the terrain and its planches (E03), in centimetres: selection,
  // drag with Pointer Events (mouse, pen, finger after a long press), the 8
  // handles. The planche follows the pointer; Rust judges at the drop and the
  // planche goes back to its place if it refuses (S05 §4).
  import type { PlancheView, TerrainView, TypePlanche } from "../bindings";
  import { m } from "../paraglide/messages.js";
  import { enMetres } from "../planches";
  import {
    ControleurGlisser,
    deposer,
    pixelsVersCm,
    positionDeposee,
    redimensionner,
    type Boite,
    type Geo,
    type Point,
    type Poignee,
    type Reponse,
    type TypePointeur,
  } from "./glisser";

  /** The shadow of a planche carried from the tray. */
  export type Fantome = Geo & { type: TypePlanche };

  let {
    terrain,
    planches,
    selectionId = null,
    lectureSeule = false,
    fantome = null,
    onselect = () => {},
    onmodifier = async () => "ok" as Reponse,
    onavantterrain = () => {},
  }: {
    terrain: TerrainView;
    planches: PlancheView[];
    selectionId?: string | null;
    lectureSeule?: boolean;
    fantome?: Fantome | null;
    onselect?: (id: string | null) => void;
    /** Move or resize: the new geometry of the planche, judged by Rust. */
    onmodifier?: (id: string, geo: Geo, nature: "deplacer" | "redimensionner") => Promise<Reponse>;
    /** The drop is left of or above the terrain: nothing to send, refused. */
    onavantterrain?: () => void;
  } = $props();

  type Geste = {
    id: string;
    nature: "deplacer" | "redimensionner";
    origine: Geo;
    decalage: Point;
    poignee?: Poignee;
    apercu: Geo;
    envoi: boolean;
  };

  const POIGNEES: { nom: Poignee; fx: number; fy: number; curseur: string }[] = [
    { nom: "nw", fx: 0, fy: 0, curseur: "nwse-resize" },
    { nom: "n", fx: 0.5, fy: 0, curseur: "ns-resize" },
    { nom: "ne", fx: 1, fy: 0, curseur: "nesw-resize" },
    { nom: "e", fx: 1, fy: 0.5, curseur: "ew-resize" },
    { nom: "se", fx: 1, fy: 1, curseur: "nwse-resize" },
    { nom: "s", fx: 0.5, fy: 1, curseur: "ns-resize" },
    { nom: "sw", fx: 0, fy: 1, curseur: "nesw-resize" },
    { nom: "w", fx: 0, fy: 0.5, curseur: "ew-resize" },
  ];

  let svg: SVGSVGElement;
  let largeurPx = $state(600);
  let geste = $state<Geste | null>(null);
  let refuse = $state<string | null>(null);
  let arme = $state<string | null>(null);

  const marge = $derived(Math.max(terrain.longueur_cm, terrain.largeur_cm) * 0.02);
  const police = $derived(Math.max(terrain.longueur_cm, terrain.largeur_cm) / 60);
  const boite = $derived<Boite>({
    x: -marge,
    y: -marge,
    largeur: terrain.longueur_cm + 2 * marge,
    hauteur: terrain.largeur_cm + 2 * marge,
  });
  /** Centimetres per screen pixel, to keep handles the same size on screen. */
  const echelle = $derived(boite.largeur / Math.max(largeurPx, 1));
  const lignesX = $derived(
    Array.from({ length: Math.floor(terrain.longueur_cm / terrain.case_cm) - 1 }, (_, i) => (i + 1) * terrain.case_cm),
  );
  const lignesY = $derived(
    Array.from({ length: Math.floor(terrain.largeur_cm / terrain.case_cm) - 1 }, (_, i) => (i + 1) * terrain.case_cm),
  );
  /** The planche in hand is drawn last, above the others. */
  const ordonnees = $derived([...planches].sort((a, b) => Number(a.id === geste?.id) - Number(b.id === geste?.id)));
  const actif = $derived(planches.find((p) => p.id === selectionId) ?? null);
  const pasDuMetre = $derived(100 / terrain.case_cm);

  const controleur = new ControleurGlisser({ surArme: () => (arme = geste?.id ?? null) });

  /** Pixels of the screen to centimetres of the plan, or `null` outside the plan. */
  export function versCm(clientX: number, clientY: number): Point | null {
    const rect = svg.getBoundingClientRect();
    if (clientX < rect.left || clientX > rect.right || clientY < rect.top || clientY > rect.bottom) {
      return null;
    }
    return pixelsVersCm({ x: clientX, y: clientY }, rect, boite);
  }

  function enCm(e: PointerEvent): Point {
    return pixelsVersCm({ x: e.clientX, y: e.clientY }, svg.getBoundingClientRect(), boite);
  }

  function geoDe(p: PlancheView): Geo {
    return { x: p.x, y: p.y, longueur_cm: p.longueur_cm, largeur_cm: p.largeur_cm };
  }

  function debut(e: PointerEvent, p: PlancheView, nature: Geste["nature"], poignee?: Poignee) {
    if (e.button !== 0 || geste?.envoi) return;
    e.stopPropagation();
    svg.setPointerCapture(e.pointerId);
    const origine = geoDe(p);
    const pointeur = enCm(e);
    geste = {
      id: p.id,
      nature,
      origine,
      decalage: { x: pointeur.x - p.x, y: pointeur.y - p.y },
      poignee,
      apercu: origine,
      envoi: false,
    };
    arme = null;
    controleur.appui(e.pointerType as TypePointeur, { x: e.clientX, y: e.clientY });
  }

  function bouge(e: PointerEvent) {
    if (!geste || geste.envoi) return;
    const mouvement = controleur.deplacer({ x: e.clientX, y: e.clientY });
    if (mouvement === "abandon") {
      geste = null;
      return;
    }
    if (mouvement === "debut" && selectionId !== geste.id) onselect(geste.id);
    if (mouvement === "debut" || mouvement === "glisse") {
      const pointeur = enCm(e);
      const pas = terrain.case_cm;
      geste.apercu =
        geste.nature === "deplacer"
          ? {
              ...geste.origine,
              ...positionDeposee(pointeur, geste.decalage, pas),
            }
          : redimensionner(geste.origine, geste.poignee!, pointeur, pas);
    }
  }

  async function leve(e: PointerEvent) {
    if (!geste || geste.envoi) return;
    if (svg.hasPointerCapture(e.pointerId)) svg.releasePointerCapture(e.pointerId);
    const fin = controleur.relacher();
    arme = null;
    const courant = geste;
    if (fin !== "fin") {
      geste = null;
      if (fin === "clic") onselect(courant.id);
      return;
    }
    courant.envoi = true;
    const issue = await deposer(courant.origine, courant.apercu, (g) =>
      onmodifier(courant.id, g, courant.nature),
    );
    if (issue.reponse === "avant-terrain") onavantterrain();
    geste = null;
    if (issue.reponse === "refus" || issue.reponse === "avant-terrain") {
      refuse = courant.id;
      setTimeout(() => (refuse = null), 1200);
    }
  }

  function annule() {
    controleur.annuler();
    arme = null;
    if (!geste?.envoi) geste = null;
  }

  async function touche(e: KeyboardEvent, p: PlancheView) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onselect(p.id);
      return;
    }
    if (e.key === "Escape") {
      onselect(null);
      return;
    }
    const pas = terrain.case_cm;
    const delta: Record<string, Point> = {
      ArrowLeft: { x: -pas, y: 0 },
      ArrowRight: { x: pas, y: 0 },
      ArrowUp: { x: 0, y: -pas },
      ArrowDown: { x: 0, y: pas },
    };
    const d = delta[e.key];
    if (!d || lectureSeule || selectionId !== p.id) return;
    e.preventDefault();
    const cible = { ...geoDe(p), x: p.x + d.x, y: p.y + d.y };
    await deposer(geoDe(p), cible, (g) => onmodifier(p.id, g, "deplacer")).then((issue) => {
      if (issue.reponse === "avant-terrain") onavantterrain();
    });
  }

  // A finger that holds a planche must not scroll the page: `touch-action`
  // is not honoured on every SVG engine, so the touchmove is stopped too.
  $effect(() => {
    const arreter = (e: TouchEvent) => {
      if (controleur.etat === "arme" || controleur.etat === "glisse") e.preventDefault();
    };
    svg.addEventListener("touchmove", arreter, { passive: false });
    return () => svg.removeEventListener("touchmove", arreter);
  });
</script>

<svg
  bind:this={svg}
  bind:clientWidth={largeurPx}
  class="plan"
  viewBox="{boite.x} {boite.y} {boite.largeur} {boite.hauteur}"
  role="group"
  aria-label={m.plan_aria({ nombre: planches.length })}
  onpointermove={bouge}
  onpointerup={leve}
  onpointercancel={annule}
>
  <rect
    class="herbe"
    width={terrain.longueur_cm}
    height={terrain.largeur_cm}
    rx={terrain.case_cm / 3}
    fill="var(--grass)"
    role="presentation"
    onpointerdown={() => !geste && onselect(null)}
  />
  {#each lignesX as x, i (x)}
    <line x1={x} x2={x} y1="0" y2={terrain.largeur_cm} class="grille" class:metre={(i + 1) % pasDuMetre === 0} />
  {/each}
  {#each lignesY as y, i (y)}
    <line y1={y} y2={y} x1="0" x2={terrain.longueur_cm} class="grille" class:metre={(i + 1) % pasDuMetre === 0} />
  {/each}

  {#each ordonnees as planche (planche.id)}
    {@const g = geste?.id === planche.id ? geste.apercu : geoDe(planche)}
    {@const bois = planche.planche_type === "bac" || planche.planche_type === "carre"}
    {@const choisie = selectionId === planche.id}
    {#if geste?.id === planche.id && geste.nature === "deplacer"}
      <rect
        class="origine"
        x={geste.origine.x}
        y={geste.origine.y}
        width={geste.origine.longueur_cm}
        height={geste.origine.largeur_cm}
        rx="6"
      />
    {/if}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <g
      class="planche"
      class:choisie
      class:refuse={refuse === planche.id}
      class:arme={arme === planche.id}
      class:libre={lectureSeule}
      role="button"
      tabindex="0"
      aria-pressed={choisie}
      aria-label={m.planche_aria({
        nom: planche.nom,
        longueur: enMetres(g.longueur_cm),
        largeur: enMetres(g.largeur_cm),
      })}
      onpointerdown={(e) => (lectureSeule ? onselect(planche.id) : debut(e, planche, "deplacer"))}
      onkeydown={(e) => touche(e, planche)}
    >
      <rect x={g.x} y={g.y} width={g.longueur_cm} height={g.largeur_cm} rx="6" fill={bois ? "var(--wood)" : "var(--bed)"} />
      {#if bois}
        <rect x={g.x + 6} y={g.y + 6} width={Math.max(g.longueur_cm - 12, 0)} height={Math.max(g.largeur_cm - 12, 0)} rx="3" fill="var(--bed)" />
      {:else}
        {#each Array.from({ length: Math.max(Math.floor(g.largeur_cm / 25) - 1, 0) }, (_, k) => k + 1) as k (k)}
          <line x1={g.x + 8} x2={g.x + g.longueur_cm - 8} y1={g.y + k * 25} y2={g.y + k * 25} class="sillon" />
        {/each}
      {/if}
      {#if refuse === planche.id}
        <rect class="alerte" x={g.x} y={g.y} width={g.longueur_cm} height={g.largeur_cm} rx="6" />
      {/if}
      <text class="etiquette" x={g.x + 10} y={g.y + 10 + police} font-size={police}>{planche.nom}</text>
    </g>
  {/each}

  {#if actif}
    {@const g = geste?.id === actif.id ? geste.apercu : geoDe(actif)}
      <rect class="cadre" x={g.x} y={g.y} width={g.longueur_cm} height={g.largeur_cm} rx="6" />
      <text class="cote" x={g.x + g.longueur_cm / 2} y={g.y - police * 0.5} font-size={police} text-anchor="middle">
        {m.panneau_dimensions({ longueur: enMetres(g.longueur_cm), largeur: enMetres(g.largeur_cm) })}
      </text>
      {#if !lectureSeule}
        {#each POIGNEES as h (h.nom)}
          <circle
            class="poignee-zone"
            role="presentation"
            cx={g.x + h.fx * g.longueur_cm}
            cy={g.y + h.fy * g.largeur_cm}
            r={18 * echelle}
            style:cursor={h.curseur}
            aria-label={m.poignee_aria()}
            onpointerdown={(e) => debut(e, actif, "redimensionner", h.nom)}
          />
          <circle
            class="poignee"
            cx={g.x + h.fx * g.longueur_cm}
            cy={g.y + h.fy * g.largeur_cm}
            r={6 * echelle}
          />
        {/each}
      {/if}
    {/if}

  {#if fantome}
    <rect
      class="fantome"
      x={fantome.x}
      y={fantome.y}
      width={fantome.longueur_cm}
      height={fantome.largeur_cm}
      rx="6"
    />
  {/if}
</svg>

<style>
  .plan {
    display: block;
    width: 100%;
    height: auto;
    max-height: 70vh;
    user-select: none;
    -webkit-user-select: none;
    -webkit-touch-callout: none;
  }

  .grille {
    stroke: #ffffff;
    stroke-opacity: 0.25;
    stroke-width: 1;
    vector-effect: non-scaling-stroke;
    pointer-events: none;
  }

  .grille.metre {
    stroke-opacity: 0.55;
  }

  .planche {
    cursor: grab;
    touch-action: none;
  }

  .planche.libre {
    cursor: pointer;
    touch-action: auto;
  }

  .planche.arme {
    filter: drop-shadow(0 0 6px rgba(242, 166, 90, 0.9));
  }

  .planche:focus-visible {
    outline: none;
  }

  .planche:focus-visible rect:first-child {
    stroke: var(--accent);
    stroke-width: 3px;
    vector-effect: non-scaling-stroke;
  }

  .sillon {
    stroke: var(--bed-furrow);
    stroke-width: 1.5;
    stroke-linecap: round;
    vector-effect: non-scaling-stroke;
  }

  .etiquette {
    font-family: var(--font-titre);
    font-weight: 600;
    fill: #ffffff;
    paint-order: stroke;
    stroke: rgba(31, 58, 46, 0.55);
    stroke-width: 3px;
    stroke-linejoin: round;
    vector-effect: non-scaling-stroke;
    pointer-events: none;
  }

  .alerte {
    fill: rgba(200, 68, 60, 0.45);
    stroke: var(--crit);
    stroke-width: 2px;
    vector-effect: non-scaling-stroke;
  }

  .origine {
    fill: none;
    stroke: var(--white);
    stroke-width: 2px;
    stroke-dasharray: 6 4;
    vector-effect: non-scaling-stroke;
    pointer-events: none;
  }

  .fantome {
    fill: rgba(255, 255, 255, 0.35);
    stroke: var(--green);
    stroke-width: 2px;
    stroke-dasharray: 6 4;
    vector-effect: non-scaling-stroke;
    pointer-events: none;
  }

  .cadre {
    fill: none;
    stroke: var(--accent);
    stroke-width: 3px;
    vector-effect: non-scaling-stroke;
    pointer-events: none;
  }

  .cote {
    font-family: var(--font);
    font-weight: 700;
    fill: var(--ink);
    paint-order: stroke;
    stroke: rgba(255, 255, 255, 0.85);
    stroke-width: 3px;
    pointer-events: none;
  }

  .poignee {
    fill: var(--white);
    stroke: var(--accent);
    stroke-width: 2px;
    vector-effect: non-scaling-stroke;
    pointer-events: none;
  }

  .poignee-zone {
    fill: transparent;
    touch-action: none;
  }
</style>
