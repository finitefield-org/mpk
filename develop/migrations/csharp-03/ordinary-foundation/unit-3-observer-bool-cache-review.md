# Body-demanded Boolean observer cache review (partial W09)

The test-only ordinary evaluator previously populated a lambda's Boolean cache
only when the argument was already evaluated before application. A suspension
first forced by the body did not populate it. A fresh-closure regression failed
before the correction: a subsequent concrete Boolean application repeated all
3,205 evaluator transitions. Both Boolean keys are covered independently.

The correction inspects the argument again after the body returns and records
the result only if the argument is already a Boolean. Inspection never forces a
pending suspension. Cache ownership remains local to the same lambda closure;
no generated operation name, certificate identity, or host semantic oracle is
used. Arguments whose available values are functions, and suspensions still
pending after the body returns, do not acquire a Boolean cache entry. The frame retains the argument only until the
body returns. Existing environment-release and returned-closure tests pass.

Targeted verification includes the six generic observer regressions, three
aggregate consumers (modes/saturation/odd counts, invalid/empty short-circuiting,
and computed-predicate sharing), and original-source small domain values. These
exercise the changed evaluation timing and its direct fold/domain consumers.
The package's library/test clippy and the two edited Rust files' format checks
also pass. See `unit-3-observer-bool-cache-verification.json` for retained logs.
Production generators and certificate fixtures were not changed by this fix;
both-checker acceptance and full-capacity traversals are not rerun for it.

Direct review found no remaining actionable issue in this correction. It is
not evidence that the outstanding large domain or JSON-capacity cases have
passed, nor that their runtime is dominated by this cache miss. A two-sample
stack observation located the live string-map test in count evaluation but
cannot establish a statistical hotspot. The preexisting long-running processes
continue with their original evaluator executable; they were not restarted.

Unit 3, the remaining original work units, and W09 remain open. No application
VC proof or W09 completion is claimed. The whole gate stays deferred to T06-W12,
and commit/push remains at the approved original-unit boundary.
