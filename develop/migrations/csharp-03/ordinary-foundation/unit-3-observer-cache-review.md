# W09 test observer cache experiment — direct review checkpoint

This reviews only the test observer and its new targeted probe. Units 3–8 and
W09 remain open; the actual workload experiments have not completed. This is
not an acceptance receipt or an original-unit commit boundary.

## Findings addressed

- Adding `csharp_practical_ordinary_test_eval_tests.rs` introduced one new
  `Std.` consumer. Independently rebuilding the search paths and removing that
  one path reproduced the old 126-path hash exactly. Updated the 127-path
  fingerprint, aggregate edge count (4,950), and inventory byte hash; the exact
  inventory test passed in 15.98 seconds. No search exclusion was added.
- Scoped rustfmt found a parent module ordering difference; corrected it and
  checked the observer, its tests, parent, domain test, and inventory test.

## Reviewed invariants

- Free-variable analysis includes runtime application arguments and both Let
  value/body dependencies, shifts the body under each binder, and treats high
  variables or missing/forward children conservatively. It does not force an
  argument or evaluate erased type annotations. A closed constant continues to
  use its own empty environment.
- An unused argument retains its De Bruijn slot; only the slot's value is
  replaced. Captured outer variables remain at their original indices.
  Results with a used, not-yet-demanded argument are not marked independent.
- Application continuations consult the immutable term's original arguments,
  preserving order and laziness without copying the vector.
- Closed-definition cache lookup is scoped to one evaluation and Certificate.
  Its FIFO queue has one entry per live root, with a maximum of 64. It does not
  force an unrequested definition or identify generated operations by name.
  This root limit does **not** bound the transitive graph retained by a root.
- The new string duplicate probe selects one existing source/context/input
  case. The original entry point still runs all six string/compound cases;
  expected counts, actual generated terms and model validation remain intact.

Ten affected core tests passed, including 736 independently evaluated Boolean
truth cases, high binders, captured/obsolete environments, laziness, repeated
closed-definition work, cache eviction and certificate isolation. Library and
integration lint passed. See `unit-3-observer-cache-experiment.json` for commands,
selection reasons, hashes, stopped experiments and outstanding live runs.

No additional semantic correctness finding was identified in this scoped
review. Actual workload runtime and retained-memory behavior remain unverified;
the optimization is not yet treated as a completed component. Previously passed
decimal source/checker evidence remains evidence for those exact certificates,
not evidence that this new observer has finished its workload checks.

The repository-wide gate remains deferred to T06-W12. Application/native-body
proofs, original-unit completion, commit and push remain outstanding.
