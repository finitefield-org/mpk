"""Inspect the exact current positive and damaged inputs in the independent Go core."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess,time
OUT=Path(__file__).resolve().parent
PRIOR=OUT.parent/'core-interfaces/paired/result.json'
EPOCH=OUT/'epochs/complete-harness'
def seal(path):
    raw=Path(path).read_bytes();return {'path':str(path),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
prior=json.loads(PRIOR.read_bytes());binary=prior['go_binary']
assert seal(binary['path'])==binary and all(seal(r['path'])==r for r in prior['go_production_sources'])
modes=['foundation','alphabet','definitions','laws']
executions={m:EPOCH/m/'result.json'for m in modes}
foundation=json.loads(executions['foundation'].read_bytes())
assert foundation['status']=='passed'and foundation['whole_source_inputs_unchanged']
source=foundation['source_inputs']
run=OUT/'paired';run.mkdir(exist_ok=False);driver=run/'check-paired.py';driver.write_bytes(Path(__file__).read_bytes())
launches={}
for mode,path in executions.items():
    raw=path.read_bytes();row=json.loads(raw);assert row['status']in {'running','passed'}and row['source_inputs']==source
    copied=run/f'Rust-{mode}-at-launch.json';copied.write_bytes(raw);launches[mode]=seal(copied)
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch':seal(driver),'Rust_executions_at_launch':launches,'go_binary':binary,'go_production_sources':prior['go_production_sources'],'unchanged_Go_build_receipt':seal(PRIOR),'cases':[],'source_program_or_registered_context_proof':False,'native_release_acceptance':False,'full_T_gate':'deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh'}
def save():(run/'result.json').write_text(json.dumps(record,indent=2)+'\n')
positive={'descriptor-foundation','descriptor-honest-extra'};seen=set();save()
try:
    while True:
        candidates=sorted(path for mode in modes for path in (EPOCH/mode/'candidates').glob('*.mpcert'))
        assert len({p.stem for p in candidates})==len(candidates)
        pending=sorted((p for p in candidates if p.stem not in seen and p.with_suffix('.rust.json').exists()),key=lambda p:(p.stem not in positive,p.stem))
        if not pending:
            states={m:json.loads(p.read_bytes())for m,p in executions.items()}
            if all(r['status']!='running'for r in states.values()):
                record['Rust_executions_at_completion']={}
                for mode,path in executions.items():
                    row=states[mode];assert row['status']=='passed'and row['whole_source_inputs_unchanged']and row['source_inputs']==source
                    copy=run/f'Rust-{mode}-at-completion.json';copy.write_bytes(path.read_bytes());record['Rust_executions_at_completion'][mode]={'original':seal(path),'snapshot':seal(copy)}
                save();break
            time.sleep(0.25);continue
        path=pending[0]
        try:rust=json.loads(path.with_suffix('.rust.json').read_bytes())
        except json.JSONDecodeError:time.sleep(0.05);continue
        command=[binary['path'],'verify',str(path)];start=time.monotonic();p=subprocess.run(command,capture_output=True,check=False)
        stdout=run/(path.stem+'.stdout.log');stderr=run/(path.stem+'.stderr.log');stdout.write_bytes(p.stdout);stderr.write_bytes(p.stderr);go=json.loads(p.stdout)
        accepted=path.stem in positive
        record['cases'].append({'name':path.stem,'expected_acceptance':accepted,'input':seal(path),'Rust_report':seal(path.with_suffix('.rust.json')),'command':command,'exit_code':p.returncode,'seconds':time.monotonic()-start,'stdout':seal(stdout),'stderr':seal(stderr),'Rust':rust,'Go':go});save();seen.add(path.stem);assert not p.stderr
        if accepted:
            assert p.returncode==0 and rust['verdict']==go['verdict']=='accepted'and rust['hashes']==go['hashes']and rust['declaration_count']==go['declaration_count']
            assert rust['axiom_count']==0 and not rust['axiom_report']['entries']and go['hashes']['axiom_report']=='0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5'
        else:
            assert p.returncode!=0 and rust['verdict']==go['verdict']=='rejected'and go['error_kind']=='core_check'
            detail=json.loads(rust['error_detail']);assert detail['code']=='CORE_TYPE_MISMATCH'
            if detail['details']['kind']=='check_type_mismatch':
                a,b=[detail['details'][k]for k in ['inferred_kind','expected_kind']];assert re.fullmatch(r'term [0-9]+ inferred '+re.escape(a)+' but expected '+re.escape(b),go['error_detail'])
            else:assert detail['details']['kind']=='lambda_domain_mismatch'and go['error_detail']=='lambda domain mismatch'
    inventory=json.loads((EPOCH/'laws/candidates/control-inventory.json').read_bytes());total=inventory['complete_candidates'];negative=total-2
    assert len(seen)==len(candidates)==total and positive<=seen
    assert sum(p.stem.startswith('false-')for p in candidates)==inventory['altered_conclusions']==113
    assert sum(p.stem.startswith('premise-')for p in candidates)==inventory['actual_typed_witness_domains']>30
    assert sum(p.stem.startswith('definition-')for p in candidates)==inventory['typed_wrong_definitions']==28
    assert seal(binary['path'])==binary and all(seal(r['path'])==r for r in prior['go_production_sources'])
    assert all(hashlib.sha256(Path(r['path']).read_bytes()).hexdigest()==r['sha256']for r in source)
    record.update(status='passed',same_byte_inputs=total,zero_axiom_acceptances=2,genuine_core_type_mismatches=negative,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({k:record[k]for k in ['status','same_byte_inputs','zero_axiom_acceptances','genuine_core_type_mismatches']}))
except BaseException:
    record['status']='failed';save();raise
