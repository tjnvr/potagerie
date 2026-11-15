<script lang="ts">
  // The mini-frise of the steps of a serie (S01 F-PLA-1, D02 E07): pots,
  // growth and harvest, with the dates Rust calculated. Same patterns as the
  // timeline: stripes, dots, solid.
  import { couleurCulture, melange } from "../frise/couleurs";
  import type { Segment } from "./serie";

  let { segments, cultureId }: { segments: Segment[]; cultureId: string } = $props();

  const couleur = $derived(couleurCulture(cultureId));
  const total = $derived(segments.reduce((somme, s) => somme + s.jours, 0) || 1);
  const positions = $derived.by(() => {
    let cumul = 0;
    return segments.map((s) => {
      const x = (cumul / total) * 100;
      cumul += s.jours;
      return { ...s, x, largeur: (s.jours / total) * 100 };
    });
  });
</script>

<svg class="mini" width="100%" height="14" aria-hidden="true">
  <defs>
    <pattern id="mini-godets" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
      <rect width="8" height="8" fill={melange(couleur, 0.85)} />
      <rect width="4" height="8" fill={melange(couleur, 0.55)} />
    </pattern>
    <pattern id="mini-croissance" width="8" height="8" patternUnits="userSpaceOnUse">
      <rect width="8" height="8" fill={melange(couleur, 0.45)} />
      <circle cx="4" cy="4" r="0.9" fill={melange(couleur, 0.2)} />
    </pattern>
    <clipPath id="mini-forme"><rect width="100%" height="14" rx="7" /></clipPath>
  </defs>
  <g clip-path="url(#mini-forme)">
    {#each positions as s (s.phase)}
      <rect
        x="{s.x}%"
        width="{s.largeur}%"
        height="14"
        fill={s.phase === "godets" ? "url(#mini-godets)" : s.phase === "croissance" ? "url(#mini-croissance)" : couleur}
      />
    {/each}
  </g>
</svg>

<style>
  .mini {
    display: block;
  }
</style>
