<script lang="ts">
  // Right panel of the editor for the selected bed (D01 E03, D02 E06):
  // name, type, dimensions, plantings of the year, history of the past years,
  // deletion with the warning for the plantings that straddle the next year.
  import { commands, type BedView, type TypeBed, type YearView } from "../bindings";
  import Confirmation from "../shell/Confirmation.svelte";
  import Icon from "../shell/Icon.svelte";
  import { app } from "../state.svelte";
  import { get } from "svelte/store";
  import { m } from "../paraglide/messages.js";
  import { cursor } from "../time";
  import { dateShort, formatM2, formatMeters, nameCrop, nameOfType } from "../beds";
  import type { Geo, Response } from "./drag";

  let {
    view,
    bed,
    onclose,
    onmodify,
  }: {
    view: YearView;
    bed: BedView;
    onclose: () => void;
    onmodify: (id: string, geo: Geo, nature: "move" | "resize") => Promise<Response>;
  } = $props();

  const TYPES: TypeBed[] = ["bed", "square", "mound", "planter"];
  const MODES = { indoor: m.mode_indoor, plants: m.mode_plants, open_ground: m.mode_open_ground };

  let name = $state("");
  let length = $state("");
  let width = $state("");
  let historyOpen = $state(false);
  let suppression = $state(false);

  // The fields follow the bed of the view (selection, drag, Rust's answer).
  $effect(() => {
    name = bed.name;
    length = String(bed.length_cm / 100);
    width = String(bed.width_cm / 100);
  });

  const plantings = $derived(view.plantings.filter((s) => s.bed_id === bed.id));
  const history = $derived(view.history.filter((s) => s.bed_id === bed.id));
  const notes = $derived(
    view.annotations.filter((a) => a.bed_id === bed.id).sort((a, b) => a.date_start.localeCompare(b.date_start)),
  );
  const deleted = $derived(view.plantings.filter((s) => bed.plantings_deleted.includes(s.id)));
  const geo = $derived<Geo>({
    x: bed.x,
    y: bed.y,
    length_cm: bed.length_cm,
    width_cm: bed.width_cm,
  });

  async function rename() {
    if (name === bed.name) return;
    const ok = await app.modify((year) => commands.modifyBed(year, bed.id, name, null));
    if (!ok) name = bed.name;
  }

  async function changerType(type: TypeBed) {
    if (type === bed.bed_type) return;
    await app.modify((year) =>
      commands.modifyBed(year, bed.id, null, { bed_type: type, ...geo }),
    );
  }

  /** Dimensions typed in meters; Rust refuses a length that is not a whole
   *  number of cases, and the field goes back to the real value. */
  async function resize() {
    // A number input bound with `bind:value` holds a number (or null when empty), not a string.
    const meters = (typed: unknown) => Math.round(Number(String(typed ?? "").replace(",", ".")) * 100);
    const l = meters(length);
    const h = meters(width);
    if (!Number.isFinite(l) || !Number.isFinite(h) || l <= 0 || h <= 0) {
      length = String(bed.length_cm / 100);
      width = String(bed.width_cm / 100);
      return;
    }
    const response = await onmodify(bed.id, { ...geo, length_cm: l, width_cm: h }, "resize");
    if (response !== "ok") {
      length = String(bed.length_cm / 100);
      width = String(bed.width_cm / 100);
    }
  }

  async function remove() {
    suppression = false;
    const ok = await app.modify((year) => commands.deleteBed(year, bed.id));
    if (ok) onclose();
  }
</script>

<aside class="panel" aria-label={m.panel_selection()}>
  <div class="header">
    <div class="title">
      <p class="kicker">{m.panel_selection()}</p>
      <h2>{bed.name}</h2>
    </div>
    <button class="rond" aria-label={m.panel_close()} onclick={onclose}><Icon name="close" size={16} /></button>
  </div>

  <div class="puces">
    <span class="puce">
      {m.panel_dimensions({ length: formatMeters(bed.length_cm), width: formatMeters(bed.width_cm) })}
    </span>
    <span class="puce">{m.panel_surface({ value: formatM2(bed.length_cm * bed.width_cm) })}</span>
  </div>

  {#if view.read_only}
    <p class="help">{m.panel_read_only()}</p>
  {:else}
    <label class="field">
      <span>{m.field_name()}</span>
      <input type="text" maxlength="30" bind:value={name} onchange={rename} />
    </label>

    <fieldset class="field">
      <legend>{m.field_type()}</legend>
      <div class="choice">
        {#each TYPES as type (type)}
          <label class:active={bed.bed_type === type}>
            <input
              type="radio"
              name="type-bed"
              checked={bed.bed_type === type}
              onchange={() => changerType(type)}
            />
            {nameOfType(type)}
          </label>
        {/each}
      </div>
    </fieldset>

    <div class="dimensions">
      <label class="field">
        <span>{m.field_length()}</span>
        <input
          type="number"
          inputmode="decimal"
          min={view.plot ? view.plot.cell_cm / 100 : 0.25}
          step={view.plot ? view.plot.cell_cm / 100 : 0.25}
          bind:value={length}
          onchange={resize}
        />
      </label>
      <label class="field">
        <span>{m.field_width()}</span>
        <input type="number" inputmode="decimal" min="0.1" step="0.01" bind:value={width} onchange={resize} />
      </label>
    </div>
    {#if view.plot}
      <p class="help">{m.field_length_help({ case: view.plot.cell_cm })}</p>
    {/if}
  {/if}

  <h3 class="section">{m.panel_plantings({ year: view.year })}</h3>
  {#each plantings as planting (planting.id)}
    <div class="planting">
      <strong>{nameCrop(planting.crop_id)}</strong>
      <span class="badge">{MODES[planting.mode]()}</span>
      <p>
        {m.panel_planting_segments({
          start: planting.segment_start,
          end: planting.segment_end,
          plants: planting.rows * planting.plants_by_row,
          rows: planting.rows,
          by_row: planting.plants_by_row,
        })}
      </p>
      <p>{m.panel_planting_dates({ start: dateShort(planting.date, view.year), end: dateShort(planting.end, view.year) })}</p>
      <button class="link" onclick={() => (app.plantingOpen = { type: "edition", planting })}>{m.planting_modify()}</button>
    </div>
  {:else}
    <p class="help">{m.panel_none_planting()}</p>
  {/each}
  <button class="button principal" onclick={() => (app.plantingOpen = { type: "new", bedId: bed.id })}>
    <Icon name="plus" size={16} />{m.planting_plant_here()}
  </button>

  <h3 class="section">{m.note_bed_title()}</h3>
  {#each notes as note (note.id)}
    <div class="planting">
      <strong>{note.text}</strong>
      {#if note.hidden}<span class="badge">{m.note_bed_hidden()}</span>{/if}
      <p>
        {note.date_end
          ? m.note_aria_period({ start: dateShort(note.date_start, view.year), end: dateShort(note.date_end, view.year) })
          : dateShort(note.date_start, view.year)}
      </p>
      <button class="link" onclick={() => (app.noteOpen = { type: "edition", annotation: note })}>{m.planting_modify()}</button>
    </div>
  {:else}
    <p class="help">{m.note_bed_none()}</p>
  {/each}
  <button class="button" onclick={() => (app.noteOpen = { type: "new", date: get(cursor), bedId: bed.id })}>
    <Icon name="plus" size={16} />{m.note_bed_add()}
  </button>

  <button class="section fallback" aria-expanded={historyOpen} onclick={() => (historyOpen = !historyOpen)}>
    {m.panel_history()}
  </button>
  {#if historyOpen}
    {#each history as planting (planting.id)}
      <div class="planting">
        <strong>{nameCrop(planting.crop_id)}</strong>
        <p>
          {m.panel_planting_dates({ start: dateShort(planting.date), end: dateShort(planting.end) })} ·
          ≈ {planting.rows * planting.plants_by_row}
        </p>
      </div>
    {:else}
      <p class="help">{m.panel_history_empty()}</p>
    {/each}
  {/if}

  {#if !view.read_only}
    <p class="help">{m.panel_follow()}</p>
    <button class="button danger" onclick={() => (suppression = true)}>
      <Icon name="trash" size={16} />{m.delete()}
    </button>
  {/if}
</aside>

{#if suppression}
  <Confirmation
    title={m.delete_bed_title({ name: bed.name })}
    confirm={m.delete_bed_button()}
    onconfirm={remove}
    oncancel={() => (suppression = false)}
  >
    <p>{m.delete_bed_text({ year: view.year })}</p>
    {#if bed.plantings_deleted.length > 0}
      <div class="warning">
        <p>{m.delete_bed_plantings({ count: bed.plantings_deleted.length, year: view.year })}</p>
        <ul>
          {#each deleted as planting (planting.id)}
            <li>
              {m.delete_bed_planting({
                crop: nameCrop(planting.crop_id),
                start: dateShort(planting.date),
                end: dateShort(planting.end),
              })}
            </li>
          {/each}
        </ul>
        <p>{m.delete_bed_kept({ year: view.year })}</p>
      </div>
    {/if}
  </Confirmation>
{/if}

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 20px;
    border-radius: 28px;
    background: var(--white);
  }

  .header {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }

  .kicker {
    margin: 0 0 4px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }

  h2 {
    font-size: 24px;
    overflow-wrap: anywhere;
  }

  .rond {
    flex: none;
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border: 0;
    border-radius: 50%;
    background: var(--bg);
  }

  .puces {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .puce {
    padding: 4px 12px;
    border-radius: var(--pill);
    background: var(--bg);
    font-size: 13px;
    font-weight: 700;
  }

  .field {
    display: block;
    border: 0;
    margin: 0;
    padding: 0;
  }

  .field > span,
  legend {
    display: block;
    margin-bottom: 6px;
    font-size: 12px;
    font-weight: 700;
    color: var(--muted);
  }

  input[type="text"],
  input[type="number"] {
    width: 100%;
    height: 44px;
    padding: 0 14px;
    border: 0;
    border-radius: 14px;
    background: var(--bg);
    font-weight: 600;
  }

  .choice {
    display: flex;
    gap: 2px;
    padding: 4px;
    border-radius: var(--pill);
    background: var(--bg);
  }

  .choice label {
    flex: 1;
    padding: 8px 0;
    text-align: center;
    border-radius: var(--pill);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .choice label.active {
    background: var(--ink);
    color: var(--white);
  }

  .choice input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .choice label:has(input:focus-visible) {
    outline: 3px solid var(--accent);
  }

  .dimensions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .help {
    margin: 0;
    font-size: 13px;
    color: var(--muted);
  }

  .section {
    margin: 8px 0 0;
    font-family: var(--font);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .fallback {
    padding: 0;
    border: 0;
    background: none;
    text-align: left;
  }

  .fallback::after {
    content: " ▾";
  }

  .fallback[aria-expanded="true"]::after {
    content: " ▴";
  }

  .planting {
    padding: 12px 14px;
    border-radius: 16px;
    background: var(--bg);
    font-size: 13px;
  }

  .link {
    padding: 0;
    border: 0;
    background: none;
    color: var(--green);
    font-size: 13px;
    font-weight: 600;
    text-decoration: underline;
  }

  .planting p {
    margin: 2px 0 0;
    color: var(--muted);
  }

  .badge {
    margin-left: 6px;
    padding: 2px 8px;
    border-radius: var(--pill);
    background: var(--warn-bg);
    font-size: 11px;
    font-weight: 700;
    color: var(--accent-ink);
  }

  .warning {
    padding: 12px 14px;
    border-radius: 14px;
    background: var(--warn-bg);
  }

  .warning p {
    margin: 0 0 6px;
  }

  .warning ul {
    margin: 0 0 6px;
    padding-left: 18px;
  }
</style>
