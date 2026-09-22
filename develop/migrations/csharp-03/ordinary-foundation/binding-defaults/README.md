# Original closed binding-default conditions

The 45 retained binding source contexts regenerate actual CLR-default candidates,
public domains, projections and semantic-arm predicates as ordinary definitions.
Original W06 actual_default sequents are retained verbatim. Representable eligible
default conditions are closed Bool definitions with both source public membership
and the declared semantic arm as goals. They do not depend on a declaration-level
public-admission flag. No proof obligation is discharged.

Ineligible default declarations retain their original DefaultUseForbidden
sequent as a source-use proof requirement. A declared eligible default with no
structural CLR candidate also remains unresolved. All other condition IDs and
all original proof IDs remain explicit.

The certificate/import test preserves full transitive implementations from the
independent default, guard/projection and public-domain components. It rejects
foreign contexts, every metadata field mutation, corrupted certificate bytes
and either oversized input. The runtime test observes every available original
source default and each composed condition, with independent public-domain
results. In-memory one-bit mutations of small default constants separately
exercise an invalid source with a matching arm and a valid source with a wrong
arm. Those mutated ASTs are test inputs, not accepted source certificates.

Targeted verification passed under
`../verification-logs/binding-defaults/`. Both unchanged local checkers accept all 45 identical canonical byte sets with
zero axioms and reject changed hashes. Three closed conditions are defined; 32
default use restrictions and all 987 original proofs remain pending.
Unit 4 and W09 remain in progress; the T06 gate is deferred to T06-W12.
