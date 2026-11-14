//! Edge cases that S03 describes but T1 to T13 do not cover.

mod common;

use catalogue::Catalogue;
use common::*;
use plan::{
    Creneau, Disposition, Fraction, Mode, Occupation, Plan, PlanError, Serie, TypeTache, Zone,
    evaluer,
};

// ---- §2 dates ----------------------------------------------------------

#[test]
fn plants_achetes_mise_en_place_le_jour_saisi() {
    let s = serie("tomate", planche_id(100), 1, 1, Mode::Plants, d(2027, 5, 5));
    let dates = s.dates();
    assert_eq!(dates.semis, None);
    assert_eq!(dates.mise_en_place, d(2027, 5, 5));
    assert_eq!(dates.recolte, d(2027, 7, 19));
}

#[test]
fn pleine_terre_ajoute_la_phase_de_godets_de_la_culture() {
    // Tomato: 56 days in pots would have been spent in the ground: 75 + 56.
    let s = serie(
        "tomate",
        planche_id(100),
        1,
        1,
        Mode::PleineTerre,
        d(2027, 5, 5),
    );
    assert_eq!(
        s.dates().recolte,
        d(2027, 5, 5) + chrono::Duration::days(131)
    );
}

#[test]
fn saisie_du_repiquage_enregistre_la_date_de_semis() {
    // "Je connais plutôt la date de repiquage": s = x − A.
    let semis = Serie::semis_depuis_repiquage(d(2027, 5, 5), 56);
    assert_eq!(semis, d(2027, 3, 10));
    let s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, semis);
    assert_eq!(s.dates().mise_en_place, d(2027, 5, 5));
}

#[test]
fn abri_sans_godets_pour_une_culture_sans_abri_est_refuse() {
    for godets in [None, Some(0)] {
        let erreur = Serie::nouvelle(
            culture("haricot-vert"),
            planche_id(100),
            zone(1, 1),
            Mode::Abri,
            d(2027, 5, 10),
            godets,
            None,
        )
        .unwrap_err();
        assert_eq!(erreur, PlanError::GodetsRequis);
    }
}

#[test]
fn godets_saisis_ignores_si_la_culture_a_deja_un_abri() {
    let s = Serie::nouvelle(
        culture("tomate"),
        planche_id(100),
        zone(1, 1),
        Mode::Abri,
        d(2027, 3, 10),
        Some(10),
        None,
    )
    .unwrap();
    assert_eq!(s.abri_jours, 56);
}

#[test]
fn les_durees_sont_copiees_a_la_creation() {
    let mut tomate = culture("tomate").clone();
    let s = Serie::nouvelle(
        &tomate,
        planche_id(100),
        zone(1, 1),
        Mode::Abri,
        d(2027, 3, 10),
        None,
        None,
    )
    .unwrap();
    let avant = s.dates();
    // A catalogue update does not move an existing serie.
    tomate.recolte_apres_jours = 90;
    tomate.recolte_duree_jours = 10;
    assert_eq!(s.dates(), avant);
    assert_eq!((s.abri_jours, s.apres_jours, s.duree_jours), (56, 75, 70));
}

#[test]
fn changer_de_culture_recopie_les_durees() {
    let mut s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, d(2027, 3, 10));
    s.changer_culture(culture("laitue"), None).unwrap();
    assert_eq!(
        (
            s.culture_id.as_str(),
            s.abri_jours,
            s.apres_jours,
            s.duree_jours
        ),
        ("laitue", 28, 50, 20)
    );
}

#[test]
fn changer_le_mode_d_une_carotte_retire_les_godets_saisis() {
    let mut s = Serie::nouvelle(
        culture("carotte"),
        planche_id(100),
        zone(1, 1),
        Mode::Abri,
        d(2027, 3, 1),
        Some(21),
        None,
    )
    .unwrap();
    s.changer_mode(Mode::PleineTerre, culture("carotte"), None)
        .unwrap();
    // As T3: nothing added for a carrot sown in the ground.
    assert_eq!(s.dates().recolte, d(2027, 6, 9));
}

#[test]
fn fin_manuelle_obligatoirement_apres_le_debut_de_recolte() {
    let tomate = culture("tomate");
    let nouvelle = |fin| {
        Serie::nouvelle(
            tomate,
            planche_id(100),
            zone(1, 1),
            Mode::Abri,
            d(2027, 3, 10),
            None,
            Some(fin),
        )
    };
    assert!(matches!(
        nouvelle(d(2027, 7, 19)),
        Err(PlanError::FinManuelleInvalide { .. })
    ));
    assert_eq!(
        nouvelle(d(2027, 7, 20)).unwrap().dates().fin,
        d(2027, 7, 20)
    );
}

#[test]
fn la_fin_manuelle_ne_se_recalcule_pas() {
    let mut s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, d(2027, 3, 10));
    s.fin_manuelle = Some(d(2027, 8, 1));
    s.date = d(2027, 3, 20);
    assert_eq!(s.dates().recolte, d(2027, 7, 29));
    assert_eq!(s.dates().fin, d(2027, 8, 1));
}

#[test]
fn glisser_decale_la_date_et_la_fin_manuelle() {
    let mut s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, d(2027, 3, 10));
    s.fin_manuelle = Some(d(2027, 8, 1));
    let decalee = s.decalee(7);
    assert_eq!(decalee.date, d(2027, 3, 17));
    assert_eq!(decalee.fin_manuelle, Some(d(2027, 8, 8)));
    // The duration is kept.
    let (avant, apres) = (s.occupation(), decalee.occupation());
    assert_eq!(avant.fin - avant.debut, apres.fin - apres.debut);
}

#[test]
fn phases_de_la_frise() {
    let s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, d(2027, 3, 10));
    let phases = s.phases();
    assert_eq!(
        phases.godets,
        Some(Occupation {
            debut: d(2027, 3, 10),
            fin: d(2027, 5, 5)
        })
    );
    assert_eq!(
        phases.croissance,
        Occupation {
            debut: d(2027, 5, 5),
            fin: d(2027, 7, 19)
        }
    );
    assert_eq!(
        phases.recolte,
        Occupation {
            debut: d(2027, 7, 19),
            fin: d(2027, 9, 27)
        }
    );
    let plants = serie("tomate", planche_id(100), 1, 1, Mode::Plants, d(2027, 5, 5));
    assert_eq!(plants.phases().godets, None);
}

// ---- §4 zone shortcuts ----------------------------------------------------

#[test]
fn taille_des_raccourcis() {
    // k = max(1, round(n × fraction)), halves rounded up.
    assert_eq!(
        [
            Fraction::Entiere,
            Fraction::Moitie,
            Fraction::Tiers,
            Fraction::Quart
        ]
        .map(|f| f.taille(6)),
        [6, 3, 2, 2]
    );
    assert_eq!(Fraction::Moitie.taille(5), 3);
    assert_eq!(Fraction::Quart.taille(5), 1);
    assert_eq!(Fraction::Quart.taille(1), 1);
    assert_eq!(Fraction::Tiers.taille(1), 1);
}

#[test]
fn raccourci_premiere_suite_de_troncons_libres() {
    let (mut plan, planche) = plan_avec_planche();
    // [1,2] taken during the whole season.
    let occupe = serie("carotte", planche, 1, 2, Mode::PleineTerre, d(2027, 3, 15));
    plan.ajouter_serie(serie_id(1), occupe).unwrap();
    let occupation = Occupation {
        debut: d(2027, 4, 1),
        fin: d(2027, 6, 1),
    };
    assert_eq!(
        plan.raccourci(planche, Fraction::Moitie, occupation, None),
        Some(zone(3, 5))
    );
    assert_eq!(
        plan.raccourci(planche, Fraction::Tiers, occupation, None),
        Some(zone(3, 4))
    );
    // The whole planche is not free.
    assert_eq!(
        plan.raccourci(planche, Fraction::Entiere, occupation, None),
        None
    );
    // Free after the carrot ends: back to the first troncons.
    let apres = Occupation {
        debut: d(2027, 8, 2),
        fin: d(2027, 9, 1),
    };
    assert_eq!(
        plan.raccourci(planche, Fraction::Entiere, apres, None),
        Some(zone(1, 6))
    );
    // Editing the carrot itself ignores it.
    assert_eq!(
        plan.raccourci(planche, Fraction::Entiere, occupation, Some(serie_id(1))),
        Some(zone(1, 6))
    );
}

#[test]
fn raccourci_grise_si_aucune_suite_libre() {
    let (mut plan, planche) = plan_avec_planche();
    // Troncons 2 and 5 are taken: no run of 3 is left free.
    for (i, t) in [2, 5].into_iter().enumerate() {
        let s = serie("carotte", planche, t, t, Mode::PleineTerre, d(2027, 3, 15));
        plan.ajouter_serie(serie_id(i as u128 + 1), s).unwrap();
    }
    let occupation = Occupation {
        debut: d(2027, 4, 1),
        fin: d(2027, 6, 1),
    };
    assert_eq!(
        plan.raccourci(planche, Fraction::Moitie, occupation, None),
        None
    );
    assert_eq!(
        plan.raccourci(planche, Fraction::Quart, occupation, None),
        Some(zone(3, 4))
    );
}

#[test]
fn planche_raccourcie_signale_les_series_qui_ne_tiennent_plus() {
    let (mut plan, planche) = plan_avec_planche();
    // 2027: carrots on [1,6]; 2028: lettuce on [1,6]; and a short serie on [1,3] in 2029.
    let carotte = serie("carotte", planche, 1, 6, Mode::PleineTerre, d(2027, 3, 15));
    let laitue = serie("laitue", planche, 1, 6, Mode::Plants, d(2028, 4, 1));
    let courte = serie("carotte", planche, 1, 3, Mode::PleineTerre, d(2029, 3, 15));
    for (n, s) in [(1, carotte), (2, laitue), (3, courte)] {
        plan.ajouter_serie(serie_id(n), s).unwrap();
    }
    // Shortened to 4 troncons from 2027: both series on [1,6] are flagged, not [1,3].
    assert_eq!(
        plan.series_hors_planche_si_raccourcie(planche, 2027, 4),
        vec![serie_id(1), serie_id(2)]
    );
    // From 2028: the carrots ended in 2027.
    assert_eq!(
        plan.series_hors_planche_si_raccourcie(planche, 2028, 4),
        vec![serie_id(2)]
    );
    assert!(
        plan.series_hors_planche_si_raccourcie(planche, 2029, 4)
            .is_empty()
    );
}

// ---- §5 validity and conflicts -------------------------------------------

#[test]
fn zones_disjointes_meme_periode_pas_de_conflit() {
    let (mut plan, planche) = plan_avec_planche();
    plan.ajouter_serie(
        serie_id(1),
        serie("carotte", planche, 1, 2, Mode::PleineTerre, d(2027, 3, 15)),
    )
    .unwrap();
    plan.ajouter_serie(
        serie_id(2),
        serie("carotte", planche, 3, 4, Mode::PleineTerre, d(2027, 3, 15)),
    )
    .unwrap();
    assert!(plan.conflits().is_empty());
}

#[test]
fn conflit_donne_les_troncons_et_la_periode_en_commun() {
    let (mut plan, planche) = plan_avec_planche();
    plan.ajouter_serie(
        serie_id(1),
        serie("carotte", planche, 1, 4, Mode::PleineTerre, d(2027, 3, 15)),
    )
    .unwrap();
    let autre = serie("laitue", planche, 3, 6, Mode::Plants, d(2027, 7, 1));
    let PlanError::Conflit {
        avec,
        troncons,
        periode,
    } = plan.ajouter_serie(serie_id(2), autre.clone()).unwrap_err()
    else {
        panic!("a conflict is expected");
    };
    assert_eq!(avec, serie_id(1));
    assert_eq!(troncons, Zone { t1: 3, t2: 4 });
    // Carrot ends 2027-08-02, the lettuce starts 2027-07-01: [max(m), min(f)[.
    assert_eq!(
        periode,
        Occupation {
            debut: d(2027, 7, 1),
            fin: d(2027, 8, 2)
        }
    );
    assert!(!plan.series.contains_key(&serie_id(2)));
}

#[test]
fn series_de_planches_differentes_ne_sont_pas_en_conflit() {
    let (mut plan, planche) = plan_avec_planche();
    let autre_planche = planche_id(200);
    plan.ajouter_planche(
        autre_planche,
        "Planche B".into(),
        2026,
        geometrie(6),
        aujourd_hui(),
    )
    .unwrap();
    plan.ajouter_serie(
        serie_id(1),
        serie("carotte", planche, 1, 6, Mode::PleineTerre, d(2027, 3, 15)),
    )
    .unwrap();
    plan.ajouter_serie(
        serie_id(2),
        serie(
            "carotte",
            autre_planche,
            1,
            6,
            Mode::PleineTerre,
            d(2027, 3, 15),
        ),
    )
    .unwrap();
}

#[test]
fn zone_plus_longue_que_la_planche_est_refusee() {
    let (mut plan, planche) = plan_avec_planche();
    let s = serie("carotte", planche, 1, 7, Mode::PleineTerre, d(2027, 3, 15));
    assert_eq!(
        plan.ajouter_serie(serie_id(1), s).unwrap_err(),
        PlanError::ZoneHorsPlanche {
            annee: 2027,
            troncons: 6,
            zone: zone(1, 7)
        }
    );
}

#[test]
fn serie_sur_une_annee_ou_la_planche_est_supprimee_est_refusee() {
    let (mut plan, planche) = plan_avec_planche();
    plan.supprimer_planche(planche, 2028, aujourd_hui())
        .unwrap();
    // The poireau runs to 2028-02-05: the planche is absent in 2028.
    let s = serie("poireau", planche, 1, 1, Mode::Abri, d(2027, 5, 1));
    assert_eq!(
        plan.ajouter_serie(serie_id(1), s).unwrap_err(),
        PlanError::PlancheAbsente {
            planche_id: planche,
            annee: 2028
        }
    );
    // One that ends on 2028-01-01 does not touch 2028.
    let mut finie = serie("carotte", planche, 1, 1, Mode::PleineTerre, d(2027, 3, 15));
    finie.fin_manuelle = Some(d(2028, 1, 1));
    plan.ajouter_serie(serie_id(2), finie).unwrap();
}

#[test]
fn serie_avant_la_premiere_disposition_est_refusee() {
    let (mut plan, planche) = plan_avec_planche();
    // The planche only exists from 2026.
    let s = serie("carotte", planche, 1, 1, Mode::PleineTerre, d(2025, 3, 15));
    assert!(matches!(
        plan.ajouter_serie(serie_id(1), s),
        Err(PlanError::PlancheAbsente { annee: 2025, .. })
    ));
}

#[test]
fn remplacer_une_serie_ne_la_compare_pas_a_son_ancienne_version() {
    let (mut plan, planche) = plan_avec_planche();
    plan.ajouter_serie(
        serie_id(1),
        serie("carotte", planche, 1, 6, Mode::PleineTerre, d(2027, 3, 15)),
    )
    .unwrap();
    plan.decaler_serie(serie_id(1), 3).unwrap();
    assert_eq!(plan.series[&serie_id(1)].date, d(2027, 3, 18));
}

#[test]
fn ajouter_deux_fois_le_meme_id_est_refuse() {
    let (mut plan, planche) = plan_avec_planche();
    let s = serie("carotte", planche, 1, 2, Mode::PleineTerre, d(2027, 3, 15));
    plan.ajouter_serie(serie_id(1), s.clone()).unwrap();
    assert_eq!(
        plan.ajouter_serie(serie_id(1), s).unwrap_err(),
        PlanError::SerieExistante(serie_id(1))
    );
}

// ---- §5 deleting a planche -------------------------------------------------

#[test]
fn suppression_garde_les_series_finies_au_plus_tard_le_1er_janvier() {
    let (mut plan, planche) = plan_avec_planche();
    let mut finit_le_1er = serie("carotte", planche, 1, 1, Mode::PleineTerre, d(2027, 3, 15));
    finit_le_1er.fin_manuelle = Some(d(2028, 1, 1));
    let mut finit_le_2 = serie("carotte", planche, 2, 2, Mode::PleineTerre, d(2027, 3, 15));
    finit_le_2.fin_manuelle = Some(d(2028, 1, 2));
    plan.ajouter_serie(serie_id(1), finit_le_1er).unwrap();
    // The second one touches 2028, where the planche exists until the deletion.
    plan.ajouter_serie(serie_id(2), finit_le_2).unwrap();

    let supprimees = plan
        .supprimer_planche(planche, 2028, aujourd_hui())
        .unwrap();

    assert_eq!(supprimees, vec![serie_id(2)]);
    assert!(plan.series.contains_key(&serie_id(1)));
}

#[test]
fn suppression_retire_aussi_les_series_futures() {
    let (mut plan, planche) = plan_avec_planche();
    plan.ajouter_serie(
        serie_id(1),
        serie("carotte", planche, 1, 1, Mode::PleineTerre, d(2028, 3, 15)),
    )
    .unwrap();
    let supprimees = plan
        .supprimer_planche(planche, 2028, aujourd_hui())
        .unwrap();
    assert_eq!(supprimees, vec![serie_id(1)]);
}

#[test]
fn suppression_efface_les_dispositions_suivantes_de_cette_planche() {
    let (mut plan, planche) = plan_avec_planche();
    plan.enregistrer_disposition(
        planche,
        2030,
        Disposition::Presente(geometrie(4)),
        aujourd_hui(),
    )
    .unwrap();
    plan.supprimer_planche(planche, 2028, aujourd_hui())
        .unwrap();
    let p = &plan.planches[&planche];
    assert!(p.disposition_en(2027).is_some());
    for annee in [2028, 2029, 2030, 2031] {
        assert!(p.disposition_en(annee).is_none(), "{annee}");
    }
}

#[test]
fn suppression_ne_touche_pas_les_autres_planches() {
    let (mut plan, planche) = plan_avec_planche();
    let autre = planche_id(200);
    plan.ajouter_planche(autre, "Planche B".into(), 2026, geometrie(6), aujourd_hui())
        .unwrap();
    plan.ajouter_serie(
        serie_id(1),
        serie("poireau", autre, 1, 6, Mode::Abri, d(2027, 5, 1)),
    )
    .unwrap();
    assert!(
        plan.supprimer_planche(planche, 2028, aujourd_hui())
            .unwrap()
            .is_empty()
    );
    assert!(plan.series.contains_key(&serie_id(1)));
    assert!(plan.planches[&autre].disposition_en(2029).is_some());
}

// ---- §7 layouts ---------------------------------------------------------------

#[test]
fn modification_vaut_jusqu_a_la_prochaine_disposition_enregistree() {
    let (mut plan, planche) = plan_avec_planche();
    plan.enregistrer_disposition(
        planche,
        2029,
        Disposition::Presente(geometrie(4)),
        aujourd_hui(),
    )
    .unwrap();
    // Modified in 2027: holds for 2028, but 2029 stays as it was.
    plan.enregistrer_disposition(
        planche,
        2027,
        Disposition::Presente(geometrie(8)),
        aujourd_hui(),
    )
    .unwrap();
    let nb = |annee| plan.nb_troncons(planche, annee);
    assert_eq!(
        [nb(2026), nb(2027), nb(2028), nb(2029), nb(2030)],
        [Some(6), Some(8), Some(8), Some(4), Some(4)]
    );
}

#[test]
fn les_autres_planches_ne_sont_pas_touchees_par_une_modification() {
    let (mut plan, planche) = plan_avec_planche();
    let autre = planche_id(200);
    plan.ajouter_planche(autre, "Planche B".into(), 2026, geometrie(6), aujourd_hui())
        .unwrap();
    plan.enregistrer_disposition(
        planche,
        2027,
        Disposition::Presente(geometrie(8)),
        aujourd_hui(),
    )
    .unwrap();
    assert_eq!(plan.nb_troncons(autre, 2027), Some(6));
}

#[test]
fn annee_passee_en_lecture_seule() {
    let (mut plan, planche) = plan_avec_planche();
    let avant = plan.clone();
    let erreur = plan
        .enregistrer_disposition(
            planche,
            2025,
            Disposition::Presente(geometrie(4)),
            aujourd_hui(),
        )
        .unwrap_err();
    assert_eq!(
        erreur,
        PlanError::AnneePassee {
            annee: 2025,
            aujourd_hui: aujourd_hui()
        }
    );
    assert!(matches!(
        plan.supprimer_planche(planche, 2025, aujourd_hui()),
        Err(PlanError::AnneePassee { .. })
    ));
    // The year of today can still be modified.
    plan.enregistrer_disposition(
        planche,
        2026,
        Disposition::Presente(geometrie(4)),
        aujourd_hui(),
    )
    .unwrap();
    assert_ne!(plan, avant);
}

#[test]
fn onzieme_planche_refusee() {
    let (mut plan, _) = plan_avec_planche();
    for n in 1..10u128 {
        plan.ajouter_planche(
            planche_id(200 + n),
            format!("P{n}"),
            2026,
            geometrie(4),
            aujourd_hui(),
        )
        .unwrap();
    }
    assert_eq!(plan.planches_en(2026).len(), 10);
    let avant = plan.clone();
    let erreur = plan
        .ajouter_planche(
            planche_id(999),
            "Trop".into(),
            2026,
            geometrie(4),
            aujourd_hui(),
        )
        .unwrap_err();
    assert_eq!(
        erreur,
        PlanError::TropDePlanches {
            annee: 2026,
            max: 10
        }
    );
    assert_eq!(plan, avant);
    // A planche deleted from 2027 frees a place from 2027 on.
    plan.supprimer_planche(planche_id(100), 2027, aujourd_hui())
        .unwrap();
    plan.ajouter_planche(
        planche_id(999),
        "Ok".into(),
        2027,
        geometrie(4),
        aujourd_hui(),
    )
    .unwrap();
}

#[test]
fn longueur_non_multiple_de_la_case_refusee() {
    let (mut plan, planche) = plan_avec_planche();
    let mut geo = geometrie(4);
    geo.longueur_cm = 210;
    assert_eq!(
        plan.enregistrer_disposition(planche, 2027, Disposition::Presente(geo), aujourd_hui())
            .unwrap_err(),
        PlanError::LongueurInvalide {
            longueur_cm: 210,
            case_cm: 50
        }
    );
}

// ---- §8 free slots -----------------------------------------------------------

/// A planche with a serie of one troncon ending when the slot starts: builds
/// slots of an exact length.
fn plan_avec_occupation(
    zone_occupee: (u32, u32),
    debut: chrono::NaiveDate,
    fin: chrono::NaiveDate,
) -> (Plan, plan::PlancheId) {
    let (mut plan, planche) = plan_avec_planche();
    let mut s = serie(
        "carotte",
        planche,
        zone_occupee.0,
        zone_occupee.1,
        Mode::PleineTerre,
        debut,
    );
    s.fin_manuelle = Some(fin);
    plan.ajouter_serie(serie_id(1), s).unwrap();
    (plan, planche)
}

#[test]
fn creneau_de_35_jours_ecarte_et_de_36_jours_garde() {
    // Occupied from 2027-04-05: the free period 03-01..04-04 is 35 days (31 + 4).
    let (plan, _) = plan_avec_occupation((1, 6), d(2027, 4, 5), d(2027, 11, 20));
    assert!(plan.creneaux(2027).is_empty());
    // One day later: 03-01..04-05 is 36 days.
    let (plan, planche) = plan_avec_occupation((1, 6), d(2027, 4, 6), d(2027, 11, 20));
    assert_eq!(
        plan.creneaux(2027),
        vec![Creneau {
            planche_id: planche,
            annee: 2027,
            zone: zone(1, 6),
            debut: d(2027, 3, 1),
            fin: d(2027, 4, 5)
        }]
    );
}

#[test]
fn creneau_borne_par_la_fenetre_du_1er_mars_au_15_novembre() {
    let (plan, planche) = plan_avec_planche();
    assert_eq!(
        plan.creneaux(2027),
        vec![Creneau {
            planche_id: planche,
            annee: 2027,
            zone: zone(1, 6),
            debut: d(2027, 3, 1),
            fin: d(2027, 11, 15)
        }]
    );
}

#[test]
fn creneaux_de_zones_differentes_maximaux_en_largeur_et_en_temps() {
    // [1,2] busy [04-01, 07-01[ and [5,6] busy [06-01, 09-01[.
    let (mut plan, planche) = plan_avec_planche();
    plan.ajouter_serie(
        serie_id(1),
        occupant("carotte", planche, 1, 2, d(2027, 4, 1), d(2027, 7, 1)),
    )
    .unwrap();
    plan.ajouter_serie(
        serie_id(2),
        occupant("carotte", planche, 5, 6, d(2027, 6, 1), d(2027, 9, 1)),
    )
    .unwrap();
    let attendu = |t1, t2, debut, fin| Creneau {
        planche_id: planche,
        annee: 2027,
        zone: zone(t1, t2),
        debut,
        fin,
    };
    // Dropped: [1,4] and [1,6] 03-01..03-31 (31 days, below the minimum), and
    // [3,6] 09-01..11-15 (it can be widened to [1,6] over that period).
    assert_eq!(
        plan.creneaux(2027),
        vec![
            attendu(1, 4, d(2027, 7, 1), d(2027, 11, 15)),
            attendu(1, 6, d(2027, 9, 1), d(2027, 11, 15)),
            attendu(3, 4, d(2027, 3, 1), d(2027, 11, 15)),
            attendu(3, 6, d(2027, 3, 1), d(2027, 5, 31)),
        ]
    );
}

#[test]
fn creneaux_calcules_pour_chaque_annee_ou_la_planche_existe() {
    let (mut plan, planche) = plan_avec_planche();
    plan.supprimer_planche(planche, 2028, aujourd_hui())
        .unwrap();
    assert!(!plan.creneaux(2027).is_empty());
    assert!(
        plan.creneaux(2028).is_empty(),
        "the planche no longer exists"
    );
    assert!(
        plan.creneaux(2025).is_empty(),
        "the planche does not exist yet"
    );
}

#[test]
fn une_serie_de_l_annee_precedente_occupe_le_debut_de_l_annee() {
    let (mut plan, planche) = plan_avec_planche();
    plan.ajouter_serie(
        serie_id(1),
        serie("poireau", planche, 1, 6, Mode::Abri, d(2027, 5, 1)),
    )
    .unwrap();
    // The leek runs until 2028-02-05: the 2028 window opens on 02-05... which is before 03-01.
    assert_eq!(
        plan.creneaux(2028),
        vec![Creneau {
            planche_id: planche,
            annee: 2028,
            zone: zone(1, 6),
            debut: d(2028, 3, 1),
            fin: d(2028, 11, 15)
        }]
    );
    // In 2027 the leek starts on 07-10: free 03-01..07-09.
    assert_eq!(plan.creneaux(2027)[0].fin, d(2027, 7, 9));
}

// ---- §9 suggestions ------------------------------------------------------------

fn creneau(debut: chrono::NaiveDate, fin: chrono::NaiveDate) -> Creneau {
    Creneau {
        planche_id: planche_id(100),
        annee: debut.year_(),
        zone: zone(1, 6),
        debut,
        fin,
    }
}

trait Annee {
    fn year_(&self) -> i32;
}
impl Annee for chrono::NaiveDate {
    fn year_(&self) -> i32 {
        chrono::Datelike::year(self)
    }
}

#[test]
fn culture_ecartee_si_la_recolte_minimale_ne_tient_pas() {
    // Lettuce, harvest 20 days, minimum 10: r = 2027-09-21 for m = 08-02.
    let assez = creneau(d(2027, 8, 2), d(2027, 9, 30)); // r + 10 = 10-01 = F + 1: fits
    assert!(evaluer(&assez, culture("laitue"), aujourd_hui()).is_some());
    let trop_court = creneau(d(2027, 8, 2), d(2027, 9, 29)); // F + 1 = 09-30 < r + 10
    assert_eq!(evaluer(&trop_court, culture("laitue"), aujourd_hui()), None);
}

#[test]
fn la_mise_en_place_est_cherchee_dans_les_28_premiers_jours() {
    // Tomato window opens 05-01: reachable from 04-04 (D + 27), not from 04-03.
    let ok = creneau(d(2027, 4, 4), d(2027, 11, 15));
    assert_eq!(
        evaluer(&ok, culture("tomate"), d(2027, 1, 1))
            .unwrap()
            .mise_en_place,
        d(2027, 5, 1)
    );
    let trop_tot = creneau(d(2027, 4, 3), d(2027, 11, 15));
    assert_eq!(evaluer(&trop_tot, culture("tomate"), d(2027, 1, 1)), None);
}

#[test]
fn semis_a_l_abri_le_jour_meme_d_aujourd_hui_est_encore_possible() {
    let c = creneau(d(2027, 5, 1), d(2027, 11, 15));
    // Tomato sowing would be 03-06.
    assert_eq!(
        evaluer(&c, culture("tomate"), d(2027, 3, 6)).unwrap().mode,
        Mode::Abri
    );
    assert_eq!(
        evaluer(&c, culture("tomate"), d(2027, 3, 7)).unwrap().mode,
        Mode::Plants
    );
}

#[test]
fn culture_sans_abri_semee_en_pleine_terre() {
    let c = creneau(d(2027, 5, 10), d(2027, 11, 15));
    let haricot = evaluer(&c, culture("haricot-vert"), aujourd_hui()).unwrap();
    assert_eq!(
        (haricot.mode, haricot.date),
        (Mode::PleineTerre, d(2027, 5, 10))
    );
}

#[test]
fn au_plus_3_suggestions_classees_par_remplissage_puis_nom() {
    let (plan, planche) = plan_avec_planche();
    let c = Creneau {
        planche_id: planche,
        annee: 2027,
        zone: zone(1, 6),
        debut: d(2027, 5, 1),
        fin: d(2027, 11, 15),
    };
    let suggestions = plan.suggestions(&c, Catalogue::v1(), d(2027, 4, 20));
    assert_eq!(suggestions.len(), 3);
    let ids: Vec<&str> = suggestions.iter().map(|s| s.culture_id.as_str()).collect();
    // Leek fills the slot (score 1.0, end cut to the slot); then the 0.7 group by name:
    // Carotte, Courgette, Tomate (cut off by the limit of 3).
    assert_eq!(ids, ["poireau", "carotte", "courgette"]);
    assert_eq!(suggestions[0].fin_manuelle, Some(d(2027, 11, 16)));
    assert_eq!(suggestions[0].score_dixiemes(), 10);
}

#[test]
fn a_score_egal_une_autre_famille_que_la_derniere_serie_passe_devant() {
    let ids = |plan: &Plan, planche| -> Vec<String> {
        let c = Creneau {
            planche_id: planche,
            annee: 2027,
            zone: zone(1, 6),
            debut: d(2027, 5, 1),
            fin: d(2027, 11, 15),
        };
        plan.suggestions(&c, Catalogue::v1(), d(2027, 4, 20))
            .into_iter()
            .map(|s| s.culture_id)
            .collect()
    };
    // The last serie before the slot is a tomato (Solanacées): in the 0.7 group
    // (carotte, courgette, tomate) it was already last by name.
    let (mut plan, planche) = plan_avec_planche();
    plan.ajouter_serie(
        serie_id(1),
        occupant("tomate", planche, 1, 6, d(2027, 3, 1), d(2027, 5, 1)),
    )
    .unwrap();
    assert_eq!(ids(&plan, planche), ["poireau", "carotte", "courgette"]);

    // It is a carrot (Apiacées): the carrot goes behind the other crops of its score.
    let (mut plan, planche) = plan_avec_planche();
    plan.ajouter_serie(
        serie_id(1),
        occupant("carotte", planche, 1, 6, d(2027, 3, 1), d(2027, 5, 1)),
    )
    .unwrap();
    assert_eq!(ids(&plan, planche), ["poireau", "courgette", "tomate"]);

    // An earlier serie does not count once a later one has ended.
    plan.ajouter_serie(
        serie_id(2),
        occupant("tomate", planche, 1, 6, d(2027, 1, 1), d(2027, 3, 1)),
    )
    .unwrap();
    assert_eq!(ids(&plan, planche), ["poireau", "courgette", "tomate"]);
}

#[test]
fn suggestion_en_serie_prerempli_la_creation() {
    let c = creneau(d(2027, 8, 2), d(2027, 11, 15));
    let suggestion = evaluer(&c, culture("laitue"), aujourd_hui()).unwrap();
    let serie = suggestion.en_serie(culture("laitue"), &c).unwrap();
    assert_eq!(
        (serie.mode, serie.date, serie.zone()),
        (Mode::Abri, d(2027, 7, 5), zone(1, 6))
    );
    assert_eq!(serie.dates().mise_en_place, suggestion.mise_en_place);
    assert_eq!(serie.dates().fin, suggestion.fin);
}

// ---- §11 tasks -------------------------------------------------------------

#[test]
fn taches_par_mode() {
    let types = |mode| -> Vec<TypeTache> {
        serie("tomate", planche_id(100), 1, 1, mode, d(2027, 3, 10))
            .taches(serie_id(1))
            .into_iter()
            .map(|t| t.type_tache)
            .collect()
    };
    assert_eq!(
        types(Mode::Abri),
        [
            TypeTache::SemerAbri,
            TypeTache::Repiquer,
            TypeTache::Recolter
        ]
    );
    assert_eq!(
        types(Mode::Plants),
        [TypeTache::Planter, TypeTache::Recolter]
    );
    assert_eq!(
        types(Mode::PleineTerre),
        [TypeTache::SemerTerre, TypeTache::Recolter]
    );
}

#[test]
fn dates_des_taches_d_une_serie_a_l_abri() {
    let s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, d(2027, 3, 10));
    let dates: Vec<_> = s.taches(serie_id(1)).into_iter().map(|t| t.date).collect();
    assert_eq!(dates, [d(2027, 3, 10), d(2027, 5, 5), d(2027, 7, 19)]);
}

#[test]
fn identifiant_de_tache_stable_quand_la_date_bouge() {
    let id = serie_id(7);
    let s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, d(2027, 3, 10));
    let avant = s.taches(id);
    let apres = s.decalee(10).taches(id);
    let ids = |t: &[plan::Tache]| t.iter().map(|t| t.id.clone()).collect::<Vec<_>>();
    assert_eq!(ids(&avant), ids(&apres));
    assert_ne!(avant[0].date, apres[0].date);
    assert_eq!(avant[0].id, format!("{id}/semer-abri"));
    assert_eq!(avant[2].id, format!("{id}/recolter"));
}

#[test]
fn calendrier_des_taches_par_mois() {
    let (mut plan, planche) = plan_avec_planche();
    plan.ajouter_serie(
        serie_id(1),
        serie("tomate", planche, 1, 3, Mode::Abri, d(2027, 3, 10)),
    )
    .unwrap();
    let calendrier = plan.calendrier_par_mois();
    let mois: Vec<_> = calendrier.keys().copied().collect();
    assert_eq!(mois, [(2027, 3), (2027, 5), (2027, 7)]);
    assert_eq!(calendrier[&(2027, 5)][0].type_tache, TypeTache::Repiquer);
}

#[test]
fn familles_des_deux_saisons_precedentes() {
    let (mut plan, planche) = plan_avec_planche();
    // 2027: tomato (Solanacées) and lettuce; 2028: carrot; 2026 would be Y−2 of 2028.
    plan.ajouter_serie(
        serie_id(1),
        serie("tomate", planche, 1, 3, Mode::Abri, d(2027, 3, 10)),
    )
    .unwrap();
    plan.ajouter_serie(
        serie_id(2),
        serie("carotte", planche, 4, 6, Mode::PleineTerre, d(2028, 3, 15)),
    )
    .unwrap();
    plan.ajouter_serie(
        serie_id(3),
        serie("laitue", planche, 4, 6, Mode::Plants, d(2026, 4, 1)),
    )
    .unwrap();
    let familles = |annee| plan.familles_precedentes(planche, annee, Catalogue::v1());
    assert_eq!(familles(2028), ["Astéracées", "Solanacées"]);
    assert_eq!(familles(2029), ["Apiacées", "Solanacées"]);
    assert_eq!(familles(2030), ["Apiacées"]);
    assert!(familles(2031).is_empty());
}

// ---- §1 and S05 §5: model --------------------------------------------------------

#[test]
fn le_plan_se_serialise_sans_donnee_derivee_et_se_relit() {
    let (mut plan, planche) = plan_avec_planche();
    plan.ajouter_serie(
        serie_id(1),
        serie("tomate", planche, 1, 3, Mode::Abri, d(2027, 3, 10)),
    )
    .unwrap();
    plan.enregistrer_disposition(planche, 2028, Disposition::Absente, aujourd_hui())
        .unwrap();

    let json = serde_json::to_value(&plan).unwrap();
    // Collections are maps indexed by id, never lists.
    assert!(json["planches"].is_object());
    assert!(json["series"].is_object());
    assert!(json["annotations"].is_object());
    assert!(json["planches"][planche.to_string()]["dispositions"]["2026"].is_object());
    // Only entered values are stored: no m, r, f, quantity, slot or task.
    let serie_json = &json["series"][serie_id(1).to_string()];
    let mut cles: Vec<_> = serie_json.as_object().unwrap().keys().cloned().collect();
    cles.sort();
    assert_eq!(
        cles,
        [
            "abri_jours",
            "apres_jours",
            "culture_id",
            "date",
            "duree_jours",
            "mode",
            "planche_id",
            "troncon_debut",
            "troncon_fin"
        ]
    );
    assert_eq!(serie_json["mode"], "abri");
    assert_eq!(serie_json["date"], "2027-03-10");

    let relu: Plan = serde_json::from_value(json).unwrap();
    assert_eq!(relu, plan);
}

#[test]
fn horodatage_technique_en_utc() {
    let (mut plan, _) = plan_avec_planche();
    assert_eq!(plan.version, 1);
    plan.toucher(maintenant() + chrono::Duration::hours(1));
    assert_eq!(plan.version, 2);
    let json = serde_json::to_value(&plan).unwrap();
    assert!(json["mis_a_jour_le"].as_str().unwrap().ends_with('Z'));
}

#[test]
fn les_ids_generes_sont_des_uuid_v7_distincts() {
    let (a, b) = (plan::SerieId::nouveau(), plan::SerieId::nouveau());
    assert_ne!(a, b);
    assert_eq!(a.uuid().get_version_num(), 7);
}

#[test]
fn codes_d_erreur_stables_pour_l_interface() {
    assert_eq!(PlanError::GodetsRequis.code(), "godets_requis");
    assert_eq!(
        PlanError::TropDePlanches {
            annee: 2027,
            max: 10
        }
        .code(),
        "trop_de_planches"
    );
}

#[test]
fn zone_type_est_reexporte() {
    assert_eq!(Zone::new(1, 2).unwrap().nb_troncons(), 2);
}
