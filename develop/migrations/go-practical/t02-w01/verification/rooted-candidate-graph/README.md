# Root-held candidate and module graph

The private Rust source consumer can now retain a validated candidate request,
its complete original source-selection owner and an independently imported
module graph together. It compares the full source-selection values, including
their original data, rather than accepting a caller-supplied digest. This
source-level type alone grants no root or launcher authority.

The owners that already retain both matched source/export root sets can bind
that request and graph only when the graph's complete export owner equals their
own. The installed-compiler owner has the same binding method. The returned
binding borrows all inputs, so those owners remain live while it is used.

The new Rust test imported two complete original graphs with 5,697 and 5,667
source/export objects. Both have a valid `example.com/policy.Root` selection;
each binds to its own graph, and crossing the owners rejects. The existing five
request/source tests also passed, including all 8,192 original boundary roots.
The affected Linux capture-root integration test passed all five tests after
restoring the full 16,880 evidence files and 5,577 registered SDK files. These
tests were selected for the new source/graph ownership comparison and its
held-root consumer.

The full live Go capture and Rust test observer passed together. The original
182-package capture retained 5,516 source objects, 181 export objects and the
entire 5,577-file SDK. The observer bound its request and reconstructed graph
while both roots were held, then independently imported the other complete
5,667-object graph. Its same-named function resolved successfully, but the
root-held binding rejected the different export owner. The test-only SDK
owner also bound the correct graph. Go ran as Linux/amd64 under local emulation;
Rust ran on Linux arm64. The observer is not the installed launcher.

Strict Clippy passed for the affected Rust tests and Linux observers, including
the observer's test configuration. Changed-file Rust formatting and
`git diff --check` passed. Go source was unchanged; no unrelated Go suite was
repeated. No new validation archive is committed.

The candidate foundation, launcher/read authority, native Linux release
acceptance and source/program proof remain open. T02-W01 and R04/R06 are not
complete, and the practical profile remains inactive. The cumulative
`./scripts/check-fast.sh` gate is deferred to final T02-W04.
