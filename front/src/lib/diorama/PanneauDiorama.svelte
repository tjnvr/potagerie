<script lang="ts">
  // The planche panel (E06) opened from the garden view. Same panel as in the
  // atelier; here it only edits through its fields (no drag), so the move and
  // resize go straight to the commands, with the same warning before a
  // shorter planche drops series (S03 §4).
  import { commands, type PlancheView, type YearView } from "../bindings";
  import PanneauPlanche from "../atelier/PanneauPlanche.svelte";
  import type { Geo, Reponse } from "../atelier/glisser";
  import Confirmation from "../coquille/Confirmation.svelte";
  import { appli } from "../etat.svelte";
  import { m } from "../paraglide/messages.js";

  let { vue, planche, onfermer }: { vue: YearView; planche: PlancheView; onfermer: () => void } = $props();

  let raccourcissement = $state<{ nombre: number; repondre: (ok: boolean) => void } | null>(null);

  async function modifier(id: string, geo: Geo, nature: "deplacer" | "redimensionner"): Promise<Reponse> {
    if (nature === "redimensionner" && geo.longueur_cm < planche.longueur_cm) {
      const resultat = await commands.seriesDepassant(appli.annee, id, geo.longueur_cm);
      if (resultat.status === "ok" && resultat.data.length > 0) {
        const ok = await new Promise<boolean>((repondre) => {
          raccourcissement = { nombre: resultat.data.length, repondre };
        });
        raccourcissement = null;
        if (!ok) return "annule";
      }
    }
    const ok =
      nature === "deplacer"
        ? await appli.modifier((annee) => commands.deplacerPlanche(annee, id, geo.x, geo.y))
        : await appli.modifier((annee) =>
            commands.modifierPlanche(annee, id, null, { planche_type: planche.planche_type, ...geo }),
          );
    return ok ? "ok" : "refus";
  }
</script>

<PanneauPlanche {vue} {planche} {onfermer} onmodifier={modifier} />

{#if raccourcissement}
  {@const demande = raccourcissement}
  <Confirmation
    titre={m.raccourcir_titre({ nom: planche.nom })}
    confirmer={m.raccourcir_bouton()}
    ton="alerte"
    onconfirmer={() => demande.repondre(true)}
    onannuler={() => demande.repondre(false)}
  >
    <p>{m.raccourcir_texte({ nombre: demande.nombre })}</p>
  </Confirmation>
{/if}
