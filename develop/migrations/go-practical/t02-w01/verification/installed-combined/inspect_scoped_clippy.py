"""Verify the target-only lint invocations and their rejecting control."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
run = ROOT / 'scoped-final'
result = json.loads((run / 'result.json').read_bytes())
inputs = json.loads((run / 'inputs.json').read_bytes())
assert result['status'] == 'passed' and result['original_inputs_unchanged']
assert all(row['status'] == 'passed' for row in result['checks'].values())
assert result['checks']['lint-control']['exit_code'] != 0 and 'clippy::bool_comparison' in (run / 'lint-control.stderr').read_text()
expected_extra = ['--cfg', 'test', '-Aexplicit_builtin_cfgs_in_flags', '-Dwarnings']
records = []
for name, suffix in [('lint-control', 'examples/go_practical_live_combined_roots.rs'), ('lint-original', 'crates/mpk-cli/examples/go_practical_live_combined_roots.rs')]:
    path = run / (name + '.target.jsonl')
    rows = [json.loads(line) for line in path.read_text().splitlines()]
    assert len(rows) == 1
    row = rows[0]
    args = row['arguments']
    assert args[args.index('--crate-name') + 1] == 'go_practical_live_combined_roots'
    assert suffix in args and row['extra_arguments'] == expected_extra and row['driver'] == inputs['clippy_driver']
    raw = path.read_bytes()
    records.append({'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'target_source': suffix})
for name in ['scoped_clippy_wrapper.py', 'run_scoped_clippy.py']:
    path = ROOT / name
    expected = next(row for row in inputs['inputs'] if row['path'] == str(path))
    raw = path.read_bytes()
    assert len(raw) == expected['bytes'] and hashlib.sha256(raw).hexdigest() == expected['sha256'] and path.stat().st_mode & 0o777 == expected['mode']
target = REPO / 'crates/mpk-cli/examples/go_practical_live_combined_roots.rs'
expected = next(row for row in inputs['inputs'] if row['path'] == str(target))
raw = target.read_bytes()
assert len(raw) == expected['bytes'] and hashlib.sha256(raw).hexdigest() == expected['sha256']
audit = {'status': 'passed', 'selection_reason': 'Check that the real Clippy driver rejected a separate deliberate lint error and was applied with test cfg only to the exact original example target.', 'target_invocations': records, 'original_target_sha256': expected['sha256'], 'control_rejected_by_clippy': True, 'normal_clippy_passed': True, 'test_binary_build_passed': True, 'dependency_sources_replaced': False, 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
out = ROOT / 'lint-scope-audit.json'
assert not out.exists()
out.write_text(json.dumps(audit, indent=2) + '\n')
print(json.dumps({'status': audit['status'], 'target_invocations': len(records), 'control_rejected_by_clippy': True}), flush=True)
