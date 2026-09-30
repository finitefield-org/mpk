"""Check changed source-condition bytes and re-audit exact unchanged predecessor bytes.

Retained exits come from a completed, pinned predecessor receipt. Equality of
both the certificate input and checker binary is required before reuse. Source-condition
definitions and observations do not discharge an application proof.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
from hashlib import sha256
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sys


def sha(data):
    return sha256(data).hexdigest()


def write(path, data):
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(data, indent=2, sort_keys=True) + '\n')
    os.replace(temporary, path)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--pins', type=Path, required=True)
    parser.add_argument('--reports', type=Path, required=True)
    parser.add_argument('--prior-reports', type=Path, required=True)
    parser.add_argument('--rust', type=Path, required=True)
    parser.add_argument('--go', type=Path, required=True)
    parser.add_argument('--audit-only', action='store_true')
    args = parser.parse_args()
    foundation = args.repo / 'develop/migrations/csharp-03/ordinary-foundation'
    harness = foundation / 'verification-logs/control-predicates/tool-sources/check.py'
    spec = importlib.util.spec_from_file_location('predicate_check', harness)
    checker = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(checker)
    checker.CASES = tuple(case for case in checker.CASES if case != 'measures')
    prior_pins = foundation / 'control-predicates/with-pattern-observations'
    prior_receipt_path = args.prior_reports / 'verification.json'
    prior = json.loads(prior_receipt_path.read_bytes())
    assert prior['status'] == 'passed' and prior['stage_count'] == 72
    for backend in ('go', 'rust'):
        assert sha(getattr(args, backend).read_bytes()) == prior['binaries'][backend]['sha256']
    args.reports.mkdir(parents=True, exist_ok=True)
    certificates = {}
    for case in checker.CASES:
        data = bytes.fromhex((args.pins / f'{case}.hex').read_text())
        metadata = json.loads((args.pins / f'{case}.json').read_bytes())
        assert checker.cert_hash(data) == metadata['certificate_sha256']
        assert metadata['application_scope_pending'] is True
        certificates[case] = data
    stages_path = args.reports / 'stages.json'
    stages = json.loads(stages_path.read_bytes()) if stages_path.exists() else {}
    reuse_path = args.reports / 'retained-stages.json'
    retained = json.loads(reuse_path.read_bytes()) if reuse_path.exists() else {}
    if not args.audit_only:
        status_path = args.reports / 'runner-status.json'
        if status_path.exists():
            previous = json.loads(status_path.read_bytes())
            if previous['status'] == 'running' and previous['supervisor_pid'] != os.getpid():
                try:
                    os.kill(previous['supervisor_pid'], 0)
                except ProcessLookupError:
                    pass
                else:
                    raise RuntimeError('A source-condition checker supervisor is still running')
        status = {'status': 'running', 'supervisor_pid': os.getpid(),
                  'started_at': datetime.now(timezone.utc).isoformat()}
        write(status_path, status)
        jobs = []
        for case, data in certificates.items():
            unchanged = data == bytes.fromhex((prior_pins / f'{case}.hex').read_text())
            for mutation in ('positive', 'hash'):
                for backend in ('go', 'rust'):
                    stem = f'{case}-{mutation}-{backend}'
                    if stem in stages:
                        continue
                    if unchanged:
                        row = prior['stages'][stem]
                        candidate = data if mutation == 'positive' else data[:-1] + bytes([data[-1] ^ 1])
                        assert (args.prior_reports / f'{stem}.mpcert').read_bytes() == candidate
                        assert row['certificate_sha256'] == checker.cert_hash(candidate)
                        for suffix, key in [('json', 'report_sha256'), ('stderr', 'stderr_sha256')]:
                            assert sha((args.prior_reports / f'{stem}.{suffix}').read_bytes()) == row[key]
                        for suffix in ('mpcert', 'json', 'stderr'):
                            shutil.copyfile(args.prior_reports / f'{stem}.{suffix}', args.reports / f'{stem}.{suffix}')
                        stages[stem] = row
                        retained[stem] = {'receipt_sha256': sha(prior_receipt_path.read_bytes()),
                                          'source_reports': str(args.prior_reports),
                                          'reason': 'Identical certificate and checker bytes; observed predecessor exit retained'}
                    else:
                        jobs.append((case, data, mutation, backend))
        write(stages_path, stages)
        write(reuse_path, retained)
        print('RETAINED', len(retained), 'NEW', len(jobs), flush=True)
        with ThreadPoolExecutor(max_workers=2) as pool:
            futures = [pool.submit(checker.run, args, *job) for job in jobs]
            for future in as_completed(futures):
                stem, row = future.result()
                stages[stem] = row
                write(stages_path, stages)
                print('PASS', stem, row['elapsed_seconds'], flush=True)
    sys.argv = [str(harness), '--pins', str(args.pins), '--reports', str(args.reports),
                '--go', str(args.go), '--rust', str(args.rust), '--audit-only']
    checker.main()
    receipt = json.loads((args.reports / 'verification.json').read_bytes())
    assert set(retained) <= set(receipt['stages'])
    receipt['retained_stages'] = retained
    receipt['newly_executed_stages'] = len(stages) - len(retained)
    receipt['retained_stage_count'] = len(retained)
    write(args.reports / 'verification.json', receipt)
    write(args.reports / 'runner-status.json',
          {'status': 'passed', 'supervisor_pid': os.getpid(), 'stage_count': len(stages),
           'retained_stage_count': len(retained), 'newly_executed_stages': len(stages) - len(retained),
           'finished_at': datetime.now(timezone.utc).isoformat()})


if __name__ == '__main__':
    main()
