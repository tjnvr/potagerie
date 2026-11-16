<script lang="ts">
  // Garden tab (E05): the garden view with its time cursor, then the timeline.
  import type { YearView } from "../bindings";
  import CreationPlot from "../shell/CreationPlot.svelte";
  import Timeline from "../timeline/Timeline.svelte";
  import Diorama from "./Diorama.svelte";
  import { cursor } from "../time";
  import { todayLocal } from "../dates";

  let { view }: { view: YearView } = $props();

  // The tab is built again each time it is opened, and it opens on today (S01
  // F-VIS-3), not on the date the cursor was left on (end of the playback, a
  // week moved by hand). This runs before the timeline and the diorama read it.
  cursor.set(todayLocal());
</script>

{#if !view.plot}
  <CreationPlot />
{:else}
  <Diorama {view} />
  <Timeline {view} />
{/if}
