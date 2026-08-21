# MPK Certificate v0 Specification

Status: frozen for implementation. This is the stable MPK Certificate v0 schema for canonical `.mpcert` files. Changes require a new certificate format version or a governance-approved amendment.

## Goals

- Source-free checking.
- Deterministic binary encoding.
- Stable module export identity.
- Explicit axiom reporting.
- Hash-pinned imports.
- Efficient proof-node DAG checking.
- No comments, source maps, tactic traces, or AI traces inside trusted payload.

## Logical layout

```text
Certificate:
  header:
    magic = "MPKCERT"
    format = "MPK-CERT-0.1"
    core_spec = "MPK-Core-0.1"
    module
  imports:
    Import[]
  name_table:
    Name[]
  level_table:
    LevelNode[]
  term_table:
    TermNode[]
  proof_node_table:
    ProofNode[]
  declarations:
    Decl[]
  theory_certificates:
    TheoryCertificate[]
  export_block:
    ExportEntry[]
  axiom_report:
    AxiomReport
  source_manifest:
    SourceManifest?      # hash-pinned, untrusted metadata
  hashes:
    export_hash
    axiom_report_hash
    certificate_hash
```

## Import entry

```text
Import:
  module_name
  export_hash
  optional certificate_hash
```

Normal mode may resolve imports by module and export hash. High-trust mode requires certificate hash verification in the current session.

## Canonical binary encoding rules

- Fixed field order.
- Explicit vector lengths.
- Minimal unsigned LEB128 varint encodings for variable-width unsigned integer fields.
- Fixed-width bitvector literals encode exactly their declared width in big-endian byte order; signed interpretation is a type-level view, not a different payload encoding.
- UTF-8 names with canonical validation.
- Sorted import table.
- Sorted reachable name table.
- Topologically ordered level DAG.
- Topologically ordered term DAG.
- Topologically ordered proof-node DAG.
- Dependency-ordered declarations.
- Sorted axiom vectors.
- No unreachable table entries.
- No duplicate entries after canonicalization.
- No whitespace, comments, source maps, notation, tactic scripts, unresolved metavariables, or AI traces.

## Hash domains

MVP uses SHA-256 for every certificate-side hash. Hash input is:

```text
domain_tag || 0x00 || canonical_payload
```

Manifests and JSON diagnostics render hashes as lowercase hex. Changing the hash function or domain-tag format requires a new certificate format version.

Use explicit domain separation:

```text
MPK-MODULE-EXPORT-0.1
MPK-MODULE-CERT-0.1
MPK-AXIOM-REPORT-0.1
MPK-LEVEL-0.1
MPK-TERM-0.1
MPK-PROOF-NODE-0.1
MPK-DECL-0.1
MPK-THEORY-CERT-0.1
MPK-SOURCE-MANIFEST-0.1
```

## Export block

The export block includes:

- axiom interfaces;
- definition interfaces and reducible bodies;
- theorem interfaces without proof bodies;
- inductive family interfaces;
- generated constructor interfaces;
- generated recursor interfaces;
- theory primitive interfaces.

Changing an opaque theorem proof body changes the certificate hash. It should not change the export hash unless axiom dependencies or public interfaces change.

## Proof-node table

Certificate v0 reserves tags for these proof nodes:

```text
ProofNode ::=
    Exact(term_id, expected_type_id)
  | Apply(fn_proof_id, arg_proof_ids, expected_type_id)
  | Intro(domain_type_id, body_proof_id, expected_type_id)
  | LetProof(value_term_id, body_proof_id, expected_type_id)
  | Refl(term_id, expected_type_id)
  | Rewrite(eq_proof_id, target_proof_id, expected_type_id)
  | EqRec(motive_id, eq_proof_id, base_proof_id, expected_type_id)
  | Constructor(constructor_id, arg_proof_ids, expected_type_id)
  | Recursor(recursor_id, motive_id, minor_proof_ids, major_proof_id, expected_type_id)
  | Conv(proof_id, expected_type_id, defeq_witness_id?)
  | Theory(theory_certificate_id, expected_type_id)
```

The checker must not trust `expected_type_id`; it recomputes or validates it.

Implementation support is profile-gated:

| Profile | Required nodes | Release behavior for other node tags |
|---|---|---|
| `core-bootstrap` | `Exact`, `Apply`, `Intro`, `Refl`, `Conv` | reject as unsupported |
| `mvp-structural` | `core-bootstrap` plus `LetProof`, `Rewrite`, `EqRec`, `Constructor`, `Recursor` | reject `Theory` until enabled |
| `mvp-strict` | `mvp-structural` plus `Theory` | reject unknown future tags |

The active checker profile is part of package/release policy. A certificate may decode under the binary schema but still reject if it uses a node not enabled by the active profile.

## Source manifest

The optional source-manifest field remains the same length-prefixed opaque byte
payload in Certificate v0. This governance amendment changes only the example
and terminology: it does not change a certificate tag, field order, byte
encoding, hash preimage, or checker acceptance input.

When the payload follows `SOURCE_MANIFEST_V0.md`, certificate assembly embeds
the canonical certificate-stage `mpk.source_manifest.v0` value. Its
language-neutral shape records:

```text
SourceManifestPayload:
  schema = "mpk.source_manifest.v0"
  source_language
  semantic_profile
  semantic_parameters
  selection
  limit_profile
  release_registry
  toolchain
  frontend
  units
  target
  inputs
  input_set_hash
  vir_hash
  source_map_hash
  vc_hash                 # required only at certificate stage
  source_manifest_hash
```

The payload is audit traceability only. The certificate checker treats the
payload bytes and every internal schema, release identity, input digest, and
VIR/map/VC/hash claim as untrusted metadata; none proves source correctness or
changes proof acceptance. Certificate identity still commits to the encoded
payload bytes through the unchanged Certificate v0 encoding.

Input paths use the portable source-root-relative UTF-8 grammar from
`SOURCE_MANIFEST_V0.md` with `/` separators. Absolute local paths, links,
escapes, and platform-specific separators must not enter the canonical payload
because they would make hashes machine-dependent.

## Rejection conditions

Reject if:

- the bytes are not canonical;
- any hash fails to recompute;
- an import cannot be resolved by policy;
- any table has unreachable or out-of-order entries;
- a proof node references a future node;
- a theorem proof does not check;
- a reducible definition body fails to check;
- a theory certificate fails to check;
- an unsupported feature flag is present;
- an unapproved axiom appears under the active policy.
