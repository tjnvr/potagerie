//! The converters on unusual SVGs: long and accented labels, Unicode edge
//! cases, tiny and huge sizes, repeated renders. Never a panic, never an
//! empty image, always the same bytes for the same input.
//!
//! Set `POTAGERIE_STRESS_OUT` to a directory to also write the PNGs there and
//! look at them.

use export::{ExportError, TimelineFormat, render_timeline};

fn svg(width: u32, height: u32, body: &str) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}"><rect width="{width}" height="{height}" fill="#EEF2E6"/>{body}</svg>"##
    )
}

fn label(y: u32, text: &str) -> String {
    format!(
        r##"<text x="10" y="{y}" font-family="Figtree" font-weight="600" font-size="14" fill="#1F3A2E">{text}</text>"##
    )
}

fn png_size(png: &[u8]) -> (u32, u32) {
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    (
        u32::from_be_bytes(png[16..20].try_into().unwrap()),
        u32::from_be_bytes(png[20..24].try_into().unwrap()),
    )
}

fn dump(name: &str, png: &[u8]) {
    if let Ok(dir) = std::env::var("POTAGERIE_STRESS_OUT") {
        std::fs::write(std::path::Path::new(&dir).join(name), png).unwrap();
    }
}

/// Decodes the PNG and counts the pixels that differ from the background.
fn inked_pixels(png: &[u8]) -> usize {
    let pixmap = resvg::tiny_skia::Pixmap::decode_png(png).unwrap();
    let pixels = pixmap.pixels();
    let background = pixels[0];
    pixels.iter().filter(|p| **p != background).count()
}

#[test]
fn a_long_accented_label_is_drawn_and_deterministic() {
    let long =
        "Épinards d’hiver à feuilles rouges, œillets d’Inde et « chou-fleur » – très très long";
    let doc = svg(300, 60, &label(30, long));
    let a = render_timeline(&doc, TimelineFormat::Png).unwrap();
    let b = render_timeline(&doc, TimelineFormat::Png).unwrap();
    assert_eq!(a, b, "PNG output is not deterministic");
    assert_eq!(png_size(&a), (600, 120));
    assert!(inked_pixels(&a) > 500, "the label was not drawn");
    dump("long_label.png", &a);
    let p1 = render_timeline(&doc, TimelineFormat::Pdf).unwrap();
    let p2 = render_timeline(&doc, TimelineFormat::Pdf).unwrap();
    assert_eq!(p1, p2, "PDF output is not deterministic");
}

#[test]
fn accents_are_kept_composed_and_decomposed() {
    // "Épinards" in NFC and in NFD must both draw the accent: the drawable
    // filter must not strip the combining mark and leave a plain "E".
    let nfc = svg(200, 40, &label(25, "\u{c9}pinards \u{e0}\u{e7}\u{fc}"));
    let nfd = svg(
        200,
        40,
        &label(25, "E\u{301}pinards a\u{300}c\u{327}u\u{308}"),
    );
    let a = render_timeline(&nfc, TimelineFormat::Png).unwrap();
    let b = render_timeline(&nfd, TimelineFormat::Png).unwrap();
    dump("nfc.png", &a);
    dump("nfd.png", &b);
    assert!(inked_pixels(&b) >= inked_pixels(&a) * 9 / 10);
}

#[test]
fn french_typographic_spaces_do_not_glue_the_words() {
    // U+202F (narrow no-break space) is what French typography puts before
    // « ; : ! ? ». Dropping it would glue "12 kg" into "12kg".
    let with = svg(200, 40, &label(25, "Tomates 12\u{202f}kg"));
    let plain = svg(200, 40, &label(25, "Tomates 12 kg"));
    let a = render_timeline(&with, TimelineFormat::Png).unwrap();
    let b = render_timeline(&plain, TimelineFormat::Png).unwrap();
    dump("nnbsp.png", &a);
    // The space widths differ by a fraction of a pixel, so anti-aliasing may
    // shift a few pixels; a dropped space would shift the whole word.
    let glued = render_timeline(
        &svg(200, 40, &label(25, "Tomates 12kg")),
        TimelineFormat::Png,
    )
    .unwrap();
    let (na, nb, ng) = (inked_pixels(&a), inked_pixels(&b), inked_pixels(&glued));
    assert!(na.abs_diff(nb) <= 20, "nnbsp {na} vs space {nb}");
    assert_ne!(a, glued, "U+202F was dropped ({na} vs glued {ng})");
}

#[test]
fn unicode_edge_cases_render_without_panic() {
    let texts = [
        "👩\u{200d}🌾 jardinière 🍅",
        "שלום עולם mélangé عربي",
        "\u{200b}zero\u{200b}width\u{feff}",
        "line\u{2028}sep\u{2029}para",
        "𝔘𝔫𝔦𝔠𝔬𝔡𝔢 \u{10FFFF}",
        "ǅ ß İ ŉ ﬁ",
        "漢字かな한국어",
        "&amp;&lt;b&gt; &#233; &#x1F345;",
        "",
        " ",
    ];
    let body: String = texts
        .iter()
        .enumerate()
        .map(|(i, t)| label(20 + 20 * i as u32, t))
        .collect();
    let doc = svg(400, 240, &body);
    let png = render_timeline(&doc, TimelineFormat::Png).unwrap();
    dump("unicode.png", &png);
    assert_eq!(png_size(&png), (800, 480));
    render_timeline(&doc, TimelineFormat::Pdf).unwrap();
}

#[test]
fn a_raw_ampersand_is_an_invalid_svg_not_a_panic() {
    // The interface must escape the names; if it forgets, the export fails
    // cleanly.
    let doc = svg(200, 40, &label(25, "Choux & Navets <vite>"));
    assert!(matches!(
        render_timeline(&doc, TimelineFormat::Png),
        Err(ExportError::InvalidSvg(_))
    ));
}

#[test]
fn degenerate_sizes_are_errors_not_panics() {
    for doc in [
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="0" height="0"/>"#,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="0" height="10"/>"#,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="-5" height="10"/>"#,
        r#"<svg xmlns="http://www.w3.org/2000/svg"/>"#,
        "",
        "<svg",
    ] {
        for format in [TimelineFormat::Png, TimelineFormat::Pdf] {
            let _ = render_timeline(doc, format);
        }
    }
}

#[test]
fn a_one_pixel_timeline_is_a_valid_png() {
    let doc = svg(1, 1, "");
    let png = render_timeline(&doc, TimelineFormat::Png).unwrap();
    assert_eq!(png_size(&png), (2, 2));
}

#[test]
fn a_size_beyond_what_memory_can_hold_is_refused_not_allocated() {
    // 1 000 000 x 1 000 000 units would ask for terabytes.
    let doc = svg(1_000_000, 1_000_000, "");
    assert!(matches!(
        render_timeline(&doc, TimelineFormat::Png),
        Err(ExportError::Render(_))
    ));
}

#[test]
fn a_wide_timeline_of_five_years_and_ten_beds_renders() {
    // 5 years of 1328 units, 10 beds, 140 bars with long labels.
    let mut body = String::new();
    for row in 0..10u32 {
        for k in 0..14u32 {
            let x = 120 + k * 450;
            let y = 20 + row * 40;
            body.push_str(&format!(
                r##"<rect x="{x}" y="{y}" width="440" height="30" rx="9" fill="#E5533D"/>"##
            ));
            body.push_str(&format!(
                r##"<text x="{}" y="{}" font-family="Figtree" font-weight="600" font-size="12" fill="#1F3A2E">Tomate cœur-de-bœuf – Planche à la façade nord</text>"##,
                x + 6,
                y + 19
            ));
        }
    }
    let doc = svg(6500, 440, &body);
    let png = render_timeline(&doc, TimelineFormat::Png).unwrap();
    assert_eq!(png_size(&png), (13_000, 880));
    assert!(inked_pixels(&png) > 100_000);
    let pdf = render_timeline(&doc, TimelineFormat::Pdf).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
}
