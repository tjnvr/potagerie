//! The test cases T1 to T13 of S03 §12. Expected values are copied as they are
//! in the spec.

mod common;

use catalogue::Catalogue;
use chrono::NaiveDate;
use common::*;
use plan::{Creneau, Mode, Occupation, PlanError, Quantite, Serie, evaluer, quantite};

fn dates_de(serie: &Serie) -> (NaiveDate, NaiveDate, NaiveDate) {
    let dates = serie.dates();
    (dates.mise_en_place, dates.recolte, dates.fin)
}

/// T1 · Tomate, semis à l'abri le 2027-03-10
/// → `m` 2027-05-05 · `r` 2027-07-19 · `f` 2027-09-27
#[test]
fn t01_tomate_semis_a_l_abri() {
    let s = serie("tomate", planche_id(100), 1, 3, Mode::Abri, d(2027, 3, 10));
    assert_eq!(
        dates_de(&s),
        (d(2027, 5, 5), d(2027, 7, 19), d(2027, 9, 27))
    );
}

/// T2 · Laitue, pleine terre le 2027-04-01
/// → `m` 2027-04-01 · `r` 2027-06-18 (50 + 28 j) · `f` 2027-07-08
#[test]
fn t02_laitue_pleine_terre() {
    let s = serie(
        "laitue",
        planche_id(100),
        1,
        1,
        Mode::PleineTerre,
        d(2027, 4, 1),
    );
    assert_eq!(dates_de(&s), (d(2027, 4, 1), d(2027, 6, 18), d(2027, 7, 8)));
}

/// T3 · Carotte, pleine terre le 2027-03-15
/// → `r` 2027-06-23 · `f` 2027-08-02
#[test]
fn t03_carotte_pleine_terre() {
    let s = serie(
        "carotte",
        planche_id(100),
        1,
        1,
        Mode::PleineTerre,
        d(2027, 3, 15),
    );
    let (_, r, f) = dates_de(&s);
    assert_eq!((r, f), (d(2027, 6, 23), d(2027, 8, 2)));
}

/// T4 · Carotte, zone de 2 tronçons → 4 rangs × 25 pieds = ≈ 100
#[test]
fn t04_quantite_carotte() {
    let q = quantite(120, 2, plan::CaseCm::C50, culture("carotte"));
    assert_eq!(
        q,
        Quantite {
            rangs: 4,
            pieds_par_rang: 25
        }
    );
    assert_eq!(q.total(), 100);
}

/// T5 · Tomate, zone de 3 tronçons → 1 rang × 2 pieds = ≈ 2
#[test]
fn t05_quantite_tomate() {
    let q = quantite(120, 3, plan::CaseCm::C50, culture("tomate"));
    assert_eq!(
        q,
        Quantite {
            rangs: 1,
            pieds_par_rang: 2
        }
    );
    assert_eq!(q.total(), 2);
}

/// T6 · Tomate T1 sur [1,3], laitue plants le 2027-09-27 sur [1,2]
/// → Pas de conflit : la tomate se termine le jour où la laitue démarre
fn plan_t6() -> (plan::Plan, plan::PlancheId, plan::SerieId, plan::SerieId) {
    let (mut plan, planche) = plan_avec_planche();
    let tomate = serie("tomate", planche, 1, 3, Mode::Abri, d(2027, 3, 10));
    let laitue = serie("laitue", planche, 1, 2, Mode::Plants, d(2027, 9, 27));
    let (tomate_id, laitue_id) = (serie_id(1), serie_id(2));
    plan.ajouter_serie(tomate_id, tomate).unwrap();
    plan.ajouter_serie(laitue_id, laitue).unwrap();
    (plan, planche, tomate_id, laitue_id)
}

#[test]
fn t06_pas_de_conflit_le_jour_de_la_fin() {
    let (plan, _, tomate, laitue) = plan_t6();
    assert_eq!(plan.series[&tomate].dates().fin, d(2027, 9, 27));
    assert_eq!(plan.series[&laitue].dates().mise_en_place, d(2027, 9, 27));
    assert!(plan.conflits().is_empty());
}

/// T7 · Planche de 6 tronçons, avec seulement la carotte T3 sur [1,6], année 2027
/// → Créneaux : [1,6] du 03-01 au 03-14 écarté (< 35 j) ;
/// **[1,6] du 2027-08-02 au 2027-11-15** (106 j)
fn plan_t7() -> (plan::Plan, plan::PlancheId) {
    let (mut plan, planche) = plan_avec_planche();
    let carotte = serie("carotte", planche, 1, 6, Mode::PleineTerre, d(2027, 3, 15));
    plan.ajouter_serie(serie_id(1), carotte).unwrap();
    (plan, planche)
}

#[test]
fn t07_creneaux_apres_la_carotte() {
    let (plan, planche) = plan_t7();
    let creneaux = plan.creneaux(2027);
    assert_eq!(
        creneaux,
        vec![Creneau {
            planche_id: planche,
            annee: 2027,
            zone: zone(1, 6),
            debut: d(2027, 8, 2),
            fin: d(2027, 11, 15),
        }]
    );
    assert_eq!(creneaux[0].longueur_jours(), 106);
}

/// T8 · Suggestions pour le créneau de T7
/// → Laitue seule : semis à l'abri le 2027-07-05, `m` 2027-08-02, `r` 2027-09-21,
/// `f` 2027-10-11 ; les 5 autres cultures sont hors fenêtre de mise en place
#[test]
fn t08_suggestions_du_creneau() {
    let (plan, _) = plan_t7();
    let creneau = plan.creneaux(2027)[0];
    let suggestions = plan.suggestions(&creneau, Catalogue::v1(), aujourd_hui());
    assert_eq!(suggestions.len(), 1);
    let laitue = &suggestions[0];
    assert_eq!(laitue.culture_id, "laitue");
    assert_eq!(laitue.mode, Mode::Abri);
    assert_eq!(laitue.date, d(2027, 7, 5));
    assert_eq!(laitue.mise_en_place, d(2027, 8, 2));
    assert_eq!(laitue.recolte, d(2027, 9, 21));
    assert_eq!(laitue.fin, d(2027, 10, 11));
    assert_eq!(laitue.fin_manuelle, None);
    // The 5 other crops are out of their planting window.
    for id in ["tomate", "carotte", "haricot-vert", "courgette", "poireau"] {
        assert_eq!(evaluer(&creneau, culture(id), aujourd_hui()), None, "{id}");
    }
}

/// T9 · Carotte, semis à l'abri le 2027-03-01, 21 jours en godets saisis
/// → `m` 2027-03-22 · `r` 2027-06-30 · `f` 2027-08-09
#[test]
fn t09_carotte_a_l_abri_avec_godets_saisis() {
    let s = Serie::nouvelle(
        culture("carotte"),
        planche_id(100),
        zone(1, 1),
        Mode::Abri,
        d(2027, 3, 1),
        Some(21),
        None,
    )
    .unwrap();
    assert_eq!(
        dates_de(&s),
        (d(2027, 3, 22), d(2027, 6, 30), d(2027, 8, 9))
    );
}

/// T10 · Créneau [1,6] du 2027-05-01 au 2027-11-15, aujourd'hui = 2027-04-20
/// → La tomate est proposée **en plants** (son semis à l'abri tomberait le
/// 2027-03-06, dans le passé) : `m` 2027-05-01 · `r` 2027-07-15 · `f` 2027-09-23
#[test]
fn t10_tomate_en_plants_si_le_semis_serait_dans_le_passe() {
    let creneau = Creneau {
        planche_id: planche_id(100),
        annee: 2027,
        zone: zone(1, 6),
        debut: d(2027, 5, 1),
        fin: d(2027, 11, 15),
    };
    let tomate = evaluer(&creneau, culture("tomate"), d(2027, 4, 20)).unwrap();
    assert_eq!(tomate.mode, Mode::Plants);
    assert_eq!(tomate.date, d(2027, 5, 1));
    assert_eq!(tomate.mise_en_place, d(2027, 5, 1));
    assert_eq!(tomate.recolte, d(2027, 7, 15));
    assert_eq!(tomate.fin, d(2027, 9, 23));
    assert_eq!(tomate.fin_manuelle, None);

    // Without the cut-off, the same slot would have given a sowing under cover.
    let abri = evaluer(&creneau, culture("tomate"), d(2027, 3, 6)).unwrap();
    assert_eq!((abri.mode, abri.date), (Mode::Abri, d(2027, 3, 6)));
}

/// T11 · Planche de 6 tronçons : carotte T3 sur [1,3], poireau semé à l'abri le
/// 2027-05-01 sur [4,6] ; planche supprimée à partir de 2028
/// → Poireau : `m` 2027-07-10 · `r` 2027-11-07 · `f` 2028-02-05 → **supprimé**
/// (à cheval sur 2028). Carotte gardée
#[test]
fn t11_suppression_de_la_planche_a_partir_de_2028() {
    let (mut plan, planche) = plan_avec_planche();
    let carotte = serie("carotte", planche, 1, 3, Mode::PleineTerre, d(2027, 3, 15));
    let poireau = serie("poireau", planche, 4, 6, Mode::Abri, d(2027, 5, 1));
    assert_eq!(
        dates_de(&poireau),
        (d(2027, 7, 10), d(2027, 11, 7), d(2028, 2, 5))
    );
    plan.ajouter_serie(serie_id(1), carotte).unwrap();
    plan.ajouter_serie(serie_id(2), poireau).unwrap();

    assert_eq!(plan.series_supprimees_par(planche, 2028), vec![serie_id(2)]);
    let supprimees = plan
        .supprimer_planche(planche, 2028, aujourd_hui())
        .unwrap();

    assert_eq!(supprimees, vec![serie_id(2)]);
    assert!(plan.series.contains_key(&serie_id(1)));
    assert!(!plan.series.contains_key(&serie_id(2)));
}

/// T12 · Séries de T6 ; on glisse la laitue de −1 jour
/// → Refusé : la laitue démarrerait le 2027-09-26, en conflit avec la tomate sur [1,2]
#[test]
fn t12_deplacement_refuse_en_cas_de_conflit() {
    let (mut plan, _, tomate, laitue) = plan_t6();
    let avant = plan.clone();

    let erreur = plan.decaler_serie(laitue, -1).unwrap_err();

    assert_eq!(
        erreur,
        PlanError::Conflit {
            avec: tomate,
            troncons: zone(1, 2),
            periode: Occupation {
                debut: d(2027, 9, 26),
                fin: d(2027, 9, 27),
            },
        }
    );
    assert_eq!(erreur.code(), "conflit");
    assert_eq!(plan, avant, "a refused move changes nothing");
    assert_eq!(plan.series[&laitue].date, d(2027, 9, 27));
}

/// T13 · Créneau [1,6] du 2027-08-02 au 2027-10-05 (65 j)
/// → Laitue : `m` 2027-08-02 · `r` 2027-09-21 · `f` proposée 2027-10-06 au lieu du
/// 2027-10-11, enregistrée comme **fin modifiée à la main**
#[test]
fn t13_fin_raccourcie_enregistree_comme_fin_manuelle() {
    let creneau = Creneau {
        planche_id: planche_id(100),
        annee: 2027,
        zone: zone(1, 6),
        debut: d(2027, 8, 2),
        fin: d(2027, 10, 5),
    };
    assert_eq!(creneau.longueur_jours(), 65);

    let laitue = evaluer(&creneau, culture("laitue"), aujourd_hui()).unwrap();
    assert_eq!(laitue.mise_en_place, d(2027, 8, 2));
    assert_eq!(laitue.recolte, d(2027, 9, 21));
    assert_eq!(laitue.fin, d(2027, 10, 6));
    assert_eq!(laitue.fin_manuelle, Some(d(2027, 10, 6)));

    // Touching the suggestion creates a serie whose end no longer follows `r + duree`.
    let serie = laitue.en_serie(culture("laitue"), &creneau).unwrap();
    assert_eq!(serie.fin_manuelle, Some(d(2027, 10, 6)));
    assert_eq!(serie.dates().fin, d(2027, 10, 6));
}
