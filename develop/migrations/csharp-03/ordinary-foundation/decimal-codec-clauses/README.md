# Decimal contract codec connections (targeted verification passed)

The adapter implements decimal.normalized and all 145 decimal.fixed settings,
sharing existing digit helpers and selecting exact nominal definitions. The
corrected fixture has six contexts and 177 attachments. All 292 unique aliases
retain complete dependency comparisons; runtime selects nine new source paths.

Review of a failed assertion corrected the maximum-value expectation: fixed
parsing strips excess fractional zeros to fit a 96-bit coefficient, so MAX
formatted with 28 fractional zeros must parse successfully. A separate MAX+1
literal tests genuine Range rejection. Both expectations are checked against
the independent reference parser when constructing the requests.

The original six helper candidates passed both checkers but their ToEven
semantic test failed because of that incorrect assertion. They are retained in
previous-range-expectation/ with their own manifests and captures. Those checker
passes do not prove the failed source condition or application VCs. Corrected
captures and candidates passed structural/import checks. Five changed
candidates passed both checkers; unchanged normalized bytes reuse their prior
checker evidence. All seven affected
fixed-mode source conditions passed; two unchanged normalized conditions reuse
the completed earlier context on identical program bytes. Final evidence is in
../unit-4-decimal-codec-clauses-progress.json.

The first, wider adapter runtime run was intentionally stopped because it
repeated unchanged standalone semantics. The next narrowed run produced the
reported expectation failure. Neither run counts as a completed semantic pass.
Original unit 4 and W09 remain open; the full gate is deferred to T06-W12.
