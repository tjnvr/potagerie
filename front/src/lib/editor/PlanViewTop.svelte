<script lang="ts">
  // Top view of the plot and its beds (E03), in centimetres: selection,
  // drag with Pointer Events (mouse, pen, finger after a long press), the 8
  // handles. The bed follows the pointer; Rust judges at the drop and the
  // bed goes back to its place if it refuses (S05 §4).
  import type { BedView, PlotView, TypeBed } from "../bindings";
  import { m } from "../paraglide/messages.js";
  import { formatMeters } from "../beds";
  import {
    cmPerPixel,
    ControllerDrag,
    deposer,
    HANDLE_HIT_MOUSE_PX,
    HANDLE_HIT_TOUCH_PX,
    handlePoint,
    handleZones,
    pixelsToCm,
    positionDropped,
    resize,
    type Box,
    type Geo,
    type Point,
    type Handle,
    type Response,
    type TypePointer,
  } from "./drag";

  /** The shadow of a bed carried from the tray. */
  export type Ghost = Geo & { type: TypeBed };

  let {
    plot,
    beds,
    selectionId = null,
    readOnly = false,
    ghost = null,
    onselect = () => {},
    onmodify = async () => "ok" as Response,
    onbeforeplot = () => {},
    onplace = () => {},
    ondraw = () => {},
  }: {
    plot: PlotView;
    beds: BedView[];
    selectionId?: string | null;
    readOnly?: boolean;
    ghost?: Ghost | null;
    onselect?: (id: string | null) => void;
    /** Move or resize: the new geometry of the bed, judged by Rust. */
    onmodify?: (id: string, geo: Geo, nature: "move" | "resize") => Promise<Response>;
    /** The drop is left of or above the plot: nothing to send, refused. */
    onbeforeplot?: () => void;
    /** A click on free grass, with nothing selected: put a bed there. */
    onplace?: (point: Point) => void;
    /** A zone drawn by dragging on free grass, snapped to the cases. */
    ondraw?: (geo: Geo) => void;
  } = $props();

  // A click on the grass first clears the selection; only a click made with
  // nothing selected puts a bed down.
  let selectionBeforeClick: string | null = null;
  // Drawing a zone: a mouse press on free grass then a drag draws a rectangle
  // snapped to the cases, shown from the first pixel (no drag threshold).
  let bandStart: Point | null = null;
  let band = $state<Geo | null>(null);
  let drew = false;
  function pressGrass(e: PointerEvent) {
    if (geste) return;
    selectionBeforeClick = selectionId;
    onselect(null);
    drew = false;
    bandStart = !readOnly && e.pointerType === "mouse" && e.button === 0 ? (versCm(e.clientX, e.clientY) ?? null) : null;
  }
  function growBand(e: PointerEvent) {
    if (!bandStart) return;
    const p = versCm(e.clientX, e.clientY);
    if (!p) return;
    const c = plot.cell_cm;
    const x0 = Math.floor(Math.min(bandStart.x, p.x) / c) * c;
    const y0 = Math.floor(Math.min(bandStart.y, p.y) / c) * c;
    const x1 = Math.ceil(Math.max(bandStart.x, p.x) / c) * c;
    const y1 = Math.ceil(Math.max(bandStart.y, p.y) / c) * c;
    const moved = Math.hypot(p.x - bandStart.x, p.y - bandStart.y) > c / 4;
    if (!moved && !band) return;
    band = { x: x0, y: y0, length_cm: Math.min(x1, plot.length_cm) - x0, width_cm: Math.min(y1, plot.width_cm) - y0 };
  }
  function endBand() {
    const zone = band;
    band = null;
    bandStart = null;
    if (zone && zone.length_cm >= plot.cell_cm && zone.width_cm >= plot.cell_cm) {
      drew = true;
      ondraw(zone);
    }
  }
  function clickGrass(e: MouseEvent) {
    if (drew) {
      drew = false;
      return;
    }
    if (readOnly || selectionBeforeClick !== null) return;
    const point = versCm(e.clientX, e.clientY);
    if (point) onplace(point);
  }

  type Geste = {
    id: string;
    nature: "move" | "resize";
    origin: Geo;
    shift: Point;
    /** The pointer that holds the bed: another finger is not part of the gesture. */
    pointerId: number;
    handle?: Handle;
    preview: Geo;
    sending: boolean;
  };

  const CURSORS: Record<Handle, string> = {
    nw: "nwse-resize",
    n: "ns-resize",
    ne: "nesw-resize",
    e: "ew-resize",
    se: "nwse-resize",
    s: "ns-resize",
    sw: "nesw-resize",
    w: "ew-resize",
  };

  // Touch targets of 40 px at least (S05 §19): on a touch screen, or as soon
  // as a finger has touched the plan.
  const coarse = typeof matchMedia === "function" && matchMedia("(any-pointer: coarse)").matches;
  let touched = $state(false);
  const hitPx = $derived(coarse || touched ? HANDLE_HIT_TOUCH_PX : HANDLE_HIT_MOUSE_PX);

  let svg: SVGSVGElement;
  let widthPx = $state(600);
  let heightPx = $state(360);
  let geste = $state<Geste | null>(null);
  let refuse = $state<string | null>(null);
  let armed = $state<string | null>(null);

  const marge = $derived(Math.max(plot.length_cm, plot.width_cm) * 0.02);
  const police = $derived(Math.max(plot.length_cm, plot.width_cm) / 60);
  const box = $derived<Box>({
    x: -marge,
    y: -marge,
    width: plot.length_cm + 2 * marge,
    height: plot.width_cm + 2 * marge,
  });
  /** Centimetres per screen pixel, to keep handles the same size on screen. */
  const scale = $derived(cmPerPixel({ left: 0, top: 0, width: widthPx, height: heightPx }, box));
  const rowsX = $derived(
    Array.from({ length: Math.floor(plot.length_cm / plot.cell_cm) - 1 }, (_, i) => (i + 1) * plot.cell_cm),
  );
  const rowsY = $derived(
    Array.from({ length: Math.floor(plot.width_cm / plot.cell_cm) - 1 }, (_, i) => (i + 1) * plot.cell_cm),
  );
  /** The bed in hand is drawn last, above the others. */
  const sorted = $derived([...beds].sort((a, b) => Number(a.id === geste?.id) - Number(b.id === geste?.id)));
  const active = $derived(beds.find((p) => p.id === selectionId) ?? null);
  const stepFromMeter = $derived(100 / plot.cell_cm);

  const controller = new ControllerDrag({ onArmed: () => (armed = geste?.id ?? null) });

  /** Pixels of the screen to centimetres of the plan, or `null` outside the
   *  drawing (the element may be wider than it, see `pixelsToCm`). */
  export function versCm(clientX: number, clientY: number): Point | null {
    const p = pixelsToCm({ x: clientX, y: clientY }, svg.getBoundingClientRect(), box);
    const inside = p.x >= box.x && p.x <= box.x + box.width && p.y >= box.y && p.y <= box.y + box.height;
    return inside ? p : null;
  }

  function enCm(e: PointerEvent): Point {
    return pixelsToCm({ x: e.clientX, y: e.clientY }, svg.getBoundingClientRect(), box);
  }

  function geoOf(p: BedView): Geo {
    return { x: p.x, y: p.y, length_cm: p.length_cm, width_cm: p.width_cm };
  }

  function start(e: PointerEvent, p: BedView, nature: Geste["nature"], handle?: Handle) {
    // A second finger on the plan leaves the bed in hand to the first one.
    if (e.button !== 0 || geste?.sending || (geste && geste.pointerId !== e.pointerId)) return;
    e.stopPropagation();
    svg.setPointerCapture(e.pointerId);
    const origin = geoOf(p);
    const pointer = enCm(e);
    // Held by the top left of the bed for a move, by the handle for a resize.
    const held = handle ? handlePoint(origin, handle) : { x: p.x, y: p.y };
    geste = {
      id: p.id,
      nature,
      origin,
      shift: { x: pointer.x - held.x, y: pointer.y - held.y },
      pointerId: e.pointerId,
      handle,
      preview: origin,
      sending: false,
    };
    armed = null;
    controller.press(e.pointerType as TypePointer, { x: e.clientX, y: e.clientY }, { immediate: nature === "resize" });
  }

  function bouge(e: PointerEvent) {
    if (!geste || geste.sending || e.pointerId !== geste.pointerId) return;
    const movement = controller.move({ x: e.clientX, y: e.clientY });
    if (movement === "abandon") {
      geste = null;
      return;
    }
    if (movement === "start" && selectionId !== geste.id) onselect(geste.id);
    if (movement === "start" || movement === "dragged") {
      const pointer = enCm(e);
      const step = plot.cell_cm;
      geste.preview =
        geste.nature === "move"
          ? {
              ...geste.origin,
              ...positionDropped(pointer, geste.shift, step),
            }
          : resize(geste.origin, geste.handle!, pointer, step, geste.shift);
    }
  }

  async function raised(e: PointerEvent) {
    if (!geste || geste.sending || e.pointerId !== geste.pointerId) return;
    if (svg.hasPointerCapture(e.pointerId)) svg.releasePointerCapture(e.pointerId);
    const end = controller.release();
    armed = null;
    const courant = geste;
    if (end !== "end") {
      geste = null;
      if (end === "click") onselect(courant.id);
      return;
    }
    courant.sending = true;
    const issue = await deposer(courant.origin, courant.preview, (g) =>
      onmodify(courant.id, g, courant.nature),
    );
    if (issue.response === "before-plot") onbeforeplot();
    geste = null;
    if (issue.response === "refusal" || issue.response === "before-plot") {
      refuse = courant.id;
      setTimeout(() => (refuse = null), 1200);
    }
  }

  function cancelled(e: PointerEvent) {
    if (geste && e.pointerId !== geste.pointerId) return;
    controller.cancel();
    armed = null;
    if (!geste?.sending) geste = null;
  }

  async function key(e: KeyboardEvent, p: BedView) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onselect(p.id);
      return;
    }
    if (e.key === "Escape") {
      onselect(null);
      return;
    }
    const step = plot.cell_cm;
    const delta: Record<string, Point> = {
      ArrowLeft: { x: -step, y: 0 },
      ArrowRight: { x: step, y: 0 },
      ArrowUp: { x: 0, y: -step },
      ArrowDown: { x: 0, y: step },
    };
    const d = delta[e.key];
    if (!d || readOnly || selectionId !== p.id) return;
    e.preventDefault();
    const target = { ...geoOf(p), x: p.x + d.x, y: p.y + d.y };
    await deposer(geoOf(p), target, (g) => onmodify(p.id, g, "move")).then((issue) => {
      if (issue.response === "before-plot") onbeforeplot();
    });
  }

  /** Chromium moves a touch to the nearest element that listens for mouse
   *  presses on itself; Svelte delegates its listeners to the document, so a
   *  handle's press would go to the bed under it. A listener on the handle
   *  keeps the finger on it (S05 §19). */
  function tappable(node: Element) {
    const nothing = () => {};
    node.addEventListener("mousedown", nothing);
    return () => node.removeEventListener("mousedown", nothing);
  }

  // A finger that holds a bed must not scroll the page: `touch-action`
  // is not honoured on every SVG engine, so the touchmove is stopped too.
  $effect(() => {
    const stop = (e: TouchEvent) => {
      const resizing = geste?.nature === "resize" && controller.state !== "rest";
      if (resizing || controller.state === "armed" || controller.state === "dragged") e.preventDefault();
    };
    svg.addEventListener("touchmove", stop, { passive: false });
    return () => svg.removeEventListener("touchmove", stop);
  });

  // A long press with a finger or a pen grabs a bed: it must not open the
  // context menu (Windows touch screens, Android). The mouse keeps its own.
  let pressedWith = "mouse";
</script>

<svg
  bind:this={svg}
  bind:clientWidth={widthPx}
  bind:clientHeight={heightPx}
  class="plan"
  viewBox="{box.x} {box.y} {box.width} {box.height}"
  role="group"
  aria-label={m.plan_aria({ count: beds.length })}
  onpointerdowncapture={(e) => (pressedWith = e.pointerType)}
  oncontextmenu={(e) => pressedWith !== "mouse" && e.preventDefault()}
  onpointermove={(e) => {
    growBand(e);
    bouge(e);
  }}
  onpointerup={(e) => {
    endBand();
    raised(e);
  }}
  onpointercancel={cancelled}
  onpointerdowncapture={(e) => {
    if (e.pointerType === "touch") touched = true;
  }}
>
  <rect
    class="grass"
    width={plot.length_cm}
    height={plot.width_cm}
    rx={plot.cell_cm / 3}
    fill="var(--grass)"
    role="presentation"
    onpointerdown={pressGrass}
    onclick={clickGrass}
  />
  {#each rowsX as x, i (x)}
    <line x1={x} x2={x} y1="0" y2={plot.width_cm} class="grille" class:metre={(i + 1) % stepFromMeter === 0} />
  {/each}
  {#each rowsY as y, i (y)}
    <line y1={y} y2={y} x1="0" x2={plot.length_cm} class="grille" class:metre={(i + 1) % stepFromMeter === 0} />
  {/each}

  {#each sorted as bed (bed.id)}
    {@const g = geste?.id === bed.id ? geste.preview : geoOf(bed)}
    {@const bois = bed.bed_type === "planter" || bed.bed_type === "square"}
    {@const chosen = selectionId === bed.id}
    {#if geste?.id === bed.id && geste.nature === "move"}
      <rect
        class="origin"
        x={geste.origin.x}
        y={geste.origin.y}
        width={geste.origin.length_cm}
        height={geste.origin.width_cm}
        rx="6"
      />
    {/if}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <g
      class="bed"
      class:chosen
      class:refuse={refuse === bed.id}
      class:armed={armed === bed.id}
      class:free={readOnly}
      role="button"
      tabindex="0"
      aria-pressed={chosen}
      aria-label={m.bed_aria({
        name: bed.name,
        length: formatMeters(g.length_cm),
        width: formatMeters(g.width_cm),
      })}
      onpointerdown={(e) => (readOnly ? onselect(bed.id) : start(e, bed, "move"))}
      onkeydown={(e) => key(e, bed)}
    >
      <rect x={g.x} y={g.y} width={g.length_cm} height={g.width_cm} rx="6" fill={bois ? "var(--wood)" : "var(--bed)"} />
      {#if bois}
        <rect x={g.x + 6} y={g.y + 6} width={Math.max(g.length_cm - 12, 0)} height={Math.max(g.width_cm - 12, 0)} rx="3" fill="var(--bed)" />
      {:else}
        {#each Array.from({ length: Math.max(Math.floor(g.width_cm / 25) - 1, 0) }, (_, k) => k + 1) as k (k)}
          <line x1={g.x + 8} x2={g.x + g.length_cm - 8} y1={g.y + k * 25} y2={g.y + k * 25} class="sillon" />
        {/each}
      {/if}
      {#if refuse === bed.id}
        <rect class="alert" x={g.x} y={g.y} width={g.length_cm} height={g.width_cm} rx="6" />
      {/if}
      <text class="etiquette" x={g.x + 10} y={g.y + 10 + police} font-size={police}>{bed.name}</text>
    </g>
  {/each}

  {#if active}
    {@const g = geste?.id === active.id ? geste.preview : geoOf(active)}
      <rect class="cadre" x={g.x} y={g.y} width={g.length_cm} height={g.width_cm} rx="6" />
      <text class="cote" x={g.x + g.length_cm / 2} y={g.y - police * 0.5} font-size={police} text-anchor="middle">
        {m.panel_dimensions({ length: formatMeters(g.length_cm), width: formatMeters(g.width_cm) })}
      </text>
      {#if !readOnly}
        {#each handleZones(g, scale, hitPx) as h (h.name)}
          <circle
            class="handle-zone"
            role="presentation"
            data-handle={h.name}
            cx={h.cx}
            cy={h.cy}
            r={h.r}
            style:cursor={CURSORS[h.name]}
            onpointerdown={(e) => start(e, active, "resize", h.name)}
            {@attach tappable}
          />
          <circle class="handle" cx={h.x} cy={h.y} r={6 * scale} />
        {/each}
      {/if}
    {/if}

  {#if band}
    <rect class="ghost" x={band.x} y={band.y} width={band.length_cm} height={band.width_cm} rx="6" />
  {/if}
  {#if ghost}
    <rect
      class="ghost"
      x={ghost.x}
      y={ghost.y}
      width={ghost.length_cm}
      height={ghost.width_cm}
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
    pointer-events: none;
    stroke: #ffffff;
    stroke-opacity: 0.25;
    stroke-width: 1;
    vector-effect: non-scaling-stroke;
    pointer-events: none;
  }

  .grille.metre {
    stroke-opacity: 0.55;
  }

  .bed {
    cursor: grab;
    touch-action: none;
  }

  .bed.free {
    cursor: pointer;
    touch-action: auto;
  }

  .bed.armed {
    filter: drop-shadow(0 0 6px rgba(242, 166, 90, 0.9));
  }

  .bed:focus-visible {
    outline: none;
  }

  .bed:focus-visible rect:first-child {
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
    font-family: var(--font-title);
    font-weight: 600;
    fill: #ffffff;
    paint-order: stroke;
    stroke: rgba(31, 58, 46, 0.55);
    stroke-width: 3px;
    stroke-linejoin: round;
    vector-effect: non-scaling-stroke;
    pointer-events: none;
  }

  .alert {
    fill: rgba(200, 68, 60, 0.45);
    stroke: var(--crit);
    stroke-width: 2px;
    vector-effect: non-scaling-stroke;
  }

  .origin {
    fill: none;
    stroke: var(--white);
    stroke-width: 2px;
    stroke-dasharray: 6 4;
    vector-effect: non-scaling-stroke;
    pointer-events: none;
  }

  .ghost {
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

  .handle {
    fill: var(--white);
    stroke: var(--accent);
    stroke-width: 2px;
    vector-effect: non-scaling-stroke;
    pointer-events: none;
  }

  .handle-zone {
    fill: transparent;
    touch-action: none;
  }
</style>
