"""Recheck only the scoped test changed by the two unnecessary_unwrap repairs.

Emitter/library sources and the other two predicate mode bodies are unchanged.
Retain their observed passing exits, original source manifest and exact test
binaries; rerun the changed scope test, integration/library Clippy and formatting.
"""
import argparse
from datetime import datetime, timezone
from hashlib import sha256
import json
import os
from pathlib import Path
import re
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--reports', type=Path, required=True)
    parser.add_argument('--target', type=Path, required=True)
    args = parser.parse_args()
    manifest = json.loads((args.reports / 'source-manifest.json').read_bytes())
    prior = args.reports / 'before-style-fix'
    old = json.loads((prior / 'source-manifest.json').read_bytes())
    assert manifest['fixture_hashes'] == old['fixture_hashes']
    assert set(manifest['source_hashes']) == set(old['source_hashes'])
    changed = {name for name in manifest['source_hashes'] if manifest['source_hashes'][name] != old['source_hashes'][name]}
    assert changed == {'crates/mpk-vc/tests/support/csharp_practical_ordinary_control_predicate_tests.rs'}
    stages = [row for row in json.loads((prior / 'selected-stages.json').read_bytes())
              if row['stage'] in ('predicate-regressions', 'scope-composition')]
    assert len(stages) == 2 and all(row['exit_code'] == 0 for row in stages)
    for row in stages:
        row.update(retained_reason='Only the scope-mode test changed for equivalent if-let style; emitter, library unit and other two mode bodies are unchanged',
                   source_manifest='attempt-1/source-manifest.json',
                   source_manifest_sha256=sha256((prior / 'source-manifest.json').read_bytes()).hexdigest())
    binaries = json.loads((prior / 'binaries.json').read_bytes())
    env = os.environ.copy()
    env.update(CARGO_TARGET_DIR=str(args.target), CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0',
               CARGO_PROFILE_TEST_OPT_LEVEL='2', CARGO_PROFILE_TEST_DEBUG='0',
               CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true', CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
    for name in ('MPK_W09_CONTROL_PATTERN_SCOPE_OUTPUT', 'MPK_W09_CONTROL_PREDICATE_OUTPUT',
                 'MPK_W09_CONTROL_PREDICATE_INTEGRATED_OUTPUT'):
        env.pop(name, None)
    for label, arguments, count in [
        ('pinned-scopes', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w09_control_predicates_with_pattern_scopes_original_sources', '--', '--nocapture', '--test-threads=1'], 1),
        ('clippy', ['clippy', '-p', 'mpk-vc', '--lib', '--test', 'csharp_practical_vc', '--', '-D', 'warnings'], 0),
        ('format', ['fmt', '-p', 'mpk-vc', '--', '--check'], 0),
    ]:
        for name, digest in manifest['source_hashes'].items():
            assert sha256((args.repo / name).read_bytes()).hexdigest() == digest, name
        log = args.reports / f'{label}.log'
        start = time.monotonic()
        with log.open('wb') as out:
            code = subprocess.call(['cargo', *arguments], cwd=args.repo, env=env,
                                   stdin=subprocess.DEVNULL, stdout=out, stderr=subprocess.STDOUT)
        stages.append({'stage': label, 'command': ['cargo', *arguments], 'exit_code': code,
                       'elapsed_seconds': round(time.monotonic() - start, 3), 'tests_passed': count,
                       'log_sha256': sha256(log.read_bytes()).hexdigest()})
        (args.reports / 'selected-stages.json').write_text(json.dumps(stages, indent=2, sort_keys=True) + '\n')
        print(label, code, flush=True)
        assert code == 0, (label, code)
        if count:
            assert f'test result: ok. {count} passed' in log.read_text()
            for path in re.findall(r'Running [^\n]*\(([^\n]+)\)', log.read_text()):
                binaries[path] = sha256(Path(path).read_bytes()).hexdigest()
    for name, digest in manifest['source_hashes'].items():
        assert sha256((args.repo / name).read_bytes()).hexdigest() == digest, name
    result = {'status': 'passed_targeted_tests_lint_format', 'unique_tests_passed': 4,
              'test_runs_passed': 5, 'stages': stages, 'binaries': binaries,
              'source_hashes': manifest['source_hashes'], 'fixture_hashes': manifest['fixture_hashes'],
              'selection_reason': manifest['selection_reason'], 'full_t_gate': 'deferred to T06-W12',
              'recorded_at': datetime.now(timezone.utc).isoformat()}
    (args.reports / 'local-verification.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')


if __name__ == '__main__':
    main()
