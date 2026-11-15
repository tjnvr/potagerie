<script lang="ts">
  // The garden view of E05 (D01, S01 F-VIS-1 to F-VIS-4): the isometric
  // diorama in SVG, the time cursor under it, « Play la season », and the
  // bed panel (E06) beside it. Stages and dates come from the Rust view;
  // the only calculation here is the animation (animation.ts).
  import { untrack } from "svelte";
  import type { YearView } from "../bindings";
  import Icon from "../shell/Icon.svelte";
  import { m } from "../paraglide/messages.js";
  import { nameOfType } from "../beds";
  import { cursor, playbackEnProgress } from "../temps";
  import { apparenceTo, dayOfPlayback, SYMBOL_ETIQUETTE } from "./animation";
  import CounterImages from "./CounterImages.svelte";
  import Cursor from "./Cursor.svelte";
  import { ETIQUETTE_SVG } from "./etiquette";
  import { bornesYear, dateEnAllLetters, isoSince, daySince } from "./days";
  import { imagesBySeconde, measure, type Measure } from "./measure";
  import PanelDiorama from "./PanelDiorama.svelte";
  import type { Quarter } from "./projection";
  import { RenderingPlants } from "./rendering";
  import { PALETTES, seasonOf } from "./season";
  import { buildScene, type BedScene } from "./scene";
  import { SYMBOLES, VIEWBOX_SYMBOL } from "./symboles";

  let { view }: { view: YearView } = $props();

  /** Plants redrawn at most per frame while the season plays (see rendering.ts). */
  const CEILING_BY_IMAGE = 50;

  // `?fps` in the address, in dev only: the frame counter of P01.
  const measureActive = import.meta.env.DEV && new URLSearchParams(location.search).has("fps");

  const NAMES_SEASON = {
    winter: m.season_winter,
    spring: m.season_spring,
    summer: m.season_summer,
    autumn: m.season_autumn,
  };

  let quarter = $state<Quarter>(0);
  let selectionId = $state<string | null>(null);
  let speed = $state(1);
  let playback = $state(false);
  let width = $state(900);
  /** The day of the cursor, whole; the playback moves a fraction of it in `dayExact`. */
  let dayWhole = $state(daySince(untrack(() => $cursor)));
  let dayExact = untrack(() => dayWhole);

  const reducedMq = window.matchMedia("(prefers-reduced-motion: reduce)");
  let reduced = $state(reducedMq.matches);
  $effect(() => {
    const follow = () => (reduced = reducedMq.matches);
    reducedMq.addEventListener("change", follow);
    return () => reducedMq.removeEventListener("change", follow);
  });

  const scene = $derived(buildScene(view, quarter));
  const bornes = $derived(bornesYear(view.year));
  const compact = $derived(width < 560);
  const palette = $derived(PALETTES[seasonOf(dayWhole)]);
  const season = $derived(seasonOf(dayWhole));
  const selection = $derived(view.beds.find((p) => p.id === selectionId) ?? null);

  /** For each bed, how many plantings have a plant in the ground on the day. */
  const enGround = $derived.by(() => {
    const byBed = new Map<string, number>();
    for (const p of scene?.beds ?? []) {
      let n = 0;
      for (const s of p.plantings) if (apparenceTo(s.chrono, dayWhole).appearance === "plant") n++;
      byBed.set(p.id, n);
    }
    return byBed;
  });
  const countEnGround = $derived([...enGround.values()].reduce((a, b) => a + b, 0));
  const countFree = $derived([...enGround.values()].filter((n) => n === 0).length);
  /** A planting with a label (sown under cover) is not a free bed for the sign either. */
  const labeled = $derived.by(() => {
    const ids = new Set<string>();
    for (const p of scene?.beds ?? []) {
      if (p.plantings.some((s) => apparenceTo(s.chrono, dayWhole).appearance === "label")) ids.add(p.id);
    }
    return ids;
  });

  // The cursor starts on today; on another year, on its first day.
  $effect(() => {
    const year = view.year;
    untrack(() => {
      const b = bornesYear(year);
      if (dayWhole < b.start || dayWhole > b.end) {
        const today = daySince(view.today);
        choose(today >= b.start && today <= b.end ? today : b.start);
      }
    });
  });

  // A date set from outside (the timeline, the « Aujourd'hui » button).
  $effect(() => {
    const j = daySince($cursor);
    untrack(() => {
      if (j === dayWhole) return;
      if (playback) stop();
      dayWhole = j;
      dayExact = j;
    });
  });

  // Build the plants once the beds are in the DOM; show them on the date.
  const rendering = new RenderingPlants();
  let svgEl: SVGSVGElement | undefined = $state();
  $effect(() => {
    const beds = scene?.beds;
    if (!svgEl || !beds) return;
    untrack(() => {
      rendering.clear();
      for (const p of beds) {
        const layer = svgEl!.querySelector<SVGGElement>(`[data-layer="${CSS.escape(p.id)}"]`);
        if (layer) rendering.build(layer, p.plantings);
      }
      rendering.apply(dayExact, false);
    });
  });
  $effect(() => {
    const j = dayWhole;
    untrack(() => {
      if (!playback) rendering.apply(j, false);
    });
  });

  function choose(day: number) {
    if (playback) stop();
    dayWhole = day;
    dayExact = day;
    $cursor = isoSince(day);
  }

  // ---- « Play la season » ----
  let trame = 0;
  let depart = 0;
  let origin: number | null = null;
  let previous: number | null = null;
  let durations: number[] = [];
  let recent: number[] = [];
  let ips = $state(0);
  let plantsMax = $state(0);
  let dernierResult = $state<(Measure & { plants: number }) | null>(null);

  function play() {
    if (playback) {
      stop();
      return;
    }
    if (dayExact >= bornes.end) dayExact = bornes.start;
    depart = dayExact;
    origin = null;
    previous = null;
    durations = [];
    recent = [];
    plantsMax = 0;
    playback = true;
    $playbackEnProgress = true;
    trame = requestAnimationFrame(loop);
  }

  function stop(termine = false) {
    cancelAnimationFrame(trame);
    rendering.apply(dayExact, false);
    playback = false;
    $playbackEnProgress = false;
    if (termine && measureActive) conclude();
  }

  function conclude() {
    const result = measure(durations);
    if (!result) return;
    dernierResult = { ...result, plants: plantsMax };
    const globalWindow = window as unknown as { __potagerieMeasures?: unknown[] };
    (globalWindow.__potagerieMeasures ??= []).push({ ...result, plants: plantsMax });
  }

  function loop(t: number) {
    if (!playback) return;
    origin ??= t;
    if (measureActive && previous !== null) {
      const dt = t - previous;
      durations.push(dt);
      recent.push(dt);
      if (recent.length > 60) recent.shift();
      ips = imagesBySeconde(recent);
    }
    previous = t;
    const { day, termine } = dayOfPlayback({
      start: depart,
      elapsedMs: t - origin,
      speed,
      end: bornes.end,
      reduced,
    });
    dayExact = day;
    rendering.apply(day, !reduced, CEILING_BY_IMAGE);
    if (measureActive) plantsMax = Math.max(plantsMax, rendering.visible);
    const whole = Math.floor(day);
    if (whole !== dayWhole) {
      dayWhole = whole;
      $cursor = isoSince(whole);
    }
    if (termine) stop(true);
    else trame = requestAnimationFrame(loop);
  }

  function changerSpeed(v: number) {
    speed = v;
    if (playback) {
      // Carry on from here at the new speed.
      depart = dayExact;
      origin = null;
    }
  }

  $effect(() => () => {
    cancelAnimationFrame(trame);
    $playbackEnProgress = false;
  });

  function open(p: BedScene) {
    selectionId = selectionId === p.id ? null : p.id;
  }

  function key(e: KeyboardEvent, p: BedScene) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      open(p);
    }
  }

  function rotate() {
    quarter = ((quarter + 1) % 4) as Quarter;
  }

  const [vx, vy, vl, vh] = VIEWBOX_SYMBOL;
</script>

<div class="screen">
  <section class="scene" style:background={palette.fond} aria-labelledby="diorama-date">
    <div class="header">
      <div class="title">
        <h1 id="diorama-date">{dateEnAllLetters(dayWhole)}</h1>
        <div class="puces">
          <span class="puce">{NAMES_SEASON[season]()}</span>
          <span class="puce">{m.diorama_plantings_in_ground({ count: countEnGround })}</span>
          {#if countFree > 0}
            <span class="puce free">{m.diorama_beds_free({ count: countFree })}</span>
          {/if}
        </div>
      </div>
      <button class="button rotate" type="button" onclick={rotate}>
        <Icon name="rotate" size={17} />{m.diorama_rotate()}
      </button>
    </div>

    <div class="cadre" bind:clientWidth={width}>
      {#if scene}
        <svg
          bind:this={svgEl}
          class="diorama"
          viewBox={scene.cadre.join(" ")}
          role="group"
          aria-label={m.diorama_aria({
            date: dateEnAllLetters(dayWhole),
            beds: scene.beds.length,
            plantings: countEnGround,
          })}
          style:--grass={palette.grass}
          style:--grass-g={palette.grassGauche}
          style:--grass-d={palette.grassRight}
          data-season={season}
        >
          <defs>
            {#each Object.entries(SYMBOLES) as [id, drawing] (id)}
              <symbol id="plant-{id}" viewBox="{vx} {vy} {vl} {vh}" overflow="visible">{@html drawing}</symbol>
            {/each}
            <symbol id={SYMBOL_ETIQUETTE} viewBox="{vx} {vy} {vl} {vh}" overflow="visible">{@html ETIQUETTE_SVG}</symbol>
          </defs>

          <g class="socle" aria-hidden="true">
            {#each scene.socle as f, i (i)}
              <polygon class={f.class} points={f.points} />
            {/each}
            {#each scene.tufts as t, i (i)}
              {#if t.flower}
                <circle class="flower tint{t.tint}" cx={t.x} cy={t.y - 2} r="2" />
              {:else}
                <polygon class="tuft" points="{t.x - 3},{t.y} {t.x - 1},{t.y - 6} {t.x},{t.y}" />
                <polygon class="tuft sombre" points="{t.x},{t.y} {t.x + 1.5},{t.y - 7} {t.x + 3},{t.y}" />
              {/if}
            {/each}
          </g>

          {#each scene.beds as p (p.id)}
            <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
            <g
              class="bed"
              class:chosen={selectionId === p.id}
              role="button"
              tabindex="0"
              aria-pressed={selectionId === p.id}
              aria-label={m.diorama_bed_aria({
                name: p.name,
                type: nameOfType(p.type),
                count: enGround.get(p.id) ?? 0,
              })}
              onclick={() => open(p)}
              onkeydown={(e) => key(e, p)}
            >
              {#each p.faces as f, i (i)}
                <polygon class={f.class} points={f.points} />
              {/each}
              {#each p.sillons as s, i (i)}
                <line class="sillon" x1={s.a[0]} y1={s.a[1]} x2={s.b[0]} y2={s.b[1]} />
              {/each}
              <g data-layer={p.id}></g>
              {#if (enGround.get(p.id) ?? 0) === 0 && !labeled.has(p.id)}
                <g class="sign-free" aria-hidden="true">
                  <line x1={p.centre[0]} y1={p.centre[1]} x2={p.centre[0]} y2={p.centre[1] - 30} />
                  <rect x={p.centre[0] - 34} y={p.centre[1] - 52} width="68" height="24" rx="8" />
                  <text x={p.centre[0]} y={p.centre[1] - 36} text-anchor="middle">{m.diorama_free()}</text>
                </g>
              {/if}
              <polygon class="contour" points={p.contour} />
            </g>
          {/each}
          <g class="names" aria-hidden="true">
            {#each scene.beds as p (p.id)}
            {#if !compact || selectionId === p.id}
              {@const widthName = p.name.length * 6.6 + 20}
              <g class="name" aria-hidden="true">
                <rect
                  x={p.coinName[0] - widthName + 6}
                  y={p.coinName[1] - 17}
                  width={widthName}
                  height="22"
                  rx="11"
                  class:accent={selectionId === p.id}
                />
                <text x={p.coinName[0] - widthName / 2 + 6} y={p.coinName[1] - 2} text-anchor="middle">{p.name}</text>
              </g>
            {/if}
            {/each}
          </g>
        </svg>
      {/if}
      {#if view.beds.length === 0}
        <p class="empty">{m.diorama_empty()}</p>
      {/if}
      {#if measureActive}
        <CounterImages direct={ips} plants={plantsMax} dernier={dernierResult} />
      {/if}
    </div>

    {#if reduced}
      <p class="help-reduced">{m.diorama_playback_reduced()}</p>
    {/if}

    <Cursor
      year={view.year}
      day={dayWhole}
      {playback}
      {speed}
      {compact}
      onchoose={choose}
      onplay={play}
      onspeed={changerSpeed}
    />
  </section>

  <aside class="lateral" aria-live="polite">
    {#if selection}
      <PanelDiorama {view} bed={selection} onclose={() => (selectionId = null)} />
    {:else}
      <div class="invite">
        <h2>{m.diorama_invite_title()}</h2>
        <p>{m.diorama_invite()}</p>
      </div>
    {/if}
  </aside>
</div>

<style>
  .screen {
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

  .header {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
  }

  .title {
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

  .puce.free {
    background: #fff3e4;
    color: var(--accent-ink);
  }

  .rotate {
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

  .empty {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    margin: 0;
    color: var(--muted);
    text-align: center;
  }

  .help-reduced {
    margin: 0;
    font-size: 13px;
    color: var(--muted);
  }

  /* The ground: its colours follow the season, with a slow transition. */
  .socle polygon,
  .bed polygon {
    transition: fill 0.8s ease;
  }
  .ground-gauche {
    fill: #8a5e3c;
  }
  .ground-right {
    fill: #74492d;
  }
  .grass-top {
    fill: var(--grass);
  }
  .grass-gauche {
    fill: var(--grass-g);
  }
  .grass-right {
    fill: var(--grass-d);
  }
  .tuft {
    fill: #86bc72;
  }
  .tuft.sombre {
    fill: #74aa63;
  }
  [data-season="winter"] .tuft {
    fill: #dbe8ea;
  }
  [data-season="winter"] .tuft.sombre {
    fill: #c3d6da;
  }
  [data-season="autumn"] .tuft {
    fill: #c9b45f;
  }
  [data-season="autumn"] .tuft.sombre {
    fill: #b39c4b;
  }
  .flower {
    opacity: 0;
    transition: opacity 0.8s ease;
  }
  [data-season="spring"] .flower,
  [data-season="summer"] .flower {
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

  .lit-top {
    fill: #a0704a;
  }
  .lit-gauche {
    fill: #7e5236;
  }
  .lit-right {
    fill: #6b432b;
  }
  .bois-top {
    fill: #d9a86e;
  }
  .bois-gauche {
    fill: #c08a53;
  }
  .bois-right {
    fill: #a87442;
  }
  .sillon {
    stroke: #8e5f3d;
    stroke-width: 1.4;
    stroke-linecap: round;
  }

  .bed {
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
    .bed:hover .contour {
      opacity: 0.75;
    }
  }

  .bed:focus-visible .contour,
  .bed.chosen .contour {
    opacity: 1;
  }

  .bed:focus-visible .contour {
    stroke-width: 4.5;
  }

  .name {
    pointer-events: none;
  }
  .name rect {
    fill: #ffffff;
  }
  .name rect.accent {
    fill: var(--accent);
  }
  .name text {
    font-family: var(--font-title);
    font-weight: 600;
    font-size: 12px;
    fill: var(--ink);
  }

  .sign-free {
    pointer-events: none;
  }
  .sign-free line {
    stroke: #8a5a36;
    stroke-width: 2.4;
    stroke-linecap: round;
  }
  .sign-free rect {
    fill: #ffffff;
  }
  .sign-free text {
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
    .screen {
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
    .rotate {
      min-height: 40px;
      padding: 0 14px;
    }
  }
</style>
