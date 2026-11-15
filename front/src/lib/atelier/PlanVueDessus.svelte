<script lang="ts">
  // Top view of the terrain and its planches (E03), in centimetres.
  // TODO(atelier): selection, drag and drop, handles (F-PLN-6).
  import type { PlancheView, TerrainView } from "../bindings";
  import { m } from "../paraglide/messages.js";

  let { terrain, planches }: { terrain: TerrainView; planches: PlancheView[] } = $props();

  const marge = $derived(Math.max(terrain.longueur_cm, terrain.largeur_cm) * 0.02);
  const police = $derived(Math.max(terrain.longueur_cm, terrain.largeur_cm) / 60);
  const lignesX = $derived(
    Array.from({ length: Math.floor(terrain.longueur_cm / terrain.case_cm) - 1 }, (_, i) => (i + 1) * terrain.case_cm),
  );
  const lignesY = $derived(
    Array.from({ length: Math.floor(terrain.largeur_cm / terrain.case_cm) - 1 }, (_, i) => (i + 1) * terrain.case_cm),
  );
  const pasDuMetre = $derived(100 / terrain.case_cm);
</script>

<svg
  class="plan"
  viewBox="{-marge} {-marge} {terrain.longueur_cm + 2 * marge} {terrain.largeur_cm + 2 * marge}"
  role="img"
  aria-label={m.plan_aria({ nombre: planches.length })}
>
  <rect
    width={terrain.longueur_cm}
    height={terrain.largeur_cm}
    rx={terrain.case_cm / 3}
    fill="var(--grass)"
  />
  {#each lignesX as x, i (x)}
    <line
      x1={x}
      x2={x}
      y1="0"
      y2={terrain.largeur_cm}
      class="grille"
      class:metre={(i + 1) % pasDuMetre === 0}
    />
  {/each}
  {#each lignesY as y, i (y)}
    <line
      y1={y}
      y2={y}
      x1="0"
      x2={terrain.longueur_cm}
      class="grille"
      class:metre={(i + 1) % pasDuMetre === 0}
    />
  {/each}

  {#each planches as planche (planche.id)}
    {@const bois = planche.planche_type === "bac" || planche.planche_type === "carre"}
    <g>
      <rect
        x={planche.x}
        y={planche.y}
        width={planche.longueur_cm}
        height={planche.largeur_cm}
        rx="6"
        fill={bois ? "var(--wood)" : "var(--bed)"}
      />
      {#if bois}
        <rect
          x={planche.x + 6}
          y={planche.y + 6}
          width={planche.longueur_cm - 12}
          height={planche.largeur_cm - 12}
          rx="3"
          fill="var(--bed)"
        />
      {:else}
        {#each Array.from({ length: Math.floor(planche.largeur_cm / 25) - 1 }, (_, k) => k + 1) as k (k)}
          <line
            x1={planche.x + 8}
            x2={planche.x + planche.longueur_cm - 8}
            y1={planche.y + k * 25}
            y2={planche.y + k * 25}
            class="sillon"
          />
        {/each}
      {/if}
      <text class="etiquette" x={planche.x + 10} y={planche.y + 10 + police} font-size={police}>
        {planche.nom}
      </text>
    </g>
  {/each}
</svg>

<style>
  .plan {
    display: block;
    width: 100%;
    height: auto;
    max-height: 70vh;
  }

  .grille {
    stroke: #ffffff;
    stroke-opacity: 0.25;
    stroke-width: 1;
    vector-effect: non-scaling-stroke;
  }

  .grille.metre {
    stroke-opacity: 0.55;
  }

  .sillon {
    stroke: var(--bed-furrow);
    stroke-width: 1.5;
    stroke-linecap: round;
    vector-effect: non-scaling-stroke;
  }

  .etiquette {
    font-family: var(--font-titre);
    font-weight: 600;
    fill: #ffffff;
    paint-order: stroke;
    stroke: rgba(31, 58, 46, 0.55);
    stroke-width: 3px;
    stroke-linejoin: round;
    vector-effect: non-scaling-stroke;
  }
</style>
