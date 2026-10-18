"""Repeat actual capture into a new directory, preserving every previous run."""
import argparse
import datetime
import hashlib
import json
import subprocess
import time
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('output_directory', type=Path)
args = parser.parse_args()
repo = Path(__file__).resolve().parents[6]
evidence = Path(__file__).resolve().parent
inputs = repo / 'develop/migrations/csharp-03/control-emission/boolean-refreeze/control-requests.json'
output = args.output_directory.resolve()
output.mkdir(parents=True, exist_ok=False)
command = json.loads((evidence / 'native-capture-result.json').read_text())['command']
command[command.index('--mount') + 1] = f'type=bind,src={repo},dst=/workspace,readonly'
started = time.monotonic()
with inputs.open('rb') as source, (output / 'stdout.json').open('xb') as stdout, (output / 'stderr.log').open('xb') as stderr:
    result = subprocess.run(command, cwd=repo, stdin=source, stdout=stdout, stderr=stderr)
record = {
    'command': command, 'exit_code': result.returncode,
    'wall_seconds': time.monotonic() - started,
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'scope': 'Fresh private CSharp capture in local Linux-amd64 emulation; not native installed-release evidence',
    'input': str(inputs.relative_to(repo)),
    'input_sha256': hashlib.sha256(inputs.read_bytes()).hexdigest(),
}
for stream, name in [('stdout', 'stdout.json'), ('stderr', 'stderr.log')]:
    record[stream + '_sha256'] = hashlib.sha256((output / name).read_bytes()).hexdigest()
(output / 'result.json').write_text(json.dumps(record, indent=2, sort_keys=True) + '\n')
print(json.dumps(record, indent=2))
raise SystemExit(result.returncode)
