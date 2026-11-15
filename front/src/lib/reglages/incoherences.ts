// French text of the incoherences listed for an imported plan (S03 §13).
// Rust decides what is incoherent; this only writes it.
import type { IncoherenceView } from "../bindings";
import { m } from "../paraglide/messages.js";
import { nomCulture } from "../planches";

/** « Tomate » for a catalogue crop, the raw id for an unknown one. */
function culture(id: string | null): string {
  if (!id) return "?";
  return nomCulture(id);
}

export function texteIncoherence(i: IncoherenceView): string {
  const planche = i.planche_nom ?? "?";
  const autre = i.autre_planche_nom ?? "?";
  const annee = i.annee ?? 0;
  const nombre = i.nombre ?? 0;
  switch (i.code) {
    case "trop_de_planches":
      return m.import_i_trop_de_planches({ annee, nombre });
    case "serie_orpheline":
      return i.annee === null
        ? m.import_i_serie_orpheline_inconnue({ culture: culture(i.culture_id) })
        : m.import_i_serie_orpheline({ culture: culture(i.culture_id), planche, annee });
    case "chevauchement":
      return m.import_i_chevauchement({
        planche,
        culture: culture(i.culture_id),
        autre: culture(i.autre_culture_id),
      });
    case "zone_hors_planche":
      return m.import_i_zone_hors_planche({ culture: culture(i.culture_id), planche, annee, nombre });
    case "culture_inconnue":
      return m.import_i_culture_inconnue({ culture: i.culture_id ?? "?" });
    case "planche_hors_terrain":
      return m.import_i_planche_hors_terrain({ planche, annee });
    case "planche_chevauche":
      return m.import_i_planche_chevauche({ planche, autre, annee });
    case "allee_trop_etroite":
      return m.import_i_allee_trop_etroite({ planche, autre, annee });
    case "largeur_invalide":
      return m.import_i_largeur_invalide();
    case "note_planche_inconnue":
      return m.import_i_note_planche_inconnue();
    case "note_invalide":
      return m.import_i_note_invalide();
    default:
      return m.import_i_autre();
  }
}
