# Architecture

```text
Patient.json
  → serde_json::Value
  → validate_patient_value()
       · meta.profile (US Core Patient URL)
       · Must Support fields (identifier, name, gender; telecom/address warnings)
       · SSN identifier system URI
       · birthDate or _birthDate + data-absent-reason
  → Vec<ValidationIssue>  (printed as JSON from the CLI)
```

| Module | Role |
|--------|------|
| `profile.rs` | IG URLs |
| `outcome.rs` | severity, path, detail, rule_id |
| `validate.rs` | rules |
| `bin/validate_patient.rs` | file input, stdout |

Out of scope: full StructureDefinition validation, terminology services, live FHIR servers, PHI.
