#!/usr/bin/env python3
"""Replay actual ordinary word definitions/proofs and pinned-Go observations."""
import argparse
import json
import os
import subprocess
import tempfile
import time
from collections import Counter
from pathlib import Path
from word_foundation import MODULE,WIDTHS,plan
from word_cases import queries,model_observations,mathematical_observation,negative_plan,NEGATIVES
from word_bindings import fixture,combined_plan
from proof_canonical import canonical
from run_proof_artifacts import inputs as prior_inputs,RECORD as PRIOR
from closed_carriers import SEED
from carrier_build import prepare,TARGET
from run_proof_pilot import ROOT,HERE,DEFAULT_GOROOT,sha,json_bytes,timed,observe,semantic_expectation,verify_host_toolchain
from check_scope_report import strict_json,require

RECORD=ROOT/'develop/migrations/go-practical/t01-w04/word-foundation'
NEW=('word_foundation.py','word_cases.py','word_bindings.py','run_word_foundation.py','test_word_foundation.py','check_word_metadata.py','word_observer.go')
DOCS=('schema.json','bindings.json','queries.json','expansion.json')
SCOPE='ordinary8/16/32/64-bit circuits and universal carry/bit identities,source-typed value/definedness definitions and finite real-Go comparison; no universal Nat bridge,source/operation/frame proof or complete Go VC elaboration'
PENDING=['full-width Nat/Int interpretation/conservation/division/shift/source correctness proofs',
    'remaining contract/operation/source/state/permission/frame/control/totality ordinary definitions and proofs; complete Go statement/program/skeleton assembly',
    'frontend-envelope/policy/API/release roots,reviewed migrations,W04 freeze,W05 inactive publication/T01 gate and full T02–T11']


def catalogue():
    return {'schema':'mpk.go_practical.word_foundation_draft.v0','module':MODULE,'widths':list(WIDTHS),
        'carrier':'retained Bool-indexed full bit vector; addresses MSB first,integer bit0 least significant',
        'ordinary_ast_arms':['const','var','app','pi','lam'],'primitive':'unchanged Std.Bool.cases and ordinary Eq.congr/trans/symm; no arithmetic opcode or axiom',
        'operators':['Not','And','Or','Xor','AndNot','Add','Neg','Sub','Mul','Equal','NotEqual','UnsignedLess','UnsignedLessEqual','SignedLess','SignedLessEqual',
            'ShiftLeft','ShiftRight','SignedShiftRight','UnsignedDiv','UnsignedRem','SignedDiv','SignedRem','SignedTo8/16/32/64','UnsignedTo8/16/32/64'],
        'division':'restoring unsigned division retains lost high remainder bit; signed magnitude division wraps minimum/-1 and remainder follows dividend sign; zero totalizers are inaccessible without DivideDefined',
        'shifts':'full unsigned64 count; saturation at width; signed count requires explicit sign-extended nonnegative condition',
        'proofs':'universal Bool parity/majority/equality and compact named carry-zero lemmas; pointwise AddZero/XorZero/AndOnes/NotNot over arbitrary words and every address; no extensionality',
        'source_specialization':{'operation_ids':['GP-OP-'+str(n).zfill(2) for n in sorted((1,2,3,4,5,6,7,9))],
            'binding_fields':['schema','semantic_context','source_scope_sha256','operation_hash','source_input_types','source_output_type','value_name','definedness_name','declarations','source_evaluation_proved','frame_permission_proved'],
            'rule':'complete captured operation reconstruction before ordinary value/definedness selection; retain all nominal IDs and exact signed/unsigned count type; no caller formula or proof flag'},
        'observation_boundary':'actual emitted ordinary AST interpreter,independent Python integer oracle and pinned native Go; finite cases do not prove general source semantics',
        'owners':{'ordinary_definitions':'crates/mpk-vc/src/go_practical_ordinary.rs (planned)','source_binding':'crates/mpk-vc/src/go_practical_artifacts.rs (planned)',
            'word_vcs':'crates/mpk-vc/src/go_practical_vc.rs (planned)','source_operations':'go-tools/go2vir (planned practical lowering)'},
        'source_evaluation_proved':False,'full_word_Nat_bridge_proved':False,'complete_go_elaboration':False,'profile_activated':False,'remaining':PENDING}


def documents():
    data=fixture();return dict(zip(DOCS,(catalogue(),data,queries(),combined_plan(data))))


def inputs():
    rows={x['path']:x['sha256'] for x in prior_inputs()}
    paths=[PRIOR/'observations.json',HERE/'carrier_build.py']+[HERE/n for n in NEW]+[RECORD/n for n in DOCS]
    for p in paths:rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    for p,h in rows.items():require(sha((ROOT/p).read_bytes())==h,'unchanged word/predecessor input: '+p)
    return [{'path':p,'sha256':h} for p,h in sorted(rows.items())]


def cases():return [{'id':'w'+str(w)+'-'+kind,'width':w,'mutation':kind,'expected':'accepted' if kind=='valid' else 'rejected'} for w in WIDTHS for kind in ('valid',)+NEGATIVES]+[{'id':'source-word-values','expected':'accepted','mutation':'combined','width':None}]
def case_plan(case,docs):return docs['expansion.json'] if case['mutation']=='combined' else plan(case['width']) if case['mutation']=='valid' else negative_plan(case['width'],case['mutation'])


def classify(case,obs):
    if case['expected']=='accepted':
        semantic_expectation(case,obs,declarations=case['declarations'],module=case['module']);return 'accepted_zero_axiom'
    r,g=obs['rust'],obs['go']
    require(r['exit_code']==g['exit_code']==1 and r['report'].get('error_code')=='KERNEL_CORE_CHECK' and g['report'].get('error_kind')=='core_check','paired ordinary core rejection,not parser/format/resource failure')
    require(json.loads(r['report']['error_detail'])['code']=='CORE_TYPE_MISMATCH' and ' inferred ' in g['report']['error_detail'],'paired ordinary type mismatch')
    return 'ill_typed_ordinary_term' if case['mutation']=='ill-typed-shift' else 'incorrect_ordinary_claim'


def check_retained():
    docs=documents();r=strict_json((RECORD/'observations.json').read_bytes())
    for n,value in docs.items():require((RECORD/n).read_bytes()==canonical(value)+b'\n','complete ordinary expansion/binding/query reconstruction: '+n)
    require(r['scope']==SCOPE and r['pending']==PENDING and r['inputs']==inputs(),'word scope/source/input closure')
    actual=__import__('termination_foundation_build').files()
    require(len(actual)==len(r['mirrored_checker_inputs'])==77 and r['mirrored_checker_inputs']==[{'path':p,'sha256':sha(raw)} for p,raw in sorted(actual.items())],'unchanged joint private checker77-source set')
    raw_native=(RECORD/'native.json').read_bytes();native=strict_json(raw_native);model=strict_json((RECORD/'model.json').read_bytes());oracle=[mathematical_observation(q) for q in docs['queries.json']]
    require(len(native)==len(model)==len(oracle)==1640 and model==oracle,'all ordinary AST observations match independent mathematical oracle')
    require([{k:v for k,v in row.items() if k!='detail'} for row in native]==model,'all real Go/ordinary value and runtime-safety observations')
    require(len(r['native_runs'])==2 and all(x['exit_code']==0 and x['stdout_sha256']==sha(raw_native) for x in r['native_runs']),'two identical real-Go outputs')
    require(r['observation_outcomes']==dict(Counter(x['kind'] for x in model)),'exact observation classifications')
    require(len(r['cases'])==len(cases())==21,'five positive/sixteen control certificates')
    for row,c in zip(r['cases'],cases()):
        require(all(row['case'][k]==v for k,v in c.items()),'exact ordinary control identity')
        cp=case_plan(c,docs);raw=(RECORD/(c['id']+'.mpcert')).read_bytes()
        require(row['ast_sha256']==sha(canonical(cp)+b'\n') and row['raw_sha256']==sha(raw) and row['case']['bytes']==len(raw),'complete ordinary source/certificate bytes')
        require(row['classification']==classify(row['case'],row['observations']),'actual paired core classifications')
        require(row['case']['bytes']<=2097152 and row['case']['terms']<=131072 and row['case']['declarations']<=4096,'candidate proof artifact envelope')
        require(len(row['generations'])==2 and all(x['exit_code']==0 and x['stdout_sha256']==row['generations'][0]['stdout_sha256'] for x in row['generations']),'two exact ordinary emissions')
    require(r['outcomes']==dict(Counter(x['classification'] for x in r['cases'])),'all ordinary case outcomes')
    require(r['source_word_types']==13 and r['source_operations']==787 and r['source_operations']==len(docs['bindings.json']['bindings']),'complete fixture word specialization inventory')
    for flag in ('source_evaluation_proved','full_word_Nat_bridge_proved','complete_go_elaboration','profile_activated'):require(r[flag] is False,'word evidence boundary: '+flag)
    return r


def build(command,*,cwd=ROOT,env=None):
    r=subprocess.run(list(map(str,command)),cwd=cwd,env=env,capture_output=True,timeout=300)
    require(r.returncode==0 and b'warning:' not in r.stderr,'build: '+r.stdout.decode(errors='replace')+r.stderr.decode(errors='replace'))


def main():
    a=argparse.ArgumentParser(description=__doc__);m=a.add_mutually_exclusive_group(required=True);m.add_argument('--write',action='store_true');m.add_argument('--check',action='store_true');args=a.parse_args()
    docs=documents();RECORD.mkdir(exist_ok=True)
    if args.write:
        for n,v in docs.items():(RECORD/n).write_bytes(canonical(v)+b'\n')
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/'host-toolchain.json').read_bytes()))
    with tempfile.TemporaryDirectory(prefix='mpk-word-replay-',dir='/private/tmp') as temporary:
        temp=Path(temporary);workspace=temp/'workspace';receipt=prepare(workspace)
        environment={**os.environ,'CARGO_TARGET_DIR':str(TARGET),'CARGO_INCREMENTAL':'0','CARGO_PROFILE_RELEASE_DEBUG':'0','CARGO_PROFILE_RELEASE_CODEGEN_UNITS':'1'}
        build(['cargo','build','--release','--offline','--locked','--manifest-path',workspace/'Cargo.toml','-p','go-practical-closed-carriers'],env=environment)
        rust=TARGET/'release/go-practical-closed-carriers';go=temp/'checker';native=temp/'observer'
        goenv={**os.environ,'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOTELEMETRY':'off','CGO_ENABLED':'0','GOFLAGS':'','GOEXPERIMENT':'','GOCACHE':'/private/tmp/mpk-go-practical-checker-cache'}
        build([DEFAULT_GOROOT/'bin/go','build','-trimpath','-buildvcs=false','-o',go,'./cmd/mpk-checker-ref'],cwd=workspace/'go-tools/mpk-checker-ref',env=goenv)
        build([DEFAULT_GOROOT/'bin/go','build','-trimpath','-buildvcs=false','-o',native,HERE/'word_observer.go'],env=goenv)
        binaries={'rust':sha(rust.read_bytes()),'go':sha(go.read_bytes()),'observer':sha(native.read_bytes())};rows=[];outputs={}
        for case in cases():
            cp=case_plan(case,docs);raw_ast=canonical(cp)+b'\n';ast=temp/'plan.json';ast.write_bytes(raw_ast)
            files=[temp/(case['id']+str(i)+'.mpcert') for i in range(2)];generations=[timed([rust,'emit',SEED,ast,f]) for f in files]
            require(all(x['exit_code']==0 for x in generations) and files[0].read_bytes()==files[1].read_bytes(),'two exact ordinary emitter results')
            counts=strict_json(generations[0]['stdout']);require(counts['added']==[d['name'] for d in cp['declarations']],'all ordinary members emitted')
            c={**case,'module':cp['module'],**{k:counts[k] for k in ('bytes','terms','declarations')},'new_definitions':sum(d['kind']=='definition' for d in cp['declarations']),'new_theorems':sum(d['kind']=='theorem' for d in cp['declarations'])}
            obs={n:observe(exe,files[0]) for n,exe in (('rust',rust),('go',go))};classification=classify(c,obs)
            require(all(x['wall_seconds']<=60 and x['peak_rss_bytes']<=1073741824 for x in generations+list(obs.values())),'candidate direct-child proof resource envelope')
            raw=files[0].read_bytes();outputs[case['id']+'.mpcert']=raw
            rows.append({'case':c,'ast_sha256':sha(raw_ast),'raw_sha256':sha(raw),'classification':classification,
                'generations':[{k:v for k,v in g.items() if k!='stdout'}|{'stdout_sha256':sha(g['stdout'].encode())} for g in generations],'observations':obs})
            print(case['id']+': '+classification,flush=True)
        # Pass exact canonical query bytes on stdin, without shell expansion.
        native_raw=[];native_runs=[]
        for _ in range(2):
            start=time.perf_counter();r=subprocess.run([native],input=canonical(docs['queries.json']),capture_output=True,timeout=60)
            require(r.returncode==0 and not r.stderr,'native Go observation run');native_raw.append(r.stdout)
            native_runs.append({'exit_code':r.returncode,'wall_seconds':time.perf_counter()-start,'stdin_sha256':sha(canonical(docs['queries.json'])),'stdout_sha256':sha(r.stdout)})
        require(native_raw[0]==native_raw[1],'two exact real-Go observations')
        start=time.perf_counter();model=model_observations(docs['queries.json'],lambda n,t:print(f'ordinary AST observations {n}/{t}',flush=True));model_seconds=time.perf_counter()-start
        oracle=[mathematical_observation(q) for q in docs['queries.json']];native_values=strict_json(native_raw[0])
        require(model==oracle==[{k:v for k,v in row.items() if k!='detail'} for row in native_values],'all1640 actual ordinary AST/native Go/integer values and safety classifications')
        require(before==inputs() and binaries=={'rust':sha(rust.read_bytes()),'go':sha(go.read_bytes()),'observer':sha(native.read_bytes())},'source and binary stability')
        for x in receipt['generated']:require(sha((workspace/x['path']).read_bytes())==x['sha256'],'unchanged private checker source')
        report={'schema':'mpk.go_practical.word_foundation_probe.v0','scope':SCOPE,'pending':PENDING,'inputs':before,'host_toolchain_inventory':inventory,
            'mirrored_checker_inputs':receipt['generated'],'checker_binaries':binaries,'cases':rows,'outcomes':dict(Counter(x['classification'] for x in rows)),
            'source_word_types':13,'source_operations':787,'native_runs':native_runs,'model_wall_seconds':model_seconds,
            'observation_outcomes':dict(Counter(x['kind'] for x in model)),'direct_child_parallelism':1,
            'proof_resource_scope':'60s/1GiB per direct proof child; Python fixture/AST observation construction is not a certificate check or release measurement',
            'source_evaluation_proved':False,'full_word_Nat_bridge_proved':False,'complete_go_elaboration':False,'profile_activated':False}
        if args.write:
            for n,raw in outputs.items():(RECORD/n).write_bytes(raw)
            (RECORD/'model.json').write_bytes(canonical(model)+b'\n');(RECORD/'native.json').write_bytes(native_raw[0]);(RECORD/'observations.json').write_bytes(json_bytes(report))
        old=check_retained();require(old['checker_binaries']==binaries and all((RECORD/n).read_bytes()==raw for n,raw in outputs.items()),'fresh full ordinary binary/certificate replay')
    print('PASS:5 zero-axiom ordinary sets,12 false-claim/4 ill-typed controls,787 source-word value/definedness bindings,1640 ordinary AST/Go/integer observations; full Go elaboration and source/Nat proofs remain due',flush=True)


if __name__=='__main__':main()
