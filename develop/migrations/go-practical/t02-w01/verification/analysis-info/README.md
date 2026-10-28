# Go analysis module cache info: in progress

The private analysis module owner now reads and retains each original cached
`.info` from the same snapshot as its `.mod`, `.ziphash` and archive. The Rust
validator checks the pinned version and the Go metadata shape, including
unknown fields, case-folded field names and their input order, null values,
original time literals and the existing strict JSON limits. Go's time parser
accepts some spellings beyond strict RFC 3339, including one-digit hours,
comma fractions and offset boundaries; the original bytes are preserved.
VCS fields are checked as metadata types only, not trusted as VCS authority.

The [native observations](native/result.json) retain all 213 original source
objects, 133 vendor files and three original cache records. Go and Rust agree
on 36 original and adversarial observations (15 accepted, 21 rejected),
including string/depth limits, duplicate and case-folded fields, invalid UTF-8,
time spellings and unchanged opaque fields. Five [snapshot controls](inspection.json)
exercise all three infos through the module owner, including unread, wrong-version
and escaped-time rejection. The inspector recomputes all metadata and snapshot
identities and checks that every other original source and directory stays intact.

The [affected Rust tests](info-tests/result.json) pass both new roots, the
previous 20-case module comparison and the two namespace and two archive roots.
The existing complete pinned Go Unicode fold comparison passes for every scalar;
scoped Clippy, Go vet and changed-file formats pass. These checks were selected
for the metadata reader and its direct consumers. The cumulative T02 gate
`./scripts/check-fast.sh` stays deferred to final T02-W04.

These observations do not establish exhaustive JSON/time equivalence, original
root or VCS authentication, Go source syntax, source-to-binary correspondence,
private context or application proofs. The Go checks used local Linux emulation,
not native Linux release acceptance. T02-W01 remains open and the practical Go
profile remains inactive.
