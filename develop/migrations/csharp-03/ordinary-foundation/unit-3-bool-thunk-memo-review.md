# W09 ordinary observer evaluated-Bool cache review

The test-only core machine cached lambda applications for immediate V::Bit
arguments, but ignored the equivalent Suspension::Evaluated(V::Bit) form. A
regression case forced a delayed Bool through the normal machine, seeded the
same lambda's Bool cache, then reapplied that lambda with the evaluated wrapper.
Before the fix, this repeated 3,203 transitions compared with 3,205 for the
uncached chain; the test failed. After the fix it takes four transitions for
both false and true. This is an operation-count regression case, not a claim
that complete source suites speed up by that ratio.

The cache-key lookup now also reads an already-evaluated Bool from a suspension.
It does not evaluate pending arguments or recognize any generated operation
name. Evaluated suspensions are immutable results in this pure core machine;
the key is exactly the same Boolean used for the existing two-entry cache.
Non-Boolean closures and pending suspensions retain the previous path. The
lookup borrow ends before execution/cache mutation. Original environment
release and result-closure retention behavior are unchanged. No checker,
certificate-generation rule, proof assumption or application semantics changes.

The regression also passes an invalid pending term to an argument-ignoring
function to enforce laziness. Six other checks cover unused arguments, released
and retained environments, deep environment destruction, sparse/dense selector
equivalence and generated integer circuits versus their independent network.
All seven passed in 0.37 seconds; targeted lint passed. The 65-source/456-case
integer parser suite then passed with this observer version and its fixed
certificates (279.26 seconds). This is not a controlled wall-time comparison. The previous long-running decimal and domain jobs are preserved;
they are not restarted or labeled failed because of elapsed time.

Direct review found no additional actionable issue in the key lookup. Completed scoped
consumer verification is tracked in unit-3-bool-thunk-memo-progress.json. The
observer provides finite test evidence only and never discharges a theorem.
Units 3–8 and W09 acceptance remain open; no component-only commit is made.
