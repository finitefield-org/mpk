"""Check immutable completed layout streams while the original Rust run lives.

Every size0..70 is checked once. Inline0/3/70 may supply a stream early; later
matrix files must be literally identical. No individual checker is restarted.
"""
from pathlib import Path
import datetime
import hashlib
import json
import subprocess
import time

ROOT=Path(__file__).resolve().parent


def seal(path):
    raw=Path(path).read_bytes()
    return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}


prior_path=ROOT.parent/'core-interfaces/paired/result.json'
prior=json.loads(prior_path.read_bytes())
binary=prior['go_binary']
assert seal(binary['path'])==binary
for row in prior['go_production_sources']:
    assert seal(row['path'])==row
assert json.loads((ROOT/'inline/result.json').read_bytes())['status']=='passed'
out=ROOT/'incremental-go'
out.mkdir(exist_ok=False)
driver=out/'driver.py'
driver.write_bytes(Path(__file__).read_bytes())
record={'status':'running','actual_driver_prelaunch':seal(driver),
        'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'go_binary':binary,'go_production_sources':prior['go_production_sources'],
        'actual_unchanged_Go_build_evidence':seal(prior_path),
        'Rust_matrix_execution_path':str(ROOT/'final/result.json'),
        'completed_inline_execution':seal(ROOT/'inline/result.json'),
        'cases':[],'scope':'Only complete actual layout streams0..70; no counterexample, source/canonical context, full-suite or release claim.'}


def save():
    (out/'result.json').write_text(json.dumps(record,indent=2)+'\n')


def completed(n):
    candidates=[ROOT/f'final/certificates/matrix-N{n}.mpcert']
    if n in {0,3,70}:
        candidates.append(ROOT/f'inline/certificates/layout-N{n}.mpcert')
    for path in candidates:
        report=path.with_suffix('.rust.json')
        if not report.is_file():
            continue
        try:
            value=json.loads(report.read_bytes())
        except json.JSONDecodeError:
            # The original writer may not yet have closed its new report.
            continue
        assert value['verdict']=='accepted' and value['axiom_count']==0
        assert value['module']==f'Mpk.Go.Practical.ErrorLayout.N{n}'
        return path,report,value
    return None


save()
try:
    pending=[70,*range(70)]
    while pending:
        ready=next(((n,completed(n)) for n in pending if completed(n) is not None),None)
        if ready is None:
            state=json.loads((ROOT/'final/result.json').read_bytes())
            if state['status']!='running':
                raise AssertionError(('original Rust run is terminal with missing matrix streams',state['status'],pending))
            time.sleep(5)
            continue
        n,(certificate,rust_path,rust)=ready
        before,rust_before=seal(certificate),seal(rust_path)
        command=[binary['path'],'verify',str(certificate)]
        start=time.monotonic()
        result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
        stdout,stderr=out/f'N{n}.stdout.log',out/f'N{n}.stderr.log'
        stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr)
        go=json.loads(result.stdout)
        record['cases'].append({'case_count':n,'certificate':before,'rust_report':rust_before,
            'command':command,'exit_code':result.returncode,'seconds':time.monotonic()-start,
            'stdout':seal(stdout),'stderr':seal(stderr),'rust_observation':rust,'go_observation':go})
        save()
        assert result.returncode==0 and not result.stderr and go['verdict']=='accepted'
        assert rust['hashes']==go['hashes'] and rust['declaration_count']==go['declaration_count']
        assert go['hashes']['axiom_report']=='0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5'
        assert seal(certificate)==before and seal(rust_path)==rust_before
        pending.remove(n)
        print(json.dumps({'case_count':n,'verdict':'accepted','seconds':record['cases'][-1]['seconds']}),flush=True)
    aliases=[]
    for row in record['cases']:
        n=row['case_count']
        matrix=ROOT/f'final/certificates/matrix-N{n}.mpcert'
        while not matrix.with_suffix('.rust.json').is_file():
            state=json.loads((ROOT/'final/result.json').read_bytes())
            assert state['status']=='running'
            time.sleep(5)
        assert matrix.read_bytes()==Path(row['certificate']['path']).read_bytes()
        assert matrix.with_suffix('.rust.json').read_bytes()==Path(row['rust_report']['path']).read_bytes()
        aliases.append({'case_count':n,'matrix_stream':seal(matrix),'matrix_report':seal(matrix.with_suffix('.rust.json')),
                        'original_checked_stream':row['certificate'],'whole_original_bytes_and_reports_equal':True})
    assert seal(binary['path'])==binary
    for source in prior['go_production_sources']:
        assert seal(source['path'])==source
    record.update(status='layout_matrix_passed',distinct_full_streams=71,zero_axiom_acceptances=71,
                  literal_matrix_correspondence=aliases,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    save()
except BaseException:
    record['status']='failed'
    save()
    raise
