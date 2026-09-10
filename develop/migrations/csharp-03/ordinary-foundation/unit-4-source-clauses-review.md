# W09 ordinary source clauses — review in progress

Imported public clauses now lower to ordinary core values through explicit
nominal type and constant maps. Variables must match their precise de Bruijn
binder types,even if two types have the same Boolean-cube storage. Application
checks domain and result annotations;lambda/let preserve nested binder scope.
No unknown constant is introduced as an axiom. Frozen term/binder/certificate
limits remain unchanged.

Current concrete recipes are stored product fields,total Bool/integer scalar
operations,and Bool/fixed integer literals. Source clause generation rejects
partial operations whose definedness conditions are not compiled yet. Every
source public clause must compile before this component returns a program.
This is not the recursive PublicDomain equation,construction preservation proof,
source reconstruction,AcceptInput,or a completed W09 assembly path. Richer
recipes and all remaining proof/application obligations stay open.

The binder test executes nested lets and captured lambdas over all Bool inputs
and rejects same-depth nominal confusion,unbound variables,unknown constants,
wrong result annotations and unknown types. It passes0.01s. Original-source
validation caught an invalid ordinary global name:contract recipe hashes may
start with digits. Source symbol lookup now retains the imported identity but
maps to an identifier-prefixed encoded core name. This fixes canonical naming
without changing the source contract or checker rules. Source/runtime and
current-byte dual-checker validation are still pending. No clean unit review
or W09 completion is claimed.

Original-source tests exposed a test-only storage assumption:initializer cases retain a string before their integer. The test now derives the integer slot from the captured member order and uses the full padded carrier through sparse storage;its expected predicate stays the original positive/nonnegative condition. Consumer closure passes17.54s with unchanged fingerprints;current production lint and binder tests pass. Full seven-source runtime and dual checking are pending.

Final seven-source runtime passes1.99s,including mixed-width initializer
storage and30 signed predicate boundary observations. All metadata replay,
cross-source reuse and byte/metadata corruptions reject. Maximum generated
source size is1328terms/30declarations. The final binder helper also passes and
its exact bytes are retained with seven hash-verified source candidates.
Same-byte dual checking of all8 is running. Final formatting passes;the only
last formatting correction was test line wrapping. No application or source
publication proof is supplied by these clause-definition certificates.

All8 exact source/helper candidates now pass both unchanged checkers with zero
axioms and hash mutations in37.014s;the complete PASS set and current module
hashes match retained candidates. Final affected lint passes2m44s including
queue;format and consumer closure pass. Direct review has no remaining finding
in the currently implemented total Bool/integer/field clause component. Partial
operations,richer recipes,recursive public domains,source reconstruction and
all remaining W09 proof/assembly obligations remain open. This component is
not a completed internal unit and is not committed separately.
