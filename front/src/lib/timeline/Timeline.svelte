<script lang="ts">
  // The continuous timeline of the crops (S01 F-VIS-2, D01 E05): one row per
  // bed, scrolling through time without a break between years, the bars
  // that can be dragged (Pointer Events, long press on touch), the free slots,
  // the cursor of the time.
  //
  // This component only handles the gestures and the loading of the year
  // views around the visible days. What is drawn comes from the views of
  // Rust (`TimelineSvg`), and a drag is only a proposal: Rust judges (S03 T12).
  import { untrack } from "svelte";
  import { get } from "svelte/store";
  import { ControllerDrag, type Response, type TypePointer } from "../editor/drag";
  import { commands, type YearView } from "../bindings";
  import { yearOf, dateOf, dayOf } from "../dates";
  import { translateError } from "../errors";
  import { call, app } from "../state.svelte";
  import { m } from "../paraglide/messages.js";
  import { goTo, cursor, viewportTimeline, recentering } from "../time";
  import { shiftShown, deposerBar } from "./bar";
  import { PALETTE, colorCrop, melange } from "./colors";
  import TimelineSvg, { WIDTH_LABELS, WIDTH_LABELS_NARROW } from "./TimelineSvg.svelte";
  import {
    DAYS_BY_DEFAULT_NARROW,
    THRESHOLD_DEFILEMENT_PX,
    yearsVisible,
    centerOn,
    buildRows,
    defiler,
    scaleCentered,
    keepVisible,
    dayUnderLePointer,
    daysVisible,
    zoomer,
    type Scale,
  } from "./geometry";
  import IconTimeline from "./IconTimeline.svelte";

  let { view }: { view: YearView } = $props();

  let widthTotal = $state(0);
  let scale = $state<Scale | null>(null);
  let collapsed = $state(false);
  let showAnnotations = $state(true);
  let corps: HTMLElement | undefined = $state();

  const narrow = $derived(widthTotal > 0 && widthTotal < 560);
  const widthLabels = $derived(narrow ? WIDTH_LABELS_NARROW : WIDTH_LABELS);
  const widthZone = $derived(Math.max(widthTotal - widthLabels, 120));

  const reduced = () => matchMedia("(prefers-reduced-motion: reduce)").matches;

  // ---------- Scale: opens on 12 months around the cursor ----------

  $effect(() => {
    if (widthTotal <= 0) return;
    untrack(() => {
      if (!scale) {
        scale = scaleCentered(widthZone, dayOf(get(cursor)), narrow ? DAYS_BY_DEFAULT_NARROW : undefined);
      } else if (scale.width !== widthZone) {
        scale = { ...scale, width: widthZone, pxByDay: widthZone / daysVisible(scale) };
      }
    });
  });

  // Tell the export (E11) which days are on screen.
  $effect(() => {
    viewportTimeline.set(scale ? { startDay: scale.startDay, days: daysVisible(scale) } : null);
  });

  // The cursor moved elsewhere (diorama, playback, ‹ ›): scroll to keep it in
  // view, and show the season it is in (F-PLA-4).
  $effect(() => {
    const date = $cursor;
    untrack(() => {
      if (scale) scale = keepVisible(scale, dayOf(date));
      if (yearOf(date) !== app.year) app.charger(yearOf(date));
    });
  });

  // “Today”: back to the middle of the timeline, even if the cursor did not move.
  $effect(() => {
    if ($recentering === 0) return;
    untrack(() => {
      if (scale) scale = centerOn(scale, dayOf(get(cursor)));
    });
  });

  // ---------- The year views around the visible days ----------

  let loaded = $state<Record<number, YearView>>({});
  const enProgress = new Set<number>();
  let token = 0;

  const years = $derived(scale ? yearsVisible(scale) : [view.year]);
  const views = $derived(years.map((a) => (a === view.year ? view : loaded[a])).filter((v): v is YearView => !!v));
  const rows = $derived(buildRows(views, view.year));
  /** Notes and periods of the loaded years, each once (a period can touch two years). */
  const annotations = $derived([...new Map(views.flatMap((v) => v.annotations).map((a) => [a.id, a])).values()]);
  const nonePlanting = $derived(views.every((v) => v.plantings.length === 0));

  // A change of the plan (a command returned a new view): the other years are stale.
  $effect(() => {
    void view.version;
    void view.year;
    untrack(() => {
      token++;
      enProgress.clear();
      loaded = {};
    });
  });

  $effect(() => {
    for (const year of years) {
      if (year === view.year || loaded[year] || enProgress.has(year)) continue;
      enProgress.add(year);
      const mon = token;
      void call(() => commands.viewYear(year)).then((r) => {
        if (mon !== token) return;
        enProgress.delete(year);
        if (r.status === "ok") loaded[year] = r.data;
      });
    }
  });

  // ---------- Gestures ----------

  type Geste =
    | { type: "pan"; depart: Scale; x0: number; bouge: boolean; target: Element | null }
    | { type: "bar"; id: string; x0: number; dx: number; sending: boolean }
    | { type: "cursor" };

  let geste: Geste | null = $state(null);
  let shifts = $state<Record<string, number>>({});
  let scope = $state<string | null>(null);
  let refusal = $state<string | null>(null);
  const controller = new ControllerDrag({ onArmed: () => (scope = geste?.type === "bar" ? geste.id : null) });
  const pointers = new Map<number, { x: number; y: number }>();
  let distancePincement = 0;

  function gauche(): number {
    return (corps?.getBoundingClientRect().left ?? 0) + widthLabels;
  }

  function enZone(e: PointerEvent): boolean {
    return e.clientX >= gauche();
  }

  function low(e: PointerEvent) {
    if (!scale || geste?.type === "bar" && geste.sending) return;
    if (e.pointerType === "mouse" && e.button !== 0) return;
    pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
    corps?.setPointerCapture(e.pointerId);
    if (pointers.size === 2) {
      controller.cancel();
      geste = null;
      scope = null;
      const [a, b] = [...pointers.values()] as [{ x: number; y: number }, { x: number; y: number }];
      distancePincement = Math.hypot(a.x - b.x, a.y - b.y);
      return;
    }
    const target = e.target as Element;
    if (target.closest("[data-cursor]")) {
      geste = { type: "cursor" };
      return;
    }
    const bar = target.closest("[data-bar]");
    if (bar && enZone(e)) {
      geste = { type: "bar", id: bar.getAttribute("data-bar")!, x0: e.clientX, dx: 0, sending: false };
      scope = null;
      controller.press(e.pointerType as TypePointer, { x: e.clientX, y: e.clientY });
      return;
    }
    geste = { type: "pan", depart: scale, x0: e.clientX, bouge: false, target };
  }

  function bouge(e: PointerEvent) {
    if (!scale || !pointers.has(e.pointerId)) return;
    pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (pointers.size === 2) {
      const [a, b] = [...pointers.values()] as [{ x: number; y: number }, { x: number; y: number }];
      const distance = Math.hypot(a.x - b.x, a.y - b.y);
      if (distancePincement > 0 && distance > 0) {
        scale = zoomer(scale, distancePincement / distance, (a.x + b.x) / 2 - gauche());
      }
      distancePincement = distance;
      return;
    }
    if (!geste) return;
    if (geste.type === "cursor") {
      goTo(dateOf(dayUnderLePointer(scale, e.clientX, gauche())));
    } else if (geste.type === "bar") {
      if (geste.sending) return;
      const movement = controller.move({ x: e.clientX, y: e.clientY });
      if (movement === "abandon") {
        // A finger that moves before the long press scrolls the timeline.
        geste = { type: "pan", depart: scale, x0: geste.x0, bouge: true, target: null };
        scope = null;
        scale = defiler(scale, e.clientX - geste.x0);
      } else if (movement === "start" || movement === "dragged") {
        scope = geste.id;
        geste.dx = e.clientX - geste.x0;
        shifts = { [geste.id]: shiftShown(geste.dx, scale.pxByDay) };
      }
    } else {
      const dx = e.clientX - geste.x0;
      if (!geste.bouge && Math.abs(dx) <= THRESHOLD_DEFILEMENT_PX) return;
      geste.bouge = true;
      scale = defiler(geste.depart, dx);
    }
  }

  async function raised(e: PointerEvent) {
    pointers.delete(e.pointerId);
    if (corps?.hasPointerCapture(e.pointerId)) corps.releasePointerCapture(e.pointerId);
    distancePincement = 0;
    const courant = geste;
    if (!courant || !scale) return;
    if (courant.type === "cursor") {
      geste = null;
      return;
    }
    if (courant.type === "pan") {
      geste = null;
      if (courant.bouge || !enZone(e) && !courant.target?.closest("[data-slot],[data-annotation]")) return;
      const slot = courant.target?.closest("[data-slot]");
      const note = courant.target?.closest("[data-annotation]");
      if (note) openAnnotation(note.getAttribute("data-annotation")!);
      else if (slot) openSlot(slot.getAttribute("data-slot")!);
      else goTo(dateOf(dayUnderLePointer(scale, e.clientX, gauche())));
      return;
    }
    if (courant.sending) return;
    const end = controller.release();
    scope = null;
    if (end === "click") {
      geste = null;
      openBar(courant.id);
      return;
    }
    if (end !== "end") {
      geste = null;
      shifts = {};
      return;
    }
    courant.sending = true;
    await shift(courant.id, courant.dx, scale.pxByDay);
    geste = null;
  }

  function cancelled(e: PointerEvent) {
    pointers.delete(e.pointerId);
    controller.cancel();
    scope = null;
    if (geste?.type === "bar" && !geste.sending) {
      geste = null;
      shifts = {};
    } else if (geste?.type !== "bar") {
      geste = null;
    }
  }

  /** Drops the bar: Rust accepts the shift, or refuses and the bar goes back. */
  async function shift(id: string, dxPx: number, pxByDay: number) {
    const issue = await deposerBar(dxPx, pxByDay, async (days): Promise<Response> => {
      const ok = await app.modify((year) => commands.shiftPlanting(year, id, days));
      return ok ? "ok" : "refusal";
    });
    if (issue.response === "refusal") {
      const reason = app.dernierRefusal ? translateError(app.dernierRefusal) : "";
      app.message = { text: m.timeline_refusal({ reason }), ton: "error" };
      refusal = id;
      setTimeout(() => (refusal = null), 1500);
      await bringBack(id, shiftShown(dxPx, pxByDay));
    } else {
      shifts = {};
    }
  }

  /** The bar goes back to its place; at once if the system asks to reduce motion. */
  async function bringBack(id: string, since: number) {
    if (reduced()) {
      shifts = {};
      return;
    }
    const duration = 180;
    const start = performance.now();
    await new Promise<void>((finished) => {
      const step = (t: number) => {
        const p = Math.min(1, (t - start) / duration);
        shifts = { [id]: since * (1 - (1 - (1 - p) * (1 - p))) };
        if (p < 1) requestAnimationFrame(step);
        else finished();
      };
      requestAnimationFrame(step);
    });
    shifts = {};
  }

  function openBar(id: string) {
    const planting = [...views.flatMap((v) => v.plantings), ...views.flatMap((v) => v.history)].find((s) => s.id === id);
    if (planting) app.plantingOpen = { type: "edition", planting };
  }

  function openAnnotation(id: string) {
    const annotation = annotations.find((a) => a.id === id);
    if (annotation) app.noteOpen = { type: "edition", annotation };
  }

  function annotate() {
    app.noteOpen = { type: "new", date: get(cursor), bedId: null };
  }

  function openSlot(key: string) {
    const row = rows.find((l) => l.slots.some((c) => c.key === key));
    const slot = row?.slots.find((c) => c.key === key);
    if (!row || !slot) return;
    app.plantingOpen = {
      type: "new",
      bedId: row.bed.id,
      slot: slot.source,
      suggestion: slot.source.suggestions[0],
    };
  }

  function key(e: KeyboardEvent) {
    const target = e.target as Element;
    const bar = target.closest("[data-bar]");
    const slot = target.closest("[data-slot]");
    const note = target.closest("[data-annotation]");
    if (e.key === "Enter" || e.key === " ") {
      if (note) openAnnotation(note.getAttribute("data-annotation")!);
      else if (bar) openBar(bar.getAttribute("data-bar")!);
      else if (slot) openSlot(slot.getAttribute("data-slot")!);
      else return;
      e.preventDefault();
    } else if (bar && (e.key === "ArrowLeft" || e.key === "ArrowRight") && scale && !geste) {
      // The keyboard shifts a bar by a day, or a week with Shift.
      e.preventDefault();
      const days = (e.key === "ArrowLeft" ? -1 : 1) * (e.shiftKey ? 7 : 1);
      void shift(bar.getAttribute("data-bar")!, days * scale.pxByDay, scale.pxByDay);
    }
  }

  // The wheel: with Ctrl (or a trackpad pinch) it zooms, sideways it scrolls.
  // The listeners are not passive: they stop the page from scrolling too.
  $effect(() => {
    const el = corps;
    if (!el) return;
    const roue = (e: WheelEvent) => {
      if (!scale) return;
      const inside = e.clientX - gauche();
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        scale = zoomer(scale, Math.exp(e.deltaY * 0.0015), inside);
      } else if (Math.abs(e.deltaX) > Math.abs(e.deltaY) || e.shiftKey) {
        e.preventDefault();
        scale = defiler(scale, -(e.deltaX || e.deltaY));
      }
    };
    const touch = (e: TouchEvent) => {
      if (controller.state === "armed" || controller.state === "dragged") e.preventDefault();
    };
    el.addEventListener("wheel", roue, { passive: false });
    el.addEventListener("touchmove", touch, { passive: false });
    return () => {
      el.removeEventListener("wheel", roue);
      el.removeEventListener("touchmove", touch);
    };
  });

  function aller(direction: -1 | 1) {
    if (scale) scale = defiler(scale, -direction * scale.width * 0.33);
  }

  function zoom(factor: number) {
    if (scale) scale = zoomer(scale, factor, scale.width / 2);
  }
</script>

<section class="timeline" aria-labelledby="timeline-title">
  <div class="tete">
    <div class="titles">
      <h2 id="timeline-title">{m.timeline_title()}</h2>
      <p>{m.timeline_help()}</p>
    </div>
    <ul class="legend" aria-label={m.timeline_title()}>
      <li>
        <svg width="26" height="14" aria-hidden="true">
          <defs>
            <pattern id="legend-pots" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
              <rect width="8" height="8" fill={melange(colorCrop("tomato"), 0.85)} />
              <rect width="4" height="8" fill={melange(colorCrop("tomato"), 0.55)} />
            </pattern>
          </defs>
          <rect width="26" height="14" rx="4" fill="url(#legend-pots)" />
        </svg>
        {m.timeline_legend_pots()}
      </li>
      <li>
        <svg width="26" height="14" aria-hidden="true">
          <defs>
            <pattern id="legend-growth" width="8" height="8" patternUnits="userSpaceOnUse">
              <rect width="8" height="8" fill={melange(colorCrop("tomato"), 0.45)} />
              <circle cx="4" cy="4" r="0.9" fill={melange(colorCrop("tomato"), 0.2)} />
            </pattern>
          </defs>
          <rect width="26" height="14" rx="4" fill="url(#legend-growth)" />
        </svg>
        {m.timeline_legend_growth()}
      </li>
      <li>
        <svg width="26" height="14" aria-hidden="true"><rect width="26" height="14" rx="4" fill={colorCrop("tomato")} /></svg>
        {m.timeline_legend_harvest()}
      </li>
      <li>
        <svg width="26" height="14" aria-hidden="true">
          <rect x="1" y="1" width="24" height="12" rx="4" fill={PALETTE.slotFond} stroke={PALETTE.accent} stroke-width="1.5" stroke-dasharray="4 3" />
        </svg>
        {m.timeline_legend_slot()}
      </li>
    </ul>
    <div class="tools">
      <button class="rond" aria-label={m.timeline_previous()} title={m.timeline_previous()} onclick={() => aller(-1)}><IconTimeline name="before" /></button>
      <button class="rond" aria-label={m.timeline_next()} title={m.timeline_next()} onclick={() => aller(1)}><IconTimeline name="after" /></button>
      <button class="rond" aria-label={m.timeline_zoom_in()} title={m.timeline_zoom_in()} onclick={() => zoom(0.7)}><IconTimeline name="plus" /></button>
      <button class="rond" aria-label={m.timeline_zoom_out()} title={m.timeline_zoom_out()} onclick={() => zoom(1 / 0.7)}><IconTimeline name="less" /></button>
      <button class="button" onclick={annotate}><IconTimeline name="plus" size={16} />{m.note_annotate()}</button>
      <button class="button" onclick={() => (app.exportOpen = true)}>{m.export_open()}</button>
      <label class="case"><input type="checkbox" bind:checked={showAnnotations} />{m.note_show()}</label>
      <button class="button" aria-expanded={!collapsed} onclick={() => (collapsed = !collapsed)}>
        {collapsed ? m.timeline_expand() : m.timeline_collapse()}
        <IconTimeline name={collapsed ? "low" : "top"} size={16} />
      </button>
    </div>
  </div>

  {#if !collapsed}
    {#if rows.length === 0}
      <p class="empty">{m.timeline_empty_beds()}</p>
    {:else}
      {#if nonePlanting}
        <p class="empty leger">{m.timeline_empty()}</p>
      {/if}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="corps"
        bind:this={corps}
        bind:clientWidth={widthTotal}
        onpointerdown={low}
        onpointermove={bouge}
        onpointerup={raised}
        onpointercancel={cancelled}
        onkeydown={key}
      >
        {#if scale}
          <TimelineSvg
            {rows}
            {scale}
            {widthLabels}
            today={view.today}
            cursor={$cursor}
            {shifts}
            {refusal}
            {scope}
            yearShown={view.year}
            annotations={showAnnotations ? annotations : []}
          />
        {/if}
      </div>
      <p class="help">{m.timeline_help_bar()}</p>
    {/if}
  {/if}
</section>

<style>
  .timeline {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 22px 24px 18px;
    border-radius: 32px;
    background: var(--white);
  }

  .tete {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px 20px;
  }

  .titles {
    flex: 1 1 260px;
  }

  h2 {
    font-size: 26px;
  }

  .titles p,
  .help {
    margin: 4px 0 0;
    color: var(--muted);
    font-size: 13px;
  }

  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    margin: 0;
    padding: 0;
    list-style: none;
    font-size: 13px;
    color: var(--muted);
  }

  .legend li {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .tools {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }

  .rond {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border: 1px solid var(--line);
    border-radius: 50%;
    background: var(--white);
  }

  .case {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    color: var(--muted);
  }

  .button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-height: 40px;
    padding: 0 14px;
    font-size: 14px;
  }

  .corps {
    overflow: hidden;
    border-radius: 12px;
    /* Horizontal swipes scroll the timeline; vertical ones scroll the page. */
    touch-action: pan-y;
    user-select: none;
    -webkit-user-select: none;
  }

  .empty {
    margin: 0;
    padding: 48px 20px;
    border: 1px dashed var(--line);
    border-radius: 16px;
    background: var(--white);
    text-align: center;
    color: var(--muted);
  }

  .empty.leger {
    padding: 10px 16px;
    font-size: 14px;
  }

  @media (max-width: 720px) {
    .timeline {
      padding: 16px 12px 14px;
      border-radius: 24px;
    }
    h2 {
      font-size: 22px;
    }
  }
</style>
