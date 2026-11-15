<script lang="ts">
  // Export of the timeline (E11, D02): format, period, planches and content on
  // the left, the preview on the right. The drawing is `FriseSvg`, mounted off
  // screen on the chosen period and serialised (`serialiserFrise`); « Enregistrer
  // sous » asks for the path, then Rust converts it (`exporter_frise`).
  import { save } from "@tauri-apps/plugin-dialog";
  import { get } from "svelte/store";
  import { tick } from "svelte";
  import { commands, type AppError, type FriseFormat, type YearView } from "../bindings";
  import Icone from "../coquille/Icone.svelte";
  import Logo from "../coquille/Logo.svelte";
  import { aujourdhuiLocal } from "../dates";
  import { appeler, appli } from "../etat.svelte";
  import { m } from "../paraglide/messages.js";
  import { curseur, fenetreFrise } from "../temps";
  import { serialiserFrise } from "./exportSvg";
  import { anneesDePeriode, echellePourPeriode, periodeParDefaut, periodeVisible, problemePeriode } from "./exportPlan";
  import FriseSvg, { LARGEUR_LIBELLES } from "./FriseSvg.svelte";
  import { construireLignes } from "./geometrie";

  // The period shown on screen, zoom and scroll included; the cursor's year
  // only if the timeline was never drawn.
  const fenetre = get(fenetreFrise);
  const defaut = fenetre ? periodeVisible(fenetre) : periodeParDefaut(get(curseur));
  let format = $state<FriseFormat>("pdf");
  let du = $state(defaut.du);
  let au = $state(defaut.au);
  let choisies = $state<string[]>((appli.vue?.planches ?? []).map((p) => p.id));
  let phases = $state(true);
  let repere = $state(true);
  let annotations = $state(true);
  let enCours = $state(false);
  /** Why the last export failed (ET01): shown in the panel, with « Réessayer ». */
  let echec = $state<AppError["code"] | null>(null);
  let racine: HTMLElement;
  let dessin: HTMLElement | undefined = $state();
  let svgTexte = $state("");

  const planches = $derived(appli.vue?.planches ?? []);
  const probleme = $derived(problemePeriode(du, au));
  const echelle = $derived(probleme ? null : echellePourPeriode(du, au));

  // The year views of the period (the current one is already loaded).
  let chargees = $state<Record<number, YearView>>({});
  $effect(() => {
    if (probleme) return;
    for (const annee of anneesDePeriode(du, au)) {
      if (annee === appli.vue?.year || chargees[annee]) continue;
      void appeler(() => commands.vueAnnee(annee)).then((r) => {
        if (r.status === "ok") chargees[annee] = r.data;
      });
    }
  });
  const vues = $derived(
    probleme || !appli.vue
      ? []
      : anneesDePeriode(du, au)
          .map((a) => (a === appli.vue!.year ? appli.vue! : chargees[a]))
          .filter((v): v is YearView => !!v),
  );
  const prete = $derived(!probleme && vues.length === anneesDePeriode(du, au).length);
  const lignes = $derived(
    prete ? construireLignes(vues, appli.vue!.year).filter((l) => choisies.includes(l.planche.id)) : [],
  );
  const notes = $derived(
    annotations
      ? [...new Map(vues.flatMap((v) => v.annotations).map((a) => [a.id, a])).values()].filter(
          (a) => !a.masquee && (a.planche_id === null || choisies.includes(a.planche_id)),
        )
      : [],
  );
  const exportable = $derived(prete && lignes.length > 0 && !enCours);

  // Serialise after each redraw of the off-screen drawing; the preview shows that very text.
  $effect(() => {
    void [lignes, echelle, notes, phases, repere];
    void tick().then(() => {
      const svg = dessin?.querySelector<SVGSVGElement>("svg[data-frise]");
      svgTexte = svg && lignes.length > 0 ? serialiserFrise(svg) : "";
    });
  });
  const apercu = $derived(svgTexte ? `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svgTexte)}` : "");

  $effect(() => {
    racine.focus();
  });

  function fermer() {
    appli.exportOuvert = false;
  }

  function basculer(id: string) {
    choisies = choisies.includes(id) ? choisies.filter((c) => c !== id) : [...choisies, id];
  }

  const toutes = $derived(choisies.length === planches.length);

  async function exporter() {
    if (!exportable || !svgTexte) return;
    echec = null;
    const nom = m.export_nom_fichier({ date: aujourdhuiLocal(), format });
    const chemin = await save({
      defaultPath: nom,
      filters: [{ name: format === "pdf" ? m.export_format_pdf() : m.export_format_png(), extensions: [format] }],
    });
    if (typeof chemin !== "string") return;
    enCours = true;
    const resultat = await appeler(() => commands.exporterFrise(svgTexte, format, chemin));
    enCours = false;
    if (resultat.status === "ok") {
      appli.informer(m.export_ok());
      fermer();
    } else if (resultat.error.code === "fichier") {
      echec = "fichier";
    } else {
      appli.signaler(resultat.error);
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && fermer()} />

<div class="ecran" role="dialog" aria-modal="true" aria-labelledby="export-titre" tabindex="-1" bind:this={racine}>
  <header class="entete">
    <Logo />
    <button class="rond" aria-label={m.fermer()} title={m.fermer()} onclick={fermer}><Icone nom="close" taille={18} /></button>
  </header>

  <div class="colonnes">
    <section class="options">
      <h1 id="export-titre">{m.export_titre()}</h1>

      <div class="formats" role="group" aria-label={m.export_format()}>
        {#each [{ f: "pdf", titre: m.export_format_pdf(), aide: m.export_format_pdf_aide() }, { f: "png", titre: m.export_format_png(), aide: m.export_format_png_aide() }] as choix (choix.f)}
          <button type="button" class="format" class:actif={format === choix.f} aria-pressed={format === choix.f} onclick={() => (format = choix.f as FriseFormat)}>
            <strong>{choix.titre}</strong>
            <span>{choix.aide}</span>
          </button>
        {/each}
      </div>

      <fieldset>
        <legend>{m.export_periode()}</legend>
        <div class="dates">
          <label>{m.export_du()}<input type="date" bind:value={du} /></label>
          <label>{m.export_au()}<input type="date" bind:value={au} /></label>
        </div>
        <p class="aide" class:erreur={probleme}>
          {probleme === "vide"
            ? m.export_periode_vide()
            : probleme === "inversee"
              ? m.export_periode_inversee()
              : probleme === "trop_longue"
                ? m.export_periode_trop_longue()
                : m.export_periode_aide()}
        </p>
      </fieldset>

      <fieldset>
        <legend>
          {m.export_planches()}
          <button type="button" class="lien" onclick={() => (choisies = toutes ? [] : planches.map((p) => p.id))}>
            {toutes ? m.export_aucune() : m.export_toutes()}
          </button>
        </legend>
        <div class="puces">
          {#each planches as p (p.id)}
            <label class="puce"><input type="checkbox" checked={choisies.includes(p.id)} onchange={() => basculer(p.id)} />{p.nom}</label>
          {/each}
        </div>
      </fieldset>

      <fieldset>
        <legend>{m.export_contenu()}</legend>
        <label class="case"><input type="checkbox" bind:checked={phases} />{m.export_phases()}</label>
        <label class="case"><input type="checkbox" bind:checked={repere} />{m.export_repere()}</label>
        <label class="case"><input type="checkbox" bind:checked={annotations} />{m.note_afficher()}</label>
      </fieldset>

      {#if echec}
        <div class="carte-erreur" role="alert">
          <div class="titre"><Icone nom="alert" taille={18} /><strong>{m.etat_fichier_titre()}</strong></div>
          <p>{m.etat_fichier_texte()}</p>
          <button class="bouton" onclick={exporter}><Icone nom="retry" taille={16} />{m.reessayer()}</button>
        </div>
      {/if}

      <button class="bouton principal" disabled={!exportable || !svgTexte} onclick={exporter}>
        <Icone nom="export" taille={16} />{format === "pdf" ? m.export_bouton_pdf() : m.export_bouton_png()}
      </button>
    </section>

    <section class="apercu" aria-label={m.export_apercu()}>
      <div class="apercu-tete">
        <h2>{m.export_apercu()}</h2>
        <span>{m.export_du_jour({ date: new Date().toLocaleDateString("fr-FR", { day: "numeric", month: "short", year: "numeric" }) })}</span>
      </div>
      <div class="feuille">
        {#if apercu}
          <img src={apercu} alt={m.export_apercu_alt()} />
        {:else}
          <p class="vide">{lignes.length === 0 && prete ? m.export_aucune_planche() : m.export_apercu_vide()}</p>
        {/if}
      </div>
      <p class="aide">{m.export_pdf_aide()}</p>
    </section>
  </div>
</div>

<!-- The drawing that is serialised: off screen, never shown. -->
<div class="hors-ecran" aria-hidden="true" bind:this={dessin}>
  {#if echelle && lignes.length > 0}
    <FriseSvg
      {lignes}
      {echelle}
      largeurLibelles={LARGEUR_LIBELLES}
      aujourdhui={appli.vue?.today ?? aujourdhuiLocal()}
      curseur={null}
      interactif={false}
      anneeAffichee={appli.vue?.year}
      annotations={notes}
      repereAujourdhui={repere}
      {phases}
    />
  {/if}
</div>

<style>
  .ecran {
    position: fixed;
    inset: 0;
    z-index: 15;
    overflow-y: auto;
    padding: 20px 32px 40px;
    background: var(--bg);
    outline: none;
  }

  .entete {
    display: flex;
    align-items: center;
    justify-content: space-between;
    max-width: 1280px;
    margin: 0 auto 20px;
  }

  .rond {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border: 1px solid var(--line);
    border-radius: 50%;
    background: var(--white);
  }

  .colonnes {
    display: grid;
    grid-template-columns: minmax(0, 420px) minmax(0, 1fr);
    gap: 24px;
    align-items: start;
    max-width: 1280px;
    margin: 0 auto;
  }

  .options,
  .apercu {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 24px;
    border-radius: 32px;
  }

  .options {
    background: var(--white);
  }

  .apercu {
    background: var(--card);
  }

  h1 {
    font-size: 26px;
  }

  h2 {
    font-size: 15px;
  }

  .formats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .format {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 16px 10px;
    border: 2px solid var(--line);
    border-radius: 20px;
    background: var(--white);
    text-align: center;
  }

  .format span {
    color: var(--muted);
    font-size: 12px;
  }

  .format.actif {
    border-color: var(--accent);
    background: #fff1e0;
  }

  fieldset {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 0;
    padding: 0;
    border: 0;
  }

  legend {
    display: flex;
    justify-content: space-between;
    width: 100%;
    padding: 0;
    margin-bottom: 8px;
    color: var(--muted);
    font-size: 13px;
    font-weight: 600;
  }

  .dates {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .dates label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: var(--muted);
    font-size: 12px;
    font-weight: 600;
  }

  input[type="date"] {
    min-height: 44px;
    padding: 0 12px;
    border: 0;
    border-radius: 14px;
    background: var(--bg);
    color: var(--ink);
    font: inherit;
  }

  .aide {
    margin: 0;
    color: var(--muted);
    font-size: 13px;
  }

  .aide.erreur {
    color: var(--crit);
  }

  .lien {
    border: 0;
    background: none;
    color: var(--green);
    font: inherit;
    font-weight: 600;
    text-decoration: underline;
  }

  .puces {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .puce,
  .case {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
  }

  .puce {
    padding: 8px 12px;
    border-radius: var(--pill);
    background: var(--bg);
    font-weight: 600;
  }

  input[type="checkbox"] {
    accent-color: var(--green);
    width: 18px;
    height: 18px;
  }

  .carte-erreur {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    padding: 14px 16px;
    border-radius: 20px;
    background: var(--crit-bg);
    color: var(--crit);
  }

  .carte-erreur .titre {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .carte-erreur p {
    margin: 0;
    color: var(--ink);
    font-size: 14px;
  }

  .apercu-tete {
    display: flex;
    justify-content: space-between;
    color: var(--muted);
    font-size: 13px;
  }

  .feuille {
    overflow: auto;
    padding: 16px;
    border-radius: 24px;
    background: var(--white);
  }

  .feuille img {
    display: block;
    width: 100%;
    height: auto;
  }

  .vide {
    margin: 24px 0;
    color: var(--muted);
    text-align: center;
  }

  .hors-ecran {
    position: absolute;
    left: -100000px;
    top: 0;
    width: 0;
    height: 0;
    overflow: hidden;
  }

  @media (max-width: 900px) {
    .ecran {
      padding: 16px;
    }

    .colonnes {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
