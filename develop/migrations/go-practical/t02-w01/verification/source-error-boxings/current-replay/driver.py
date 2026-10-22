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
BASE = '310acba813149c17ccba76ba283b351efc1f42f6'
epoch, mode = sys.argv[1:]
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
sources += [p for epoch in ['first','first-large'] for p in (ROOT.parent/'source-error-layouts'/epoch/'certificates').iterdir() if p.is_file()]
sources += [ROOT.parent/'error-layouts/final/certificates'/f'matrix-N{n}.{ext}' for n in range(71) for ext in ['mpcert','rust.json']]

if mode == 'diagnostic':
    sources += [p for p in Path(__file__).parent.joinpath('first/certificates').iterdir() if p.is_file()]

sources += [Path('develop/migrations/go-practical/t02-w01/verification')/name/'corpus.json' for name in ['error-dispatch','source-domains']]
sources += [Path('develop/migrations/go-practical/t01-w04/word-foundation/queries.json'), Path('develop/probes/go-practical/closed_carriers.py'), Path('develop/probes/go-practical/carrier_terms.py'), Path('develop/migrations/go-practical/t01-w04/type-error-zero-decisions.md')]
sources += sorted(Path('.cargo').rglob('*')) if Path('.cargo').is_dir() else []
sources += [p for p in [Path('rust-toolchain'), Path('rust-toolchain.toml')] if p.exists()]
sources = [p.relative_to(REPO) if p.is_absolute() else p for p in sources]
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
assert mode=='replay'
commands={'replay':['cargo','test','--offline','--locked','-p','mpk-vc','--test','go_practical_source_error_layouts','--test','go_practical_error_layout_replay','current_producer','--','--nocapture','--test-threads=1']}
environment = {'CARGO_NET_OFFLINE': 'true', 'CARGO_PROFILE_TEST_OPT_LEVEL': '1',
               'CARGO_PROFILE_TEST_DEBUG_ASSERTIONS': 'true', 'CARGO_PROFILE_TEST_OVERFLOW_CHECKS': 'true',
               'CARGO_INCREMENTAL': '0', 'RUST_TEST_THREADS': '1'}
environment['MPK_GO_SOURCE_ERROR_REPLAY_OUTPUT']=str(out/'source-literal-map.json')
environment['MPK_GO_ERROR_LAYOUT_REPLAY_OUTPUT']=str(out/'generic-literal-map.json')
env = os.environ.copy()
env.update(environment)
for name in ['RUST_MIN_STACK', 'MPK_CORE_MAX_STEPS', 'MPK_CORE_MAX_SECONDS']:
    env.pop(name, None)
record = {'status': 'running', 'command': commands[mode], 'environment': environment,
          'source_receipt': seal(receipt), 'actual_driver_prelaunch': seal(driver),
          'tools': {name: seal(shutil.which(name)) for name in ['cargo', 'rustc', 'rustfmt']},
          'selection_reason': 'New source-U boxing constructors retain the complete balanced error carrier, full active payload and every inactive source zero. Exact private source tags, whole payload-vector and active/tag observations, nonnil for all values including concrete zero, and own-case comparison have actual checked witnesses. Retain every core prerequisite and source type/zero identity with exact dependency closure; unused generic adapter laws stay unchanged in their original full library. Replay all prior source-layout and generic artifacts literally after the builder refactor. Check original empty/single/dispatch-grown/root-with-slice source units before separate actual70-case source unit. No smaller case limit, kernel/axiom/native budget or frontend changes; source runtime validity/canonical context/boxing correspondence remain due. Scoped tests only, no unrelated suites or intermediate broad gate.',
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
