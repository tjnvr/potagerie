//! Potagerie crop catalogue (S02).
//!
//! `cultures.json` follows S02 §4 exactly and is embedded in the binary with
//! [`include_str!`]. Use [`Catalogue::v1`] to get the parsed catalogue.

use std::fmt;
use std::sync::OnceLock;

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// Raw catalogue file, as shipped with the app (S02 §4).
pub const CULTURES_JSON: &str = include_str!("../cultures.json");

/// One crop sheet (S02 §2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Culture {
    /// Stable identifier, lowercase without accents (`tomate`).
    pub id: String,
    /// Display name (`Tomate`).
    pub nom: String,
    /// Botanical family, used to rank suggestions.
    pub famille: String,
    /// Days in pots before transplanting; 0 if not usually sown under cover.
    pub abri_jours: u32,
    /// Days from planting out to the start of harvest.
    pub recolte_apres_jours: u32,
    /// Length of the harvest period, in days.
    pub recolte_duree_jours: u32,
    /// Ideal planting-out window; may span two years (`debut > fin`).
    pub mise_en_place: Periode,
    /// Spacing between rows, in cm.
    pub rang_cm: u32,
    /// Spacing along the row, in cm.
    pub pied_cm: u32,
    /// Not used in V1; reserved for frost alerts.
    pub sensible_gel: bool,
}

/// A `MM-JJ` → `MM-JJ` period of the catalogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Periode {
    pub debut: MoisJour,
    pub fin: MoisJour,
}

impl Periode {
    /// True when the period spans two years (`debut > fin`, S03 §10).
    pub fn a_cheval(&self) -> bool {
        self.debut > self.fin
    }
}

/// A month and day without a year (`MM-JJ`), serialised as `"05-01"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MoisJour {
    mois: u32,
    jour: u32,
}

impl MoisJour {
    /// Builds a month-day; `02-29` is accepted.
    pub fn new(mois: u32, jour: u32) -> Option<Self> {
        // 2024 is a leap year, so every valid MM-JJ exists in it.
        NaiveDate::from_ymd_opt(2024, mois, jour).map(|_| Self { mois, jour })
    }

    pub fn mois(&self) -> u32 {
        self.mois
    }

    pub fn jour(&self) -> u32 {
        self.jour
    }

    /// Places the month-day in year `annee`. `02-29` becomes `02-28` in a
    /// non-leap year (S03 §1).
    pub fn dans_annee(&self, annee: i32) -> NaiveDate {
        NaiveDate::from_ymd_opt(annee, self.mois, self.jour)
            .or_else(|| NaiveDate::from_ymd_opt(annee, self.mois, self.jour - 1))
            .expect("a valid MM-JJ always exists in a year, up to 02-29")
    }

    /// Month-day of a civil date.
    pub fn de_date(date: NaiveDate) -> Self {
        Self {
            mois: date.month(),
            jour: date.day(),
        }
    }
}

impl fmt::Display for MoisJour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}-{:02}", self.mois, self.jour)
    }
}

impl std::str::FromStr for MoisJour {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalide = || format!("invalid MM-JJ date: {s:?}");
        let (m, j) = s.split_once('-').ok_or_else(invalide)?;
        if m.len() != 2 || j.len() != 2 {
            return Err(invalide());
        }
        let mois = m.parse().map_err(|_| invalide())?;
        let jour = j.parse().map_err(|_| invalide())?;
        Self::new(mois, jour).ok_or_else(invalide)
    }
}

impl Serialize for MoisJour {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for MoisJour {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}

/// The list of crops, in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalogue {
    cultures: Vec<Culture>,
}

impl Catalogue {
    /// Parses a catalogue in the S02 §4 format.
    pub fn depuis_json(json: &str) -> Result<Self, serde_json::Error> {
        Ok(Self {
            cultures: serde_json::from_str(json)?,
        })
    }

    /// Builds a catalogue from crops, for tests or a future custom catalogue.
    pub fn depuis_cultures(cultures: Vec<Culture>) -> Self {
        Self { cultures }
    }

    /// The embedded V1 catalogue (the 6 crops of S02 §3), parsed once.
    pub fn v1() -> &'static Catalogue {
        static V1: OnceLock<Catalogue> = OnceLock::new();
        V1.get_or_init(|| {
            Catalogue::depuis_json(CULTURES_JSON).expect("embedded cultures.json is valid")
        })
    }

    /// The crop with this `id`, if any.
    pub fn culture(&self, id: &str) -> Option<&Culture> {
        self.cultures.iter().find(|c| c.id == id)
    }

    pub fn cultures(&self) -> &[Culture] {
        &self.cultures
    }

    pub fn iter(&self) -> impl Iterator<Item = &Culture> {
        self.cultures.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_contient_les_6_cultures() {
        let ids: Vec<&str> = Catalogue::v1().iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "tomate",
                "laitue",
                "carotte",
                "haricot-vert",
                "courgette",
                "poireau"
            ]
        );
    }

    #[test]
    fn tomate_conforme_a_s02() {
        let tomate = Catalogue::v1().culture("tomate").unwrap();
        assert_eq!(
            *tomate,
            Culture {
                id: "tomate".into(),
                nom: "Tomate".into(),
                famille: "Solanacées".into(),
                abri_jours: 56,
                recolte_apres_jours: 75,
                recolte_duree_jours: 70,
                mise_en_place: Periode {
                    debut: MoisJour::new(5, 1).unwrap(),
                    fin: MoisJour::new(6, 15).unwrap(),
                },
                rang_cm: 80,
                pied_cm: 60,
                sensible_gel: true,
            }
        );
    }

    #[test]
    fn json_relu_a_l_identique() {
        let relu: Vec<Culture> =
            serde_json::from_str(&serde_json::to_string(Catalogue::v1().cultures()).unwrap())
                .unwrap();
        assert_eq!(relu, Catalogue::v1().cultures());
    }

    #[test]
    fn mois_jour_29_fevrier() {
        let d = MoisJour::new(2, 29).unwrap();
        assert_eq!(
            d.dans_annee(2028),
            NaiveDate::from_ymd_opt(2028, 2, 29).unwrap()
        );
        assert_eq!(
            d.dans_annee(2027),
            NaiveDate::from_ymd_opt(2027, 2, 28).unwrap()
        );
    }

    #[test]
    fn mois_jour_refuse_les_formats_invalides() {
        for s in ["5-01", "13-01", "02-30", "0501", "ab-cd"] {
            assert!(s.parse::<MoisJour>().is_err(), "{s}");
        }
        assert_eq!("09-30".parse::<MoisJour>().unwrap().to_string(), "09-30");
    }
}
