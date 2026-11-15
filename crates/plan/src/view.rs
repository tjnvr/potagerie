//! Year view (S05 §5): what the commands return to the interface.
//!
//! It is computed from the document, the catalogue and today, and never
//! stored. It only assembles the calculations of the other modules (S03): it
//! adds no rule of its own.

use catalogue::Catalogue;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::Serialize;

use crate::ids::{AnnotationId, PlancheId, SerieId};
use crate::model::{Plan, TypeAnnotation, TypePlanche};
use crate::serie::Mode;
use crate::stages::{Appearance, AppearanceChange};
use crate::tasks::TaskType;
use crate::zone::Occupation;

/// The year view.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct YearView {
    pub year: i32,
    pub today: NaiveDate,
    /// Past years are read-only (S03 §7).
    pub read_only: bool,
    /// `None` until the terrain is created (E2): there is no plan yet.
    pub terrain: Option<TerrainView>,
    /// `version` of the plan, 0 when there is none. Exported as a JS number.
    pub version: u32,
    pub updated_at: Option<DateTime<Utc>>,
    /// The planches that exist in `year`, with their layout of that year.
    pub planches: Vec<PlancheView>,
    /// The series whose timeline (pots included) touches `year`.
    pub series: Vec<SerieView>,
    /// Conflicting pairs among `series` (S03 §5).
    pub conflicts: Vec<ConflictView>,
    /// The series that ended on or before January 1st of `year`: the past
    /// seasons of the planche panel (S01 F-VIS-7), read-only.
    pub historique: Vec<SerieView>,
    /// Free slots of `year` and their suggestions (S03 §8, §9).
    pub creneaux: Vec<CreneauView>,
    /// Annotations that touch `year`.
    pub annotations: Vec<AnnotationView>,
    /// Tasks dated in `year` (S03 §11).
    pub tasks: Vec<TaskView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TerrainView {
    pub longueur_cm: u32,
    pub largeur_cm: u32,
    pub case_cm: u32,
    pub allee_cm: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PlancheView {
    pub id: PlancheId,
    pub nom: String,
    pub planche_type: TypePlanche,
    pub x: u32,
    pub y: u32,
    pub longueur_cm: u32,
    pub largeur_cm: u32,
    /// `n = longueur ÷ case` (S03 §4).
    pub troncons: u32,
    /// The series deleted with this planche if it is deleted from this year
    /// on (S03 §5): the interface warns before confirming.
    pub series_supprimees: Vec<SerieId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SerieView {
    pub id: SerieId,
    pub culture_id: String,
    pub planche_id: PlancheId,
    pub troncon_debut: u32,
    pub troncon_fin: u32,
    pub mode: Mode,
    /// Entered date: sowing (abri, pleine terre) or planting (plants).
    pub date: NaiveDate,
    pub sowing: Option<NaiveDate>,
    /// `m`.
    pub planting_out: NaiveDate,
    /// `r`.
    pub harvest_start: NaiveDate,
    /// `f`.
    pub end: NaiveDate,
    /// True when `end` was modified by hand.
    pub manual_end: bool,
    /// Days in pots (mode abri): from the catalogue, or entered for a crop
    /// without pots (S03 §2). The edit form shows it back.
    pub pot_days: u32,
    /// Rows × plants per row on the planche (S03 §6); 0 when the planche has
    /// no layout to measure.
    pub rangs: u32,
    pub pieds_par_rang: u32,
    /// What the garden view draws today (S03 §3).
    pub appearance_today: Appearance,
    /// Every change of display over the life of the serie (S03 §3), so the
    /// garden view can show any date, and play the season, without
    /// calculating a stage itself.
    pub appearance_changes: Vec<AppearanceChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ConflictView {
    pub a: SerieId,
    pub b: SerieId,
    pub troncon_debut: u32,
    pub troncon_fin: u32,
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CreneauView {
    pub planche_id: PlancheId,
    pub troncon_debut: u32,
    pub troncon_fin: u32,
    /// First free day.
    pub start: NaiveDate,
    /// Last free day, included.
    pub end: NaiveDate,
    pub suggestions: Vec<SuggestionView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SuggestionView {
    pub culture_id: String,
    pub mode: Mode,
    pub date: NaiveDate,
    pub planting_out: NaiveDate,
    pub harvest_start: NaiveDate,
    pub end: NaiveDate,
    pub fin_manuelle: Option<NaiveDate>,
    /// Filling score rounded to a tenth, in tenths (0 to 10).
    pub score_dixiemes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AnnotationView {
    pub id: AnnotationId,
    pub annotation_type: TypeAnnotation,
    pub date_debut: NaiveDate,
    pub date_fin: Option<NaiveDate>,
    pub planche_id: Option<PlancheId>,
    pub texte: String,
    pub masquee: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TaskView {
    pub id: String,
    pub serie_id: SerieId,
    pub task_type: TaskType,
    pub date: NaiveDate,
}

/// The view of one serie, with its quantities on the planche of the year it
/// starts in (S03 §6).
fn serie_view(
    plan: &Plan,
    id: SerieId,
    s: &crate::serie::Serie,
    today: NaiveDate,
    catalogue: &Catalogue,
) -> SerieView {
    let dates = s.dates();
    let start = dates
        .sowing
        .unwrap_or(dates.planting_out)
        .min(dates.planting_out);
    let largeur = plan
        .planches
        .get(&s.planche_id)
        .and_then(|p| p.disposition_in(start.year()))
        .map(|g| g.largeur_cm);
    let (rangs, pieds_par_rang) = match (largeur, catalogue.culture(&s.culture_id)) {
        (Some(largeur), Some(culture)) => {
            let q = crate::quantities::quantity(
                largeur,
                s.troncon_fin - s.troncon_debut + 1,
                plan.terrain.case_cm,
                culture,
            );
            (q.rows, q.plants_per_row)
        }
        _ => (0, 0),
    };
    SerieView {
        id,
        culture_id: s.culture_id.clone(),
        planche_id: s.planche_id,
        troncon_debut: s.troncon_debut,
        troncon_fin: s.troncon_fin,
        mode: s.mode,
        date: s.date,
        sowing: dates.sowing,
        planting_out: dates.planting_out,
        harvest_start: dates.harvest_start,
        end: dates.end,
        manual_end: s.fin_manuelle.is_some(),
        pot_days: s.abri_jours,
        rangs,
        pieds_par_rang,
        appearance_today: s.appearance_on(today),
        appearance_changes: s.appearance_changes(),
    }
}

impl YearView {
    /// The view of `year`. `plan` is `None` before the terrain is created.
    pub fn build(plan: Option<&Plan>, year: i32, today: NaiveDate, catalogue: &Catalogue) -> Self {
        let read_only = year < today.year();
        let Some(plan) = plan else {
            return Self {
                year,
                today,
                read_only,
                terrain: None,
                version: 0,
                updated_at: None,
                planches: Vec::new(),
                series: Vec::new(),
                historique: Vec::new(),
                conflicts: Vec::new(),
                creneaux: Vec::new(),
                annotations: Vec::new(),
                tasks: Vec::new(),
            };
        };

        let planches = plan
            .planches_in(year)
            .into_iter()
            .map(|(id, planche, geometry)| PlancheView {
                id,
                nom: planche.nom.clone(),
                planche_type: geometry.planche_type,
                x: geometry.x,
                y: geometry.y,
                longueur_cm: geometry.longueur_cm,
                largeur_cm: geometry.largeur_cm,
                troncons: geometry.troncons(plan.terrain.case_cm),
                series_supprimees: plan.series_deleted_by(id, year),
            })
            .collect();

        let series: Vec<SerieView> = plan
            .series
            .iter()
            .filter(|(_, s)| {
                let dates = s.dates();
                Occupation {
                    start: dates
                        .sowing
                        .unwrap_or(dates.planting_out)
                        .min(dates.planting_out),
                    end: dates.end,
                }
                .touches_year(year)
            })
            .map(|(id, s)| serie_view(plan, *id, s, today, catalogue))
            .collect();

        let january_first = crate::days::january_first(year);
        let historique = plan
            .series
            .iter()
            .filter(|(_, s)| s.dates().end <= january_first)
            .map(|(id, s)| serie_view(plan, *id, s, today, catalogue))
            .collect();

        let shown = |id: &SerieId| series.iter().any(|s| s.id == *id);
        let conflicts = plan
            .conflicts()
            .into_iter()
            .filter(|(a, b, _)| shown(a) || shown(b))
            .map(|(a, b, detail)| ConflictView {
                a,
                b,
                troncon_debut: detail.troncons.t1,
                troncon_fin: detail.troncons.t2,
                start: detail.period.start,
                end: detail.period.end,
            })
            .collect();

        let creneaux = plan
            .creneaux(year)
            .iter()
            .map(|creneau| CreneauView {
                planche_id: creneau.planche_id,
                troncon_debut: creneau.zone.t1,
                troncon_fin: creneau.zone.t2,
                start: creneau.start,
                end: creneau.end,
                suggestions: plan
                    .suggestions(creneau, catalogue, today)
                    .into_iter()
                    .map(|s| SuggestionView {
                        score_dixiemes: u32::try_from(s.score_dixiemes()).unwrap_or(0),
                        culture_id: s.culture_id,
                        mode: s.mode,
                        date: s.date,
                        planting_out: s.planting_out,
                        harvest_start: s.harvest_start,
                        end: s.end,
                        fin_manuelle: s.fin_manuelle,
                    })
                    .collect(),
            })
            .collect();

        let annotations = plan
            .annotations
            .iter()
            .filter(|(_, a)| {
                let end = a.date_fin.unwrap_or(a.date_debut);
                (a.date_debut.year()..=end.year()).contains(&year)
            })
            .map(|(id, a)| AnnotationView {
                id: *id,
                annotation_type: a.type_annotation,
                date_debut: a.date_debut,
                date_fin: a.date_fin,
                planche_id: a.planche_id,
                texte: a.texte.clone(),
                masquee: a.masquee,
            })
            .collect();

        let tasks = plan
            .tasks()
            .into_iter()
            .filter(|t| t.date.year() == year)
            .map(|t| TaskView {
                id: t.id,
                serie_id: t.serie_id,
                task_type: t.task_type,
                date: t.date,
            })
            .collect();

        Self {
            year,
            today,
            read_only,
            terrain: Some(TerrainView {
                longueur_cm: plan.terrain.longueur_cm,
                largeur_cm: plan.terrain.largeur_cm,
                case_cm: plan.terrain.case_cm.cm(),
                allee_cm: plan.terrain.allee_cm,
            }),
            version: u32::try_from(plan.version).unwrap_or(u32::MAX),
            updated_at: Some(plan.mis_a_jour_le),
            planches,
            series,
            historique,
            conflicts,
            creneaux,
            annotations,
            tasks,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::ids::PlanId;
    use crate::model::{CaseCm, Geometry, Terrain};
    use crate::serie::Serie;
    use crate::zone::Zone;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn plan_with_one_planche() -> (Plan, PlancheId) {
        let now = Utc.with_ymd_and_hms(2027, 1, 10, 8, 0, 0).unwrap();
        let mut plan = Plan::new(
            PlanId::generate(),
            Terrain {
                longueur_cm: 1100,
                largeur_cm: 650,
                case_cm: CaseCm::C50,
                allee_cm: 50,
            },
            now,
        );
        let id = PlancheId::generate();
        plan.add_planche(
            id,
            "Planche A".into(),
            2027,
            Geometry {
                planche_type: TypePlanche::Planche,
                x: 50,
                y: 50,
                longueur_cm: 300,
                largeur_cm: 120,
            },
            day(2027, 1, 10),
        )
        .unwrap();
        (plan, id)
    }

    #[test]
    fn no_plan_gives_an_empty_view() {
        let view = YearView::build(None, 2027, day(2027, 6, 20), Catalogue::reference());
        assert_eq!(view.terrain, None);
        assert!(view.planches.is_empty());
        assert!(!view.read_only);
        assert_eq!(view.version, 0);
    }

    #[test]
    fn past_years_are_read_only() {
        let view = YearView::build(None, 2026, day(2027, 6, 20), Catalogue::reference());
        assert!(view.read_only);
    }

    #[test]
    fn planches_follow_their_dispositions() {
        let (plan, id) = plan_with_one_planche();
        let today = day(2027, 1, 10);
        let view = YearView::build(Some(&plan), 2027, today, Catalogue::reference());
        assert_eq!(view.planches.len(), 1);
        assert_eq!(view.planches[0].id, id);
        assert_eq!(view.planches[0].troncons, 6);
        let before = YearView::build(Some(&plan), 2026, today, Catalogue::reference());
        assert!(before.planches.is_empty());
        let after = YearView::build(Some(&plan), 2028, today, Catalogue::reference());
        assert_eq!(after.planches.len(), 1);
    }

    #[test]
    fn series_carry_their_calculated_dates() {
        let (mut plan, planche_id) = plan_with_one_planche();
        let tomate = Catalogue::reference().culture("tomate").unwrap();
        let serie = Serie::new(
            tomate,
            planche_id,
            Zone::new(1, 3).unwrap(),
            Mode::Abri,
            day(2027, 3, 10),
            None,
            None,
        )
        .unwrap();
        let expected = serie.dates();
        plan.add_serie(SerieId::generate(), serie).unwrap();

        let view = YearView::build(Some(&plan), 2027, day(2027, 6, 20), Catalogue::reference());
        assert_eq!(view.series.len(), 1);
        assert_eq!(view.series[0].planting_out, expected.planting_out);
        assert_eq!(view.series[0].end, expected.end);
        assert_eq!(view.tasks.len(), 3);
        assert!(view.conflicts.is_empty());
        let other_year =
            YearView::build(Some(&plan), 2029, day(2027, 6, 20), Catalogue::reference());
        assert!(other_year.series.is_empty());
    }

    #[test]
    fn deletion_warning_quantities_and_history() {
        let (mut plan, planche_id) = plan_with_one_planche();
        let carotte = Catalogue::reference().culture("carotte").unwrap();
        let serie = |date| {
            Serie::new(
                carotte,
                planche_id,
                Zone::new(1, 2).unwrap(),
                Mode::PleineTerre,
                date,
                None,
                None,
            )
            .unwrap()
        };
        let in_2027 = SerieId::generate();
        plan.add_serie(in_2027, serie(day(2027, 3, 15))).unwrap();
        let view = YearView::build(Some(&plan), 2027, day(2027, 1, 10), Catalogue::reference());
        assert_eq!(view.planches[0].series_supprimees, vec![in_2027]);
        // 2 troncons of 50 cm on 1.2 m: 4 rows (30 cm) of 100 / 4 cm plants (S03 T4).
        assert_eq!(
            (view.series[0].rangs, view.series[0].pieds_par_rang),
            (4, 25)
        );
        assert!(view.historique.is_empty());
        // The 2027 serie is history in 2028 and is no longer deleted by a 2028 deletion.
        let next = YearView::build(Some(&plan), 2028, day(2027, 1, 10), Catalogue::reference());
        assert_eq!(next.historique.len(), 1);
        assert!(next.planches[0].series_supprimees.is_empty());
    }
}
