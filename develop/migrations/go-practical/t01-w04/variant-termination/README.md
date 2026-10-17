# Private ordinary decreasing-rank termination

Status: W04 mathematical feasibility checked; source binding and freeze remain
open. Entry: `540ee9033324c62b7f204f3bca337f2f4c976e81`.
The [receipt](observations.json) records ten ordinary candidates and three
retained regression certificates, checked from identical bytes by Rust and Go.
Three candidates are accepted with zero axioms; seven altered candidates fail
core type checking. A rejected candidate is not a certified counterexample.

## Checked signatures and equations

Use the complete function carrier `Frame = Bool^7 -> Bool` (128 state bits),
with arbitrary functions `H : Frame -> Bool`, `N : Frame -> Frame`,
`I : Frame -> Bool` and `R : Frame -> Nat`. H is the stopped flag, N a
transition, I the invariant and R the mathematical natural-number rank.
The earlier [ordinary order definitions](../nat-termination/README.md) define
`Lt(a,b) = Eq Bool (Less(a,b)) true`.

```text
Step(s)         = if H(s) then s else N(s)
Iter(f,0,s)     = s
Iter(f,succ n,s)= f(Iter(f,n,s))

Preserve : (s : Frame) -> I(s)=true -> H(s)=false -> H(N(s))=false
           -> I(N(s))=true
Decrease : (s : Frame) -> I(s)=true -> H(s)=false -> H(N(s))=false
           -> Lt(R(N(s)), R(s))

VTerminates : Preserve -> Decrease ->
              (n : Nat) -> (s : Frame) -> I(s)=true -> Lt(R(s),n)
              -> H(Iter(Step,n,s))=true

VRankTerminates : Preserve -> Decrease ->
                  (s : Frame) -> I(s)=true
                  -> H(Iter(Step,succ(R(s)),s))=true
```

H, N, I and R are explicit preceding arguments of the last two statements.
Every displayed equality is ordinary typed Std.Eq; no implicit assumption
is added. The rank corollary derives `Lt(r,succ r)` by Nat.induct and supplies
the bound itself. Counts and ranks are arbitrary genuine Nat values, with no
finite unrolling, fixed machine-word iteration limit or producer enumeration.

Preservation and decrease apply only when both states are live: an exiting
transition requires neither. The accepted zero-rank exit instance uses an
arbitrary full input frame, an observable exit bit, a transition setting the
complete frame to true, a true invariant and constant zero rank. It supplies
ordinary proofs of all premises and applies VRankTerminates. This demonstrates
that a rank-zero state may exit immediately without a fabricated decrease.

The proof derives strict-order transitivity and the bound lemma
`a<b -> b<succ(n) -> a<n`, then inducts on n. The zero case eliminates the
impossible rank bound. The successor case splits the current stopped flag and
then the next stopped flag. Already stopped states absorb every remaining
iteration. A continuing next state uses Preserve, Decrease and the inductive
hypothesis. Ordinary equality congruence and an iteration-advance lemma link
the complete frames; no eta rule or function extensionality is added.

The altered candidates separately weaken preservation, strict decrease,
initial invariant or the general bound, relax the bound by one, change the
stopped result, or drop the successor from the rank-derived count. All seven
fail with CORE_TYPE_MISMATCH in both private checkers. Resource failure or
fuel exhaustion is never classified as one of these rejections.

## Application binding decisions and remaining obligations

The source binding in T06-W03/W04 must use checked loop boundaries. A boundary
stores the already evaluated condition and control state; H reads that state
without reevaluating a source expression. The initial condition evaluation is
a separate terminating, safe, domain-valid source obligation. A live N step
executes the body, applicable post statement and next condition evaluation in
the specified order, with all calls and joint results evaluated exactly once.
The source/model theorem must show this complete transition equals N.

Normal completion and continue take the proper post/backedge path. Loop break,
return and a false next condition produce a stopped frame. A break consumed
by a switch does not stop its enclosing loop. Missing conditions, range
snapshots, language-version iteration bindings and nested loops retain the
[operation/loop decisions](../operation-loop-decisions.md); this theorem
does not replace those source laws or their tests. Exit payloads, result slots,
regions, alias observations and all other frame fields remain in the transition
and its checked frame theorem.

For totality, every condition, body, post statement and called function must
terminate; nested loops need their own discharged obligations. N is already a
total function in the logical schema. A potentially diverging Go computation
cannot be justified merely by encoding it as N. The source proof must also
establish the state representation, initial invariant, continuing-backedge
preservation/decrease, arithmetic rank bridge and domain/safety/frame checks.
An annotation or callee summary supplies a candidate obligation, never a trusted
entry oracle. Partial correctness still cannot be presented as total correctness.

An integer decrease with wraparound is not this Nat decrease. Source word/Nat
bridges, signed-domain side conditions and exact overlay expression typing
remain required. Automatically generated range ranks remain candidates until
their source/storage/length and backedge bindings are checked. The 128-bit
carrier is a representative full-frame proof, not the complete practical-Go
state representation or a frozen state-size restriction.

R020 retains ordinary foundation ownership at T02-W03; T06-W03/W04 owns the
source and totality bindings. No wire tag, schema, parser, public profile,
normative budget or installed strategy is introduced by this checkpoint.

## Checker and verification scope

The new untrusted [producer](../../../../probes/go-practical/variant-certificates/src/variant.rs)
layers on the retained producer sources without editing them. All 77 private
checker source files are byte-identical to the preceding type-case proposal;
no new trusted computation rule is needed. The rebuilt Go checker binary also
has the same identity. The Rust binary includes the new untrusted generator,
while its verifier uses those unchanged mirrored sources.

The retained general Sort0 strong-induction, arbitrary-Nat invariant and legacy
Bool certificates still pass with zero axioms and matching hashes. Two identical
generations and a fresh-build replay check candidate and binary identities.
The report pins source inputs and the exact Go1.25.0 distribution. The private
60-second child ceiling is a measurement setting, not a profile limit or a
guarantee about whole-program proof cost.

This receipt pins 152 source inputs and 77 mirrored checker files. Its largest
candidate is 8,475 canonical bytes, 921 stored terms and46 declarations.
Twenty-four evidence/adversarial checks pass; fresh replay reproduces all
candidate and checker identities. Current metadata checks cover 20 strict
JSON documents, the preceding39 proof/interface/regression cases, two retained
construction diagnostics and143 unchanged predecessor inputs. These selected
checks do not replace the later complete source/application gates.

The [verification record](verification/verification.json) separates selected
checks from the cumulative T01 gate deferred to W05. W04 remains In progress:
exact source/overlay/wire/hash/limit and migration freeze, remaining foundation
laws, complete application binding and release evidence are still required.
No T is complete and no practical Go profile is active.
