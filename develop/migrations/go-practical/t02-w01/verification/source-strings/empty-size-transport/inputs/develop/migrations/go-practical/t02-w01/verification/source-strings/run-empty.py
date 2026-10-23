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
BASE = '82bd5eedf30c51a443df7bf7092407a41c265892'
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
sources = [p for root in [Path('crates'), Path('proofs/std'), Path('go-tools/go2vir')] for p in root.rglob('*') if p.is_file()]
sources += [Path('Cargo.toml'), Path('Cargo.lock'), Path(__file__).relative_to(REPO)]
sources += [ROOT.parent/'source-error-representation/corpus-heterogeneous.json', ROOT.parent/'error-dispatch/corpus.json']
if (ROOT/'corpus.json').exists(): sources += [ROOT/'corpus.json']
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
goroot=Path('/private/tmp/mpk-go-practical-original-packages-sdk-20261021/go')
changed=['crates/mpk-vc/src/go_practical_ordinary.rs','crates/mpk-vc/src/go_practical_ordinary/composition.rs','crates/mpk-vc/src/go_practical_ordinary/source_string.rs','crates/mpk-vc/tests/go_practical_source_string.rs']
commands={
 'format-write':['rustfmt','--edition=2021','--config','skip_children=true',*changed],
 'format':['rustfmt','--edition=2021','--check','--config','skip_children=true',*changed],
 'strings':['cargo','test','--offline','--locked','-p','mpk-vc','--test','go_practical_source_string','source_strings_empty_and_entry','--','--nocapture','--test-threads=1'],
 'clippy':['cargo','clippy','--offline','--locked','-p','mpk-vc','--lib','--test','go_practical_source_string','--','-D','warnings'],
 'go-format-write':[str(goroot/'bin/gofmt'),'-w','go-tools/go2vir/practical_source_string_test.go'],
 'go-format':[str(goroot/'bin/gofmt'),'-l','go-tools/go2vir/practical_source_string_test.go'],
 'native':[str(goroot/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalSourceStringOriginalTypes$'],
}
environment = {'CARGO_NET_OFFLINE': 'true', 'CARGO_PROFILE_TEST_OPT_LEVEL': '1',
               'CARGO_PROFILE_TEST_DEBUG_ASSERTIONS': 'true', 'CARGO_PROFILE_TEST_OVERFLOW_CHECKS': 'true',
               'CARGO_INCREMENTAL': '0', 'RUST_TEST_THREADS': '1'}
if mode=='strings': environment['MPK_GO_SOURCE_STRING_OUTPUT'] = str(out/'certificates')
if mode.startswith('go-') or mode=='native':
 environment.update({'GOROOT':str(goroot),'GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOMAXPROCS':'1','GOCACHE':'/private/tmp/mpk-go-practical-t02-go-cache','GOMODCACHE':'/private/tmp/mpk-go-practical-t02-go-mod'})
if mode=='native':
 (out/'native-originals').mkdir()
 environment['MPK_GO_PRACTICAL_SYNTAX_EVIDENCE']=str(out/'native-originals')
env = os.environ.copy()
env.update(environment)
for name in ['RUST_MIN_STACK', 'MPK_CORE_MAX_STEPS', 'MPK_CORE_MAX_SECONDS']:
    env.pop(name, None)
record = {'status': 'running', 'command': commands[mode], 'environment': environment,
          'source_receipt': seal(receipt), 'actual_driver_prelaunch': seal(driver),
          'tools': {name: seal(path) for name,path in ({'go':goroot/'bin/go','gofmt':goroot/'bin/gofmt'} if mode.startswith('go-') or mode=='native' else {n:shutil.which(n) for n in ['cargo','rustc','rustfmt']}).items()},
          'selection_reason': 'Affected new source string adapters and multi-input composition only: all original source types and complete U, builtin and nominal string rows, whole source/math roundtrips, original zero equality and validity, whole 64-bit lengths and all256 byte slots, constructor and guarded byte observation laws. Existing unrelated ordinary producers and kernel code unchanged. No Go expression evaluation, signed int result/index, full context authority, funext, axiom, reduced universe or new admission cap. New native Go fixture observations are comparison data. Scoped format, Clippy and fixture-only native test; full T02 gate deferred.',
          'full_T_gate': 'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh',
          'formatter_mutation_expected': mode in {'format-write','go-format-write'},
          'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat()}
result = out / 'result.json'
result.write_text(json.dumps(record, indent=2) + '\n')
start = time.monotonic()
with (out/'stdout.log').open('wb') as stdout, (out/'stderr.log').open('wb') as stderr:
    code = subprocess.run(commands[mode], cwd='go-tools/go2vir' if mode=='native' else '.', env=env, stdout=stdout, stderr=stderr, check=False).returncode
unchanged = all(seal(r['path']) == {k:r[k] for k in ['path', 'bytes', 'sha256']}
                and Path(r['path']).stat().st_mode & 0o777 == r['mode'] for r in records)
record.update(status='passed' if code == 0 and (unchanged or mode in {'format-write','go-format-write'}) else 'failed',
              exit_code=code, seconds=time.monotonic()-start, whole_source_inputs_unchanged=unchanged,
              stdout=seal(out/'stdout.log'), stderr=seal(out/'stderr.log'),
              certificates=[seal(p) for p in sorted((out/'certificates').rglob('*.mpcert'))],
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
result.write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({k:record[k] for k in ['status', 'exit_code', 'seconds', 'whole_source_inputs_unchanged']}))
raise SystemExit(record['status'] != 'passed')
