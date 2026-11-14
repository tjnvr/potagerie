//! Fixtures shared by the integration tests. Data come from S02, and the
//! planche is 1.2 m wide with 50 cm cases (S03 §12).
#![allow(dead_code)]

use catalogue::{Catalogue, Culture};
use chrono::{DateTime, NaiveDate, Utc};
use plan::{
    CaseCm, Geometrie, Mode, Plan, PlanId, PlancheId, Serie, SerieId, Terrain, TypePlanche, Zone,
};
use uuid::Uuid;

/// Civil date.
pub fn d(annee: i32, mois: u32, jour: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(annee, mois, jour).unwrap()
}

/// "Aujourd'hui" of S03 §12 unless a case says otherwise.
pub fn aujourd_hui() -> NaiveDate {
    d(2026, 11, 14)
}

pub fn culture(id: &str) -> &'static Culture {
    Catalogue::v1().culture(id).unwrap()
}

pub fn zone(t1: u32, t2: u32) -> Zone {
    Zone::new(t1, t2).unwrap()
}

pub fn planche_id(n: u128) -> PlancheId {
    PlancheId::depuis_uuid(Uuid::from_u128(n))
}

pub fn serie_id(n: u128) -> SerieId {
    SerieId::depuis_uuid(Uuid::from_u128(n))
}

pub fn maintenant() -> DateTime<Utc> {
    DateTime::from_timestamp(1_800_000_000, 0).unwrap()
}

pub fn terrain() -> Terrain {
    Terrain {
        longueur_cm: 2000,
        largeur_cm: 1000,
        case_cm: CaseCm::C50,
        allee_cm: 50,
    }
}

/// A planche of `nb_troncons` cases (50 cm), 1.2 m wide.
pub fn geometrie(nb_troncons: u32) -> Geometrie {
    Geometrie {
        type_planche: TypePlanche::Planche,
        x: 0,
        y: 0,
        longueur_cm: nb_troncons * 50,
        largeur_cm: 120,
    }
}

/// A plan with one planche of 6 troncons (3 m × 1.2 m), present from 2026.
pub fn plan_avec_planche() -> (Plan, PlancheId) {
    plan_avec_planche_de(6)
}

pub fn plan_avec_planche_de(nb_troncons: u32) -> (Plan, PlancheId) {
    let mut plan = Plan::nouveau(
        PlanId::depuis_uuid(Uuid::from_u128(1)),
        terrain(),
        maintenant(),
    );
    let id = planche_id(100);
    plan.ajouter_planche(
        id,
        "Planche A".into(),
        2026,
        geometrie(nb_troncons),
        aujourd_hui(),
    )
    .unwrap();
    (plan, id)
}

/// A serie of catalogue crop `culture_id`, without godets days or manual end.
pub fn serie(
    culture_id: &str,
    planche: PlancheId,
    t1: u32,
    t2: u32,
    mode: Mode,
    date: NaiveDate,
) -> Serie {
    Serie::nouvelle(
        culture(culture_id),
        planche,
        zone(t1, t2),
        mode,
        date,
        None,
        None,
    )
    .unwrap()
}
