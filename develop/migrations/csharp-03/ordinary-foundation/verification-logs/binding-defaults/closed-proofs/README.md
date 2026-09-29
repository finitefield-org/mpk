# Closed actual-default Boolean proof candidates

These three source-bound certificates prove equality of an already-closed
`actual_default` Boolean condition with `true`. The corresponding metadata
retains every original W06 application proof ID as pending. Ineligible defaults,
source-use absence, and complete W06 binding proof assembly remain open.

The scoped regression test
`csharp_03_t06_w09_binding_defaults_closed_boolean_proof_candidates` compares
all six generated files here byte for byte, checks exact original pending sets,
and rejects changed metadata and certificate bytes at the importer. The
original 45-source default test continues to pin the definition-only output.

The independent Go/Rust checker review is recorded separately when complete.
The repository-wide gate remains deferred to T06-W12.
