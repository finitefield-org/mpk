# Practical Go request selection

The private selection importer checks the frozen
`mpk.selection.go_functions.v1` envelope and derives its canonical bytes and
domain-separated digest. It retains each raw package, function and nullable
value-receiver selector. It checks the pinned Go Unicode identifier table,
rejects keywords and blank names, distinguishes package `init` from a method
named `init`, and requires one to 8,192 unique roots in canonical byte order.
The raw package grammar remains the frozen request grammar; source-path and
alias resolution are later checks. Programmatically constructed duplicate JSON
members cannot acquire a selection identity.

The targeted Rust selection test passed all four tests, exercising all 29 Go
selection cases from the complete 152-case frozen context corpus, the exact
8,192/8,193 boundary, duplicate members, raw receiver aliases and an identity
independently calculated by the frozen Python canonical/hash observer. The
existing five source-overlay tests also passed because that consumer shares
the same pinned Go identifier rules and later resolves selectors against
original source. Strict Clippy and Rust format checks passed for the changed
selection and shared-validation code. These tests were selected for the new
request boundary and its shared identifier consumer; unrelated language and
proof tests were not rerun.

The imported selection alone is neither a registered semantic context nor an
authorization to use the practical profile. Declaration existence, source
capture, alias-resolved duplicate roots, retained-profile policy and proof
authority remain separate. T02-W01 and R01 remain open, public selection remains
inactive, and the cumulative `./scripts/check-fast.sh` gate is deferred to
T02-W04. No new validation archive is committed.
