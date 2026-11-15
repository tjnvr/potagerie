import type { AppError, ErrorCode } from "./bindings";
import { m } from "./paraglide/messages.js";

/** One message per error code returned by Rust. Typed as a full record, so a
 *  new code in `bindings.ts` breaks `pnpm check` until it is translated. */
const messages: Record<ErrorCode, () => string> = {
  annee_passee: m.erreur_annee_passee,
  too_many_planches: m.erreur_too_many_planches,
  planche_inconnue: m.erreur_planche_inconnue,
  serie_inconnue: m.erreur_serie_inconnue,
  planche_existante: m.erreur_planche_existante,
  serie_existante: m.erreur_serie_existante,
  culture_inconnue: m.erreur_culture_inconnue,
  zone_invalide: m.erreur_zone_invalide,
  planche_absente: m.erreur_planche_absente,
  zone_hors_planche: m.erreur_zone_hors_planche,
  conflict: m.erreur_conflict,
  pot_days_required: m.erreur_pot_days_required,
  fin_manuelle_invalide: m.erreur_fin_manuelle_invalide,
  occupation_vide: m.erreur_occupation_vide,
  longueur_invalide: m.erreur_longueur_invalide,
  terrain_absent: m.erreur_terrain_absent,
  terrain_existant: m.erreur_terrain_existant,
  case_invalide: m.erreur_case_invalide,
  annotation_inconnue: m.erreur_annotation_inconnue,
  enregistrement: m.erreur_enregistrement,
  version_perimee: m.erreur_version_perimee,
  base_trop_recente: m.erreur_base_trop_recente,
  fichier: m.erreur_fichier,
  import_invalide: m.erreur_import_invalide,
  import_trop_recent: m.erreur_import_trop_recent,
  svg_invalide: m.erreur_svg_invalide,
  rendu_impossible: m.erreur_rendu_impossible,
};

/** The French text of an error returned by a command. */
export function traduireErreur(erreur: AppError): string {
  const message = messages[erreur.code];
  return message ? message() : m.erreur_inattendue();
}

/** Write failures: the change is kept on screen and can be retried (F-CPT-2). */
export function estEchecEnregistrement(erreur: AppError): boolean {
  return erreur.code === "enregistrement";
}
