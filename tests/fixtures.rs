use std::fs;
use std::path::PathBuf;

use us_core_patient::validate_patient_json;

fn fixture(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("examples");
    path.push(name);
    fs::read_to_string(path).expect("fixture")
}

#[test]
fn valid_patient_passes() {
    let report = validate_patient_json(&fixture("valid-patient.json")).unwrap();
    assert!(report.is_valid(), "issues: {:?}", report.issues);
}

#[test]
fn missing_profile_fails() {
    let report = validate_patient_json(&fixture("invalid-missing-profile.json")).unwrap();
    assert!(!report.is_valid());
}

#[test]
fn ssn_slice_wrong_system_fails() {
    let report = validate_patient_json(&fixture("invalid-identifier-ssn-slice.json")).unwrap();
    assert!(!report.is_valid());
}

#[test]
fn missing_birthdate_without_extension_fails() {
    let report = validate_patient_json(&fixture("invalid-missing-birthdate.json")).unwrap();
    assert!(!report.is_valid());
}

#[test]
fn birthdate_absent_reason_passes() {
    let report = validate_patient_json(&fixture("valid-birthdate-absent-reason.json")).unwrap();
    assert!(report.is_valid(), "issues: {:?}", report.issues);
}
