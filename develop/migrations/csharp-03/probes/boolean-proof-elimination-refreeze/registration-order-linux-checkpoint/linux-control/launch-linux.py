import datetime
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

commit = sys.argv[1]
assert len(commit) == 40 and all(c in '0123456789abcdef' for c in commit)
control = Path('/root/mpk-w09-bool-cases-' + commit[:8] + '-control')
repo = Path('/root/mpk-w09-bool-cases-' + commit[:8])
reports = repo.with_name(repo.name + '-linux')
target = repo.with_name(repo.name + '-target')
parent = Path('/root/mpk-w09-context-application-f02dcb32')
pins = json.loads((control / 'control-hashes.json').read_text())
for name, digest in pins.items():
    assert hashlib.sha256((control / name).read_bytes()).hexdigest() == digest, name


def git(*args, cwd=parent):
    return subprocess.check_output(['/usr/bin/git', *args], cwd=cwd, text=True).strip()


git('fetch', 'https://github.com/finitefield-org/mpk.git', 'codex/w09-packed-pattern-proofs')
assert git('rev-parse', 'FETCH_HEAD') == commit
assert not repo.exists() and not reports.exists() and not target.exists()
git('worktree', 'add', '--detach', '--no-checkout', str(repo), commit)
# Preserve all exact selected public source/input paths and relevant fixtures.
# Omit large unrelated immutable ordinary evidence from this read-only checkout.
git('sparse-checkout', 'set', '--cone', 'crates', 'go-tools/mpk-checker-ref',
    'fixtures', 'develop/probes', 'develop/specs', 'examples', 'rust-tools', 'fuzz',
    'develop/migrations/csharp-03/probes/boolean-proof-elimination-logs/producer',
    'develop/migrations/csharp-03/ordinary-foundation/verification-logs/source-invariants/deep-equation-diagnostic', cwd=repo)
assert git('rev-parse', 'HEAD', cwd=repo) == commit
assert not git('status', '--porcelain', cwd=repo)
manifest = json.loads((control / 'source-manifest.json').read_text())
files = 0
for group in ('source_hashes', 'fixture_hashes'):
    for name, digest in manifest[group].items():
        data = (repo / name).read_bytes()
        assert hashlib.sha256(data).hexdigest() == digest, name
        assert subprocess.check_output(['/usr/bin/git', 'show', commit + ':' + name], cwd=repo) == data
        files += 1
# This cache's earlier build/diagnostic is terminal; Cargo revalidates its inputs.
base = Path('/root/mpk-w09-bool-cases-97559877-target')
subprocess.run(['cp', '-a', '--reflink=auto', str(base), str(target)], check=True)
reports.mkdir()
command = ['python3', str(control / 'run-linux.py'), '--repo', str(repo),
           '--reports', str(reports), '--target', str(target), '--control', str(control), '--commit', commit]
with (reports / 'supervisor.log.txt').open('wb') as log:
    process = subprocess.Popen(command, cwd=repo, stdin=subprocess.DEVNULL, stdout=log,
                               stderr=subprocess.STDOUT, start_new_session=True)
launch = dict(commit=commit, supervisor_pid=process.pid, command=command,
              source=str(repo), reports=str(reports), target=str(target), control=str(control),
              git_blob_files_verified=files, source_clean=True, public_git_source=True,
              control_hashes=pins, started_at=datetime.datetime.now(datetime.timezone.utc).isoformat())
(reports / 'launch.json').write_text(json.dumps(launch, indent=2, sort_keys=True) + '\n')
print(json.dumps(launch))
