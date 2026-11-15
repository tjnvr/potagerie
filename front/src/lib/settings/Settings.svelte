<script lang="ts">
  // Settings (E10, D02): allée width, export and import of the garden
  // (F-CPT-3), empty the plan (F-PLN-4), erase the data (F-CPT-4). A side
  // panel on a computer, the full screen on a phone.
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { commands, type AppError, type ImportSummaryView } from "../bindings";
  import Confirmation from "../shell/Confirmation.svelte";
  import Icon from "../shell/Icon.svelte";
  import { call, app, dateLong } from "../state.svelte";
  import { m } from "../paraglide/messages.js";
  import { textIncoherence } from "./incoherences";

  const AISLES = [30, 50, 80];
  const FILTERS = [{ name: m.filter_garden(), extensions: ["json"] }];

  type Import = { path: string; summary: ImportSummaryView };
  let importEnWaiting = $state<Import | null>(null);
  /** Why the last import or export failed (ET01): shown in the panel, not as a toast. */
  let failureFile = $state<AppError["code"] | null>(null);
  let effacement = $state<0 | 1 | 2>(0);
  let clearing = $state(false);
  let panel: HTMLElement;

  const plot = $derived(app.view?.plot ?? null);
  const nbBeds = $derived(app.view?.beds.length ?? 0);
  const nbPlantings = $derived(app.view?.beds.reduce((n, p) => n + p.plantings_deleted.length, 0) ?? 0);
  const readOnly = $derived(app.view?.read_only ?? false);
  const aClear = $derived(nbBeds > 0 && !readOnly);
  const incoherences = $derived(importEnWaiting?.summary.incoherences ?? []);

  $effect(() => {
    panel.focus();
  });

  function close() {
    app.settingsOpen = false;
  }

  /** Import and file errors stay in the panel; anything else is a message. */
  function fail(error: AppError) {
    if (["file", "import_invalid", "import_too_recent"].includes(error.code)) {
      failureFile = error.code;
    } else {
      app.signaler(error);
    }
  }

  async function changerAisle(aisle_cm: number) {
    const t = plot;
    if (!t) return;
    await app.modify((year) =>
      commands.modifyPlot(year, { length_cm: t.length_cm, width_cm: t.width_cm, aisle_cm }),
    );
  }

  async function exportNow() {
    failureFile = null;
    const today = new Date().toISOString().slice(0, 10);
    const path = await save({ defaultPath: m.export_garden_file_name({ date: today }), filters: FILTERS });
    if (!path) return;
    const result = await call(() => commands.exportGarden(path));
    if (result.status === "ok") app.informer(m.export_garden_ok());
    else fail(result.error);
  }

  async function chooseImport() {
    failureFile = null;
    importEnWaiting = null;
    const path = await open({ multiple: false, directory: false, filters: FILTERS });
    if (typeof path !== "string") return;
    const result = await call(() => commands.readImport(path));
    if (result.status === "ok") importEnWaiting = { path, summary: result.data };
    else fail(result.error);
  }

  async function importGarden() {
    const path = importEnWaiting?.path;
    importEnWaiting = null;
    if (!path) return;
    if (await app.modify((year) => commands.importGarden(year, path))) {
      app.informer(m.import_ok());
    }
  }

  async function clear() {
    clearing = false;
    if (await app.modify((year) => commands.clearPlan(year))) {
      app.informer(m.settings_clear_ok());
    }
  }

  async function erase() {
    effacement = 0;
    const result = await call(() => commands.eraseData());
    if (result.status === "ok") {
      await app.charger();
      app.informer(m.erase_ok());
    } else {
      app.signaler(result.error);
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="voile" onclick={close}></div>
<div
  class="panel"
  role="dialog"
  aria-modal="true"
  aria-labelledby="settings-title"
  tabindex="-1"
  bind:this={panel}
  onkeydown={(e) => e.key === "Escape" && close()}
>
  <div class="tete">
    <button class="rond back" aria-label={m.settings_back()} onclick={close}>
      <Icon name="back" size={16} />
    </button>
    <h1 id="settings-title">{m.settings()}</h1>
    <button class="rond close" aria-label={m.close()} onclick={close}><Icon name="close" size={16} /></button>
  </div>

  {#if plot}
    <section>
      <h2 class="petit">{m.settings_aisle()}</h2>
      <div class="choice" role="group" aria-label={m.settings_aisle()}>
        {#each AISLES as value (value)}
          <button
            class:active={plot.aisle_cm === value}
            aria-pressed={plot.aisle_cm === value}
            onclick={() => changerAisle(value)}>{m.cm({ value })}</button
          >
        {/each}
      </div>
    </section>
  {/if}

  <section>
    <h2 class="petit uppercase">{m.settings_data()}</h2>
    <div class="row">
      <span class="pastille"><Icon name="export" size={16} /></span>
      <div class="label"><strong>{m.export_garden_title()}</strong><span>{m.export_help()}</span></div>
      <button class="button" disabled={!plot} onclick={exportNow}>
        <Icon name="export" size={16} />{m.export()}
      </button>
    </div>
    <div class="row">
      <span class="pastille"><Icon name="upload" size={16} /></span>
      <div class="label"><strong>{m.import_title()}</strong><span>{m.import_help()}</span></div>
      <button class="button" onclick={chooseImport}><Icon name="upload" size={16} />{m.import()}</button>
    </div>
    {#if importEnWaiting}
      {@const summary = importEnWaiting.summary}
      {@const prudent = summary.older || incoherences.length > 0}
      <div class="carte" class:alert={prudent} role="group" aria-labelledby="import-title">
        <div class="carte-title">
          {#if prudent}<Icon name="alert" size={18} />{/if}
          <strong id="import-title">{summary.older ? m.import_older() : m.import_confirm()}</strong>
        </div>
        <p>
          {m.import_summary({
            date: dateLong(summary.updated_at),
            beds: summary.bed_count,
            plantings: summary.planting_count,
          })}
          {#if summary.older && app.view?.updated_at}
            {m.import_local_newer({ date: dateLong(app.view.updated_at) })}
          {/if}
        </p>
        {#if incoherences.length > 0}
          <p><strong>{m.import_incoherences_title({ count: incoherences.length })}</strong></p>
          <ul class="incoherences">
            {#each incoherences as incoherence, i (i)}
              <li>{textIncoherence(incoherence)}</li>
            {/each}
          </ul>
          <p>{m.import_incoherences_text()}</p>
        {/if}
        {#if summary.replaces}<p>{m.import_replaces_all()}</p>{/if}
        <div class="actions">
          <button class="button" onclick={() => (importEnWaiting = null)}>{m.cancel()}</button>
          <button class="button" class:alert={prudent} class:principal={!prudent} onclick={importGarden}>
            {prudent ? m.replace_anyway() : m.replace()}
          </button>
        </div>
      </div>
    {/if}
    {#if failureFile}
      <div class="carte error" role="alert">
        <div class="carte-title">
          <Icon name="alert" size={18} />
          <strong>
            {failureFile === "file" ? m.state_file_title() : m.state_import_impossible()}
          </strong>
        </div>
        <p>
          {failureFile === "file"
            ? m.state_file_text()
            : failureFile === "import_too_recent"
              ? m.state_import_too_recent()
              : m.state_import_unreadable()}
        </p>
      </div>
    {/if}
    <div class="row">
      <span class="pastille"><Icon name="retry" size={16} /></span>
      <div class="label">
        <strong>{m.settings_clear_title()}</strong>
        <span>{readOnly ? m.settings_clear_past() : nbBeds === 0 ? m.settings_clear_nothing() : m.settings_clear_help()}</span>
      </div>
      <button class="button" disabled={!aClear} onclick={() => (clearing = true)}>{m.settings_clear()}</button>
    </div>
    <div class="row">
      <span class="pastille danger"><Icon name="trash" size={16} /></span>
      <div class="label">
        <strong class="rouge">{m.erase_title()}</strong><span>{m.erase_help()}</span>
      </div>
      <button class="button danger" onclick={() => (effacement = 1)}>
        <Icon name="trash" size={16} />{m.erase()}
      </button>
    </div>
  </section>
</div>

{#if clearing}
  <Confirmation
    title={m.settings_clear_confirm({ year: app.year })}
    confirm={m.settings_clear_button()}
    onconfirm={clear}
    oncancel={() => (clearing = false)}
  >
    <p>{m.settings_clear_text({ beds: nbBeds, plantings: nbPlantings, year: app.year })}</p>
    <p>{m.settings_clear_kept()}</p>
  </Confirmation>
{/if}

{#if effacement === 1}
  <Confirmation
    title={m.erase_confirm_1()}
    confirm={m.continue_button()}
    onconfirm={() => (effacement = 2)}
    oncancel={() => (effacement = 0)}
  >
    <p>{m.erase_confirm_1_text()}</p>
  </Confirmation>
{:else if effacement === 2}
  <Confirmation
    title={m.erase_confirm_2()}
    confirm={m.erase_permanently()}
    onconfirm={erase}
    oncancel={() => (effacement = 0)}
  >
    <p>{m.erase_confirm_2_text()}</p>
  </Confirmation>
{/if}

<style>
  .voile {
    position: fixed;
    inset: 0;
    background: rgba(238, 242, 230, 0.6);
    z-index: 10;
  }

  .panel {
    position: fixed;
    top: 16px;
    right: 16px;
    bottom: 16px;
    width: min(500px, calc(100vw - 32px));
    overflow-y: auto;
    z-index: 11;
    display: flex;
    flex-direction: column;
    gap: 22px;
    padding: 24px;
    border-radius: 28px;
    background: var(--white);
    box-shadow: 0 12px 40px rgba(31, 58, 46, 0.15);
    outline: none;
  }

  .tete {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .tete h1 {
    font-size: 22px;
  }

  .rond {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border: 0;
    border-radius: 50%;
    background: var(--bg);
  }

  section {
    display: flex;
    flex-direction: column;
  }

  .petit {
    font-family: var(--font);
    font-size: 12px;
    font-weight: 700;
    color: var(--muted);
    margin-bottom: 8px;
  }

  .uppercase {
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--line);
    margin-bottom: 0;
  }

  .choice {
    display: flex;
    gap: 4px;
    padding: 4px;
    border-radius: var(--pill);
    background: var(--bg);
  }

  .choice button {
    flex: 1;
    height: 38px;
    border: 0;
    border-radius: var(--pill);
    background: transparent;
    font-weight: 600;
    font-size: 14px;
  }

  .choice button.active {
    background: var(--ink);
    color: var(--white);
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px 14px;
    padding: 14px 0;
    border-bottom: 1px solid var(--line);
  }

  .row:last-child {
    border-bottom: 0;
  }

  .pastille {
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    border-radius: 12px;
    background: var(--bg);
  }

  .pastille.danger {
    background: var(--crit-bg);
    color: var(--crit);
  }

  .label {
    flex: 1 1 120px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 15px;
  }

  .label span {
    font-size: 13px;
    color: var(--muted);
  }

  .button:disabled {
    opacity: 0.45;
  }

  .carte {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 8px 0 14px;
    padding: 16px;
    border: 1px solid var(--line);
    border-radius: 20px;
    background: var(--bg);
    font-size: 14px;
    line-height: 1.5;
  }

  .carte p {
    margin: 0;
  }

  .carte.alert {
    border-color: #f0c99d;
    background: var(--warn-bg);
  }

  .carte.error {
    border-color: #f1bdb9;
    background: var(--crit-bg);
  }

  .carte-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .carte.alert .carte-title {
    color: var(--warn);
  }

  .carte.error .carte-title {
    color: var(--crit);
  }

  .carte-title strong {
    color: var(--ink);
    font-size: 15px;
  }

  .incoherences {
    margin: 0;
    padding-left: 20px;
    max-height: 180px;
    overflow-y: auto;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 10px;
  }

  .back {
    display: none;
  }

  .rouge {
    color: var(--crit);
  }

  /* Phone: the full screen (D02, E10 mobile). */
  @media (max-width: 720px) {
    .panel {
      inset: 0;
      width: 100%;
      border-radius: 0;
      padding: 18px 16px;
      background: var(--bg);
      box-shadow: none;
    }

    .tete {
      justify-content: flex-start;
    }

    .row {
      flex-wrap: nowrap;
      gap: 10px;
    }

    .row .button {
      padding: 0 14px;
    }

    .pastille {
      background: transparent;
      width: 24px;
    }

    .back {
      display: grid;
      background: var(--white);
    }

    .close {
      display: none;
    }
  }
</style>
