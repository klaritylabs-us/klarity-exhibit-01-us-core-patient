//! US Core Patient checks on parsed JSON.
//!
//! These are the kinds of rules you wire into a mapper or outbound validator before
//! you hit a payer FHIR endpoint. Not a full FHIR validator and not Inferno.

use serde_json::Value;

use crate::outcome::{ValidationIssue, ValidationReport};
use crate::profile::{DATA_ABSENT_REASON_URL, US_CORE_PATIENT_PROFILE, US_SSN_SYSTEM};

pub fn validate_patient_json(json: &str) -> Result<ValidationReport, String> {
    let value: Value = serde_json::from_str(json).map_err(|e| format!("JSON parse: {e}"))?;
    Ok(validate_patient_value(&value))
}

pub fn validate_patient_value(patient: &Value) -> ValidationReport {
    let mut report = ValidationReport::ok();

    if patient.get("resourceType").and_then(Value::as_str) != Some("Patient") {
        report.push(ValidationIssue::error(
            "Patient.resourceType",
            "resourceType must be Patient",
            "patient-resource-type",
        ));
        return report;
    }

    check_meta_profile(patient, &mut report);
    check_must_support_elements(patient, &mut report);
    check_identifier_slices(patient, &mut report);
    check_birth_date_or_absent_reason(patient, &mut report);

    report
}

fn check_meta_profile(patient: &Value, report: &mut ValidationReport) {
    let profiles = patient
        .pointer("/meta/profile")
        .and_then(Value::as_array);

    match profiles {
        None => {
            report.push(ValidationIssue::error(
                "Patient.meta.profile",
                "declare meta.profile with the US Core Patient canonical URL",
                "us-core-meta-profile",
            ));
        }
        Some(arr) => {
            let has_us_core = arr
                .iter()
                .filter_map(Value::as_str)
                .any(|p| p == US_CORE_PATIENT_PROFILE);
            if !has_us_core {
                report.push(ValidationIssue::error(
                    "Patient.meta.profile",
                    format!("meta.profile must include {}", US_CORE_PATIENT_PROFILE),
                    "us-core-meta-profile",
                ));
            }
        }
    }
}

// US Core marks several Patient elements Must Support — your code path has to read/write them.
fn check_must_support_elements(patient: &Value, report: &mut ValidationReport) {
    let identifiers = patient.get("identifier").and_then(Value::as_array);
    if identifiers.is_none() || identifiers.is_some_and(|a| a.is_empty()) {
        report.push(ValidationIssue::error(
            "Patient.identifier",
            "need at least one identifier (system + value)",
            "identifier-required",
        ));
    } else if let Some(ids) = identifiers {
        let any_usable = ids.iter().any(|id| {
            id.get("system").and_then(Value::as_str).is_some()
                && id.get("value").and_then(Value::as_str).is_some_and(|v| !v.is_empty())
        });
        if !any_usable {
            report.push(ValidationIssue::error(
                "Patient.identifier",
                "identifier entries should include both system and value",
                "identifier-system-value",
            ));
        }
    }

    let names = patient.get("name").and_then(Value::as_array);
    if names.is_none() || names.is_some_and(|a| a.is_empty()) {
        report.push(ValidationIssue::error(
            "Patient.name",
            "name is required (family and/or given)",
            "name-required",
        ));
    } else if let Some(names) = names {
        let any_name = names.iter().any(|n| {
            n.get("family").and_then(Value::as_str).is_some()
                || n.get("given")
                    .and_then(Value::as_array)
                    .is_some_and(|g| g.iter().any(|v| v.as_str().is_some_and(|s| !s.is_empty())))
        });
        if !any_name {
            report.push(ValidationIssue::error(
                "Patient.name",
                "HumanName needs family and/or given",
                "name-content",
            ));
        }
    }

    if patient.get("gender").and_then(Value::as_str).is_none() {
        report.push(ValidationIssue::error(
            "Patient.gender",
            "gender is required for US Core Patient",
            "gender-required",
        ));
    }

    if patient.get("telecom").and_then(Value::as_array).is_none() {
        report.push(ValidationIssue::warning(
            "Patient.telecom",
            "telecom is Must Support; add phone or email when you have it",
            "telecom-missing",
        ));
    }

    if patient.get("address").and_then(Value::as_array).is_none() {
        report.push(ValidationIssue::warning(
            "Patient.address",
            "address is Must Support for demographics payloads",
            "address-missing",
        ));
    }
}

// If the identifier is typed as SSN, US Core expects the FHIR US SSN system URI (not the raw OID).
fn check_identifier_slices(patient: &Value, report: &mut ValidationReport) {
    let Some(ids) = patient.get("identifier").and_then(Value::as_array) else {
        return;
    };

    for (index, id) in ids.iter().enumerate() {
        let path = format!("Patient.identifier[{}]", index);
        let type_text = id
            .pointer("/type/text")
            .and_then(Value::as_str)
            .map(|s| s.to_ascii_lowercase());
        let type_coding = id.pointer("/type/coding").and_then(Value::as_array);

        let is_ssn_slice = type_text.is_some_and(|t| t.contains("ssn"))
            || type_coding.is_some_and(|codings| {
                codings.iter().any(|c| {
                    c.get("code").and_then(Value::as_str) == Some("SS")
                        || c.get("display")
                            .and_then(Value::as_str)
                            .is_some_and(|d| d.eq_ignore_ascii_case("social security number"))
                })
            });

        if !is_ssn_slice {
            continue;
        }

        let system = id.get("system").and_then(Value::as_str);
        if system != Some(US_SSN_SYSTEM) {
            report.push(ValidationIssue::error(
                format!("{path}.system"),
                format!(
                    "SSN identifier system should be `{}` (found {:?})",
                    US_SSN_SYSTEM,
                    system
                ),
                "ssn-wrong-system",
            ));
        }
    }
}

fn check_birth_date_or_absent_reason(patient: &Value, report: &mut ValidationReport) {
    if patient.get("birthDate").and_then(Value::as_str).is_some() {
        return;
    }

    let has_absent_extension = patient
        .pointer("/_birthDate/extension")
        .and_then(Value::as_array)
        .is_some_and(|exts| {
            exts.iter().any(|ext| {
                ext.get("url").and_then(Value::as_str) == Some(DATA_ABSENT_REASON_URL)
                    && ext.get("valueCode").and_then(Value::as_str).is_some()
            })
        });

    if has_absent_extension {
        return;
    }

    report.push(ValidationIssue::error(
        "Patient.birthDate",
        "set birthDate or _birthDate with data-absent-reason when DOB is unknown",
        "birthdate-or-absent-reason",
    ));
}
