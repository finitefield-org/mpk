# CSHARP-03-T06-W01 review

Scope: the independent verification byte parser, shared concrete type/value
semantics, complete ordinary expression encoding, attachment identities,
original-input VIR and VC capability handoff, resource accounting and owning
proof groups. No delegation, later obligation generation or proof discharge.

## Findings resolved

1. The existing closure walk bounded each method but omitted the frozen global
   8,192-node contract limit. Accumulate its already-derived per-contract counts
   and reject the closure before specialization and VC generation. Literal value
   objects remain opaque to that existing walk.
2. The control attachment path checks method clauses through both its loop owner
   and data attachment. Recording both would duplicate the verification input.
   The loop/exception owner now records those clauses once in its exact scope;
   the subsequent data validation retains its checks without recording again.
3. Attachment identities must include owner, entry/current/result permissions,
   the complete exception universe and partial-callable state as well as concrete
   roots and semantic signatures. These now participate in the recomputed digest.
   An old-state result or a captured partial constructor rejects, while local
   alpha renaming preserves the ordinary term and retains distinct input hashes.
4. Boundary and transition contracts inherited the old type-contract fallback
   proof owner in the VC model. Route their retained subjects to W07 and W08;
   the original-input handoff test checks the actual groups. Definition bodies
   and checked-operation definedness remain pending their later owners.
5. A pre-existing W06 test-helper visibility change used `pub(super)` at the root
   of the standalone transition integration target. That compiled when included
   as a child module but failed in its standalone target. `pub(crate)` permits
   both uses; the standalone W04 transition target is rerun.

## Final review boundaries

The byte importer never trusts a supplied normalized term, hash or declaration
name: it reconstructs the expected record from strict original expression bytes
and the reattached scope. It calls the shared type/value semantic validator,
not the frontend byte parser or attachment implementation. The validated record
has private fields and no deserializer. The ordinary term admits only explicit
typed variable, constant, application, lambda and let forms. Definition recipes
retain frozen metadata and checks, not source-language evaluation or axioms.

All 33 tags have positive typing/import comparisons and negative result-type
cases. Additional coverage checks malformed/duplicate syntax, partial/impure and
ill-scoped expressions, alpha normalization, exact transport mutation, preserved
UTF-16 values, specialized identities, inclusive resource boundaries and actual
captured type/method/boundary/transition attachment into VC import. The resource
mutation preserves canonical field order and asserts the resource error phase.

There are no remaining actionable findings in the final task diff. The
completed verification and hashes are recorded separately in verification.json.
The full gate is explicitly deferred to T06 completion; no historical result is
claimed as verification of this diff. Source producer, historical receipts,
frozen vectors, schemas and active release remain unchanged.
