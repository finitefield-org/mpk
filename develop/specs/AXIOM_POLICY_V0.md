# MPK Axiom Policy v0

Status: approved governance baseline for the MPK MVP.

## Scope

This specification defines the fixed axiom categories and release-policy behavior for MPK v0. It applies to every certificate, package manifest, axiom report, checker verdict, and release evidence report that mentions axioms.

This policy depends on `specs/TRUST_BOUNDARY_V0.md`: an axiom is trusted only after it appears in a checked certificate and the recomputed axiom report is permitted by the active release policy.

## Core Rule

Every axiom used by an accepted declaration must be visible in the recomputed axiom report. Hidden assumptions, admitted proof bodies, unchecked imports, solver yes/no answers, and unreported primitive facts are release blockers.

The checker must reject a package when:

- an axiom report entry has an unknown category;
- an axiom report omits a direct or transitive axiom dependency;
- the active policy profile does not allow an observed axiom;
- an observed axiom hash, type hash, or origin module does not match the approved entry;
- a package tries to treat a theorem index, registry entry, source comment, tactic replay, or AI trace as an axiom approval.

## Fixed Categories

MPK v0 has exactly four axiom categories:

| Category | Meaning | MVP release posture |
|---|---|---|
| `CoreAxiom` | Required logical primitive that the core cannot reduce to a checked definition in v0. | Allowed only by explicit name and hash in a release profile. Target count is zero or near-zero. |
| `BuiltinTheoryAxiom` | Primitive theory-interface assumption used to connect checked theory certificates to core terms. | Allowed only by explicit name, hash, theory, and checker profile. Must be justified by a checked theory-certificate path. |
| `GoSemanticsAxiom` | Temporary semantic assumption about the modeled Go subset. This is the v0 semantic-axiom category. | Blocked for release-ready packages unless a governance-approved alpha or experimental profile explicitly allows it. |
| `ExternalAxiom` | User, library, imported, or tool-provided assumption without a certificate proof accepted by MPK. | Rejected by default for all release-ready packages. Any occurrence is a release blocker unless an explicitly experimental profile allows it. |

No other category name is valid in v0. Renaming, splitting, or adding categories requires a new policy revision and matching certificate/report migration.

## Category Examples

Examples are illustrative; they do not approve a concrete axiom name:

- `CoreAxiom`: a minimal primitive equality eliminator, if equality is not fully represented by checked inductive declarations in the active core profile.
- `BuiltinTheoryAxiom`: a bridge principle that lets a checked bitvector-normalization certificate inhabit an expected MPK proposition.
- `GoSemanticsAxiom`: a temporary assumption that an encoded `Go.I64.signed_lt` relation matches the restricted Go frontend's intended signed comparison semantics.
- `ExternalAxiom`: a user-declared theorem imported without a checked `.mpcert` proof body or accepted theory certificate.

## Axiom Identity

An approved axiom is identified by all of:

- category;
- fully qualified MPK name;
- origin module;
- type hash;
- declaration hash;
- export hash of the defining module;
- certificate hash when high-trust mode is required.

Changing any identity field creates a new axiom for policy purposes. The new axiom must be reviewed and approved before release.

## Axiom Report Entries

Every recomputed axiom report must include one entry for each observed axiom. Each axiom entry must include at least:

```text
AxiomReportEntry:
  category
  name
  origin_module
  type_hash
  declaration_hash
  source_certificate_hash?
  direct_dependent_declarations
  transitive_dependent_declarations
  approval_profile?
  reviewer_note?
```

For each checked declaration that depends on axioms, the report must include direct and transitive dependencies by axiom identity:

```text
DeclarationAxiomDependencies:
  declaration_name
  declaration_hash
  direct_axiom_dependencies
  transitive_axiom_dependencies
```

The report must also include deterministic category summaries:

```text
AxiomReportSummary:
  core_axiom_count
  builtin_theory_axiom_count
  go_semantics_axiom_count
  external_axiom_count
  total_axiom_count
```

Axiom identity, dependency, and summary fields must be derived from checked declarations and proof nodes, not from source files, manifests, comments, or registry metadata. Policy annotations such as `approval_profile` and `reviewer_note` may be attached after recomputation, but they must not change the recomputed dependency set.

## Policy Profiles

Policy profiles are named allowlists used by package manifests and release gates. A profile can approve only concrete axiom identities, not whole categories by default.

MPK v0 reserves these profile names:

| Profile | Allowed categories | Intended use |
|---|---|---|
| `zero-axiom` | none | Packages that must contain no axioms. |
| `core-mvp` | explicitly approved `CoreAxiom` entries only | Core bootstrap and packages that need the minimal logical base. |
| `mvp-theory` | explicitly approved `CoreAxiom` and `BuiltinTheoryAxiom` entries only | Packages using checked theory-certificate interfaces. |
| `go-artifact-alpha` | explicitly approved `CoreAxiom`, `BuiltinTheoryAxiom`, and `GoSemanticsAxiom` entries only | Alpha Go-verification artifacts while semantic axioms are being replaced. |
| `experimental-external` | explicitly approved entries from any v0 category | Non-release experiments only. Must not be used for release-ready packages. |

`ExternalAxiom` is never allowed by `zero-axiom`, `core-mvp`, `mvp-theory`, or `go-artifact-alpha`.

## Release Gate

Before release, the release evidence must show:

1. the axiom report was recomputed by the checker;
2. every entry has one of the four v0 categories;
3. every entry matches an approved identity in the active profile;
4. category counts match the report entries;
5. no `ExternalAxiom` appears in a release-ready package;
6. any `GoSemanticsAxiom` is explicitly marked alpha or experimental and has a tracked replacement plan;
7. any newly observed axiom blocks release until approved.

## Checker Requirements

Checker-facing components must:

- preserve axiom category metadata through certificate decoding, declaration checking, export generation, and report generation;
- compute direct and transitive axiom dependencies;
- include axiom category in the report hash payload;
- reject unknown categories;
- reject unapproved axioms under the active policy profile;
- emit deterministic structured errors for unknown, missing, mismatched, or unapproved axiom entries.

## Package Manifest Requirements

Package manifests that enable release checking must declare an `allowed_axiom_profiles` list. If the list is absent, the package must be treated as `zero-axiom`.

Example:

```yaml
policy:
  allowed_axiom_profiles:
    - core-mvp
```

The manifest profile is only an allowlist selector. It is not proof evidence and cannot override the recomputed axiom report.

## Review Requirements

Approving a new axiom requires a review record that states:

- the category;
- the fully qualified name;
- why it cannot yet be replaced by a checked definition or certificate;
- the expected release profile;
- the owner responsible for removing or preserving it;
- the test fixture that proves the checker reports and gates it deterministically.

For `GoSemanticsAxiom`, the review must also name the future checked semantics or certified lemma that should replace it. For `ExternalAxiom`, the review must state why the package is not release-ready.
