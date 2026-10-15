"""Retain completed checker stages if an interactive runner is interrupted.

Wait for that runner and every live checker to exit before resuming only stages
whose process exits were not recorded. Never infer an exit code from a verdict.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from types import SimpleNamespace

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--runner-pid', type=int, required=True)
parser.add_argument('--pins', type=Path, required=True)
parser.add_argument('--reports', type=Path, required=True)
parser.add_argument('--rust', type=Path, required=True)
parser.add_argument('--go', type=Path, required=True)
args = parser.parse_args()
spec = importlib.util.spec_from_file_location('integrated_check', Path(__file__).with_name('check.py'))
wrapper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(wrapper)
checker = wrapper.checker


def live():
    processes = subprocess.check_output(['ps', '-axo', 'pid=,command='], text=True).splitlines()
    for row in processes:
        fields = row.strip().split(None, 1)
        if len(fields) != 2:
            continue
        pid, command = fields
        if int(pid) == args.runner_pid:
            return True
        if str(args.reports) in command and (
            command.startswith(str(args.go) + ' ') or command.startswith(str(args.rust) + ' ')
        ):
            return True
    return False


while live():
    time.sleep(15)
state_path = args.reports / 'stages.json'
stages = json.loads(state_path.read_bytes()) if state_path.exists() else {}
pending = [(case, mutation, backend) for case in checker.CASES
           for mutation in ('positive', 'hash') for backend in ('go', 'rust')
           if f'{case}-{mutation}-{backend}' not in stages]
print('RETAINED', len(stages), 'PENDING', len(pending), flush=True)
with ThreadPoolExecutor(max_workers=2) as pool:
    futures = [pool.submit(checker.run, args, case,
                           bytes.fromhex((args.pins / f'{case}.hex').read_text()), mutation, backend)
               for case, mutation, backend in pending]
    for future in as_completed(futures):
        stem, row = future.result()
        stages[stem] = row
        temporary = state_path.with_suffix('.json.tmp')
        temporary.write_text(json.dumps(stages, indent=2, sort_keys=True) + '\n')
        os.replace(temporary, state_path)
        print('PASS', stem, row['elapsed_seconds'], flush=True)
sys.argv = [str(Path(__file__).with_name('check.py')), '--pins', str(args.pins),
            '--reports', str(args.reports), '--rust', str(args.rust), '--go', str(args.go), '--audit-only']
checker.main()
