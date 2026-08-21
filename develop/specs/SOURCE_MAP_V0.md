# VIR Source Map v0 Specification

Status: normative and frozen for implementation.

Schema "mpk.source_map.v0" is an untrusted auxiliary mapping from public VIR
nodes to captured source bytes. It is used for traceability and diagnostics,
not for proof acceptance. A consumer validates it before exposing any span.

## 1. Conformance language and validation order

The terms MUST, MUST NOT, REQUIRED, and REJECT are normative. REJECT means that
no source map or downstream artifact derived from it is returned.

Every object and tagged-union branch is closed. Unknown or inapplicable fields,
missing fields, duplicate JSON names, and null reject. Strings and enum values
are case-sensitive. Floating-point numbers and integers outside
[-9007199254740991, 9007199254740991] reject. A Sha256 is exactly 64 lowercase
hexadecimal characters.

Validation uses this first-error phase order:

1. "transport": enclosing byte and entry-count limits, UTF-8, JSON nesting,
   per-string limit, duplicate-name detection, and JSON number syntax;
2. "shape": closed root, entry, reference, and origin shapes plus schema;
3. "scalar": identifier, normalized path, range, and synthetic-reason grammar;
4. "order": reference order and uniqueness;
5. "linkage": source IR identity, known VIR references, and manifest input
   kind/path/size linkage;
6. "coverage": exact profile mapping and synthetic-node rules;
7. "utf8": source-range scalar boundaries;
8. "canonical_size": complete-root JCS byte limit;
9. "hash": "source_map_hash" recomputation.

An implementation may collect more than one finding within one phase but MUST
NOT report a later-phase code as primary.

## 2. Root object and hash

The root has exactly:

| Field | Type and rule |
|---|---|
| "schema" | exactly "mpk.source_map.v0" |
| "source_ir_schema" | exactly "mpk.vir.v0" |
| "source_ir_hash" | equals the validated VIR "vir_hash" |
| "entries" | canonical array of SourceMapEntry |
| "source_map_hash" | self-hash below |

"source_map_hash" is:

~~~text
SHA256(
  UTF8("MPK-SOURCE-MAP-0.1") || 0x00 ||
  JCS(SourceMap with only source_map_hash removed)
)
~~~

The domain is exactly the 18 ASCII bytes "MPK-SOURCE-MAP-0.1". Only the root
"source_map_hash" member is removed. The compact JCS bytes contain no LF.
Object source-key order is irrelevant; array order is semantic.

The hash can be recomputed from the map bytes and does not require original
source bytes. Semantic range validation still uses the immutable captured bytes
at the frontend boundary.

## 3. Closed VIR reference union

Every SourceMapEntry has exactly "reference" and "origin".

"reference" is one of these exact tagged shapes.

FunctionRef:

~~~json
{"kind":"function","unit_id":"example.com/mpk/vector","function_id":"example.com/mpk/vector.Identity"}
~~~

InstructionRef:

~~~json
{"kind":"instruction","unit_id":"example.com/mpk/vector","function_id":"example.com/mpk/vector.Identity","block":"bb0","instruction":"t0"}
~~~

TerminatorRef:

~~~json
{"kind":"terminator","unit_id":"example.com/mpk/vector","function_id":"example.com/mpk/vector.Identity","block":"bb0"}
~~~

"unit_id" and "function_id" obey VIR_V0.md and resolve to the named containing
unit and function. "block" and "instruction" are the canonical dense IDs from
that VIR function. A function reference names the function declaration; an
instruction reference names exactly one instruction; a terminator reference
names exactly one block terminator. Type declarations, constants, contracts,
safety-check records, values, block parameters, and blocks themselves have no
v0 reference tag.

Unknown references, a function in the wrong unit, a block outside the function,
an instruction outside the block, and an unknown reference kind reject.

## 4. Closed origin union

"origin" is selected by "kind".

SourceOrigin has exactly:

~~~json
{"kind":"source","input_kind":"source","normalized_path":"identity.go","start":16,"end":24}
~~~

"input_kind" is literally "source". The path must resolve exactly one
SOURCE_MANIFEST_V0.md input whose kind is "source". "start" and "end" are
zero-based half-open UTF-8 byte offsets and satisfy:

~~~text
0 <= start < end <= input.size_bytes
~~~

Both offsets lie on Unicode scalar boundaries in the immutable captured input.
No empty range is accepted. The map stores no raw source, line, column, source
snippet, file URI, compiler span, macro expansion, or absolute path.

SyntheticOrigin has exactly:

~~~json
{"kind":"synthetic","reason":"profile.control_flow_join"}
~~~

"reason" is a ProfileSyntheticId: 1 through 128 ASCII bytes matching
[a-z0-9]+([._-][a-z0-9]+)*, the same grammar as a release ProfileId. The
selected language profile owns a closed allowlist of reason IDs and a predicate
that maps each reason to permitted VIR reference shapes. An implementation
cannot mint a reason from compiler prose.

Function references MUST use SourceOrigin. An instruction or terminator uses
SyntheticOrigin only when the language profile proves that the exact VIR node
has no faithful source span and permits that reason for that node. A synthetic
node MUST NOT also receive a fabricated or nearest-neighbor source span.

## 5. Total mapping and profile policy

The map contains exactly one entry for:

- every VirFunction;
- every instruction in every reachable VirBlock;
- every reachable block terminator.

There are no entries for any other object. Thus the expected entry count is:

~~~text
function_count + instruction_count + reachable_block_count
~~~

Each reference occurs exactly once. Multiple references may share the same
SourceOrigin range; short-circuit lowering and one source expression producing
multiple VIR nodes are not duplicate references. A language profile MUST state
which of its emitted instructions and terminators are source-derived and which
exact synthetic reason, if any, is allowed. It MUST NOT omit a reference merely
because the compiler did not preserve a convenient span.

For Go v0 and Rust v0, selected function declarations are source-derived.
Compiler builtin files and expansion spans are not captured source. A profile
may model an allowed compiler-created control-flow node only through an
explicit SyntheticOrigin. A node caused by a macro, external file, unknown
compiler transformation, or disallowed source construct rejects before public
map emission rather than becoming synthetic.

This total rule is checked using only validated VIR, the selected language
profile, and map entries. Source text is needed only for scalar-boundary
validation.

## 6. Canonical reference order

Entries are strictly increasing by this tuple:

1. "unit_id" UTF-8 bytes;
2. "function_id" UTF-8 bytes;
3. reference rank: function = 0, instruction = 1, terminator = 2;
4. numeric block index, with function using -1;
5. numeric instruction index, with non-instruction references using -1.

"bbN" and "tN" have the dense canonical decimal spelling required by
VIR_V0.md, so their indices are parsed without ambiguity. Within a function the
function declaration therefore comes first, then all instructions in block
index/instruction index order, then all terminators in block order only if the
rank comparison reaches them. This ordering is independent of source path or
span. Equal keys are duplicate references and reject.

## 7. Manifest linkage

Map validation receives a validated frontend-stage source manifest and
validated VIR:

- root "source_ir_schema" equals VIR "schema";
- root "source_ir_hash" equals recomputed VIR "vir_hash";
- every SourceOrigin path resolves one unique manifest input;
- the input kind and entry "input_kind" are both "source";
- its recorded "size_bytes" equals the immutable captured byte count;
- the source bytes hash to the manifest input "sha256";
- the manifest "source_map_hash" equals the recomputed map hash.

A path naming a contract, build manifest, or lockfile rejects even if its byte
range is otherwise valid. A captured source omitted from the map is permitted
when it contributes no selected VIR node; the source inventory is not itself a
mapping-coverage list.

## 8. Portable paths and path exclusion

"normalized_path" uses exactly the portable grammar frozen by
SOURCE_MANIFEST_V0.md. It is source-root-relative, slash-separated, already
normalized, and no longer than 1,024 UTF-8 bytes. A POSIX-rooted path,
drive-letter path, UNC path, file URI, parent component, empty component,
backslash, NUL, private "/mpk/" locator, source root, toolchain path, or
temporary locator rejects.

## 9. Limits

The intrinsic map limit profile is "mpk.source_map.limits.v0".

| Limit ID | Inclusive maximum |
|---|---:|
| "canonical_json_bytes" | 33,554,432 (32 MiB) |
| "json_nesting" | 256 levels |
| "string_bytes" | 1,048,576 per decoded string |
| "normalized_path_bytes" | 1,024 |
| "entries" | 323,728 |

The entry ceiling is the checked sum of the VIR v0 maxima: 8,192 functions,
250,000 instructions, and 65,536 reachable blocks. A selected language profile
may impose a smaller count through its registered limit profile. Counters are
checked before allocating the complete collection. "canonical_json_bytes"
counts JCS of the complete root including "source_map_hash"; the map is nested
inside the separately bounded frontend transport and has no independent LF.

At the exact canonical byte ceiling the map is accepted. One byte above
rejects with "SOURCE_MAP_LIMIT_CANONICAL_BYTES" before hash acceptance.

## 10. Stable shared codes

The shared validator owns:

| Code | Meaning |
|---|---|
| "SOURCE_MAP_JSON_DUPLICATE_KEY" | duplicate object name |
| "SOURCE_MAP_JSON_INVALID" | invalid UTF-8/JSON/number |
| "SOURCE_MAP_SCHEMA" | wrong schema discriminator |
| "SOURCE_MAP_SHAPE" | missing, unknown, or inapplicable field/tag |
| "SOURCE_MAP_PATH" | nonportable or forbidden path |
| "SOURCE_MAP_RANGE" | empty, negative, overflowed, or out-of-bounds range |
| "SOURCE_MAP_ORDER" | noncanonical order or duplicate reference |
| "SOURCE_MAP_REFERENCE" | reference does not resolve in VIR |
| "SOURCE_MAP_INPUT_KIND" | path/kind does not resolve a source input |
| "SOURCE_MAP_TOTAL" | missing or extra VIR reference |
| "SOURCE_MAP_SYNTHETIC" | unregistered or inapplicable synthetic reason |
| "SOURCE_MAP_UTF8_BOUNDARY" | offset splits a Unicode scalar |
| "SOURCE_MAP_IR_IDENTITY" | repeated VIR schema/hash mismatch |
| "SOURCE_MAP_LIMIT_ENTRIES" | entry count exceeds limit |
| "SOURCE_MAP_LIMIT_CANONICAL_BYTES" | complete JCS exceeds 32 MiB |
| "SOURCE_MAP_HASH" | self-hash mismatch |

Language-specific source-span failures detected before public construction use
that frontend's code family. The shared importer uses the codes above.

## 11. Conformance vectors and ownership

"develop/specs/vectors/source-map-v0.json" has schema
"mpk.source_map.conformance.v0". Its exact top-level fields are "schema",
"spec_schema", "dependencies", "owner_tests", "fixture_sources",
"map_cases", "reference_cases", "mapping_cases", "path_cases",
"hash_cases", and "limit_cases".

"fixture_sources" supplies exact base64 source bytes keyed by normalized path.
The valid map case names one VIR module case and exact "fixture_sources"
records that model the source-kind manifest entries; this avoids a cyclic
vector dependency. A case contains exactly one of "input", "json_text", or
"construction". "input" and "json_text" are passed directly to the root
validator. A construction names either an earlier "base" plus optional
"context" and ordered RFC 6901/RFC 6902 add-remove-replace "patches", or one
of the exact model fixtures below. Patches are never implicitly hash-repaired.

"valid_synthetic_instruction" creates a one-function, one-block model VIR with
one synthetic instruction allowed only by reason
"profile.control_flow_join", maps all three required references, restores
canonical order, and recomputes both VIR and map hashes. "canonical_size"
feeds exactly "count" bytes from an already canonical, semantically valid model
root to the isolated complete-root size counter. "entry_count" feeds exactly
"count" accepted entries to the isolated checked entry counter. These cases
carry "context.validator" naming the counter and test inclusive comparisons
without claiming that independent entry and canonical-byte maxima can coexist
in one map.

"normalized_path" creates the shortest valid component sequence of exactly
"count" bytes, starts from "map.valid_go_identity" and
"source.identity_go", and applies the path to every linked fixture/input before
recomputing the map hash. Case "context.source_cases" is the complete
source-input set;
"context.additional_manifest_inputs" adds the exact non-source entries used
only to test input-kind linkage and does not make them eligible source origins.
"expect" has "outcome" and, for rejection,
exact "phase" and "code"; accepted hash cases may additionally freeze lengths
and digests.

The required root "owner_tests" array is exactly, in order:

1. "crates/mpk-vc/tests/source_map.rs";
2. "go-tools/go2vir/corpus_test.go"; and
3. "rust-tools/rust2vir/tests/frontend_envelope.rs".

The owning tests MUST load every case, reject unknown vector/case fields,
verify unique IDs across all arrays, and prove no case is skipped.
