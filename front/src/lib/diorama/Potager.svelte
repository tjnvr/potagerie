<script lang="ts">
  // Potager tab (E05): the garden view with its time cursor, then the timeline.
  import type { YearView } from "../bindings";
  import CreationTerrain from "../coquille/CreationTerrain.svelte";
  import Frise from "../frise/Frise.svelte";
  import Diorama from "./Diorama.svelte";

  let { vue }: { vue: YearView } = $props();
</script>

{#if !vue.terrain}
  <CreationTerrain />
{:else}
  <Diorama {vue} />
  <Frise {vue} />
{/if}
