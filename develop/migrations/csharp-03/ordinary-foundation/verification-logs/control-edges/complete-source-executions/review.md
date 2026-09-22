# T06-W09 internal unit 5: complete reachable source execution checkpoint

This checkpoint extends the preserved 625-path normal-only source execution
record to every reachable source path in the current 18-context corpus.
Function entry, normal, exceptional, alias-update, constructor-call and terminal
steps now have exact execution relations. The resulting program contains 761
source executions over 703 source nodes.

The local-step layer contains 582 complete predicates. Its remaining 121 nodes
are classified as 18 function entries, 93 exceptional steps and ten
structurally unreachable nodes. Function entries and exceptions are composed by
their dedicated execution relations. The ten unreachable IDs are independently
reconstructed and stored in `excluded_unreachable_source_node_ids`; no reachable
execution ID remains pending.

State bridges retain exact assignedness and compare payload storage whenever a
slot is assigned. The execution component list fixes entry, local operation or
special step, edge, phi, handler and target selection as applicable. Alias
updates omit only the slot rewritten by the already checked memory effect and
frame every other slot. The constructor source step binds the retained non-data
call to its exact normal result. Its native invocation node is serialized in
`source_constructor_invocation_node_ids`; the pending native-invocation set is
empty.

Canonical metadata commits to the complete reconstructed argument/component
map with its exact count and SHA-256. Import regenerates the whole program from
validated VIR and requires byte-identical metadata and certificate bytes. Tests
remove real execution and exclusion collections and forge pending/exclusion
members; every mutation rejects. Source-step replay and preservation checks
also pass, including the two formerly pending alias frames.

Two independent 36-file metadata generations match each other and the promoted
fixture byte for byte. All 15 distinct certificate byte sets pass the unchanged
Go and Rust checkers. Their accepted module, declaration, axiom and report
hashes agree, axiom count is zero, and both checkers reject every one-bit hash
corruption. All five inventory tests, targeted Clippy and format checks pass.

This checkpoint closes source execution for the current 18-source corpus. It
does not establish the remaining complete native-body integration or assemble
the application theorem types and proof terms. Unit 5 and W09 remain
incomplete, W10 remains blocked, and the full T06 gate stays deferred to W12.
