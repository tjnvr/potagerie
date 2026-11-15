<script lang="ts">
  // Dev-only frame counter (`?fps` in the address, never in a build): the
  // metrics of P01, live while the season plays and as a result at its end.
  import { m } from "../paraglide/messages.js";
  import { CRITERE_BAS_1PCT, CRITERE_MOYENNE, respecteLeCritere, type Mesure } from "./mesure";

  let {
    direct,
    plantes,
    dernier,
  }: { direct: number; plantes: number; dernier: (Mesure & { plantes: number }) | null } = $props();
</script>

<div class="compteur" data-compteur-images role="status">
  <strong>{m.diorama_fps_direct({ valeur: Math.round(direct) })}</strong>
  <span>{m.diorama_fps_plantes({ nombre: plantes })}</span>
  {#if dernier}
    <span class:bon={respecteLeCritere(dernier)} class:mauvais={!respecteLeCritere(dernier)}>
      {m.diorama_fps_resultat({
        moyenne: Math.round(dernier.moyenne),
        bas: Math.round(dernier.bas1pct),
        criteremoyenne: CRITERE_MOYENNE,
        criterebas: CRITERE_BAS_1PCT,
      })}
    </span>
  {/if}
</div>

<style>
  .compteur {
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

  .mauvais {
    color: #ffb4ab;
  }
</style>
