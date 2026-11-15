<script lang="ts">
  // Form to create or edit a note or a period (S01 F-VIS-5), on a planche or
  // on the whole potager. Standalone: it calls no command, it hands the
  // input to `onenregistrer` and the host (the timeline, later) sends it to
  // `creer_annotation` or `modifier_annotation`.
  import type { AnnotationInput, AnnotationView, PlancheId, PlancheView } from "../bindings";
  import { m } from "../paraglide/messages.js";
  import { brouillonDe, brouillonVide, envoyable, TEXTE_MAX, versEntree, type Brouillon } from "./note";

  let {
    planches,
    annotation = null,
    date,
    planche_id = null,
    onenregistrer,
    onannuler,
    onsupprimer,
  }: {
    /** The planches of the year, for « Concerne ». */
    planches: PlancheView[];
    /** The annotation to edit; none to create one. */
    annotation?: AnnotationView | null;
    /** Start day of a new annotation, `YYYY-MM-DD`. */
    date: string;
    /** Planche preselected for a new annotation; none: the whole potager. */
    planche_id?: PlancheId | null;
    onenregistrer: (entree: AnnotationInput) => void;
    onannuler: () => void;
    /** Only offered when editing. */
    onsupprimer?: () => void;
  } = $props();

  const initial = (): Brouillon => (annotation ? brouillonDe(annotation) : brouillonVide(date, planche_id));
  let brouillon = $state<Brouillon>(initial());

  const periode = $derived(brouillon.type === "period");
  const reste = $derived(TEXTE_MAX - brouillon.texte.length);
  const titre = $derived(
    annotation
      ? periode
        ? m.note_titre_modifier_periode()
        : m.note_titre_modifier_note()
      : periode
        ? m.note_titre_creer_periode()
        : m.note_titre_creer_note(),
  );

  function envoyer(e: SubmitEvent) {
    e.preventDefault();
    if (envoyable(brouillon)) onenregistrer(versEntree(brouillon));
  }
</script>

<form class="note" onsubmit={envoyer} aria-labelledby="note-titre">
  <h2 id="note-titre">{titre}</h2>

  <div class="champ" role="group" aria-label={m.note_type()}>
    <div class="choix">
      <button
        type="button"
        class:actif={!periode}
        aria-pressed={!periode}
        onclick={() => (brouillon.type = "note")}>{m.note_type_note()}</button
      >
      <button
        type="button"
        class:actif={periode}
        aria-pressed={periode}
        onclick={() => (brouillon.type = "period")}>{m.note_type_periode()}</button
      >
    </div>
  </div>

  <label class="champ">
    <span>{m.note_texte()}</span>
    <input type="text" maxlength={TEXTE_MAX} required bind:value={brouillon.texte} />
    <small>{m.note_texte_reste({ reste })}</small>
  </label>

  <div class="dates">
    <label class="champ">
      <span>{periode ? m.note_debut() : m.note_date()}</span>
      <input type="date" required bind:value={brouillon.debut} />
    </label>
    {#if periode}
      <label class="champ">
        <span>{m.note_fin()}</span>
        <input type="date" required min={brouillon.debut} bind:value={brouillon.fin} />
      </label>
    {/if}
  </div>

  <label class="champ">
    <span>{m.note_cible()}</span>
    <select bind:value={brouillon.planche_id}>
      <option value={null}>{m.note_cible_potager()}</option>
      {#each planches as planche (planche.id)}
        <option value={planche.id}>{planche.nom}</option>
      {/each}
    </select>
  </label>

  {#if annotation}
    <label class="case">
      <input type="checkbox" bind:checked={brouillon.masquee} />
      <span>{m.note_masquee()}</span>
    </label>
  {/if}

  <div class="actions">
    {#if annotation && onsupprimer}
      <button type="button" class="bouton danger" onclick={onsupprimer}>{m.supprimer()}</button>
    {/if}
    <span class="espace"></span>
    <button type="button" class="bouton" onclick={onannuler}>{m.annuler()}</button>
    <button type="submit" class="bouton principal" disabled={!envoyable(brouillon)}>{m.note_enregistrer()}</button>
  </div>
</form>

<style>
  .note {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 20px;
    border-radius: 24px;
    background: var(--white);
  }

  h2 {
    font-size: 19px;
  }

  .champ {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    font-weight: 600;
    color: var(--muted);
  }

  .champ small {
    font-weight: 500;
  }

  input[type="text"],
  input[type="date"],
  select {
    min-height: 44px;
    padding: 0 14px;
    border: 1px solid var(--line);
    border-radius: 14px;
    background: var(--white);
    color: var(--ink);
    font: inherit;
    font-size: 15px;
  }

  .dates {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }

  .dates .champ {
    flex: 1 1 140px;
  }

  .choix {
    display: flex;
    gap: 4px;
    padding: 4px;
    border-radius: var(--pill);
    background: var(--bg);
  }

  .choix button {
    flex: 1;
    height: 38px;
    border: 0;
    border-radius: var(--pill);
    background: transparent;
    font-weight: 600;
    font-size: 14px;
  }

  .choix button.actif {
    background: var(--ink);
    color: var(--white);
  }

  .case {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 14px;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
  }

  .espace {
    flex: 1;
  }

  .bouton:disabled {
    opacity: 0.5;
  }
</style>
