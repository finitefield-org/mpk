# W09 boundary field decoders — review in progress

The new generator takes immutable `EmittedDataPhase`, reconstructs the boundary
VC contracts and emits field decoders in the same Builder as the existing typed
JSON parsers. Each field retains its contract hash, field/source/payload types,
semantic carrier, exact name matcher, decoder and complete packet projections.
Optional omission retains a validated ordinary literal and its complete semantic
cell count. Required omission has no value definition.

Supplied values use the exact configured primitive codec or the existing typed
compound parser. Option/presence fields wrap the decoded payload as some/value
and add one semantic cell. Null becomes none/null only when nullable. The raw
JSON depth is checked without adding depth for an implicit semantic wrapper.
Invalid packets, including padding, are zeroed. The full-document layer must
still apply canonical typed-value bounds and source-domain/reconstruction rules.

Two review corrections are implemented:

- JSON null is classified before payload acceptance. A non-nullable field must
  reject it even if the ordinary payload parser itself accepts null (Unit).
- An Int64 field with the Unix-millisecond codec needs the registered Instant
  adapter even when no source binding makes Instant otherwise reachable. The
  initial source run passed six contexts, then failed closed at this missing
  adapter. The correction adds only that validated codec dependency. The focused
  raw-instant run passed ten complete packet cases, including Int64 minimum.

The 432 complete header cases pass, covering semantic wrapper counts, inclusive
65,536 total cells, invalid/high counts, raw depth 32/33, and reserved-bit masking.
The existing rich source-product generator reproduces its exact pinned metadata
and certificate, including child dependency closures. Integration lint and the
affected inventory test pass. The initial six source contexts retain their
runtime evidence only after exact current-byte comparison; their core cases are
not rerun. The other eight nullable/presence contexts also passed; see the final evidence below.

All 15 field-decoder candidates now pass same-byte dual checking and hash
mutations. Envelope integration separately implements field order,
unknown/duplicate rejection, omission handling, separators/end-of-input,
cumulative defaults/payload cells and canonical typed depth. Its evidence is
tracked in the envelope and depth-guard progress records; it is not an
application-VC proof. Source-domain/reconstruction and full canonical value
bounds remain open. Output, native/control/replay relations,
universal propositions/proofs, assembly and W09 mutation/predecessor acceptance
remain open. Units 3-8 and W09 are not complete. No component-only commit/push
or `check-fast.sh` is performed here; the T gate remains at T06-W12.

All source scopes have now passed: the six earlier contexts reconcile to identical current bytes, the corrected raw-instant context passes, and eight nullable/presence contexts pass18original documents/54complete packet cases. All15source candidates were hash-verified and published in `json-boundary-fields/`. Both unchanged checkers passed all15 exact pinned candidates, zero-axiom assertions and hash mutations in4754.368s. The PASS filename set and current certificate hashes were reconciled before recording success. Full W09 remains open.
