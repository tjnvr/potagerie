//! Unusual but valid gardens: empty, full, long accented names, notes at year
//! boundaries, long plantings, Unicode edge cases. Each one must survive an
//! export then an import unchanged, with a stable byte output, and the views
//! built from it must not panic.

use std::collections::BTreeMap;
use std::path::Path;

use catalogue::Catalogue;
use chrono::{DateTime, Duration, NaiveDate};
use export::{export_garden, read_garden, read_import};
use plan::{
    Annotation, AnnotationId, Bed, BedId, CellCm, Geometry, Layout, Mode, Plan, PlanId, Planting,
    PlantingId, Plot, TypeAnnotation, TypeBed, YearView,
};

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// A plot with room for ten 10 m beds, one under the other.
fn plan() -> Plan {
    Plan::new(
        PlanId::generate(),
        Plot {
            length_cm: 1500,
            width_cm: 2000,
            cell_cm: CellCm::C50,
            aisle_cm: 50,
        },
        DateTime::from_timestamp(1_800_000_000, 0).unwrap(),
    )
}

fn bed(plan: &mut Plan, index: u32, name: &str, year: i32) -> BedId {
    let id = BedId::generate();
    let geometry = Geometry {
        bed_type: TypeBed::Bed,
        x: 0,
        y: index * 150,
        length_cm: 1000,
        width_cm: 100,
    };
    plan.beds.insert(
        id,
        Bed {
            name: name.to_owned(),
            layouts: BTreeMap::from([(year, Layout::Present(geometry))]),
        },
    );
    id
}

fn planting(bed_id: BedId, t1: u32, t2: u32, date: NaiveDate, days: [u32; 3]) -> Planting {
    Planting {
        crop_id: "tomato".into(),
        bed_id,
        segment_start: t1,
        segment_end: t2,
        mode: Mode::Plants,
        date,
        indoor_days: days[0],
        after_days: days[1],
        duration_days: days[2],
        perennial: None,
        manual_end: None,
    }
}

fn note(date: NaiveDate, text: &str) -> Annotation {
    Annotation {
        type_annotation: TypeAnnotation::Note,
        date_start: date,
        date_end: None,
        bed_id: None,
        text: text.into(),
        hidden: false,
    }
}

fn period(start: NaiveDate, end: NaiveDate, text: &str) -> Annotation {
    Annotation {
        type_annotation: TypeAnnotation::Period,
        date_start: start,
        date_end: Some(end),
        bed_id: None,
        text: text.into(),
        hidden: false,
    }
}

fn add_note(plan: &mut Plan, annotation: Annotation) {
    plan.annotations
        .insert(AnnotationId::generate(), annotation);
}

fn export_to(plan: &Plan, dir: &Path, name: &str) -> Vec<u8> {
    let path = dir.join(name);
    export_garden(plan, &path).unwrap();
    std::fs::read(path).unwrap()
}

/// Export, import, compare; export again and compare the bytes; build the
/// views of every year the plan touches. Returns the imported plan.
fn round_trip(plan: &Plan) -> Plan {
    let dir = tempfile::tempdir().unwrap();
    let first = export_to(plan, dir.path(), "a.json");
    let back = read_garden(&dir.path().join("a.json")).unwrap();
    assert_eq!(&back, plan, "the round trip changed the data");
    let second = export_to(&back, dir.path(), "b.json");
    assert_eq!(first, second, "the export is not stable");
    assert!(std::str::from_utf8(&first).is_ok());
    let today = day(2027, 6, 1);
    for year in [2026, 2027, 2028, 2029, 2030] {
        let _ = YearView::build(Some(&back), year, today, Catalogue::v1());
    }
    back
}

#[test]
fn an_empty_garden_round_trips() {
    let plan = plan();
    let dir = tempfile::tempdir().unwrap();
    round_trip(&plan);
    export_garden(&plan, &dir.path().join("g.json")).unwrap();
    let summary = read_import(&dir.path().join("g.json"), None).unwrap();
    assert_eq!((summary.bed_count, summary.planting_count), (0, 0));
    assert!(summary.incoherences.is_empty());
}

#[test]
fn ten_beds_each_fully_planted_make_140_plantings_without_incoherence() {
    let mut plan = plan();
    let mut count = 0;
    for b in 0..10 {
        let id = bed(&mut plan, b, &format!("Planche {b}"), 2027);
        // 14 plantings one after the other on the whole bed (20 segments),
        // spread over several years.
        let mut start = day(2027, 3, 1);
        for _ in 0..14 {
            plan.plantings.insert(
                PlantingId::generate(),
                planting(id, 1, 20, start, [0, 60, 30]),
            );
            start += Duration::days(91);
            count += 1;
        }
    }
    assert_eq!(count, 140);
    let back = round_trip(&plan);
    let dir = tempfile::tempdir().unwrap();
    export_garden(&back, &dir.path().join("g.json")).unwrap();
    let summary = read_import(&dir.path().join("g.json"), Some(&plan)).unwrap();
    assert_eq!(summary.planting_count, 140);
    assert_eq!(summary.bed_count, 10);
    assert!(
        summary.incoherences.is_empty(),
        "{:?}",
        summary.incoherences
    );
    assert!(!summary.older);
}

#[test]
fn many_small_plantings_on_one_bed_round_trip() {
    let mut plan = plan();
    let id = bed(&mut plan, 0, "Longue", 2027);
    // 20 segments x 7 sequential slots = 140 plantings on a single bed.
    for slot in 0..7 {
        for t in 1..=20 {
            let date = day(2027, 3, 1) + Duration::days(60 * slot);
            plan.plantings.insert(
                PlantingId::generate(),
                planting(id, t, t, date, [0, 40, 19]),
            );
        }
    }
    let back = round_trip(&plan);
    assert_eq!(back.plantings.len(), 140);
    assert!(back.incoherences().is_empty());
}

#[test]
fn long_accented_names_and_notes_round_trip_untouched() {
    let mut plan = plan();
    // 30 characters, the longest bed name.
    let name: String = "Éàçüîôœ-Ñandú ’Été’ « Œillet » "
        .chars()
        .cycle()
        .take(30)
        .collect();
    assert_eq!(name.chars().count(), 30);
    let id = bed(&mut plan, 0, &name, 2027);
    // 80 characters, the longest note.
    let text: String = "Rentrer les poivrons avant les gelées d’octobre – œufs/été "
        .chars()
        .cycle()
        .take(80)
        .collect();
    assert_eq!(text.chars().count(), 80);
    add_note(&mut plan, note(day(2027, 10, 1), &text));
    let mut with_bed = note(day(2027, 10, 2), &text);
    with_bed.bed_id = Some(id);
    add_note(&mut plan, with_bed);
    let back = round_trip(&plan);
    assert!(back.incoherences().is_empty());
    assert_eq!(back.beds[&id].name, name);
}

#[test]
fn names_beyond_the_documented_limits_are_kept_and_flagged_not_lost() {
    // A hand-edited file: a 5000-character bed name, an unknown crop id of
    // 2000 characters and a note of 500. The import accepts it (S03 §13).
    let mut plan = plan();
    let id = bed(&mut plan, 0, &"é".repeat(5000), 2027);
    let mut p = planting(id, 1, 2, day(2027, 5, 1), [0, 30, 30]);
    p.crop_id = "à".repeat(2000);
    plan.plantings.insert(PlantingId::generate(), p);
    add_note(&mut plan, note(day(2027, 5, 1), &"ü".repeat(500)));
    let back = round_trip(&plan);
    let codes: Vec<_> = back.incoherences().iter().map(|i| i.code()).collect();
    assert!(codes.contains(&"crop_unknown"), "{codes:?}");
    assert!(codes.contains(&"note_invalid"), "{codes:?}");
}

#[test]
fn notes_and_periods_on_year_boundaries_round_trip() {
    let mut plan = plan();
    let id = bed(&mut plan, 0, "A", 2027);
    for (y, m, d) in [(2027, 1, 1), (2027, 12, 31), (2028, 2, 29), (2028, 12, 31)] {
        add_note(&mut plan, note(day(y, m, d), "Limite"));
    }
    // A period that ends on the first day of the year, one that starts on the
    // last, one of a single day, one of five years, one on the 29th of February.
    add_note(
        &mut plan,
        period(day(2026, 12, 31), day(2027, 1, 1), "Pont"),
    );
    add_note(
        &mut plan,
        period(day(2027, 12, 31), day(2028, 1, 1), "Pont 2"),
    );
    add_note(
        &mut plan,
        period(day(2027, 6, 15), day(2027, 6, 15), "Un jour"),
    );
    add_note(
        &mut plan,
        period(day(2027, 1, 1), day(2031, 12, 31), "Cinq ans"),
    );
    add_note(
        &mut plan,
        period(day(2028, 2, 29), day(2028, 2, 29), "Bissextile"),
    );
    let mut on_bed = period(day(2027, 1, 1), day(2027, 12, 31), "Toute l’année");
    on_bed.bed_id = Some(id);
    add_note(&mut plan, on_bed);
    let back = round_trip(&plan);
    assert!(back.incoherences().is_empty(), "{:?}", back.incoherences());
    assert_eq!(back.annotations.len(), 10);
}

#[test]
fn the_extreme_years_of_the_calendar_round_trip() {
    let mut plan = plan();
    bed(&mut plan, 0, "A", 1900);
    bed(&mut plan, 1, "B", 2200);
    add_note(&mut plan, note(day(1900, 1, 1), "Début"));
    add_note(&mut plan, note(day(2200, 12, 31), "Fin"));
    let back = round_trip(&plan);
    let _ = YearView::build(Some(&back), 1900, day(1900, 1, 1), Catalogue::v1());
    let _ = YearView::build(Some(&back), 2200, day(2200, 1, 1), Catalogue::v1());
}

#[test]
fn a_planting_spanning_three_years_round_trips() {
    let mut plan = plan();
    let id = bed(&mut plan, 0, "Asperges", 2027);
    // Planted in November 2027, 800 days to harvest, 400 days of harvest:
    // it touches 2027, 2028, 2029 and 2030 (artichokes, asparagus).
    let p = planting(id, 1, 20, day(2027, 11, 15), [0, 800, 400]);
    let years: Vec<i32> = p.occupation().years().collect();
    assert!(years.len() >= 3, "{years:?}");
    plan.plantings.insert(PlantingId::generate(), p);
    // The same with a manual end and pots.
    let mut q = planting(id, 1, 20, day(2033, 1, 1), [30, 100, 200]);
    q.mode = Mode::Indoor;
    q.manual_end = Some(day(2034, 12, 31));
    plan.plantings.insert(PlantingId::generate(), q);
    let back = round_trip(&plan);
    assert!(back.incoherences().is_empty(), "{:?}", back.incoherences());
}

#[test]
fn a_planting_of_one_day_and_one_of_ten_years_round_trip() {
    let mut plan = plan();
    let id = bed(&mut plan, 0, "A", 2027);
    // Harvest of 1 day, no growth: the shortest occupation that is not empty.
    plan.plantings.insert(
        PlantingId::generate(),
        planting(id, 1, 1, day(2027, 5, 1), [0, 1, 1]),
    );
    // 3650 days (the maximum) of growth and 3650 of harvest.
    plan.plantings.insert(
        PlantingId::generate(),
        planting(id, 3, 4, day(2028, 1, 1), [0, 3650, 3650]),
    );
    let back = round_trip(&plan);
    assert!(back.incoherences().is_empty(), "{:?}", back.incoherences());
}

#[test]
fn unicode_edge_cases_in_texts_survive_the_file() {
    let texts = [
        // Composed and decomposed accents are different strings: neither may be
        // normalised by the file.
        "Épinards".to_owned(),
        "E\u{301}pinards".to_owned(),
        "👩\u{200d}🌾 jardinière 🍅".to_owned(),
        "שלום עולם mélangé عربي".to_owned(),
        "tab\tnew\nline \"quote\" \\ back/slash".to_owned(),
        "\u{0}\u{1}\u{1f}control".to_owned(),
        "\u{feff}bom inside".to_owned(),
        "line\u{2028}sep\u{2029}para".to_owned(),
        "\u{200b}zero\u{200b}width".to_owned(),
        "<b>&amp;</b> ]]> <!-- -->".to_owned(),
        "𝔘𝔫𝔦𝔠𝔬𝔡𝔢 \u{10FFFF}".to_owned(),
        "ǅ ß İ ŉ".to_owned(),
    ];
    let mut plan = plan();
    for (i, text) in texts.iter().enumerate() {
        bed(&mut plan, i as u32 % 10, text, 2027);
        add_note(&mut plan, note(day(2027, 3, 1 + i as u32), text));
    }
    // 12 beds is too many beds (flagged, kept).
    let back = round_trip(&plan);
    let names: Vec<&String> = back.beds.values().map(|b| &b.name).collect();
    for text in &texts {
        assert!(names.contains(&text), "{text:?} lost");
    }
}

#[test]
fn a_whitespace_only_or_untrimmed_note_is_flagged_not_rewritten() {
    let mut plan = plan();
    add_note(&mut plan, note(day(2027, 3, 1), "   "));
    add_note(&mut plan, note(day(2027, 3, 2), "\u{a0}"));
    add_note(&mut plan, note(day(2027, 3, 3), "  espaces  "));
    let back = round_trip(&plan);
    assert_eq!(back.incoherences().len(), 2);
}

#[test]
fn the_file_is_the_same_whatever_the_insertion_order() {
    // Maps are ordered, so two plans with the same content give the same bytes.
    let dir = tempfile::tempdir().unwrap();
    let mut a = plan();
    let mut b = a.clone();
    let ids: Vec<BedId> = (0..5).map(|_| BedId::generate()).collect();
    let make = |i: usize| Bed {
        name: format!("B{i}"),
        layouts: BTreeMap::from([(
            2027,
            Layout::Present(Geometry {
                bed_type: TypeBed::Square,
                x: 0,
                y: i as u32 * 150,
                length_cm: 500,
                width_cm: 100,
            }),
        )]),
    };
    for (i, id) in ids.iter().enumerate() {
        a.beds.insert(*id, make(i));
    }
    for (i, id) in ids.iter().enumerate().rev() {
        b.beds.insert(*id, make(i));
    }
    assert_eq!(
        export_to(&a, dir.path(), "a.json"),
        export_to(&b, dir.path(), "b.json")
    );
}

#[test]
fn a_planting_with_an_enormous_zone_is_refused_or_harmless() {
    let mut plan = plan();
    let id = bed(&mut plan, 0, "A", 2027);
    plan.plantings.insert(
        PlantingId::generate(),
        planting(id, 1, u32::MAX, day(2027, 5, 1), [0, 30, 30]),
    );
    let dir = tempfile::tempdir().unwrap();
    export_garden(&plan, &dir.path().join("g.json")).unwrap();
    let started = std::time::Instant::now();
    if let Ok(back) = read_garden(&dir.path().join("g.json")) {
        let _ = back.incoherences();
        let _ = YearView::build(Some(&back), 2027, day(2027, 6, 1), Catalogue::v1());
    }
    assert!(started.elapsed().as_secs() < 20);
}
