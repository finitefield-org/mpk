# Total conditional source clauses

The source-clause emitter now lowers the frozen conditional recipe in its
condition/when_true/when_false argument order. It verifies exact nominal branch
and result types, a Bool condition and empty parameters. Every result leaf is
selected with ordinary Bool elimination; no elimination into a function carrier,
new checker rule, generic construct or trusted evaluation is introduced.

The helper corpus checks Bool, i32 and a depth-19 array-shaped carrier, including
low and high addresses of both branches. Same-depth nominal mismatches and
unexpected parameters reject. The original-source case tests the public clause
`Amount > 0 ? Amount == 1 : Amount == -1` at seven signed boundaries. Existing
seven construction-source clause candidates and thirty signed observations
remain unchanged. Source tests passed in 2.21 seconds; helper tests in 0.01 seconds.

Changing the sidecar changes the captured compilation context. An attempted
old-context facts replay rejected and was replaced by a fresh capture of the
exact request, using the same pinned frontend and isolated container settings
as the public-domain source. `capture.json` retains request/response hashes.

The three candidates (helper, source clause and integrated structural/public
program) passed unchanged Rust/Go same-byte checking with zero axioms and
hash-corruption rejection. The new integrated-only run retained the earlier two
successful unchanged candidates. Its full source-clause dependency closure is
identical; source integration/import checks passed in 1.12 seconds. Combined PASS
sets and current hashes were reconciled. This adds total conditionals only; partial
operation definedness, remaining expression recipes, native body/control and
actual application proof assembly remain open. See
`../unit-4-conditional-clauses-progress.json` for current verification. W09 is
incomplete and the full T gate remains deferred to T06-W12.
