<script lang="ts">
  // E09 · Catalogue (D02 E09_catalogue_desktop; S01 F-CAT-3, S02): the 6 crops
  // of the V1, filtered by family and by « de saison maintenant », and the
  // read-only fiche. Names, families and months come from the messages and the
  // locale; the data only holds identifiers and numbers (S05 §4).
  import { commands } from "../bindings";
  import Icone from "../coquille/Icone.svelte";
  import Logo from "../coquille/Logo.svelte";
    import { appeler, appli, dateLongue } from "../etat.svelte";
  import IconeFrise from "../frise/IconeFrise.svelte";
  import { m } from "../paraglide/messages.js";
  import { nomCulture } from "../planches";
  import { enOk } from "./appel";
  import { catalogue } from "./cultures.svelte";
  import PlanteIcone from "./PlanteIcone.svelte";
  import { nomFamille, tri } from "./catalogue";

  const aujourdhui = $derived(appli.vue?.today ?? new Date().toISOString().slice(0, 10));
  let saison = $state<string[]>([]);
  let seulementSaison = $state(false);
  let famille = $state<string | null>(null);
  let choisie = $state<string | null>(null);
  let racine: HTMLElement | undefined = $state();

  $effect(() => {
    const date = aujourdhui;
    void appeler(() => enOk(commands.culturesDeSaison(date))).then((r) => {
      if (r.status === "ok") saison = r.data;
    });
  });

  $effect(() => {
    racine?.focus();
  });

  const familles = $derived([...new Set(catalogue.cultures.map((c) => c.famille))].sort((a, b) => nomFamille(a).localeCompare(nomFamille(b), "fr")));
  const liste = $derived(
    tri(catalogue.cultures, nomCulture).filter(
      (c) => (!famille || c.famille === famille) && (!seulementSaison || saison.includes(c.id)),
    ),
  );
  const fiche = $derived(catalogue.culture(choisie ?? liste[0]?.id ?? ""));

  function fermer() {
    appli.catalogueOuvert = false;
  }

  /** `MM-JJ` as « 1er mars ». */
  function jourMois(mmjj: string): string {
    const [mois = 1, jour = 1] = mmjj.split("-").map(Number);
    return new Date(Date.UTC(2001, mois - 1, jour)).toLocaleDateString("fr-FR", { day: "numeric", month: "long", timeZone: "UTC" });
  }

  const STADES = [
    { stade: 0, texte: m.catalogue_stade_semis },
    { stade: 1, texte: m.catalogue_stade_pousse },
    { stade: 2, texte: m.catalogue_stade_adulte },
    { stade: 3, texte: m.catalogue_stade_recolte },
  ] as const;
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && fermer()} />

<div class="ecran" role="dialog" aria-modal="true" aria-labelledby="catalogue-titre" tabindex="-1" bind:this={racine}>
  <header class="entete">
    <Logo />
    <span class="nom">{m.app_nom()}</span>
    <div class="espace"></div>
    <button class="rond" aria-label={m.fermer()} title={m.fermer()} onclick={fermer}><Icone nom="close" taille={18} /></button>
  </header>

  <div class="colonnes">
    <section class="liste">
      <div class="tete">
        <h1 id="catalogue-titre">{m.catalogue_titre()}</h1>
        <label class="bascule">
          <input type="checkbox" role="switch" bind:checked={seulementSaison} />
          <span class="interrupteur" aria-hidden="true"></span>
          {m.catalogue_de_saison({ date: dateLongue(aujourdhui) })}
        </label>
      </div>

      <div class="filtres" role="group" aria-label={m.catalogue_filtre_famille()}>
        <button class="filtre" class:actif={famille === null} aria-pressed={famille === null} onclick={() => (famille = null)}>
          {m.catalogue_toutes()}
        </button>
        {#each familles as f (f)}
          <button class="filtre" class:actif={famille === f} aria-pressed={famille === f} onclick={() => (famille = f)}>
            {nomFamille(f)}
          </button>
        {/each}
      </div>

      <div class="grille">
        {#each liste as c (c.id)}
          <button class="carte" class:actif={fiche?.id === c.id} aria-pressed={fiche?.id === c.id} onclick={() => (choisie = c.id)}>
            <PlanteIcone culture={c.id} stade={2} largeur={64} />
            <strong>{nomCulture(c.id)}</strong>
            <span>{nomFamille(c.famille)}</span>
            {#if saison.includes(c.id)}<em class="badge">{m.catalogue_badge_saison()}</em>{/if}
          </button>
        {:else}
          <p class="aucune">{m.catalogue_aucune()}</p>
        {/each}
      </div>
      <p class="note">{m.catalogue_compte({ nombre: catalogue.cultures.length })}</p>
    </section>

    {#if fiche}
      <aside class="fiche" aria-label={m.catalogue_fiche()}>
        <div class="fiche-tete">
          <div>
            <p class="surtitre">{m.catalogue_fiche()}</p>
            <h2>{nomCulture(fiche.id)}</h2>
          </div>
          {#if saison.includes(fiche.id)}<em class="badge">{m.catalogue_badge_saison()}</em>{/if}
        </div>
        <div class="stades">
          {#each STADES as s (s.stade)}
            <figure>
              <PlanteIcone culture={fiche.id} stade={s.stade} largeur={58} />
              <figcaption>{s.texte()}</figcaption>
            </figure>
          {/each}
        </div>
        <dl>
          <div><dt>{m.catalogue_famille()}</dt><dd>{nomFamille(fiche.famille)}</dd></div>
          <div>
            <dt>{m.catalogue_abri()}</dt>
            <dd>{fiche.abri_jours > 0 ? m.catalogue_jours({ valeur: fiche.abri_jours }) : m.catalogue_sans_abri()}</dd>
          </div>
          <div><dt>{m.catalogue_apres()}</dt><dd>{m.catalogue_jours({ valeur: fiche.recolte_apres_jours })}</dd></div>
          <div><dt>{m.catalogue_duree()}</dt><dd>{m.catalogue_jours({ valeur: fiche.recolte_duree_jours })}</dd></div>
          <div>
            <dt>{m.catalogue_fenetre()}</dt>
            <dd>{m.catalogue_fenetre_valeur({ debut: jourMois(fiche.mise_en_place_debut), fin: jourMois(fiche.mise_en_place_fin) })}</dd>
          </div>
          <div><dt>{m.catalogue_rang()}</dt><dd>{m.cm({ valeur: fiche.rang_cm })}</dd></div>
          <div><dt>{m.catalogue_pied()}</dt><dd>{m.cm({ valeur: fiche.pied_cm })}</dd></div>
        </dl>
        <p class="verrou"><IconeFrise nom="lock" taille={14} />{m.catalogue_lecture_seule()}</p>
      </aside>
    {/if}
  </div>
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
    gap: 10px;
    max-width: 1300px;
    margin: 0 auto 20px;
  }

  .nom {
    font-family: var(--font-titre);
    font-weight: 600;
    font-size: 26px;
  }

  .espace {
    flex: 1;
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
    grid-template-columns: minmax(0, 1.8fr) minmax(280px, 1fr);
    gap: 24px;
    align-items: start;
    max-width: 1300px;
    margin: 0 auto;
  }

  .liste {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 24px;
    border-radius: 32px;
    background: var(--white);
  }

  .tete {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  h1 {
    font-size: 30px;
  }

  .bascule {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
  }

  .bascule input {
    position: absolute;
    opacity: 0;
  }

  .interrupteur {
    position: relative;
    width: 38px;
    height: 22px;
    border-radius: var(--pill);
    background: var(--line);
    transition: background 0.15s;
  }

  .interrupteur::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--white);
    transition: transform 0.15s;
  }

  .bascule input:checked + .interrupteur {
    background: var(--green);
  }

  .bascule input:checked + .interrupteur::after {
    transform: translateX(16px);
  }

  .bascule input:focus-visible + .interrupteur {
    outline: 3px solid var(--accent);
    outline-offset: 2px;
  }

  .filtres {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .filtre {
    min-height: 34px;
    padding: 0 16px;
    border: 0;
    border-radius: var(--pill);
    background: var(--card);
    font-size: 14px;
    font-weight: 600;
  }

  .filtre.actif {
    background: var(--ink);
    color: var(--white);
  }

  .grille {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 14px;
  }

  .carte {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    min-height: 150px;
    padding: 18px 10px 14px;
    border: 2px solid var(--line);
    border-radius: 20px;
    background: var(--white);
  }

  .carte.actif {
    border-color: var(--accent);
    background: var(--warn-bg);
  }

  .carte span {
    color: var(--muted);
    font-size: 13px;
  }

  .badge {
    padding: 1px 10px;
    border-radius: var(--pill);
    background: var(--green);
    color: var(--white);
    font-size: 11px;
    font-style: normal;
    font-weight: 700;
  }

  .aucune,
  .note {
    margin: 0;
    color: var(--muted);
    font-size: 13px;
  }

  .fiche {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 24px;
    border-radius: 32px;
    background: var(--card);
  }

  .fiche-tete {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
  }

  .surtitre {
    margin: 0;
    color: var(--muted);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  h2 {
    font-size: 28px;
  }

  .stades {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 4px;
    padding: 6px 0 4px;
  }

  figure {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    margin: 0;
    font-size: 12px;
    color: var(--muted);
  }

  dl {
    margin: 0;
  }

  dl div {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 9px 0;
    border-top: 1px solid var(--line);
    font-size: 14px;
  }

  dt {
    color: var(--muted);
  }

  dd {
    margin: 0;
    font-weight: 700;
    text-align: right;
  }

  .verrou {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    color: var(--muted);
    font-size: 13px;
  }

  @media (max-width: 900px) {
    .ecran {
      padding: 12px 12px 32px;
    }
    .colonnes {
      grid-template-columns: minmax(0, 1fr);
    }
    .liste,
    .fiche {
      padding: 16px;
      border-radius: 24px;
    }
    h1 {
      font-size: 24px;
    }
  }
</style>
