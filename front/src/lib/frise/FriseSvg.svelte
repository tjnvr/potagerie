<script lang="ts" module>
  /** Layout constants, shared with the controller (pointer to row, export). */
  export const LARGEUR_LIBELLES = 150;
  export const LARGEUR_LIBELLES_ETROIT = 84;
  export const HAUTEUR_ENTETE = 62;
  export const HAUTEUR_VOIE = 34;
  export const MARGE_LIGNE = 10;

  export function hauteurLigne(voies: number): number {
    return voies * HAUTEUR_VOIE + MARGE_LIGNE;
  }
</script>

<script lang="ts">
  // The drawing of the timeline (S01 F-VIS-2, D01 E05): months, one row per
  // planche, bars cut into phases, free slots, today and the cursor.
  //
  // It is a standalone SVG on purpose (S05 §4, export): every colour, font and
  // pattern is an attribute of the drawing, never a CSS rule, so the same
  // markup serialised by `serialiserFrise` renders alike in resvg. CSS here only
  // adds the pointer feedback (cursor, focus), which the export drops.
  import { m } from "../paraglide/messages.js";
  import { dateAbregee, dateDe, jourDe } from "../dates";
  import { enMetres, nomCulture, nomDeType } from "../planches";
  import { PALETTE, couleurCulture, melange } from "./couleurs";
  import { moisVisibles, xDe, type Barre, type Echelle, type Ligne } from "./geometrie";

  let {
    lignes,
    echelle,
    largeurLibelles = LARGEUR_LIBELLES,
    aujourdhui,
    curseur = null,
    interactif = true,
    decalages = {},
    refus = null,
    portee = null,
    anneeAffichee,
    selectionPlancheId = null,
  }: {
    lignes: Ligne[];
    echelle: Echelle;
    largeurLibelles?: number;
    /** `YYYY-MM-DD`. */
    aujourdhui: string;
    /** The cursor of the time, or `null` (export). */
    curseur?: string | null;
    interactif?: boolean;
    /** Pixel offset of the bars being dragged, by id. */
    decalages?: Record<string, number>;
    /** The bar Rust refused to move: outlined for a moment. */
    refus?: string | null;
    /** The bar carried by a finger or a mouse: lifted. */
    portee?: string | null;
    anneeAffichee?: number;
    selectionPlancheId?: string | null;
  } = $props();

  const largeur = $derived(echelle.largeur + largeurLibelles);
  const jourAujourdhui = $derived(jourDe(aujourdhui));
  const jourCurseur = $derived(curseur ? jourDe(curseur) : null);

  const positions = $derived.by(() => {
    let y = HAUTEUR_ENTETE;
    return lignes.map((ligne) => {
      const haut = y;
      const hauteur = hauteurLigne(ligne.voies);
      y += hauteur + 1;
      return { haut, hauteur };
    });
  });
  const hauteurTotale = $derived(positions.length ? positions[positions.length - 1]!.haut + positions[positions.length - 1]!.hauteur + 1 : HAUTEUR_ENTETE + 1);

  const mois = $derived(moisVisibles(echelle));
  const compact = $derived(largeurLibelles < LARGEUR_LIBELLES);
  /** Month names that fit their column: three letters, one letter when narrow, none when tiny. */
  const nomsDeMois = $derived(
    mois.map((mo) => {
      const place = mo.jours * echelle.pxParJour - 8;
      if (place < 9) return "";
      const nom = new Date(Date.UTC(mo.annee, mo.mois, 1)).toLocaleDateString("fr-FR", { month: "short", timeZone: "UTC" }).replace(".", "");
      return place < 34 ? nom.slice(0, 1) : nom;
    }),
  );

  /** Years to label: the visible part of each one, if it is wide enough to read. */
  const annees = $derived.by(() => {
    const visibles = [...new Set(mois.map((mo) => mo.annee))];
    return visibles
      .map((annee) => {
        const debut = xDe(echelle, jourDe(`${annee}-01-01`));
        const fin = xDe(echelle, jourDe(`${annee + 1}-01-01`));
        const gauche = Math.max(debut, 0);
        return { annee, x: Math.min(gauche + 6, fin - 48) + largeurLibelles, visible: Math.min(fin, echelle.largeur) - gauche };
      })
      .filter((a) => a.visible >= 54);
  });

  const motifs = $derived([...new Set(lignes.flatMap((l) => l.barres.map((b) => b.cultureId)))]);

  function x(jour: number): number {
    return xDe(echelle, jour) + largeurLibelles;
  }

  function nomDeBarre(b: Barre): string {
    return nomCulture(b.cultureId);
  }

  function titreDeBarre(b: Barre): string {
    const fin = dateAbregee(dateDe(b.fin - 1), anneeAffichee);
    const base = m.frise_barre_titre({ culture: nomDeBarre(b), debut: dateAbregee(dateDe(b.debut), anneeAffichee), fin });
    return [base, b.finManuelle ? m.frise_fin_manuelle() : "", b.enConflit ? m.frise_conflit() : ""].filter(Boolean).join(" · ");
  }

  function dureeJours(fin: number, debut: number): number {
    return fin - debut;
  }
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={largeur}
  height={hauteurTotale}
  viewBox="0 0 {largeur} {hauteurTotale}"
  font-family="Figtree, system-ui, sans-serif"
  role="group"
  aria-label={m.frise_aria()}
  data-frise
>
  <defs>
    <clipPath id="frise-zone"><rect x={largeurLibelles} y="0" width={echelle.largeur} height={hauteurTotale} /></clipPath>
    <clipPath id="frise-libelles"><rect x="0" y="0" width={largeurLibelles - 8} height={hauteurTotale} /></clipPath>
    {#each motifs as culture (culture)}
      {@const couleur = couleurCulture(culture)}
      <!-- Pots: diagonal stripes. Growth: pale with dots. Harvest: solid. The
           phases differ by pattern, not only by colour (S01 F-VIS-2). -->
      <pattern id="frise-godets-{culture}" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
        <rect width="8" height="8" fill={melange(couleur, 0.85)} />
        <rect width="4" height="8" fill={melange(couleur, 0.55)} />
      </pattern>
      <pattern id="frise-croissance-{culture}" width="8" height="8" patternUnits="userSpaceOnUse">
        <rect width="8" height="8" fill={melange(couleur, 0.45)} />
        <circle cx="4" cy="4" r="0.9" fill={melange(couleur, 0.2)} />
      </pattern>
    {/each}
    {#each lignes as ligne, i (ligne.planche.id)}
      {#each ligne.barres as b (b.id)}
        <clipPath id="frise-barre-{b.id}">
          <rect
            x={x(b.debut) + (decalages[b.id] ?? 0)}
            y={positions[i]!.haut + b.voie * HAUTEUR_VOIE + 8}
            width={Math.max(dureeJours(b.fin, b.debut) * echelle.pxParJour, 1)}
            height="26"
            rx="9"
          />
        </clipPath>
      {/each}
    {/each}
    <pattern id="frise-absence" width="10" height="10" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
      <rect width="10" height="10" fill={PALETTE.absence} />
      <rect width="1.5" height="10" fill={PALETTE.trait} />
    </pattern>
  </defs>

  <rect width={largeur} height={hauteurTotale} fill={PALETTE.fond} />

  <g clip-path="url(#frise-zone)">
    <!-- Rows: background, greyed periods where the planche does not exist. -->
    {#each lignes as ligne, i (ligne.planche.id)}
      {@const p = positions[i]!}
      <rect x={largeurLibelles} y={p.haut} width={echelle.largeur} height={p.hauteur} fill={ligne.planche.id === selectionPlancheId ? PALETTE.selection : PALETTE.fond} />
      {#each ligne.absences as a (a.debut)}
        <rect x={x(a.debut)} y={p.haut} width={Math.max((a.fin - a.debut) * echelle.pxParJour, 0)} height={p.hauteur} fill="url(#frise-absence)" />
      {/each}
      <line x1={largeurLibelles} x2={largeur} y1={p.haut - 0.5} y2={p.haut - 0.5} stroke={PALETTE.trait} />
    {/each}

    <!-- Month columns. -->
    {#each mois as mo, i (mo.jour)}
      <line x1={x(mo.jour)} x2={x(mo.jour)} y1={mo.mois === 0 ? 20 : 40} y2={hauteurTotale} stroke={PALETTE.trait} stroke-width={mo.mois === 0 ? 2 : 1} />
      <text x={x(mo.jour) + 4} y={HAUTEUR_ENTETE - 8} font-size="11" font-weight="600" letter-spacing="0.4" fill={PALETTE.discret}>{nomsDeMois[i]?.toUpperCase()}</text>
    {/each}
    {#each annees as a (a.annee)}
      <text x={a.x} y="38" font-size="18" font-weight="600" font-family="Fredoka, Figtree, sans-serif" fill={PALETTE.encre}>{a.annee}</text>
    {/each}

    <!-- Free slots (S03 §8): dashed, with the crops Rust suggests. -->
    {#each lignes as ligne, i (ligne.planche.id)}
      {@const p = positions[i]!}
      {#each ligne.creneaux as c (c.cle)}
        {@const largeurC = (c.fin - c.debut) * echelle.pxParJour}
        {@const jours = c.fin - c.debut}
        {@const partiel = c.source.troncon_debut !== 1 || c.source.troncon_fin !== ligne.planche.troncons}
        {@const noms = c.source.suggestions.map((s) => nomCulture(s.culture_id)).join(", ")}
        <g
          class="creneau"
          data-creneau={c.cle}
          role="button"
          tabindex={interactif ? 0 : undefined}
          aria-label={m.frise_creneau_aria({ planche: ligne.planche.nom, jours, debut: dateAbregee(c.source.start, anneeAffichee), fin: dateAbregee(c.source.end, anneeAffichee) })}
        >
          <rect
            x={x(c.debut) + 1}
            y={p.haut + c.voie * HAUTEUR_VOIE + 8}
            width={Math.max(largeurC - 2, 0)}
            height="26"
            rx="9"
            fill={PALETTE.creneauFond}
            stroke={PALETTE.accent}
            stroke-width="2"
            stroke-dasharray="6 4"
          />
          {#if x(c.fin) - Math.max(x(c.debut), largeurLibelles) > 70}
          <svg x={Math.max(x(c.debut), largeurLibelles) + 10} y={p.haut + c.voie * HAUTEUR_VOIE + 8} width={Math.max(x(c.fin) - Math.max(x(c.debut), largeurLibelles) - 14, 0)} height="26" overflow="hidden">
            <text y="17" font-size="12" fill={PALETTE.accentEncre}>
              <tspan font-weight="700">{m.frise_creneau({ jours })}</tspan>
              {#if partiel}<tspan> · {m.frise_creneau_troncons({ debut: c.source.troncon_debut, fin: c.source.troncon_fin })}</tspan>{/if}
              {#if noms}<tspan> · {m.frise_creneau_suggestions({ cultures: noms })}</tspan>{/if}
            </text>
          </svg>
          {/if}
        </g>
      {/each}
    {/each}

    <!-- Bars. -->
    {#each lignes as ligne, i (ligne.planche.id)}
      {@const p = positions[i]!}
      {#each ligne.barres as b (b.id)}
        {@const dx = decalages[b.id] ?? 0}
        {@const haut = p.haut + b.voie * HAUTEUR_VOIE + 8}
        {@const gauche = x(b.debut) + dx}
        {@const largeurB = Math.max((b.fin - b.debut) * echelle.pxParJour, 1)}
        <g
          class="barre"
          class:portee={portee === b.id}
          data-barre={b.id}
          role="button"
          tabindex={interactif ? 0 : undefined}
          aria-label={titreDeBarre(b)}
        >
          <title>{titreDeBarre(b)}</title>
          <rect x={gauche} y={haut + 1} width={largeurB} height="26" rx="9" fill={PALETTE.encre} opacity={portee === b.id ? 0.25 : 0.12} />
          <g clip-path="url(#frise-barre-{b.id})" transform="translate(0 0)">
            {#each b.segments as s (s.phase)}
              <rect
                x={x(s.debut) + dx}
                y={haut}
                width={Math.max((s.fin - s.debut) * echelle.pxParJour, 0)}
                height="26"
                fill={s.phase === "godets" ? `url(#frise-godets-${b.cultureId})` : s.phase === "croissance" ? `url(#frise-croissance-${b.cultureId})` : couleurCulture(b.cultureId)}
              />
            {/each}
          </g>
          <rect
            x={gauche + 1}
            y={haut + 1}
            width={Math.max(largeurB - 2, 0)}
            height="24"
            rx="8"
            fill="none"
            stroke={refus === b.id || b.enConflit ? PALETTE.crit : "none"}
            stroke-width="3"
          />
          <svg x={Math.max(gauche, largeurLibelles) + 9} y={haut} width={Math.max(gauche + largeurB - Math.max(gauche, largeurLibelles) - 12, 0)} height="26" overflow="hidden">
            {#if b.mode === "plants"}
              <!-- The « plant » icon of bought plants (F-VIS-2). -->
              <g fill="none" stroke={PALETTE.encre} stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" transform="translate(0 6)">
                <path d="M7 14V7M7 8C7 5 5 3.5 2.5 3.5 2.5 6 4 8 7 8ZM7 7C7 4.5 8.8 3 11.5 3 11.5 5.5 10 7 7 7Z" />
              </g>
            {/if}
            <text x={b.mode === "plants" ? 18 : 0} y="17" font-size="12" font-weight="600" fill={PALETTE.encre}>{nomDeBarre(b)}{#if b.finManuelle}<tspan font-weight="400">  ✎</tspan>{/if}</text>
          </svg>
        </g>
      {/each}
    {/each}

    <!-- Today (background marker) and the cursor of the time (F-VIS-3). -->
    <line x1={x(jourAujourdhui)} x2={x(jourAujourdhui)} y1="40" y2={hauteurTotale} stroke={PALETTE.accent} stroke-width="1.5" stroke-dasharray="3 3" />
    {#if jourCurseur !== null}
      <g data-curseur>
        <line x1={x(jourCurseur)} x2={x(jourCurseur)} y1="16" y2={hauteurTotale} stroke={PALETTE.accent} stroke-width="2" />
        <rect x={x(jourCurseur) - 34} y="0" width="68" height="20" rx="10" fill={PALETTE.accent} />
        <text x={x(jourCurseur)} y="14" font-size="11" font-weight="700" text-anchor="middle" fill={PALETTE.accentEncre}>{dateAbregee(dateDe(jourCurseur), anneeAffichee)}</text>
      </g>
    {/if}
  </g>

  <!-- Labels of the rows, drawn over what scrolls under them. -->
  <rect x="0" y="0" width={largeurLibelles} height={hauteurTotale} fill={PALETTE.fond} />
  <g clip-path="url(#frise-libelles)">
    {#each lignes as ligne, i (ligne.planche.id)}
      {@const p = positions[i]!}
      {#if ligne.planche.id === selectionPlancheId}
        <rect x="0" y={p.haut} width={largeurLibelles} height={p.hauteur} fill={PALETTE.selection} />
        <rect x="0" y={p.haut} width="3" height={p.hauteur} fill={PALETTE.accent} />
      {/if}
      <line x1="0" x2={largeurLibelles} y1={p.haut - 0.5} y2={p.haut - 0.5} stroke={PALETTE.trait} />
      <text x={compact ? 8 : 12} y={p.haut + p.hauteur / 2 + (compact ? 4 : -2)} font-size={compact ? 12 : 14} font-weight="600" fill={PALETTE.encre}>{ligne.planche.nom}</text>
      {#if !compact}
        <text x="12" y={p.haut + p.hauteur / 2 + 15} font-size="12" fill={PALETTE.discret}>
          {m.frise_planche_dim({
            type: nomDeType(ligne.planche.planche_type),
            longueur: enMetres(ligne.planche.longueur_cm),
            largeur: enMetres(ligne.planche.largeur_cm),
          })}
        </text>
      {/if}
    {/each}
  </g>
  <line x1="0" x2={largeur} y1={hauteurTotale - 0.5} y2={hauteurTotale - 0.5} stroke={PALETTE.trait} />
</svg>

<style>
  .barre,
  .creneau {
    outline: none;
  }
  .barre {
    cursor: grab;
  }
  .barre.portee {
    cursor: grabbing;
  }
  .creneau {
    cursor: pointer;
  }
  .barre:focus-visible rect:last-of-type,
  .creneau:focus-visible rect {
    stroke: #1f3a2e;
    stroke-width: 3;
  }
  .barre:hover:not(.portee) {
    filter: brightness(0.97);
  }
</style>
