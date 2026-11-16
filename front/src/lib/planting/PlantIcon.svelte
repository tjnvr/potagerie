<script lang="ts">
  // A small drawing of a crop at one of the four stages (S02: sowing, young
  // shoot, adult, harvest) for the cards of E07 and E09. It is the same drawing
  // as in the diorama (symboles.ts), framed on the plant: the young stages
  // are zoomed in, the tall crops get a taller frame.
  import { drawingOf } from "../appearances";
  import { PLANT_HEIGHT, SYMBOLES } from "../diorama/symboles";

  let {
    crop,
    stade = 2,
    width = 56,
  }: { crop: string; stade?: 0 | 1 | 2 | 3; width?: number } = $props();

  const STAGES = ["sowing", "young_shoot", "adult", "harvest"] as const;
  const key = $derived(drawingOf(crop));
  const drawing = $derived(SYMBOLES[`${key}-${STAGES[stade] ?? "adult"}`] ?? "");
  /** Height of the frame, in drawing units: the plant plus a margin. */
  const frame = $derived([22, 30, (PLANT_HEIGHT[key] ?? 30) + 8, (PLANT_HEIGHT[key] ?? 30) + 8][stade] ?? 38);
  const frameWidth = $derived(Math.max(frame * 1.25, 46));
</script>

<svg
  width={width}
  height={width * 0.8}
  viewBox="{-frameWidth / 2} {-(frame - 6)} {frameWidth} {frame}"
  aria-hidden="true"
>
  <ellipse cx="0" cy="1" rx={frameWidth * 0.36} ry={frameWidth * 0.1} fill="#A0704A" />
  <ellipse cx="0" cy="0" rx={frameWidth * 0.3} ry={frameWidth * 0.075} fill="#8E5F3D" />
  {@html drawing}
</svg>
