<script lang="ts">
  // Form to create or edit a note or a period (S01 F-VIS-5), on a bed or
  // on the whole garden. Standalone: it calls no command, it hands the
  // input to `onsave` and the host (the timeline, later) sends it to
  // `create_annotation` or `modify_annotation`.
  import type { AnnotationInput, AnnotationView, BedId, BedView } from "../bindings";
  import { m } from "../paraglide/messages.js";
  import { draftOf, draftEmpty, sendable, TEXT_MAX, versEntree, type Draft } from "./note";

  let {
    beds,
    annotation = null,
    date,
    bed_id = null,
    onsave,
    oncancel,
    ondelete,
  }: {
    /** The beds of the year, for “Applies to”. */
    beds: BedView[];
    /** The annotation to edit; none to create one. */
    annotation?: AnnotationView | null;
    /** Start day of a new annotation, `YYYY-MM-DD`. */
    date: string;
    /** Bed preselected for a new annotation; none: the whole garden. */
    bed_id?: BedId | null;
    onsave: (entree: AnnotationInput) => void;
    oncancel: () => void;
    /** Only offered when editing. */
    ondelete?: () => void;
  } = $props();

  const initial = (): Draft => (annotation ? draftOf(annotation) : draftEmpty(date, bed_id));
  let draft = $state<Draft>(initial());

  const period = $derived(draft.type === "period");
  const rest = $derived(TEXT_MAX - draft.text.length);
  const title = $derived(
    annotation
      ? period
        ? m.note_title_modify_period()
        : m.note_title_modify_note()
      : period
        ? m.note_title_create_period()
        : m.note_title_create_note(),
  );

  function envoyer(e: SubmitEvent) {
    e.preventDefault();
    if (sendable(draft)) onsave(versEntree(draft));
  }
</script>

<form class="note" onsubmit={envoyer} aria-labelledby="note-title">
  <h2 id="note-title">{title}</h2>

  <div class="field" role="group" aria-label={m.note_type()}>
    <div class="choice">
      <button
        type="button"
        class:active={!period}
        aria-pressed={!period}
        onclick={() => (draft.type = "note")}>{m.note_type_note()}</button
      >
      <button
        type="button"
        class:active={period}
        aria-pressed={period}
        onclick={() => (draft.type = "period")}>{m.note_type_period()}</button
      >
    </div>
  </div>

  <label class="field">
    <span>{m.note_text()}</span>
    <input type="text" maxlength={TEXT_MAX} required bind:value={draft.text} />
    <small>{m.note_text_rest({ rest })}</small>
  </label>

  <div class="dates">
    <label class="field">
      <span>{period ? m.note_start() : m.note_date()}</span>
      <input type="date" required bind:value={draft.start} />
    </label>
    {#if period}
      <label class="field">
        <span>{m.note_end()}</span>
        <input type="date" required min={draft.start} bind:value={draft.end} />
      </label>
    {/if}
  </div>

  <label class="field">
    <span>{m.note_target()}</span>
    <select bind:value={draft.bed_id}>
      <option value={null}>{m.note_target_garden()}</option>
      {#each beds as bed (bed.id)}
        <option value={bed.id}>{bed.name}</option>
      {/each}
    </select>
  </label>

  {#if annotation}
    <label class="case">
      <input type="checkbox" bind:checked={draft.hidden} />
      <span>{m.note_hidden()}</span>
    </label>
  {/if}

  <div class="actions">
    {#if annotation && ondelete}
      <button type="button" class="button danger" onclick={ondelete}>{m.delete()}</button>
    {/if}
    <span class="space"></span>
    <button type="button" class="button" onclick={oncancel}>{m.cancel()}</button>
    <button type="submit" class="button principal" disabled={!sendable(draft)}>{m.note_save()}</button>
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

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    font-weight: 600;
    color: var(--muted);
  }

  .field small {
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

  .dates .field {
    flex: 1 1 140px;
  }

  .choice {
    display: flex;
    gap: 4px;
    padding: 4px;
    border-radius: var(--pill);
    background: var(--bg);
  }

  .choice button {
    flex: 1;
    height: 38px;
    border: 0;
    border-radius: var(--pill);
    background: transparent;
    font-weight: 600;
    font-size: 14px;
  }

  .choice button.active {
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

  .space {
    flex: 1;
  }

  .button:disabled {
    opacity: 0.5;
  }
</style>
