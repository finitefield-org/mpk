# CSHARP-03-T05-W03 canonical output evidence

W03 adds `EmittedDataPhase::capture_boundary_output` and
`import_boundary_output_run`. Both require the immutable W02 input run and
revalidate its original canonical bytes/provenance against the selected source
and context. The returned value must have the original method's exact result
type. The result is an immutable output run with the original returned value,
independently reparsed typed value, canonical document, output receipt, and
cumulative frontend manifest/source artifacts.

The output contains every stored source field in declaration order, including
storage that a semantic binding does not project. It uses the shared frozen
scalar and structural codecs. Internal semantic values have their fixed tags,
active payloads and ordered entry arrays; source products retain their complete
source structure. This avoids treating projection or an unproved reconstruction
as field-complete source equality. Arrays and immutable sequence snapshots use
the existing common value representation. Decimal equality follows the frozen
value semantics (scale/sign of zero are unobservable); other fields, enum tags,
UTF-16 units, array order and floating-point bits remain exact. Lossy configured
decimal formatting fails reparse equality.

The typed decoder is independent of encoding. It reconstructs values from the
emitted canonical UTF-8 bytes under the original source type; all value limits
and equality are checked again. The complete document, including the declared
boundary field name, is independently reparsed. Root objects count toward the
65,536-cell bound. Output documents are at most 1,048,576 bytes, depth is at most
32, strings at most 16,384 UTF-16 units, and each bounded array at most 4,096
items. Void methods encode `{}` and require the exact unit result type.

Only then does W03 create `mpk.csharp.boundary_output.v1` and link both boundary
input and output captures into a new frontend manifest. The original VIR,
source map, source inputs, contracts, bindings and closed operations remain
linked. The source-artifact root binds this manifest. Reproduction compares
exact document bytes and all complete regenerated artifacts; replacing a
hash, context, source value, reparsed value, input provenance or manifest fails.
The output API returns no run/partial artifacts on failure and emits no parser
or serializer exception text. A transport-only T02 receipt or output-shaped
sidecar does not create a validated output run.

This is reproduction evidence. It does not prove that an external adapter ran,
that its production serializer preserved meaning, or that the application
produced a caller-supplied value. Ordinary VIR execution/proof and pending
source invariants, postconditions and projection commutation remain with their
existing owners. W04 transitions and public activation are not implemented.

## Retained files and verification

- `source-requests.json` and `source-responses.json`: three actual C# compiler
  captures from the existing pinned local Linux control-emission harness.
  `Payload` constructors are reachable and reconstruct all their stored fields.
- `conformance.json`: canonical source-product and void output goldens and
  deterministic capture/manifest/source-artifact bytes.
- `review.md`: local task review and resolved findings.
- `verification.json`: final local command results and file/dependency hashes.

The primary owner is `crates/mpk-cli/tests/csharp_practical_boundary.rs`; W03
cases are in `support/csharp_practical_boundary_output.rs`. Tests include
surrogate output names/strings, nullable product fields, two-run reproduction,
field/tag order and token/escape mutations, rehashed capture fields, wrong
return types, stale manifests and input-provenance splicing. An actual-source
32-field array/string product isolates cell and document byte bounds at
minus-one/exact/plus-one with a small input document. Unit tests in
`csharp_practical_boundary_output.rs` cover every admitted primitive and
structural family, source arrays, tagged outcomes, selected codecs and invalid
typed values. Existing W02 depth/canonical parser regressions remain active.

Local commands:

```sh
cargo test -p mpk-cli --test csharp_practical_boundary
cargo test -p mpk-vc --lib csharp_03_t05_w03
./scripts/check-fast.sh
```

The compiler capture is run offline with `mpk-java-t10-gate:local`,
`--platform linux/amd64 --network none`, using
`./scripts/build-csharp-practical-frontend.sh --test-control-emission-requests`.
No frozen producer inventory, historical probe record, published vector,
workflow or installed frontend has changed.
