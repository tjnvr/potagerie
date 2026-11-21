<script lang="ts">
  // « Keep here » / « Go back » (S01 F-PLN-2): a bed dropped closer than the
  // aisle to another one. The bed waits where it was dropped; Escape goes back.
  import { m } from "../paraglide/messages.js";
  import Icon from "../shell/Icon.svelte";

  let { aisle, answer }: { aisle: number; answer: (keep: boolean) => void } = $props();

  let keep: HTMLButtonElement;

  $effect(() => {
    keep.focus();
  });
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && answer(false)} />

<div class="bubble" role="alertdialog" aria-labelledby="aisle-bubble-text">
  <span class="icon"><Icon name="alert" size={18} /></span>
  <p id="aisle-bubble-text">{m.aisle_bubble_text({ aisle })}</p>
  <div class="actions">
    <button class="button" onclick={() => answer(false)}>{m.aisle_go_back()}</button>
    <button class="button alert" bind:this={keep} onclick={() => answer(true)}>{m.aisle_keep_here()}</button>
  </div>
</div>

<style>
  .bubble {
    position: fixed;
    left: 50%;
    bottom: 24px;
    transform: translateX(-50%);
    z-index: 21;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 12px;
    width: min(520px, calc(100vw - 32px));
    padding: 14px 16px;
    border-radius: 20px;
    background: var(--warn-bg);
    color: var(--ink);
    font-size: 14px;
    box-shadow: 0 8px 20px rgba(31, 58, 46, 0.25);
  }

  .icon {
    display: inline-flex;
    color: var(--warn);
  }

  p {
    flex: 1;
    min-width: 180px;
    margin: 0;
  }

  .actions {
    display: flex;
    gap: 8px;
    margin-left: auto;
  }
</style>
