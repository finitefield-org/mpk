"""Affected practical-Go native-host/resource helpers; no CSharp test execution."""
from pathlib import Path
import datetime, hashlib, json, os, re, subprocess, sys, time
root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
mode, attempt = sys.argv[1:]
assert mode in {'selected', 'regression', 'lint', 'format'} and attempt.replace('-', '').isalnum()
out = root / attempt / mode; out.mkdir(parents=True, exist_ok=False)
input_path = root / 'source-epochs' / attempt / 'inputs.json'
if not input_path.exists(): input_path = root / 'inputs.json'
inputs = json.loads(input_path.read_text())
def seal(path):
    raw = Path(path).read_bytes(); return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}
changed = ['crates/mpk-cli/src/frontend_sandbox.rs', 'crates/mpk-cli/src/go_practical_frontend_runner.rs',
           'crates/mpk-linux-sandbox/src/lib.rs', 'crates/mpk-linux-sandbox/src/go_policy.rs']
commands = {
 'selected': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli', '--lib', 'frontend_sandbox::', '--', '--nocapture', '--test-threads=1'],
 'regression': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli', '--lib', 'frontend_sandbox::tests::go_practical_probe_resource_failure_cannot_become_environment_failure', '--', '--exact', '--nocapture', '--test-threads=1'],
 'lint': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--lib', '--', '-D', 'warnings'],
 'format': ['rustfmt', '--check', '--edition', '2021', '--config', 'skip_children=true', *changed],
}
env = os.environ.copy(); overrides = {'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'RUST_TEST_THREADS': '1'}
env.update(overrides)
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
for path in changed:
    copy = out / 'inputs' / path; copy.parent.mkdir(parents=True, exist_ok=True); copy.write_bytes(Path(path).read_bytes())
record = {'status': 'running', 'command': commands[mode], 'environment': overrides,
 'sources': inputs['sources'], 'complete_input_receipt': seal(input_path),
 'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'selection_reason': 'Separate frozen practical-Go native-host preparation and syscall/thread/process policy. Run affected shared host/resource/accounting/cleanup/watchdog/namespace helpers and new Go contract/guard cases only. Current Rust lint and changed-file format. Linux separately compiles/runs Go/unchanged Java BPF cases and real non-native rejection. Go source/compiler/frontend/foundation are unchanged; retain their passing checkpoints without rerun. No CSharp, agents, Actions or intermediate whole gate.',
 'CSharp_tests_executed': False, 'T02_W01_complete': False, 'R06_status': 'pending',
 'native_Linux_release_acceptance_claimed': False, 'full_T02_gate': 'deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp'}
result = out / 'result.json'; result.write_text(json.dumps(record, indent=2) + '\n'); start = time.monotonic()
with (out / 'stdout.log').open('wb') as stdout, (out / 'stderr.log').open('wb') as stderr:
    code = subprocess.run(commands[mode], env=env, stdout=stdout, stderr=stderr, check=False).returncode
record.update(status='passed' if code == 0 else 'failed', exit_code=code, seconds=time.monotonic()-start,
    finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), stdout=seal(out/'stdout.log'), stderr=seal(out/'stderr.log'))
record['whole_inputs_unchanged_during_execution'] = all(seal(r['path'])['sha256'] == r['sha256'] and seal(r['path'])['bytes'] == r['bytes'] for r in inputs['sources'])
if not record['whole_inputs_unchanged_during_execution']:record['status']='failed'
if mode in {'selected', 'regression'} and code == 0:
    text = (out / 'stdout.log').read_text(); record['passed_tests'] = re.findall(r'^test ([^ ]+) \.\.\. ok$', text, re.M)
    assert record['passed_tests'] and all('csharp' not in name.lower() for name in record['passed_tests'])
result.write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({k: record[k] for k in ['status', 'exit_code', 'seconds', 'whole_inputs_unchanged_during_execution']}))
if code:print((out/'stderr.log').read_text()[-2200:])
raise SystemExit(0 if record['status'] == 'passed' else 1)
