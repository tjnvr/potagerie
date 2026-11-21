<script lang="ts">
  // The shell: header, the two tabs without a router (S05 §4), settings and
  // the message line.
  import AisleBubble from "./lib/editor/AisleBubble.svelte";
  import Editor from "./lib/editor/Editor.svelte";
  import Header from "./lib/shell/Header.svelte";
  import Icon from "./lib/shell/Icon.svelte";
  import Garden from "./lib/diorama/Garden.svelte";
  import { app } from "./lib/state.svelte";
  import { today } from "./lib/time";
  import { translateError } from "./lib/errors";
  import { m } from "./lib/paraglide/messages.js";
  import Settings from "./lib/settings/Settings.svelte";
  import { catalogue } from "./lib/planting/crops.svelte";
  import Catalogue from "./lib/planting/Catalogue.svelte";
  import ExportTimeline from "./lib/timeline/ExportTimeline.svelte";
  import ToResolve from "./lib/settings/ToResolve.svelte";
  import NoteEdition from "./lib/notes/NoteEdition.svelte";
  import PlantingEdition from "./lib/planting/PlantingEdition.svelte";

  $effect(() => {
    app.load();
    catalogue.load();
  });

  // Past midnight, Rust computes « today » again: read-only years, suggestions,
  // the appearance of the day. Without this the view would keep yesterday's.
  $effect(() => {
    let previous: string | null = null;
    return today.subscribe((day) => {
      if (previous !== null && day !== previous && app.view) void app.load();
      previous = day;
    });
  });

  $effect(() => {
    if (!app.message) return;
    const delay = setTimeout(() => (app.message = null), 6000);
    return () => clearTimeout(delay);
  });
</script>

<Header />

<main class="page">
  {#if app.toResolve.length > 0}
    <ToResolve />
  {/if}
  {#if !app.view && app.loadFailure}
    <section class="load-failed" role="alert">
      <h1>{m.load_failed_title()}</h1>
      <p>{translateError(app.loadFailure)}</p>
      <p class="muted">{m.load_failed_text()}</p>
      <div class="actions">
        <button class="button" onclick={() => app.load()}><Icon name="retry" size={16} />{m.retry()}</button>
        <button class="button" onclick={() => (app.settingsOpen = true)}>{m.load_failed_settings()}</button>
      </div>
    </section>
  {:else if !app.view}
    <p class="loading">{m.loading()}</p>
  {:else if app.tab === "plan"}
    <Editor view={app.view} />
  {:else}
    <Garden view={app.view} />
  {/if}
</main>

{#if app.settingsOpen}
  <Settings />
{/if}

{#if app.view && app.plantingOpen}
  <PlantingEdition opening={app.plantingOpen} />
{/if}

{#if app.view && app.noteOpen}
  <NoteEdition opening={app.noteOpen} />
{/if}

{#if app.view && app.exportOpen}
  <ExportTimeline />
{/if}

{#if app.catalogueOpen}
  <Catalogue />
{/if}

{#if app.saving === "error"}
  <div class="failure" role="alert">
    <span class="pastille"><Icon name="alert" size={16} />{m.state_error()}</span>
    <p>{m.error_saving_text()}</p>
    <button class="button" onclick={() => app.retry()}><Icon name="retry" size={16} />{m.retry()}</button>
  </div>
{/if}

{#if app.aisleQuestion && app.view?.plot}
  <AisleBubble aisle={app.view.plot.aisle_cm} answer={app.aisleQuestion.answer} />
{/if}

{#if app.message}
  <div class="announce" class:error={app.message.tone === "error"} role="alert">
    {#if app.message.tone === "error"}<Icon name="alert" />{/if}
    <span>{app.message.text}</span>
    <button aria-label={m.close()} onclick={() => (app.message = null)}><Icon name="close" size={14} /></button>
  </div>
{/if}

<style>
  .page {
    padding: 0 32px 32px;
    max-width: 1440px;
  }

  .loading {
    color: var(--muted);
  }

  .load-failed {
    max-width: 520px;
    margin: 48px auto;
    padding: 24px;
    border-radius: 20px;
    background: var(--card);
    text-align: center;
  }

  .load-failed .muted {
    color: var(--muted);
    font-size: 14px;
  }

  .load-failed .actions {
    display: flex;
    gap: 10px;
    justify-content: center;
    flex-wrap: wrap;
    margin-top: 16px;
  }

  /* ET01, state 4: the write failed, the change stays on screen. */
  .failure {
    position: fixed;
    left: 50%;
    bottom: 80px;
    transform: translateX(-50%);
    z-index: 19;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    width: min(440px, calc(100vw - 32px));
    padding: 18px;
    border-radius: 20px;
    background: var(--card);
    box-shadow: 0 8px 20px rgba(31, 58, 46, 0.2);
    text-align: center;
    font-size: 14px;
    color: var(--muted);
  }

  .failure p {
    margin: 0;
  }

  .failure .pastille {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    border-radius: var(--pill);
    background: var(--crit-bg);
    color: var(--crit);
    font-weight: 700;
  }

  .announce {
    position: fixed;
    left: 50%;
    bottom: 24px;
    transform: translateX(-50%);
    z-index: 20;
    display: flex;
    align-items: center;
    gap: 10px;
    width: max-content;
    max-width: calc(100vw - 32px);
    padding: 12px 12px 12px 16px;
    border-radius: 16px;
    background: var(--ink);
    color: var(--white);
    font-size: 14px;
    box-shadow: 0 8px 20px rgba(31, 58, 46, 0.25);
  }

  .announce.error {
    background: var(--crit-bg);
    color: #5a1e1a;
  }

  .announce button {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: inherit;
  }

  @media (max-width: 720px) {
    .page {
      padding: 0 12px 24px;
    }
  }
</style>
