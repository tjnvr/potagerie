//! Timeline export to PNG (resvg) or PDF (svg2pdf), with embedded fonts (S05 §4).

use std::fs;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use resvg::{tiny_skia, usvg};
use serde::Deserialize;

use crate::ExportError;

/// Fonts of the interface (D01), embedded so the export matches the screen
/// on every machine and never reads system fonts. SIL Open Font License.
const FONTS: &[&[u8]] = &[
    include_bytes!("../fonts/Figtree-Regular.ttf"),
    include_bytes!("../fonts/Figtree-SemiBold.ttf"),
    include_bytes!("../fonts/Figtree-Bold.ttf"),
    include_bytes!("../fonts/Fredoka-SemiBold.ttf"),
];

/// PNG pixels per SVG user unit: 2 gives a sharp image on high-density screens.
const PNG_SCALE: f32 = 2.0;

/// Output format of the timeline export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimelineFormat {
    Png,
    Pdf,
}

fn fontdb() -> Arc<usvg::fontdb::Database> {
    static DB: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        for font in FONTS {
            db.load_font_data(font.to_vec());
        }
        db.set_sans_serif_family("Figtree");
        Arc::new(db)
    })
    .clone()
}

fn parse(svg: &str) -> Result<usvg::Tree, ExportError> {
    let options = usvg::Options {
        font_family: "Figtree".into(),
        fontdb: fontdb(),
        ..usvg::Options::default()
    };
    usvg::Tree::from_str(svg, &options).map_err(|e| ExportError::InvalidSvg(e.to_string()))
}

/// True for the characters the embedded fonts (Figtree, Fredoka) can draw: Latin
/// letters with their accents, the usual punctuation and a few symbols.
fn is_drawable(c: char) -> bool {
    matches!(c,
        '\t' | '\n' | '\r' | ' '..='~'
        | '\u{A0}'..='\u{17F}'
        | '\u{2010}'..='\u{2027}'
        | '\u{2030}'..='\u{203A}'
        | '\u{20AC}' | '\u{2192}')
}

/// Removes what the fonts cannot draw (emoji, symbols such as `≈` or `✓`):
/// they would come out as empty boxes in the file. The markup of the SVG is
/// ASCII, so only the texts of the garden are touched.
pub(crate) fn drawable_only(svg: &str) -> String {
    svg.chars().filter(|c| is_drawable(*c)).collect()
}

/// Converts the SVG of the timeline to the bytes of a PNG or PDF file.
pub fn render_timeline(svg: &str, format: TimelineFormat) -> Result<Vec<u8>, ExportError> {
    let tree = parse(&drawable_only(svg))?;
    match format {
        TimelineFormat::Png => {
            let size = tree
                .size()
                .to_int_size()
                .scale_by(PNG_SCALE)
                .ok_or_else(|| ExportError::Render("the SVG is too large or has no size".into()))?;
            let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())
                .ok_or_else(|| ExportError::Render("the SVG has no size".into()))?;
            resvg::render(
                &tree,
                tiny_skia::Transform::from_scale(PNG_SCALE, PNG_SCALE),
                &mut pixmap.as_mut(),
            );
            pixmap
                .encode_png()
                .map_err(|e| ExportError::Render(e.to_string()))
        }
        TimelineFormat::Pdf => svg2pdf::to_pdf(
            &tree,
            svg2pdf::ConversionOptions::default(),
            svg2pdf::PageOptions::default(),
        )
        .map_err(|e| ExportError::Render(e.to_string())),
    }
}

/// Converts the SVG of the timeline and writes it to `path`.
pub fn export_timeline(svg: &str, format: TimelineFormat, path: &Path) -> Result<(), ExportError> {
    let bytes = render_timeline(svg, format)?;
    fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="60" viewBox="0 0 200 60">
        <rect x="0" y="0" width="200" height="60" fill="#EEF2E6"/>
        <rect x="10" y="20" width="120" height="26" rx="9" fill="#E5533D"/>
        <text x="16" y="38" font-family="Figtree" font-weight="600" font-size="12" fill="#1F3A2E">Tomato</text>
        <text x="140" y="38" font-family="Fredoka" font-weight="600" font-size="14" fill="#1F3A2E">2027</text>
    </svg>"##;

    #[test]
    fn embedded_fonts_are_loaded() {
        let db = fontdb();
        let families: Vec<String> = db
            .faces()
            .flat_map(|f| f.families.iter().map(|(name, _)| name.clone()))
            .collect();
        assert!(families.iter().any(|f| f == "Figtree"), "{families:?}");
        assert!(families.iter().any(|f| f == "Fredoka"), "{families:?}");
    }

    #[test]
    fn png_is_rendered_at_twice_the_size() {
        let png = render_timeline(SVG, TimelineFormat::Png).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        // IHDR: width and height, big-endian, at bytes 16..24.
        let width = u32::from_be_bytes(png[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(png[20..24].try_into().unwrap());
        assert_eq!((width, height), (400, 120));
    }

    #[test]
    fn text_is_drawn_with_the_embedded_font() {
        let tree = parse(SVG).unwrap();
        let mut texts = 0;
        for node in tree.root().children() {
            if let usvg::Node::Text(text) = node {
                texts += 1;
                // Text converted to glyph outlines: a font was found.
                assert!(!text.flattened().children().is_empty());
            }
        }
        assert_eq!(texts, 2);
    }

    #[test]
    fn pdf_is_written() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("timeline.pdf");
        export_timeline(SVG, TimelineFormat::Pdf, &path).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn invalid_svg_is_refused() {
        assert!(matches!(
            render_timeline("<svg", TimelineFormat::Png),
            Err(ExportError::InvalidSvg(_))
        ));
    }
}

#[cfg(test)]
mod drawable_tests {
    use super::drawable_only;

    #[test]
    fn keeps_accents_and_punctuation_and_drops_what_the_fonts_lack() {
        let svg = "<text>Côté cabane · Bout de l’allée – Œillet « 1,2 m » ½ → €</text>";
        assert_eq!(drawable_only(svg), svg);
        let with_emoji = "<text>Tomates 🍅 ≈ 120 ✓ ⅓ ✿</text>";
        assert_eq!(drawable_only(with_emoji), "<text>Tomates   120   </text>");
    }
}
