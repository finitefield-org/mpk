#!/usr/bin/env python3
"""Rebuild/recheck the actual ripple-to-Nat and signed/source-value bridge."""
import argparse
import json
import os
import subprocess
import tempfile
from collections import Counter
from pathlib import Path
from word_numeric_bridge import MODULE,SEED,Bridge,WIDTHS,WORD_BINDINGS
from numeric_bridge_cases import MUTATIONS,negative_plan,queries,model_observations,mathematical_observation
from proof_canonical import canonical
from carrier_build import prepare,TARGET
from run_word_foundation import inputs as prior_inputs,RECORD as PRIOR,build
from run_proof_pilot import ROOT,HERE,DEFAULT_GOROOT,sha,json_bytes,timed,observe,semantic_expectation,verify_host_toolchain
from check_scope_report import require,strict_json

RECORD=ROOT/'develop/migrations/go-practical/t01-w04/word-numeric-bridge'
NEW=('word_numeric_bridge.py','numeric_bridge_cases.py','run_numeric_bridge.py','test_numeric_bridge.py','check_numeric_metadata.py','numeric_observer.go')
DOCS=('schema.json','source-values.json','queries.json','expansions.json')
SCOPE='actual ripple/balanced word pointwise/carry/sign-stage correspondence,exact Nat addition with final carry and conditional signed cross sums,13 exact source Value additions; not full word arithmetic/Int/contract/source/frame or complete Go VC elaboration'
PENDING=['general Nat/Int operations,full arithmetic/multiplication/division/shift/contract/source correctness beyond the checked addition bridge',
    'all remaining ordinary source/state/permission/frame/control/totality definitions/proofs and complete Go theorem/program/skeleton assembly',
    'frontend-envelope/policy/API/release roots,reviewed migrations,W04 freeze,W05 inactive publication/T01 gate and full T02–T11']


def catalogue():
    return {'schema':'mpk.go_practical.word_numeric_bridge_draft.v0','module':MODULE,'widths':list(WIDTHS),
        'seed':SEED.relative_to(ROOT).as_posix(),'seed_sha256':sha(SEED.read_bytes()),
        'unsigned':'forall whole words x,y,incoming Bool and initial Nat; exact successor denotation of x+y+incoming equals actual AddCarry result plus its actual final2^width carry',
        'signed':'balanced natural cross sums express two-complement integer addition under equality of actual sign-stage incoming and final carry; signs/result stay bound to the same ripple operation',
        'correspondence':'each aligned binary block proves old carry/current prefix carry,pointwise old/current sum and old/current sign-stage carry; checked children compose to the full word',
        'value_congruence':'pointwise bit equality lifted through balanced value definitions by finite structural composition,without word function extensionality',
        'source_values':'copy exact preceding typed Value/Defined declarations after complete captured operation reconstruction; first check word equality,then lift through sign/value/Nat contexts',
        'controls':{'ordinary_type_mismatch':14,'candidate_conversion_fuel_failure':2,'fuel_cases':'only32/64 dropped-final-carry candidates; not false-claim/source-counterexample evidence'},
        'observations':'actual Go typed addition/math.bits/math.big versus actual ordinary ripple/carry/guard evaluation and independent integer oracle; finite observations do not replace universal checked Nat proofs',
        'initial_nat':'arbitrary unbounded Nat in proofs; native observations use0,17,10^50+37 as exact decimal strings',
        'production_owners':{'ordinary':'crates/mpk-vc/src/go_practical_ordinary.rs (planned)','vcs':'crates/mpk-vc/src/go_practical_vc.rs (planned)',
            'source_bindings':'crates/mpk-vc/src/go_practical_artifacts.rs (planned)'},
        'ripple_addition_Nat_proved':True,'conditional_signed_cross_sums_proved':True,'all_word_Nat_Int_operations_proved':False,
        'source_evaluation_proved':False,'complete_go_elaboration':False,'profile_activated':False,'remaining':PENDING}


def documents():
    plans={};sources=[]
    for w in WIDTHS:
        b=Bridge(w);plans[str(w)]=b.build();sources+=b.source_rows
    return dict(zip(DOCS,(catalogue(),sources,queries(),plans)))


def cases():return [{'id':'w'+str(w)+'-'+k,'width':w,'mutation':k,'expected':'accepted' if k=='valid' else 'rejected'} for w in WIDTHS for k in ('valid',)+MUTATIONS]
def case_plan(c,docs):
    p=docs['expansions.json'][str(c['width'])]
    return p if c['mutation']=='valid' else negative_plan(c['width'],c['mutation'],p)


def classify(c,obs):
    if c['expected']=='accepted':semantic_expectation(c,obs,declarations=c['declarations'],module=c['module']);return 'accepted_zero_axiom'
    r,g=obs['rust'],obs['go'];require(r['exit_code']==g['exit_code']==1 and r['report'].get('error_code')=='KERNEL_CORE_CHECK' and g['report'].get('error_kind')=='core_check','paired core boundary,not timeout/format rejection')
    code=json.loads(r['report']['error_detail'])['code'];detail=g['report']['error_detail']
    if c['mutation']=='dropped-final-carry' and c['width'] in (32,64):
        require(code=='CORE_FUEL_EXHAUSTED' and 'fuel exhausted' in detail,'paired candidate conversion-fuel failure');return 'candidate_conversion_fuel_failure'
    require(code=='CORE_TYPE_MISMATCH' and (' inferred ' in detail or detail=='lambda domain mismatch'),'paired ordinary type mismatch')
    return 'ordinary_type_mismatch'


def inputs():
    rows={x['path']:x['sha256'] for x in prior_inputs()}
    for p in [PRIOR/'observations.json',SEED,WORD_BINDINGS]+[HERE/n for n in NEW]+[RECORD/n for n in DOCS]:rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    for p,h in rows.items():require(sha((ROOT/p).read_bytes())==h,'numeric/current predecessor pin: '+p)
    return [{'path':p,'sha256':h} for p,h in sorted(rows.items())]


def check_retained():
    docs=documents();r=strict_json((RECORD/'observations.json').read_bytes())
    for n,d in docs.items():require((RECORD/n).read_bytes()==canonical(d)+b'\n','full numeric expansion/context/source/queries: '+n)
    require(r['scope']==SCOPE and r['pending']==PENDING and r['inputs']==inputs(),'numeric bridge source/scope closure')
    actual=__import__('termination_foundation_build').files()
    require(len(actual)==len(r['mirrored_checker_inputs'])==77 and r['mirrored_checker_inputs']==[{'path':p,'sha256':sha(b)} for p,b in sorted(actual.items())],'unchanged joint77-source checker')
    require(len(r['cases'])==20 and len(docs['source-values.json'])==r['source_values']==13,'full width/control/source-value coverage')
    for x,c in zip(r['cases'],cases()):
        require(all(x['case'][k]==v for k,v in c.items()),'exact numeric case identity')
        cp=case_plan(c,docs);raw=(RECORD/(c['id']+'.mpcert')).read_bytes()
        require(x['raw_sha256']==sha(raw) and x['case']['bytes']==len(raw) and x['ast_sha256']==sha(canonical(cp)+b'\n'),'full numeric source/certificate bytes')
        require(x['classification']==classify(x['case'],x['observations']),'actual paired proof outcomes')
        require(x['case']['bytes']<=2097152 and x['case']['terms']<=131072 and x['case']['declarations']<=4096,'candidate proof artifact envelope')
        require(len(x['generations'])==2 and all(g['exit_code']==0 and g['stdout_sha256']==x['generations'][0]['stdout_sha256'] for g in x['generations']),'two exact ordinary emissions')
    require(r['outcomes']=={'accepted_zero_axiom':4,'ordinary_type_mismatch':14,'candidate_conversion_fuel_failure':2},'no fuel-to-false-claim promotion')
    native_raw=(RECORD/'native.json').read_bytes();native=strict_json(native_raw);model=strict_json((RECORD/'model.json').read_bytes());qs=docs['queries.json']
    require(len(qs)==len(native)==len(model)==288 and model==native==[mathematical_observation(q) for q in qs],'all actual ripple/Go/integer comparisons')
    require(all(x['unsigned_left']==x['unsigned_right'] and (x['signed_cross_left']==x['signed_cross_right'])==x['no_overflow'] for x in model),'actual final carry and exact signed guard')
    require(r['native_no_overflow']==sum(x['no_overflow'] for x in model)==210 and r['native_overflow']==78,'signed overflow observed separately')
    require(len(r['native_runs'])==2 and all(x['exit_code']==0 and x['stdin_sha256']==sha(canonical(qs)) and x['stdout_sha256']==sha(native_raw) for x in r['native_runs']),'two exact independent Go observations')
    for f in (RECORD/'verification').glob('initial-*/diagnostic.json'):
        d=strict_json(f.read_bytes())
        for x in d['inputs']:require(sha((f.parent/x['path']).read_bytes())==x['sha256'],'preserved initial conversion finding')
    for flag in ('all_word_Nat_Int_operations_proved','source_evaluation_proved','complete_go_elaboration','profile_activated'):require(r[flag] is False,'numeric proof evidence boundary: '+flag)
    return r


def main():
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group(required=True);g.add_argument('--write',action='store_true');g.add_argument('--check',action='store_true');args=p.parse_args()
    docs=documents();RECORD.mkdir(exist_ok=True)
    if args.write:
        for n,d in docs.items():(RECORD/n).write_bytes(canonical(d)+b'\n')
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/'host-toolchain.json').read_bytes()))
    with tempfile.TemporaryDirectory(prefix='mpk-numeric-replay-',dir='/private/tmp') as temporary:
        temp=Path(temporary);workspace=temp/'workspace';receipt=prepare(workspace)
        env={**os.environ,'CARGO_TARGET_DIR':str(TARGET),'CARGO_INCREMENTAL':'0','CARGO_PROFILE_RELEASE_DEBUG':'0','CARGO_PROFILE_RELEASE_CODEGEN_UNITS':'1'}
        build(['cargo','build','--release','--offline','--locked','--manifest-path',workspace/'Cargo.toml','-p','go-practical-closed-carriers'],env=env)
        rust=TARGET/'release/go-practical-closed-carriers';go=temp/'checker';native=temp/'observer'
        goenv={**os.environ,'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOTELEMETRY':'off','CGO_ENABLED':'0','GOFLAGS':'','GOEXPERIMENT':'','GOCACHE':'/private/tmp/mpk-go-practical-checker-cache'}
        build([DEFAULT_GOROOT/'bin/go','build','-trimpath','-buildvcs=false','-o',go,'./cmd/mpk-checker-ref'],cwd=workspace/'go-tools/mpk-checker-ref',env=goenv)
        build([DEFAULT_GOROOT/'bin/go','build','-trimpath','-buildvcs=false','-o',native,HERE/'numeric_observer.go'],env=goenv)
        binaries={'rust':sha(rust.read_bytes()),'go':sha(go.read_bytes()),'observer':sha(native.read_bytes())};rows=[];outputs={}
        for c in cases():
            cp=case_plan(c,docs);ast=canonical(cp)+b'\n';wire=temp/'plan.json';wire.write_bytes(ast)
            files=[temp/(c['id']+str(i)+'.mpcert') for i in range(2)];runs=[timed([rust,'emit',SEED,wire,f]) for f in files]
            require(all(x['exit_code']==0 for x in runs) and files[0].read_bytes()==files[1].read_bytes(),'two byte-identical numeric emissions')
            counts=strict_json(runs[0]['stdout']);require(counts['added']==[d['name'] for d in cp['declarations']],'every numeric ordinary member emitted in order')
            case={**c,'module':cp['module'],**{k:counts[k] for k in ('bytes','terms','declarations')},'new_definitions':sum(d['kind']=='definition' for d in cp['declarations']),'new_theorems':sum(d['kind']=='theorem' for d in cp['declarations'])}
            obs={n:observe(exe,files[0]) for n,exe in (('rust',rust),('go',go))};classification=classify(case,obs)
            require(all(x['wall_seconds']<=60 and x['peak_rss_bytes']<=1073741824 for x in runs+list(obs.values())),'per-direct-proof-child resource envelope')
            raw=files[0].read_bytes();outputs[c['id']+'.mpcert']=raw
            rows.append({'case':case,'raw_sha256':sha(raw),'ast_sha256':sha(ast),'classification':classification,
                'generations':[{k:v for k,v in x.items() if k!='stdout'}|{'stdout_sha256':sha(x['stdout'].encode())} for x in runs],'observations':obs})
            print(c['id']+': '+classification,flush=True)
        native_runs=[];native_raw=[];qs=canonical(docs['queries.json'])
        for _ in range(2):
            x=subprocess.run([native],input=qs,capture_output=True,timeout=60);require(x.returncode==0 and not x.stderr,'actual Go/big.Int observations')
            native_raw.append(x.stdout);native_runs.append({'exit_code':x.returncode,'stdin_sha256':sha(qs),'stdout_sha256':sha(x.stdout)})
        require(native_raw[0]==native_raw[1],'two identical actual Go outputs')
        model=model_observations(docs['queries.json'],{int(k):v for k,v in docs['expansions.json'].items()})
        require(model==strict_json(native_raw[0])==[mathematical_observation(q) for q in docs['queries.json']],'actual current ripple/Go/big integer agreement')
        require(before==inputs() and binaries=={'rust':sha(rust.read_bytes()),'go':sha(go.read_bytes()),'observer':sha(native.read_bytes())},'source and binary stability')
        for x in receipt['generated']:require(sha((workspace/x['path']).read_bytes())==x['sha256'],'unchanged private checker sources')
        report={'schema':'mpk.go_practical.word_numeric_bridge_probe.v0','scope':SCOPE,'pending':PENDING,'inputs':before,'host_toolchain_inventory':inventory,'checker_binaries':binaries,
            'mirrored_checker_inputs':receipt['generated'],'cases':rows,'outcomes':dict(Counter(x['classification'] for x in rows)),'source_values':13,
            'native_runs':native_runs,'native_no_overflow':sum(x['no_overflow'] for x in model),'native_overflow':sum(not x['no_overflow'] for x in model),'direct_child_parallelism':1,
            'proof_resource_scope':'60s/1GiB per direct proof child; Darwin measurement,not Linux release/process-tree evidence',
            'ripple_addition_Nat_proved':True,'conditional_signed_cross_sums_proved':True,'all_word_Nat_Int_operations_proved':False,'source_evaluation_proved':False,'complete_go_elaboration':False,'profile_activated':False}
        if args.write:
            for n,raw in outputs.items():(RECORD/n).write_bytes(raw)
            (RECORD/'model.json').write_bytes(canonical(model)+b'\n');(RECORD/'native.json').write_bytes(native_raw[0]);(RECORD/'observations.json').write_bytes(json_bytes(report))
        old=check_retained();require(old['checker_binaries']==binaries and all((RECORD/n).read_bytes()==raw for n,raw in outputs.items()),'fresh complete numeric certificate/binary replay')
    print('PASS:4 zero-axiom full-width Nat/signed/source-value sets,14 paired type mismatches,2 paired conversion-fuel controls (not false-claim evidence),13 exact typed additions and288 actual ripple/Go/integer observations; W04 open',flush=True)


if __name__=='__main__':main()
