<script lang="ts">
  import { localeTag } from "../locale";
  // Export of the timeline (E11, D02): format, period, beds and content on
  // the left, the preview on the right. The drawing is `TimelineSvg`, mounted off
  // screen on the chosen period and serialised (`serializeTimeline`); « Save
  // under » asks for the path, then Rust converts it (`export_timeline`).
  import { save } from "@tauri-apps/plugin-dialog";
  import { get } from "svelte/store";
  import { tick } from "svelte";
  import { commands, type AppError, type TimelineExportFormat, type YearView } from "../bindings";
  import Icon from "../shell/Icon.svelte";
  import Logo from "../shell/Logo.svelte";
  import { todayLocal } from "../dates";
  import { call, app } from "../state.svelte";
  import { m } from "../paraglide/messages.js";
  import { cursor, viewportTimeline } from "../time";
  import { serializeTimeline } from "./exportSvg";
  import { yearsOfPeriod, scaleForPeriod, periodByDefault, periodVisible, problemPeriod } from "./exportPlan";
  import TimelineSvg, { WIDTH_LABELS } from "./TimelineSvg.svelte";
  import { buildRows } from "./geometry";

  // The period shown on screen, zoom and scroll included; the cursor's year
  // only if the timeline was never drawn.
  const viewport = get(viewportTimeline);
  const initial = viewport ? periodVisible(viewport) : periodByDefault(get(cursor));
  let format = $state<TimelineExportFormat>("pdf");
  let from = $state(initial.from);
  let to = $state(initial.to);
  let chosen = $state<string[]>((app.view?.beds ?? []).map((p) => p.id));
  let phases = $state(true);
  let marker = $state(true);
  let annotations = $state(true);
  let enProgress = $state(false);
  /** Why the last export failed (ET01): shown in the panel, with « Retry ». */
  let failure = $state<AppError["code"] | null>(null);
  let root: HTMLElement;
  let drawing: HTMLElement | undefined = $state();
  let svgText = $state("");

  const beds = $derived(app.view?.beds ?? []);
  const problem = $derived(problemPeriod(from, to));
  const scale = $derived(problem ? null : scaleForPeriod(from, to));

  // The year views of the period (the current one is already loaded).
  let loaded = $state<Record<number, YearView>>({});
  $effect(() => {
    if (problem) return;
    for (const year of yearsOfPeriod(from, to)) {
      if (year === app.view?.year || loaded[year]) continue;
      void call(() => commands.viewYear(year)).then((r) => {
        if (r.status === "ok") loaded[year] = r.data;
      });
    }
  });
  const views = $derived(
    problem || !app.view
      ? []
      : yearsOfPeriod(from, to)
          .map((a) => (a === app.view!.year ? app.view! : loaded[a]))
          .filter((v): v is YearView => !!v),
  );
  const ready = $derived(!problem && views.length === yearsOfPeriod(from, to).length);
  const rows = $derived(
    ready ? buildRows(views, app.view!.year).filter((l) => chosen.includes(l.bed.id)) : [],
  );
  const notes = $derived(
    annotations
      ? [...new Map(views.flatMap((v) => v.annotations).map((a) => [a.id, a])).values()].filter(
          (a) => !a.hidden && (a.bed_id === null || chosen.includes(a.bed_id)),
        )
      : [],
  );
  const exportable = $derived(ready && rows.length > 0 && !enProgress);

  // Serialise after each redraw of the off-screen drawing; the preview shows that very text.
  $effect(() => {
    void [rows, scale, notes, phases, marker];
    void tick().then(() => {
      const svg = drawing?.querySelector<SVGSVGElement>("svg[data-timeline]");
      svgText = svg && rows.length > 0 ? serializeTimeline(svg) : "";
    });
  });
  const preview = $derived(svgText ? `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svgText)}` : "");

  $effect(() => {
    root.focus();
  });

  function close() {
    app.exportOpen = false;
  }

  function toggle(id: string) {
    chosen = chosen.includes(id) ? chosen.filter((c) => c !== id) : [...chosen, id];
  }

  const all = $derived(chosen.length === beds.length);

  async function exportNow() {
    if (!exportable || !svgText) return;
    failure = null;
    const name = m.export_name_file({ date: todayLocal(), format });
    const path = await save({
      defaultPath: name,
      filters: [{ name: format === "pdf" ? m.export_format_pdf() : m.export_format_png(), extensions: [format] }],
    });
    if (typeof path !== "string") return;
    enProgress = true;
    const result = await call(() => commands.exportTimeline(svgText, format, path));
    enProgress = false;
    if (result.status === "ok") {
      app.informer(m.export_ok());
      close();
    } else if (result.error.code === "file") {
      failure = "file";
    } else {
      app.report(result.error);
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && close()} />

<div class="screen" role="dialog" aria-modal="true" aria-labelledby="export-title" tabindex="-1" bind:this={root}>
  <header class="header">
    <Logo />
    <button class="rond" aria-label={m.close()} title={m.close()} onclick={close}><Icon name="close" size={18} /></button>
  </header>

  <div class="columns">
    <section class="options">
      <h1 id="export-title">{m.export_title()}</h1>

      <div class="formats" role="group" aria-label={m.export_format()}>
        {#each [{ f: "pdf", title: m.export_format_pdf(), help: m.export_format_pdf_help() }, { f: "png", title: m.export_format_png(), help: m.export_format_png_help() }] as choice (choice.f)}
          <button type="button" class="format" class:active={format === choice.f} aria-pressed={format === choice.f} onclick={() => (format = choice.f as TimelineExportFormat)}>
            <strong>{choice.title}</strong>
            <span>{choice.help}</span>
          </button>
        {/each}
      </div>

      <fieldset>
        <legend>{m.export_period()}</legend>
        <div class="dates">
          <label>{m.export_from()}<input type="date" bind:value={from} /></label>
          <label>{m.export_to()}<input type="date" bind:value={to} /></label>
        </div>
        <p class="help" class:error={problem}>
          {problem === "empty"
            ? m.export_period_empty()
            : problem === "reversed"
              ? m.export_period_reversed()
              : problem === "too_long"
                ? m.export_period_too_long()
                : m.export_period_help()}
        </p>
      </fieldset>

      <fieldset>
        <legend>
          {m.export_beds()}
          <button type="button" class="link" onclick={() => (chosen = all ? [] : beds.map((p) => p.id))}>
            {all ? m.export_none() : m.export_all()}
          </button>
        </legend>
        <div class="puces">
          {#each beds as p (p.id)}
            <label class="puce"><input type="checkbox" checked={chosen.includes(p.id)} onchange={() => toggle(p.id)} />{p.name}</label>
          {/each}
        </div>
      </fieldset>

      <fieldset>
        <legend>{m.export_content()}</legend>
        <label class="case"><input type="checkbox" bind:checked={phases} />{m.export_phases()}</label>
        <label class="case"><input type="checkbox" bind:checked={marker} />{m.export_marker()}</label>
        <label class="case"><input type="checkbox" bind:checked={annotations} />{m.note_show()}</label>
      </fieldset>

      {#if failure}
        <div class="carte-error" role="alert">
          <div class="title"><Icon name="alert" size={18} /><strong>{m.state_file_title()}</strong></div>
          <p>{m.state_file_text()}</p>
          <button class="button" onclick={exportNow}><Icon name="retry" size={16} />{m.retry()}</button>
        </div>
      {/if}

      <button class="button principal" disabled={!exportable || !svgText} onclick={exportNow}>
        <Icon name="export" size={16} />{format === "pdf" ? m.export_button_pdf() : m.export_button_png()}
      </button>
    </section>

    <section class="preview" aria-label={m.export_preview()}>
      <div class="preview-tete">
        <h2>{m.export_preview()}</h2>
        <span>{m.export_from_day({ date: new Date().toLocaleDateString(localeTag(), { day: "numeric", month: "short", year: "numeric" }) })}</span>
      </div>
      <div class="feuille">
        {#if preview}
          <img src={preview} alt={m.export_preview_alt()} />
        {:else}
          <p class="empty">{rows.length === 0 && ready ? m.export_none_bed() : m.export_preview_empty()}</p>
        {/if}
      </div>
      <p class="help">{m.export_pdf_help()}</p>
    </section>
  </div>
</div>

<!-- The drawing that is serialised: off screen, never shown. -->
<div class="outside-screen" aria-hidden="true" bind:this={drawing}>
  {#if scale && rows.length > 0}
    <TimelineSvg
      {rows}
      {scale}
      widthLabels={WIDTH_LABELS}
      today={app.view?.today ?? todayLocal()}
      cursor={null}
      interactive={false}
      yearShown={app.view?.year}
      annotations={notes}
      markerToday={marker}
      {phases}
    />
  {/if}
</div>

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
    justify-content: space-between;
    max-width: 1280px;
    margin: 0 auto 20px;
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
    grid-template-columns: minmax(0, 420px) minmax(0, 1fr);
    gap: 24px;
    align-items: start;
    max-width: 1280px;
    margin: 0 auto;
  }

  .options,
  .preview {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 24px;
    border-radius: 32px;
  }

  .options {
    background: var(--white);
  }

  .preview {
    background: var(--card);
  }

  h1 {
    font-size: 26px;
  }

  h2 {
    font-size: 15px;
  }

  .formats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .format {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 16px 10px;
    border: 2px solid var(--line);
    border-radius: 20px;
    background: var(--white);
    text-align: center;
  }

  .format span {
    color: var(--muted);
    font-size: 12px;
  }

  .format.active {
    border-color: var(--accent);
    background: #fff1e0;
  }

  fieldset {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 0;
    padding: 0;
    border: 0;
  }

  legend {
    display: flex;
    justify-content: space-between;
    width: 100%;
    padding: 0;
    margin-bottom: 8px;
    color: var(--muted);
    font-size: 13px;
    font-weight: 600;
  }

  .dates {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .dates label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: var(--muted);
    font-size: 12px;
    font-weight: 600;
  }

  input[type="date"] {
    min-height: 44px;
    padding: 0 12px;
    border: 0;
    border-radius: 14px;
    background: var(--bg);
    color: var(--ink);
    font: inherit;
  }

  .help {
    margin: 0;
    color: var(--muted);
    font-size: 13px;
  }

  .help.error {
    color: var(--crit);
  }

  .link {
    border: 0;
    background: none;
    color: var(--green);
    font: inherit;
    font-weight: 600;
    text-decoration: underline;
  }

  .puces {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .puce,
  .case {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
  }

  .puce {
    padding: 8px 12px;
    border-radius: var(--pill);
    background: var(--bg);
    font-weight: 600;
  }

  input[type="checkbox"] {
    accent-color: var(--green);
    width: 18px;
    height: 18px;
  }

  .carte-error {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    padding: 14px 16px;
    border-radius: 20px;
    background: var(--crit-bg);
    color: var(--crit);
  }

  .carte-error .title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .carte-error p {
    margin: 0;
    color: var(--ink);
    font-size: 14px;
  }

  .preview-tete {
    display: flex;
    justify-content: space-between;
    color: var(--muted);
    font-size: 13px;
  }

  .feuille {
    overflow: auto;
    padding: 16px;
    border-radius: 24px;
    background: var(--white);
  }

  .feuille img {
    display: block;
    width: 100%;
    height: auto;
  }

  .empty {
    margin: 24px 0;
    color: var(--muted);
    text-align: center;
  }

  .outside-screen {
    position: absolute;
    left: -100000px;
    top: 0;
    width: 0;
    height: 0;
    overflow: hidden;
  }

  @media (max-width: 900px) {
    .screen {
      padding: 16px;
    }

    .columns {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
