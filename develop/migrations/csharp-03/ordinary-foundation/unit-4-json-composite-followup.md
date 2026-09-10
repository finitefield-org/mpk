# Remaining JSON composite rules within the unchanged W09 scope

These are implementation notes from the existing boundary encoder/decoder and
validate_value_inner/value_limits rules, not a reduction of W09 acceptance.

- Transition contributes one logical cell plus state, each event and response.
  Its events JSON array is an implicit storage role, not another semantic value
  cell. Enforce the existing event-count bound. Parsing a normal array and
  subtracting its cell after a bounded join can wrongly reject a valid total
  exactly at65,536; remove the implicit container contribution before checking
  the parent's cumulative sum. The existing grammar header requires positive
  child counts, so a zero-contribution empty role needs an explicit checked join
  rule rather than a forged zero-cell child packet.
- An ordered map counts one map cell plus each key and value. Its JSON entry
  objects are implicit storage; unlike a standalone ordered_entry, they add no
  entry cell. Enforce exact key/value order, actual key order/uniqueness and
  map capacity. Ordered sets retain their own cell plus each element and need
  their actual element ordering/equality rules.
- Sum JSON tag strings are grammar metadata, not additional semantic value
  cells. They still occupy a JSON value depth. An empty-payload sum encoded as
  an object with a tag string therefore has a child for the depth check; it
  cannot reuse the depth allowance of an empty source object. Preserve exact
  tag/payload presence and the active constructor's complete zero padding.
- Raw boundary JSON value_limits permits at most4*65,536 raw JSON nodes and
  depth32, excluding field-name strings from the node count. The typed value
  domain has its separate65,536 logical-cell limit. Strings contribute1+UTF-16
  units, with at most16,384 units per string. Keep these meanings
  distinct; a raw-node bound must be counted or justified for the full exact
  grammar, not inferred by treating every metadata wrapper as a semantic cell.
- Native arrays, bounded sequences, role bounds such as validation errors,
  and source/semantic child types must retain their actual capacities and
  completely initialized zero inactive storage. Preserve the frozen static
  transformer accounting when composing their finite parse steps.
- The outer boundary object starts its field values at the appropriate JSON
  depth and owns required/optional/null/default rules. Existing fragment parsers
  expose explicit start/ending/depth and do not implement that whole envelope.

The integration sections below track Transition, collection and sum grammar
implementation separately from pending verification. Complete boundary
relations, native and replay relations, actual propositions/proofs,
assembly/mutations and the predecessor audit remain outstanding in the eight
approved internal units.

## Container-count join progress

The ordinary child-header emitter now has a separate ContainerPayload variant
which validates both positive bounded headers, then adds parent cells to child
cells minus one before checking the cumulative bound. It admits an empty
implicit container's zero contribution without admitting a zero-cell child
header. The ordinary Child variant retains its original emitted bytes; source
and Money/ordered-entry products continue using that variant.

The new join is exercised on complete core C7 packets. Its original checkpoint
preceded the Transition/map integrations below; the helper alone does not
establish their shape, order, role capacities, depth or storage construction.
Only those implicit wrapper roles select ContainerPayload. Whole grammar and
W09 acceptance remain open. See unit-4-json-container-join-progress.json.

Source member identifiers are frozen to ASCII and at most 512 characters by
the original capture; probes of longer member names were rejected there.
Arbitrary boundary-sidecar json_name strings still require the existing
16,384 UTF-16 limit in the future complete envelope rules.

## Recursive sequence integration

Exact closed bounded_sequence carriers now have typed JSON parsers. Original
source arrays use their 4,096-element boundary carrier; the separate native
construction storage bound must not be substituted for this value bound.
Arrays of source structs containing arrays exercise supported source recursion
without weakening the frontend's jagged-array exclusion. The implementation
preserves positive child counts and adds each complete child's logical cells.
Map/Transition implicit roles use the separate container-payload join and their
own ordering/role rules in the integrations below; sequence support alone does
not implement those.
See unit-4-json-sequences-progress.json for the capacity and checker status.

## Ordered collection integration

The map parser now selects ContainerPayload specifically for its implicit entry
objects. Set uses the normal complete-child join. Both share the existing
4096-step sequence pipeline and require strict semantic key/element comparison
before each append. The existing Relations compiler emits the reachable key
comparisons in the same core context; no host order or reduced comparison is
substituted. Four original-source key contexts and exact array/sum preservation
passed. Runtime and dual-checker status are kept separate in
unit-4-json-ordered-progress.json. Transition's implicit events container is
integrated in the following section.

## Transition integration

Exact closed Transition products now parse state/events/response with their
typed children and preserve event order and duplicates. Only the events array
uses the container-payload join; source products retain its wrapper cell.
Map and Transition share that helper when both are reachable. Two accepted
original-source contexts and unchanged Map/Set and sum certificate replay
passed. Actual-core runtime and dual-checker verification are tracked in
unit-4-json-transition-progress.json; full boundary relations and W09 acceptance
remain open.

## Remaining boundary envelope contract

The next composition must follow `capture_boundary_input` in
`crates/mpk-vc/src/csharp_practical_boundary_input.rs`, including rules that
cannot be supplied by a typed product parser alone:

- Check the complete input object against boundary field identities and
  declaration order, while allowing only fields whose missing rule admits
  omission. Match sidecar UTF-16 names using the existing exact syntax path.
- Decode a supplied non-null value with the field's payload type and selected
  codec. Apply `check_state` before wrapping it as a presence value or Option.
  JSON null, missing fields and tagged typed sum representations have different
  meanings here. Missing uses the exact frozen default or exposed missing arm;
  null requires an admitted presence/Option rule.
- Sum complete resulting argument cells, including frozen defaults and
  presence/Option wrappers, with the outer object's initial cell. The input's
  raw node/depth limits and the resulting typed value's limits are separate.
  A default omitted from input still contributes typed cells, and its canonical
  typed representation must meet `value_limits` from depth1.
- Retain the exact source reconstruction operation when an argument's
  projected type differs from its source type. Generated predicates must keep
  pending source invariant obligations; parsing a value does not prove an
  application constructor or discharge its invocation.

These are outstanding implementation requirements, not a claim that the
current fragment/product certificates implement complete boundary admission or
its universal propositions/proofs.
