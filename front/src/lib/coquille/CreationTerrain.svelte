<script lang="ts">
  // First launch without a terrain (ET01 ①), then the E2 form (F-TER-1).
  import { commands } from "../bindings";
  import { appli } from "../etat.svelte";
  import { m } from "../paraglide/messages.js";
  import Icone from "./Icone.svelte";
  import Logo from "./Logo.svelte";

  const CASES = [25, 50, 100];
  const ALLEES = [30, 50, 80];

  let formulaire = $state(false);
  let longueur = $state(11);
  let largeur = $state(6.5);
  let caseCm = $state(50);
  let alleeCm = $state(50);

  async function creer(event: SubmitEvent) {
    event.preventDefault();
    await appli.modifier((annee) =>
      commands.creerTerrain(annee, {
        longueur_cm: Math.round(longueur * 100),
        largeur_cm: Math.round(largeur * 100),
        case_cm: caseCm,
        allee_cm: alleeCm,
      }),
    );
  }
</script>

<section class="vide">
  {#if !formulaire}
    <Logo taille={40} />
    <p>{m.terrain_absent()}</p>
    <button class="bouton principal" onclick={() => (formulaire = true)}>
      <Icone nom="plus" />{m.terrain_creer()}
    </button>
  {:else}
    <form class="formulaire" onsubmit={creer}>
      <h2>{m.terrain_titre()}</h2>
      <div class="dimensions">
        <label>
          <span>{m.terrain_longueur()}</span>
          <input type="number" min="1" max="100" step="0.5" required bind:value={longueur} />
        </label>
        <label>
          <span>{m.terrain_largeur()}</span>
          <input type="number" min="1" max="100" step="0.5" required bind:value={largeur} />
        </label>
      </div>
      <fieldset>
        <legend>{m.terrain_case()}</legend>
        <div class="choix">
          {#each CASES as valeur (valeur)}
            <label class:actif={caseCm === valeur}>
              <input type="radio" name="case" value={valeur} bind:group={caseCm} />
              {m.cm({ valeur })}
            </label>
          {/each}
        </div>
        <small>{m.terrain_case_aide()}</small>
      </fieldset>
      <fieldset>
        <legend>{m.terrain_allee()}</legend>
        <div class="choix">
          {#each ALLEES as valeur (valeur)}
            <label class:actif={alleeCm === valeur}>
              <input type="radio" name="allee" value={valeur} bind:group={alleeCm} />
              {m.cm({ valeur })}
            </label>
          {/each}
        </div>
      </fieldset>
      <div class="actions">
        <button type="button" class="bouton" onclick={() => (formulaire = false)}>{m.annuler()}</button>
        <button type="submit" class="bouton principal"><Icone nom="check" />{m.terrain_valider()}</button>
      </div>
    </form>
  {/if}
</section>

<style>
  .vide {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    min-height: 360px;
    padding: 32px 20px;
    border-radius: 32px;
    background: var(--card);
    text-align: center;
    color: var(--muted);
  }

  .formulaire {
    width: min(460px, 100%);
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 24px;
    border-radius: 28px;
    background: var(--white);
    color: var(--ink);
    text-align: left;
  }

  .dimensions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  label > span,
  legend {
    display: block;
    font-size: 12px;
    font-weight: 700;
    color: var(--muted);
    margin-bottom: 6px;
  }

  input[type="number"] {
    width: 100%;
    height: 44px;
    padding: 0 12px;
    border-radius: 14px;
    border: 1px solid var(--line);
    background: var(--bg);
  }

  fieldset {
    border: 0;
    margin: 0;
    padding: 0;
  }

  small {
    display: block;
    margin-top: 6px;
    color: var(--muted);
  }

  .choix {
    display: flex;
    gap: 4px;
    padding: 4px;
    border-radius: var(--pill);
    background: var(--bg);
  }

  .choix label {
    flex: 1;
    text-align: center;
    padding: 10px 0;
    border-radius: var(--pill);
    font-weight: 600;
    font-size: 14px;
    cursor: pointer;
  }

  .choix label.actif {
    background: var(--ink);
    color: var(--white);
  }

  .choix input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .choix label:has(input:focus-visible) {
    outline: 3px solid var(--accent);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 10px;
  }
</style>
