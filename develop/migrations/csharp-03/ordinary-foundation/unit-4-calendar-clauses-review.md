# W09 calendar/temporal contract component — direct review

This review covers the new emitter entry points, contract dispatch, source
cases and checker harness entry. W09 and original units 3–8 remain open.

## Findings addressed

- The initial emitter visibility did not permit the scoped parent reexport.
  Matched the existing scalar emitters' `pub(in super::super)` boundary; no
  public API was added.
- The initial integration output helper referenced a nonexistent function.
  Reused the existing component pattern for exact pinned-file reads.
- The first capture lacked the capability required for the root user-ID map.
  Retried with SETFCAP while preserving read-only mounts/root, network isolation,
  default seccomp and bounded resources. All eight native captures accepted.
- Capture output and test output used different JSON whitespace. Preserved raw
  transports and proved parsed equality before pinning the test serialization;
  certificate and metadata bytes already matched exactly across both runs.

## Reviewed behavior

The adapter delegates to the existing Calendar/Temporal circuits and keeps exact
nominal argument/result types and ordered checks. All 70 standalone certificate
hashes and definitions are preserved. The supported 65 adapter signatures each
reject wrong arguments, results and checks (195 mutations). Their exception
checks are single predicates, so the existing ordered failure is also the raw
failure needed for contract definedness. The three Instant ErrorOutcome entries
are explicitly rejected and do not publish a normal-result alias.

Fresh native inputs cover 50 operations and 54 attachments across eight contexts.
Every operation alias and transitive definition is compared structurally with
its standalone counterpart; changed dependency bodies and metadata reject.
Fourteen selected core evaluations passed, including all four deliberately
undefined cases. Tests do not substitute native execution or a host result for
an ordinary definition or application theorem.

Final scoped review has no findings. All eight same-byte checker cases passed
with zero axioms and hash-corruption rejection (1,002.221 seconds). Regeneration
after the shared compiler's string-cache addition preserved all eight current
certificate/metadata pins (45.61 seconds). The terminal case set, manifest,
capture transports and runtime outputs have been reconciled. This completes
this component's targeted verification, not an original unit or W09. See the
progress JSON for commands, selection reasons, hashes and remaining scope.
