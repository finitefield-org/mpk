# Current test execution policy (2026-09-20)

The user now delegates **all tests, including long tests, to the agent**.
This supersedes the earlier one-minute handoff policy below. Select affected
checks, retain actual timing and results, and defer the whole gate to T06-W12.
Earlier handoff sections are historical evidence, not pending user actions.

All fourteen symbolic ownership proof candidates now pass both checkers and hash rejection; no ownership corpus check remains running.
All nine construction ownership record certificates, six control-slot
certificates and seventeen control-edge certificates (including 62 native phi joins) have passed both checkers. Exact statuses are recorded in
`verification-logs/ownership-proofs/current-proof-verification.json`,
`verification-logs/construction-data/ownership-records/verification.json` and
`verification-logs/control-slots/verification.json` and
`verification-logs/control-edges/verification.json`.
There are no pending test commands for the user. Timeouts remain diagnostics;
completed same-byte stages may be preserved when the remaining stages are run
with separate deadlines.

# W09 local testing handoff

Historical execution policy (superseded above): the agent ran targeted tests expected to finish in
under one minute; the user runs longer tests. This supersedes the earlier
all-tests-user-owned handoff. Agent test processes use a wall timeout below one minute (normally 55 seconds;
59 seconds for the final corrected interpolation) and a shorter evaluator
deadline; compilation can be prepared separately.
Historical cancelled runs and user results below retain their original status.
W09 is incomplete; the full repository gate remains deferred to T06-W12.

## Completed nullable-reference normal runtime

Both distinct normal partitions passed: guarded/some in 41.17 seconds wall,
conditional/some in 39.13 seconds wall. All future affected runs are agent-owned, regardless of duration. The earlier bounded
timeout is retained as diagnostic history; no source change or speedup is claimed.

With the previous null cases, 30 observations were executed. Exact certificate,
definition, SSA and semantic payload-type equivalence was revalidated before
reusing 18 additional context observations (48 across four source contexts).
Original source-context identity checks remain independent. Both unique
certificates covering all four artifacts also pass Rust/Go verification.

Original logs, manifest, result receipts and import verification are preserved
in `verification-logs/reference-data/user-long-completed/`. All current source,
checker and fixture hashes matched. No tests were rerun during import. Pending
lists are empty; do not rerun this completed handoff. Published-sequence SSA
checks also remain complete: seven contexts, 411 observations and seven
certificate pairs. Unit 5 and W09 remain incomplete.

## Completed option-data runtime

The remaining nested-string GetValueOrDefault (`value_or`) partition passed in
235.32 test seconds (235.39 seconds wall). The option adapter now has all twelve
runtime contexts and 177 result observations passing, alongside twelve
same-byte Rust/Go certificate checks. It covers sixteen definitions and
twenty-two original SSA points.

Logs, the original handoff manifest and result receipts are retained in
`verification-logs/option-data/user-long-completed/`. All current source,
checker and input/certificate hashes matched. No test was rerun to import this
result. The pending lists are empty; do not rerun the completed handoff.
Future affected runs of this partition are agent-owned under the current policy. Unit 5 and W09 remain incomplete.

## Completed dependent-domain checks

All five user checks passed. Together with the fifteen agent checker passes,
all eighteen changed public/boundary certificates now pass same-byte Rust/Go
checking, zero-axiom/report agreement and corrupted-hash rejection.
The full 16,384-element aggregate passed in 42.55 seconds (42.65 seconds wall),
and all eight decimal map/set order cases passed in 92.64 seconds (92.72 seconds
wall). The total-cell 65,536/65,537 boundary already passed in 42.82 seconds.

Logs, the original manifest and result receipts are retained in
`verification-logs/domain-dependent-refresh/user-long-completed/`. All current
certificate hashes, 86 checker source/dependency hashes and 80 runtime source
hashes match the handoff. No tests were rerun to import these results.
Both pending lists are now empty; do not rerun the completed handoff.

Future affected aggregate, checker and decimal-domain tests are all agent-owned. Earlier 45/55-second aggregate cutoffs are retained as
historical attempts and do not override the completed result. No source change
or measured speed improvement is claimed for this import. Other dependent
clause/data fixtures and full unit-3/W09 obligations remain outstanding.

## Completed domain/aggregate certificate checks

All 11 user-owned checks passed; with the 92 earlier agent checks, every one of
103 changed certificates now passes same-byte Rust/Go verification, zero-axiom
and report agreement checks, and hash-corruption rejection. Imported logs,
manifest and receipts are retained in
`verification-logs/domain-pin-refresh/user-long-completed/`. All current pin
hashes and 86 checker source/dependency hashes match the handoff.

No tests were rerun to import these results. The pending list is empty; the
completed 11-case manifest is preserved in `user-long-completed/manifest.json`.
Do not rerun these checks without a relevant change. All future affected runs of these checks are agent-owned, including the ten
checks whose measured durations exceeded one minute.
Other downstream consumers and whole-unit evidence reconciliation remain open.
W09 is incomplete and no partial internal-unit commit has been made.

## Completed string checks

All 25 remaining runtime partitions passed, adding 184 result/guard observations.
The combined scoped corpus now covers all 48 signatures and 320 observations.
The complete 44-case constructor oracle passed in 30.63 seconds and the final
binder fixture's same-byte Rust/Go agreement check passed in 21.53 seconds.
All fourteen changed certificates and all eight data contexts are checked.

The 27 imported logs, manifest, source/hash checks, per-operation times and
future execution ownership are recorded in
`verification-logs/string-data/user-long-completed/verification.json`.
No test was rerun to import these results. The completed handoff script and
its formerly pending manifest remain reproducibility artifacts, not a current
request to run tests. All tests are agent-owned when next affected by a change. Other W09 work remains open.

## Completed user long checks

All eleven pending runtime partitions and four pending Rust/Go checker contexts
passed. Logs were copied from `/tmp/mpk-w09-lifted-long` to
`verification-logs/lifted-data/user-long-completed/`; `verification.json` records
hashes, all operand-pair sets, observation counts and source consistency checks.
No tests were rerun to import these results.

Checked i32 multiplication now completes all 36 pairs (72 correct/wrong-result
observations) in 163.15 seconds, following the previous 600.29-second timeout.
The i64 divide test passes in 516.56 seconds, and all six decimal comparisons
pass in 399.92–495.10 seconds. All four remaining checker contexts pass in
505.21 seconds total, including acceptance of the same bytes with zero axioms
and rejection of corrupted hashes. No full-run memory measurement was recorded.

Together with the earlier short-run evidence, the scoped corpus covers 46
signatures and 2,550 result/guard observations; all seven certificate contexts
are checked. The earlier 35 signatures were not all rerun after the observer
change. These results close this component's pending test list, not the whole
unit 5 or W09. Do not rerun passed partitions without a relevant change.

## Current verified results

The reported total-cell timeout is fixed in the targeted runtime check. The
user's previous run stopped before the 65,536-cell result at 606.15 seconds
(600-second evaluator ceiling). The agent ran the original 3,855-element test
unchanged: both 65,536 accepted and 65,537 rejected, in 38.19 seconds under a
45-second evaluator ceiling and 55-second process timeout. Peak physical
footprint was 564,020,040 bytes (537.89 MiB); maximum RSS was 583,548,928 bytes.
No baseline memory peak was supplied for the timeout, so this does not establish
an additional memory reduction. Do not repeat this passed boundary test.

The generator folds constant-one product fields into a constant while keeping
all padding checks. The fixed saturated-cell adder now rejects out-of-range u32
operands before using a 17-bit sum and carry, and binds its small Bool circuit
outside output selectors. It stays within the unchanged binder-depth limit.
The observer can normalize large, sufficiently sparse concrete dense inputs
into exact sparse/uniform cubes; all addresses and demanded selectors remain.

Scoped checks passed: 19 observer, 4 recursive-domain, 4 aggregate behavior/
sharing tests, 7 source-domain tests, and a new addition regression covering
4,624 operand pairs and four lazy-rejection cases. One old sharing-performance
assertion depended on the adder being slow; its workload now uses fixed Bool
operations and the original >2 ratio assertion passes. Both unchanged checkers
accept the identical new adder certificate with zero axioms. Clippy and targeted
formatting passed. Evidence, failed attempts and source hashes:
`verification-logs/total-cell-runtime/verification.json`.

The user completed the full 16,384-element aggregate-sum test successfully:
`aggregate_fold_full_capacity_sums_exact_logical_bound`, one passed, no failures,
35.03 seconds, with MPK_CORE_MAX_STEPS unset and MPK_CORE_MAX_SECONDS=300.
Receipts: `verification-logs/total-cell-runtime/full-capacity-user.json` and
`full-capacity-user.log`. No memory peak was reported for this run. This closes
the outstanding affected full-capacity runtime check. Future affected runs belong to the agent under the current policy, regardless
of duration. Do not rerun it
now; no runtime code changed while importing this result.

The user-run decimal addition probe remains passed: 127.59 seconds and
255,066,688 bytes (243.25 MiB) peak physical footprint. Against the earlier
weak-cache run (173.35 seconds / 134.88 MiB), this is less time with increased
memory, not a further memory reduction. Its pinned certificate is unchanged;
no decimal rerun was performed for this fix. Decimal observations do not measure
the different total-cell input/certificate.

At that earlier checkpoint, string-data runtime cases and regenerated aggregate,
domain and consumer fixtures remained pending. String checks are now completed
as recorded above; current aggregate/domain fixture progress is recorded in
`verification-logs/domain-pin-refresh/verification.json`. Prior receipts remain
historical for replaced bytes. No T-wide gate or W09 internal unit was completed
at this checkpoint.

## Earlier handoff and diagnostic history

Run from `/Users/kazuyoshitoshiya/mpk`. Release mode and a single test thread
avoid debug evaluation overhead and simultaneous test memory peaks. The
observer's lower-memory policy is already the default; no setting is required.
Keep the normal pinned-fixture mode:

```sh
cd /Users/kazuyoshitoshiya/mpk
unset MPK_W09_STRING_DATA_OUT MPK_W09_STRING_DATA_RESPONSES MPK_W09_STRING_DATA_REQUESTS_OUT MPK_W09_STRING_DATA_RUNTIME_PREFIX
unset MPK_W09_CALENDAR_DATA_OUT MPK_W09_CALENDAR_DATA_RESPONSES MPK_W09_CALENDAR_DATA_REQUESTS_OUT
```

## Latest correction: propagate one carry bit, not computed length words

The user-run `zero_tail_` command failed both tests at 500 million steps per
thread (120-second ceilings); total test time was 15.00 seconds, compilation
32.74 seconds. The wide test completed its three length-zero observations
(13,917 / 5,174 / 23,878 transitions) but not the next length-one observation.
The small exhaustive test also exhausted its budget. No source probe was run.

The helper no longer reconstructs `HalfLength`/`Add1` word functions. At level
k it retains the original length and carries one Boolean, representing the
active count `(original_length >> k) + carry`. The even and odd branches update
carry with OR and AND respectively using the original bit k. Empty/full checks
read only original fixed bit addresses. This removes the nested arithmetic
word reevaluation path while preserving the same physical-slot coverage.
The wrapper checks physical capacity before any step; the outer sequence still
rejects lengths over the declared capacity.

**The user rerun passed both tests**, with unchanged limits and coverage:
0.58 seconds test time, 30.20 seconds compilation, 38 wide observations and a
maximum of 314,475 transitions (I12/D20, length 2048, last physical corruption).
The small exhaustive physical-address test passed too. The agent ran no tests.
The command below is retained for reproduction only; do not repeat it without
relevant changes:

```sh
MPK_CORE_MAX_STEPS=500000000 MPK_CORE_MAX_SECONDS=120 cargo test --release -p mpk-vc --lib zero_tail_ -- --nocapture --test-threads=1
```

The increasing-map/dirty-tail probe and original six-case suite have also
passed (details below). Next run the affected collection/full-UTF16 tests. No passed generic observer or C31-only
tests need rerunning for this helper-only change. Certificate regeneration is
still pending; the carry helpers change generated bytes again.

## Current next checks: active count and complete inactive tail

The six-case user run failed after the string duplicate and descending cases
completed in 0.132 and 0.131 seconds. The increasing case exhausted the
cumulative one-billion-step budget; Cargo reported 16.97 seconds total. Compound
cases were not reached. The fixed C31 zero-region improvement remains valid,
but did not remove the sequence fold's scan over every inactive slot.

The revised generator sums only `length` active slots. A separate ordinary
`ZeroTail` predicate checks every slot from `length` through rounded physical
capacity, including slots beyond a non-power-of-two declared capacity. It
partitions LSB-first indices with the equivalent original-bit/scalar-carry
recurrence described above, and checks entire zero subregions without counting
them. Declared length bounds, nested element
domains, ordering and saturated logical-cell limits remain enforced. There is
no host domain verdict or new checker primitive. Generated bytes change again;
fixtures/checker receipts remain pending.

**Both commands below have passed in user runs; they are retained for
reproduction only.** The increasing C33 map completed in 0.311 seconds. Dirty
first inactive entry (address 5) and last physical address (8,589,934,591) were
rejected in 0.002 and 0.003 seconds. Cargo reported one test passed in 0.35
seconds after 1m25s compilation. The agent ran no compilation or tests. Budgets were not increased for the failed
source scenario. The new helper tests exercise small truth tables, all child
addresses, empty/odd/half/full lengths, and first/last inactive corruption in
full string/map-sized storage. Each wide helper observation must use fewer
than ten million transitions. The source probe checks the previously failing
increasing map and both first-inactive and final-address corruptions.

```sh
cd /Users/kazuyoshitoshiya/mpk
MPK_CORE_MAX_STEPS=500000000 MPK_CORE_MAX_SECONDS=120 cargo test --release -p mpk-vc --lib -- recursive_domain_zero_tail_matches_physical_addresses recursive_domain_sparse_zero_tail_work_is_bounded --nocapture --test-threads=1
MPK_CORE_MAX_STEPS=1000000000 MPK_CORE_MAX_SECONDS=300 cargo test --release -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_domains_source_string_increasing_and_tail_probe -- --nocapture --test-threads=1
```

The original complete six-case suite **passed in the user run**, retaining
its one-billion-step / 300-second limits and C33/C34 carriers. Cargo reported
1.22 seconds total, one test passed. String duplicate/descending/increasing
took 0.117 / 0.116 / 0.302 seconds; compound duplicate/descending/increasing
took 0.123 / 0.126 / 0.362 seconds. This confirms the original reported runtime
failure is resolved for all six cases; it is not a new peak-memory measurement.
The following command is retained for reproduction only; no rerun is needed
without relevant changes:

```sh
MPK_CORE_MAX_STEPS=1000000000 MPK_CORE_MAX_SECONDS=300 cargo test --release -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_domains_source_string_compound_map_order -- --nocapture --test-threads=1
```

Next run affected collection-domain cases for general collection counts,
capacity rejection and UTF-16 storage. These consumers now use the changed
active-count/tail split, so previous passes do not cover this wiring:

```sh
MPK_CORE_MAX_STEPS=1000000000 MPK_CORE_MAX_SECONDS=300 cargo test --release -p mpk-vc --test csharp_practical_vc -- csharp_03_t06_w09_domains_source_collections_and_rejections csharp_03_t06_w09_domains_source_full_utf16_capacity --nocapture --test-threads=1
cargo clippy -p mpk-vc --lib --test csharp_practical_vc -- -D warnings
```

The generic evaluator and fixed-zero-only generator did not change in this
revision, so do not repeat their passed tests. The standalone duplicate probe
need not repeat separately because the full six-case suite covers that case.
Keep the fixture regeneration/replay section below for after runtime checks;
this revision affects the same current fixture families listed there. Tests
that cover total logical-cell saturation remain outstanding below. T06's full
gate stays deferred to W12.

## Earlier diagnostic results (historical commands)

**2026-09-20: use the bounded checks below before rerunning the old three-test
command.** The user-run
string/compound-map test remained on its first duplicate-key case after more
than seven hours. A process snapshot recorded PID 39637 at 07:29:18 elapsed,
98.8% CPU and 10,640 KiB RSS. This is a resident-memory snapshot, not a peak
measurement or a completed test. The process was absent on a subsequent check;
the agent did not stop it and has no terminal verdict. The existing standalone
duplicate probe executes the same expensive predicate and is not a workaround.

The source case contains two entries and only 13 true input leaves, but its
physical carrier is C33 (2^33 Boolean addresses). The current domain generator
checks length padding before the collection count/order result. Its first
length-padding sibling is a C31 region. The generator at the time recursively expanded
that region through counted folds to 32-bit leaf checks; sparse input storage
alone does not make this core predicate evaluation sparse. Exact executed
transition counts were not measured. This is a code-level explanation of the
cost mechanism, not a measured attribution to one cache policy.

Further work must preserve complete inactive-storage checks and original
limits while making this evaluation practical. Reordering rejection alone
would only accelerate invalid maps and would leave the increasing-map case
unresolved. The earlier small-decimal memory/time comparison does not justify
the runtime of this nested string-map domain. At that stage an evaluation-cost regression was required before repeating the
full cases. Those observations did not count as passed. The historical user-owned
test policy is superseded by the current all-tests-agent-owned policy above.

### Revised fixed-region generator after the failed observer-only attempt

The user ran the C31 regression with 500,000,000 steps and 120 seconds. It
exhausted the step budget in 7.89 seconds before the positive assertion.
The observer-only change is insufficient; this is a failed performance result,
not a domain rejection. Do not increase the budget.

The generator now defines each fixed zero region above depth 5 as the AND of
its two complete depth-minus-one regions. The 32-leaf scalar base remains.
Child definitions are emitted before parents and shared by depth. Every address
still belongs to exactly one branch; collection lengths, inactive storage and
capacity limits are unchanged. Actual counted collection operations retain
their existing folds. Uniform sparse subviews and demand-triggered memo reuse
allow identical empty halves to share results, without a host domain oracle.
The bounded weak-cache memory policy remains intact.

**This changes generated certificate bytes.** The previous statement that the
observer-only fix preserved all certificates does not apply to this revision.
Existing checked fixtures are historical evidence until regeneration and
same-byte checker replay; see the fixture section below. Neither compilation
nor tests were run by the agent for this revision.

The revised C31 regression **passed in the user run**: empty 24,856 transitions;
nonzero address 0: 1,603; address 1,073,741,824: 2,547; address 2,147,483,647:
47,388. Cargo reported 0.01 seconds for the test and 27.38 seconds compilation.
This establishes the bounded C31 observations only; source-map performance and
memory peaks remain unmeasured. Do not repeat this passing test without a new
relevant change.

The user also reports that both the observer/padding command and the C33
duplicate-source probe below passed. No detailed timing or test counts were
supplied for those two commands. All three commands below are now retained
for reproduction only; do not rerun them without relevant changes. It retains all 2^31 addresses, tests the empty
region and corruptions at the first, middle and final address, and now requires
less than one million core transitions for each of those four observations.
The environment budget covers all observations together.

```sh
cd /Users/kazuyoshitoshiya/mpk

MPK_CORE_MAX_STEPS=10000000 MPK_CORE_MAX_SECONDS=30 cargo test --release -p mpk-vc --lib recursive_domain_sparse_zero_work_is_bounded -- --nocapture --test-threads=1

MPK_CORE_MAX_STEPS=100000000 MPK_CORE_MAX_SECONDS=120 cargo test --release -p mpk-vc --lib -- ordinary_core_ recursive_domain_zero_regions_and_padding --nocapture --test-threads=1

MPK_CORE_MAX_STEPS=500000000 MPK_CORE_MAX_SECONDS=120 cargo test --release -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_domains_source_string_duplicate_probe -- --nocapture --test-threads=1
```

The second command covers the affected evaluator, weak-cache/laziness behavior,
and exhaustive small-region/padding mutations. The third generates the current
certificate in memory and reproduces the original two-entry C33 duplicate-key
map; it does not read pinned domain certificate bytes.

Limits accumulate per evaluation thread, including separate output-bit reads.
The deadline starts at the first evaluator call, excluding compilation and
source/certificate preparation; it is checked at call entry and every 65,536
core transitions. Exceeding either limit fails explicitly, never passes or
skips. C31, observer/padding and the duplicate-source probe have passed;
the complete six-case C33 map suite remains pending.

At that checkpoint the next test was the complete six-case map suite (now
reported failed on increasing, as recorded above): duplicate, descending
and increasing maps for both string and compound keys. This is necessary to
exclude an optimization that only makes invalid maps reject earlier.

```sh
MPK_CORE_MAX_STEPS=1000000000 MPK_CORE_MAX_SECONDS=300 cargo test --release -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_domains_source_string_compound_map_order -- --nocapture --test-threads=1
cargo clippy -p mpk-vc --lib --test csharp_practical_vc -- -D warnings
```

### Certificate regeneration and follow-up scope (pending)

After the bounded runtime checks pass, regenerate the current domain fixtures,
then replay them with generation mode unset. These commands write generated
fixtures; they do not alter historical `previous-*` snapshots or old receipts.

```sh
MPK_W09_DOMAINS_OUT=/Users/kazuyoshitoshiya/mpk/develop/migrations/csharp-03/ordinary-foundation/domains cargo test --release -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_domains_original_source_certificates -- --nocapture --test-threads=1
unset MPK_W09_DOMAINS_OUT
cargo test --release -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_domains_original_source_certificates -- --nocapture --test-threads=1
(cd go-tools/mpk-checker-ref && GOCACHE=/tmp/mpk-w09-go-cache go test -timeout 0 -tags checkeragreement -run '^TestCheckerAgreementWithRustCLIRecursiveDomains$' -count=1 -v)
```

The output path is absolute because Cargo runs tests in the crate directory.
No successful regeneration is recorded yet.

Reading the stored certificate name tables identified wide `Domain.Zero.D*`
definitions in these current fixture families. Regenerate their certificates
and metadata with the corresponding output variable, replay without that
variable, and rerun each family's same-byte dual-checker before considering
its old receipts current. This is affected-consumer coverage, not a W-wide gate.
Use the full test name below with the `csharp_03_t06_w09_` prefix. These are
follow-up checks after the bounded diagnosis, not one long initial test command.

| Current fixture directory | Output variable | Generation/replay test suffix |
| --- | --- | --- |
| domains | MPK_W09_DOMAINS_OUT | domains_original_source_certificates |
| collection-operations | MPK_W09_COLLECTIONS_OUT | collections_original_source_certificates |
| structural-foundations | MPK_W09_STRUCTURAL_FOUNDATIONS_OUT | structural_foundation_original_sources |
| structural-boundary | MPK_W09_STRUCTURAL_BOUNDARY_OUT | structural_boundary_original_sources |
| public-domains | MPK_W09_PUBLIC_DOMAINS_OUT | public_domains_construction_sources |
| public-defaults | MPK_W09_PUBLIC_DEFAULTS_OUT | public_defaults_source_conditions |
| structural-public | MPK_W09_STRUCTURAL_PUBLIC_OUT | structural_public_composes_existing_definitions |
| conditional-clauses | MPK_W09_CONDITIONAL_CLAUSES_OUT | conditional_clause_original_source |
| structural-clauses | MPK_W09_STRUCTURAL_CLAUSES_OUT | structural_clauses_original_source |
| literal-clauses | MPK_W09_LITERAL_CLAUSES_OUT | literal_clauses_original_source |
| total-clauses | MPK_W09_TOTAL_CLAUSES_OUT | total_clauses_original_source |
| json-calendar | MPK_W09_JSON_CALENDAR_OUT | json_calendar_programs |

Hash-dependent manifests/inventory and preservation tests must also be reviewed
against regenerated outputs. Keep historical archives unchanged. Fixture replay
will currently fail where the old generated bytes differ; old checker receipts
do not validate the new implementation. Actual collection-fold reconstruction
still expects its existing 8,217 transformers; only fixed zero-region tests now
expect zero fold transformers. Full T06 validation remains deferred to W12.

The original general string-data/total-cell cases below remain outstanding;
this fix is not evidence that either now finishes within a practical duration.
Do not start them as part of this initial performance diagnosis.

These are the unfinished tests, not repeats of completed semantic runs. The
string command covers all eight contexts, combining the two cancelled jobs.
The total-cell and compound-map cases retain their original full capacities.
The latter was queued but had not started. Each command may take a long time.

```sh
cargo test --release -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_string_data_original_source -- --nocapture --test-threads=1
cargo test --release -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_domains_source_total_cell_boundary -- --nocapture --test-threads=1
cargo test --release -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_domains_source_string_compound_map_order -- --nocapture --test-threads=1
```

## Checks for the calendar/string changes

These checks already passed before handoff; they are listed for independent
reproduction. The new lifted-data work below is not covered by these successes.
The filters select the new comparators, changed string cache and its consumers.

```sh
cargo test --release -p mpk-vc --lib -- ordinary_calendar_data_ ordinary_string_data_ string_contract_ string_native_cache_ --nocapture --test-threads=1
cargo test --release -p mpk-vc --test csharp_practical_vc -- csharp_03_t06_w09_string_data_requests csharp_03_t06_w09_string_data_candidates csharp_03_t06_w09_string_clauses_candidates csharp_03_t06_w09_calendar_data_requests csharp_03_t06_w09_calendar_data_candidates --nocapture --test-threads=1
cargo test --release -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_calendar_data_original_source -- --nocapture --test-threads=1
cargo test -p mpk-vc --test csharp_practical_inventory -- --nocapture --test-threads=1
cargo clippy -p mpk-vc --lib --test csharp_practical_vc -- -D warnings
```

The inventory module checks all recorded consumer counts and fingerprints.
The calendar runtime already passed 124 observations; string runtime remains
pending. The same-byte dual-checker commands below also already passed for all
eight calendar and all eight string certificates, including hash mutations.
They run locally and do not use CI workflows.

```sh
cd /Users/kazuyoshitoshiya/mpk/go-tools/mpk-checker-ref
GOCACHE=/tmp/mpk-w09-go-cache go test -timeout 0 -tags checkeragreement -run '^TestCheckerAgreementWithRustCLI(CalendarData|StringData)$' -count=1 -v
```

## In-progress lifted nullable adapter

`crates/mpk-vc/src/csharp_practical_ordinary_lifted_data.rs` is newly implemented
but not complete or verified. It connects lifted bool/integer/float/decimal
results and null-gated failures to W03 predicates. `cargo check -p mpk-vc --lib`
passed before the handoff. New source captures, tests, pins, checker wiring,
inventory changes and direct review are still required. In particular, the new
standard-namespace consumer has not yet been added to the inventory; the
inventory command above is expected to fail on this intermediate worktree.
Do not treat the existing scalar or string tests as proof of this adapter.
The agent should finish preparing this work and provide its commands, while
leaving execution to the user.
