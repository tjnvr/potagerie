//! Names of the app commands (S05 §7), shared by `build.rs` (permissions) and
//! the app. A test checks they match `collect_commands!` in `lib.rs`.

pub const COMMANDS: &[&str] = &[
    "catalogue",
    "vue_annee",
    "creer_terrain",
    "modifier_terrain",
    "creer_planche",
    "modifier_planche",
    "deplacer_planche",
    "series_depassant",
    "supprimer_planche",
    "creer_serie",
    "modifier_serie",
    "supprimer_serie",
    "creer_annotation",
    "modifier_annotation",
    "supprimer_annotation",
    "exporter_frise",
    "exporter_potager",
    "lire_import",
    "importer_potager",
    "effacer_donnees",
];
