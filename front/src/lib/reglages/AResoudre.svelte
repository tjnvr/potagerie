<script lang="ts">
  // The « to resolve » list kept after an import (S03 §13): Rust computes it
  // from the plan of the device after each change, so it empties by itself
  // when each case is settled with the usual actions. Nothing is corrected here.
  import Icone from "../coquille/Icone.svelte";
  import { appli } from "../etat.svelte";
  import { m } from "../paraglide/messages.js";
  import { texteIncoherence } from "./incoherences";

  let ouverte = $state(false);
</script>

<section class="bandeau" aria-labelledby="resoudre-titre">
  <div class="tete">
    <Icone nom="alert" taille={18} />
    <strong id="resoudre-titre">{m.resoudre_titre({ nombre: appli.aResoudre.length })}</strong>
    <button class="lien" aria-expanded={ouverte} onclick={() => (ouverte = !ouverte)}>
      {ouverte ? m.resoudre_masquer() : m.resoudre_afficher()}
    </button>
  </div>
  {#if ouverte}
    <p>{m.resoudre_aide()}</p>
    <ul>
      {#each appli.aResoudre as incoherence, i (i)}
        <li>{texteIncoherence(incoherence)}</li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .bandeau {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 16px;
    padding: 12px 18px;
    border-radius: 20px;
    background: var(--warn-bg);
    color: var(--ink);
  }

  .tete {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .tete strong {
    flex: 1;
  }

  p {
    margin: 0;
    color: var(--muted);
    font-size: 14px;
  }

  ul {
    margin: 0;
    padding-left: 20px;
    font-size: 14px;
  }

  .lien {
    border: 0;
    background: none;
    color: var(--green);
    font: inherit;
    font-weight: 600;
    text-decoration: underline;
  }
</style>
