# Core generated recursor iota fixtures

These fixtures enumerate MVP generated recursor iota-reduction verdicts. The
`mpk-core` unit tests load every `*.fixture` file in this directory and require
each listed case to match its expected verdict.

The current IND-004 fixture set covers:

- generated Bool recursor reductions for false and true;
- generated Nat recursor reductions for zero and succ;
- generated Eq recursor reduction for refl;
- rejection of a non-generated recursor equation;
- rejection of an unknown constructor equation for a generated recursor.
