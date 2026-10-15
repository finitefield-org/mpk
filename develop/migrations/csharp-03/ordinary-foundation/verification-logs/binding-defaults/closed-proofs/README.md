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

`tool-sources/audit.py` binds the pinned metadata and bytes to the original
45-source manifest, independent Go/Rust checker reports, observed process exit
codes, and one-bit hash mutations. Its receipt is added when every stage ends.
The repository-wide gate remains deferred to T06-W12.
