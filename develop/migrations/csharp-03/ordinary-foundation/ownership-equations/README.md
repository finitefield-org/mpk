# Ordinary symbolic ownership equations

These 14 captured-source programs encode each live allocation slot as a full
32-bit SSA token. Zero means absent; the exact source-specific symbol table
assigns nonzero values. Padding slots are zero. All 7,457 equations are ordinary
Bool definitions, with 7,179 source state records and 40 private-receiver points.

The generator derives allocation, read, write, freeze, local discard, actor,
exceptional cleanup, phi and edge transitions from the original validated VIR.
The opt-in ownership witness supplies candidate state constants. Entry/terminal
conditions, all CFG edges, delayed backedges, and the union of incoming cleanup
origins are checked by explicit equations. Exceptional invocation edges retain
the pre-invocation state. Output comparison includes every token bit and slot;
input/output states reject two origins sharing a nonzero token.

All original-source reconstructions, byte pins, metadata/byte mutations and
19,831 runtime observations pass. This includes changed output states, high
32-bit token bits, empty receiver states, and same-input/output aliasing with
eight simultaneously live origins. Another source allocates two distinct
origins in separate lifetimes. All 14 identical byte sequences pass both
unchanged Rust and Go checkers with zero axioms; hash corruption rejects.

`pending_flow_proofs` intentionally lists every closed flow condition. Ordinary
Bool evaluation is not a checked proof of the application obligation. Private
receiver failure definitions must be linked to those flow proofs before use.
Untraced concrete ownership/borrow/transfer functions are explicitly listed in
`pending_concrete_function_ids`. No empty result establishes such obligations.
The nine existing ConstructionData ownership records and 183 formula roles
remain pending. Unit 5 and W09 are incomplete. See
`../verification-logs/ownership-proofs/current-equation-verification.json` for exact receipts.

Equal live maps share a closed definition. Reduced selector trees preserve each
physical bit; the runtime corpus independently checks every bit of every unique
state definition against the exact allocation/token map.
