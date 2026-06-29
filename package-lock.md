# MPK Package Lock v0

Status: implementation baseline for lock fixture validation.

The package lock records the resolved, hash-pinned dependency and checker-policy
state for one package manifest. It is a reproducibility artifact: a package
checker can use it to detect manifest drift, import hash drift, or policy drift
before running full package verification. It is not trusted proof evidence by
itself; certificate acceptance still depends on source-free checker verdicts and
recomputed certificate hashes.

## Format

The normative interchange format is strict JSON with no duplicate object keys.

Top-level fields:

| Field | Required | Meaning |
|---|---:|---|
| `schema` | yes | Must be `mpk.package.lock.v0`. |
| `manifest` | yes | The package manifest path and byte hash this lock was generated from. |
| `module` | yes | Canonical package module name copied from the manifest. |
| `locked_imports` | yes | Resolved imports pinned by module, export hash, and certificate hash. |
| `checker_policy` | yes | Checker and axiom policy copied from the manifest. |

## Manifest reference

`manifest` has:

| Field | Required | Meaning |
|---|---:|---|
| `path` | yes | Package-root relative path to the JSON package manifest. |
| `sha256` | yes | Lowercase SHA-256 hex digest of the manifest file bytes. |

The path must use `/` separators, must not be absolute, and must not contain
empty, `.`, or `..` path components. Verification rejects a lock when the
manifest bytes no longer hash to `manifest.sha256`.

## Locked imports

Each `locked_imports` entry has:

| Field | Required | Meaning |
|---|---:|---|
| `module` | yes | Imported module name. |
| `export_hash` | yes | Lowercase SHA-256 hex public interface identity. |
| `certificate_hash` | yes | Lowercase SHA-256 hex certificate identity. |

Entries must be sorted by `(module, export_hash, certificate_hash)` and must not
contain duplicate `(module, export_hash)` pairs. All hashes must be lowercase
64-character hex strings and must not be all zeroes.

Lock verification compares `locked_imports` exactly against the package
manifest imports after requiring every manifest import to include
`certificate_hash`. This makes v0 locks high-trust by default: an import is not
fully locked unless both its public interface and its certificate identity are
pinned.

## Checker policy

`checker_policy` must be equivalent at the JSON data model level to the manifest
`policy` object:

| Field | Required | Meaning |
|---|---:|---|
| `checker_profile` | yes | One of the manifest-supported checker profiles. |
| `allowed_axiom_profiles` | yes | Nonempty list of release-policy axiom profiles. |
| `require_reference_checker` | yes | Whether package verification requires Go reference-checker agreement. |
| `require_source_free_check` | yes | Must remain `true` for release-ready packages. |

Lock verification rejects policy drift instead of merging or overriding policy.
Changing checker policy requires regenerating and reviewing the lock.

## Fixture contract

`fixtures/package-lock/valid/*.json` must validate successfully. Invalid lock
fixtures must be rejected for deterministic reasons, including import hash drift
and checker-policy drift.

CI-005 only defines and validates lock files. Later milestones implement
user-facing package commands and release evidence generation.
