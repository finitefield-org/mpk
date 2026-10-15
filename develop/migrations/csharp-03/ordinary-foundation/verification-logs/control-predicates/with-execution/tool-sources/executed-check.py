"""Check integrated native/predicate certificates using the existing harness.

The unchanged harness checks exact bytes, observed exits, all axiom categories,
Go/Rust reports and hash mutations. Integral measure pins are unchanged and were
checked at the prior checkpoint; this run covers the 18 new integrated pins.
"""
import importlib.util
from pathlib import Path

HARNESS = Path('/Users/kazuyoshitoshiya/.codex/worktrees/w09-default-use/mpk/develop/migrations/csharp-03/ordinary-foundation/verification-logs/control-predicates/tool-sources/check.py')
spec = importlib.util.spec_from_file_location('predicate_check', HARNESS)
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)
checker.CASES = tuple(case for case in checker.CASES if case != 'measures')
checker.__doc__ = __doc__

if __name__ == '__main__':
    checker.main()
