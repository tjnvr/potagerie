<script lang="ts">
  // The garden view of E05 (D01, S01 F-VIS-1 to F-VIS-4): the isometric
  // diorama in SVG, the time cursor under it, « Jouer la saison », and the
  // planche panel (E06) beside it. Stages and dates come from the Rust view;
  // the only calculation here is the animation (animation.ts).
  import { untrack } from "svelte";
  import type { YearView } from "../bindings";
  import Icone from "../coquille/Icone.svelte";
  import { m } from "../paraglide/messages.js";
  import { nomDeType } from "../planches";
  import { dateCurseur, lectureEnCours } from "../temps";
  import { apparenceAu, jourDeLecture, SYMBOLE_ETIQUETTE } from "./animation";
  import CompteurImages from "./CompteurImages.svelte";
  import Curseur from "./Curseur.svelte";
  import { ETIQUETTE_SVG } from "./etiquette";
  import { bornesAnnee, dateEnToutesLettres, isoDepuis, jourDepuis } from "./jours";
  import { imagesParSeconde, mesurer, type Mesure } from "./mesure";
  import PanneauDiorama from "./PanneauDiorama.svelte";
  import type { Quart } from "./projection";
  import { RenduPlantes } from "./rendu";
  import { PALETTES, saisonDe } from "./saison";
  import { construireScene, type PlancheScene } from "./scene";
  import { SYMBOLES, VIEWBOX_SYMBOLE } from "./symboles";

  let { vue }: { vue: YearView } = $props();

  /** Plants redrawn at most per frame while the season plays (see rendu.ts). */
  const PLAFOND_PAR_IMAGE = 50;

  // `?fps` in the address, in dev only: the frame counter of P01.
  const mesureActive = import.meta.env.DEV && new URLSearchParams(location.search).has("fps");

  const NOMS_SAISON = {
    hiver: m.saison_hiver,
    printemps: m.saison_printemps,
    ete: m.saison_ete,
    automne: m.saison_automne,
  };

  let quart = $state<Quart>(0);
  let selectionId = $state<string | null>(null);
  let vitesse = $state(1);
  let lecture = $state(false);
  let largeur = $state(900);
  /** The day of the cursor, whole; the playback moves a fraction of it in `jourExact`. */
  let jourEntier = $state(jourDepuis(untrack(() => $dateCurseur)));
  let jourExact = untrack(() => jourEntier);

  const reduitMq = window.matchMedia("(prefers-reduced-motion: reduce)");
  let reduit = $state(reduitMq.matches);
  $effect(() => {
    const suivre = () => (reduit = reduitMq.matches);
    reduitMq.addEventListener("change", suivre);
    return () => reduitMq.removeEventListener("change", suivre);
  });

  const scene = $derived(construireScene(vue, quart));
  const bornes = $derived(bornesAnnee(vue.year));
  const compact = $derived(largeur < 560);
  const palette = $derived(PALETTES[saisonDe(jourEntier)]);
  const saison = $derived(saisonDe(jourEntier));
  const selection = $derived(vue.planches.find((p) => p.id === selectionId) ?? null);

  /** For each planche, how many series have a plant in the ground on the day. */
  const enTerre = $derived.by(() => {
    const parPlanche = new Map<string, number>();
    for (const p of scene?.planches ?? []) {
      let n = 0;
      for (const s of p.series) if (apparenceAu(s.chrono, jourEntier).appearance === "plant") n++;
      parPlanche.set(p.id, n);
    }
    return parPlanche;
  });
  const nombreEnTerre = $derived([...enTerre.values()].reduce((a, b) => a + b, 0));
  const nombreLibres = $derived([...enTerre.values()].filter((n) => n === 0).length);
  /** A serie with a label (sown under cover) is not a free planche for the sign either. */
  const etiquetees = $derived.by(() => {
    const ids = new Set<string>();
    for (const p of scene?.planches ?? []) {
      if (p.series.some((s) => apparenceAu(s.chrono, jourEntier).appearance === "label")) ids.add(p.id);
    }
    return ids;
  });

  // The cursor starts on today; on another year, on its first day.
  $effect(() => {
    const annee = vue.year;
    untrack(() => {
      const b = bornesAnnee(annee);
      if (jourEntier < b.debut || jourEntier > b.fin) {
        const aujourdhui = jourDepuis(vue.today);
        choisir(aujourdhui >= b.debut && aujourdhui <= b.fin ? aujourdhui : b.debut);
      }
    });
  });

  // A date set from outside (the timeline, the « Aujourd'hui » button).
  $effect(() => {
    const j = jourDepuis($dateCurseur);
    untrack(() => {
      if (j === jourEntier) return;
      if (lecture) arreter();
      jourEntier = j;
      jourExact = j;
    });
  });

  // Build the plants once the planches are in the DOM; show them on the date.
  const rendu = new RenduPlantes();
  let svgEl: SVGSVGElement | undefined = $state();
  $effect(() => {
    const planches = scene?.planches;
    if (!svgEl || !planches) return;
    untrack(() => {
      rendu.vider();
      for (const p of planches) {
        const couche = svgEl!.querySelector<SVGGElement>(`[data-couche="${CSS.escape(p.id)}"]`);
        if (couche) rendu.construire(couche, p.series);
      }
      rendu.appliquer(jourExact, false);
    });
  });
  $effect(() => {
    const j = jourEntier;
    untrack(() => {
      if (!lecture) rendu.appliquer(j, false);
    });
  });

  function choisir(jour: number) {
    if (lecture) arreter();
    jourEntier = jour;
    jourExact = jour;
    $dateCurseur = isoDepuis(jour);
  }

  // ---- « Jouer la saison » ----
  let trame = 0;
  let depart = 0;
  let origine: number | null = null;
  let precedente: number | null = null;
  let durees: number[] = [];
  let recentes: number[] = [];
  let ips = $state(0);
  let plantesMax = $state(0);
  let dernierResultat = $state<(Mesure & { plantes: number }) | null>(null);

  function jouer() {
    if (lecture) {
      arreter();
      return;
    }
    if (jourExact >= bornes.fin) jourExact = bornes.debut;
    depart = jourExact;
    origine = null;
    precedente = null;
    durees = [];
    recentes = [];
    plantesMax = 0;
    lecture = true;
    $lectureEnCours = true;
    trame = requestAnimationFrame(boucle);
  }

  function arreter(termine = false) {
    cancelAnimationFrame(trame);
    rendu.appliquer(jourExact, false);
    lecture = false;
    $lectureEnCours = false;
    if (termine && mesureActive) conclure();
  }

  function conclure() {
    const mesure = mesurer(durees);
    if (!mesure) return;
    dernierResultat = { ...mesure, plantes: plantesMax };
    const fenetre = window as unknown as { __potagerieMesures?: unknown[] };
    (fenetre.__potagerieMesures ??= []).push({ ...mesure, plantes: plantesMax });
  }

  function boucle(t: number) {
    if (!lecture) return;
    origine ??= t;
    if (mesureActive && precedente !== null) {
      const dt = t - precedente;
      durees.push(dt);
      recentes.push(dt);
      if (recentes.length > 60) recentes.shift();
      ips = imagesParSeconde(recentes);
    }
    precedente = t;
    const { jour, termine } = jourDeLecture({
      debut: depart,
      ecouleMs: t - origine,
      vitesse,
      fin: bornes.fin,
      reduit,
    });
    jourExact = jour;
    rendu.appliquer(jour, !reduit, PLAFOND_PAR_IMAGE);
    if (mesureActive) plantesMax = Math.max(plantesMax, rendu.visibles);
    const entier = Math.floor(jour);
    if (entier !== jourEntier) {
      jourEntier = entier;
      $dateCurseur = isoDepuis(entier);
    }
    if (termine) arreter(true);
    else trame = requestAnimationFrame(boucle);
  }

  function changerVitesse(v: number) {
    vitesse = v;
    if (lecture) {
      // Carry on from here at the new speed.
      depart = jourExact;
      origine = null;
    }
  }

  $effect(() => () => {
    cancelAnimationFrame(trame);
    $lectureEnCours = false;
  });

  function ouvrir(p: PlancheScene) {
    selectionId = selectionId === p.id ? null : p.id;
  }

  function touche(e: KeyboardEvent, p: PlancheScene) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      ouvrir(p);
    }
  }

  function tourner() {
    quart = ((quart + 1) % 4) as Quart;
  }

  const [vx, vy, vl, vh] = VIEWBOX_SYMBOLE;
</script>

<div class="ecran">
  <section class="scene" style:background={palette.fond} aria-labelledby="diorama-date">
    <div class="entete">
      <div class="titre">
        <h1 id="diorama-date">{dateEnToutesLettres(jourEntier)}</h1>
        <div class="puces">
          <span class="puce">{NOMS_SAISON[saison]()}</span>
          <span class="puce">{m.diorama_series_en_terre({ nombre: nombreEnTerre })}</span>
          {#if nombreLibres > 0}
            <span class="puce libre">{m.diorama_planches_libres({ nombre: nombreLibres })}</span>
          {/if}
        </div>
      </div>
      <button class="bouton tourner" type="button" onclick={tourner}>
        <Icone nom="rotate" taille={17} />{m.diorama_tourner()}
      </button>
    </div>

    <div class="cadre" bind:clientWidth={largeur}>
      {#if scene}
        <svg
          bind:this={svgEl}
          class="diorama"
          viewBox={scene.cadre.join(" ")}
          role="group"
          aria-label={m.diorama_aria({
            date: dateEnToutesLettres(jourEntier),
            planches: scene.planches.length,
            series: nombreEnTerre,
          })}
          style:--herbe={palette.herbe}
          style:--herbe-g={palette.herbeGauche}
          style:--herbe-d={palette.herbeDroite}
          data-saison={saison}
        >
          <defs>
            {#each Object.entries(SYMBOLES) as [id, dessin] (id)}
              <symbol id="plante-{id}" viewBox="{vx} {vy} {vl} {vh}" overflow="visible">{@html dessin}</symbol>
            {/each}
            <symbol id={SYMBOLE_ETIQUETTE} viewBox="{vx} {vy} {vl} {vh}" overflow="visible">{@html ETIQUETTE_SVG}</symbol>
          </defs>

          <g class="socle" aria-hidden="true">
            {#each scene.socle as f, i (i)}
              <polygon class={f.classe} points={f.points} />
            {/each}
            {#each scene.touffes as t, i (i)}
              {#if t.fleur}
                <circle class="fleur teinte{t.teinte}" cx={t.x} cy={t.y - 2} r="2" />
              {:else}
                <polygon class="touffe" points="{t.x - 3},{t.y} {t.x - 1},{t.y - 6} {t.x},{t.y}" />
                <polygon class="touffe sombre" points="{t.x},{t.y} {t.x + 1.5},{t.y - 7} {t.x + 3},{t.y}" />
              {/if}
            {/each}
          </g>

          {#each scene.planches as p (p.id)}
            <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
            <g
              class="planche"
              class:choisie={selectionId === p.id}
              role="button"
              tabindex="0"
              aria-pressed={selectionId === p.id}
              aria-label={m.diorama_planche_aria({
                nom: p.nom,
                type: nomDeType(p.type),
                nombre: enTerre.get(p.id) ?? 0,
              })}
              onclick={() => ouvrir(p)}
              onkeydown={(e) => touche(e, p)}
            >
              {#each p.faces as f, i (i)}
                <polygon class={f.classe} points={f.points} />
              {/each}
              {#each p.sillons as s, i (i)}
                <line class="sillon" x1={s.a[0]} y1={s.a[1]} x2={s.b[0]} y2={s.b[1]} />
              {/each}
              <g data-couche={p.id}></g>
              {#if (enTerre.get(p.id) ?? 0) === 0 && !etiquetees.has(p.id)}
                <g class="signe-libre" aria-hidden="true">
                  <line x1={p.centre[0]} y1={p.centre[1]} x2={p.centre[0]} y2={p.centre[1] - 30} />
                  <rect x={p.centre[0] - 34} y={p.centre[1] - 52} width="68" height="24" rx="8" />
                  <text x={p.centre[0]} y={p.centre[1] - 36} text-anchor="middle">{m.diorama_libre()}</text>
                </g>
              {/if}
              <polygon class="contour" points={p.contour} />
            </g>
          {/each}
          <g class="noms" aria-hidden="true">
            {#each scene.planches as p (p.id)}
            {#if !compact || selectionId === p.id}
              {@const largeurNom = p.nom.length * 6.6 + 20}
              <g class="nom" aria-hidden="true">
                <rect
                  x={p.coinNom[0] - largeurNom + 6}
                  y={p.coinNom[1] - 17}
                  width={largeurNom}
                  height="22"
                  rx="11"
                  class:accent={selectionId === p.id}
                />
                <text x={p.coinNom[0] - largeurNom / 2 + 6} y={p.coinNom[1] - 2} text-anchor="middle">{p.nom}</text>
              </g>
            {/if}
            {/each}
          </g>
        </svg>
      {/if}
      {#if vue.planches.length === 0}
        <p class="vide">{m.diorama_vide()}</p>
      {/if}
      {#if mesureActive}
        <CompteurImages direct={ips} plantes={plantesMax} dernier={dernierResultat} />
      {/if}
    </div>

    {#if reduit}
      <p class="aide-reduite">{m.diorama_lecture_reduite()}</p>
    {/if}

    <Curseur
      annee={vue.year}
      jour={jourEntier}
      {lecture}
      {vitesse}
      {compact}
      onchoisir={choisir}
      onjouer={jouer}
      onvitesse={changerVitesse}
    />
  </section>

  <aside class="lateral" aria-live="polite">
    {#if selection}
      <PanneauDiorama {vue} planche={selection} onfermer={() => (selectionId = null)} />
    {:else}
      <div class="invite">
        <h2>{m.diorama_invite_titre()}</h2>
        <p>{m.diorama_invite()}</p>
      </div>
    {/if}
  </aside>
</div>

<style>
  .ecran {
    display: flex;
    flex-wrap: wrap;
    gap: 24px;
    align-items: stretch;
  }

  .scene {
    flex: 999 1 640px;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 24px 28px 26px;
    border-radius: 32px;
    transition: background-color 0.8s ease;
  }

  .entete {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
  }

  .titre {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  h1 {
    font-size: 34px;
  }

  .puces {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .puce {
    padding: 6px 12px;
    border-radius: var(--pill);
    background: var(--white);
    font-size: 13px;
    font-weight: 600;
  }

  .puce.libre {
    background: #fff3e4;
    color: var(--accent-ink);
  }

  .tourner {
    box-shadow: 0 2px 6px rgba(31, 58, 46, 0.08);
    border: 0;
  }

  .cadre {
    position: relative;
    min-height: 120px;
    /* The date, the chips and the slider change every few frames: keep their
       layout and paint apart from the drawing. */
    contain: layout paint style;
  }

  .diorama {
    display: block;
    width: 100%;
    height: auto;
    max-height: 70vh;
    margin: 0 auto;
    user-select: none;
    -webkit-user-select: none;
    touch-action: manipulation;
  }

  .vide {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    margin: 0;
    color: var(--muted);
    text-align: center;
  }

  .aide-reduite {
    margin: 0;
    font-size: 13px;
    color: var(--muted);
  }

  /* The ground: its colours follow the season, with a slow transition. */
  .socle polygon,
  .planche polygon {
    transition: fill 0.8s ease;
  }
  .terre-gauche {
    fill: #8a5e3c;
  }
  .terre-droite {
    fill: #74492d;
  }
  .herbe-haut {
    fill: var(--herbe);
  }
  .herbe-gauche {
    fill: var(--herbe-g);
  }
  .herbe-droite {
    fill: var(--herbe-d);
  }
  .touffe {
    fill: #86bc72;
  }
  .touffe.sombre {
    fill: #74aa63;
  }
  [data-saison="hiver"] .touffe {
    fill: #dbe8ea;
  }
  [data-saison="hiver"] .touffe.sombre {
    fill: #c3d6da;
  }
  [data-saison="automne"] .touffe {
    fill: #c9b45f;
  }
  [data-saison="automne"] .touffe.sombre {
    fill: #b39c4b;
  }
  .fleur {
    opacity: 0;
    transition: opacity 0.8s ease;
  }
  [data-saison="printemps"] .fleur,
  [data-saison="ete"] .fleur {
    opacity: 1;
  }
  .teinte0 {
    fill: #f6e58a;
  }
  .teinte1 {
    fill: #f3b3c6;
  }
  .teinte2 {
    fill: #ffffff;
  }

  .lit-haut {
    fill: #a0704a;
  }
  .lit-gauche {
    fill: #7e5236;
  }
  .lit-droite {
    fill: #6b432b;
  }
  .bois-haut {
    fill: #d9a86e;
  }
  .bois-gauche {
    fill: #c08a53;
  }
  .bois-droite {
    fill: #a87442;
  }
  .sillon {
    stroke: #8e5f3d;
    stroke-width: 1.4;
    stroke-linecap: round;
  }

  .planche {
    cursor: pointer;
    outline: none;
    -webkit-tap-highlight-color: transparent;
  }

  .contour {
    fill: none;
    stroke: var(--accent);
    stroke-width: 3;
    stroke-linejoin: round;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.15s ease;
  }

  @media (hover: hover) {
    .planche:hover .contour {
      opacity: 0.75;
    }
  }

  .planche:focus-visible .contour,
  .planche.choisie .contour {
    opacity: 1;
  }

  .planche:focus-visible .contour {
    stroke-width: 4.5;
  }

  .nom {
    pointer-events: none;
  }
  .nom rect {
    fill: #ffffff;
  }
  .nom rect.accent {
    fill: var(--accent);
  }
  .nom text {
    font-family: var(--font-titre);
    font-weight: 600;
    font-size: 12px;
    fill: var(--ink);
  }

  .signe-libre {
    pointer-events: none;
  }
  .signe-libre line {
    stroke: #8a5a36;
    stroke-width: 2.4;
    stroke-linecap: round;
  }
  .signe-libre rect {
    fill: #ffffff;
  }
  .signe-libre text {
    font-family: var(--font);
    font-weight: 700;
    font-size: 11px;
    fill: var(--green);
  }

  .lateral {
    flex: 1 1 320px;
    max-width: 400px;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .lateral > :global(*) {
    flex: 1;
  }

  .invite {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 24px;
    border-radius: 28px;
    background: var(--white);
  }

  .invite h2 {
    font-size: 22px;
  }

  .invite p {
    margin: 0;
    color: var(--muted);
  }

  /* The panel drops under the garden view when the two do not fit side by side. */
  @media (max-width: 1100px) {
    .lateral {
      max-width: none;
    }
  }

  @media (max-width: 720px) {
    .ecran {
      gap: 16px;
    }
    .scene {
      padding: 16px;
      border-radius: 24px;
      gap: 12px;
    }
    h1 {
      font-size: 26px;
    }
    .lateral {
      flex-basis: 100%;
      max-width: none;
    }
    .tourner {
      min-height: 40px;
      padding: 0 14px;
    }
  }
</style>
