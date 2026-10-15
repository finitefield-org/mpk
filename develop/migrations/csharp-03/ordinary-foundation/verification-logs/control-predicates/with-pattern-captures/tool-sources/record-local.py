"""Record completed executor observations; do not infer exits from log text."""
import argparse
from hashlib import sha256
import json
from pathlib import Path
import re


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--reports', type=Path, required=True)
    args = parser.parse_args()
    root = args.reports
    manifest = json.loads((root / 'source-manifest.json').read_bytes())
    pins = args.repo / 'develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-pattern-captures'
    for mapping, base in ((manifest['source_hashes'], args.repo), (manifest['fixture_hashes'], pins)):
        for name, digest in mapping.items():
            assert sha256((base / name).read_bytes()).hexdigest() == digest, name
    observations = json.loads((root / 'observed-exits.json').read_bytes())
    jobs = [
        ('predicate-regressions', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc',
                                  'csharp_03_t06_w09_control_predicates_', '--', '--nocapture', '--test-threads=1'], 4),
        ('units', ['test', '-p', 'mpk-vc', '--lib', 'pattern_', '--', '--nocapture', '--test-threads=1'], 3),
        ('pinned-captures', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc',
                            'csharp_03_t06_w09_control_predicates_with_pattern_captures_original_sources',
                            '--', '--nocapture', '--test-threads=1'], 1),
        ('clippy', ['clippy', '-p', 'mpk-vc', '--lib', '--test', 'csharp_practical_vc', '--', '-D', 'warnings'], 0),
        ('format', ['fmt', '-p', 'mpk-vc', '--', '--check'], 0),
    ]
    stages, binaries = [], {}
    for label, arguments, count in jobs:
        observation = observations[label]
        assert observation['exit_code'] == 0
        data = (root / f'{label}.log').read_bytes()
        text = data.decode()
        if count:
            assert f'test result: ok. {count} passed' in text
            for path in re.findall(r'Running [^\n]*\(([^\n]+)\)', text):
                binaries[path] = sha256(Path(path).read_bytes()).hexdigest()
        stages.append({'stage': label, 'reproduction_command': ['cargo', *arguments],
                       **observation, 'tests_passed': count, 'log_sha256': sha256(data).hexdigest()})
    result = {'status': 'passed_targeted_tests_lint_format', 'unique_tests_passed': 7,
              'test_runs_passed': 8, 'stages': stages, 'binaries': binaries,
              'source_hashes': manifest['source_hashes'], 'fixture_hashes': manifest['fixture_hashes'],
              'selection_reason': manifest['selection_reason'],
              'initial_compile_attempt': {'status': 'failed_compilation',
                  'log_sha256': sha256((root / 'compile-attempt-1.log').read_bytes()).hexdigest(),
                  'reason': 'Test flag shadowed CapturedInputSet; rename only the test flag before any semantic run'},
              'full_t_gate': 'deferred to T06-W12'}
    (root / 'local-verification.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print('RECORDED', len(stages), 'stages;', 7, 'unique tests')


if __name__ == '__main__':
    main()
