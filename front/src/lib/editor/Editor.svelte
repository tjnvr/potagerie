<script lang="ts">
  // Plan tab: the editor (E03). Top view, tray (Bed, Square, Mound, Planter),
  // right panel with the selected bed (E06) or the balance of the plan.
  import { commands, type BedView, type TypeBed, type YearView } from "../bindings";
  import Confirmation from "../shell/Confirmation.svelte";
  import CreationPlot from "../shell/CreationPlot.svelte";
  import Icon from "../shell/Icon.svelte";
  import { app } from "../state.svelte";
  import { m } from "../paraglide/messages.js";
  import {
    MAX_BEDS,
    SIZE_BY_DEFAULT,
    summary,
    formatM2,
    formatMeters,
    nameOfType,
    nameByDefault,
    positionFree,
  } from "../beds";
  import {
    ControllerDrag,
    positionSincePlanter,
    type Geo,
    type Response,
    type TypePointer,
  } from "./drag";
  import PanelBed from "./PanelBed.svelte";
  import PlanViewTop, { type Ghost } from "./PlanViewTop.svelte";

  let { view }: { view: YearView } = $props();

  const TYPES: TypeBed[] = ["bed", "square", "mound", "planter"];
  const complete = $derived(view.beds.length >= MAX_BEDS);
  const account = $derived(view.plot ? summary(view.plot, view.beds) : null);

  let top: PlanViewTop | undefined = $state();
  // « Move » from the garden's bed panel opens the editor on that bed (F-VIS-7).
  let selectionId = $state<string | null>(app.takeBedToSelect());
  let ghost = $state<Ghost | null>(null);
  let scope = $state<TypeBed | null>(null);
  let shortening = $state<{ bed: BedView; count: number; answer: (ok: boolean) => void } | null>(null);

  const selection = $derived(view.beds.find((p) => p.id === selectionId) ?? null);

  // A bed that disappears (deleted, or another season) is no longer selected.
  $effect(() => {
    if (selectionId && !view.beds.some((p) => p.id === selectionId)) selectionId = null;
  });

  async function poser(type: TypeBed, x: number, y: number) {
    const name = nameByDefault(type, view.beds);
    const before = new Set(view.beds.map((p) => p.id));
    const ok = (await app.place((year, keepClose) => commands.createBed(year, type, name, x, y, keepClose))) === "ok";
    if (ok && app.view) {
      // The new bed stays selected (F-PLN-6).
      selectionId = app.view.beds.find((p) => !before.has(p.id))?.id ?? null;
    }
  }

  /** The tray without a pointer drag (keyboard, plain click): first free spot. */
  async function add(type: TypeBed) {
    if (!view.plot) return;
    const position = positionFree(view.plot, view.beds, type);
    if (!position) {
      app.informer(m.plan_no_free_space());
      return;
    }
    await poser(type, position.x, position.y);
  }

  /** A click on free grass: a bed of the chosen tray type, centred there. */
  async function poserHere(point: { x: number; y: number }) {
    if (!view.plot || complete) return;
    const type = scope ?? "bed";
    const size = SIZE_BY_DEFAULT[type];
    const pos = positionSincePlanter(point, size, view.plot.cell_cm);
    if (pos.x < 0 || pos.y < 0) {
      app.report({ code: "bed_outside_plot", detail: "" });
      return;
    }
    await poser(type, pos.x, pos.y);
  }

  /** A zone drawn on the grass: a bed of the chosen type, then resized to the zone. */
  async function drawZone(zone: { x: number; y: number; length_cm: number; width_cm: number }) {
    if (!view.plot || complete) return;
    const type = scope ?? "bed";
    // Created on a free spot (its default size may not fit where the zone starts,
    // near the edge), then moved and resized to the zone in one change.
    const spot = positionFree(view.plot, view.beds, type);
    if (!spot) {
      app.informer(m.plan_no_free_space());
      return;
    }
    const before = new Set(view.beds.map((p) => p.id));
    await poser(type, spot.x, spot.y);
    const id = selectionId;
    const bed = id && !before.has(id) ? app.view?.beds.find((b) => b.id === id) : null;
    if (id && bed) {
      await app.place((year, keepClose) =>
        commands.modifyBed(year, id, null, { bed_type: bed.bed_type, ...zone }, keepClose),
      );
    }
  }

  // Drag from the tray: the same long press on a finger, the same pointer logic.
  const planter = new ControllerDrag({ onArmed: () => {} });
  let typePlanter: TypeBed | null = null;

  function startPlanter(e: PointerEvent, type: TypeBed) {
    if (e.button !== 0 || complete || view.read_only) return;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    typePlanter = type;
    scope = type;
    planter.press(e.pointerType as TypePointer, { x: e.clientX, y: e.clientY });
  }

  function bougePlanter(e: PointerEvent) {
    if (!typePlanter) return;
    const movement = planter.move({ x: e.clientX, y: e.clientY });
    if (movement === "abandon") {
      typePlanter = null;
      scope = null;
      return;
    }
    if ((movement === "start" || movement === "dragged") && view.plot) {
      const point = top?.versCm(e.clientX, e.clientY) ?? null;
      const size = SIZE_BY_DEFAULT[typePlanter];
      ghost = point ? { type: typePlanter, ...size, ...positionSincePlanter(point, size, view.plot.cell_cm) } : null;
    }
  }

  async function raisedPlanter(e: PointerEvent) {
    const type = typePlanter;
    if (!type) return;
    const button = e.currentTarget as HTMLElement;
    if (button.hasPointerCapture(e.pointerId)) button.releasePointerCapture(e.pointerId);
    const end = planter.release();
    const depot = ghost;
    typePlanter = null;
    scope = null;
    ghost = null;
    if (end === "click") await add(type);
    else if (end === "end" && depot) {
      if (depot.x < 0 || depot.y < 0) app.report({ code: "bed_outside_plot", detail: "" });
      else await poser(type, depot.x, depot.y);
    }
  }

  function cancelledPlanter() {
    planter.cancel();
    typePlanter = null;
    scope = null;
    ghost = null;
  }

  /** A move or a resize: Rust judges. A shorter bed first asks which
   *  plantings would no longer fit (S03 §4) and waits for the user; a bed
   *  closer than the aisle asks « Keep here » or « Go back » (S03 §15). */
  async function modify(id: string, geo: Geo, nature: "move" | "resize"): Promise<Response> {
    const bed = view.beds.find((p) => p.id === id);
    if (!bed) return "refusal";
    if (nature === "resize" && geo.length_cm < bed.length_cm) {
      const result = await commands.plantingsExceeding(app.year, id, geo.length_cm);
      if (result.status === "ok" && result.data.length > 0) {
        const ok = await new Promise<boolean>((answer) => {
          shortening = { bed, count: result.data.length, answer };
        });
        shortening = null;
        if (!ok) return "cancelled";
      }
    }
    return nature === "move"
      ? app.place((year, keepClose) => commands.moveBed(year, id, geo.x, geo.y, keepClose))
      : app.place((year, keepClose) =>
          commands.modifyBed(year, id, null, { bed_type: bed.bed_type, ...geo }, keepClose),
        );
  }
</script>

{#if !view.plot}
  <CreationPlot />
{:else}
  <div class="bandeau">
    <span class="year">
      {#if view.read_only}
        <strong>{m.plan_consultation({ year: view.year })}</strong>
      {:else}
        <strong>{m.plan_title({ year: view.year })}</strong>
        <span class="scope"><span class="dot">· </span>{m.plan_scope({ year: view.year })}</span>
      {/if}
    </span>
    <span class="space"></span>
    <button class="button principal" onclick={() => (app.tab = "garden")}>
      <Icon name="check" />{m.see_garden()}
    </button>
  </div>

  <div class="editor">
    <section class="scene">
      <p class="plot">
        {m.plan_plot({
          length: formatMeters(view.plot.length_cm),
          width: formatMeters(view.plot.width_cm),
          case: view.plot.cell_cm,
        })}
      </p>
      <PlanViewTop
        bind:this={top}
        plot={view.plot}
        beds={view.beds}
        {selectionId}
        readOnly={view.read_only}
        {ghost}
        onselect={(id) => (selectionId = id)}
        onmodify={modify}
        onbeforeplot={() => app.report({ code: "bed_outside_plot", detail: "" })}
        onplace={poserHere}
        ondraw={drawZone}
      />
      {#if view.beds.length === 0}
        <p class="help">{m.plan_empty()}</p>
      {/if}

      <p class="tips">
        <span><b>{m.plan_help_touch()}</b> {m.plan_help_touch_text()}</span>
        <span><b>{m.plan_help_drag()}</b> {m.plan_help_drag_text()}</span>
        <span><b>{m.plan_help_handles()}</b> {m.plan_help_handles_text()}</span>
      </p>

      {#if !view.read_only}
        <div class="planter">
          <div class="planter-title">
            <strong>{m.tray_title()}</strong>
            <span class="counter">{m.plan_counter({ count: view.beds.length, max: MAX_BEDS })}</span>
          </div>
          <div class="elements">
            {#each TYPES as type (type)}
              <button
                class="element"
                class:scope={scope === type}
                disabled={complete}
                onpointerdown={(e) => startPlanter(e, type)}
                onpointermove={bougePlanter}
                onpointerup={raisedPlanter}
                onpointercancel={cancelledPlanter}
                onclick={(e) => e.detail === 0 && add(type)}
              >
                <span class="name">{nameOfType(type)}</span>
                <span class="size">
                  {m.tray_size({
                    length: formatMeters(SIZE_BY_DEFAULT[type].length_cm),
                    width: formatMeters(SIZE_BY_DEFAULT[type].width_cm),
                  })}
                </span>
              </button>
            {/each}
          </div>
          {#if complete}<p class="help">{m.plan_complete()}</p>{:else}<p class="help">{m.tray_click()}</p>{/if}
        </div>
      {/if}
    </section>

    <div class="lateral">
      {#if selection}
        <PanelBed {view} bed={selection} onclose={() => (selectionId = null)} onmodify={modify} />
      {:else if account}
        <aside class="summary">
          <h2>{m.summary_title()}</h2>
          <dl>
            <div><dd>{m.surface_m2({ value: formatM2(account.total) })}</dd><dt>{m.summary_plot()}</dt></div>
            <div><dd>{m.surface_m2({ value: formatM2(account.cultivable) })}</dd><dt>{m.summary_workable()}</dt></div>
            <div><dd>{m.surface_m2({ value: formatM2(account.aisles) })}</dd><dt>{m.summary_aisles()}</dt></div>
            <div><dd>{account.count} / {MAX_BEDS}</dd><dt>{m.summary_beds()}</dt></div>
          </dl>
        </aside>
      {/if}
    </div>
  </div>
{/if}

{#if shortening}
  {@const request = shortening}
  <Confirmation
    title={m.shorten_title({ name: request.bed.name })}
    confirm={m.shorten_button()}
    tone="alert"
    onconfirm={() => request.answer(true)}
    oncancel={() => request.answer(false)}
  >
    <p>{m.shorten_text({ count: request.count })}</p>
  </Confirmation>
{/if}

<style>
  .bandeau {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .year {
    display: inline-flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 8px 14px;
    border-radius: var(--pill);
    background: var(--white);
    border: 1px solid var(--line);
    font-size: 14px;
  }

  .scope {
    color: var(--muted);
  }

  @media (max-width: 480px) {
    .scope {
      flex-basis: 100%;
    }
    .scope .dot {
      display: none;
    }
  }

  .space {
    flex: 1;
  }

  .editor {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 340px;
    gap: 24px;
    align-items: start;
  }

  .scene {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 20px;
    border-radius: 32px;
    background: var(--card);
  }

  .plot {
    margin: 0;
    align-self: flex-end;
    font-size: 13px;
    color: var(--muted);
  }

  .help {
    margin: 0;
    font-size: 13px;
    color: var(--muted);
  }

  .tips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 18px;
    margin: 0;
    font-size: 13px;
    color: var(--muted);
  }

  .tips b {
    padding: 2px 10px;
    border-radius: var(--pill);
    background: var(--white);
    color: var(--ink);
    font-size: 12px;
  }

  .planter {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px;
    border-radius: 24px;
    background: var(--white);
  }

  .planter-title {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 8px;
  }

  .counter {
    font-size: 13px;
    font-weight: 700;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  .elements {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 8px;
  }

  .element {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    min-height: 72px;
    border-radius: 16px;
    border: 1px solid transparent;
    background: var(--bg);
    touch-action: none;
    user-select: none;
    -webkit-user-select: none;
    cursor: grab;
  }

  .element .name {
    font-weight: 700;
    font-size: 14px;
  }

  .element .size {
    font-size: 12px;
    color: var(--muted);
  }

  .element.scope {
    border: 1px dashed var(--green);
    background: #e7f4ec;
  }

  .element:disabled {
    cursor: default;
  }

  .lateral {
    position: sticky;
    top: 12px;
  }

  .summary {
    padding: 20px;
    border-radius: 28px;
    background: var(--white);
  }

  .summary h2 {
    margin-bottom: 14px;
    font-size: 20px;
  }

  dl {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
    margin: 0;
  }

  dl div {
    display: flex;
    flex-direction: column-reverse;
  }

  dd {
    margin: 0;
    font-family: var(--font-title);
    font-weight: 600;
    font-size: 22px;
    font-variant-numeric: tabular-nums;
  }

  dt {
    font-size: 12px;
    color: var(--muted);
  }

  @media (max-width: 1000px) {
    .editor {
      grid-template-columns: minmax(0, 1fr);
    }

    .lateral {
      position: static;
    }
  }

  @media (max-width: 720px) {
    .scene {
      padding: 12px;
      border-radius: 24px;
    }

    .elements {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
</style>
