//! US Core Patient validation helpers (FHIR R4 JSON).

pub mod outcome;
pub mod profile;
pub mod validate;

pub use outcome::{IssueSeverity, ValidationIssue, ValidationReport};
pub use validate::{validate_patient_json, validate_patient_value};
