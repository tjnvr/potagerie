<script lang="ts">
  // Confirmation box (D02, E10): a modal <dialog>, never window.confirm.
  import type { Snippet } from "svelte";
  import { m } from "../paraglide/messages.js";
  import Icone from "./Icone.svelte";

  let {
    titre,
    confirmer,
    ton = "danger",
    onconfirmer,
    onannuler,
    children,
  }: {
    titre: string;
    confirmer: string;
    ton?: "danger" | "alerte" | "principal";
    onconfirmer: () => void;
    onannuler: () => void;
    children: Snippet;
  } = $props();

  let boite: HTMLDialogElement;

  $effect(() => {
    boite.showModal();
    return () => boite.close();
  });
</script>

<dialog bind:this={boite} class="boite" class:alerte={ton === "alerte"} oncancel={onannuler}>
  <div class="titre">
    {#if ton !== "principal"}<span class="icone"><Icone nom="alert" taille={20} /></span>{/if}
    <h2>{titre}</h2>
  </div>
  <div class="texte">{@render children()}</div>
  <div class="actions">
    <button class="bouton" onclick={onannuler}>{m.annuler()}</button>
    <button
      class="bouton"
      class:danger={ton === "danger"}
      class:plein={ton === "danger"}
      class:alerte={ton === "alerte"}
      class:principal={ton === "principal"}
      onclick={onconfirmer}>{confirmer}</button
    >
  </div>
</dialog>

<style>
  .boite {
    width: min(440px, calc(100vw - 32px));
    border: 0;
    border-radius: 24px;
    padding: 24px;
    background: var(--white);
    color: var(--ink);
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .boite.alerte {
    background: var(--warn-bg);
  }

  .boite::backdrop {
    background: rgba(31, 58, 46, 0.35);
  }

  .titre {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .titre h2 {
    font-size: 19px;
  }

  .icone {
    display: inline-flex;
    color: var(--warn);
  }

  .texte {
    font-size: 14px;
    line-height: 1.5;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 10px;
  }
</style>
