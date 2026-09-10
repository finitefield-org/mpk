# Direct review: ordinary contract attachment expressions

No outstanding findings in this changed component after targeted verification.
This is not a complete internal-unit or W09 review receipt.

- The generator obtains expressions and obligations from the canonical data-VC
  generator, verifies attachment/subject correspondence and Boolean definedness,
  and lowers the actual obligation term. No reconstructed approximation or
  runtime result replaces W03 definedness. Unsupported recipes reject the whole
  result, and existing size/structure limits remain in force.
- Bound values use the exact ordered subjects and reverse lambda wrapping. The
  original concrete result type supports non-Boolean loop measures. Distinct
  binder markers and current/entry/result probes passed. Frozen empty loop owners
  are preserved rather than fabricated; concrete control locations remain open.
- Attachment identities distinguish method/loop/result scopes. Exact regeneration
  ties metadata and certificate bytes to the supplied VIR and foundation. Changed
  owner, subjects, old mode, definition names, subject order and cross-context
  imports reject. Shared symbols are reused only under canonical identities.
- Review identified a source-public collision: the same expression hash may
  have different nominal argument types. Both value and definedness now use an
  attachment suffix for those signatures. The two-owner i32/i64 regression
  checks actual different carrier depths, total integrated closure and undefined
  arithmetic; it would fail under the former shared symbol. Existing unambiguous
  names and four prior source-clause fixture families retain their exact bytes.
- Nine new candidates passed same-byte checking by both unchanged checkers,
  matching reports and zero axioms; actual hash corruptions reject. The exact PASS
  set, current module hashes, byte lengths, sibling metadata and tested output
  copies were reconciled. Lint, format, inventory and request pins passed.
- Failed development attempts are retained: incorrect test type/hash API usage,
  an incorrect nonempty-loop-owner assertion, a two-filter Cargo invocation and
  the data frontend's loop-handoff rejection. Each was corrected without changing
  frozen frontend semantics or weakening production validation.

Verification was selected for the new generator, shared compiler and its actual
consumers. Unchanged expensive arithmetic/full-capacity observations were not
repeated. These checked definitions do not discharge application VCs. Original
internal units remain incomplete, no component-only commit is made, and the
whole gate remains deferred to T06-W12.
