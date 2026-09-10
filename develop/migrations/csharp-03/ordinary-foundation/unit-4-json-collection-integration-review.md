# W09 shared collection/JSON integration review (in progress)

Actual full collection emission consumes 8,192 static transformers. The former
JSON emitter consumed 8,744; sharing helper declarations alone was insufficient.
The corrected integration test reached count 8,399 before adding the 8,193-leaf
string scan and rejected with Limit. The initial auxiliary-only test missed the
lazy collection pipeline; it was corrected to emit the depth-14 fold and require
at least 8,192 transformers before JSON emission.

The scan now defines StepFour by composing two existing StepTwo calls, then
composes 4,096 StepFour occurrences and one final StepTwo. Exactly 16,386 primitive
Step calls remain in the same order. Every group body and outer occurrence is
charged by the unchanged Builder counter. No counter reset, dropped operation,
raised limit, or different accepted string capacity is used. The standalone
string program costs 4,312 transformers; the shared lexical program costs 4,650.

The test-only audit expands the actual ordinary Scan/StepTwo/StepFour term trees,
requires every leaf to be Step applied to the unchanged source variable, and
checks all 16,386 occurrences. It separately compares every old declaration,
import, level, binder, initial-state term and complete transitive dependency,
normalizing only the validated Scan composition tree. Guarded composition is
associative: both bracketings first run f; when More is false both return that
state, otherwise both run g and conditionally h. The unchanged Compose and More
bodies are included in the dependency comparison. Negative tests shorten the
step group, change primitive Step, and alter the source cursor.

Shared structural storage can already own C19/C24 generic cube helpers. Both
boundary fragment and string emitters now check the existing complete helper
registration before emitting it. The integration test prepopulates these helper
depths, compares standalone bytes with regenerated pins, emits the full
collection and lexical definitions in one Builder, and serializes that combined
certificate. Its source-free fixture proves definition compatibility only;
complete actual-source foundation assembly and application VC proofs remain open.

Both affected 68-context/three-program source suites passed in 148.04s, including
old sequence/declaration preservation and source/import/metadata mutations.
Each family has one distinct certificate byte vector; all three vectors were
explicitly compared equal before representative checker cases were selected.
Pins were hashed with the certificate domain and read back exactly after staging.

Runtime checks focus on group-boundary whole/prefix results, mixed UTF-8/escape
packets, failure points, and the final partial group at 16,384/16,385 units.
Unchanged scalar numeric/keyword bodies need no repeated semantic matrix.
The new checker fixture warrants the two consumer-edge inventory tests; unrelated
inventory tests and the whole-project gate are not run. Shared integration passed in 4.42s at 12,842 transformers (53,948 terms,
756 declarations), including preexisting C19/C24 helper reuse and exact
standalone pin identity. Final lint passed; both selected consumer-edge tests
passed in 39.42s. All three distinct certificates also passed both unchanged checkers with zero
axioms and hash corruption rejection: standalone string parser (153.38s), shared
lexical program (68.02s), and combined collection/JSON program (132.85s).
Group-boundary and audit-mutation tests passed in 574.86s. The final partial-group
capacity test remains live; final scoped review awaits that result.
Full W09 is incomplete; no component-only commit or push.
