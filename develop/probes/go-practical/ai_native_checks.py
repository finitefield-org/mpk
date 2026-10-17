"""Targeted unchanged native AI consumer tests in the existing local container."""
import os
from pathlib import Path
import re
import subprocess
import time
import tomllib
from policy_native_build import prefix, CARGO, linked_inputs
from run_proof_pilot import ROOT, sha
from check_scope_report import require

FIXTURES = ('fixtures/csharp/ai', 'fixtures/csharp/policy',
            'fixtures/rust-basic/positive/module-calls/artifacts',
            'rust-tools/rust2vir/testdata/positive/module-calls/source')
FILES = ('crates/mpk-cli/tests/csharp_ai_explain.rs',
         'crates/mpk-cli/tests/support/successor_policy.rs',
         'develop/specs/vectors/ai-explain-v1.json',
         'develop/specs/vectors/csharp-profile-v0.json',
         'develop/specs/vectors/java-profile-v0.json',
         'develop/specs/vectors/semantic-profile-registry-v1.json',
         'develop/specs/vectors/semantic-profile-registry-v2.json',
         'develop/specs/vectors/semantic-profile-registry-v3.json',
         'develop/specs/vectors/manifest.json')
TESTS = ('all_profiles_stage_deterministic_redacted_requests_and_untrusted_reports',
         'csharp_ai_owner_is_registered_for_every_consumed_frozen_vector')


def inputs():
    rows = {r['path']: r['sha256'] for r in linked_inputs()}
    paths = [ROOT/p for p in FILES]
    for folder in FIXTURES: paths += [p for p in (ROOT/folder).rglob('*') if p.is_file()]
    for p in paths: rows[p.relative_to(ROOT).as_posix()] = sha(p.read_bytes())
    return [{'path': p, 'sha256': h} for p, h in sorted(rows.items())]


def run(record):
    before = inputs(); p, audit = prefix()
    # Only exact root-lock archives already present on this host are copied.
    # The selected test's dev dependencies are absent from the policy-only lock.
    registry = Path.home()/'.cargo/registry'; copied = []
    for row in tomllib.loads((ROOT/'Cargo.lock').read_text())['package']:
        if 'checksum' not in row: continue
        available = sorted(x for x in (registry/'cache').glob('*/'+row['name']+'-'+row['version']+'.crate') if sha(x.read_bytes()) == row['checksum'])
        if not available: continue
        source = available[0]; target = CARGO/'registry/cache'/source.parent.name/source.name
        target.parent.mkdir(parents=True, exist_ok=True)
        if not target.exists() or sha(target.read_bytes()) != row['checksum']: target.write_bytes(source.read_bytes())
        copied.append({'name': row['name'], 'version': row['version'], 'sha256': row['checksum']})
    command = p + ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli',
                   '--test', 'csharp_ai_explain', '--', '--nocapture', '--test-threads=1']
    record.mkdir(parents=True, exist_ok=True); runs = []
    for index in (1, 2):
        start = time.monotonic()
        result = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=600)
        for stream in ('stdout', 'stderr'): (record/f'native-ai-{index}-{stream}.log').write_bytes(getattr(result, stream))
        require(result.returncode == 0, 'actual native AI test setup/execution failed; inspect raw logs,not a source/ordinary falsehood')
        text = result.stdout.decode()
        require('running 2 tests' in text and '2 passed; 0 failed; 0 ignored' in text and
                all(re.search(r'test '+name+r' \.\.\. ok', text) for name in TESTS), 'two actual Linux test bodies ran; no cfg-skipped or filtered substitute')
        runs.append({'exit_code': result.returncode, 'wall_seconds': time.monotonic()-start,
                     'stdout_sha256': sha(result.stdout), 'stderr_sha256': sha(result.stderr),
                     'tests': list(TESTS), 'passed': 2})
    require(before == inputs(), 'native AI consumer/test/fixture sources fixed during both runs')
    return {'scope': 'unchanged public AI consumers over real GoFixed,Rust,CSharp certificates; Java AI and APIv4 execution not observed here',
            'command': command, 'container': audit, 'available_root_archives': copied,
            'inputs': before, 'runs': runs, 'network_used': False,
            'release_gate': False, 'public_source_changed': False}
