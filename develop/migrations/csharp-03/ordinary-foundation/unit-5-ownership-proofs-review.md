# Unit 5 ownership proof component review — validation in progress

This component produces ordinary equality proofs for the independently
reconstructed symbolic flow equations and receiver-state checks. It does not
complete native execution, unit 5, or W09. W03 linkage at eighteen symbolic
receiver use points is separately verified.

- The equality foundation is copied only after its frozen SHA-256 matches.
  Definitions and generated proof terms introduce no axioms or special proof
  nodes. Certificate limits and both checker implementations are unchanged.
- All token comparisons retain 32 bits, including unused high bits. Named
  comparison prefixes (up to two origins), balanced comparison ranges (more
  than two origins), and balanced flow conjunctions preserve every original
  comparison and equation. The equation corpus has separate current evidence.
- Normalization records computed Bool conditions with ordinary congruence and
  transitivity. A computed recursor major is not silently replaced by a literal
  where the kernel would not perform that conversion. Closed literal word
  substitution is transported through checked equality proofs.
- Multiple-slot cases share two ordinary BoolStep theorems. Each is universally
  quantified over both branches, the condition, and the result, with explicit
  equality premises for the condition and selected branch. The de Bruijn
  contexts of both premises, the inner lambda, and the final conclusion were
  reviewed; the small two-origin candidate passes both unchanged checkers.
  Source equations and final theorem conclusions are unchanged by this sharing.
  Wider states also use a universal WordTransport theorem with equality premises
  for both words and the compared result. Balanced ranges still cover all 32
  positions exactly once.
- Wider closed word conditions are transported once across the complete C5
  function by explicit congruence and transitivity. Selector-dependent conditions
  decline this optimization. Beta substitution shifts free variables under
  binders; a dedicated regression checks capture avoidance. All thirteen other
  proof pins, three semantic mutants and nine construction/record certificates
  regenerate to unchanged bytes. The changed eight-live candidate remains
  unpromoted until both checker stages and corruption rejection pass.
- Exact regeneration binds source/foundation identity, every function/point,
  proof names, pending application status, and certificate bytes. The tests
  reject removal of proof metadata and a false application-complete flag.
  All thirteen other contexts regenerate to their previously pinned bytes.
- Valid-hash semantic mutations must produce a type mismatch in both checkers.
  A fuel limit, process failure, or test timeout cannot satisfy those rejection
  tests. All three existing mutants meet that stricter condition.
- Closed receiver results require the corresponding source flow proof. Their
  existence cannot justify defining a generic ownership-failure predicate as
  false for arbitrary inputs. Eighteen W03 uses now have exact function, node,
  receiver and state bindings with checked flow dependencies; that six-certificate
  checkpoint passed both checkers. The subsequent symbolic-record extension
  covers seventeen records across nine data/control contexts; all nine current
  certificates pass both unchanged checkers and hash-corruption rejection. Concrete
  borrow/transfer and application scopes remain explicit outstanding work.

`verification-logs/ownership-proofs/current-proof-verification.json` records
the exact current bytes and per-case acceptance status. The complete current
proof corpus, including eight simultaneous live origins, must finish checking
before this component receives a completed verification record. Historical
accepted candidates and failed optimization attempts are retained separately;
acceptance is never transferred to changed bytes. No component-only commit or
downstream unblock is made. The full repository gate remains T06-W12.

The word-wide mux candidate completed all four stages on identical canonical
bytes: Go/Rust acceptance and hash rejection, zero axioms and report agreement.
All fourteen current proof candidates are now checked; the thirteen other pins
and three semantic mutants remain byte-identical. The new eight-live pin is
470,792 bytes and 2,621 declarations. Before-promotion bytes/diagnostics are
archived, and final receipts are in word-mux-eight-complete. Final regeneration
also reconciles stale producer/test source hashes in the earlier aggregate
receipt (see word-mux-promotion-source-audit.json). This completes the scoped
symbolic proof corpus, not native execution or application assembly.
