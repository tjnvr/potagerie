<script lang="ts">
  // The mini-timeline of the steps of a planting (S01 F-PLA-1, D02 E07): pots,
  // growth and harvest, with the dates Rust calculated. Same patterns as the
  // timeline: stripes, dots, solid.
  import { colorCrop, melange } from "../timeline/colors";
  import type { Segment } from "./planting";

  let { segments, cropId }: { segments: Segment[]; cropId: string } = $props();

  const color = $derived(colorCrop(cropId));
  const total = $derived(segments.reduce((sum, s) => sum + s.days, 0) || 1);
  const positions = $derived.by(() => {
    let cumulative = 0;
    return segments.map((s) => {
      const x = (cumulative / total) * 100;
      cumulative += s.days;
      return { ...s, x, width: (s.days / total) * 100 };
    });
  });
</script>

<svg class="mini" width="100%" height="14" aria-hidden="true">
  <defs>
    <pattern id="mini-pots" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
      <rect width="8" height="8" fill={melange(color, 0.85)} />
      <rect width="4" height="8" fill={melange(color, 0.55)} />
    </pattern>
    <pattern id="mini-growth" width="8" height="8" patternUnits="userSpaceOnUse">
      <rect width="8" height="8" fill={melange(color, 0.45)} />
      <circle cx="4" cy="4" r="0.9" fill={melange(color, 0.2)} />
    </pattern>
    <clipPath id="mini-forme"><rect width="100%" height="14" rx="7" /></clipPath>
  </defs>
  <g clip-path="url(#mini-forme)">
    {#each positions as s (s.phase)}
      <rect
        x="{s.x}%"
        width="{s.width}%"
        height="14"
        fill={s.phase === "pots" ? "url(#mini-pots)" : s.phase === "growth" ? "url(#mini-growth)" : color}
      />
    {/each}
  </g>
</svg>

<style>
  .mini {
    display: block;
  }
</style>
