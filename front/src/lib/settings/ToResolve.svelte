<script lang="ts">
  // The « to resolve » list kept after an import (S03 §13): Rust computes it
  // from the plan of the device after each change, so it empties by itself
  // when each case is settled with the usual actions. Nothing is corrected here.
  import Icon from "../shell/Icon.svelte";
  import { app } from "../state.svelte";
  import { m } from "../paraglide/messages.js";
  import { textIncoherence } from "./incoherences";

  let open = $state(false);
</script>

<section class="bandeau" aria-labelledby="resolve-title">
  <div class="tete">
    <Icon name="alert" size={18} />
    <strong id="resolve-title">{m.resolve_title({ count: app.toResolve.length })}</strong>
    <button class="link" aria-expanded={open} onclick={() => (open = !open)}>
      {open ? m.resolve_hide() : m.resolve_show()}
    </button>
  </div>
  {#if open}
    <p>{m.resolve_help()}</p>
    <ul>
      {#each app.toResolve as incoherence, i (i)}
        <li>{textIncoherence(incoherence)}</li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .bandeau {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 16px;
    padding: 12px 18px;
    border-radius: 20px;
    background: var(--warn-bg);
    color: var(--ink);
  }

  .tete {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .tete strong {
    flex: 1;
  }

  p {
    margin: 0;
    color: var(--muted);
    font-size: 14px;
  }

  ul {
    margin: 0;
    padding-left: 20px;
    font-size: 14px;
  }

  .link {
    border: 0;
    background: none;
    color: var(--green);
    font: inherit;
    font-weight: 600;
    text-decoration: underline;
  }
</style>
