//! Upgrade of documents written before the English renaming (app versions 0.1 and 0.2).
//!
//! Those documents name their fields and a few values in French (`planches`,
//! `fin_manuelle`, `pleine_terre`, `tomate`…). [`upgrade_legacy_json`] renames them
//! so that [`crate::Plan`] reads them; a document already in English is left
//! untouched. Generated from the former field names; never extended: no new
//! document will use the old names.

use serde_json::{Map, Value};

/// Former field name, current field name.
const KEYS: &[(&str, &str)] = &[
    ("abri_jours", "indoor_days"),
    ("allee_cm", "aisle_cm"),
    ("apres_jours", "after_days"),
    ("case_cm", "cell_cm"),
    ("culture_id", "crop_id"),
    ("date_debut", "date_start"),
    ("date_fin", "date_end"),
    ("dispositions", "layouts"),
    ("duree_jours", "duration_days"),
    ("fin_manuelle", "manual_end"),
    ("largeur_cm", "width_cm"),
    ("longueur_cm", "length_cm"),
    ("masquee", "hidden"),
    ("mis_a_jour_le", "updated_at"),
    ("nom", "name"),
    ("planche_id", "bed_id"),
    ("planche_type", "bed_type"),
    ("planches", "beds"),
    ("series", "plantings"),
    ("terrain", "plot"),
    ("texte", "text"),
    ("troncon_debut", "segment_start"),
    ("troncon_fin", "segment_end"),
    ("etat", "state"),
];

/// Fields whose string value was renamed too (modes, types, crop ids).
const VALUE_KEYS: &[&str] = &["mode", "type", "state", "etat", "crop_id", "culture_id"];

/// Former value, current value.
const VALUES: &[(&str, &str)] = &[
    ("abri", "indoor"),
    ("absente", "absent"),
    ("ail", "garlic"),
    ("aneth", "dill"),
    ("artichaut", "artichoke"),
    ("aubergine", "eggplant"),
    ("bac", "planter"),
    ("basilic", "basil"),
    ("betterave", "beetroot"),
    ("blette", "swiss-chard"),
    ("brocoli", "broccoli"),
    ("butte", "mound"),
    ("carotte", "carrot"),
    ("carre", "square"),
    ("celeri-branche", "celery"),
    ("celeri-rave", "celeriac"),
    ("cerfeuil", "chervil"),
    ("chicoree-frisee", "curly-endive"),
    ("chou-chinois", "chinese-cabbage"),
    ("chou-de-bruxelles", "brussels-sprout"),
    ("chou-fleur", "cauliflower"),
    ("chou-kale", "kale"),
    ("chou-pomme", "cabbage"),
    ("chou-rave", "kohlrabi"),
    ("ciboulette", "chives"),
    ("concombre", "cucumber"),
    ("coriandre", "coriander"),
    ("cornichon", "gherkin"),
    ("courge-butternut", "butternut-squash"),
    ("courgette", "zucchini"),
    ("echalote", "shallot"),
    ("epinard", "spinach"),
    ("fenouil", "fennel"),
    ("feve", "fava-bean"),
    ("gombo", "okra"),
    ("haricot-a-ecosser", "shelling-bean"),
    ("haricot-a-rames", "pole-bean"),
    ("haricot-vert", "green-bean"),
    ("laitue", "lettuce"),
    ("laitue-romaine", "romaine-lettuce"),
    ("mache", "lambs-lettuce"),
    ("mais-doux", "sweet-corn"),
    ("navet", "turnip"),
    ("oignon", "onion"),
    ("oignon-blanc", "spring-onion"),
    ("panais", "parsnip"),
    ("pasteque", "watermelon"),
    ("patate-douce", "sweet-potato"),
    ("patisson", "pattypan-squash"),
    ("periode", "period"),
    ("persil", "parsley"),
    ("petit-pois", "pea"),
    ("piment", "chili-pepper"),
    ("planche", "bed"),
    ("pleine_terre", "open_ground"),
    ("poireau", "leek"),
    ("pois-gourmand", "snow-pea"),
    ("poivron", "bell-pepper"),
    ("pomme-de-terre", "potato"),
    ("potimarron", "red-kuri-squash"),
    ("potiron", "pumpkin"),
    ("pourpier", "purslane"),
    ("presente", "present"),
    ("radis", "radish"),
    ("radis-noir", "black-radish"),
    ("roquette", "arugula"),
    ("salsifis", "salsify"),
    ("scarole", "escarole"),
    ("tomate", "tomato"),
    ("tomate-cerise", "cherry-tomato"),
    ("topinambour", "jerusalem-artichoke"),
];

fn rename(table: &'static [(&'static str, &'static str)], name: &str) -> Option<&'static str> {
    table
        .iter()
        .find(|(old, _)| *old == name)
        .map(|(_, new)| *new)
}

/// Renames the French fields and values of a stored plan document.
pub fn upgrade_legacy_json(value: Value) -> Value {
    upgrade(value, None)
}

fn upgrade(value: Value, key: Option<&str>) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| {
                    let new_key = rename(KEYS, &k).map(str::to_string).unwrap_or(k);
                    let v = upgrade(v, Some(&new_key));
                    (new_key, v)
                })
                .collect::<Map<_, _>>(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(|v| upgrade(v, key)).collect()),
        Value::String(text) if key.is_some_and(|k| VALUE_KEYS.contains(&k)) => {
            Value::String(rename(VALUES, &text).map(str::to_string).unwrap_or(text))
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_former_document_reads_with_the_current_names() {
        let old = json!({
            "id": "0192d5e0-0000-7000-8000-000000000001", "version": 4,
            "mis_a_jour_le": "2027-06-01T10:00:00Z",
            "terrain": {"longueur_cm": 900, "largeur_cm": 650, "case_cm": 50, "allee_cm": 50},
            "planches": {"p1": {"nom": "A", "dispositions": {"2027": {"etat": "presente", "type": "planche", "x": 50, "y": 50, "longueur_cm": 300, "largeur_cm": 120}}}},
            "series": {"s1": {"culture_id": "haricot-vert", "planche_id": "p1", "troncon_debut": 1, "troncon_fin": 2, "mode": "pleine_terre", "date": "2027-05-10", "abri_jours": 0, "apres_jours": 60, "duree_jours": 30}},
            "annotations": {}
        });
        let new = upgrade_legacy_json(old);
        assert_eq!(new["beds"]["p1"]["name"], "A");
        assert_eq!(new["beds"]["p1"]["layouts"]["2027"]["state"], "present");
        assert_eq!(new["beds"]["p1"]["layouts"]["2027"]["type"], "bed");
        assert_eq!(new["plantings"]["s1"]["crop_id"], "green-bean");
        assert_eq!(new["plantings"]["s1"]["mode"], "open_ground");
        assert_eq!(new["plantings"]["s1"]["after_days"], 60);
        assert_eq!(new["plot"]["length_cm"], 900);
        assert_eq!(new["updated_at"], "2027-06-01T10:00:00Z");
    }

    #[test]
    fn a_current_document_is_left_untouched() {
        let current = json!({"beds": {"p1": {"name": "planche", "layouts": {}}}, "plantings": {}});
        assert_eq!(upgrade_legacy_json(current.clone()), current);
    }
}
