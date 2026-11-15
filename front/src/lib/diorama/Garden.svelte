<script lang="ts">
  // Garden tab (E05): the garden view with its time cursor, then the timeline.
  import type { YearView } from "../bindings";
  import CreationPlot from "../shell/CreationPlot.svelte";
  import Timeline from "../timeline/Timeline.svelte";
  import Diorama from "./Diorama.svelte";

  let { view }: { view: YearView } = $props();
</script>

{#if !view.plot}
  <CreationPlot />
{:else}
  <Diorama {view} />
  <Timeline {view} />
{/if}
