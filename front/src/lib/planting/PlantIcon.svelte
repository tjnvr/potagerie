<script lang="ts">
  // A small drawing of a crop at one of the four stages (S02: sowing, jeune
  // grows, adult, récolte) for the cards of E07 and E09. Simplified: the
  // illustrations of the garden view (P01) belong to the diorama; this one
  // only needs to tell the crops and the stages apart at a glance.
  import { colorCrop, melange } from "../timeline/colors";
  import { drawingOf } from "../appearances";

  let {
    crop,
    stade = 2,
    width = 56,
  }: { crop: string; stade?: 0 | 1 | 2 | 3; width?: number } = $props();

  const color = $derived(colorCrop(crop));
  const light = $derived(melange(color, 0.35));
  const SHAPES: Record<string, "rosette" | "high" | "tuft" | "lames" | "large"> = {
    lettuce: "rosette",
    tomato: "high",
    green_bean: "high",
    carrot: "tuft",
    leek: "lames",
    zucchini: "large",
  };
  const forme = $derived(SHAPES[drawingOf(crop).replaceAll("-", "_")] ?? "rosette");
  /** Height of the plant, by stage. */
  const h = $derived([0, 0.35, 0.8, 1][stade] ?? 1);
  const fruit = $derived(stade === 3 ? (crop === "carrot" ? "#EF8A2E" : crop === "tomato" ? "#E5533D" : crop === "zucchini" ? "#F2C94C" : null) : null);
</script>

<svg width={width} height={width * 0.8} viewBox="0 0 60 48" aria-hidden="true">
  <ellipse cx="30" cy="38" rx="22" ry="7" fill="#A0704A" />
  <ellipse cx="30" cy="36.5" rx="19" ry="5.5" fill="#8E5F3D" />
  {#if stade === 0}
    <circle cx="24" cy="36" r="1.6" fill="#D9A86E" />
    <circle cx="31" cy="37.5" r="1.6" fill="#D9A86E" />
    <circle cx="37" cy="35.8" r="1.6" fill="#D9A86E" />
  {:else if forme === "rosette"}
    <ellipse cx="30" cy={35 - 6 * h} rx={6 + 9 * h} ry={3 + 7 * h} fill={color} />
    <ellipse cx="30" cy={34 - 6 * h} rx={3 + 6 * h} ry={2 + 5 * h} fill={light} />
  {:else if forme === "high"}
    <path d="M30 36V{36 - 28 * h}" stroke="#4A9450" stroke-width="2.4" stroke-linecap="round" />
    <ellipse cx={24 + 0} cy={32 - 20 * h} rx={5 * h + 1} ry={2.5 * h + 0.5} fill={color === "#E5533D" ? "#58AD56" : color} />
    <ellipse cx={36} cy={26 - 16 * h} rx={5 * h + 1} ry={2.5 * h + 0.5} fill={color === "#E5533D" ? "#58AD56" : color} />
    {#if stade === 3 && crop === "tomato"}
      <circle cx="25" cy="22" r="3.2" fill="#E5533D" />
      <circle cx="35" cy="16" r="3" fill="#E86A45" />
      <circle cx="26" cy="12" r="2.6" fill="#F08A4B" />
    {/if}
  {:else if forme === "lames"}
    {#each [-6, -2, 2, 6] as dx (dx)}
      <path d="M{30 + dx} 36L{30 + dx * 1.4} {36 - 26 * h}" stroke={color} stroke-width="3.2" stroke-linecap="round" />
    {/each}
  {:else if forme === "tuft"}
    {#each [-7, -3, 1, 5] as dx (dx)}
      <path d="M30 36Q{30 + dx} {36 - 14 * h} {30 + dx * 2} {36 - 20 * h}" stroke={color === "#EF8A2E" ? "#4F9A3A" : color} stroke-width="2" fill="none" stroke-linecap="round" />
    {/each}
    {#if fruit}<path d="M26 35h8l-4 4z" fill={fruit} />{/if}
  {:else}
    <ellipse cx="22" cy={34 - 5 * h} rx={8 * h + 2} ry={4 * h + 1.5} fill={color} />
    <ellipse cx="38" cy={34 - 6 * h} rx={8 * h + 2} ry={4 * h + 1.5} fill={light} />
    <ellipse cx="30" cy={32 - 8 * h} rx={7 * h + 2} ry={4 * h + 1.5} fill={color} />
    {#if fruit}<ellipse cx="30" cy="34" rx="5" ry="2.4" fill={fruit} />{/if}
  {/if}
</svg>
