//! JSON export and import of the potager (F-CPT-3, S05 §6).

use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use plan::{Incoherence, Plan};
use serde::{Deserialize, Serialize};

use crate::ExportError;

/// Value of the `format` field of an export.
pub const FORMAT: &str = "potagerie";
/// Version of the export format this build writes and reads.
pub const FORMAT_VERSION: u32 = 1;

/// The file: an envelope that names the format around the plan document.
#[derive(Serialize)]
struct EnvelopeOut<'a> {
    format: &'static str,
    format_version: u32,
    plan: &'a Plan,
}

#[derive(Deserialize)]
struct EnvelopeIn {
    format: String,
    format_version: u32,
    plan: serde_json::Value,
}

/// What an import file holds, compared with the plan of the device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSummary {
    pub updated_at: DateTime<Utc>,
    pub version: u64,
    pub planche_count: usize,
    pub serie_count: usize,
    /// True when the file was last modified before the plan of the device.
    pub older: bool,
    /// True when the device already has a plan, which the import would replace.
    pub replaces: bool,
    /// What breaks the rules of the plan (S03 §13). The file is accepted all
    /// the same: nothing is deleted or corrected, the user resolves them.
    pub incoherences: Vec<Incoherence>,
}

/// Writes the whole plan to `path`, in pretty JSON.
pub fn export_garden(plan: &Plan, path: &Path) -> Result<(), ExportError> {
    let envelope = EnvelopeOut {
        format: FORMAT,
        format_version: FORMAT_VERSION,
        plan,
    };
    let json = serde_json::to_string_pretty(&envelope)
        .map_err(|e| ExportError::InvalidImport(e.to_string()))?;
    fs::write(path, json)?;
    Ok(())
}

/// Reads the plan of an export file: schema, format and version, then the
/// structure (S03 §13). Incoherences do not refuse it.
pub fn read_garden(path: &Path) -> Result<Plan, ExportError> {
    let text = fs::read_to_string(path)?;
    let envelope: EnvelopeIn =
        serde_json::from_str(&text).map_err(|e| ExportError::InvalidImport(e.to_string()))?;
    if envelope.format != FORMAT {
        return Err(ExportError::InvalidImport(format!(
            "format {:?}",
            envelope.format
        )));
    }
    if envelope.format_version > FORMAT_VERSION {
        return Err(ExportError::ImportTooNew {
            found: envelope.format_version,
            supported: FORMAT_VERSION,
        });
    }
    let plan: Plan = serde_json::from_value(envelope.plan)
        .map_err(|e| ExportError::InvalidImport(e.to_string()))?;
    // A plan the calculations cannot run on is refused; the other
    // incoherences are listed by `read_import` (S03 §13).
    plan.check_structure()
        .map_err(|e| ExportError::InvalidImport(e.to_string()))?;
    Ok(plan)
}

/// Reads an export file and compares it with the plan of the device, without
/// replacing anything (`lire_import`, S05 §7).
///
/// "Older" compares the `mis_a_jour_le` timestamps: versions are counted per
/// device and cannot be compared across devices.
pub fn read_import(path: &Path, local: Option<&Plan>) -> Result<ImportSummary, ExportError> {
    Ok(summarise(&read_garden(path)?, local))
}

/// The summary of a plan received, compared with the plan of the device.
pub fn summarise(plan: &Plan, local: Option<&Plan>) -> ImportSummary {
    ImportSummary {
        updated_at: plan.mis_a_jour_le,
        version: plan.version,
        planche_count: plan.planches.len(),
        serie_count: plan.series.len(),
        older: local.is_some_and(|l| plan.mis_a_jour_le < l.mis_a_jour_le),
        replaces: local.is_some(),
        incoherences: plan.incoherences(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use plan::{CaseCm, PlanId, Terrain};

    use super::*;

    fn plan_at(hour: u32) -> Plan {
        Plan::new(
            PlanId::generate(),
            Terrain {
                longueur_cm: 1100,
                largeur_cm: 650,
                case_cm: CaseCm::C50,
                allee_cm: 50,
            },
            Utc.with_ymd_and_hms(2027, 3, 1, hour, 0, 0).unwrap(),
        )
    }

    #[test]
    fn export_then_read_gives_the_same_plan() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("potager.json");
        let plan = plan_at(9);
        export_garden(&plan, &path).unwrap();
        assert_eq!(read_garden(&path).unwrap(), plan);
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"format\": \"potagerie\""));
    }

    #[test]
    fn summary_warns_when_the_file_is_older() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("potager.json");
        export_garden(&plan_at(9), &path).unwrap();

        let newer_local = plan_at(10);
        let summary = read_import(&path, Some(&newer_local)).unwrap();
        assert!(summary.older);
        assert!(summary.replaces);

        let older_local = plan_at(8);
        assert!(!read_import(&path, Some(&older_local)).unwrap().older);

        let summary = read_import(&path, None).unwrap();
        assert!(!summary.older);
        assert!(!summary.replaces);
    }

    #[test]
    fn other_files_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("other.json");
        fs::write(&path, r#"{"hello": "world"}"#).unwrap();
        assert!(matches!(
            read_garden(&path),
            Err(ExportError::InvalidImport(_))
        ));

        fs::write(
            &path,
            r#"{"format": "potagerie", "format_version": 99, "plan": {}}"#,
        )
        .unwrap();
        assert!(matches!(
            read_garden(&path),
            Err(ExportError::ImportTooNew { found: 99, .. })
        ));
    }

    fn write_plan_json(path: &Path, edit: impl FnOnce(&mut serde_json::Value)) {
        let mut doc = serde_json::json!({
            "format": "potagerie",
            "format_version": 1,
            "plan": plan_at(9),
        });
        edit(&mut doc["plan"]);
        fs::write(path, doc.to_string()).unwrap();
    }

    #[test]
    fn invalid_files_are_refused_and_nothing_is_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json");
        let refused = |path: &Path| matches!(read_garden(path), Err(ExportError::InvalidImport(_)));

        fs::write(&path, "not json at all").unwrap();
        assert!(refused(&path));
        fs::write(&path, "").unwrap();
        assert!(refused(&path));
        // Right envelope, wrong plan schema.
        fs::write(
            &path,
            r#"{"format":"potagerie","format_version":1,"plan":{"id":3}}"#,
        )
        .unwrap();
        assert!(refused(&path));
        // Unknown case size, invalid allée, broken layout, broken serie.
        write_plan_json(&path, |p| p["terrain"]["case_cm"] = 40.into());
        assert!(refused(&path));
        write_plan_json(&path, |p| p["terrain"]["allee_cm"] = 7.into());
        assert!(refused(&path));
        write_plan_json(&path, |p| {
            p["planches"] = serde_json::json!({"0190c5a0-0000-7000-8000-000000000001": {
                "nom": "A",
                "dispositions": {"2027": {"etat":"presente","type":"planche","x":0,"y":0,"longueur_cm":30,"largeur_cm":100}}
            }});
        });
        assert!(refused(&path));
        // Missing file: an I/O error, not an invalid import.
        assert!(matches!(
            read_garden(&dir.path().join("none.json")),
            Err(ExportError::Io(_))
        ));
    }

    #[test]
    fn incoherent_files_are_accepted_with_their_list() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("incoherent.json");
        write_plan_json(&path, |p| {
            p["series"] = serde_json::json!({"0190c5a0-0000-7000-8000-000000000002": {
                "culture_id": "tomate",
                "planche_id": "0190c5a0-0000-7000-8000-000000000003",
                "troncon_debut": 1, "troncon_fin": 2, "mode": "plants",
                "date": "2027-05-20", "abri_jours": 0, "apres_jours": 60, "duree_jours": 40
            }});
        });
        let summary = read_import(&path, None).unwrap();
        assert_eq!(summary.serie_count, 1);
        assert_eq!(summary.incoherences.len(), 1);
        assert_eq!(summary.incoherences[0].code(), "serie_orpheline");
    }
}
