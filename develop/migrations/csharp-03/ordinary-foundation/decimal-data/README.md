# W09 native decimal data relation candidates

Three C# sources connect nine decimal definition occurrences and eleven W03
SSA use points. Native cases cover add, divide, less, Round ToEven with digits,
int32-to-decimal and decimal-to-int32. The exact existing scalar bodies remain
unchanged. Result relations observe the complete 512-bit product representation;
individual exception predicates retain raw conditions, with original W03 guards
applying their frozen priority.

`certificates.json` pins 2,315,545 decoded bytes across three contexts. Source
capture, candidate reconstruction/mutations, scalar-body closure comparison,
513 storage comparisons, a core-reduction sharing regression, unchanged six
integer/floating pins and final formatting pass. The revised relations share
the computed result with an ordinary Let. Final runtime and dual checking are
pending; prior unshared checker outputs are historical evidence, retained in
`../unit-5-decimal-data-progress.json`. These definitions do not prove complete
native bodies or application VCs; original units 3-8 and W09 remain open.
