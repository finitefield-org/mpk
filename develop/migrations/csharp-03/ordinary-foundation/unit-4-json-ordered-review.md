# W09 unit 4 ordered collection JSON: implementation review in progress

Map and Set parsing now resolve their exact closed carriers and frozen key/value
arguments. Map storage's inline key/value product must equal its reconstructed
ordered_entry carrier. The compiler reuses the standalone entry parser's exact
key/value syntax and constructor; source Pair's differently named members are
not substituted for these semantic field names. Set elements use their existing
typed parser directly. Unsupported child parsers keep the parent deferred.

Key comparison is emitted by the existing Relations compiler with its shared
finite folds. It receives the same Builder, preserving all prior declarations,
dependencies and static-transformer accounting. Only reachable collection key
roots are requested, with their complete transitive definitions. A key lacking
total comparison rejects generation. Decimal value order, UTF-16 string order,
source product member order, enums and other eligible types keep their existing
comparison implementations. No host comparison defines a proof predicate.

The generic sequence parser has a shared emission body with optional order and
child-join hooks. For ordinary sequences the hooks emit no terms; exact complete
metadata/certificate replay passed for the original recursive array and six-sum
fixtures after extraction. Their previously passed checker/runtime results and
live capacity runs remain valid and are not restarted.

Before every append, an ordered collection reads the previous element at n-1,
projects the map key when applicable, and requires the canonical comparison
result to be negative. An empty prefix accepts its first typed element. Equal
keys/elements and descending input invalidate the complete output packet. This
internal predicate assumes a parser-built prefix; it does not establish an
arbitrary input prefix's representation or order. The public parser always
constructs its prefix from zero and applies the condition at every step.

Map joins subtract the anonymous entry object's cell before adding its complete
key/value payload, using ChildContainerPayload. The entry object still consumes
JSON nesting depth. Set joins count the complete child. The fixed 4096-step
pipeline, closing bracket and original grammar retain capacity, byte/depth/cell
limits and invalid masking. A valid map entry cannot lose a required cell: its
JSON wrapper is absent from MonomorphicValue's map cell count, whereas a
standalone ordered_entry keeps its own cell. Both cases use distinct joins.

Four accepted original-source contexts cover integer, decimal, string and
compound keys. Each reaches Map and Set, one key relation and the required
source/semantic products and sequences. Source reconstruction, carrier/key/join
linkage mutations, all primitive and standalone syntax dependency closures and
actual structure limits passed. Decimal is the largest candidate at180327 terms,
2644 declarations,8743 static transformers; no budget is reset and no definition
is removed. A separate standard Go/Rust same-byte check targets the four new
certificates. Earlier certificate families remain unchanged.

Runtime coverage is explicitly selected wide storage observation. It includes
all logical header and length bits, occupied key/value leaves and nearby padding,
first/last inactive slots, ascending/descending/equal inputs, syntax and absolute
outer endings. Numeric decimal2/10 and compound numeric fields distinguish value
order from lexical JSON order. Integer cases distinguish populated Map depth30
from31 and Set depth31 from32, and map cell counts exclude implicit entries.

The new previous-slot reader also passed26 direct ordinary-core observations on
legal ascending integer storage prefixes of length0,1,2048,2049,4095 (both Map
and Set): a larger next key accepts, an equal or smaller key rejects. These
checks isolate high address bits; they do not claim that complete maximum-size
Map/Set JSON documents have been evaluated. Existing generic sequence capacity
and Validation capacity runs remain separate. Terminal results and pending
runtime/checker jobs are recorded in the progress receipt.

Review is not closed until outstanding runtime/checker results are consumed and
any deterministic failures fixed. Transition JSON, remaining full boundary
relations, source/control/replay relations, propositions/proofs and assembly
acceptance remain outstanding in W09. No component-only commit or push is made;
the full T gate remains deferred to T06-W12.
