"""Check the 18 integrated native/predicate pins with the existing harness.

Integral measure pins retain the prior bytes and are outside this run. These
definitions do not discharge execution scopes or application proof obligations.
"""
import importlib.util
from pathlib import Path

HARNESS = Path(__file__).resolve().parents[2] / 'tool-sources' / 'check.py'
spec = importlib.util.spec_from_file_location('predicate_check', HARNESS)
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)
checker.CASES = tuple(case for case in checker.CASES if case != 'measures')
checker.__doc__ = __doc__

if __name__ == '__main__':
    checker.main()
