<script lang="ts">
  // Right panel of the atelier for the selected planche (D01 E03, D02 E06):
  // name, type, dimensions, series of the year, history of the past years,
  // deletion with the warning for the series that straddle the next year.
  import { commands, type PlancheView, type TypePlanche, type YearView } from "../bindings";
  import Confirmation from "../coquille/Confirmation.svelte";
  import Icone from "../coquille/Icone.svelte";
  import { appli } from "../etat.svelte";
  import { get } from "svelte/store";
  import { m } from "../paraglide/messages.js";
  import { curseur } from "../temps";
  import { dateCourte, enM2, enMetres, nomCulture, nomDeType } from "../planches";
  import type { Geo, Reponse } from "./glisser";

  let {
    vue,
    planche,
    onfermer,
    onmodifier,
  }: {
    vue: YearView;
    planche: PlancheView;
    onfermer: () => void;
    onmodifier: (id: string, geo: Geo, nature: "deplacer" | "redimensionner") => Promise<Reponse>;
  } = $props();

  const TYPES: TypePlanche[] = ["planche", "carre", "butte", "bac"];
  const MODES = { abri: m.mode_abri, plants: m.mode_plants, pleine_terre: m.mode_pleine_terre };

  let nom = $state("");
  let longueur = $state("");
  let largeur = $state("");
  let historiqueOuvert = $state(false);
  let suppression = $state(false);

  // The fields follow the planche of the view (selection, drag, Rust's answer).
  $effect(() => {
    nom = planche.nom;
    longueur = String(planche.longueur_cm / 100);
    largeur = String(planche.largeur_cm / 100);
  });

  const series = $derived(vue.series.filter((s) => s.planche_id === planche.id));
  const historique = $derived(vue.historique.filter((s) => s.planche_id === planche.id));
  const notes = $derived(
    vue.annotations.filter((a) => a.planche_id === planche.id).sort((a, b) => a.date_debut.localeCompare(b.date_debut)),
  );
  const supprimees = $derived(vue.series.filter((s) => planche.series_supprimees.includes(s.id)));
  const geo = $derived<Geo>({
    x: planche.x,
    y: planche.y,
    longueur_cm: planche.longueur_cm,
    largeur_cm: planche.largeur_cm,
  });

  async function renommer() {
    if (nom === planche.nom) return;
    const ok = await appli.modifier((annee) => commands.modifierPlanche(annee, planche.id, nom, null));
    if (!ok) nom = planche.nom;
  }

  async function changerType(type: TypePlanche) {
    if (type === planche.planche_type) return;
    await appli.modifier((annee) =>
      commands.modifierPlanche(annee, planche.id, null, { planche_type: type, ...geo }),
    );
  }

  /** Dimensions typed in metres; Rust refuses a length that is not a whole
   *  number of cases, and the field goes back to the real value. */
  async function redimensionner() {
    const l = Math.round(Number(longueur.replace(",", ".")) * 100);
    const h = Math.round(Number(largeur.replace(",", ".")) * 100);
    if (!Number.isFinite(l) || !Number.isFinite(h) || l <= 0 || h <= 0) return;
    const reponse = await onmodifier(planche.id, { ...geo, longueur_cm: l, largeur_cm: h }, "redimensionner");
    if (reponse !== "ok") {
      longueur = String(planche.longueur_cm / 100);
      largeur = String(planche.largeur_cm / 100);
    }
  }

  async function supprimer() {
    suppression = false;
    const ok = await appli.modifier((annee) => commands.supprimerPlanche(annee, planche.id));
    if (ok) onfermer();
  }
</script>

<aside class="panneau" aria-label={m.panneau_selection()}>
  <div class="entete">
    <div class="titre">
      <p class="surtitre">{m.panneau_selection()}</p>
      <h2>{planche.nom}</h2>
    </div>
    <button class="rond" aria-label={m.panneau_fermer()} onclick={onfermer}><Icone nom="close" taille={16} /></button>
  </div>

  <div class="puces">
    <span class="puce">
      {m.panneau_dimensions({ longueur: enMetres(planche.longueur_cm), largeur: enMetres(planche.largeur_cm) })}
    </span>
    <span class="puce">{m.panneau_surface({ valeur: enM2(planche.longueur_cm * planche.largeur_cm) })}</span>
  </div>

  {#if vue.read_only}
    <p class="aide">{m.panneau_lecture_seule()}</p>
  {:else}
    <label class="champ">
      <span>{m.champ_nom()}</span>
      <input type="text" maxlength="30" bind:value={nom} onchange={renommer} />
    </label>

    <fieldset class="champ">
      <legend>{m.champ_type()}</legend>
      <div class="choix">
        {#each TYPES as type (type)}
          <label class:actif={planche.planche_type === type}>
            <input
              type="radio"
              name="type-planche"
              checked={planche.planche_type === type}
              onchange={() => changerType(type)}
            />
            {nomDeType(type)}
          </label>
        {/each}
      </div>
    </fieldset>

    <div class="dimensions">
      <label class="champ">
        <span>{m.champ_longueur()}</span>
        <input
          type="number"
          inputmode="decimal"
          min={vue.terrain ? vue.terrain.case_cm / 100 : 0.25}
          step={vue.terrain ? vue.terrain.case_cm / 100 : 0.25}
          bind:value={longueur}
          onchange={redimensionner}
        />
      </label>
      <label class="champ">
        <span>{m.champ_largeur()}</span>
        <input type="number" inputmode="decimal" min="0.1" step="0.01" bind:value={largeur} onchange={redimensionner} />
      </label>
    </div>
    {#if vue.terrain}
      <p class="aide">{m.champ_longueur_aide({ case: vue.terrain.case_cm })}</p>
    {/if}
  {/if}

  <h3 class="section">{m.panneau_series({ annee: vue.year })}</h3>
  {#each series as serie (serie.id)}
    <div class="serie">
      <strong>{nomCulture(serie.culture_id)}</strong>
      <span class="badge">{MODES[serie.mode]()}</span>
      <p>
        {m.panneau_serie_troncons({
          debut: serie.troncon_debut,
          fin: serie.troncon_fin,
          pieds: serie.rangs * serie.pieds_par_rang,
          rangs: serie.rangs,
          par_rang: serie.pieds_par_rang,
        })}
      </p>
      <p>{m.panneau_serie_dates({ debut: dateCourte(serie.date, vue.year), fin: dateCourte(serie.end, vue.year) })}</p>
      <button class="lien" onclick={() => (appli.serieOuverte = { type: "edition", serie })}>{m.serie_modifier()}</button>
    </div>
  {:else}
    <p class="aide">{m.panneau_aucune_serie()}</p>
  {/each}
  <button class="bouton principal" onclick={() => (appli.serieOuverte = { type: "nouvelle", plancheId: planche.id })}>
    <Icone nom="plus" taille={16} />{m.serie_planter_ici()}
  </button>

  <h3 class="section">{m.note_planche_titre()}</h3>
  {#each notes as note (note.id)}
    <div class="serie">
      <strong>{note.texte}</strong>
      {#if note.masquee}<span class="badge">{m.note_planche_masquee()}</span>{/if}
      <p>
        {note.date_fin
          ? m.note_aria_periode({ debut: dateCourte(note.date_debut, vue.year), fin: dateCourte(note.date_fin, vue.year) })
          : dateCourte(note.date_debut, vue.year)}
      </p>
      <button class="lien" onclick={() => (appli.noteOuverte = { type: "edition", annotation: note })}>{m.serie_modifier()}</button>
    </div>
  {:else}
    <p class="aide">{m.note_planche_aucune()}</p>
  {/each}
  <button class="bouton" onclick={() => (appli.noteOuverte = { type: "nouvelle", date: get(curseur), plancheId: planche.id })}>
    <Icone nom="plus" taille={16} />{m.note_planche_ajouter()}
  </button>

  <button class="section repli" aria-expanded={historiqueOuvert} onclick={() => (historiqueOuvert = !historiqueOuvert)}>
    {m.panneau_historique()}
  </button>
  {#if historiqueOuvert}
    {#each historique as serie (serie.id)}
      <div class="serie">
        <strong>{nomCulture(serie.culture_id)}</strong>
        <p>
          {m.panneau_serie_dates({ debut: dateCourte(serie.date), fin: dateCourte(serie.end) })} ·
          ≈ {serie.rangs * serie.pieds_par_rang}
        </p>
      </div>
    {:else}
      <p class="aide">{m.panneau_historique_vide()}</p>
    {/each}
  {/if}

  {#if !vue.read_only}
    <p class="aide">{m.panneau_suivent()}</p>
    <button class="bouton danger" onclick={() => (suppression = true)}>
      <Icone nom="trash" taille={16} />{m.supprimer()}
    </button>
  {/if}
</aside>

{#if suppression}
  <Confirmation
    titre={m.supprimer_planche_titre({ nom: planche.nom })}
    confirmer={m.supprimer_planche_bouton()}
    onconfirmer={supprimer}
    onannuler={() => (suppression = false)}
  >
    <p>{m.supprimer_planche_texte({ annee: vue.year })}</p>
    {#if planche.series_supprimees.length > 0}
      <div class="avertissement">
        <p>{m.supprimer_planche_series({ nombre: planche.series_supprimees.length, annee: vue.year })}</p>
        <ul>
          {#each supprimees as serie (serie.id)}
            <li>
              {m.supprimer_planche_serie({
                culture: nomCulture(serie.culture_id),
                debut: dateCourte(serie.date),
                fin: dateCourte(serie.end),
              })}
            </li>
          {/each}
        </ul>
        <p>{m.supprimer_planche_gardees({ annee: vue.year })}</p>
      </div>
    {/if}
  </Confirmation>
{/if}

<style>
  .panneau {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 20px;
    border-radius: 28px;
    background: var(--white);
  }

  .entete {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }

  .surtitre {
    margin: 0 0 4px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }

  h2 {
    font-size: 24px;
    overflow-wrap: anywhere;
  }

  .rond {
    flex: none;
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border: 0;
    border-radius: 50%;
    background: var(--bg);
  }

  .puces {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .puce {
    padding: 4px 12px;
    border-radius: var(--pill);
    background: var(--bg);
    font-size: 13px;
    font-weight: 700;
  }

  .champ {
    display: block;
    border: 0;
    margin: 0;
    padding: 0;
  }

  .champ > span,
  legend {
    display: block;
    margin-bottom: 6px;
    font-size: 12px;
    font-weight: 700;
    color: var(--muted);
  }

  input[type="text"],
  input[type="number"] {
    width: 100%;
    height: 44px;
    padding: 0 14px;
    border: 0;
    border-radius: 14px;
    background: var(--bg);
    font-weight: 600;
  }

  .choix {
    display: flex;
    gap: 2px;
    padding: 4px;
    border-radius: var(--pill);
    background: var(--bg);
  }

  .choix label {
    flex: 1;
    padding: 8px 0;
    text-align: center;
    border-radius: var(--pill);
    font-size: 13px;
    font-weight: 600;
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

  .dimensions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .aide {
    margin: 0;
    font-size: 13px;
    color: var(--muted);
  }

  .section {
    margin: 8px 0 0;
    font-family: var(--font);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .repli {
    padding: 0;
    border: 0;
    background: none;
    text-align: left;
  }

  .repli::after {
    content: " ▾";
  }

  .repli[aria-expanded="true"]::after {
    content: " ▴";
  }

  .serie {
    padding: 12px 14px;
    border-radius: 16px;
    background: var(--bg);
    font-size: 13px;
  }

  .lien {
    padding: 0;
    border: 0;
    background: none;
    color: var(--green);
    font-size: 13px;
    font-weight: 600;
    text-decoration: underline;
  }

  .serie p {
    margin: 2px 0 0;
    color: var(--muted);
  }

  .badge {
    margin-left: 6px;
    padding: 2px 8px;
    border-radius: var(--pill);
    background: var(--warn-bg);
    font-size: 11px;
    font-weight: 700;
    color: var(--accent-ink);
  }

  .avertissement {
    padding: 12px 14px;
    border-radius: 14px;
    background: var(--warn-bg);
  }

  .avertissement p {
    margin: 0 0 6px;
  }

  .avertissement ul {
    margin: 0 0 6px;
    padding-left: 18px;
  }
</style>
