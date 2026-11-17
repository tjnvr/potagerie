<script lang="ts">
  import { localeTag } from "../locale";
  // E09 · Catalogue (D02 E09_catalogue_desktop; S01 F-CAT-3, S02): the 6 crops
  // of the V1, filtered by family and by « of season now », and the
  // read-only fiche. Names, families and months come from the messages and the
  // locale; the data only holds identifiers and numbers (S05 §4).
  import { tick } from "svelte";
  import { commands } from "../bindings";
  import Icon from "../shell/Icon.svelte";
  import { modal } from "../shell/modal";
  import Logo from "../shell/Logo.svelte";
    import { call, app, dateLong } from "../state.svelte";
  import IconTimeline from "../timeline/IconTimeline.svelte";
  import { m } from "../paraglide/messages.js";
  import { nameCrop } from "../beds";
  import { isOk } from "./call";
  import { catalogue } from "./crops.svelte";
  import PlantIcon from "./PlantIcon.svelte";
  import { nameFamily, tri } from "./catalogue";

  const today = $derived(app.view?.today ?? new Date().toISOString().slice(0, 10));
  let season = $state<string[]>([]);
  let onlySeason = $state(false);
  let family = $state<string | null>(null);
  let chosen = $state<string | null>(null);

  $effect(() => {
    const date = today;
    void call(() => isOk(commands.cropsOfSeason(date))).then((r) => {
      if (r.status === "ok") season = r.data;
    });
  });

  const families = $derived([...new Set(catalogue.crops.map((c) => c.family))].sort((a, b) => nameFamily(a).localeCompare(nameFamily(b), localeTag())));
  const list = $derived(
    tri(catalogue.crops, nameCrop).filter(
      (c) => (!family || c.family === family) && (!onlySeason || season.includes(c.id)),
    ),
  );
  // The sheet follows the cards shown: a crop filtered out no longer keeps its sheet.
  const fiche = $derived(catalogue.crop(list.find((c) => c.id === chosen)?.id ?? list[0]?.id ?? ""));
  let sheet = $state<HTMLElement>();

  // On a phone the sheet sits under the 67 cards: bring it into view.
  async function choose(id: string) {
    chosen = id;
    if (!window.matchMedia("(max-width: 900px)").matches) return;
    await tick();
    sheet?.scrollIntoView({ block: "start" });
  }

  function close() {
    app.catalogueOpen = false;
  }

  /** `MM-DD` as « 1er mars ». */
  function dayMonth(monthDay: string): string {
    const [month = 1, day = 1] = monthDay.split("-").map(Number);
    return new Date(Date.UTC(2001, month - 1, day)).toLocaleDateString(localeTag(), { day: "numeric", month: "long", timeZone: "UTC" });
  }

  const STAGES = [
    { stade: 0, text: m.catalogue_stage_sowing },
    { stade: 1, text: m.catalogue_stage_grows },
    { stade: 2, text: m.catalogue_stage_adult },
    { stade: 3, text: m.catalogue_stage_harvest },
  ] as const;
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && close()} />

<div class="screen" role="dialog" aria-modal="true" aria-labelledby="catalogue-title" tabindex="-1" use:modal>
  <header class="header">
    <Logo />
    <span class="name">{m.app_name()}</span>
    <div class="space"></div>
    <button class="rond" aria-label={m.close()} title={m.close()} onclick={close}><Icon name="close" size={18} /></button>
  </header>

  <div class="columns">
    <section class="list">
      <div class="tete">
        <h1 id="catalogue-title">{m.catalogue_title()}</h1>
        <label class="bascule">
          <input type="checkbox" role="switch" bind:checked={onlySeason} />
          <span class="switch" aria-hidden="true"></span>
          {m.catalogue_of_season({ date: dateLong(today) })}
        </label>
      </div>

      <div class="filters" role="group" aria-label={m.catalogue_filter_family()}>
        <button class="filter" class:active={family === null} aria-pressed={family === null} onclick={() => (family = null)}>
          {m.catalogue_all()}
        </button>
        {#each families as f (f)}
          <button class="filter" class:active={family === f} aria-pressed={family === f} onclick={() => (family = f)}>
            {nameFamily(f)}
          </button>
        {/each}
      </div>

      <div class="grille">
        {#each list as c (c.id)}
          <button class="carte" class:active={fiche?.id === c.id} aria-pressed={fiche?.id === c.id} onclick={() => choose(c.id)}>
            <PlantIcon crop={c.id} stade={2} width={64} />
            <strong>{nameCrop(c.id)}</strong>
            <span>{nameFamily(c.family)}</span>
            {#if season.includes(c.id)}<em class="badge">{m.catalogue_badge_season()}</em>{/if}
          </button>
        {:else}
          <p class="none">{m.catalogue_none()}</p>
        {/each}
      </div>
      <p class="note">{m.catalogue_count({ count: catalogue.crops.length })}</p>
    </section>

    {#if fiche}
      <aside class="fiche" bind:this={sheet} aria-label={m.catalogue_sheet()}>
        <div class="fiche-tete">
          <div>
            <p class="kicker">{m.catalogue_sheet()}</p>
            <h2>{nameCrop(fiche.id)}</h2>
          </div>
          {#if season.includes(fiche.id)}<em class="badge">{m.catalogue_badge_season()}</em>{/if}
        </div>
        <div class="stades">
          {#each STAGES as s (s.stade)}
            <figure>
              <PlantIcon crop={fiche.id} stade={s.stade} width={58} />
              <figcaption>{s.text()}</figcaption>
            </figure>
          {/each}
        </div>
        <dl>
          <div><dt>{m.catalogue_family()}</dt><dd>{nameFamily(fiche.family)}</dd></div>
          <div>
            <dt>{m.catalogue_indoor()}</dt>
            <dd>{fiche.indoor_days > 0 ? m.catalogue_days({ value: fiche.indoor_days }) : m.catalogue_without_indoor()}</dd>
          </div>
          <div><dt>{m.catalogue_after()}</dt><dd>{m.catalogue_days({ value: fiche.harvest_after_days })}</dd></div>
          <div><dt>{m.catalogue_duration()}</dt><dd>{m.catalogue_days({ value: fiche.harvest_duration_days })}</dd></div>
          <div>
            <dt>{m.catalogue_window()}</dt>
            <dd>{m.catalogue_window_value({ start: dayMonth(fiche.planting_out_start), end: dayMonth(fiche.planting_out_end) })}</dd>
          </div>
          <div><dt>{m.catalogue_row()}</dt><dd>{m.cm({ value: fiche.row_cm })}</dd></div>
          <div><dt>{m.catalogue_spacing()}</dt><dd>{m.cm({ value: fiche.spacing_cm })}</dd></div>
        </dl>
        <p class="lock"><IconTimeline name="lock" size={14} />{m.catalogue_read_only()}</p>
      </aside>
    {/if}
  </div>
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
    gap: 10px;
    max-width: 1300px;
    margin: 0 auto 20px;
  }

  .name {
    font-family: var(--font-title);
    font-weight: 600;
    font-size: 26px;
  }

  .space {
    flex: 1;
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
    grid-template-columns: minmax(0, 1.8fr) minmax(280px, 1fr);
    gap: 24px;
    align-items: start;
    max-width: 1300px;
    margin: 0 auto;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 24px;
    border-radius: 32px;
    background: var(--white);
  }

  .tete {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  h1 {
    font-size: 30px;
  }

  .bascule {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
  }

  .bascule input {
    position: absolute;
    opacity: 0;
  }

  .switch {
    position: relative;
    width: 38px;
    height: 22px;
    border-radius: var(--pill);
    background: var(--line);
    transition: background 0.15s;
  }

  .switch::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--white);
    transition: transform 0.15s;
  }

  .bascule input:checked + .switch {
    background: var(--green);
  }

  .bascule input:checked + .switch::after {
    transform: translateX(16px);
  }

  .bascule input:focus-visible + .switch {
    outline: 3px solid var(--accent);
    outline-offset: 2px;
  }

  .filters {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .filter {
    min-height: 34px;
    padding: 0 16px;
    border: 0;
    border-radius: var(--pill);
    background: var(--card);
    font-size: 14px;
    font-weight: 600;
  }

  .filter.active {
    background: var(--ink);
    color: var(--white);
  }

  .grille {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 14px;
  }

  .carte {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    min-height: 150px;
    padding: 18px 10px 14px;
    border: 2px solid var(--line);
    border-radius: 20px;
    background: var(--white);
  }

  .carte.active {
    border-color: var(--accent);
    background: var(--warn-bg);
  }

  .carte span {
    color: var(--muted);
    font-size: 13px;
  }

  .badge {
    padding: 1px 10px;
    border-radius: var(--pill);
    background: var(--green);
    color: var(--white);
    font-size: 11px;
    font-style: normal;
    font-weight: 700;
  }

  .none,
  .note {
    margin: 0;
    color: var(--muted);
    font-size: 13px;
  }

  .fiche {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 24px;
    border-radius: 32px;
    background: var(--card);
  }

  .fiche-tete {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
  }

  .kicker {
    margin: 0;
    color: var(--muted);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  h2 {
    font-size: 28px;
  }

  .stades {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 4px;
    padding: 6px 0 4px;
  }

  figure {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    margin: 0;
    font-size: 12px;
    color: var(--muted);
  }

  dl {
    margin: 0;
  }

  dl div {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 9px 0;
    border-top: 1px solid var(--line);
    font-size: 14px;
  }

  dt {
    color: var(--muted);
  }

  dd {
    margin: 0;
    font-weight: 700;
    text-align: right;
  }

  .lock {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    color: var(--muted);
    font-size: 13px;
  }

  @media (max-width: 900px) {
    .screen {
      padding: 12px 12px 32px;
    }
    .columns {
      grid-template-columns: minmax(0, 1fr);
    }
    .list,
    .fiche {
      padding: 16px;
      border-radius: 24px;
    }
    h1 {
      font-size: 24px;
    }
    /* 67 crops: two cards a row, and the families on one scrolling line. */
    .grille {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 10px;
    }
    .carte {
      min-height: 112px;
      padding: 12px 8px 10px;
    }
    .filters {
      flex-wrap: nowrap;
      overflow-x: auto;
      scrollbar-width: none;
      margin-inline: -16px;
      padding-inline: 16px;
    }
    .filter {
      flex: none;
    }
  }
</style>
