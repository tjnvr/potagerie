<script lang="ts">
  // Host of the note form (F-VIS-5): a dialog over the screen that sends the
  // input to `creer_annotation`, `modifier_annotation` or `supprimer_annotation`.
  // Rust checks the rules (S03 §16); a refusal stays in the dialog, worded.
  import { commands, type AnnotationInput } from "../bindings";
  import { traduireErreur } from "../erreurs";
  import { appli, type OuvertureNote } from "../etat.svelte";
  import NoteForm from "./NoteForm.svelte";

  let { ouverture }: { ouverture: OuvertureNote } = $props();

  let raison = $state("");
  let racine: HTMLElement;

  $effect(() => {
    racine.focus();
  });

  function fermer() {
    appli.noteOuverte = null;
  }

  async function enregistrer(entree: AnnotationInput) {
    raison = "";
    const ok = await appli.modifier((annee) =>
      ouverture.type === "edition"
        ? commands.modifierAnnotation(annee, ouverture.annotation.id, entree)
        : commands.creerAnnotation(annee, entree),
    );
    if (ok) fermer();
    else raison = appli.dernierRefus ? traduireErreur(appli.dernierRefus) : "";
  }

  async function supprimer() {
    if (ouverture.type !== "edition") return;
    const id = ouverture.annotation.id;
    raison = "";
    if (await appli.modifier((annee) => commands.supprimerAnnotation(annee, id))) fermer();
    else raison = appli.dernierRefus ? traduireErreur(appli.dernierRefus) : "";
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && fermer()} />

<div class="voile" role="presentation" onpointerdown={(e) => e.target === e.currentTarget && fermer()}>
  <div class="boite" role="dialog" aria-modal="true" aria-labelledby="note-titre" tabindex="-1" bind:this={racine}>
    <NoteForm
      planches={appli.vue?.planches ?? []}
      annotation={ouverture.type === "edition" ? ouverture.annotation : null}
      date={ouverture.type === "nouvelle" ? ouverture.date : ouverture.annotation.date_debut}
      planche_id={ouverture.type === "nouvelle" ? ouverture.plancheId : null}
      onenregistrer={enregistrer}
      onannuler={fermer}
      onsupprimer={ouverture.type === "edition" ? supprimer : undefined}
    />
    {#if raison}
      <p class="refus" role="alert">{raison}</p>
    {/if}
  </div>
</div>

<style>
  .voile {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: grid;
    place-items: center;
    padding: 16px;
    background: rgb(31 58 46 / 0.35);
  }

  .boite {
    width: min(480px, 100%);
    max-height: 100%;
    overflow-y: auto;
    border-radius: 24px;
    outline: none;
  }

  .refus {
    margin: 8px 20px 0;
    padding: 10px 14px;
    border-radius: 14px;
    background: var(--crit-bg, #fbe4e2);
    color: var(--crit, #c8443c);
    font-size: 14px;
  }
</style>
