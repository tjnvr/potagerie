//! The timeline as the interface serialises it (S05 §4): `fixtures/timeline.svg`
//! is the output of `serializeTimeline` (front/src/lib/timeline/exportSvg.ts) on the
//! example garden. It must be standalone, rendered with the embedded fonts and
//! accepted by both converters. To refresh it, serialise a drawn timeline with
//! the dev fake (`?example&date=2027-06-20`) and replace the file.

use export::{TimelineFormat, render_timeline};

const TIMELINE: &str = include_str!("fixtures/timeline.svg");

#[test]
fn the_serialised_timeline_has_no_external_dependency() {
    assert!(TIMELINE.starts_with("<?xml"));
    for forbidden in [
        "<style", " style=", " class=", "var(--", "href=", "@import", "<image",
    ] {
        assert!(
            !TIMELINE.contains(forbidden),
            "the exported SVG must not contain {forbidden:?}"
        );
    }
    assert!(!TIMELINE.contains("http://") || TIMELINE.matches("http://").count() == 1);
    assert!(TIMELINE.contains("font-family=\"Figtree"));
}

#[test]
fn it_renders_to_a_png_at_twice_its_size() {
    let png = render_timeline(TIMELINE, TimelineFormat::Png).unwrap();
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    let width = u32::from_be_bytes(png[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(png[20..24].try_into().unwrap());
    // `width="1328" height="694"` in the fixture.
    assert_eq!((width, height), (2656, 1388));
}

#[test]
fn it_renders_to_a_pdf() {
    let pdf = render_timeline(TIMELINE, TimelineFormat::Pdf).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
}

/// Text is drawn with a real font: a missing family would render nothing.
#[test]
fn the_labels_are_drawn_with_the_embedded_fonts() {
    let png = render_timeline(TIMELINE, TimelineFormat::Png).unwrap();
    // A PNG of a drawing full of text and bars is far from the size of a blank page.
    assert!(png.len() > 30_000, "png of {} bytes looks empty", png.len());
}
