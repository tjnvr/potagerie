<script lang="ts">
  // Plan tab: the atelier (E03). Top view, tray (Planche, Carré, Butte, Bac),
  // right panel with the selected planche (E06) or the balance of the plan.
  import { commands, type PlancheView, type TypePlanche, type YearView } from "../bindings";
  import Confirmation from "../coquille/Confirmation.svelte";
  import CreationTerrain from "../coquille/CreationTerrain.svelte";
  import Icone from "../coquille/Icone.svelte";
  import { appli } from "../etat.svelte";
  import { m } from "../paraglide/messages.js";
  import {
    MAX_PLANCHES,
    TAILLE_PAR_DEFAUT,
    bilan,
    enM2,
    enMetres,
    nomDeType,
    nomParDefaut,
    positionLibre,
  } from "../planches";
  import {
    ControleurGlisser,
    positionDepuisBac,
    type Geo,
    type Reponse,
    type TypePointeur,
  } from "./glisser";
  import PanneauPlanche from "./PanneauPlanche.svelte";
  import PlanVueDessus, { type Fantome } from "./PlanVueDessus.svelte";

  let { vue }: { vue: YearView } = $props();

  const TYPES: TypePlanche[] = ["planche", "carre", "butte", "bac"];
  const complet = $derived(vue.planches.length >= MAX_PLANCHES);
  const compte = $derived(vue.terrain ? bilan(vue.terrain, vue.planches) : null);

  let dessus: PlanVueDessus | undefined = $state();
  let selectionId = $state<string | null>(null);
  let fantome = $state<Fantome | null>(null);
  let portee = $state<TypePlanche | null>(null);
  let raccourcissement = $state<{ planche: PlancheView; nombre: number; repondre: (ok: boolean) => void } | null>(null);

  const selection = $derived(vue.planches.find((p) => p.id === selectionId) ?? null);

  // A planche that disappears (deleted, or another season) is no longer selected.
  $effect(() => {
    if (selectionId && !vue.planches.some((p) => p.id === selectionId)) selectionId = null;
  });

  async function poser(type: TypePlanche, x: number, y: number) {
    const nom = nomParDefaut(type, vue.planches);
    const avant = new Set(vue.planches.map((p) => p.id));
    const ok = await appli.modifier((annee) => commands.creerPlanche(annee, type, nom, x, y));
    if (ok && appli.vue) {
      // The new planche stays selected (F-PLN-6).
      selectionId = appli.vue.planches.find((p) => !avant.has(p.id))?.id ?? null;
    }
  }

  /** The tray without a pointer drag (keyboard, plain click): first free spot. */
  async function ajouter(type: TypePlanche) {
    if (!vue.terrain) return;
    const position = positionLibre(vue.terrain, vue.planches, type);
    if (!position) {
      appli.informer(m.plan_plus_de_place());
      return;
    }
    await poser(type, position.x, position.y);
  }

  /** A click on free grass: a planche of the chosen tray type, centred there. */
  async function poserIci(point: { x: number; y: number }) {
    if (!vue.terrain || complet) return;
    const type = portee ?? "planche";
    const taille = TAILLE_PAR_DEFAUT[type];
    const pos = positionDepuisBac(point, taille, vue.terrain.case_cm);
    if (pos.x < 0 || pos.y < 0) {
      appli.signaler({ code: "planche_hors_terrain", detail: "" });
      return;
    }
    await poser(type, pos.x, pos.y);
  }

  // Drag from the tray: the same long press on a finger, the same pointer logic.
  const bac = new ControleurGlisser({ surArme: () => {} });
  let typeBac: TypePlanche | null = null;

  function debutBac(e: PointerEvent, type: TypePlanche) {
    if (e.button !== 0 || complet || vue.read_only) return;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    typeBac = type;
    portee = type;
    bac.appui(e.pointerType as TypePointeur, { x: e.clientX, y: e.clientY });
  }

  function bougeBac(e: PointerEvent) {
    if (!typeBac) return;
    const mouvement = bac.deplacer({ x: e.clientX, y: e.clientY });
    if (mouvement === "abandon") {
      typeBac = null;
      portee = null;
      return;
    }
    if ((mouvement === "debut" || mouvement === "glisse") && vue.terrain) {
      const point = dessus?.versCm(e.clientX, e.clientY) ?? null;
      const taille = TAILLE_PAR_DEFAUT[typeBac];
      fantome = point ? { type: typeBac, ...taille, ...positionDepuisBac(point, taille, vue.terrain.case_cm) } : null;
    }
  }

  async function leveBac(e: PointerEvent) {
    const type = typeBac;
    if (!type) return;
    const bouton = e.currentTarget as HTMLElement;
    if (bouton.hasPointerCapture(e.pointerId)) bouton.releasePointerCapture(e.pointerId);
    const fin = bac.relacher();
    const depot = fantome;
    typeBac = null;
    portee = null;
    fantome = null;
    if (fin === "clic") await ajouter(type);
    else if (fin === "fin" && depot) {
      if (depot.x < 0 || depot.y < 0) appli.signaler({ code: "planche_hors_terrain", detail: "" });
      else await poser(type, depot.x, depot.y);
    }
  }

  function annuleBac() {
    bac.annuler();
    typeBac = null;
    portee = null;
    fantome = null;
  }

  /** A move or a resize: Rust judges. A shorter planche first asks which
   *  series would no longer fit (S03 §4) and waits for the user. */
  async function modifier(id: string, geo: Geo, nature: "deplacer" | "redimensionner"): Promise<Reponse> {
    const planche = vue.planches.find((p) => p.id === id);
    if (!planche) return "refus";
    if (nature === "redimensionner" && geo.longueur_cm < planche.longueur_cm) {
      const resultat = await commands.seriesDepassant(appli.annee, id, geo.longueur_cm);
      if (resultat.status === "ok" && resultat.data.length > 0) {
        const ok = await new Promise<boolean>((repondre) => {
          raccourcissement = { planche, nombre: resultat.data.length, repondre };
        });
        raccourcissement = null;
        if (!ok) return "annule";
      }
    }
    const ok =
      nature === "deplacer"
        ? await appli.modifier((annee) => commands.deplacerPlanche(annee, id, geo.x, geo.y))
        : await appli.modifier((annee) =>
            commands.modifierPlanche(annee, id, null, { planche_type: planche.planche_type, ...geo }),
          );
    return ok ? "ok" : "refus";
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

  <div class="atelier">
    <section class="scene">
      <p class="terrain">
        {m.plan_terrain({
          longueur: enMetres(vue.terrain.longueur_cm),
          largeur: enMetres(vue.terrain.largeur_cm),
          case: vue.terrain.case_cm,
        })}
      </p>
      <PlanVueDessus
        bind:this={dessus}
        terrain={vue.terrain}
        planches={vue.planches}
        {selectionId}
        lectureSeule={vue.read_only}
        {fantome}
        onselect={(id) => (selectionId = id)}
        onmodifier={modifier}
        onavantterrain={() => appli.signaler({ code: "planche_hors_terrain", detail: "" })}
        onposer={poserIci}
      />
      {#if vue.planches.length === 0}
        <p class="aide">{m.plan_vide()}</p>
      {/if}

      <p class="astuces">
        <span><b>{m.plan_aide_toucher()}</b> {m.plan_aide_toucher_texte()}</span>
        <span><b>{m.plan_aide_glisser()}</b> {m.plan_aide_glisser_texte()}</span>
        <span><b>{m.plan_aide_poignees()}</b> {m.plan_aide_poignees_texte()}</span>
      </p>

      {#if !vue.read_only}
        <div class="bac">
          <div class="bac-titre">
            <strong>{m.bac_titre()}</strong>
            <span class="compteur">{m.plan_compteur({ nombre: vue.planches.length, max: MAX_PLANCHES })}</span>
          </div>
          <div class="elements">
            {#each TYPES as type (type)}
              <button
                class="element"
                class:portee={portee === type}
                disabled={complet}
                onpointerdown={(e) => debutBac(e, type)}
                onpointermove={bougeBac}
                onpointerup={leveBac}
                onpointercancel={annuleBac}
                onclick={(e) => e.detail === 0 && ajouter(type)}
              >
                <span class="nom">{nomDeType(type)}</span>
                <span class="taille">
                  {m.bac_taille({
                    longueur: enMetres(TAILLE_PAR_DEFAUT[type].longueur_cm),
                    largeur: enMetres(TAILLE_PAR_DEFAUT[type].largeur_cm),
                  })}
                </span>
              </button>
            {/each}
          </div>
          {#if complet}<p class="aide">{m.plan_complet()}</p>{:else}<p class="aide">{m.bac_clic()}</p>{/if}
        </div>
      {/if}
    </section>

    <div class="lateral">
      {#if selection}
        <PanneauPlanche {vue} planche={selection} onfermer={() => (selectionId = null)} onmodifier={modifier} />
      {:else if compte}
        <aside class="bilan">
          <h2>{m.bilan_titre()}</h2>
          <dl>
            <div><dd>{m.surface_m2({ valeur: enM2(compte.total) })}</dd><dt>{m.bilan_terrain()}</dt></div>
            <div><dd>{m.surface_m2({ valeur: enM2(compte.cultivable) })}</dd><dt>{m.bilan_cultive()}</dt></div>
            <div><dd>{m.surface_m2({ valeur: enM2(compte.allees) })}</dd><dt>{m.bilan_allees()}</dt></div>
            <div><dd>{compte.nombre} / {MAX_PLANCHES}</dd><dt>{m.bilan_planches()}</dt></div>
          </dl>
        </aside>
      {/if}
    </div>
  </div>
{/if}

{#if raccourcissement}
  {@const demande = raccourcissement}
  <Confirmation
    titre={m.raccourcir_titre({ nom: demande.planche.nom })}
    confirmer={m.raccourcir_bouton()}
    ton="alerte"
    onconfirmer={() => demande.repondre(true)}
    onannuler={() => demande.repondre(false)}
  >
    <p>{m.raccourcir_texte({ nombre: demande.nombre })}</p>
  </Confirmation>
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

  .atelier {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 340px;
    gap: 24px;
    align-items: start;
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

  .astuces {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 18px;
    margin: 0;
    font-size: 13px;
    color: var(--muted);
  }

  .astuces b {
    padding: 2px 10px;
    border-radius: var(--pill);
    background: var(--white);
    color: var(--ink);
    font-size: 12px;
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
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 8px;
  }

  .element {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    min-height: 72px;
    border-radius: 16px;
    border: 1px solid transparent;
    background: var(--bg);
    touch-action: none;
    user-select: none;
    -webkit-user-select: none;
    cursor: grab;
  }

  .element .nom {
    font-weight: 700;
    font-size: 14px;
  }

  .element .taille {
    font-size: 12px;
    color: var(--muted);
  }

  .element.portee {
    border: 1px dashed var(--green);
    background: #e7f4ec;
  }

  .element:disabled {
    cursor: default;
  }

  .lateral {
    position: sticky;
    top: 12px;
  }

  .bilan {
    padding: 20px;
    border-radius: 28px;
    background: var(--white);
  }

  .bilan h2 {
    margin-bottom: 14px;
    font-size: 20px;
  }

  dl {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
    margin: 0;
  }

  dl div {
    display: flex;
    flex-direction: column-reverse;
  }

  dd {
    margin: 0;
    font-family: var(--font-titre);
    font-weight: 600;
    font-size: 22px;
    font-variant-numeric: tabular-nums;
  }

  dt {
    font-size: 12px;
    color: var(--muted);
  }

  @media (max-width: 1000px) {
    .atelier {
      grid-template-columns: minmax(0, 1fr);
    }

    .lateral {
      position: static;
    }
  }

  @media (max-width: 720px) {
    .scene {
      padding: 12px;
      border-radius: 24px;
    }

    .elements {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
</style>
