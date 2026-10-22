"""Affected ordinary frozen error-layout checks; preserve complete prelaunch originals.

Run outside the sandbox because every Git command uses /usr/bin/git.
"""
from pathlib import Path
import datetime
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time

REPO = Path.cwd()
ROOT = Path(__file__).resolve().parent
BASE = 'd4ac004547a575e5f4e81ec48385dc50f93ed435'
epoch, mode = sys.argv[1:]
assert mode in {'client-clippy','client-format'}
assert epoch.replace('-', '').isalnum()


def seal(path):
    raw = Path(path).read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args])


assert git('rev-parse', 'HEAD').decode().strip() == BASE
out = ROOT / epoch
out.mkdir(exist_ok=False)
tracked = {}
for row in git('ls-files', '--stage', '-z').split(b'\0'):
    if row:
        meta, name = row.split(b'\t', 1)
        permissions, oid, stage = meta.decode().split()
        assert stage == '0'
        tracked[name.decode()] = oid
sources = [p for root in [Path('crates'), Path('proofs/std')] for p in root.rglob('*') if p.is_file()]
sources += [Path('Cargo.toml'), Path('Cargo.lock'), Path(__file__).relative_to(REPO)]
sources += [Path('develop/migrations/go-practical/t01-w04/word-foundation/queries.json'), Path('develop/probes/go-practical/closed_carriers.py'), Path('develop/probes/go-practical/carrier_terms.py'), Path('develop/migrations/go-practical/t01-w04/type-error-zero-decisions.md')]
sources += sorted(Path('.cargo').rglob('*')) if Path('.cargo').is_dir() else []
sources += [p for p in [Path('rust-toolchain'), Path('rust-toolchain.toml')] if p.exists()]
records = []
for path in sorted(set(p for p in sources if p.is_file())):
    raw = path.read_bytes()
    row = seal(path)
    row['mode'] = path.stat().st_mode & 0o777
    oid = tracked.get(str(path))
    if oid and hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == oid:
        row.update(original_commit=BASE, original_git_blob=oid)
    else:
        target = out / 'inputs' / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)
        target.chmod(row['mode'])
        row['complete_changed_input'] = str(target.relative_to(REPO))
    records.append(row)
driver = out / 'driver.py'
driver.write_bytes(Path(__file__).read_bytes())
receipt = out / 'inputs.json'
receipt.write_text(json.dumps({'basis_commit': BASE, 'sources': records, 'actual_driver_prelaunch': seal(driver)}, indent=2) + '\n')
commands = {
    'client-clippy': ['cargo','clippy','--offline','--locked','-p','mpk-vc','--lib','--test','go_practical_error_layout_carrier_client','--','-D','warnings'],
    'client-format': ['rustfmt','--edition=2021','--check','--config','skip_children=true','crates/mpk-vc/tests/go_practical_error_layout_carrier_client.rs'],
}
environment = {'CARGO_NET_OFFLINE': 'true', 'CARGO_PROFILE_TEST_OPT_LEVEL': '1',
               'CARGO_PROFILE_TEST_DEBUG_ASSERTIONS': 'true', 'CARGO_PROFILE_TEST_OVERFLOW_CHECKS': 'true',
               'CARGO_INCREMENTAL': '0', 'RUST_TEST_THREADS': '1'}
if mode == 'carrier-client':
    environment['MPK_GO_ERROR_LAYOUT_CARRIER_CLIENT_OUTPUT'] = str(out/'certificates')
if mode in {'construction', 'tests', 'diagnostic'}:
    environment['MPK_GO_ERROR_LAYOUTS_TEST_OUTPUT'] = str(out / 'certificates')
if mode == 'affected':
    environment['MPK_GO_ERROR_VALUES_TEST_OUTPUT'] = str(out/'certificates/error-values')
if mode == 'diagnostic':
    environment['MPK_GO_ERROR_VALUES_DIAGNOSTIC'] = '1'
env = os.environ.copy()
env.update(environment)
for name in ['RUST_MIN_STACK', 'MPK_CORE_MAX_STEPS', 'MPK_CORE_MAX_SECONDS']:
    env.pop(name, None)
record = {'status': 'running', 'command': commands[mode], 'environment': environment,
          'source_receipt': seal(receipt), 'actual_driver_prelaunch': seal(driver),
          'tools': {name: seal(shutil.which(name)) for name in ['cargo', 'rustc', 'rustfmt']},
          'selection_reason': 'Scoped lint/format of the independent named-carrier client. The running original matrix and its consumed main test file are not edited or reformatted. Actual typed client of the exported N0 carrier. A wrong but unused Sort0 definition can be a valid mathematical declaration; require its real named-carrier/nil client to type-reject rather than count unreferenced definition replacement as semantic rejection. This new independent test file is not consumed by the already-running original matrix, whose source inputs stay unchanged. Original producer and matrix code do not change. Exact frozen balanced [Word64, every payload] layout, canonical frozen zero-tag function and checked adapters to/from the complete payload observation view. Check arbitrary heterogeneous field preservation and both comparison transfers, empty-U handling, exact carrier/nil structural oracle,0..70 case layouts, nontrivial wrong-layout/case-field controls and original2050 error-view named identities/raw bytes. The current inline construction root already passes0/3/70 with frozen structural and all old-identity checks; final tests skip only that duplicate root while still kernel-checking every size0..70. Preserve both executions and all original streams. The shared builder was factored; run that affected original error-view suite. No source role/tag assignment/validity/dispatch authority or frontend/kernel/axiom/budget changes, unrelated suite or equivalent intermediate full gate.',
          'full_T_gate': 'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh',
          'formatter_mutation_expected': mode == 'format-write',
          'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat()}
result = out / 'result.json'
result.write_text(json.dumps(record, indent=2) + '\n')
start = time.monotonic()
with (out/'stdout.log').open('wb') as stdout, (out/'stderr.log').open('wb') as stderr:
    code = subprocess.run(commands[mode], env=env, stdout=stdout, stderr=stderr, check=False).returncode
unchanged = all(seal(r['path']) == {k:r[k] for k in ['path', 'bytes', 'sha256']}
                and Path(r['path']).stat().st_mode & 0o777 == r['mode'] for r in records)
record.update(status='passed' if code == 0 and (unchanged or mode == 'format-write') else 'failed',
              exit_code=code, seconds=time.monotonic()-start, whole_source_inputs_unchanged=unchanged,
              stdout=seal(out/'stdout.log'), stderr=seal(out/'stderr.log'),
              certificates=[seal(p) for p in sorted((out/'certificates').rglob('*.mpcert'))],
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
result.write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({k:record[k] for k in ['status', 'exit_code', 'seconds', 'whole_source_inputs_unchanged']}))
raise SystemExit(record['status'] != 'passed')
