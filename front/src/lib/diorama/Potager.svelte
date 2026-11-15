<script lang="ts">
  // Potager tab (E05), provisional: what the year view holds.
  // TODO(diorama): isometric garden view and season playback (P01, S05 §4).
  // TODO(frise): timeline of the series (src/lib/frise).
  import type { YearView } from "../bindings";
  import CreationTerrain from "../coquille/CreationTerrain.svelte";
  import { m } from "../paraglide/messages.js";

  let { vue }: { vue: YearView } = $props();
</script>

{#if !vue.terrain}
  <CreationTerrain />
{:else}
  <section class="scene">
    <h1>{m.potager_titre({ annee: vue.year })}</h1>
    <p>{m.potager_provisoire()}</p>
    <ul class="puces">
      <li>{m.potager_planches({ nombre: vue.planches.length })}</li>
      <li>{m.potager_series({ nombre: vue.series.length })}</li>
      <li>{m.potager_creneaux({ nombre: vue.creneaux.length })}</li>
      <li>{m.potager_taches({ nombre: vue.tasks.length })}</li>
    </ul>
    {#if vue.series.length === 0}
      <p class="frise-vide">{m.frise_vide()}</p>
    {/if}
  </section>
{/if}

<style>
  .scene {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 24px 28px;
    border-radius: 32px;
    background: var(--card);
  }

  h1 {
    font-size: 34px;
  }

  p {
    margin: 0;
    color: var(--muted);
  }

  .puces {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .puces li {
    padding: 6px 12px;
    border-radius: var(--pill);
    background: var(--white);
    font-size: 13px;
    font-weight: 600;
  }

  .frise-vide {
    padding: 48px 20px;
    border: 1px dashed var(--line);
    border-radius: 16px;
    background: var(--white);
    text-align: center;
  }

  @media (max-width: 720px) {
    .scene {
      padding: 16px;
      border-radius: 24px;
    }
    h1 {
      font-size: 26px;
    }
  }
</style>
