<script lang="ts">
  // Dev-only frame counter (`?fps` in the address, never in a build): the
  // metrics of P01, live while the season plays and as a result at its end.
  import { m } from "../paraglide/messages.js";
  import { CRITERION_LOW_1PCT, CRITERION_AVERAGE, respectsLeCriterion, type Measure } from "./measure";

  let {
    direct,
    plants,
    dernier,
  }: { direct: number; plants: number; dernier: (Measure & { plants: number }) | null } = $props();
</script>

<div class="counter" data-counter-images role="status">
  <strong>{m.diorama_fps_direct({ value: Math.round(direct) })}</strong>
  <span>{m.diorama_fps_plants({ count: plants })}</span>
  {#if dernier}
    <span class:bon={respectsLeCriterion(dernier)} class:bad={!respectsLeCriterion(dernier)}>
      {m.diorama_fps_result({
        average: Math.round(dernier.average),
        low: Math.round(dernier.bas1pct),
        target_average: CRITERION_AVERAGE,
        target_low: CRITERION_LOW_1PCT,
      })}
    </span>
  {/if}
</div>

<style>
  .counter {
    position: absolute;
    top: 8px;
    right: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 10px;
    border-radius: 12px;
    background: rgba(31, 58, 46, 0.86);
    color: var(--white);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    pointer-events: none;
  }

  .bon {
    color: #9be3b7;
  }

  .bad {
    color: #ffb4ab;
  }
</style>
