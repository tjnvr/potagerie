<script lang="ts">
  // The continuous timeline of the cultures (S01 F-VIS-2, D01 E05): one row per
  // planche, scrolling through time without a break between years, the bars
  // that can be dragged (Pointer Events, long press on touch), the free slots,
  // the cursor of the time.
  //
  // This component only handles the gestures and the loading of the year
  // views around the visible days. What is drawn comes from the views of
  // Rust (`FriseSvg`), and a drag is only a proposal: Rust judges (S03 T12).
  import { untrack } from "svelte";
  import { get } from "svelte/store";
  import { ControleurGlisser, type Reponse, type TypePointeur } from "../atelier/glisser";
  import { commands, type YearView } from "../bindings";
  import { anneeDe, dateDe, jourDe } from "../dates";
  import { traduireErreur } from "../erreurs";
  import { appeler, appli } from "../etat.svelte";
  import { m } from "../paraglide/messages.js";
  import { allerA, curseur, recentrage } from "../temps";
  import { decalageAffiche, deposerBarre } from "./barre";
  import { PALETTE, couleurCulture, melange } from "./couleurs";
  import FriseSvg, { LARGEUR_LIBELLES, LARGEUR_LIBELLES_ETROIT } from "./FriseSvg.svelte";
  import {
    JOURS_PAR_DEFAUT_ETROIT,
    SEUIL_DEFILEMENT_PX,
    anneesVisibles,
    centrerSur,
    construireLignes,
    defiler,
    echelleCentree,
    garderVisible,
    jourSousLePointeur,
    joursVisibles,
    zoomer,
    type Echelle,
  } from "./geometrie";
  import IconeFrise from "./IconeFrise.svelte";

  let { vue }: { vue: YearView } = $props();

  let largeurTotale = $state(0);
  let echelle = $state<Echelle | null>(null);
  let replie = $state(false);
  let afficherAnnotations = $state(true);
  let corps: HTMLElement | undefined = $state();

  const etroit = $derived(largeurTotale > 0 && largeurTotale < 560);
  const largeurLibelles = $derived(etroit ? LARGEUR_LIBELLES_ETROIT : LARGEUR_LIBELLES);
  const largeurZone = $derived(Math.max(largeurTotale - largeurLibelles, 120));

  const reduit = () => matchMedia("(prefers-reduced-motion: reduce)").matches;

  // ---------- Scale: opens on 12 months around the cursor ----------

  $effect(() => {
    if (largeurTotale <= 0) return;
    untrack(() => {
      if (!echelle) {
        echelle = echelleCentree(largeurZone, jourDe(get(curseur)), etroit ? JOURS_PAR_DEFAUT_ETROIT : undefined);
      } else if (echelle.largeur !== largeurZone) {
        echelle = { ...echelle, largeur: largeurZone, pxParJour: largeurZone / joursVisibles(echelle) };
      }
    });
  });

  // The cursor moved elsewhere (diorama, playback, ‹ ›): scroll to keep it in
  // view, and show the season it is in (F-PLA-4).
  $effect(() => {
    const date = $curseur;
    untrack(() => {
      if (echelle) echelle = garderVisible(echelle, jourDe(date));
      if (anneeDe(date) !== appli.annee) appli.charger(anneeDe(date));
    });
  });

  // « Aujourd'hui »: back to the middle of the timeline, even if the cursor did not move.
  $effect(() => {
    if ($recentrage === 0) return;
    untrack(() => {
      if (echelle) echelle = centrerSur(echelle, jourDe(get(curseur)));
    });
  });

  // ---------- The year views around the visible days ----------

  let chargees = $state<Record<number, YearView>>({});
  const enCours = new Set<number>();
  let jeton = 0;

  const annees = $derived(echelle ? anneesVisibles(echelle) : [vue.year]);
  const vues = $derived(annees.map((a) => (a === vue.year ? vue : chargees[a])).filter((v): v is YearView => !!v));
  const lignes = $derived(construireLignes(vues, vue.year));
  /** Notes and periods of the loaded years, each once (a period can touch two years). */
  const annotations = $derived([...new Map(vues.flatMap((v) => v.annotations).map((a) => [a.id, a])).values()]);
  const aucuneSerie = $derived(vues.every((v) => v.series.length === 0));

  // A change of the plan (a command returned a new view): the other years are stale.
  $effect(() => {
    void vue.version;
    void vue.year;
    untrack(() => {
      jeton++;
      enCours.clear();
      chargees = {};
    });
  });

  $effect(() => {
    for (const annee of annees) {
      if (annee === vue.year || chargees[annee] || enCours.has(annee)) continue;
      enCours.add(annee);
      const mon = jeton;
      void appeler(() => commands.vueAnnee(annee)).then((r) => {
        if (mon !== jeton) return;
        enCours.delete(annee);
        if (r.status === "ok") chargees[annee] = r.data;
      });
    }
  });

  // ---------- Gestures ----------

  type Geste =
    | { type: "pan"; depart: Echelle; x0: number; bouge: boolean; cible: Element | null }
    | { type: "barre"; id: string; x0: number; dx: number; envoi: boolean }
    | { type: "curseur" };

  let geste: Geste | null = $state(null);
  let decalages = $state<Record<string, number>>({});
  let portee = $state<string | null>(null);
  let refus = $state<string | null>(null);
  const controleur = new ControleurGlisser({ surArme: () => (portee = geste?.type === "barre" ? geste.id : null) });
  const pointeurs = new Map<number, { x: number; y: number }>();
  let distancePincement = 0;

  function gauche(): number {
    return (corps?.getBoundingClientRect().left ?? 0) + largeurLibelles;
  }

  function enZone(e: PointerEvent): boolean {
    return e.clientX >= gauche();
  }

  function bas(e: PointerEvent) {
    if (!echelle || geste?.type === "barre" && geste.envoi) return;
    if (e.pointerType === "mouse" && e.button !== 0) return;
    pointeurs.set(e.pointerId, { x: e.clientX, y: e.clientY });
    corps?.setPointerCapture(e.pointerId);
    if (pointeurs.size === 2) {
      controleur.annuler();
      geste = null;
      portee = null;
      const [a, b] = [...pointeurs.values()] as [{ x: number; y: number }, { x: number; y: number }];
      distancePincement = Math.hypot(a.x - b.x, a.y - b.y);
      return;
    }
    const cible = e.target as Element;
    if (cible.closest("[data-curseur]")) {
      geste = { type: "curseur" };
      return;
    }
    const barre = cible.closest("[data-barre]");
    if (barre && enZone(e)) {
      geste = { type: "barre", id: barre.getAttribute("data-barre")!, x0: e.clientX, dx: 0, envoi: false };
      portee = null;
      controleur.appui(e.pointerType as TypePointeur, { x: e.clientX, y: e.clientY });
      return;
    }
    geste = { type: "pan", depart: echelle, x0: e.clientX, bouge: false, cible };
  }

  function bouge(e: PointerEvent) {
    if (!echelle || !pointeurs.has(e.pointerId)) return;
    pointeurs.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (pointeurs.size === 2) {
      const [a, b] = [...pointeurs.values()] as [{ x: number; y: number }, { x: number; y: number }];
      const distance = Math.hypot(a.x - b.x, a.y - b.y);
      if (distancePincement > 0 && distance > 0) {
        echelle = zoomer(echelle, distancePincement / distance, (a.x + b.x) / 2 - gauche());
      }
      distancePincement = distance;
      return;
    }
    if (!geste) return;
    if (geste.type === "curseur") {
      allerA(dateDe(jourSousLePointeur(echelle, e.clientX, gauche())));
    } else if (geste.type === "barre") {
      if (geste.envoi) return;
      const mouvement = controleur.deplacer({ x: e.clientX, y: e.clientY });
      if (mouvement === "abandon") {
        // A finger that moves before the long press scrolls the timeline.
        geste = { type: "pan", depart: echelle, x0: geste.x0, bouge: true, cible: null };
        portee = null;
        echelle = defiler(echelle, e.clientX - geste.x0);
      } else if (mouvement === "debut" || mouvement === "glisse") {
        portee = geste.id;
        geste.dx = e.clientX - geste.x0;
        decalages = { [geste.id]: decalageAffiche(geste.dx, echelle.pxParJour) };
      }
    } else {
      const dx = e.clientX - geste.x0;
      if (!geste.bouge && Math.abs(dx) <= SEUIL_DEFILEMENT_PX) return;
      geste.bouge = true;
      echelle = defiler(geste.depart, dx);
    }
  }

  async function leve(e: PointerEvent) {
    pointeurs.delete(e.pointerId);
    if (corps?.hasPointerCapture(e.pointerId)) corps.releasePointerCapture(e.pointerId);
    distancePincement = 0;
    const courant = geste;
    if (!courant || !echelle) return;
    if (courant.type === "curseur") {
      geste = null;
      return;
    }
    if (courant.type === "pan") {
      geste = null;
      if (courant.bouge || !enZone(e) && !courant.cible?.closest("[data-creneau],[data-annotation]")) return;
      const creneau = courant.cible?.closest("[data-creneau]");
      const note = courant.cible?.closest("[data-annotation]");
      if (note) ouvrirAnnotation(note.getAttribute("data-annotation")!);
      else if (creneau) ouvrirCreneau(creneau.getAttribute("data-creneau")!);
      else allerA(dateDe(jourSousLePointeur(echelle, e.clientX, gauche())));
      return;
    }
    if (courant.envoi) return;
    const fin = controleur.relacher();
    portee = null;
    if (fin === "clic") {
      geste = null;
      ouvrirBarre(courant.id);
      return;
    }
    if (fin !== "fin") {
      geste = null;
      decalages = {};
      return;
    }
    courant.envoi = true;
    await decaler(courant.id, courant.dx, echelle.pxParJour);
    geste = null;
  }

  function annule(e: PointerEvent) {
    pointeurs.delete(e.pointerId);
    controleur.annuler();
    portee = null;
    if (geste?.type === "barre" && !geste.envoi) {
      geste = null;
      decalages = {};
    } else if (geste?.type !== "barre") {
      geste = null;
    }
  }

  /** Drops the bar: Rust accepts the shift, or refuses and the bar goes back. */
  async function decaler(id: string, dxPx: number, pxParJour: number) {
    const issue = await deposerBarre(dxPx, pxParJour, async (jours): Promise<Reponse> => {
      const ok = await appli.modifier((annee) => commands.decalerSerie(annee, id, jours));
      return ok ? "ok" : "refus";
    });
    if (issue.reponse === "refus") {
      const raison = appli.dernierRefus ? traduireErreur(appli.dernierRefus) : "";
      appli.message = { texte: m.frise_refus({ raison }), ton: "erreur" };
      refus = id;
      setTimeout(() => (refus = null), 1500);
      await ramener(id, decalageAffiche(dxPx, pxParJour));
    } else {
      decalages = {};
    }
  }

  /** The bar goes back to its place; at once if the system asks to reduce motion. */
  async function ramener(id: string, depuis: number) {
    if (reduit()) {
      decalages = {};
      return;
    }
    const duree = 180;
    const debut = performance.now();
    await new Promise<void>((fini) => {
      const pas = (t: number) => {
        const p = Math.min(1, (t - debut) / duree);
        decalages = { [id]: depuis * (1 - (1 - (1 - p) * (1 - p))) };
        if (p < 1) requestAnimationFrame(pas);
        else fini();
      };
      requestAnimationFrame(pas);
    });
    decalages = {};
  }

  function ouvrirBarre(id: string) {
    const serie = [...vues.flatMap((v) => v.series), ...vues.flatMap((v) => v.historique)].find((s) => s.id === id);
    if (serie) appli.serieOuverte = { type: "edition", serie };
  }

  function ouvrirAnnotation(id: string) {
    const annotation = annotations.find((a) => a.id === id);
    if (annotation) appli.noteOuverte = { type: "edition", annotation };
  }

  function annoter() {
    appli.noteOuverte = { type: "nouvelle", date: get(curseur), plancheId: null };
  }

  function ouvrirCreneau(cle: string) {
    const ligne = lignes.find((l) => l.creneaux.some((c) => c.cle === cle));
    const creneau = ligne?.creneaux.find((c) => c.cle === cle);
    if (!ligne || !creneau) return;
    appli.serieOuverte = {
      type: "nouvelle",
      plancheId: ligne.planche.id,
      creneau: creneau.source,
      suggestion: creneau.source.suggestions[0],
    };
  }

  function touche(e: KeyboardEvent) {
    const cible = e.target as Element;
    const barre = cible.closest("[data-barre]");
    const creneau = cible.closest("[data-creneau]");
    const note = cible.closest("[data-annotation]");
    if (e.key === "Enter" || e.key === " ") {
      if (note) ouvrirAnnotation(note.getAttribute("data-annotation")!);
      else if (barre) ouvrirBarre(barre.getAttribute("data-barre")!);
      else if (creneau) ouvrirCreneau(creneau.getAttribute("data-creneau")!);
      else return;
      e.preventDefault();
    } else if (barre && (e.key === "ArrowLeft" || e.key === "ArrowRight") && echelle && !geste) {
      // The keyboard shifts a bar by a day, or a week with Shift.
      e.preventDefault();
      const jours = (e.key === "ArrowLeft" ? -1 : 1) * (e.shiftKey ? 7 : 1);
      void decaler(barre.getAttribute("data-barre")!, jours * echelle.pxParJour, echelle.pxParJour);
    }
  }

  // The wheel: with Ctrl (or a trackpad pinch) it zooms, sideways it scrolls.
  // The listeners are not passive: they stop the page from scrolling too.
  $effect(() => {
    const el = corps;
    if (!el) return;
    const roue = (e: WheelEvent) => {
      if (!echelle) return;
      const dedans = e.clientX - gauche();
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        echelle = zoomer(echelle, Math.exp(e.deltaY * 0.0015), dedans);
      } else if (Math.abs(e.deltaX) > Math.abs(e.deltaY) || e.shiftKey) {
        e.preventDefault();
        echelle = defiler(echelle, -(e.deltaX || e.deltaY));
      }
    };
    const toucher = (e: TouchEvent) => {
      if (controleur.etat === "arme" || controleur.etat === "glisse") e.preventDefault();
    };
    el.addEventListener("wheel", roue, { passive: false });
    el.addEventListener("touchmove", toucher, { passive: false });
    return () => {
      el.removeEventListener("wheel", roue);
      el.removeEventListener("touchmove", toucher);
    };
  });

  function aller(sens: -1 | 1) {
    if (echelle) echelle = defiler(echelle, -sens * echelle.largeur * 0.33);
  }

  function zoom(facteur: number) {
    if (echelle) echelle = zoomer(echelle, facteur, echelle.largeur / 2);
  }
</script>

<section class="frise" aria-labelledby="frise-titre">
  <div class="tete">
    <div class="titres">
      <h2 id="frise-titre">{m.frise_titre()}</h2>
      <p>{m.frise_aide()}</p>
    </div>
    <ul class="legende" aria-label={m.frise_titre()}>
      <li>
        <svg width="26" height="14" aria-hidden="true">
          <defs>
            <pattern id="legende-godets" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
              <rect width="8" height="8" fill={melange(couleurCulture("tomate"), 0.85)} />
              <rect width="4" height="8" fill={melange(couleurCulture("tomate"), 0.55)} />
            </pattern>
          </defs>
          <rect width="26" height="14" rx="4" fill="url(#legende-godets)" />
        </svg>
        {m.frise_legende_godets()}
      </li>
      <li>
        <svg width="26" height="14" aria-hidden="true">
          <defs>
            <pattern id="legende-croissance" width="8" height="8" patternUnits="userSpaceOnUse">
              <rect width="8" height="8" fill={melange(couleurCulture("tomate"), 0.45)} />
              <circle cx="4" cy="4" r="0.9" fill={melange(couleurCulture("tomate"), 0.2)} />
            </pattern>
          </defs>
          <rect width="26" height="14" rx="4" fill="url(#legende-croissance)" />
        </svg>
        {m.frise_legende_croissance()}
      </li>
      <li>
        <svg width="26" height="14" aria-hidden="true"><rect width="26" height="14" rx="4" fill={couleurCulture("tomate")} /></svg>
        {m.frise_legende_recolte()}
      </li>
      <li>
        <svg width="26" height="14" aria-hidden="true">
          <rect x="1" y="1" width="24" height="12" rx="4" fill={PALETTE.creneauFond} stroke={PALETTE.accent} stroke-width="1.5" stroke-dasharray="4 3" />
        </svg>
        {m.frise_legende_creneau()}
      </li>
    </ul>
    <div class="outils">
      <button class="rond" aria-label={m.frise_precedent()} title={m.frise_precedent()} onclick={() => aller(-1)}><IconeFrise nom="avant" /></button>
      <button class="rond" aria-label={m.frise_suivant()} title={m.frise_suivant()} onclick={() => aller(1)}><IconeFrise nom="apres" /></button>
      <button class="rond" aria-label={m.frise_zoom_avant()} title={m.frise_zoom_avant()} onclick={() => zoom(0.7)}><IconeFrise nom="plus" /></button>
      <button class="rond" aria-label={m.frise_zoom_arriere()} title={m.frise_zoom_arriere()} onclick={() => zoom(1 / 0.7)}><IconeFrise nom="moins" /></button>
      <button class="bouton" onclick={annoter}><IconeFrise nom="plus" taille={16} />{m.note_annoter()}</button>
      <button class="bouton" onclick={() => (appli.exportOuvert = true)}>{m.export_ouvrir()}</button>
      <label class="case"><input type="checkbox" bind:checked={afficherAnnotations} />{m.note_afficher()}</label>
      <button class="bouton" aria-expanded={!replie} onclick={() => (replie = !replie)}>
        {replie ? m.frise_deplier() : m.frise_replier()}
        <IconeFrise nom={replie ? "bas" : "haut"} taille={16} />
      </button>
    </div>
  </div>

  {#if !replie}
    {#if lignes.length === 0}
      <p class="vide">{m.frise_vide_planches()}</p>
    {:else}
      {#if aucuneSerie}
        <p class="vide leger">{m.frise_vide()}</p>
      {/if}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="corps"
        bind:this={corps}
        bind:clientWidth={largeurTotale}
        onpointerdown={bas}
        onpointermove={bouge}
        onpointerup={leve}
        onpointercancel={annule}
        onkeydown={touche}
      >
        {#if echelle}
          <FriseSvg
            {lignes}
            {echelle}
            {largeurLibelles}
            aujourdhui={vue.today}
            curseur={$curseur}
            {decalages}
            {refus}
            {portee}
            anneeAffichee={vue.year}
            annotations={afficherAnnotations ? annotations : []}
          />
        {/if}
      </div>
      <p class="aide">{m.frise_aide_barre()}</p>
    {/if}
  {/if}
</section>

<style>
  .frise {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 22px 24px 18px;
    border-radius: 32px;
    background: var(--white);
  }

  .tete {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px 20px;
  }

  .titres {
    flex: 1 1 260px;
  }

  h2 {
    font-size: 26px;
  }

  .titres p,
  .aide {
    margin: 4px 0 0;
    color: var(--muted);
    font-size: 13px;
  }

  .legende {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    margin: 0;
    padding: 0;
    list-style: none;
    font-size: 13px;
    color: var(--muted);
  }

  .legende li {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .outils {
    display: flex;
    align-items: center;
    gap: 8px;
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

  .case {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    color: var(--muted);
  }

  .bouton {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-height: 40px;
    padding: 0 14px;
    font-size: 14px;
  }

  .corps {
    overflow: hidden;
    border-radius: 12px;
    /* Horizontal swipes scroll the timeline; vertical ones scroll the page. */
    touch-action: pan-y;
    user-select: none;
    -webkit-user-select: none;
  }

  .vide {
    margin: 0;
    padding: 48px 20px;
    border: 1px dashed var(--line);
    border-radius: 16px;
    background: var(--white);
    text-align: center;
    color: var(--muted);
  }

  .vide.leger {
    padding: 10px 16px;
    font-size: 14px;
  }

  @media (max-width: 720px) {
    .frise {
      padding: 16px 12px 14px;
      border-radius: 24px;
    }
    h2 {
      font-size: 22px;
    }
  }
</style>
