<script lang="ts">
  // First launch without a terrain (ET01 ①), then the E2 form (F-TER-1).
  import { commands } from "../bindings";
  import { appli } from "../etat.svelte";
  import { m } from "../paraglide/messages.js";
  import Icone from "./Icone.svelte";
  import Logo from "./Logo.svelte";

  const CASES = [25, 50, 100];
  const ALLEES = [30, 50, 80];

  // `?formulaire` opens the form directly in dev, for screenshots.
  let formulaire = $state(import.meta.env.DEV && new URLSearchParams(location.search).has("formulaire"));
  let longueur = $state(9);
  let largeur = $state(6.5);
  let caseCm = $state(50);
  let alleeCm = $state(50);

  const lon = $derived(Math.round(Number(longueur) * 100));
  const lar = $derived(Math.round(Number(largeur) * 100));
  const valide = $derived(lon >= 100 && lar >= 100 && lon <= 10000 && lar <= 10000);
  const colonnes = $derived(valide ? Math.floor(lon / caseCm) : 0);
  const lignes = $derived(valide ? Math.floor(lar / caseCm) : 0);
  const metres = (valeur: number) => valeur.toLocaleString("fr-FR", { maximumFractionDigits: 2 });

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
    <div class="deux">
      <form class="formulaire" onsubmit={creer}>
        <div>
          <h2>{m.terrain_titre()}</h2>
          <p class="intro">{m.terrain_intro()}</p>
        </div>
        <div class="dimensions">
          <label>
            <span>{m.terrain_longueur()}</span>
            <span class="saisie">
              <input type="number" inputmode="decimal" min="1" max="100" step="0.5" required bind:value={longueur} />
              <abbr title={m.terrain_unite()}>{m.terrain_unite()}</abbr>
            </span>
          </label>
          <label>
            <span>{m.terrain_largeur()}</span>
            <span class="saisie">
              <input type="number" inputmode="decimal" min="1" max="100" step="0.5" required bind:value={largeur} />
              <abbr title={m.terrain_unite()}>{m.terrain_unite()}</abbr>
            </span>
          </label>
        </div>
        {#if !valide}<p class="alerte">{m.terrain_dimensions_invalides()}</p>{/if}
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
          <small>{m.terrain_case_verrou()}</small>
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
          <small>{m.terrain_allee_aide()}</small>
        </fieldset>
        <div class="actions">
          <button type="button" class="bouton" onclick={() => (formulaire = false)}>{m.annuler()}</button>
          <button type="submit" class="bouton principal" disabled={!valide}>
            <Icone nom="check" />{m.terrain_valider()}
          </button>
        </div>
      </form>

      <div class="apercu">
        <div class="apercu-titre">
          <strong>{m.terrain_apercu()}</strong>
          {#if valide}
            <span>
              {m.terrain_apercu_resume({
                longueur: metres(lon / 100),
                largeur: metres(lar / 100),
                case: caseCm,
                colonnes,
                lignes,
              })}
            </span>
          {/if}
        </div>
        {#if valide}
          <svg viewBox="0 0 {lon} {lar}" role="img" aria-label={m.terrain_apercu_aria()}>
            <rect width={lon} height={lar} rx={caseCm / 3} fill="var(--grass)" />
            {#each Array.from({ length: Math.max(colonnes - 1, 0) }, (_, i) => (i + 1) * caseCm) as x (x)}
              <line x1={x} x2={x} y1="0" y2={lar} class="grille" />
            {/each}
            {#each Array.from({ length: Math.max(lignes - 1, 0) }, (_, i) => (i + 1) * caseCm) as y (y)}
              <line y1={y} y2={y} x1="0" x2={lon} class="grille" />
            {/each}
          </svg>
        {/if}
        <p class="note">{m.terrain_apercu_vide()}</p>
      </div>
    </div>
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
    padding: 24px;
    border-radius: 32px;
    background: var(--card);
    text-align: center;
    color: var(--muted);
  }

  .deux {
    display: grid;
    grid-template-columns: minmax(0, 480px) minmax(0, 1fr);
    gap: 24px;
    width: 100%;
    align-items: start;
  }

  .intro {
    margin: 6px 0 0;
    font-size: 14px;
    color: var(--muted);
  }

  .alerte {
    margin: 0;
    padding: 10px 14px;
    border-radius: 14px;
    background: var(--crit-bg);
    color: #5a1e1a;
    font-size: 13px;
  }

  .saisie {
    display: flex;
    align-items: center;
    height: 48px;
    padding: 0 14px;
    border-radius: 14px;
    background: var(--bg);
    margin: 0;
  }

  .saisie input {
    flex: 1;
    min-width: 0;
    height: 100%;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--ink);
    font-weight: 700;
    font-size: 16px;
  }

  .saisie abbr {
    text-decoration: none;
    font-weight: 700;
    color: var(--muted);
  }

  .apercu {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 24px;
    border-radius: 28px;
    background: var(--card);
    text-align: left;
    color: var(--ink);
  }

  .apercu-titre {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 6px 16px;
    font-size: 13px;
  }

  .apercu-titre span,
  .note {
    color: var(--muted);
  }

  .apercu svg {
    display: block;
    width: 100%;
    max-height: 340px;
  }

  .grille {
    stroke: #ffffff;
    stroke-opacity: 0.4;
    stroke-width: 1;
    vector-effect: non-scaling-stroke;
  }

  .note {
    margin: 0;
    font-size: 13px;
  }

  @media (max-width: 900px) {
    .deux {
      grid-template-columns: minmax(0, 1fr);
    }
  }

  .formulaire {
    width: 100%;
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
