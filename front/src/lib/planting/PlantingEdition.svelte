<script lang="ts">
  // E07 · New planting and edition (D02 E07_planting_desktop, D01 E07_planting_mobile;
  // S01 F-PLA-1, F-PLA-2). Three questions (crop, how it starts, when), the
  // zone of the bed, and on the side what Rust calculates as the user
  // types: the steps with their dates, the quantity, the conflicts.
  //
  // Nothing is calculated here. Each change calls `preview_planting`; the form
  // shows its answer and saves with `create_planting` or `modify_planting`.
  import { untrack } from "svelte";
  import { commands, type Mode, type ProblemView, type PlantingPreview } from "../bindings";
  import Confirmation from "../shell/Confirmation.svelte";
  import Icon from "../shell/Icon.svelte";
  import { modal } from "../shell/modal";
  import Logo from "../shell/Logo.svelte";
  import { dateAbbreviated, dateComplete } from "../dates";
  import { translateError } from "../errors";
  import { call, app, type OpeningPlanting } from "../state.svelte";
  import IconTimeline from "../timeline/IconTimeline.svelte";
  import { m } from "../paraglide/messages.js";
  import { formatMeters, nameCrop } from "../beds";
  import { rankSuggestions } from "../suggestions";
  import { localeTag } from "../locale";
  import { cursor } from "../time";
  import { get } from "svelte/store";
  import { isOk } from "./call";
  import { catalogue } from "./crops.svelte";
  import MiniTimeline from "./MiniTimeline.svelte";
  import PlantIcon from "./PlantIcon.svelte";
  import {
    SHORTCUTS,
    applySuggestion,
    toggleTransplanting,
    changeDate,
    resetManualEnd,
    endInput,
    formSincePlanting,
    formEmpty,
    sameZone,
    modeInitial,
    firstShortcutFree,
    segmentsMiniTimeline,
    toPlantingInput,
    versZone,
    zoneAfterClick,
    zoneFromShortcut,
    type Form,
    type Zone,
  } from "./planting";

  let { opening }: { opening: OpeningPlanting } = $props();

  const dateValid = (date: string) => /^\d{4}-\d{2}-\d{2}$/.test(date);

  const edition = $derived(opening.type === "edition" ? opening.planting : null);
  const view = $derived(app.view!);

  function segments(bedId: string): number {
    return view.beds.find((p) => p.id === bedId)?.segments ?? 1;
  }

  function initial(): Form {
    if (opening.type === "edition") return formSincePlanting(opening.planting);
    const { bedId, slot, suggestion } = opening;
    const base = formEmpty(
      catalogue.crop(suggestion?.crop_id ?? "") ?? catalogue.crops[0],
      bedId,
      segments(bedId),
      get(cursor),
    );
    if (!slot) return base;
    const zone = { start: slot.segment_start, end: slot.segment_end };
    // A slot opens on its first day, or on the day the suggestion proposes.
    return suggestion
      ? applySuggestion({ ...base, date: slot.start }, suggestion, zone)
      : { ...base, date: slot.start, zone };
  }

  let f = $state<Form>(untrack(initial));
  let preview = $state<PlantingPreview | null>(null);
  let season = $state<string[]>([]);
  let suppression = $state(false);
  let sending = $state(false);
  /** The zone follows the first free shortcut until the user touches it. */
  let zoneAuto = $state(untrack(() => opening.type === "new" && !opening.slot));
  let cropChosen = $state(untrack(() => opening.type === "edition" || !!opening.suggestion));

  const bed = $derived(view.beds.find((p) => p.id === f.bedId));
  const countSegments = $derived(bed?.segments ?? Math.max(f.zone.end, 1));
  const crop = $derived(catalogue.crop(f.cropId));
  const dates = $derived(preview?.dates ?? null);
  const endComputed = $derived(dates?.default_end ?? "");
  const slot = $derived(opening.type === "new" ? opening.slot : undefined);
  const withoutPots = $derived(!!crop && crop.indoor_days === 0 && f.mode === "indoor");

  // ---------- What Rust says about the form ----------

  let tokenPreview = 0;
  $effect(() => {
    const entree = toPlantingInput(f);
    const id = edition?.id ?? null;
    if (!entree.crop_id) return;
    const mon = ++tokenPreview;
    const timer = setTimeout(async () => {
      const r = await call(() => commands.previewPlanting(id, entree));
      if (mon === tokenPreview && r.status === "ok") preview = r.data;
    }, 80);
    return () => clearTimeout(timer);
  });

  // From « Plant here »: if the whole bed is taken during that time, start
  // from the first free shortcut Rust offers.
  $effect(() => {
    if (!preview || !zoneAuto) return;
    untrack(() => {
      zoneAuto = false;
      if (preview!.problems.length === 0) return;
      const free = firstShortcutFree(preview!.shortcuts);
      if (free && !sameZone(free, f.zone)) f.zone = free;
    });
  });

  // The crops in season on the chosen date, highlighted in the list (F-PLA-1).
  $effect(() => {
    const date = f.date;
    if (!dateValid(date)) return;
    void call(() => isOk(commands.cropsOfSeason(date))).then((r) => {
      if (r.status === "ok") season = r.data;
    });
  });

  // A new planting from « Plant here » starts on the first crop in season.
  $effect(() => {
    if (cropChosen || season.length === 0 || opening.type !== "new") return;
    untrack(() => {
      const premiere = order.find((id) => season.includes(id));
      if (premiere) chooseCrop(premiere, false);
    });
  });

  // ---------- Crops: the suggestions first, then those in season ----------

  const slotSuggestions = $derived(slot ? rankSuggestions(slot.suggestions) : []);
  const suggested = $derived(slotSuggestions.map((s) => s.crop_id));
  const order = $derived(
    catalogue.crops
      .map((c) => c.id)
      .sort((a, b) => {
        const row = (id: string) => (suggested.includes(id) ? 0 : season.includes(id) ? 1 : 2);
        return row(a) - row(b) || nameCrop(a).localeCompare(nameCrop(b), localeTag());
      }),
  );

  function chooseCrop(id: string, user = true) {
    if (user) cropChosen = true;
    if (id === f.cropId) return;
    const c = catalogue.crop(id);
    // The card checked at first follows the crop; once the user has picked a mode it stays.
    f = resetManualEnd({ ...f, cropId: id, mode: edition || user && modeChosen ? f.mode : modeInitial(c) });
  }

  let modeChosen = $state(false);
  function chooseMode(mode: Mode) {
    modeChosen = true;
    f = resetManualEnd({ ...f, mode, transplanting: mode === "indoor" ? f.transplanting : false });
  }

  // ---------- Zone ----------

  function chooseZone(z: Zone) {
    zoneAuto = false;
    f.zone = z;
  }

  function chooseBed(id: string) {
    zoneAuto = true;
    f = { ...f, bedId: id, zone: { start: 1, end: segments(id) } };
  }

  function occupied(i: number): boolean {
    return !!preview?.occupied.some((z) => z.segment_start <= i && i <= z.segment_end);
  }

  // ---------- Dates ----------

  const question = $derived(
    f.mode === "plants" ? m.planting_q_plant() : f.mode === "indoor" && f.transplanting ? m.planting_q_transplant() : m.planting_q_sow(),
  );

  const steps = $derived.by(() => {
    if (!dates) return [];
    const list: { date: string; text: string; fort?: boolean }[] = [];
    if (f.mode === "indoor" && dates.sowing) list.push({ date: dates.sowing, text: m.planting_step_sowing_indoor() });
    list.push({
      date: dates.planting_out,
      text: f.mode === "plants" ? m.planting_step_planting() : f.mode === "indoor" ? m.planting_step_transplanting() : m.planting_step_sowing_ground(),
    });
    list.push({ date: dates.harvest_start, text: m.planting_step_harvest() });
    list.push({ date: dates.end, text: dates.manual_end ? m.planting_step_manual_end() : m.planting_step_end(), fort: dates.manual_end });
    return list;
  });

  function enterEnd(value: string) {
    f.manualEnd = endInput(value, endComputed);
  }

  // ---------- Problems and saving ----------

  function nameBed(): string {
    return bed?.name ?? "";
  }

  function textProblem(p: ProblemView): string {
    if (p.code === "conflict" && p.zone && p.start && p.end) {
      const other = [...view.plantings, ...view.history].find((s) => s.id === p.with);
      return m.planting_problem_conflict({
        crop: other ? nameCrop(other.crop_id) : m.planting_problem_other(),
        start: p.zone.segment_start,
        end: p.zone.segment_end,
        from: dateAbbreviated(p.start, view.year),
        to: dateAbbreviated(p.end, view.year),
      });
    }
    if (p.code === "zone_outside_bed" && p.segments !== null && p.year !== null) {
      return m.planting_problem_zone({ segments: p.segments, year: p.year });
    }
    if (p.code === "bed_absent" && p.year !== null) return m.planting_problem_absent({ year: p.year });
    return translateError({ code: p.code, detail: "" });
  }

  const valid = $derived(!!dates && (preview?.problems.length ?? 1) === 0);
  const plants = $derived(dates ? dates.rows * dates.plants_by_row : 0);

  function close() {
    app.plantingOpen = null;
  }

  async function validate() {
    if (!valid || sending) return;
    sending = true;
    const entree = toPlantingInput(f);
    const ok = await app.modify((year) =>
      edition ? commands.modifyPlanting(year, edition.id, entree) : commands.createPlanting(year, entree),
    );
    sending = false;
    if (ok) {
      app.informer(edition ? m.planting_modified() : m.planting_created());
      close();
    }
  }

  async function remove() {
    suppression = false;
    if (!edition) return;
    const id = edition.id;
    if (await app.modify((year) => commands.deletePlanting(year, id))) {
      app.informer(m.planting_deleted());
      close();
    }
  }

  const MODES: { mode: Mode; title: () => string; help: () => string }[] = [
    { mode: "indoor", title: m.planting_mode_indoor, help: m.planting_mode_indoor_help },
    { mode: "plants", title: m.planting_mode_plants, help: m.planting_mode_plants_help },
    { mode: "open_ground", title: m.planting_mode_ground, help: m.planting_mode_ground_help },
  ];
  const LABELS_SHORTCUTS = {
    all: m.planting_shortcut_all,
    half: m.planting_shortcut_half,
    third: m.planting_shortcut_third,
    quarter: m.planting_shortcut_quarter,
  };
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape" && !suppression) close();
  }}
/>

<div class="screen" role="dialog" aria-modal="true" aria-labelledby="planting-title" tabindex="-1" use:modal>
  <header class="header">
    <Logo />
    <div class="titles">
      <h1 id="planting-title">{edition ? m.planting_title_edition() : m.planting_title_new()}</h1>
      <p>{nameBed()}</p>
    </div>
    <button class="rond" aria-label={m.close()} title={m.close()} onclick={close}><Icon name="close" size={18} /></button>
  </header>

  <div class="columns">
    <section class="questions">
      {#if slot}
        <div class="slot">
          <p class="slot-text">
            {m.planting_slot({
              start: dateAbbreviated(slot.start, view.year),
              end: dateAbbreviated(slot.end, view.year),
            })}
          </p>
          {#if slotSuggestions.length > 0}
            <p class="etiquette">{m.planting_suggestions()}</p>
            <div class="puces">
              {#each slotSuggestions as s (s.crop_id)}
                <button
                  class="puce"
                  class:active={f.cropId === s.crop_id && f.date === s.date}
                  onclick={() => {
                    cropChosen = true;
                    modeChosen = true;
                    zoneAuto = false;
                    f = applySuggestion(f, s, { start: slot.segment_start, end: slot.segment_end });
                  }}
                >
                  {nameCrop(s.crop_id)} · {dateAbbreviated(s.date, view.year)}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <h2 class="question"><span class="number">1</span>{m.planting_q_crop()}</h2>
      <div class="legumes" role="radiogroup" aria-label={m.planting_q_crop()}>
        {#each order as id (id)}
          <button
            class="crop"
            class:active={f.cropId === id}
            role="radio"
            aria-checked={f.cropId === id}
            onclick={() => chooseCrop(id)}
          >
            <PlantIcon crop={id} stade={2} width={52} />
            <span>{nameCrop(id)}</span>
            {#if suggested.includes(id)}
              <em class="badge">{m.planting_suggested()}</em>
            {:else if season.includes(id)}
              <em class="badge season">{m.planting_in_season()}</em>
            {/if}
          </button>
        {/each}
      </div>

      <h2 class="question"><span class="number">2</span>{m.planting_q_how()}</h2>
      <div class="modes" role="radiogroup" aria-label={m.planting_q_how()}>
        {#each MODES as option (option.mode)}
          <label class="mode" class:active={f.mode === option.mode}>
            <input type="radio" name="mode" checked={f.mode === option.mode} onchange={() => chooseMode(option.mode)} />
            <span class="mode-text">
              <strong>{option.title()}</strong>
              <span>{option.help()}</span>
            </span>
            <span class="radio" aria-hidden="true"></span>
          </label>
        {/each}
      </div>

      {#if withoutPots}
        <label class="field">
          <span>{m.planting_pots_question()}</span>
          <input type="number" min="1" step="1" inputmode="numeric" bind:value={f.indoorDays} onchange={() => (f = resetManualEnd(f))} />
          <small>{m.planting_pots_help()}</small>
        </label>
      {/if}

      <h2 class="question"><span class="number">3</span>{question}</h2>
      <label class="date">
        <input type="date" value={f.date} onchange={(e) => (f = changeDate(f, e.currentTarget.value))} aria-label={question} />
        <span class="day">{dateValid(f.date) ? dateComplete(f.date) : ""}</span>
      </label>
      {#if f.mode === "indoor"}
        <button class="link" onclick={() => (f = toggleTransplanting(f, preview))}>
          {f.transplanting ? m.planting_link_sowing() : m.planting_link_transplanting()}
        </button>
      {/if}
    </section>

    <aside class="summary" aria-live="polite">
      <div class="carte">
        {#if dates}
          <MiniTimeline segments={segmentsMiniTimeline(dates, f.mode)} cropId={f.cropId} />
          <ol class="steps">
            {#each steps as step (step.text)}
              <li class:fort={step.fort}>
                <strong>{dateAbbreviated(step.date, view.year)}</strong>
                <span>{step.text}</span>
              </li>
            {/each}
          </ol>
          <div class="end">
            <label>
              <span>{m.planting_end_label()}</span>
              <input type="date" value={f.manualEnd ?? dates.end} onchange={(e) => enterEnd(e.currentTarget.value)} />
            </label>
            {#if dates.manual_end}
              <p class="manual">
                <IconTimeline name="crayon" size={14} />
                {m.planting_manual_end({ date: dateAbbreviated(dates.end, view.year) })}
              </p>
              <button class="link" onclick={() => (f.manualEnd = null)}>
                {m.planting_recompute({ date: dateAbbreviated(dates.default_end, view.year) })}
              </button>
            {/if}
          </div>
        {:else}
          <p class="waiting">{m.planting_calculation()}</p>
        {/if}
      </div>

      <div class="zone">
        <div class="zone-tete">
          <label class="bed">
            <span class="etiquette">{m.planting_zone_title()}</span>
            <select value={f.bedId} onchange={(e) => chooseBed(e.currentTarget.value)} aria-label={m.planting_bed()}>
              {#each view.beds as p (p.id)}
                <option value={p.id}>{p.name}</option>
              {/each}
            </select>
          </label>
          {#if bed}
            <span class="etiquette">{m.planting_zone_dimensions({ length: formatMeters(bed.length_cm), width: formatMeters(bed.width_cm) })}</span>
          {/if}
        </div>
        <div class="shortcuts" role="group" aria-label={m.planting_zone_title()}>
          {#each SHORTCUTS as key (key)}
            {@const z = preview ? zoneFromShortcut(preview.shortcuts, key) : null}
            <button
              class="shortcut"
              class:active={sameZone(z, f.zone)}
              disabled={!z}
              onclick={() => z && chooseZone(z)}>{LABELS_SHORTCUTS[key]()}</button
            >
          {/each}
        </div>
        <div class="bande" role="group" aria-label={m.planting_zone_title()}>
          {#each Array.from({ length: countSegments }, (_, i) => i + 1) as i (i)}
            <button
              class="segment"
              class:chosen={f.zone.start <= i && i <= f.zone.end}
              class:taken={occupied(i)}
              aria-pressed={f.zone.start <= i && i <= f.zone.end}
              aria-label={m.planting_segment_aria({ number: i })}
              onclick={() => chooseZone(zoneAfterClick(f.zone, i))}
            ></button>
          {/each}
        </div>
        {#if dates}
          <p class="quantity">
            <span>{m.planting_quantity({ plants, rows: dates.rows, by_row: dates.plants_by_row })}</span>
          </p>
        {/if}
      </div>

      {#if preview && preview.problems.length > 0}
        <ul class="problems" role="alert">
          {#each preview.problems as p (p.code + (p.with ?? ""))}
            <li><Icon name="alert" size={16} />{textProblem(p)}</li>
          {/each}
        </ul>
      {/if}

      <button class="button principal validate" disabled={!valid || sending} onclick={validate}>
        <IconTimeline name="check" />
        {edition ? m.planting_save() : m.planting_plant_button({ crop: nameCrop(f.cropId), plants })}
      </button>
      {#if edition}
        <button class="button danger" onclick={() => (suppression = true)}>
          <Icon name="trash" size={16} />{m.planting_delete()}
        </button>
      {/if}
    </aside>
  </div>
</div>

{#if suppression && edition}
  <Confirmation
    title={m.planting_delete_title()}
    confirm={m.planting_delete()}
    onconfirm={remove}
    oncancel={() => (suppression = false)}
  >
    <p>
      {m.planting_delete_text({
        crop: nameCrop(edition.crop_id),
        bed: nameBed(),
        start: dateAbbreviated(edition.date, view.year),
        end: dateAbbreviated(edition.end, view.year),
      })}
    </p>
  </Confirmation>
{/if}

<style>
  .screen {
    position: fixed;
    inset: 0;
    z-index: 15;
    overflow-y: auto;
    padding: 20px 32px 40px;
    background: var(--bg);
    outline: none;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 14px;
    max-width: 1100px;
    margin: 0 auto 20px;
  }

  .titles {
    flex: 1;
  }

  .titles h1 {
    font-size: 22px;
  }

  .titles p {
    margin: 0;
    color: var(--muted);
    font-size: 14px;
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

  .columns {
    display: grid;
    grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
    gap: 24px;
    align-items: start;
    max-width: 1100px;
    margin: 0 auto;
  }

  .questions {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 24px;
    border-radius: 32px;
    background: var(--white);
  }

  .summary {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 20px;
    border-radius: 32px;
    background: var(--card);
    position: sticky;
    top: 16px;
  }

  .question {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 6px;
    font-size: 18px;
  }

  .number {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--ink);
    color: var(--white);
    font-family: var(--font);
    font-size: 14px;
    font-weight: 700;
  }

  .slot {
    padding: 12px 14px;
    border-radius: 16px;
    background: var(--warn-bg);
  }

  .slot-text {
    margin: 0 0 8px;
    font-weight: 600;
    color: var(--accent-ink);
  }

  .etiquette {
    margin: 0;
    color: var(--muted);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
  }

  .puces {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 6px;
  }

  .puce {
    min-height: 36px;
    padding: 0 14px;
    border: 1px solid var(--accent);
    border-radius: var(--pill);
    background: var(--white);
    font-size: 14px;
    font-weight: 600;
  }

  .puce.active {
    background: var(--accent);
    color: var(--accent-ink);
  }

  .legumes {
    display: grid;
    grid-template-columns: repeat(6, minmax(0, 1fr));
    gap: 8px;
  }

  .crop {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 10px 4px 8px;
    border: 2px solid transparent;
    border-radius: 16px;
    background: var(--card);
    font-size: 13px;
    font-weight: 600;
  }

  .crop.active {
    border-color: var(--accent);
    background: var(--warn-bg);
  }

  .badge {
    padding: 1px 8px;
    border-radius: var(--pill);
    background: var(--accent);
    color: var(--accent-ink);
    font-size: 10px;
    font-style: normal;
    font-weight: 700;
  }

  .badge.season {
    background: var(--green);
    color: var(--white);
  }

  .modes {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .mode {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    border: 2px solid var(--line);
    border-radius: 18px;
    cursor: pointer;
  }

  .mode input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .mode:has(input:focus-visible) {
    outline: 3px solid var(--accent);
    outline-offset: 2px;
  }

  .mode.active {
    border-color: var(--accent);
    background: var(--warn-bg);
  }

  .mode-text {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 2px;
  }

  .mode-text span {
    color: var(--muted);
    font-size: 13px;
  }

  .radio {
    width: 22px;
    height: 22px;
    border: 2px solid var(--line);
    border-radius: 50%;
  }

  .mode.active .radio {
    border-color: var(--accent);
    background: radial-gradient(circle, var(--accent) 45%, transparent 50%);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-weight: 600;
    font-size: 14px;
  }

  .field small {
    color: var(--muted);
    font-weight: 400;
  }

  input[type="number"],
  input[type="date"],
  select {
    min-height: 44px;
    padding: 0 14px;
    border: 1px solid var(--line);
    border-radius: 14px;
    background: var(--card);
  }

  .date {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
    padding: 4px 14px;
    border-radius: 16px;
    background: var(--card);
  }

  .date input {
    min-height: 44px;
    padding: 0;
    border: 0;
    background: transparent;
    font-weight: 600;
  }

  .day {
    display: inline-block;
    color: var(--muted);
    font-size: 14px;
  }

  .day::first-letter {
    text-transform: uppercase;
  }

  .link {
    align-self: flex-start;
    padding: 0;
    border: 0;
    background: none;
    color: var(--green);
    font-size: 14px;
    font-weight: 600;
    text-decoration: underline;
  }

  .carte {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border-radius: 22px;
    background: var(--white);
  }

  .steps {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .steps li {
    display: flex;
    flex-direction: column;
    gap: 1px;
    font-size: 12px;
    color: var(--muted);
  }

  .steps strong {
    color: var(--ink);
    font-size: 14px;
  }

  .steps li.fort span {
    color: var(--accent-ink);
    font-weight: 700;
  }

  .end {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 12px;
    padding-top: 10px;
    border-top: 1px solid var(--line);
  }

  .end label {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    font-weight: 600;
  }

  .end input {
    min-height: 36px;
  }

  .manual {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin: 0;
    padding: 4px 10px;
    border-radius: var(--pill);
    background: var(--warn-bg);
    color: var(--accent-ink);
    font-size: 13px;
    font-weight: 700;
  }

  .waiting {
    margin: 0;
    color: var(--muted);
  }

  .zone {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .zone-tete {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .bed {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .bed select {
    min-height: 36px;
    background: var(--white);
  }

  .shortcuts {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 4px;
    padding: 4px;
    border-radius: var(--pill);
    background: var(--white);
  }

  .shortcut {
    min-height: 40px;
    border: 0;
    border-radius: var(--pill);
    background: transparent;
    font-weight: 600;
  }

  .shortcut.active {
    background: var(--ink);
    color: var(--white);
  }

  .bande {
    display: flex;
    gap: 4px;
  }

  .segment {
    flex: 1;
    height: 38px;
    min-width: 0;
    border: 2px solid var(--line);
    border-radius: 8px;
    background: var(--white);
  }

  .segment.taken {
    border-color: var(--bed-furrow);
    background: repeating-linear-gradient(135deg, var(--bed) 0 5px, var(--bed-furrow) 5px 8px);
  }

  .segment.chosen {
    border-color: var(--green);
    background: #b8dbc6;
  }

  .segment.taken.chosen {
    border-color: var(--crit);
  }

  .quantity {
    display: flex;
    justify-content: space-between;
    margin: 0;
    padding-top: 10px;
    border-top: 1px solid var(--line);
    font-size: 14px;
  }

  .problems {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: 0;
    padding: 12px 14px;
    border-radius: 16px;
    background: var(--crit-bg);
    color: #5a1e1a;
    font-size: 14px;
    list-style: none;
  }

  .problems li {
    display: flex;
    align-items: flex-start;
    gap: 8px;
  }

  .validate {
    min-height: 52px;
    font-size: 17px;
  }

  .danger {
    color: var(--crit);
    border-color: var(--crit);
  }

  @media (max-width: 900px) {
    .screen {
      padding: 12px 12px 32px;
    }
    .columns {
      grid-template-columns: minmax(0, 1fr);
    }
    .summary {
      position: static;
    }
    .questions {
      padding: 16px;
      border-radius: 24px;
    }
    .legumes {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
  }
</style>
