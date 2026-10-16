# T01-W03 feasibility exit audit

Status: In progress. This is an audit against T01-W03's acceptance condition,
not an expansion of W03 into production implementation. The
[work plan](../../../docs/09_go_practical_subset_design-todo.md#3-go-prac-t01--feasibility-and-v1-freeze)
requires ordinary representations, representative conditional theorems in both
checkers, workload measurements and rejection of unsound projections.
Production capture/import/lowering and complete source-to-certificate feature
applications remain required in their assigned T02–T06 stages.

| W03 requirement | Retained evidence | Exit status |
| --- | --- | --- |
| Nominal values and products/closed errors | [Initial symbolic pilot](pilot-corrected/observations.json); complete payload and conditional projections | Demonstrated privately; production nominal/error linkage is due in T03/T04 |
| Source-width arithmetic and mathematical sums | [Guard discharge](../../../docs/go-practical-implementation-traceability-ledger.md#13-t01-w03-comparisonsubtraction-to-addition-guard-discharge), continuing prefixes and [actual halting Quote](halting-returns/README.md) | Full64 operands, actual continuing/success guards and genuine Nat sums demonstrated |
| Complete bytes and actual indexes | [Substring/concat](stringop/README.md) and [prefix/suffix](affix/README.md) | Complete bytes and full64 reconstruction demonstrated; no Unicode-character interpretation |
| Descriptor/store and construction laws | [Descriptor/access](../../../docs/go-practical-implementation-traceability-ledger.md#19-t01-w03-computed-descriptor-validity-and-absolute-access-closure), reslice, overlap copy and construction checkpoints | Nil/empty, alias/copy/tails, physical guards and bounded construction demonstrated |
| Abrupt control and loop proof shape | Initial continue/post observation and complete halting-frame composition | Fixed finite-prefix proofs demonstrated; the old Std.Nat.rec is Nat-to-Nat only, so general invariants need a measured additive dependent-induction interface |
| Proposed standalone workload points and false projections | Byte 0/1/32/256 and slice 0/1/8/32/128, including proposed maximum minus/plus-one cases | Measured with complete payloads; per-case observations are private rather than profile budgets |
| Coupled bounds | [Struct/slice/byte matrix](aggregate/README.md) | 98 candidates and separate replay pass; 49 zero-axiom accepts and 49 core type rejections |
| Identified-hardware resources and exact freeze inputs | Adjacent per-case resources and host receipts | Isolated representative resource envelope and cumulative exit review remain due |

The remaining W03 sequence is bounded:

1. The corrected aggregate matrix, independent replay and all 37 affected tests
   are complete. Preserve their exact source/checker/artifact identities.
2. Check an additive monomorphic Sort0 Nat induction interface in private
   mirrored Rust/Go checkers, preserving the old Nat.rec interface. Check an
   ordinary induction/invariant and abrupt-control candidate set
   for arbitrary natural iteration counts, including continue/post, stop and
   erased-premise negatives. Its preservation assumptions are generated body
   proof obligations, not new caller entry assumptions.
3. Measure representative expensive positive candidates sequentially on the
   identified host. Record construction versus per-checker resources, actual
   boundaries and scope; distinguish recipe feasibility from guarantees for
   every possible source program.
4. Review every W03 acceptance row, record any required scope correction and
   mark W03 complete only when no feasibility question remains. W04 then owns
   exact admission, limits/budgets, schema identities, migrations and executors.

No production adapter, parser, registry or source-to-certificate application
is claimed here. T01-W05 owns the cumulative `check-fast.sh` gate. W03 does not
run it or reproduce it as separate commands.
