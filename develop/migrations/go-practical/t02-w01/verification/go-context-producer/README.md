# Go candidate context producer

The Go frontend has a private producer for the inactive Go/Rust candidate
registry, practical-Go context and function selection request. It constructs
the pinned entries and foundation reference itself, checks the complete entry
and registry hashes, and hashes the complete context and selection into the
request. It accepts no caller-supplied registry, foundation, expected digest or
policy adapter. Roots are copied and checked against the frozen raw selector
grammar and canonical order; returned canonical buffers are independently
owned. The exact Go1.25.0 runtime is required.

The affected Go tests passed using the registered Go1.25.0 Linux/amd64 SDK in
local emulation. They cover the original shared canonical request, Unicode and
receiver cases, rejected malformed and unordered roots, the full 8,192/8,193
boundary and caller mutation of owned data. The changed Go package also passed
`go vet` and `gofmt` checks. The affected Rust context tests passed after adding
an import of the exact same canonical request; strict Clippy and Rust format
checks passed. The small checked-in request is a conformance vector used by
both consumers, not a validation archive. These checks were selected because
the Go producer and Rust importer meet at that byte-level boundary.

The Go producer does not establish installed-release or launcher authority,
source selection, checked foundation definitions or an application theorem.
The foundation reference remains explicitly candidate-only, public practical
Go remains inactive, and T02-W01/R01 remain open. The cumulative
`./scripts/check-fast.sh` gate remains deferred to T02-W04.
