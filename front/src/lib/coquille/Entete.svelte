<script lang="ts">
  // Header of D01 (E03, E05): logo, Potager / Plan tabs, season, today,
  // save indicator (F-CPT-2, ET01) and the settings button of D02.
  import { appli, type Onglet } from "../etat.svelte";
  import { m } from "../paraglide/messages.js";
  import { anneesProposees } from "../planches";
  import Icone, { type NomIcone } from "./Icone.svelte";
  import Logo from "./Logo.svelte";

  const anneeEnCours = new Date().getFullYear();
  const onglets: { id: Onglet; icone: NomIcone; texte: () => string }[] = [
    { id: "potager", icone: "garden", texte: m.onglet_potager },
    { id: "plan", icone: "edit", texte: m.onglet_plan },
  ];
</script>

<header class="entete">
  <div class="marque">
    <Logo />
    <span class="nom">{m.app_nom()}</span>
  </div>

  <nav class="modes" aria-label={m.mode_aria()}>
    {#each onglets as onglet (onglet.id)}
      <button
        class="mode"
        class:actif={appli.onglet === onglet.id}
        aria-current={appli.onglet === onglet.id ? "page" : undefined}
        onclick={() => (appli.onglet = onglet.id)}
      >
        <Icone nom={onglet.icone} />{onglet.texte()}
      </button>
    {/each}
  </nav>

  <div class="saisons" role="group" aria-label={m.saison_aria()}>
    {#each anneesProposees(anneeEnCours) as annee (annee)}
      <button
        class="saison"
        class:actif={appli.annee === annee}
        aria-pressed={appli.annee === annee}
        onclick={() => appli.charger(annee)}>{annee}</button
      >
    {/each}
  </div>

  <button class="bouton aujourdhui" onclick={() => appli.charger(anneeEnCours)}>
    <Icone nom="cal" taille={16} />{m.aujourdhui()}
  </button>

  <div class="espace"></div>

  <div class="etat" role="status">
    {#if appli.enregistrement === "enregistre"}
      <span class="ok"><Icone nom="check" taille={16} /></span>{m.etat_enregistre()}
    {:else if appli.enregistrement === "enregistrement"}
      {m.etat_enregistrement()}
    {:else}
      <span class="erreur"><Icone nom="alert" taille={16} />{m.etat_erreur()}</span>
      <button class="bouton reessayer" onclick={() => appli.reessayer()}>
        <Icone nom="retry" taille={16} />{m.reessayer()}
      </button>
    {/if}
  </div>

  <button
    class="rond"
    aria-label={m.reglages()}
    title={m.reglages()}
    aria-expanded={appli.reglagesOuverts}
    onclick={() => (appli.reglagesOuverts = true)}
  >
    <Icone nom="settings" taille={17} />
  </button>
</header>

<style>
  .entete {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px 18px;
    padding: 20px 32px;
  }

  .marque {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-right: 8px;
  }

  .nom {
    font-family: var(--font-titre);
    font-weight: 600;
    font-size: 26px;
  }

  .modes,
  .saisons {
    display: flex;
    gap: 2px;
    padding: 4px;
    border-radius: var(--pill);
    background: var(--white);
    border: 1px solid var(--line);
  }

  .mode,
  .saison {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 40px;
    padding: 0 18px;
    border: 0;
    border-radius: var(--pill);
    background: transparent;
    font-weight: 600;
    font-size: 15px;
  }

  .saison {
    padding: 0 14px;
    font-size: 14px;
    color: var(--muted);
    height: 36px;
  }

  .mode.actif,
  .saison.actif {
    background: var(--ink);
    color: var(--white);
  }

  .aujourdhui {
    min-height: 40px;
    font-size: 14px;
    padding: 0 16px;
  }

  .espace {
    flex: 1 1 20px;
  }

  .etat {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--muted);
  }

  .ok {
    display: inline-flex;
    color: var(--good);
  }

  .erreur {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: var(--pill);
    background: var(--crit-bg);
    color: var(--crit);
    font-weight: 700;
  }

  .reessayer {
    min-height: 36px;
    font-size: 13px;
    padding: 0 12px;
  }

  .rond {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    border: 1px solid var(--line);
    background: var(--white);
  }

  /* Phone (lot 2 layout, prepared now): two rows, smaller marks. */
  @media (max-width: 720px) {
    .entete {
      padding: 14px 16px;
      gap: 10px;
    }
    .nom {
      font-size: 22px;
    }
    .modes {
      order: 10;
      flex: 1 1 100%;
    }
    .mode {
      flex: 1;
      justify-content: center;
      padding: 0 10px;
    }
    .saisons {
      order: 11;
      flex: 1 1 auto;
      overflow-x: auto;
    }
    .saison {
      flex: 1;
      justify-content: center;
      padding: 0 8px;
    }
    .aujourdhui {
      order: 12;
    }
    .etat {
      font-size: 12px;
    }
  }
</style>
