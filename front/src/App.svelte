<script lang="ts">
  // The shell: header, the two tabs without a router (S05 §4), settings and
  // the message line.
  import Atelier from "./lib/atelier/Atelier.svelte";
  import Entete from "./lib/coquille/Entete.svelte";
  import Icone from "./lib/coquille/Icone.svelte";
  import Potager from "./lib/diorama/Potager.svelte";
  import { appli } from "./lib/etat.svelte";
  import { m } from "./lib/paraglide/messages.js";
  import Reglages from "./lib/reglages/Reglages.svelte";

  $effect(() => {
    appli.charger();
  });

  $effect(() => {
    if (!appli.message) return;
    const delai = setTimeout(() => (appli.message = null), 6000);
    return () => clearTimeout(delai);
  });
</script>

<Entete />

<main class="page">
  {#if !appli.vue}
    <p class="chargement">{m.chargement()}</p>
  {:else if appli.onglet === "plan"}
    <Atelier vue={appli.vue} />
  {:else}
    <Potager vue={appli.vue} />
  {/if}
</main>

{#if appli.reglagesOuverts}
  <Reglages />
{/if}

{#if appli.message}
  <div class="annonce" class:erreur={appli.message.ton === "erreur"} role="alert">
    {#if appli.message.ton === "erreur"}<Icone nom="alert" />{/if}
    <span>{appli.message.texte}</span>
    <button aria-label={m.fermer()} onclick={() => (appli.message = null)}><Icone nom="close" taille={14} /></button>
  </div>
{/if}

<style>
  .page {
    padding: 0 32px 32px;
    max-width: 1440px;
  }

  .chargement {
    color: var(--muted);
  }

  .annonce {
    position: fixed;
    left: 50%;
    bottom: 24px;
    transform: translateX(-50%);
    z-index: 20;
    display: flex;
    align-items: center;
    gap: 10px;
    width: max-content;
    max-width: calc(100vw - 32px);
    padding: 12px 12px 12px 16px;
    border-radius: 16px;
    background: var(--ink);
    color: var(--white);
    font-size: 14px;
    box-shadow: 0 8px 20px rgba(31, 58, 46, 0.25);
  }

  .annonce.erreur {
    background: var(--crit-bg);
    color: #5a1e1a;
  }

  .annonce button {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: inherit;
  }

  @media (max-width: 720px) {
    .page {
      padding: 0 12px 24px;
    }
  }
</style>
