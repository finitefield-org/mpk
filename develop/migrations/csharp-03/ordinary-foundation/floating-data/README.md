# W09 native floating data relation candidates

Three original C# sources connect eight definition occurrences and ten W03 SSA
use points to existing floating ordinary definitions. The seven distinct
operations are single add/negate/less, double less, int32-to-single,
int64-to-double and checked single-to-int32. The adapter emits the original
success guard, relation, goal and checked-conversion failure predicates with
unchanged subject and successor ordering. Other data families remain pending.

`certificates.json` pins 765,700 decoded bytes across three contexts. Source
capture, candidate reconstruction, complete scalar-body closure comparison,
mutation rejection, lint, inventory and scoped formatting have passed. All 84
result/guard observations pass in the optimized build, and all three same-byte
certificates pass both unchanged checkers with zero axioms and matching reports.
Runtime outputs match every candidate certificate/metadata byte. Results are in
`../unit-5-floating-data-progress.json`. Definitions are not native-body proofs;
W09 and original units 3-8 remain incomplete.
