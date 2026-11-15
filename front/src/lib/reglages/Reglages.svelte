<script lang="ts">
  // Settings (E10, D02): allée width, export and import of the potager
  // (F-CPT-3), erase the data (F-CPT-4). A side panel on a computer, the full
  // screen on a phone.
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { commands, type ImportSummaryView } from "../bindings";
  import Confirmation from "../coquille/Confirmation.svelte";
  import Icone from "../coquille/Icone.svelte";
  import { appeler, appli, dateLongue } from "../etat.svelte";
  import { m } from "../paraglide/messages.js";

  const ALLEES = [30, 50, 80];
  const FILTRES = [{ name: m.filtre_potager(), extensions: ["json"] }];

  type Import = { chemin: string; resume: ImportSummaryView };
  let importEnAttente = $state<Import | null>(null);
  let effacement = $state<0 | 1 | 2>(0);
  let panneau: HTMLElement;

  const terrain = $derived(appli.vue?.terrain ?? null);

  $effect(() => {
    panneau.focus();
  });

  function fermer() {
    appli.reglagesOuverts = false;
  }

  async function changerAllee(allee_cm: number) {
    const t = terrain;
    if (!t) return;
    await appli.modifier((annee) =>
      commands.modifierTerrain(annee, { longueur_cm: t.longueur_cm, largeur_cm: t.largeur_cm, allee_cm }),
    );
  }

  async function exporter() {
    const aujourdhui = new Date().toISOString().slice(0, 10);
    const chemin = await save({ defaultPath: m.exporter_nom_fichier({ date: aujourdhui }), filters: FILTRES });
    if (!chemin) return;
    const resultat = await appeler(() => commands.exporterPotager(chemin));
    if (resultat.status === "ok") appli.informer(m.exporter_ok());
    else appli.signaler(resultat.error);
  }

  async function choisirImport() {
    const chemin = await open({ multiple: false, directory: false, filters: FILTRES });
    if (typeof chemin !== "string") return;
    const resultat = await appeler(() => commands.lireImport(chemin));
    if (resultat.status === "ok") importEnAttente = { chemin, resume: resultat.data };
    else appli.signaler(resultat.error);
  }

  async function importer() {
    const chemin = importEnAttente?.chemin;
    importEnAttente = null;
    if (!chemin) return;
    if (await appli.modifier((annee) => commands.importerPotager(annee, chemin))) {
      appli.informer(m.importer_ok());
    }
  }

  async function effacer() {
    effacement = 0;
    const resultat = await appeler(() => commands.effacerDonnees());
    if (resultat.status === "ok") {
      await appli.charger();
      appli.informer(m.effacer_ok());
    } else {
      appli.signaler(resultat.error);
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="voile" onclick={fermer}></div>
<div
  class="panneau"
  role="dialog"
  aria-modal="true"
  aria-labelledby="reglages-titre"
  tabindex="-1"
  bind:this={panneau}
  onkeydown={(e) => e.key === "Escape" && fermer()}
>
  <div class="tete">
    <h1 id="reglages-titre">{m.reglages()}</h1>
    <button class="rond" aria-label={m.fermer()} onclick={fermer}><Icone nom="close" taille={16} /></button>
  </div>

  {#if terrain}
    <section>
      <h2 class="petit">{m.reglages_allee()}</h2>
      <div class="choix" role="group" aria-label={m.reglages_allee()}>
        {#each ALLEES as valeur (valeur)}
          <button
            class:actif={terrain.allee_cm === valeur}
            aria-pressed={terrain.allee_cm === valeur}
            onclick={() => changerAllee(valeur)}>{m.cm({ valeur })}</button
          >
        {/each}
      </div>
    </section>
  {/if}

  <section>
    <h2 class="petit majuscules">{m.reglages_donnees()}</h2>
    <div class="ligne">
      <span class="pastille"><Icone nom="export" taille={16} /></span>
      <div class="libelle"><strong>{m.exporter_titre()}</strong><span>{m.exporter_aide()}</span></div>
      <button class="bouton" disabled={!terrain} onclick={exporter}>
        <Icone nom="export" taille={16} />{m.exporter()}
      </button>
    </div>
    <div class="ligne">
      <span class="pastille"><Icone nom="upload" taille={16} /></span>
      <div class="libelle"><strong>{m.importer_titre()}</strong><span>{m.importer_aide()}</span></div>
      <button class="bouton" onclick={choisirImport}><Icone nom="upload" taille={16} />{m.importer()}</button>
    </div>
    <div class="ligne">
      <span class="pastille danger"><Icone nom="trash" taille={16} /></span>
      <div class="libelle">
        <strong class="rouge">{m.effacer_titre()}</strong><span>{m.effacer_aide()}</span>
      </div>
      <button class="bouton danger" onclick={() => (effacement = 1)}>
        <Icone nom="trash" taille={16} />{m.effacer()}
      </button>
    </div>
  </section>
</div>

{#if importEnAttente}
  {@const resume = importEnAttente.resume}
  <Confirmation
    titre={resume.older ? m.import_plus_ancien() : m.import_confirmer()}
    confirmer={resume.older ? m.remplacer_quand_meme() : m.remplacer()}
    ton={resume.older ? "alerte" : "principal"}
    onconfirmer={importer}
    onannuler={() => (importEnAttente = null)}
  >
    <p>
      {m.import_resume({
        date: dateLongue(resume.updated_at),
        planches: resume.planche_count,
        series: resume.serie_count,
      })}
    </p>
    {#if resume.older && appli.vue?.updated_at}
      <p>{m.import_local_plus_recent({ date: dateLongue(appli.vue.updated_at) })}</p>
    {/if}
    {#if resume.replaces}<p>{m.import_remplace_tout()}</p>{/if}
  </Confirmation>
{/if}

{#if effacement === 1}
  <Confirmation
    titre={m.effacer_confirmer_1()}
    confirmer={m.continuer()}
    onconfirmer={() => (effacement = 2)}
    onannuler={() => (effacement = 0)}
  >
    <p>{m.effacer_confirmer_1_texte()}</p>
  </Confirmation>
{:else if effacement === 2}
  <Confirmation
    titre={m.effacer_confirmer_2()}
    confirmer={m.effacer_definitivement()}
    onconfirmer={effacer}
    onannuler={() => (effacement = 0)}
  >
    <p>{m.effacer_confirmer_2_texte()}</p>
  </Confirmation>
{/if}

<style>
  .voile {
    position: fixed;
    inset: 0;
    background: rgba(238, 242, 230, 0.6);
    z-index: 10;
  }

  .panneau {
    position: fixed;
    top: 16px;
    right: 16px;
    bottom: 16px;
    width: min(500px, calc(100vw - 32px));
    overflow-y: auto;
    z-index: 11;
    display: flex;
    flex-direction: column;
    gap: 22px;
    padding: 24px;
    border-radius: 28px;
    background: var(--white);
    box-shadow: 0 12px 40px rgba(31, 58, 46, 0.15);
    outline: none;
  }

  .tete {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .tete h1 {
    font-size: 22px;
  }

  .rond {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border: 0;
    border-radius: 50%;
    background: var(--bg);
  }

  section {
    display: flex;
    flex-direction: column;
  }

  .petit {
    font-family: var(--font);
    font-size: 12px;
    font-weight: 700;
    color: var(--muted);
    margin-bottom: 8px;
  }

  .majuscules {
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--line);
    margin-bottom: 0;
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

  .ligne {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px 14px;
    padding: 14px 0;
    border-bottom: 1px solid var(--line);
  }

  .ligne:last-child {
    border-bottom: 0;
  }

  .pastille {
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    border-radius: 12px;
    background: var(--bg);
  }

  .pastille.danger {
    background: var(--crit-bg);
    color: var(--crit);
  }

  .libelle {
    flex: 1 1 180px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 15px;
  }

  .libelle span {
    font-size: 13px;
    color: var(--muted);
  }

  .rouge {
    color: var(--crit);
  }

  /* Phone: the full screen (D02, E10 mobile). */
  @media (max-width: 720px) {
    .panneau {
      inset: 0;
      width: 100%;
      border-radius: 0;
      padding: 18px 16px;
    }
  }
</style>
