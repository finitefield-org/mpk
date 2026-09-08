# T06-W03 review and fixes

Scope: data VC generator, validated-VIR context retention, VC/resource/group
integration, W01/W02 consumers, original-capture tests and completion records.
Review was performed directly, without delegation.

1. **Private construction carriers entered public structural specialization.**
   Initial W01/original-construction regressions rejected legal allocation
   signatures. Exclude sequence-construction carriers from comparison recipes;
   retain their concrete carrier and independent ownership evidence instead.
   Existing initialization/attachment tests now pass.
2. **Output-bound predicates could mention a not-yet-produced result.**
   Derive every failure predicate from invocation operands, including the
   computed mathematical output length. The semantic result is bound only in
   success/tagged-result goals. Corpus tests typecheck every prefix/failure
   predicate with the result binder removed; the old version fails this check.
3. **Shared ordinary references were counted twice across W02/W03.**
   Union W01, construction and data definition names before cumulative resource
   reservation. Keep per-program limits and each sequent reservation. The
   construction consumer now independently checks the exact cumulative
   declaration count as well as term nodes; the old sum fails this assertion.
4. **Final review: no actionable findings.**
   Reviewed actual SSA scopes, first-error guards, static versus tagged versus
   exceptional paths, original attachment and ownership retention, primitive
   carrier parameters, immutable capability reconstruction, budgets, dependency
   propagation and schema-order-preserving mutation tests. Reviewed the complete
   tracked diff and both new Rust files. W02 golden changes affect only current
   VC hashes, not construction handoffs or source VIR hashes. No public schema,
   producer/vector bytes, core/checker rule, runtime oracle or activation changes.

Verification is deliberately scoped to VC generation/import and its consumers.
Boolean success/counterexample witnesses establish guard composition with
semantic predicates as atoms. They do not claim that arithmetic/library
semantics have passed a kernel. All data relation definitions and goals remain
pending until W09's ordinary expansion/proof/checker work. The W03 private
handoff cannot mint a proof or accept caller-supplied discharge. The full T06
check-fast gate remains deferred to W12 under AGENTS.md.
