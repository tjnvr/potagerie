//! Layouts by year (S03 §7).

use crate::ids::PlancheId;
use crate::model::{Disposition, Geometrie, Plan, Planche};

impl Planche {
    /// The layout in force in `annee`: the last one recorded for a year ≤ `annee`.
    /// `None` if the planche does not exist yet, or was deleted.
    pub fn disposition_en(&self, annee: i32) -> Option<&Geometrie> {
        match self.dispositions.range(..=annee).next_back() {
            Some((_, Disposition::Presente(geometrie))) => Some(geometrie),
            _ => None,
        }
    }
}

impl Plan {
    /// The planches that exist in `annee`, with their layout.
    pub fn planches_en(&self, annee: i32) -> Vec<(PlancheId, &Planche, &Geometrie)> {
        self.planches
            .iter()
            .filter_map(|(id, planche)| Some((*id, planche, planche.disposition_en(annee)?)))
            .collect()
    }

    /// Number of troncons of a planche in `annee`, if it exists then.
    pub fn nb_troncons(&self, planche_id: PlancheId, annee: i32) -> Option<u32> {
        let geometrie = self.planches.get(&planche_id)?.disposition_en(annee)?;
        Some(geometrie.troncons(self.terrain.case_cm))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::model::TypePlanche;

    fn geo(longueur_cm: u32) -> Geometrie {
        Geometrie {
            type_planche: TypePlanche::Planche,
            x: 0,
            y: 0,
            longueur_cm,
            largeur_cm: 120,
        }
    }

    #[test]
    fn la_derniere_disposition_enregistree_vaut_jusqu_a_la_suivante() {
        let planche = Planche {
            nom: "A".into(),
            dispositions: BTreeMap::from([
                (2026, Disposition::Presente(geo(300))),
                (2028, Disposition::Presente(geo(200))),
                (2030, Disposition::Absente),
            ]),
        };
        assert_eq!(planche.disposition_en(2025), None);
        assert_eq!(planche.disposition_en(2026).unwrap().longueur_cm, 300);
        assert_eq!(planche.disposition_en(2027).unwrap().longueur_cm, 300);
        assert_eq!(planche.disposition_en(2028).unwrap().longueur_cm, 200);
        assert_eq!(planche.disposition_en(2029).unwrap().longueur_cm, 200);
        assert_eq!(planche.disposition_en(2030), None);
        assert_eq!(planche.disposition_en(2040), None);
    }
}
