# Forward binding contract review (partial W09)

The reviewed change lowers `source_project` through the existing ordinary
binding projection emitter. Its single `binding_id` parameter, one source
argument, and nominal result type must match exactly one reconstructed
projection. The complete projection set is emitted once per compiler instance.
The cached definitions retain the same validated VIR and ordinary builder;
neither caller-supplied metadata nor a physical carrier coincidence can select a
different nominal conversion.

The connection aliases the existing projection directly. It does not introduce
an inverse, discard unmapped source fields in a reconstruction witness, or claim
source invariant and round-trip obligations have been proved. Existing source
reconstruction work and all remaining original W09 units remain required.

Fifteen original-source contexts exercise real parameter binding in contracts,
covering all twelve original binding families, float/nullable map values, and
nested remapped signed tags with extra source fields. Each contract alias and
its complete transitive declaration closure match the standalone projection
program. Four source samples per alias are compared with the existing independent
projection oracle at every small-output address and selected large-output
addresses, including all selector bits, edges and nonzero neighbours.

The contract's self-equality is not assumed true: the original float-map source
produces a NaN-containing semantic map and its contract value must be false.
This negative case and 4,354 observations passed, together with exact import
and result-metadata corruption. The existing binder/nominal-type regression,
targeted inventory, affected library/test clippy and format checks also passed.
An initial cfg(test) initializer omitted the new empty cache field; it was
corrected before the final library test and lint runs.

Code review found no remaining actionable issue in this connection. All fifteen
identical-byte candidates passed both unchanged checkers with zero axioms and
matching reports; all actual hash corruptions were rejected (265.646 seconds).
Terminal checker names, current candidate hashes/sizes, metadata and tested
output copies were reconciled. Ordinary helper definitions and runtime observations do
not prove an application VC. W09 and original units 3-8 remain open, and the
whole gate stays deferred to T06-W12.
