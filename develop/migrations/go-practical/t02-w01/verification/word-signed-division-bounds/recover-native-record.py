"""Recover count-only wrapper metadata without repeating native observations."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import stat
import subprocess
import time

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
out = root / 'native'

def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}

original_runner = out / 'original-runner.py'
source_code = original_runner.read_text()
assert "len({r['id'] for r in observations})==128" in source_code
assert "len(observations)==96" in source_code
assert (root / 'run-native.py').read_text() == source_code.replace(
    "len({r['id'] for r in observations})==128", "len({r['id'] for r in observations})==96")
workspaces = list(Path('/private/tmp').glob('mpk-go-practical-signed-division-bounds-native-*'))
assert len(workspaces) == 1
work = workspaces[0]
toolchain = work / 'image'
goroot = toolchain / 'go'
go = goroot / 'bin/go'
binary = work / 'signed-division-bounds-observer'
source = out / 'inputs/division_bounds_observer.go'
queries = out / 'inputs/queries.json'
assert source.read_bytes() == Path('develop/probes/go-practical/division_bounds_observer.go').read_bytes()
assert queries.read_bytes() == (root / 'queries.json').read_bytes()
one = (out / 'run1.stdout.log').read_bytes()
assert one == (out / 'run2.stdout.log').read_bytes()
rows = json.loads(one)
assert len(rows) == len({row['id'] for row in rows}) == 96
assert [row['id'] for row in rows] == [query['id'] for query in json.loads(queries.read_text())]
for name in ['build', 'run1', 'run2']:
    assert (out / (name + '.stderr.log')).read_bytes() == b''
assert (out / 'build.stdout.log').read_bytes() == b''
overrides = {
    'GOROOT': str(goroot), 'GOCACHE': str(work / 'cache'), 'GOPATH': str(work / 'gopath'),
    'GOMODCACHE': str(work / 'mod'), 'GOENV': 'off', 'GOWORK': 'off', 'GOTOOLCHAIN': 'local',
    'GOPROXY': 'off', 'GOSUMDB': 'off', 'GOTELEMETRY': 'off', 'GO111MODULE': 'off',
    'CGO_ENABLED': '0', 'GOFLAGS': '', 'GOEXPERIMENT': '',
}
environment = os.environ.copy()
environment.update(overrides)
version_run = subprocess.run([str(go), 'version'], env=environment, capture_output=True, check=False)
(out / 'recovery-version.stdout.log').write_bytes(version_run.stdout)
(out / 'recovery-version.stderr.log').write_bytes(version_run.stderr)
version = version_run.stdout.decode().strip()
assert version_run.returncode == 0 and not version_run.stderr and version == 'go version go1.25.0 darwin/arm64'
prepared = root.parent / 'compiler-image/captured/selected/prepared-image.json'
image = json.loads(prepared.read_text())
expected = {row['path'] for row in image['inventory']['files']}
assert len(expected) == 5577
start = time.monotonic()
actual = set()
for directory, directories, files in os.walk(goroot, followlinks=False):
    for name in directories:
        assert not (Path(directory) / name).is_symlink()
    for name in files:
        actual.add(str((Path(directory) / name).relative_to(toolchain)))
assert actual == expected
for row in image['inventory']['files']:
    path = toolchain / row['path']
    status = path.lstat()
    assert stat.S_ISREG(status.st_mode) and not path.is_symlink()
    assert status.st_size == row['size_bytes'] and bool(status.st_mode & 0o111) == row['executable']
    assert hashlib.sha256(path.read_bytes()).hexdigest() == row['sha256']
after = {'label': 'Full image revalidation during metadata recovery after original native executions',
         'status': 'passed', 'files': 5577, 'original_bytes': sum(r['size_bytes'] for r in image['inventory']['files']),
         'seconds': time.monotonic() - start}
commands = []
for name, command in [
    ('build', [str(go), 'build', '-trimpath', '-buildvcs=false', '-o', str(binary), str(source.resolve())]),
    ('run1', [str(binary)]), ('run2', [str(binary)]),
]:
    commands.append({'name': name, 'command': command, 'exit_code': 0, 'seconds': None,
                     'stdout': seal(out / (name + '.stdout.log')), 'stderr': seal(out / (name + '.stderr.log')),
                     'metadata_provenance': 'Command/path reconstructed from the preserved exact original runner and its unique owned workspace. Zero exit status follows from its asserted run() calls and the observed later count-only assertion. Original command elapsed times were not persisted before that exception and are intentionally null.'})
record = {
    'schema': 'mpk.go_practical.fresh_signed_division_bounds_native_observation.v1', 'status': 'passed',
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'original_source': seal('develop/probes/go-practical/division_bounds_observer.go'), 'owned_source': seal(source),
    'original_queries': seal(root / 'queries.json'), 'owned_queries': seal(queries),
    'compiler_version': version, 'compiler': seal(go), 'observer_binary': seal(binary),
    'complete_original_prepared_image': seal(prepared),
    'complete_original_image_checks': [
        {'label': 'Original pre-build image validation', 'status': 'passed_by_reached_control_flow',
         'files': 5577, 'seconds': None,
         'evidence': 'The exact original runner validates every copied name/byte/mode before constructing the empty cache and running any command. Its observed traceback occurs after build and both runs, so that earlier validation completed. Its original in-memory elapsed time was lost and is not recreated.'}, after],
    'build_cache_initially_empty': True,
    'build_cache_initialization_evidence': 'Preserved original runner creates a unique new workspace and an empty new cache directory before its build; the original later count assertion was reached.',
    'environment': overrides, 'commands': commands, 'two_native_outputs_byte_identical': True, 'native_rows': 96,
    'metadata_recovery': {'original_runner': seal(original_runner), 'original_wrapper_exit_code': 1,
                          'observed_tool_failure_text': seal(out / 'observed-wrapper-failure.txt'),
                          'failure_text_provenance': 'Text retained from the terminal tool response; native command raw stdout/stderr are separate original binary files.',
                          'fixed_runner': seal(root / 'run-native.py'), 'recovery_script': seal(Path(__file__).resolve().relative_to(Path.cwd().resolve())),
                          'only_original_failure': 'Copied unique-ID expected count128 instead of96. The actual len96 and all96 unique IDs pass, as do byte-identical outputs and every native observer assertion.',
                          'native_build_or_observation_repeated': False, 'original_timing_fields_recreated': False,
                          'recovery_version_recheck': {'command': [str(go), 'version'], 'exit_code': 0,
                                                       'stdout': seal(out / 'recovery-version.stdout.log'), 'stderr': seal(out / 'recovery-version.stderr.log')}},
    'scope': 'All96 original complete signed source rows, including8 real recovered zero-divisor rows and8 normal minimum/-one rows, retain exact original native outputs. Typed source quotient/remainder, independent unsigned magnitudes, math-big gaps and corrected/exact reconstruction passed native assertions. The wrapper count metadata is recovered with explicit control-flow provenance and a fresh full5577-file image revalidation; unavailable original timings remain null. No native build/observation was repeated. No source/application proof or Linux release gate is inferred.',
    'CSharp_tests_executed': False, 'public_profile_activated': False,
    'full_T02_gate': 'deferred to W04: check-fast.sh --skip-csharp',
}
assert not (out / 'result.json').exists()
(out / 'result.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({'status': 'passed', 'native_rows': 96, 'two_outputs_identical': True,
                  'full_image_revalidated': 5577, 'native_observations_repeated': False,
                  'original_timing_fields_recreated': False}))
