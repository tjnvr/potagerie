//! Potagerie exports (S05 §4, §7).
//!
//! - [`export_garden`] and [`read_import`]: the whole garden in a JSON file,
//!   as a backup and to move it from one device to another (F-CPT-3).
//! - [`export_timeline`]: the SVG of the timeline, serialised by the
//!   interface, converted to PNG (resvg) or PDF (svg2pdf) with the fonts
//!   embedded in the app.

mod garden;
mod timeline;

pub use garden::{
    FORMAT, FORMAT_VERSION, ImportSummary, export_garden, read_garden, read_import, summarise,
};
pub use timeline::{TimelineFormat, export_timeline, render_timeline};

use thiserror::Error;

/// An export or import failure.
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a Potagerie export: {0}")]
    InvalidImport(String),
    #[error("export format {found} is newer than this app ({supported})")]
    ImportTooNew { found: u32, supported: u32 },
    #[error("invalid SVG: {0}")]
    InvalidSvg(String),
    #[error("rendering failed: {0}")]
    Render(String),
}
