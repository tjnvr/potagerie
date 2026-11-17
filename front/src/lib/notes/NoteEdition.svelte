<script lang="ts">
  // Host of the note form (F-VIS-5): a dialog over the screen that sends the
  // input to `create_annotation`, `modify_annotation` or `delete_annotation`.
  // Rust checks the rules (S03 §16); a refusal stays in the dialog, worded.
  import { commands, type AnnotationInput } from "../bindings";
  import { translateError } from "../errors";
  import { app, type OpeningNote } from "../state.svelte";
  import NoteForm from "./NoteForm.svelte";

  let { opening }: { opening: OpeningNote } = $props();

  let reason = $state("");
  import { modal } from "../shell/modal";

  function close() {
    app.noteOpen = null;
  }

  async function save(entree: AnnotationInput) {
    reason = "";
    const ok = await app.modify((year) =>
      opening.type === "edition"
        ? commands.modifyAnnotation(year, opening.annotation.id, entree)
        : commands.createAnnotation(year, entree),
    );
    if (ok) close();
    else reason = app.lastRefusal ? translateError(app.lastRefusal) : "";
  }

  async function remove() {
    if (opening.type !== "edition") return;
    const id = opening.annotation.id;
    reason = "";
    if (await app.modify((year) => commands.deleteAnnotation(year, id))) close();
    else reason = app.lastRefusal ? translateError(app.lastRefusal) : "";
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && close()} />

<div class="voile" role="presentation" onpointerdown={(e) => e.target === e.currentTarget && close()}>
  <div class="box" role="dialog" aria-modal="true" aria-labelledby="note-title" tabindex="-1" use:modal>
    <NoteForm
      beds={app.view?.beds ?? []}
      annotation={opening.type === "edition" ? opening.annotation : null}
      date={opening.type === "new" ? opening.date : opening.annotation.date_start}
      bed_id={opening.type === "new" ? opening.bedId : null}
      onsave={save}
      oncancel={close}
      ondelete={opening.type === "edition" ? remove : undefined}
    />
    {#if reason}
      <p class="refusal" role="alert">{reason}</p>
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

  .box {
    width: min(480px, 100%);
    max-height: 100%;
    overflow-y: auto;
    border-radius: 24px;
    outline: none;
  }

  .refusal {
    margin: 8px 20px 0;
    padding: 10px 14px;
    border-radius: 14px;
    background: var(--crit-bg, #fbe4e2);
    color: var(--crit, #c8443c);
    font-size: 14px;
  }
</style>
