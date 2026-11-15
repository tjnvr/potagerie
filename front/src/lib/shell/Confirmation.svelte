<script lang="ts">
  // Confirmation box (D02, E10): a modal <dialog>, never window.confirm.
  import type { Snippet } from "svelte";
  import { m } from "../paraglide/messages.js";
  import Icon from "./Icon.svelte";

  let {
    title,
    confirm,
    tone = "danger",
    onconfirm,
    oncancel,
    children,
  }: {
    title: string;
    confirm: string;
    tone?: "danger" | "alert" | "principal";
    onconfirm: () => void;
    oncancel: () => void;
    children: Snippet;
  } = $props();

  let box: HTMLDialogElement;

  $effect(() => {
    box.showModal();
    return () => box.close();
  });
</script>

<dialog bind:this={box} class="box" class:alert={tone === "alert"} oncancel={oncancel}>
  <div class="title">
    {#if tone !== "principal"}<span class="icon"><Icon name="alert" size={20} /></span>{/if}
    <h2>{title}</h2>
  </div>
  <div class="text">{@render children()}</div>
  <div class="actions">
    <button class="button" onclick={oncancel}>{m.cancel()}</button>
    <button
      class="button"
      class:danger={tone === "danger"}
      class:plein={tone === "danger"}
      class:alert={tone === "alert"}
      class:principal={tone === "principal"}
      onclick={onconfirm}>{confirm}</button
    >
  </div>
</dialog>

<style>
  .box {
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

  .box.alert {
    background: var(--warn-bg);
  }

  .box::backdrop {
    background: rgba(31, 58, 46, 0.35);
  }

  .title {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .title h2 {
    font-size: 19px;
  }

  .icon {
    display: inline-flex;
    color: var(--warn);
  }

  .text {
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
