# MPK Package Manifest v0

Status: implementation baseline for package fixture validation.

The package manifest describes the certificate set that belongs to one MPK
package, the package imports that must be hash-pinned before verification, and
the policy knobs required by later package-check commands. It is operational
metadata only. A manifest never proves a declaration by itself; acceptance still
requires source-free checker verdicts and recomputed hashes from the referenced
`.mpcert` bytes.

## Format

The normative interchange format is strict JSON with no duplicate object keys.
Human-authored YAML templates may mirror the same field names, but validation
fixtures use JSON so the field contract is unambiguous.

Top-level fields:

| Field | Required | Meaning |
|---|---:|---|
| `schema` | yes | Must be `mpk.package.v0`. |
| `module` | yes | Canonical dotted MPK module or package name. |
| `imports` | yes | Hash-pinned package imports, sorted by module, export hash, then certificate hash. |
| `certificates` | yes | Package-root relative certificate paths and expected hashes. Must contain at least one entry. |
| `policy` | yes | Verification policy used by later package commands. |

## Names

`module`, `imports[].module`, and `certificates[].module` use the same canonical
name grammar as certificate module names:

- ASCII only.
- Dotted components must be nonempty.
- A component starts with `A-Z`, `a-z`, or `_`.
- Remaining component bytes may also contain digits or `'`.

## Imports

Each import entry has:

| Field | Required | Meaning |
|---|---:|---|
| `module` | yes | Imported module name. |
| `export_hash` | yes | Lowercase SHA-256 hex public interface identity. |
| `certificate_hash` | no | Lowercase SHA-256 hex certificate identity for high-trust import resolution. |

Import entries must be sorted by `(module, export_hash, certificate_hash)` and
must not contain duplicate `(module, export_hash)` pairs. Hashes must be
lowercase 64-character hex strings and must not be all zeroes.

Normal package resolution may use `(module, export_hash)`. High-trust package
resolution also verifies `certificate_hash` in the current session.

## Certificates

Each certificate entry has:

| Field | Required | Meaning |
|---|---:|---|
| `module` | yes | Module name that the referenced certificate must report after checking. |
| `path` | yes | Package-root relative path to a `.mpcert` file or checked `.hex` fixture. |
| `expected_export_hash` | yes | Recomputed export hash expected from checker output. |
| `expected_axiom_report_hash` | yes | Recomputed axiom-report hash expected from checker output. |
| `expected_certificate_hash` | yes | Recomputed certificate hash expected from checker output. |

Certificate paths must use `/` separators, must not be absolute, and must not
contain empty, `.`, or `..` path components. A package checker resolves paths
against the package root and rejects paths that escape it after filesystem
normalization.

Validation must run the fast source-free checker on every listed certificate,
require an accepted verdict, require the reported module to equal
`certificates[].module`, and compare all three expected hashes against checker
output.

## Policy

`policy` has:

| Field | Required | Meaning |
|---|---:|---|
| `checker_profile` | yes | One of `core-bootstrap`, `mvp-structural`, or `mvp-strict`. |
| `allowed_axiom_profiles` | yes | Nonempty list of release-policy axiom profiles. |
| `require_reference_checker` | yes | Whether package verification must require Go reference-checker agreement. |
| `require_source_free_check` | yes | Must be `true` for release-ready packages. |

CI-004 only defines and validates manifest fixtures. Later milestones implement
lock-file pinning and user-facing package commands.

## Fixture contract

`fixtures/package-manifest/valid/*.json` must validate successfully, including
checker-backed certificate hash comparison. `fixtures/package-manifest/invalid`
contains malformed manifests that must be rejected by the fixture validator.
