# US Core Patient (FHIR R4) — Rust

Small library and CLI that sanity-check **US Core Patient** JSON before it leaves your integration layer: `meta.profile`, common Must Support fields, the US SSN identifier system, and unknown `birthDate` handling.

Synthetic fixtures only. Not ONC certification, not a substitute for the HL7 FHIR Validator or Inferno, and not legal/clinical advice.

Maintained by [Klarity Labs](https://klaritylabs.ai/) as open reference code for U.S. healthcare FHIR interoperability work.

**Exhibit 01** — [FHIR exhibit catalog](https://github.com/klaritylabs-us/klarity-fhir-exhibits) · [klaritylabs-us](https://github.com/klaritylabs-us)


## How this relates to other tools

| Tool class | Role |
|------------|------|
| **HL7 FHIR Validator**, **HAPI**, **Firely** | Full validation, servers, SDKs, IG packages |
| **Inferno**, **Touchstone**, vendor cert suites | **Conformance** against test plans |
| **This exhibit** | **Pre-flight construction** on synthetic fixtures—catch shape mistakes before wire traffic |

Conformance tools test whether your **API behaves**; exhibits focus on whether your **payloads and pipes are built** to survive real partners.

## Specifications cited

- [US Core Patient STU 6.1](http://hl7.org/fhir/us/core/STU6.1/StructureDefinition-us-core-patient.html)
- [FHIR R4 Patient](http://hl7.org/fhir/R4/patient.html)

## What it checks

- `meta.profile` includes [US Core Patient STU 6.1](http://hl7.org/fhir/us/core/STU6.1/StructureDefinition-us-core-patient.html)
- `identifier`, `name`, `gender` populated; warnings if `telecom` / `address` missing
- SSN-typed identifiers use `http://hl7.org/fhir/sid/us-ssn` (not only the legacy OID)
- Missing DOB uses [`data-absent-reason`](http://hl7.org/fhir/R4/extension-dataabsentreason.html) on `_birthDate`

| Element | Why it matters for USCDI-style demographics |
|---------|-----------------------------------------------|
| `identifier` | Matching across systems |
| `name`, `birthDate`, `gender` | Core patient attributes |
| `address`, `telecom` | Contact and location |

## Run it

Needs [Rust](https://www.rust-lang.org/tools/install).

```bash
cargo test
cargo run --bin validate-patient -- examples/valid-patient.json
cargo run --bin validate-patient -- examples/invalid-identifier-ssn-slice.json
```

Exit `0` if no errors, `1` if validation failed, `2` for bad usage or JSON. Cargo may print `process didn't exit successfully` on `1` — that is normal.

## Sample output

One command:

```bash
cargo run --bin validate-patient -- examples/valid-patient.json
```

**Pass** (`examples/valid-patient.json`) — exit code `0`:

```json
[]
```

**Fail** (`examples/invalid-identifier-ssn-slice.json`) — exit code `1` (issues array; more entries may appear):

```json
[
  {
    "severity": "error",
    "path": "Patient.identifier[0].system",
    "detail": "SSN identifier system should be `http://hl7.org/fhir/sid/us-ssn` (found Some(\"urn:oid:2.16.840.1.113883.4.1\"))",
    "rule_id": "ssn-wrong-system"
  }
]
```


## Layout

- `src/validate.rs` — rules
- `examples/` — valid and broken payloads for tests and demos
- [docs/architecture.md](./docs/architecture.md) — short pipeline note

## Inferno vs this repo

Inferno exercises your **API** against CMS-style scenarios. This repo is about **payload shape** in middleware — profiles, slices, and Must Support fields that still break in production after a green conformance run.

## License

MIT — see [LICENSE](./LICENSE).

**Need this on your payloads or API?** 20-minute FHIR Scan — sk@klaritylabs.ai · [klaritylabs.ai](https://klaritylabs.ai/)

## Contact

sk@klaritylabs.ai · [klaritylabs.ai](https://klaritylabs.ai/)
