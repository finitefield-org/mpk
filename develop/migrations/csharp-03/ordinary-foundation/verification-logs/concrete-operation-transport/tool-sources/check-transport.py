import concurrent.futures
import json
import os
import subprocess
import time
from datetime import datetime, timezone
from hashlib import sha256
from pathlib import Path

source = Path('/tmp/mpk-w09-concrete-operation-proofs/transport-attempt-2/certificates')
out = Path('/tmp/mpk-w09-concrete-operation-proofs/transport-checks')
out.mkdir(parents=True, exist_ok=True)
binaries = {'go': Path('/tmp/mpk-w09-default-closed-proofs/go-checker'),
            'rust': Path('/Users/kazuyoshitoshiya/mpk/target/release/mpk')}
digests = {'go': 'd631f01eb0beaab7b83bebd53e9f66b02f3a6a8c04e173b3af0acb48b9b305a2',
           'rust': 'b5207abd1d92acc1fd8b92a995ed700e1a2d929e6254ffe20b6f9f5d868bac91'}
for backend, path in binaries.items():
    assert sha256(path.read_bytes()).hexdigest() == digests[backend]
cases = sorted(p for p in source.glob('*.hex') if not p.stem.endswith('-wrong'))
assert len(cases) == 45
proofs = sum(len(json.loads(p.with_suffix('.json').read_text())['proofs']) for p in cases)
assert proofs == 436
state = dict(status='running', supervisor_pid=os.getpid(), proofs_supplied=proofs,
             source_contexts=45, launches={}, stages=[], binaries=digests,
             application_proofs_pending=987, concrete_operations_pending=31,
             full_t_gate='deferred to T06-W12',
             selection_reason='Check the exact 45 original-source complete concrete-operation proof certificates with both unchanged binaries, compare hashes and counts, require zero axioms, and reject per-certificate hash corruption plus a well-typed wrong normal implementation at core checking; the definition-only certificate accepts and the supplied proof must reject. Empty concrete-operation groups have zero supplied theorems and retain all their other obligations; this is not complete application acceptance.')

def persist():
    path = out / 'status.tmp'
    path.write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')
    os.replace(path, out / 'status.json')

def run(backend, stem, data):
    path = out / (stem + '.mpcert')
    command = [str(binaries[backend])] + (['check'] if backend == 'rust' else []) + [str(path)]
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    with (out / (stem + '-' + backend + '.json')).open('wb') as stdout, (out / (stem + '-' + backend + '.stderr')).open('wb') as stderr:
        process = subprocess.Popen(command, stdout=stdout, stderr=stderr)
        return process, dict(backend=backend, command=command, process_pid=process.pid,
                             started_at=started, input_file_sha256=sha256(data).hexdigest(),
                             binary_sha256=digests[backend]), start

persist()
for case in cases:
    data = bytes.fromhex(case.read_text())
    jobs = [(case.stem + '-positive', data, 'positive'),
            (case.stem + '-hash', data[:-1] + bytes([data[-1] ^ 1]), 'hash')]
    wrong = source / (case.stem + '-wrong.hex')
    if wrong.exists():
        jobs.append((case.stem + '-wrong', bytes.fromhex(wrong.read_text()), 'wrong'))
    for stem, contents, kind in jobs:
        (out / (stem + '.mpcert')).write_bytes(contents)
        active = {b: run(b, stem, contents) for b in binaries}
        state['launches'] = {b: v[1] for b, v in active.items()}
        state['stage'] = stem
        persist()
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            exits = dict(zip(active, pool.map(lambda b: active[b][0].wait(), active)))
        reports = {}
        for backend, (process, launch, start) in active.items():
            stdout = (out / (stem + '-' + backend + '.json')).read_bytes()
            stderr = (out / (stem + '-' + backend + '.stderr')).read_bytes()
            report = json.loads(stdout)
            assert exits[backend] == (0 if kind == 'positive' else 1), (stem, backend, exits[backend])
            assert report['verdict'] == ('accepted' if kind == 'positive' else 'rejected'), (stem, backend)
            certificate_hash = sha256(b'MPK-MODULE-CERT-0.1\0' + contents).hexdigest()
            if kind == 'positive':
                if backend == 'go':
                    core = report['report']
                    assert core['AxiomCount'] == 0
                    hashes = {key: bytes(core[field]).hex() for key, field in [('export', 'ExportHash'), ('axiom_report', 'AxiomReportHash'), ('certificate', 'CertificateHash')]}
                    counts = (core['Module'], core['DeclarationCount'], core['AxiomCount'])
                else:
                    assert report['axiom_count'] == 0
                    hashes = report['hashes']
                    counts = (report['module'], report['declaration_count'], report['axiom_count'])
                assert hashes['certificate'] == certificate_hash
                reports[backend] = (hashes, counts)
            else:
                assert (report['certificate'] if backend == 'go' else report['hashes']['certificate']) == certificate_hash
                if kind == 'wrong':
                    assert (report.get('error_kind') == 'core_check' if backend == 'go' else report.get('error_code') == 'KERNEL_CORE_CHECK')
            row = dict(launch, case=case.stem, kind=kind, exit_code=exits[backend],
                       finished_at=datetime.now(timezone.utc).isoformat(),
                       elapsed_seconds=round(time.monotonic() - start, 3),
                       certificate_sha256=certificate_hash,
                       report_sha256=sha256(stdout).hexdigest(), stderr_sha256=sha256(stderr).hexdigest())
            state['stages'].append(row)
            persist()
        if kind == 'positive':
            assert reports['go'] == reports['rust'], stem
        print(stem, 'passed both checkers', flush=True)
state.update(status='passed', finished_at=datetime.now(timezone.utc).isoformat(), launches={})
assert len(state['stages']) == 182
persist()
(out / 'verification.json').write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')
