<script lang="ts">
  // The time cursor under the garden view (D01 scrubber, S01 F-VIS-3): play
  // button, a week back and forward, the slider over the year with its months.
  import Icone from "../coquille/Icone.svelte";
  import { m } from "../paraglide/messages.js";
  import { jourDepuis } from "./jours";

  let {
    annee,
    jour,
    lecture,
    vitesse,
    compact = false,
    onchoisir,
    onjouer,
    onvitesse,
  }: {
    annee: number;
    /** The day of the cursor, a day number (see jours.ts). */
    jour: number;
    lecture: boolean;
    vitesse: number;
    compact?: boolean;
    onchoisir: (jour: number) => void;
    onjouer: () => void;
    onvitesse: (vitesse: number) => void;
  } = $props();

  const debut = $derived(jourDepuis(`${annee}-01-01`));
  const fin = $derived(jourDepuis(`${annee}-12-31`));
  const progression = $derived(Math.min(1, Math.max(0, (jour - debut) / (fin - debut))));

  /** The months of the year, with their length for the ticks. */
  const mois = $derived(
    Array.from({ length: 12 }, (_, i) => {
      const premier = Date.UTC(annee, i, 1);
      const suivant = Date.UTC(annee, i + 1, 1);
      const nom = new Date(premier).toLocaleDateString("fr-FR", { month: "short", timeZone: "UTC" });
      return { nom, jours: (suivant - premier) / 86_400_000 };
    }),
  );

  function semaine(sens: 1 | -1) {
    onchoisir(Math.min(fin, Math.max(debut, Math.floor(jour) + 7 * sens)));
  }
</script>

<div class="curseur" class:compact>
  <button
    class="jouer"
    type="button"
    data-jouer
    aria-pressed={lecture}
    aria-label={lecture ? m.diorama_pause() : m.diorama_jouer()}
    title={lecture ? m.diorama_pause() : m.diorama_jouer()}
    onclick={onjouer}
  >
    <Icone nom={lecture ? "pause" : "play"} taille={22} />
  </button>
  <button class="pas" type="button" aria-label={m.diorama_semaine_precedente()} onclick={() => semaine(-1)}>
    <Icone nom="back" taille={16} />
  </button>
  <div class="piste">
    <input
      type="range"
      min={debut}
      max={fin}
      step="1"
      value={Math.floor(jour)}
      style:--progression="{progression * 100}%"
      aria-label={m.diorama_curseur()}
      oninput={(e) => onchoisir(Number(e.currentTarget.value))}
    />
    <div class="mois" aria-hidden="true">
      {#each mois as mois_ (mois_.nom)}
        <span style:flex="{mois_.jours} 1 0">{compact ? mois_.nom.slice(0, 1) : mois_.nom}</span>
      {/each}
    </div>
  </div>
  <button class="pas suivant" type="button" aria-label={m.diorama_semaine_suivante()} onclick={() => semaine(1)}>
    <Icone nom="back" taille={16} />
  </button>
  <div class="vitesse" role="group" aria-label={m.diorama_vitesse()}>
    {#each [1, 2] as v (v)}
      <button type="button" aria-pressed={vitesse === v} onclick={() => onvitesse(v)}>×{v}</button>
    {/each}
  </div>
</div>

<style>
  .curseur {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .jouer {
    flex: none;
    display: grid;
    place-items: center;
    width: 56px;
    height: 56px;
    border: 0;
    border-radius: 50%;
    background: var(--accent);
    color: var(--accent-ink);
    box-shadow: 0 6px 16px rgba(242, 166, 90, 0.35);
  }

  .pas {
    flex: none;
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border: 1px solid var(--line);
    border-radius: 50%;
    background: var(--white);
  }

  .suivant :global(svg) {
    transform: scaleX(-1);
  }

  .piste {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  input[type="range"] {
    width: 100%;
    height: 28px;
    margin: 0;
    background: transparent;
    appearance: none;
    cursor: pointer;
  }

  input[type="range"]::-webkit-slider-runnable-track {
    height: 10px;
    border-radius: var(--pill);
    background: linear-gradient(to right, #f8d6b0 var(--progression), var(--white) var(--progression));
  }

  input[type="range"]::-moz-range-track {
    height: 10px;
    border-radius: var(--pill);
    background: linear-gradient(to right, #f8d6b0 var(--progression), var(--white) var(--progression));
  }

  input[type="range"]::-webkit-slider-thumb {
    width: 24px;
    height: 24px;
    margin-top: -7px;
    border: 4px solid var(--accent);
    border-radius: 50%;
    background: var(--white);
    appearance: none;
  }

  input[type="range"]::-moz-range-thumb {
    width: 16px;
    height: 16px;
    border: 4px solid var(--accent);
    border-radius: 50%;
    background: var(--white);
  }

  .mois {
    display: flex;
    font-size: 11px;
    color: var(--muted);
  }

  .mois span {
    min-width: 0;
    padding-left: 4px;
    border-left: 1px solid var(--line);
    overflow: hidden;
    white-space: nowrap;
  }

  .vitesse {
    flex: none;
    display: flex;
    padding: 3px;
    border-radius: var(--pill);
    background: var(--white);
  }

  .vitesse button {
    min-width: 40px;
    min-height: 36px;
    border: 0;
    border-radius: var(--pill);
    background: transparent;
    color: var(--muted);
    font-weight: 600;
    font-size: 13px;
  }

  .vitesse button[aria-pressed="true"] {
    background: var(--ink);
    color: var(--white);
  }

  .compact .jouer {
    width: 48px;
    height: 48px;
  }

  @media (max-width: 720px) {
    .curseur {
      flex-wrap: wrap;
      gap: 8px 10px;
    }
    .piste {
      order: 5;
      flex: 1 1 100%;
    }
    .vitesse {
      margin-left: auto;
    }
  }
</style>
