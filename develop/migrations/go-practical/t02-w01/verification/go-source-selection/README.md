# Go candidate selection from original packages

The Go producer now resolves its complete inactive candidate request against
the original type-checked package owner. It reconstructs and compares all four
canonical request buffers before using any selector. Function declarations
are indexed once per selected package; receiver aliases resolve to the same
named declaration, aliases selecting one declaration twice are rejected, and
pointer receivers cannot satisfy a value-receiver selector. The resulting
physical roots must also pass the existing complete source/contract selection
owner. Returned root and request buffers are independently owned.

The private compiler handoff takes the original image, package owner and
candidate request, derives the roots itself, and requires the compiler input's
complete source selection to match. It keeps the request and compiler inputs
together and does not take caller-supplied physical roots. The separate Rust
consumer still independently checks the original source selection and held
root owners; neither producer is a registered launcher or proof authority.

The affected local Go tests passed using the registered Go 1.25.0 Linux/amd64
SDK under emulation. The original selection-input suite retained all 8,192
source declarations and compared their complete physical selection against the
new name resolver. The source-overlay suite retained the original 70-error
input and checked receiver aliases, duplicate aliases, pointer receivers,
imported receivers, a method named `init`, and a Unicode function. The rejected
alias/pointer/imported/missing selectors first pass the raw request grammar,
so their rejection is exercised at source resolution. Request
controls rejected changed or crossed registry, context, selection and request
bytes, and missing or closed owners. These tests were selected because the new
resolver shares their original source scopes and physical-selection owner.

The complete live capture then passed with the new Go compiler handoff and the
independent Rust test observer. The unchanged test application and dependencies
included 182 packages, 5,516 source objects, 181 export objects and all 5,577
registered SDK files (166,354,960 bytes). Go's `Root` request resolved to the
original declaration and matched the compiler selection; Rust accepted that
request against both held root sets and the test-only SDK owner, and rejected a
request for the original unselected `Unused` function. The Rust observer ran on
Linux arm64; this is not a native Linux release or installed-launcher test.
The directly affected original-compiler test also passed with its distinct
test bundle identity and the same complete application and SDK inputs.

The new request/owner guards were rerun after adding the compiler handoff. The
larger selection and overlay tests had already passed with the same resolver,
which was unchanged by that handoff. The existing fixed-Go launcher/profile,
canonical-hash and strict-JSON tests also passed with Go 1.23 on the local
host, because this source is compiled into the retained frontend package.
Go 1.25 `vet`, changed-file `gofmt` and `git diff --check` passed. No new
validation archive is committed.

The candidate foundation remains incomplete, and launcher/read authority,
native Linux acceptance, source/program proof and T02-W01 R01/R06 remain open.
The practical profile remains inactive. The cumulative
`./scripts/check-fast.sh` gate is deferred to the final T02-W04 work item.
