# Actual signed equality and balanced integer equivalence

Status: scoped implementation checkpoint; T02-W01 remains In progress.
Entry: `c4bd126d0e347f09db444d35df3831c61e1fd4c6`.

The [producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_equality.rs)
proves source Equal/NotEqual correspondence to the actual balanced unbounded
Int.Equal at all8/16/32/64 bits. It also proves both directions between actual
source equality returning true and Int.Equivalent of the signed interpretations.
For full unsigned values a,b, modulus M and actual sign bits sx,sy, the integer
equality is the established natural comparison

```text
Equal(a + M*Bit(sy), b + M*Bit(sx)).
```

The real source Word.Equal compares every actual bit and folds Bool.And in a
balanced tree. Checked conjunction elimination follows its right subtrees in
3/4/5/6 steps, reaches the actual top-bit comparison and reflects it to equality
of the real signs. Existing EqualNatural connects actual word equality to Nat
value equality. The scalar proof uses this fully discharged coherence function
and real word bounds. Matching signs use common additive translation; opposite
signs use genuine strict order and contradiction. NotEqual follows by checked
Bool.Not congruence, and the established Int reflection/completeness laws supply
both equivalence directions. No caller truth flag, operand enumeration,
extensionality, arithmetic opcode, axiom or new core rule is introduced.

The [dependency measurement](dependency-analysis/result.json) compares the
alternative half-threshold proof closure: it would add382 members to the prior
3457-declaration component. Projecting the actual source equality tree needs
none of those extra members for this proof. The complete original signed-bounds
component and full56-family requirements remain intact. Structural dependency
planning supplies no proof authority.

The [component](final/tests/certificates/word-signed-equality.mpcert) has3489
declarations,72690 terms,1007869 bytes and zero axioms. All fixed4096/131072/
2097152 caps and1024 conversion fuel remain unchanged. The607 remaining
declaration slots are this component's margin, not whole-foundation/application
budget admission. All3457 previous signed-order types/bodies/proofs/levels/
families match exactly. Its entire regenerated raw SHA256 remains
`0f61e8eb6a89cac3e2fa786f6027901424e2edaeb730df5dee0c5e8db49e8587`.
The shared builder refactor preserves its module and wire bytes. Imports are
empty and both unchanged cores recheck all included proof bodies.

All three [affected Rust roots](final/tests/result.json), selected strict
[Clippy](final/clippy/result.json) and [format](final/format/result.json) pass.
The [paired record](paired-final/result.json) covers46 complete same-byte streams:
two zero-axiom acceptances and44 genuine CORE_TYPE_MISMATCH rejections, comprising
38 check mismatches and six lambda-domain mismatches. Controls falsify new sign/
integer/equivalence claims, replace actual Equal/NotEqual by well-typed constant
Boolean functions, erase real value-equality/bound premises and weaken the actual
sign-coherence callback while retaining its actual value-equality domain.
An honest extra valid definition passes both cores and fails exact owning import.
Deterministic regeneration, appended-byte rejection and the original byte limit
are checked separately. No format/fuel/resource rejection counts as a semantic
negative. Go's positive report has no axiom_count field; zero axioms bind the
exact checked empty report hash and full Rust report for the same complete bytes.

The [fresh native observer](native/result.json) reads the entire unchanged
1640-query document and executes actual typed signed and unsigned ==/!= for all96
original equality rows, retaining the actual signed decimal values. It owns the
new [observer source](../../../../../probes/go-practical/word_equality_observer.go)
verbatim, builds with an empty cache on the pinned Go1.25 image and produces
identical complete output twice. All5577 original compiler files are checked
before and after. Affected [vet/format](native-checks/result.json) also pass on
that same verified image.

The [independent comparison](finite-counterparts/result.json) reuses all96
original decoded Bool results without rerunning unchanged evaluation, compares
every1892 original Bool identity and all3457 signed-order identities, and checks
the fresh signed/unsigned observations against unbounded Python integers. It
retains full original query objects, actual bit comparisons/right-tree steps,
sign coherence, signed decimal values and balanced cross sums. Every original
1640 result and all40 actual native runtime failures remain cross-checked with
their full original schemas, source/query/output/image evidence. Finite checks
execute no new Nat/Int/theorem bodies and supply no universal proof authority.

The [checkpoint record](verification.json) binds full source snapshots, commands,
raw output, proof bytes and results. The initial complete construction passes
and its bytes equal the final complete positive. All255
[frozen members](frozen-design-preservation.json) remain exact. Strict changed
input review uses the unchanged127-rule/four-scope scanner and39 historical seals;
only two exact hash/length original libtest stdout EOF observations are retained.
No source/JSON/directory or blanket whitespace exclusion is added.

Full56-family bounded ordinary assembly and source/carrier/nominal/type/operation/
storage/lifetime/state/program bindings, descriptor/private registry/context and
retained-policy delegation remain due. Original application source/export/
selected-closure authoritative immutable loading/replay and all169 application
demands/T02-T11 remain open. R06 stays pending and W01 remains In progress.
C# tests stay skipped; the cumulative T02 gate stays W04:
`./scripts/check-fast.sh --skip-csharp`. No public profile, whole source/program/
application proof, immutable application replay or native Linux release is claimed.
