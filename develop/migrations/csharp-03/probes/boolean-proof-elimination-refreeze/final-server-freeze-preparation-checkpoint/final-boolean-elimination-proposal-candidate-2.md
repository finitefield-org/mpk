# W09 Boolean proof elimination: proposal requiring a design decision

The user approved this design amendment on 2026-10-07. T01-W09 is reopened for
the proof-capability feasibility and renewed freeze; T01-W10 and further W09
proof assembly wait for that freeze. All 987 application proof IDs remain
pending, and the practical profile remains inactive. Implementation and
verification of the generic core rule are in progress; no original VC is
weakened by the amendment.

The initial implementation checkpoint passed local targeted tests, lint/format
and 44 standalone checker stages over six positive, thirteen negative and three
predecessor certificates. That checkpoint and its original review remain
historical evidence in `unit-7-boolean-elimination-review.md`.

The subsequent registration-order review found a checked nonempty universe use
preceding canonical cases registration. The fix rejects such reachable uses in
all preceding checked declaration types, definition values and theorem proofs.
The expanded corpus has ten positive, twenty-one negative and three predecessor
certificates; all 68 standalone checker invocations pass locally and on Linux
at exact public source `fd99c03c8e3535bcc31f0b9d1441d4b74de80ebf`.
The corresponding local and Linux evidence is retained in
`../probes/boolean-proof-elimination-refreeze/registration-order-checkpoint/`
and `../probes/boolean-proof-elimination-refreeze/registration-order-linux-checkpoint/`.
The unchanged original capacity and recursor probe owners also reproduce all
27 complete probe certificates, and all 108 fixed-checker invocations pass;
`../probes/boolean-proof-elimination-refreeze/registration-order-capacity-recursor-checkpoint/`
retains those reports and refreshed source inventories. The subsequent exact
structural term-search index changes only Go interning lookup; independent
source reconstruction preserves all inference, reduction, fuel, declaration
and canonical cases registration bytes. All 34 Boolean/predecessor fixtures,
six exact pattern pairs and 54 capacity/recursor reference executions pass on
the approved Linux server at public source
`7505badc6684191a7dad9833e67d29af32529ee9`. Complete reference JSON results agree
with the retained original fixed-checker results; the original Rust observations
keep their original source/binary/host provenance. See
`../probes/boolean-proof-elimination-refreeze/go-term-interning-checkpoint/`.
Renewed freeze and publication remain required until their final reviewed
completion records.

## Observed proof-generation gap

The original `is_binding` first refinement retains its complete packed
environment and all eight checked scope projections. Its generator reaches
environment field 21: the `source_entry_assigned` flag for `parameter:0`.
The native source execution has 22 Boolean conjuncts. A checked proof that the
complete execution predicate equals `true` does not itself expose a proof for
each nested conjunct. The current normalizer consequently returns `Linkage`;
the flag must not be supplied as an invented equality hypothesis.

The generated Boolean recursor has the interface
`Bool -> Bool -> Bool -> Bool`. Its false and true branches cannot contain
equality proofs. Both checker implementations recognize the canonical
`<family>.rec` name and constructor equations; a differently named recursor
does not acquire reduction rules merely by being declared.

The capability diagnostic uses unchanged actual standard interfaces and
canonical certificate bytes. Both checkers accept the universal goal definition,
its two closed constructor cases, and a positive universal reflexivity control.
Both reject reflexivity as a proof of `forall b, Eq Bool (and b true) b`, and
reject equality-proof branches passed to the existing Boolean recursor.
The universal attempts use explicit Pi theorem types. The earlier named-type
attempt also encountered named Pi inference and is not evidence of the neutral
Boolean equality failure. These results rule out the two tested constructions;
they are not a proof that every alternative construction is impossible.

The original fixture and prior complete-source replay remain authoritative.
This small diagnostic neither supplies its seven refinements nor replaces an
application proof with constructor-only observations.

## Approved implementation change

Add one generic, generated Boolean eliminator for Sort0-valued motives:

```text
<family>.cases :
  (P : <family> -> Sort0) ->
  P <family>.false -> P <family>.true ->
  (b : <family>) -> P b

cases P no yes false = no
cases P no yes true  = yes
```

Keep the existing `<family>.rec` declaration, type, arity, equations and hashes
unchanged. The new rule applies to independently checked Boolean families with
exactly their two canonical nullary constructors; it has no C# namespace or
source metadata dependency. Both kernels must check the complete canonical
eliminator type, generated status, family reference and constructor declarations
before reduction. Neutral majors remain neutral. Nongenerated, renamed,
malformed, wrong-motive and wrong-branch declarations fail closed.

Use existing Certificate v0 Recursor/Lam/Pi/App fields. Introduce no proof-node
entry, theory certificate, theory primitive, trusted source observation or
axiom. The acceptance rule is nevertheless an extension: an additional
recursor name and equation become recognized by both checkers.

With this eliminator, right identity uses the motive
`P(b) = Eq Bool (and b true) b` and the two checked constructor reflexivity
proofs. The execution-premise route then needs checked general lemmas such as
`forall p q, Eq Bool (and p q) true -> Eq Bool p true`. For its false case the
given equality has the required type; for its true case reflexivity suffices.
These lemmas can expose nested execution facts without weakening the complete
scope, dropping native clauses, or assuming their consequences. The generator
must still reconstruct every original full refinement and check its final
proof; adding the eliminator alone does not complete W09.

## Required specification and validation work

The source design's section 25 previously required:

> Certificate v0 and both checker acceptance rules are unchanged;

Section 24 requires revising and re-freezing the design when a requested
capability fails feasibility. The explicit user approval authorizes this design
amendment and renewed T01 freeze for the generic core proof capability. The
earlier freeze is historical evidence; the amendment preserves predecessor
behavior and the inactive practical profile until its full acceptance gate
succeeds.

The amendment must specify the new canonical interface and equations in
`CORE_V0.md`, distinguish proof elimination from the Bool-valued carrier mux
equations in `CSHARP_PRACTICAL_FOUNDATION_V1.md`, and reconcile the source design,
frozen descriptor, bundle identities and task ledger. Approval does not itself
complete the capability's validation or the renewed freeze.

Validation must cover both constructor equations, open dependent motives,
capture and universe checks, malformed interfaces, wrong branch proofs,
nongenerated declarations and neutral majors in both implementations. The
same-byte corpus must accept universal right identity and conjunction
projections with zero axioms, reject mutated proofs, and retain the original
Boolean recursor and predecessor certificates. The next application check is
the complete original `is_binding` refinement corpus and its execution
establishment, followed by the remaining W09 assembly obligations. Targeted
checks and reviews are required throughout; each task's final whole gate follows
the repository instructions, with the current T06 gate deferred to W12.

The authorized next steps are the generic eliminator, its independent checker
validation and renewed freeze, followed by the complete original proof route.
