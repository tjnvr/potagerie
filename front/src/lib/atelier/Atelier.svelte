<script lang="ts">
  // Plan tab: the atelier (E03), reduced for now to the top view and a tray
  // that adds a planche through `creer_planche`.
  import { commands, type TypePlanche, type YearView } from "../bindings";
  import CreationTerrain from "../coquille/CreationTerrain.svelte";
  import Icone from "../coquille/Icone.svelte";
  import { appli } from "../etat.svelte";
  import { m } from "../paraglide/messages.js";
  import { MAX_PLANCHES, nomDeType, nomParDefaut, positionLibre } from "../planches";
  import PlanVueDessus from "./PlanVueDessus.svelte";

  let { vue }: { vue: YearView } = $props();

  const TYPES: TypePlanche[] = ["planche", "carre", "butte", "bac"];
  const complet = $derived(vue.planches.length >= MAX_PLANCHES);
  const enMetres = (cm: number) => (cm / 100).toLocaleString("fr-FR");

  async function ajouter(type: TypePlanche) {
    if (!vue.terrain) return;
    const position = positionLibre(vue.terrain, vue.planches, type);
    if (!position) {
      appli.informer(m.plan_plus_de_place());
      return;
    }
    const nom = nomParDefaut(type, vue.planches);
    await appli.modifier((annee) => commands.creerPlanche(annee, type, nom, position.x, position.y));
  }
</script>

{#if !vue.terrain}
  <CreationTerrain />
{:else}
  <div class="bandeau">
    <span class="annee">
      {#if vue.read_only}
        <strong>{m.plan_consultation({ annee: vue.year })}</strong>
      {:else}
        <strong>{m.plan_titre({ annee: vue.year })}</strong>
        <span class="portee">· {m.plan_portee({ annee: vue.year })}</span>
      {/if}
    </span>
    <span class="espace"></span>
    <button class="bouton principal" onclick={() => (appli.onglet = "potager")}>
      <Icone nom="check" />{m.voir_potager()}
    </button>
  </div>

  <section class="scene">
    <p class="terrain">
      {m.plan_terrain({
        longueur: enMetres(vue.terrain.longueur_cm),
        largeur: enMetres(vue.terrain.largeur_cm),
        case: vue.terrain.case_cm,
      })}
    </p>
    <PlanVueDessus terrain={vue.terrain} planches={vue.planches} />
    {#if vue.planches.length === 0}
      <p class="aide">{m.plan_vide()}</p>
    {/if}

    {#if !vue.read_only}
      <div class="bac">
        <div class="bac-titre">
          <strong>{m.plan_ajouter()}</strong>
          <span class="compteur">{m.plan_compteur({ nombre: vue.planches.length, max: MAX_PLANCHES })}</span>
        </div>
        <div class="elements">
          {#each TYPES as type (type)}
            <button class="element" disabled={complet} onclick={() => ajouter(type)}>
              <Icone nom="plus" taille={16} />{nomDeType(type)}
            </button>
          {/each}
        </div>
        {#if complet}<p class="aide">{m.plan_complet()}</p>{/if}
      </div>
    {/if}
  </section>
{/if}

<style>
  .bandeau {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .annee {
    display: inline-flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 8px 14px;
    border-radius: var(--pill);
    background: var(--white);
    border: 1px solid var(--line);
    font-size: 14px;
  }

  .portee {
    color: var(--muted);
  }

  .espace {
    flex: 1;
  }

  .scene {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 20px;
    border-radius: 32px;
    background: var(--card);
  }

  .terrain {
    margin: 0;
    align-self: flex-end;
    font-size: 13px;
    color: var(--muted);
  }

  .aide {
    margin: 0;
    font-size: 13px;
    color: var(--muted);
  }

  .bac {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px;
    border-radius: 24px;
    background: var(--white);
  }

  .bac-titre {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 8px;
  }

  .compteur {
    font-size: 13px;
    font-weight: 700;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  .elements {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: 8px;
  }

  .element {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-height: 48px;
    border-radius: 16px;
    border: 1px dashed var(--line);
    background: var(--bg);
    font-weight: 600;
  }

  @media (max-width: 720px) {
    .scene {
      padding: 12px;
      border-radius: 24px;
    }
  }
</style>
