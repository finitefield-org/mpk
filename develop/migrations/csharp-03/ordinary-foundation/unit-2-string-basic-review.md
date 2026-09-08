# W09 internal unit 2: basic UTF-16 operations

This component implements `string.length`, `string.index` and
`string.is_null_or_empty`. It does not complete unit 2 or W09. String comparison and
search remain outstanding; construction is handled in the subsequent increment. Domains, literal bodies
and W09 units 3-8 remain separate outstanding work.

## Implementation review

The string sequence retains all 16,384 UTF-16 units. Its physical carrier is
`Cube.D19`: the first selector chooses length or contents; length has thirteen
leading padding selectors followed by five selectors for its 32 bits; contents
have fourteen element-index selectors followed by four character-bit selectors.
The nullable `option<string>` carrier adds a role selector and uses `Cube.D20`.
Its canonical Some tag has bit zero set, and its active payload is the full string.
No character is decoded to a Unicode scalar or replaced; isolated surrogates and
zero code units remain ordinary 16-bit values.

The length definition selects the canonical length field. Dynamic indexing
applies the source cube directly to fourteen index bits and four output selectors.
The public operation validates the entire signed 32-bit index against length
before exposing that value; the lower fourteen bits therefore cannot silently
wrap an out-of-range index. Null-receiver failure precedes index-range failure.
Failure flags are mutually exclusive and the failed normal result is all zero.
Null-or-empty selects true on a null receiver without demanding its payload.

Production reconstructs each signature from the independently validated closed
instance set. Metadata and certificates are accepted only by exact regeneration
against the source VIR and foundation hash. Other string operations fail closed.
The bit-address helpers assume the input-domain invariants; this component does
not claim to provide those still-pending predicates or application proofs.

## Verification and result

The ordinary core observer matches the independent UTF-16 oracle on 77 cases
covering both physical carriers: null, empty, isolated surrogates, embedded NUL,
maximum length, negative indices, the final valid index, length itself and
32-bit extremes. Every result bit, success and ordered failure is observed.

Nine original captures exercise the source-to-definition path and source,
foundation, operation, check, result-name and byte substitution rejection.
All nine captures close `option<string>`; the non-null carrier is also covered
by the independent core cases. An initial test expectation that these captures
contained both carrier forms was corrected after inspecting the actual closure.
The original mixed Length/concatenation rejection becomes positive in the
construction increment. The remaining rejection case uses original ordinal-search
source and requires the whole generator to fail closed.

The nine pinned source-derived certificates have at most 1,852 terms and
44 declarations. Pinned-fixture replay, both unchanged checkers, inventory, lint
and format passed. All nine checker cases require zero axioms and rejection of
the same hash-corrupted bytes. The final direct review has zero findings.
Decimal arithmetic verification has also passed.
No Certificate v0 format, checker rule, axiom or public production route changes.
The full `check-fast.sh` gate remains deferred to T06-W12.

The consumer inventory gained exactly the new ordinary string implementation
under its existing standard-namespace query. Removing that one path reproduces
the previous 102-path hash; the basic-component set had 103 paths. The subsequent construction component
adds one further path, bringing the combined inventory to 104. The aggregate count
and the cache-correction receipt's inventory-content hash are updated together.
The recorded historical cache paths are unchanged. The inventory rejection tests
must still detect subsequent additions or removals.
