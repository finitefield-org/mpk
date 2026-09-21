# Source-array slot projection review

This checkpoint is part of W09 unit 5, not an application proof or completion
receipt. W09 units 3–8 remain open; the whole gate is deferred to T06-W12.

## Semantics

Source arrays retain the source allocation profile bound of 4096; the shared
private construction capacity of 16384 does not widen that bound. There are two separate read-only relations. The source-slot storage snapshot
checks only the 4096 bound and copies exact length/physical cells and zero public
padding. It admits partial initialization: source identifiers may be used for
first writes and initialized reads. The public-value projection additionally
requires every active initialization bit. Neither relation freezes the allocation
or consumes its ownership token. Per-element read/fill/rewrite initialization,
source element domains and actual publication proofs remain separate.

Each successful source load/store uses the latest memory SSA identity reconstructed
from the checked ownership flow. Native lowering temporaries retain native storage
equality. Source slots whose carrier is the public sequence layout use the bounded raw
storage snapshot, without claiming membership in the public value domain. All
other slot assignedness/frame rules and strict type matching remain in force.
Non-transfer memory effects, node-entry merging, nullable source slot projections,
exception slots and full native execution/proof assembly remain incomplete.

## Performance correction

The initial projection reused the counted aggregate Complete predicate. The
124-observation source-array test exceeded the 300-second evaluator deadline.
The replacement checks the low-bit-first initialization bitmap recursively.
At offset k, the active count is (length >> k) + carry. Even and odd children
receive (bit[k] OR carry) and (bit[k] AND carry). Empty subarrays impose no
condition; selected leaves must be true. Full capacity is represented explicitly
and larger counts reject. This is an ordinary Boolean definition, not a host
shortcut in the evaluator. Existing Complete/freeze operation definitions are
unchanged. Exhaustive small-bitmaps and full-capacity boundaries passed.

The initial 124-observation matrix passed in 3.69 seconds after this
change. Review against ledger section 29.1 then found the original use of
publication completeness on every source identifier was too strong. The
corrected 144-observation matrix admits incomplete internal snapshots while
rejecting their public-value projection, and rejects out-of-bound storage. The six existing source-slot programs regenerate byte-for-byte, including
metadata. All three corrected candidates now pass both unchanged checkers, zero-axiom
report agreement and hash rejection; pins are promoted. See verification.json.
Earlier positive results concern superseded candidates and are not current evidence.

## Count/fill coverage

The original count/fill source revealed duplicate C5 helper emission when a
private construction and a published sequence share a builder. The combined
foundation emitter now reuses an existing helper family, as its ordered-fold
emitter already does. Source replay generation/import and positive/negative memory checks passed;
the final 16-observation test includes partial-initialization cases for native
temporaries too. All source-slot checks total 1642 observations at this checkpoint. The regression
case is retained; failure cannot be hidden by narrowing the source corpus.
