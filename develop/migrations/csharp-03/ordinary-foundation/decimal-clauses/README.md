# Decimal source contract adapters

Nine source contexts cover 44 public decimal operations and 49 attachments.
The internal value_equality operation remains covered among 45 frozen adapter
signatures; x.Equals(y) is not an admitted native API. It is not used to bypass
the frozen frontend. Ten selected source conditions, including five failures, pass: nine in the
final v7 run (1365.79s), plus the byte-identical completed v5 add overflow case.
The earlier overall v5 test failed at a subsequently corrected char fixture;
only its completed add context is reused. All nine same-byte checker cases
also pass (883.971s).

Decimal arithmetic exports independent raw contract-failure predicates while
preserving existing result, success and ordered exception declarations. The
shared circuit cache may select different internal helper names by emission
order. Dependency comparison permits only those internal names to differ and
checks all bodies, types, levels and transitive dependencies. Public names are
exact, and a changed shared-helper body must reject.

Capture provenance is in capture-receipt.json; component status and selected
verification reasons are in ../unit-4-decimal-clauses-progress.json. These
helper certificates do not prove native method bodies or application VCs.
Original units 3-8 and W09 remain open. check-fast.sh is deferred to T06-W12.
