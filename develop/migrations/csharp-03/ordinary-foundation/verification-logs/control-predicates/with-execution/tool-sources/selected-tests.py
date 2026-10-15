"""Run affected parser, control/data/exception rejection and predicate tests."""
from hashlib import sha256
import json
import os
from pathlib import Path
import subprocess
import time

ROOT = Path('/tmp/mpk-w09-control-predicates-integrated')
REPO = Path('/Users/kazuyoshitoshiya/.codex/worktrees/w09-default-use/mpk')
env = os.environ.copy()
env.update(CARGO_TARGET_DIR='/tmp/mpk-w09-optimized-target', CARGO_BUILD_JOBS='2',
           CARGO_PROFILE_TEST_OPT_LEVEL='2', CARGO_PROFILE_TEST_DEBUG='0',
           CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true', CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true',
           CARGO_INCREMENTAL='0')
jobs = [
    ('measures', ['test', '-p', 'mpk-vc', '--lib', 'control_predicates::tests', '--', '--nocapture', '--test-threads=1']),
    ('parser', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w01_scopes_normalization_and_parser_fuzz', '--', '--nocapture', '--test-threads=1']),
    ('data-handoffs', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w03_', '--', '--nocapture', '--test-threads=1']),
    ('control-handoffs', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w04_', '--', '--nocapture', '--test-threads=1']),
    ('exception-handoffs', ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w05_original_exception_handlers_and_mutations', '--', '--nocapture', '--test-threads=1']),
    ('format', ['fmt', '-p', 'mpk-vc', '--', '--check']),
]
stages = []
for label, args in jobs:
    log = ROOT / f'{label}.log'
    start = time.monotonic()
    with log.open('wb') as out:
        code = subprocess.call(['cargo', *args], cwd=REPO, env=env, stdin=subprocess.DEVNULL,
                               stdout=out, stderr=subprocess.STDOUT)
    stages.append({'stage': label, 'command': ['cargo', *args], 'exit_code': code,
                   'elapsed_seconds': round(time.monotonic() - start, 3),
                   'log_sha256': sha256(log.read_bytes()).hexdigest()})
    (ROOT / 'selected-stages.json').write_text(json.dumps(stages, indent=2, sort_keys=True)+'\n')
    print(label, code, flush=True)
    assert code == 0, (label, code)
