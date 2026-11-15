<script lang="ts">
  // E07 · New serie and edition (D02 E07_serie_desktop, D01 E07_serie_mobile;
  // S01 F-PLA-1, F-PLA-2). Three questions (crop, how it starts, when), the
  // zone of the planche, and on the side what Rust calculates as the user
  // types: the steps with their dates, the quantity, the conflicts.
  //
  // Nothing is calculated here. Each change calls `apercu_serie`; the form
  // shows its answer and saves with `creer_serie` or `modifier_serie`.
  import { untrack } from "svelte";
  import { commands, type Mode, type ProblemeView, type SerieApercu } from "../bindings";
  import Confirmation from "../coquille/Confirmation.svelte";
  import Icone from "../coquille/Icone.svelte";
  import Logo from "../coquille/Logo.svelte";
  import { dateAbregee, dateComplete } from "../dates";
  import { traduireErreur } from "../erreurs";
  import { appeler, appli, type OuvertureSerie } from "../etat.svelte";
  import IconeFrise from "../frise/IconeFrise.svelte";
  import { m } from "../paraglide/messages.js";
  import { enMetres, nomCulture } from "../planches";
  import { curseur } from "../temps";
  import { get } from "svelte/store";
  import { enOk } from "./appel";
  import { catalogue } from "./cultures.svelte";
  import MiniFrise from "./MiniFrise.svelte";
  import PlanteIcone from "./PlanteIcone.svelte";
  import {
    RACCOURCIS,
    appliquerSuggestion,
    basculerRepiquage,
    finSaisie,
    formulaireDepuisSerie,
    formulaireVide,
    memeZone,
    modeInitial,
    premierRaccourciLibre,
    segmentsMiniFrise,
    versSerieInput,
    versZone,
    zoneApresClic,
    zoneDuRaccourci,
    type Formulaire,
    type Zone,
  } from "./serie";

  let { ouverture }: { ouverture: OuvertureSerie } = $props();

  const dateValide = (date: string) => /^\d{4}-\d{2}-\d{2}$/.test(date);

  const edition = $derived(ouverture.type === "edition" ? ouverture.serie : null);
  const vue = $derived(appli.vue!);

  function troncons(plancheId: string): number {
    return vue.planches.find((p) => p.id === plancheId)?.troncons ?? 1;
  }

  function initial(): Formulaire {
    if (ouverture.type === "edition") return formulaireDepuisSerie(ouverture.serie);
    const { plancheId, creneau, suggestion } = ouverture;
    const base = formulaireVide(
      catalogue.culture(suggestion?.culture_id ?? "") ?? catalogue.cultures[0],
      plancheId,
      troncons(plancheId),
      get(curseur),
    );
    if (!creneau) return base;
    const zone = { debut: creneau.troncon_debut, fin: creneau.troncon_fin };
    // A slot opens on its first day, or on the day the suggestion proposes.
    return suggestion
      ? appliquerSuggestion({ ...base, date: creneau.start }, suggestion, zone)
      : { ...base, date: creneau.start, zone };
  }

  let f = $state<Formulaire>(untrack(initial));
  let apercu = $state<SerieApercu | null>(null);
  let saison = $state<string[]>([]);
  let suppression = $state(false);
  let envoi = $state(false);
  /** The zone follows the first free shortcut until the user touches it. */
  let zoneAuto = $state(untrack(() => ouverture.type === "nouvelle" && !ouverture.creneau));
  let cultureChoisie = $state(untrack(() => ouverture.type === "edition" || !!ouverture.suggestion));
  let racine: HTMLElement | undefined = $state();

  const planche = $derived(vue.planches.find((p) => p.id === f.plancheId));
  const nombreTroncons = $derived(planche?.troncons ?? Math.max(f.zone.fin, 1));
  const culture = $derived(catalogue.culture(f.cultureId));
  const dates = $derived(apercu?.dates ?? null);
  const finCalculee = $derived(dates?.default_end ?? "");
  const creneau = $derived(ouverture.type === "nouvelle" ? ouverture.creneau : undefined);
  const sansGodets = $derived(!!culture && culture.abri_jours === 0 && f.mode === "abri");

  // ---------- What Rust says about the form ----------

  let jetonApercu = 0;
  $effect(() => {
    const entree = versSerieInput(f);
    const id = edition?.id ?? null;
    if (!entree.culture_id) return;
    const mon = ++jetonApercu;
    const minuteur = setTimeout(async () => {
      const r = await appeler(() => commands.apercuSerie(id, entree));
      if (mon === jetonApercu && r.status === "ok") apercu = r.data;
    }, 80);
    return () => clearTimeout(minuteur);
  });

  // From « Planter ici »: if the whole planche is taken during that time, start
  // from the first free shortcut Rust offers.
  $effect(() => {
    if (!apercu || !zoneAuto) return;
    untrack(() => {
      zoneAuto = false;
      if (apercu!.problemes.length === 0) return;
      const libre = premierRaccourciLibre(apercu!.raccourcis);
      if (libre && !memeZone(libre, f.zone)) f.zone = libre;
    });
  });

  // The crops in season on the chosen date, highlighted in the list (F-PLA-1).
  $effect(() => {
    const date = f.date;
    if (!dateValide(date)) return;
    void appeler(() => enOk(commands.culturesDeSaison(date))).then((r) => {
      if (r.status === "ok") saison = r.data;
    });
  });

  // A new serie from « Planter ici » starts on the first crop in season.
  $effect(() => {
    if (cultureChoisie || saison.length === 0 || ouverture.type !== "nouvelle") return;
    untrack(() => {
      const premiere = ordre.find((id) => saison.includes(id));
      if (premiere) choisirCulture(premiere, false);
    });
  });

  // ---------- Crops: the suggestions first, then those in season ----------

  const suggeres = $derived(creneau?.suggestions.map((s) => s.culture_id) ?? []);
  const ordre = $derived(
    catalogue.cultures
      .map((c) => c.id)
      .sort((a, b) => {
        const rang = (id: string) => (suggeres.includes(id) ? 0 : saison.includes(id) ? 1 : 2);
        return rang(a) - rang(b) || nomCulture(a).localeCompare(nomCulture(b), "fr");
      }),
  );

  function choisirCulture(id: string, utilisateur = true) {
    if (utilisateur) cultureChoisie = true;
    if (id === f.cultureId) return;
    const c = catalogue.culture(id);
    // The card checked at first follows the crop; once the user has picked a mode it stays.
    f = { ...f, cultureId: id, mode: edition || utilisateur && modeChoisi ? f.mode : modeInitial(c) };
  }

  let modeChoisi = $state(false);
  function choisirMode(mode: Mode) {
    modeChoisi = true;
    f = { ...f, mode, repiquage: mode === "abri" ? f.repiquage : false };
  }

  // ---------- Zone ----------

  function choisirZone(z: Zone) {
    zoneAuto = false;
    f.zone = z;
  }

  function choisirPlanche(id: string) {
    zoneAuto = true;
    f = { ...f, plancheId: id, zone: { debut: 1, fin: troncons(id) } };
  }

  function occupe(i: number): boolean {
    return !!apercu?.occupes.some((z) => z.troncon_debut <= i && i <= z.troncon_fin);
  }

  // ---------- Dates ----------

  const question = $derived(
    f.mode === "plants" ? m.serie_q_plante() : f.mode === "abri" && f.repiquage ? m.serie_q_repique() : m.serie_q_seme(),
  );

  const etapes = $derived.by(() => {
    if (!dates) return [];
    const liste: { date: string; texte: string; fort?: boolean }[] = [];
    if (f.mode === "abri" && dates.sowing) liste.push({ date: dates.sowing, texte: m.serie_etape_semis_abri() });
    liste.push({
      date: dates.planting_out,
      texte: f.mode === "plants" ? m.serie_etape_plantation() : f.mode === "abri" ? m.serie_etape_repiquage() : m.serie_etape_semis_terre(),
    });
    liste.push({ date: dates.harvest_start, texte: m.serie_etape_recolte() });
    liste.push({ date: dates.end, texte: dates.manual_end ? m.serie_etape_fin_manuelle() : m.serie_etape_fin(), fort: dates.manual_end });
    return liste;
  });

  function saisirFin(valeur: string) {
    f.finManuelle = finSaisie(valeur, finCalculee);
  }

  // ---------- Problems and saving ----------

  function nomPlanche(): string {
    return planche?.nom ?? "";
  }

  function texteProbleme(p: ProblemeView): string {
    if (p.code === "conflict" && p.zone && p.start && p.end) {
      const autre = [...vue.series, ...vue.historique].find((s) => s.id === p.avec);
      return m.serie_probleme_conflit({
        culture: autre ? nomCulture(autre.culture_id) : m.serie_probleme_autre(),
        debut: p.zone.troncon_debut,
        fin: p.zone.troncon_fin,
        du: dateAbregee(p.start, vue.year),
        au: dateAbregee(p.end, vue.year),
      });
    }
    if (p.code === "zone_hors_planche" && p.troncons !== null && p.annee !== null) {
      return m.serie_probleme_zone({ troncons: p.troncons, annee: p.annee });
    }
    if (p.code === "planche_absente" && p.annee !== null) return m.serie_probleme_absente({ annee: p.annee });
    return traduireErreur({ code: p.code, detail: "" });
  }

  const valide = $derived(!!dates && (apercu?.problemes.length ?? 1) === 0);
  const pieds = $derived(dates ? dates.rangs * dates.pieds_par_rang : 0);

  function fermer() {
    appli.serieOuverte = null;
  }

  async function valider() {
    if (!valide || envoi) return;
    envoi = true;
    const entree = versSerieInput(f);
    const ok = await appli.modifier((annee) =>
      edition ? commands.modifierSerie(annee, edition.id, entree) : commands.creerSerie(annee, entree),
    );
    envoi = false;
    if (ok) {
      appli.informer(edition ? m.serie_modifiee() : m.serie_creee());
      fermer();
    }
  }

  async function supprimer() {
    suppression = false;
    if (!edition) return;
    const id = edition.id;
    if (await appli.modifier((annee) => commands.supprimerSerie(annee, id))) {
      appli.informer(m.serie_supprimee());
      fermer();
    }
  }

  $effect(() => {
    racine?.focus();
  });

  const MODES: { mode: Mode; titre: () => string; aide: () => string }[] = [
    { mode: "abri", titre: m.serie_mode_abri, aide: m.serie_mode_abri_aide },
    { mode: "plants", titre: m.serie_mode_plants, aide: m.serie_mode_plants_aide },
    { mode: "pleine_terre", titre: m.serie_mode_terre, aide: m.serie_mode_terre_aide },
  ];
  const LIBELLES_RACCOURCIS = {
    toute: m.serie_raccourci_toute,
    moitie: m.serie_raccourci_moitie,
    tiers: m.serie_raccourci_tiers,
    quart: m.serie_raccourci_quart,
  };
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape" && !suppression) fermer();
  }}
/>

<div class="ecran" role="dialog" aria-modal="true" aria-labelledby="serie-titre" tabindex="-1" bind:this={racine}>
  <header class="entete">
    <Logo />
    <div class="titres">
      <h1 id="serie-titre">{edition ? m.serie_titre_edition() : m.serie_titre_nouvelle()}</h1>
      <p>{nomPlanche()}</p>
    </div>
    <button class="rond" aria-label={m.fermer()} title={m.fermer()} onclick={fermer}><Icone nom="close" taille={18} /></button>
  </header>

  <div class="colonnes">
    <section class="questions">
      {#if creneau}
        <div class="creneau">
          <p class="creneau-texte">
            {m.serie_creneau({
              debut: dateAbregee(creneau.start, vue.year),
              fin: dateAbregee(creneau.end, vue.year),
            })}
          </p>
          {#if creneau.suggestions.length > 0}
            <p class="etiquette">{m.serie_suggestions()}</p>
            <div class="puces">
              {#each creneau.suggestions as s (s.culture_id)}
                <button
                  class="puce"
                  class:actif={f.cultureId === s.culture_id && f.date === s.date}
                  onclick={() => {
                    cultureChoisie = true;
                    modeChoisi = true;
                    zoneAuto = false;
                    f = appliquerSuggestion(f, s, { debut: creneau.troncon_debut, fin: creneau.troncon_fin });
                  }}
                >
                  {nomCulture(s.culture_id)} · {dateAbregee(s.date, vue.year)}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <h2 class="question"><span class="numero">1</span>{m.serie_q_legume()}</h2>
      <div class="legumes" role="radiogroup" aria-label={m.serie_q_legume()}>
        {#each ordre as id (id)}
          <button
            class="legume"
            class:actif={f.cultureId === id}
            role="radio"
            aria-checked={f.cultureId === id}
            onclick={() => choisirCulture(id)}
          >
            <PlanteIcone culture={id} stade={2} largeur={52} />
            <span>{nomCulture(id)}</span>
            {#if suggeres.includes(id)}
              <em class="badge">{m.serie_suggere()}</em>
            {:else if saison.includes(id)}
              <em class="badge saison">{m.serie_de_saison()}</em>
            {/if}
          </button>
        {/each}
      </div>

      <h2 class="question"><span class="numero">2</span>{m.serie_q_demarrage()}</h2>
      <div class="modes" role="radiogroup" aria-label={m.serie_q_demarrage()}>
        {#each MODES as option (option.mode)}
          <label class="mode" class:actif={f.mode === option.mode}>
            <input type="radio" name="mode" checked={f.mode === option.mode} onchange={() => choisirMode(option.mode)} />
            <span class="mode-texte">
              <strong>{option.titre()}</strong>
              <span>{option.aide()}</span>
            </span>
            <span class="radio" aria-hidden="true"></span>
          </label>
        {/each}
      </div>

      {#if sansGodets}
        <label class="champ">
          <span>{m.serie_godets_question()}</span>
          <input type="number" min="1" step="1" inputmode="numeric" bind:value={f.abriJours} />
          <small>{m.serie_godets_aide()}</small>
        </label>
      {/if}

      <h2 class="question"><span class="numero">3</span>{question}</h2>
      <label class="date">
        <input type="date" bind:value={f.date} aria-label={question} />
        <span class="jour">{dateValide(f.date) ? dateComplete(f.date) : ""}</span>
      </label>
      {#if f.mode === "abri"}
        <button class="lien" onclick={() => (f = basculerRepiquage(f, apercu))}>
          {f.repiquage ? m.serie_lien_semis() : m.serie_lien_repiquage()}
        </button>
      {/if}
    </section>

    <aside class="resume" aria-live="polite">
      <div class="carte">
        {#if dates}
          <MiniFrise segments={segmentsMiniFrise(dates, f.mode)} cultureId={f.cultureId} />
          <ol class="etapes">
            {#each etapes as etape (etape.texte)}
              <li class:fort={etape.fort}>
                <strong>{dateAbregee(etape.date, vue.year)}</strong>
                <span>{etape.texte}</span>
              </li>
            {/each}
          </ol>
          <div class="fin">
            <label>
              <span>{m.serie_fin_label()}</span>
              <input type="date" value={f.finManuelle ?? dates.end} onchange={(e) => saisirFin(e.currentTarget.value)} />
            </label>
            {#if dates.manual_end}
              <p class="manuelle">
                <IconeFrise nom="crayon" taille={14} />
                {m.serie_fin_manuelle({ date: dateAbregee(dates.end, vue.year) })}
              </p>
              <button class="lien" onclick={() => (f.finManuelle = null)}>
                {m.serie_recalculer({ date: dateAbregee(dates.default_end, vue.year) })}
              </button>
            {/if}
          </div>
        {:else}
          <p class="attente">{m.serie_calcul()}</p>
        {/if}
      </div>

      <div class="zone">
        <div class="zone-tete">
          <label class="planche">
            <span class="etiquette">{m.serie_zone_titre()}</span>
            <select value={f.plancheId} onchange={(e) => choisirPlanche(e.currentTarget.value)} aria-label={m.serie_planche()}>
              {#each vue.planches as p (p.id)}
                <option value={p.id}>{p.nom}</option>
              {/each}
            </select>
          </label>
          {#if planche}
            <span class="etiquette">{m.serie_zone_dimensions({ longueur: enMetres(planche.longueur_cm), largeur: enMetres(planche.largeur_cm) })}</span>
          {/if}
        </div>
        <div class="raccourcis" role="group" aria-label={m.serie_zone_titre()}>
          {#each RACCOURCIS as cle (cle)}
            {@const z = apercu ? zoneDuRaccourci(apercu.raccourcis, cle) : null}
            <button
              class="raccourci"
              class:actif={memeZone(z, f.zone)}
              disabled={!z}
              onclick={() => z && choisirZone(z)}>{LIBELLES_RACCOURCIS[cle]()}</button
            >
          {/each}
        </div>
        <div class="bande" role="group" aria-label={m.serie_zone_titre()}>
          {#each Array.from({ length: nombreTroncons }, (_, i) => i + 1) as i (i)}
            <button
              class="troncon"
              class:choisi={f.zone.debut <= i && i <= f.zone.fin}
              class:pris={occupe(i)}
              aria-pressed={f.zone.debut <= i && i <= f.zone.fin}
              aria-label={m.serie_troncon_aria({ numero: i })}
              onclick={() => choisirZone(zoneApresClic(f.zone, i))}
            ></button>
          {/each}
        </div>
        {#if dates}
          <p class="quantite">
            <span>{m.serie_quantite({ pieds, rangs: dates.rangs, par_rang: dates.pieds_par_rang })}</span>
          </p>
        {/if}
      </div>

      {#if apercu && apercu.problemes.length > 0}
        <ul class="problemes" role="alert">
          {#each apercu.problemes as p (p.code + (p.avec ?? ""))}
            <li><Icone nom="alert" taille={16} />{texteProbleme(p)}</li>
          {/each}
        </ul>
      {/if}

      <button class="bouton principal valider" disabled={!valide || envoi} onclick={valider}>
        <IconeFrise nom="check" />
        {edition ? m.serie_enregistrer() : m.serie_planter({ culture: nomCulture(f.cultureId), pieds })}
      </button>
      {#if edition}
        <button class="bouton danger" onclick={() => (suppression = true)}>
          <Icone nom="trash" taille={16} />{m.serie_supprimer()}
        </button>
      {/if}
    </aside>
  </div>
</div>

{#if suppression && edition}
  <Confirmation
    titre={m.serie_supprimer_titre()}
    confirmer={m.serie_supprimer()}
    onconfirmer={supprimer}
    onannuler={() => (suppression = false)}
  >
    <p>
      {m.serie_supprimer_texte({
        culture: nomCulture(edition.culture_id),
        planche: nomPlanche(),
        debut: dateAbregee(edition.date, vue.year),
        fin: dateAbregee(edition.end, vue.year),
      })}
    </p>
  </Confirmation>
{/if}

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
    gap: 14px;
    max-width: 1100px;
    margin: 0 auto 20px;
  }

  .titres {
    flex: 1;
  }

  .titres h1 {
    font-size: 22px;
  }

  .titres p {
    margin: 0;
    color: var(--muted);
    font-size: 14px;
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
    grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
    gap: 24px;
    align-items: start;
    max-width: 1100px;
    margin: 0 auto;
  }

  .questions {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 24px;
    border-radius: 32px;
    background: var(--white);
  }

  .resume {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 20px;
    border-radius: 32px;
    background: var(--card);
    position: sticky;
    top: 16px;
  }

  .question {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 6px;
    font-size: 18px;
  }

  .numero {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--ink);
    color: var(--white);
    font-family: var(--font);
    font-size: 14px;
    font-weight: 700;
  }

  .creneau {
    padding: 12px 14px;
    border-radius: 16px;
    background: var(--warn-bg);
  }

  .creneau-texte {
    margin: 0 0 8px;
    font-weight: 600;
    color: var(--accent-ink);
  }

  .etiquette {
    margin: 0;
    color: var(--muted);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
  }

  .puces {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 6px;
  }

  .puce {
    min-height: 36px;
    padding: 0 14px;
    border: 1px solid var(--accent);
    border-radius: var(--pill);
    background: var(--white);
    font-size: 14px;
    font-weight: 600;
  }

  .puce.actif {
    background: var(--accent);
    color: var(--accent-ink);
  }

  .legumes {
    display: grid;
    grid-template-columns: repeat(6, minmax(0, 1fr));
    gap: 8px;
  }

  .legume {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 10px 4px 8px;
    border: 2px solid transparent;
    border-radius: 16px;
    background: var(--card);
    font-size: 13px;
    font-weight: 600;
  }

  .legume.actif {
    border-color: var(--accent);
    background: var(--warn-bg);
  }

  .badge {
    padding: 1px 8px;
    border-radius: var(--pill);
    background: var(--accent);
    color: var(--accent-ink);
    font-size: 10px;
    font-style: normal;
    font-weight: 700;
  }

  .badge.saison {
    background: var(--green);
    color: var(--white);
  }

  .modes {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .mode {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    border: 2px solid var(--line);
    border-radius: 18px;
    cursor: pointer;
  }

  .mode input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .mode:has(input:focus-visible) {
    outline: 3px solid var(--accent);
    outline-offset: 2px;
  }

  .mode.actif {
    border-color: var(--accent);
    background: var(--warn-bg);
  }

  .mode-texte {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 2px;
  }

  .mode-texte span {
    color: var(--muted);
    font-size: 13px;
  }

  .radio {
    width: 22px;
    height: 22px;
    border: 2px solid var(--line);
    border-radius: 50%;
  }

  .mode.actif .radio {
    border-color: var(--accent);
    background: radial-gradient(circle, var(--accent) 45%, transparent 50%);
  }

  .champ {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-weight: 600;
    font-size: 14px;
  }

  .champ small {
    color: var(--muted);
    font-weight: 400;
  }

  input[type="number"],
  input[type="date"],
  select {
    min-height: 44px;
    padding: 0 14px;
    border: 1px solid var(--line);
    border-radius: 14px;
    background: var(--card);
  }

  .date {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
    padding: 4px 14px;
    border-radius: 16px;
    background: var(--card);
  }

  .date input {
    min-height: 44px;
    padding: 0;
    border: 0;
    background: transparent;
    font-weight: 600;
  }

  .jour {
    display: inline-block;
    color: var(--muted);
    font-size: 14px;
  }

  .jour::first-letter {
    text-transform: uppercase;
  }

  .lien {
    align-self: flex-start;
    padding: 0;
    border: 0;
    background: none;
    color: var(--green);
    font-size: 14px;
    font-weight: 600;
    text-decoration: underline;
  }

  .carte {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border-radius: 22px;
    background: var(--white);
  }

  .etapes {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .etapes li {
    display: flex;
    flex-direction: column;
    gap: 1px;
    font-size: 12px;
    color: var(--muted);
  }

  .etapes strong {
    color: var(--ink);
    font-size: 14px;
  }

  .etapes li.fort span {
    color: var(--accent-ink);
    font-weight: 700;
  }

  .fin {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 12px;
    padding-top: 10px;
    border-top: 1px solid var(--line);
  }

  .fin label {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    font-weight: 600;
  }

  .fin input {
    min-height: 36px;
  }

  .manuelle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin: 0;
    padding: 4px 10px;
    border-radius: var(--pill);
    background: var(--warn-bg);
    color: var(--accent-ink);
    font-size: 13px;
    font-weight: 700;
  }

  .attente {
    margin: 0;
    color: var(--muted);
  }

  .zone {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .zone-tete {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .planche {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .planche select {
    min-height: 36px;
    background: var(--white);
  }

  .raccourcis {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 4px;
    padding: 4px;
    border-radius: var(--pill);
    background: var(--white);
  }

  .raccourci {
    min-height: 40px;
    border: 0;
    border-radius: var(--pill);
    background: transparent;
    font-weight: 600;
  }

  .raccourci.actif {
    background: var(--ink);
    color: var(--white);
  }

  .bande {
    display: flex;
    gap: 4px;
  }

  .troncon {
    flex: 1;
    height: 38px;
    min-width: 0;
    border: 2px solid var(--line);
    border-radius: 8px;
    background: var(--white);
  }

  .troncon.pris {
    border-color: var(--bed-furrow);
    background: repeating-linear-gradient(135deg, var(--bed) 0 5px, var(--bed-furrow) 5px 8px);
  }

  .troncon.choisi {
    border-color: var(--green);
    background: #b8dbc6;
  }

  .troncon.pris.choisi {
    border-color: var(--crit);
  }

  .quantite {
    display: flex;
    justify-content: space-between;
    margin: 0;
    padding-top: 10px;
    border-top: 1px solid var(--line);
    font-size: 14px;
  }

  .problemes {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: 0;
    padding: 12px 14px;
    border-radius: 16px;
    background: var(--crit-bg);
    color: #5a1e1a;
    font-size: 14px;
    list-style: none;
  }

  .problemes li {
    display: flex;
    align-items: flex-start;
    gap: 8px;
  }

  .valider {
    min-height: 52px;
    font-size: 17px;
  }

  .danger {
    color: var(--crit);
    border-color: var(--crit);
  }

  @media (max-width: 900px) {
    .ecran {
      padding: 12px 12px 32px;
    }
    .colonnes {
      grid-template-columns: minmax(0, 1fr);
    }
    .resume {
      position: static;
    }
    .questions {
      padding: 16px;
      border-radius: 24px;
    }
    .legumes {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
  }
</style>
