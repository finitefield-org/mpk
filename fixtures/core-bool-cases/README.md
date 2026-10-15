# Dependent Boolean proof elimination

Both source-free checkers consume these exact canonical Certificate v0 bytes.
The generated `Std.Bool.cases` declaration has motive `Bool -> Sort0`, two
dependent branches and a Boolean major. Existing `Std.Bool.rec` is unchanged.
All ten positive certificates have zero axioms, empty proof-node tables and
empty theory-certificate tables.

Accepted cases are universal right identity, both constructor equations, an
open dependent motive, and both conjunction projections from an actual
`Eq Bool (and p q) true` premise. The projections retain both quantified
Booleans and the premise.

Rejected cases cover swapped branches (including both conjunction projections),
a higher-universe motive, a malformed interface, a nongenerated declaration,
an extra constructor before or after cases registration, a renamed recursor,
and nonempty universe arguments on the eliminator or a constructor. The two
`*-levels-open` cases need no constructor equation to expose the error: one
has a neutral major and one only proves reflexivity of a constructor term.

The producer and executed local reports are retained in
`develop/migrations/csharp-03/probes/boolean-proof-elimination-logs/`.
The kernel tests, Go tests and CSHARP-03-T01-W09 feasibility owner share this
corpus. These generic proof-capability certificates do not discharge any
original C# application VC.

Registration-order regressions cover preceding reducible and opaque values,
checked theorem proofs, dependent types and Pi domains, and nested Lam/App/Let
terms. These uses reject both before and after cases registration. Positive
controls retain legacy definitions and proofs without cases, unused term-table
entries, and universe arguments on an unrelated family.
