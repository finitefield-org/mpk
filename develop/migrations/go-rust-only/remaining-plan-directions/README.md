# Go/Rust-only roadmap follow-up

The installed pipeline already accepts only Go and Rust. This follow-up removes
the remaining current instructions to start another-language phase from the
[completed Rust work record](../../../docs/05_rust_frontend_design-todo.md)
and the [shared-boundary audit](../../../docs/mlang-00-go-rust-shared-boundary-audit.md).
Both now direct development to the existing Go priority and serial T/W order,
retaining the checked Rust profile without another language's release dependency.

Complete prior document bytes and modes remain in [originals.json](originals.json).
Their historical audit results do not authorize another frontend or a current
release claim. No executable source, profile, registry identity, proof rule or
frozen conformance member changes in this follow-up.

The existing installed-registry root [passes](registry-result.json): exactly
Go/Rust are registered; Java, C#, Python, TypeScript and an unknown profile
reject. This test was selected because it exercises the current language
dispatch boundary. Its [complete stdout](stdout.log), [stderr](stderr.log),
[prelaunch inputs and command](prelaunch.json), and complete input archive are
retained. Every consumed input was unchanged while the real test ran.
The [documentation review](documentation-review.json) also verifies all nine
links and the complete prior document bytes and modes.
The [preservation review](preservation-review.json) verifies every archived
input byte and mode. The raw test stdout keeps its original final blank line;
all other staged files pass the whitespace check.

Unchanged frontend, source-proof and kernel suites are not repeated for these
documentation edits. This is an intermediate T02-W01 correction; the cumulative
`./scripts/check-fast.sh` gate remains deferred to final T02-W04. The practical
Go profile remains inactive, and native Linux release validation is still due.
