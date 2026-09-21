# Native Option and compound relations — scoped review

The public control program now completes the eight previously unresolved W03
operations in the eighteen-source W04 loop/pattern corpus. Frozen Option
constructors and presence operations reuse the existing outcome and Option-data
adapters. Compound equality uses the existing recursive structural-data adapter,
including active-payload semantics; it does not use the scalar bitwise fallback.
Every operation retains its exact W03 invocation, original source anchors,
operand bindings and normal-successor result binding. Normal execution is still
the conjunction of the success guard and result relation.

Foundation emission now moves its recursive relation and storage caches with
the owning builder. The initial control compiler retains those caches, and the
public extension can add Option foundations without redeclaring shared payload
helpers. The cache checks the VIR identity and rejects active recursive emission.
The legacy foundation wrapper starts with empty caches and keeps its original
emission order. The structural adapter change only exposes its existing emitter.
No checker, evaluator or domain semantics changed in this extension.

Review checked cache ownership across foundation, clause and public-extension
stages; definitions and aliases are installed before predicate compilation.
Already defined native operations are not recompiled. New native definitions
append to the prior list, and only previously unresolved operations gain
predicates. Unsupported families remain pending. This extension does not claim
coverage for a combined Option/sequence source that the corpus does not contain.

The targeted original-source test passed 81 additional runtime observations in
263.03 seconds: Some(empty/UTF-16 strings with an isolated surrogate),
Some(source-product zero/7/minimum i32), None/Some presence, compound equality
for all tag pairings and unequal payloads. Result mutations cover the tag,
payload and last physical address. The test checks source/SSA/anchor links and
regenerates/imports all eighteen complete candidates. All 106 W03 operations
now have normal predicates. One non-data native invocation remains unresolved.

Nine preservation tests passed. They check existing term/declaration prefixes,
prior metadata and all 98 prior native normal relations; the eight formerly
pending operations are the only allowed predicate changes. Source-frame
regeneration passed all 10,608 existing observations. Seven dependent tests
passed for Option, reference, sequence, structural-data candidates and nine
private slot pins (2,032 existing observations). These checks justify retaining
prior runtime evidence for unchanged bodies without repeating their long tests.
Clippy with warnings denied and format checking passed.

Only foreach_string, foreach_string_var, string_property and type changed
certificate bytes. The two foreach contexts have identical bytes. Three fresh
same-byte Go/Rust checks passed acceptance, report agreement, zero axioms and
hash corruption rejection. The other fourteen sources retain checker evidence
only after exact byte equality with the archived preceding pins. Regeneration
from two independently executed test modes produced identical candidates.

These are ordinary relation definitions and finite runtime observations, not
application proofs. Native invocations outside W03, straight-line/constructor
functions, source-exit production, execution witnesses, exceptional paths,
filter search/finally, loop invariants and full native/application proof
composition remain open. Units 3–8 and W09 remain incomplete; the whole gate
remains deferred to T06-W12. No component-only commit is made.
