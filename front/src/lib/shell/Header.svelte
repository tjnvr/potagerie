<script lang="ts">
  // Header of D01 (E03, E05): logo, Garden / Plan tabs, season, today,
  // save indicator (F-CPT-2, ET01) and the settings button of D02.
  import { get } from "svelte/store";
  import { sameDayYear } from "../dates";
  import { app, type Tab } from "../state.svelte";
  import IconTimeline from "../timeline/IconTimeline.svelte";
  import { m } from "../paraglide/messages.js";
  import { yearsProposees } from "../beds";
  import { goTo, allerToday, cursor } from "../time";
  import Icon, { type NameIcon } from "./Icon.svelte";
  import Logo from "./Logo.svelte";

  const yearInProgress = new Date().getFullYear();
  /** A season of the selector: the cursor goes to the same day of that year. */
  function chooseYear(year: number) {
    goTo(sameDayYear(get(cursor), year));
    void app.load(year);
  }

  /** “Today” brings back the cursor, the timeline and the season. */
  function returnToday() {
    allerToday();
    void app.load(yearInProgress);
  }

  const tabs: { id: Tab; icon: NameIcon; text: () => string }[] = [
    { id: "garden", icon: "garden", text: m.tab_garden },
    { id: "plan", icon: "edit", text: m.tab_plan },
  ];
</script>

<header class="header">
  <div class="marque">
    <Logo />
    <span class="name">{m.app_name()}</span>
  </div>

  <nav class="modes" aria-label={m.mode_aria()}>
    {#each tabs as tab (tab.id)}
      <button
        class="mode"
        class:active={app.tab === tab.id}
        aria-current={app.tab === tab.id ? "page" : undefined}
        onclick={() => (app.tab = tab.id)}
      >
        <Icon name={tab.icon} />{tab.text()}
      </button>
    {/each}
  </nav>

  <div class="seasons" role="group" aria-label={m.season_aria()}>
    {#each yearsProposees(yearInProgress) as year (year)}
      <button
        class="season"
        class:active={app.year === year}
        aria-pressed={app.year === year}
        onclick={() => chooseYear(year)}>{year}</button
      >
    {/each}
  </div>

  <button class="button today" onclick={returnToday}>
    <Icon name="cal" size={16} />{m.today()}
  </button>

  <div class="space"></div>

  <div class="state" role="status">
    {#if app.saving === "saved"}
      <span class="ok"><Icon name="check" size={16} /></span><span class="label">{m.state_saved()}</span>
    {:else if app.saving === "saving"}
      <span class="label">{m.state_saving()}</span>
    {:else}
      <span class="error"><Icon name="alert" size={16} />{m.state_error()}</span>
      <button class="button retry" onclick={() => app.retry()}>
        <Icon name="retry" size={16} />{m.retry()}
      </button>
    {/if}
  </div>

  <button
    class="rond"
    aria-label={m.catalogue_open()}
    title={m.catalogue_open()}
    aria-expanded={app.catalogueOpen}
    onclick={() => (app.catalogueOpen = true)}
  >
    <IconTimeline name="book" size={17} />
  </button>

  <button
    class="rond"
    aria-label={m.settings()}
    title={m.settings()}
    aria-expanded={app.settingsOpen}
    onclick={() => (app.settingsOpen = true)}
  >
    <Icon name="settings" size={17} />
  </button>
</header>

<style>
  .header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px 18px;
    padding: 20px 32px;
  }

  .marque {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-right: 8px;
  }

  .name {
    font-family: var(--font-title);
    font-weight: 600;
    font-size: 26px;
  }

  .modes,
  .seasons {
    display: flex;
    gap: 2px;
    padding: 4px;
    border-radius: var(--pill);
    background: var(--white);
    border: 1px solid var(--line);
  }

  .mode,
  .season {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 40px;
    padding: 0 18px;
    border: 0;
    border-radius: var(--pill);
    background: transparent;
    font-weight: 600;
    font-size: 15px;
  }

  .season {
    padding: 0 14px;
    font-size: 14px;
    color: var(--muted);
    height: 36px;
  }

  .mode.active,
  .season.active {
    background: var(--ink);
    color: var(--white);
  }

  .today {
    min-height: 40px;
    font-size: 14px;
    padding: 0 16px;
  }

  .space {
    flex: 1 1 20px;
  }

  .state {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--muted);
  }

  .ok {
    display: inline-flex;
    color: var(--good);
  }

  .error {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: var(--pill);
    background: var(--crit-bg);
    color: var(--crit);
    font-weight: 700;
  }

  .retry {
    min-height: 36px;
    font-size: 13px;
    padding: 0 12px;
  }

  .rond {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    border: 1px solid var(--line);
    background: var(--white);
  }

  /* Very narrow: only the mark of the save state stays visible, so that the
     header does not wrap onto a third row (the text stays for screen readers). */
  @media (max-width: 480px) {
    .state .label {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip-path: inset(50%);
      white-space: nowrap;
    }
  }

  /* Phone (lot 2 layout, prepared now): two rows, smaller marks. */
  @media (max-width: 720px) {
    .header {
      padding: 14px 16px;
      gap: 10px;
    }
    .name {
      font-size: 22px;
    }
    .modes {
      order: 10;
      flex: 1 1 100%;
    }
    .mode {
      flex: 1;
      justify-content: center;
      padding: 0 10px;
    }
    .seasons {
      order: 11;
      flex: 1 1 auto;
      overflow-x: auto;
    }
    .season {
      flex: 1;
      justify-content: center;
      padding: 0 8px;
    }
    .today {
      order: 12;
    }
    .state {
      font-size: 12px;
    }
  }
</style>
