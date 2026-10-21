"""Compare the complete actual inputs in the unchanged independent Go core."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess
import sys
import time

OUT=Path(__file__).resolve().parent
PRIOR=OUT.parent/'core-interfaces/paired/result.json'
epoch=sys.argv[1]

def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {'path':str(path),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}

prior=json.loads(PRIOR.read_bytes());binary=prior['go_binary']
assert seal(binary['path'])==binary and all(seal(row['path'])==row for row in prior['go_production_sources'])
execution=OUT/'epochs'/epoch/'selected/result.json'
launch=execution.read_bytes();source=json.loads(launch)
foundation_execution=OUT/'epochs/closed-complement/foundation/result.json'
foundation_launch=foundation_execution.read_bytes();foundation=json.loads(foundation_launch)
assert foundation['status']=='passed'and foundation['whole_source_inputs_unchanged']
assert [(row['path'],row['sha256'],row['bytes'])for row in source['source_inputs']]==[(row['path'],row['sha256'],row['bytes'])for row in foundation['source_inputs']]
assert source['status']in {'running','passed'}
assert all(hashlib.sha256(Path(row['path']).read_bytes()).hexdigest()==row['sha256']for row in source['source_inputs'])
run=OUT/'paired-complete';run.mkdir(exist_ok=False)
driver=run/'check-paired.py';driver.write_bytes(Path(__file__).read_bytes())
launch_path=run/'Rust-execution-at-launch.json';launch_path.write_bytes(launch)
foundation_path=run/'Rust-foundation-execution.json';foundation_path.write_bytes(foundation_launch)
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'Rust_execution_at_launch':seal(launch_path),'Rust_foundation_execution':seal(foundation_path),'Rust_foundation_original':seal(foundation_execution),'actual_driver_prelaunch':seal(driver),'go_binary':binary,'go_production_sources':prior['go_production_sources'],'unchanged_Go_build_receipt':seal(PRIOR),'cases':[],'source_program_or_registered_context_proof':False,'native_release_acceptance':False,'full_T_gate':'deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh'}

def save():
    (run/'result.json').write_text(json.dumps(record,indent=2)+'\n')

positive={'array-equality-foundation','array-equality-honest-extra'}
seen=set();save()
try:
    while True:
        candidates=sorted([*(execution.parent/'candidates').glob('*.mpcert'),*(foundation_execution.parent/'candidates').glob('*.mpcert')]);assert len({path.stem for path in candidates})==len(candidates)
        pending=sorted((path for path in candidates if path.stem not in seen and path.with_suffix('.rust.json').exists()),key=lambda path:(path.stem not in positive,path.stem))
        if not pending:
            state=json.loads(execution.read_bytes())
            if state['status']!='running':
                completion=run/'Rust-execution-at-completion.json';completion.write_bytes(execution.read_bytes())
                record['Rust_execution']=seal(execution);record['Rust_execution_at_completion']=seal(completion);save()
                assert state['status']=='passed' and state['whole_source_inputs_unchanged']
                break
            time.sleep(0.25);continue
        path=pending[0]
        try:rust=json.loads(path.with_suffix('.rust.json').read_bytes())
        except json.JSONDecodeError:
            time.sleep(0.05);continue
        command=[binary['path'],'verify',str(path)];start=time.monotonic()
        result=subprocess.run(command,capture_output=True,check=False)
        stdout=run/(path.stem+'.stdout.log');stderr=run/(path.stem+'.stderr.log')
        stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr);go=json.loads(result.stdout)
        accepted=path.stem in positive
        record['cases'].append({'name':path.stem,'expected_acceptance':accepted,'input':seal(path),'Rust_report':seal(path.with_suffix('.rust.json')),'command':command,'exit_code':result.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr),'Rust':rust,'Go':go});save();seen.add(path.stem)
        assert not result.stderr
        if accepted:
            assert result.returncode==0 and rust['verdict']==go['verdict']=='accepted'
            assert rust['axiom_count']==0 and not rust['axiom_report']['entries']
            assert rust['hashes']==go['hashes'] and rust['declaration_count']==go['declaration_count']
            assert go['hashes']['axiom_report']=='0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5'
        else:
            assert path.stem.startswith(('false-','premise-','definition-'))
            assert result.returncode!=0 and rust['verdict']==go['verdict']=='rejected'
            detail=json.loads(rust['error_detail']);assert detail['code']=='CORE_TYPE_MISMATCH' and go['error_kind']=='core_check'
            if detail['details']['kind']=='check_type_mismatch':
                a,b=[detail['details'][key]for key in ['inferred_kind','expected_kind']]
                assert re.fullmatch(r'term [0-9]+ inferred '+re.escape(a)+' but expected '+re.escape(b),go['error_detail'])
            else:assert detail['details']['kind']=='lambda_domain_mismatch' and go['error_detail']=='lambda domain mismatch'
    inventory=json.loads((execution.parent/'candidates/control-inventory.json').read_bytes())
    total=inventory['complete_candidates'];negative=total-2
    assert len(seen)==len(candidates)==total and positive<=seen
    assert sum(path.stem.startswith('false-')for path in candidates)==inventory['altered_conclusions']==246
    assert sum(path.stem.startswith('premise-')for path in candidates)==inventory['actual_typed_witness_domains']>60
    assert sum(path.stem.startswith('definition-')for path in candidates)==inventory['typed_wrong_definitions']==47
    assert inventory['zero_axiom_acceptances']==2
    assert seal(binary['path'])==binary and all(seal(row['path'])==row for row in prior['go_production_sources'])
    assert all(hashlib.sha256(Path(row['path']).read_bytes()).hexdigest()==row['sha256']for row in source['source_inputs'])
    record.update(status='passed',same_byte_inputs=total,zero_axiom_acceptances=2,genuine_core_type_mismatches=negative,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save()
    print(json.dumps({key:record[key]for key in ['status','same_byte_inputs','zero_axiom_acceptances','genuine_core_type_mismatches']}))
except BaseException:
    record['status']='failed';save();raise
