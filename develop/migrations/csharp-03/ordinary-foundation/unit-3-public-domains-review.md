# Recursive public domains: partial direct review

This is a component review, not unit 3/4 or W09 completion.

The representation-only path has an explicit `None` clause profile. Its source
product and enum representative certificates retain identical canonical bytes.
The new profile shares the source-clause emitter's builder/storage cache and
keeps the previously pinned standalone source clauses unchanged. Public domain
counts recursively call filtered child counts; each source wrapper requires all
compiled public clauses and retains the invalid-count sentinel. Metadata import
reconstructs the complete expected program and certificate.

Review found two test setup errors before runtime coverage: a shadowed integer
text binding was returned instead of the product value, and the source array
was assumed to remain an Array variant even though source lowering yields a
bounded Sequence. Both test helpers were corrected. A nonzero inactive Nullable
payload case was added alongside None's valid zero storage to distinguish active
clause filtering from representation padding validation.

The original nested source was initially rejected for an unreachable constructor.
Its root now invokes that constructor; the frozen frontend accepted the source
and returned facts. Namespace UID-map and tmpfs execution failures were container
configuration errors, not source or proof rejections. The successful capture uses
only staged required inputs, the pinned Python image, no network, a read-only
root and input mount, default seccomp and explicit namespace capabilities.

Current runtime jobs and remaining verification are recorded in
`unit-3-public-domains-progress.json`. Full generated candidate checking, richer
source-clause recipes and the remainder of W09 are still outstanding. No clean
whole-unit review, commit or W09 completion is claimed. The repository-wide gate
remains deferred to T06-W12.

The public-default extension emits the unchanged structural default definitions
and a separate ordinary Bool combining their declared admission with public
membership of the actual default value. A missing structural candidate produces
false. Nullable None still follows its absent arm; the default path does not
traverse the type argument's source conditions. Plain public-domain generation
omits the optional extension, and imports reject profile substitution.

Review of the original captures corrected another test assumption: the type
contract's structural default eligibility is distinct from the captured
`public_default` declaration. The test now observes domain membership and final
admission separately, uses the captured declaration independently, and adds the
existing seventeen nested sources that explicitly declare public defaults.
The corrected six-source runtime suite passed. All six current candidates also
passed unchanged Rust/Go same-byte checking with zero axioms and hash-corruption
rejection; the exact PASS set and hashes were reconciled. Lint found one
unnecessary test Box allocation, which was corrected; lint and inventory then
passed. No further finding was identified in this default-predicate extension.
Deep recursive-domain tests and the whole-unit review remain outstanding.

The retained nested-source runtime run completed successfully in 3976.01 seconds.
All eight cases passed, including direct and active array/Nullable clauses,
valid empty/None, later invalid array element and nonzero inactive None payload.
No duplicate long run was started. The run predates the allocation-only helper
cleanup and subsequent relation-cache extension; predecessor preservation tests
separately confirm unchanged prior generated definitions. Remaining unit and
application-proof scope is unchanged.
