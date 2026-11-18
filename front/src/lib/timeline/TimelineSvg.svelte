<script lang="ts" module>
  import { localeTag } from "../locale";
  /** Layout constants, shared with the controller (pointer to row, export). */
  export const WIDTH_LABELS = 172;
  export const WIDTH_LABELS_NARROW = 84;
  export const HEIGHT_HEADER = 62;
  export const HEIGHT_LANE = 34;
  /** Height of the invisible tap target of a bar or a free-slot chip. */
  const HIT = 40;
  export const MARGE_ROW = 10;

  export function heightRow(lanes: number): number {
    return lanes * HEIGHT_LANE + MARGE_ROW;
  }
</script>

<script lang="ts">
  // The drawing of the timeline (S01 F-VIS-2, D01 E05): months, one row per
  // bed, bars cut into phases, free slots, today and the cursor.
  //
  // It is a standalone SVG on purpose (S05 §4, export): every colour, font and
  // pattern is an attribute of the drawing, never a CSS rule, so the same
  // markup serialised by `serializeTimeline` renders alike in resvg. CSS here only
  // adds the pointer feedback (cursor, focus), which the export drops.
  import { m } from "../paraglide/messages.js";
  import { dateAbbreviated, dateOf, dayOf } from "../dates";
  import { formatMeters, nameCrop, nameOfType } from "../beds";
  import { PALETTE, colorCrop, melange } from "./colors";
  import { rankSuggestions } from "../suggestions";
  import type { AnnotationView } from "../bindings";
  import { monthVisible, xOf, type Bar, type Scale, type Row } from "./geometry";

  let {
    rows,
    scale,
    widthLabels = WIDTH_LABELS,
    today,
    cursor = null,
    interactive = true,
    shifts = {},
    refusal = null,
    scope = null,
    yearShown,
    selectionBedId = null,
    annotations = [],
    markerToday = true,
    phases = true,
  }: {
    rows: Row[];
    scale: Scale;
    widthLabels?: number;
    /** `YYYY-MM-DD`. */
    today: string;
    /** The cursor of the time, or `null` (export). */
    cursor?: string | null;
    interactive?: boolean;
    /** Pixel offset of the bars being dragged, by id. */
    shifts?: Record<string, number>;
    /** The bar Rust refused to move: outlined for a moment. */
    refusal?: string | null;
    /** The bar carried by a finger or a mouse: lifted. */
    scope?: string | null;
    yearShown?: number;
    selectionBedId?: string | null;
    /** Notes and periods to draw (S01 F-VIS-5); the caller already applied « Show the annotations ». */
    annotations?: AnnotationView[];
    /** The dashed « today » marker (an option of the export). */
    markerToday?: boolean;
    /** Pots, growth and harvest told apart; off, the bar is one solid block (an option of the export). */
    phases?: boolean;
  } = $props();

  const width = $derived(scale.width + widthLabels);
  const dayToday = $derived(dayOf(today));
  const dayCursor = $derived(cursor ? dayOf(cursor) : null);

  const positions = $derived.by(() => {
    let y = HEIGHT_HEADER;
    return rows.map((row) => {
      const top = y;
      const height = heightRow(row.lanes);
      y += height + 1;
      return { top, height };
    });
  });
  const heightTotal = $derived(positions.length ? positions[positions.length - 1]!.top + positions[positions.length - 1]!.height + 1 : HEIGHT_HEADER + 1);

  const month = $derived(monthVisible(scale));
  const compact = $derived(widthLabels < WIDTH_LABELS);

  /** The name of a bed on at most two lines that fit the label column: words are
   *  kept whole when possible, and what still does not fit ends with « … ». */
  function nameLines(name: string, columnWidth: number, fontSize: number): string[] {
    const max = Math.max(6, Math.floor(columnWidth / (fontSize * 0.52)));
    // Counted in characters, not in UTF-16 units, so an emoji is never cut in two.
    const count = (t: string) => Array.from(t).length;
    const cut = (t: string) => (count(t) > max ? Array.from(t).slice(0, max - 1).join("") + "…" : t);
    const clean = name.trim().replace(/\s+/g, " ");
    if (count(clean) <= max) return [clean];
    const words = clean.split(" ");
    let first = "";
    let i = 0;
    for (; i < words.length; i++) {
      const next = first ? first + " " + words[i] : (words[i] ?? "");
      if (count(next) > max) break;
      first = next;
    }
    if (!first) return [cut(clean)];
    return [first, cut(words.slice(i).join(" "))].filter(Boolean);
  }
  /** Month names that fit their column: three letters, one letter when narrow, none when tiny. */
  const namesOfMonth = $derived(
    month.map((mo) => {
      const place = mo.days * scale.pxByDay - 8;
      if (place < 9) return "";
      const name = new Date(Date.UTC(mo.year, mo.month, 1)).toLocaleDateString(localeTag(), { month: "short", timeZone: "UTC" }).replace(".", "");
      return place < 34 ? name.slice(0, 1) : name;
    }),
  );

  /** Years to label: the visible part of each one, if it is wide enough to read. */
  const years = $derived.by(() => {
    const visible = [...new Set(month.map((mo) => mo.year))];
    return visible
      .map((year) => {
        const start = xOf(scale, dayOf(`${year}-01-01`));
        const end = xOf(scale, dayOf(`${year + 1}-01-01`));
        const gauche = Math.max(start, 0);
        return { year, x: Math.min(gauche + 6, end - 48) + widthLabels, visible: Math.min(end, scale.width) - gauche };
      })
      .filter((a) => a.visible >= 54);
  });

  const motifs = $derived([...new Set(rows.flatMap((l) => l.bars.map((b) => b.cropId)))]);

  function x(day: number): number {
    return xOf(scale, day) + widthLabels;
  }

  function nameOfBar(b: Bar): string {
    return nameCrop(b.cropId);
  }

  function titleOfBar(b: Bar): string {
    const end = dateAbbreviated(dateOf(b.end - 1), yearShown);
    const base = m.timeline_bar_title({ crop: nameOfBar(b), start: dateAbbreviated(dateOf(b.start), yearShown), end });
    return [base, b.manualEnd ? m.timeline_manual_end() : "", b.enConflict ? m.timeline_conflict() : ""].filter(Boolean).join(" · ");
  }

  /** Notes and periods in view, with the row they belong to (`null`: the whole garden). A hidden one is only ghosted on screen. */
  const notes = $derived(
    annotations
      .filter((a) => (interactive || !a.hidden) && (a.bed_id === null || rows.some((l) => l.bed.id === a.bed_id)))
      .map((a) => {
        const i = a.bed_id === null ? -1 : rows.findIndex((l) => l.bed.id === a.bed_id);
        const start = dayOf(a.date_start);
        const end = a.annotation_type === "period" ? dayOf(a.date_end ?? a.date_start) + 1 : start;
        const top = i < 0 ? HEIGHT_HEADER : positions[i]!.top;
        const low = i < 0 ? heightTotal : top + positions[i]!.height;
        return { a, start, end, top, low, period: a.annotation_type === "period", ghost: a.hidden };
      })
      .filter((n) => x(n.end) >= widthLabels && x(n.start) <= width)
      .map((n, _, all) => {
        // The label of a note stops where the next label of the same row starts, so two close notes never overprint.
        const next = all.filter((o) => o !== n && o.top === n.top && (o.start > n.start || (o.start === n.start && o.a.id > n.a.id)));
        const room = next.length ? Math.min(...next.map((o) => x(o.start))) - (n.period ? Math.max(x(n.start), widthLabels) + 6 : x(n.start) + 9) - 8 : Infinity;
        return { ...n, room };
      }),
  );
  function titleOfNote(a: AnnotationView): string {
    const when =
      a.annotation_type === "period" && a.date_end
        ? m.note_aria_period({ start: dateAbbreviated(a.date_start, yearShown), end: dateAbbreviated(a.date_end, yearShown) })
        : dateAbbreviated(a.date_start, yearShown);
    return `${a.text} · ${when}`;
  }
  function durationDays(end: number, start: number): number {
    return end - start;
  }
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={width}
  height={heightTotal}
  viewBox="0 0 {width} {heightTotal}"
  font-family="Figtree, system-ui, sans-serif"
  role="group"
  aria-label={m.timeline_aria()}
  data-timeline
>
  <defs>
    <clipPath id="timeline-zone"><rect x={widthLabels} y="0" width={scale.width} height={heightTotal} /></clipPath>
    <clipPath id="timeline-labels"><rect x="0" y="0" width={widthLabels - 8} height={heightTotal} /></clipPath>
    {#each motifs as crop (crop)}
      {@const color = colorCrop(crop)}
      <!-- Pots: diagonal stripes. Growth: pale with dots. Harvest: solid. The
           phases differ by pattern, not only by colour (S01 F-VIS-2). -->
      <pattern id="timeline-pots-{crop}" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
        <rect width="8" height="8" fill={melange(color, 0.85)} />
        <rect width="4" height="8" fill={melange(color, 0.55)} />
      </pattern>
      <pattern id="timeline-growth-{crop}" width="8" height="8" patternUnits="userSpaceOnUse">
        <rect width="8" height="8" fill={melange(color, 0.45)} />
        <circle cx="4" cy="4" r="0.9" fill={melange(color, 0.2)} />
      </pattern>
    {/each}
    {#each rows as row, i (row.bed.id)}
      {#each row.bars as b (b.id)}
        <clipPath id="timeline-bar-{b.id}">
          <rect
            x={x(b.start) + (shifts[b.id] ?? 0)}
            y={positions[i]!.top + b.lane * HEIGHT_LANE + 8}
            width={Math.max(durationDays(b.end, b.start) * scale.pxByDay, 1)}
            height="26"
            rx="9"
          />
        </clipPath>
      {/each}
    {/each}
    <pattern id="timeline-absence" width="10" height="10" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
      <rect width="10" height="10" fill={PALETTE.absence} />
      <rect width="1.5" height="10" fill={PALETTE.trait} />
    </pattern>
  </defs>

  <rect width={width} height={heightTotal} fill={PALETTE.fond} />

  <g clip-path="url(#timeline-zone)">
    <!-- Rows: background, greyed periods where the bed does not exist. -->
    {#each rows as row, i (row.bed.id)}
      {@const p = positions[i]!}
      <rect x={widthLabels} y={p.top} width={scale.width} height={p.height} fill={row.bed.id === selectionBedId ? PALETTE.selection : PALETTE.fond} />
      {#each row.absences as a (a.start)}
        <rect x={x(a.start)} y={p.top} width={Math.max((a.end - a.start) * scale.pxByDay, 0)} height={p.height} fill="url(#timeline-absence)" />
      {/each}
      <line x1={widthLabels} x2={width} y1={p.top - 0.5} y2={p.top - 0.5} stroke={PALETTE.trait} />
    {/each}

    <!-- Month columns. -->
    {#each month as mo, i (mo.day)}
      <line x1={x(mo.day)} x2={x(mo.day)} y1={mo.month === 0 ? 20 : 40} y2={heightTotal} stroke={PALETTE.trait} stroke-width={mo.month === 0 ? 2 : 1} />
      <text x={x(mo.day) + 4} y={HEIGHT_HEADER - 8} font-size="11" font-weight="600" letter-spacing="0.4" fill={PALETTE.muted}>{namesOfMonth[i]?.toUpperCase()}</text>
    {/each}
    {#each years as a (a.year)}
      <text x={a.x} y="38" font-size="18" font-weight="600" font-family="Fredoka, Figtree, sans-serif" fill={PALETTE.ink}>{a.year}</text>
    {/each}

    <!-- Periods: a coloured band behind the bars (F-VIS-5). -->
    {#each notes.filter((n) => n.period) as n (n.a.id)}
      <rect
        x={x(n.start)}
        y={n.top}
        width={Math.max((n.end - n.start) * scale.pxByDay, 2)}
        height={n.low - n.top}
        fill={PALETTE.note}
        fill-opacity={n.ghost ? 0.05 : n.a.bed_id === null ? 0.1 : 0.16}
        stroke={n.ghost ? PALETTE.note : "none"}
        stroke-dasharray="4 3"
      />
      {#if n.a.bed_id === null}
        <rect x={x(n.start)} y={HEIGHT_HEADER - 5} width={Math.max((n.end - n.start) * scale.pxByDay, 2)} height="4" rx="2" fill={PALETTE.note} fill-opacity={n.ghost ? 0.35 : 1} />
      {/if}
    {/each}
    <!-- Free slots (S03 §8): dashed, with the crops Rust suggests. -->
    {#each rows as row, i (row.bed.id)}
      {@const p = positions[i]!}
      {#each row.slots as c (c.key)}
        {@const widthC = (c.end - c.start) * scale.pxByDay}
        {@const days = c.end - c.start}
        {@const partial = c.source.segment_start !== 1 || c.source.segment_end !== row.bed.segments}
        {@const names = rankSuggestions(c.source.suggestions).map((s) => nameCrop(s.crop_id)).join(", ")}
        <g
          class="slot"
          data-slot={c.key}
          role="button"
          tabindex={interactive ? 0 : undefined}
          aria-label={m.timeline_slot_aria({ bed: row.bed.name, days, start: dateAbbreviated(c.source.start, yearShown), end: dateAbbreviated(c.source.end, yearShown) })}
        >
          <!-- Invisible tap target of 40 px (the chip is 26 px high). -->
          <rect x={x(c.start) + 1} y={p.top + c.lane * HEIGHT_LANE + 1} width={Math.max(widthC - 2, 0)} height={HIT} fill="transparent" />
          <rect
            x={x(c.start) + 1}
            y={p.top + c.lane * HEIGHT_LANE + 8}
            width={Math.max(widthC - 2, 0)}
            height="26"
            rx="9"
            fill={PALETTE.slotFond}
            stroke={PALETTE.accent}
            stroke-width="2"
            stroke-dasharray="6 4"
          />
          {#if x(c.end) - Math.max(x(c.start), widthLabels) > 70}
          <svg x={Math.max(x(c.start), widthLabels) + 10} y={p.top + c.lane * HEIGHT_LANE + 8} width={Math.max(x(c.end) - Math.max(x(c.start), widthLabels) - 14, 0)} height="26" overflow="hidden">
            <text y="17" font-size="12" fill={PALETTE.accentInk}>
              <tspan font-weight="700">{m.timeline_slot({ days })}</tspan>
              {#if partial}<tspan> · {m.timeline_slot_segments({ start: c.source.segment_start, end: c.source.segment_end })}</tspan>{/if}
              {#if names}<tspan> · {m.timeline_slot_suggestions({ crops: names })}</tspan>{/if}
            </text>
          </svg>
          {/if}
        </g>
      {/each}
    {/each}

    <!-- Bars. -->
    {#each rows as row, i (row.bed.id)}
      {@const p = positions[i]!}
      {#each row.bars as b (b.id)}
        {@const dx = shifts[b.id] ?? 0}
        {@const top = p.top + b.lane * HEIGHT_LANE + 8}
        {@const gauche = x(b.start) + dx}
        {@const widthB = Math.max((b.end - b.start) * scale.pxByDay, 1)}
        <g
          class="bar"
          class:scope={scope === b.id}
          data-bar={b.id}
          role="button"
          tabindex={interactive ? 0 : undefined}
          aria-label={titleOfBar(b)}
        >
          <title>{titleOfBar(b)}</title>
          <!-- Invisible tap target of 40 px (the bar is 26 px high). -->
          <rect x={gauche} y={top - 7} width={widthB} height={HIT} fill="transparent" />
          <rect x={gauche} y={top + 1} width={widthB} height="26" rx="9" fill={PALETTE.ink} opacity={scope === b.id ? 0.25 : 0.12} />
          <g clip-path="url(#timeline-bar-{b.id})" transform="translate(0 0)">
            {#each b.segments as s (s.phase)}
              <rect
                x={x(s.start) + dx}
                y={top}
                width={Math.max((s.end - s.start) * scale.pxByDay, 0)}
                height="26"
                fill={!phases ? colorCrop(b.cropId) : s.phase === "pots" ? `url(#timeline-pots-${b.cropId})` : s.phase === "growth" ? `url(#timeline-growth-${b.cropId})` : colorCrop(b.cropId)}
              />
            {/each}
          </g>
          <rect
            x={gauche + 1}
            y={top + 1}
            width={Math.max(widthB - 2, 0)}
            height="24"
            rx="8"
            fill="none"
            stroke={refusal === b.id || b.enConflict ? PALETTE.crit : "none"}
            stroke-width="3"
          />
          <svg x={Math.max(gauche, widthLabels) + 9} y={top} width={Math.max(gauche + widthB - Math.max(gauche, widthLabels) - 12, 0)} height="26" overflow="hidden">
            {#if b.mode === "plants"}
              <!-- The « plant » icon of bought plants (F-VIS-2). -->
              <g fill="none" stroke={PALETTE.ink} stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" transform="translate(0 6)">
                <path d="M7 14V7M7 8C7 5 5 3.5 2.5 3.5 2.5 6 4 8 7 8ZM7 7C7 4.5 8.8 3 11.5 3 11.5 5.5 10 7 7 7Z" />
              </g>
            {/if}
            <text x={b.mode === "plants" ? 18 : 0} y="17" font-size="12" font-weight="600" fill={PALETTE.ink}>{nameOfBar(b)}{#if b.manualEnd}<tspan font-weight="400">  ✎</tspan>{/if}</text>
          </svg>
        </g>
      {/each}
    {/each}

    <!-- Notes and the names of the periods (F-VIS-5): what is clickable on screen. -->
    {#each notes as n (n.a.id)}
      {@const gauche = Math.max(x(n.start), widthLabels)}
      {@const opacite = n.ghost ? 0.4 : 1}
      <g
        class="note"
        data-annotation={n.a.id}
        role="button"
        tabindex={interactive ? 0 : undefined}
        aria-label={titleOfNote(n.a)}
        opacity={opacite}
      >
        <title>{titleOfNote(n.a)}</title>
        {#if !n.period}
          <line x1={x(n.start)} x2={x(n.start)} y1={n.top + 4} y2={n.low} stroke={PALETTE.note} stroke-width="1.5" stroke-dasharray="3 3" />
          <circle cx={x(n.start)} cy={n.top + (n.a.bed_id === null ? 0 : 7)} r="5" fill={PALETTE.note} stroke={PALETTE.fond} stroke-width="1.5" />
        {/if}
        <svg x={n.period ? gauche + 6 : x(n.start) + 9} y={n.top} width={Number.isFinite(n.room) ? Math.max(n.room, 0) : 4000} height="16" overflow="hidden">
        <text
          x="0"
          y="10"
          font-size="10"
          font-weight="600"
          fill={PALETTE.noteInk}
          stroke={PALETTE.fond}
          stroke-width="3"
          stroke-linejoin="round"
          paint-order="stroke"
          pointer-events="all">{n.a.text}</text
        >
        </svg>
      </g>
    {/each}
    <!-- Today (background marker) and the cursor of the time (F-VIS-3). -->
    {#if markerToday}
      <line x1={x(dayToday)} x2={x(dayToday)} y1="40" y2={heightTotal} stroke={PALETTE.accent} stroke-width="1.5" stroke-dasharray="3 3" />
    {/if}
    {#if dayCursor !== null}
      <g data-cursor>
        <line x1={x(dayCursor)} x2={x(dayCursor)} y1="16" y2={heightTotal} stroke={PALETTE.accent} stroke-width="2" />
        <rect x={x(dayCursor) - 34} y="0" width="68" height="20" rx="10" fill={PALETTE.accent} />
        <text x={x(dayCursor)} y="14" font-size="11" font-weight="700" text-anchor="middle" fill={PALETTE.accentInk}>{dateAbbreviated(dateOf(dayCursor), yearShown)}</text>
      </g>
    {/if}
  </g>

  <!-- Labels of the rows, drawn over what scrolls under them. -->
  <rect x="0" y="0" width={widthLabels} height={heightTotal} fill={PALETTE.fond} />
  <g clip-path="url(#timeline-labels)">
    {#each rows as row, i (row.bed.id)}
      {@const p = positions[i]!}
      {#if row.bed.id === selectionBedId}
        <rect x="0" y={p.top} width={widthLabels} height={p.height} fill={PALETTE.selection} />
        <rect x="0" y={p.top} width="3" height={p.height} fill={PALETTE.accent} />
      {/if}
      <line x1="0" x2={widthLabels} y1={p.top - 0.5} y2={p.top - 0.5} stroke={PALETTE.trait} />
      {@const size = compact ? 12 : 14}
      {@const lines = nameLines(row.bed.name, widthLabels - (compact ? 16 : 20), size)}
      <text x={compact ? 8 : 12} y={p.top + p.height / 2 + (compact ? 4 : -2) - (lines.length - 1) * (size / 2 + 1)} font-size={size} font-weight="600" fill={PALETTE.ink}>
        {#each lines as line, k (k)}<tspan x={compact ? 8 : 12} dy={k === 0 ? 0 : size + 2}>{line}</tspan>{/each}
      </text>
      {#if !compact}
        <text x="12" y={p.top + p.height / 2 + 15 + (lines.length - 1) * 8} font-size="12" fill={PALETTE.muted}>
          {m.timeline_bed_dim({
            type: nameOfType(row.bed.bed_type),
            length: formatMeters(row.bed.length_cm),
            width: formatMeters(row.bed.width_cm),
          })}
        </text>
      {/if}
    {/each}
  </g>
  <line x1="0" x2={width} y1={heightTotal - 0.5} y2={heightTotal - 0.5} stroke={PALETTE.trait} />
</svg>

<style>
  .bar,
  .note {
    cursor: pointer;
  }

  .note:focus-visible text {
    stroke: var(--accent);
  }

  .slot {
    outline: none;
  }
  .bar {
    cursor: grab;
  }
  .bar.scope {
    cursor: grabbing;
  }
  .slot {
    cursor: pointer;
  }
  .bar:focus-visible rect:last-of-type,
  .slot:focus-visible rect {
    stroke: #1f3a2e;
    stroke-width: 3;
  }
  .bar:hover:not(.scope) {
    filter: brightness(0.97);
  }
</style>
