#!/usr/bin/env python3
"""Replay unbounded ordinary arithmetic and the closed logical wire component."""
import argparse
import json
import os
import subprocess
import tempfile
from collections import Counter
from pathlib import Path
from arithmetic_terms import MODULE
from ordinary_arithmetic import plan
from logical_elaboration import HEADS,validate
from arithmetic_cases import expressions,compilations,compiled_plan,expected,observations,vectors,classify_vectors,MUTATIONS,negative_plan
from closed_carriers import SEED
from proof_canonical import canonical
from carrier_build import prepare,TARGET
from run_numeric_bridge import inputs as prior_inputs,RECORD as PRIOR
from run_word_foundation import build
from run_proof_pilot import ROOT,HERE,DEFAULT_GOROOT,sha,json_bytes,timed,observe,semantic_expectation,verify_host_toolchain
from check_scope_report import require,strict_json

RECORD=ROOT/'develop/migrations/go-practical/t01-w04/ordinary-arithmetic'
NEW=('arithmetic_terms.py','ordinary_arithmetic.py','ordinary_integers.py','ordinary_ranges.py','logical_elaboration.py','arithmetic_cases.py','run_ordinary_arithmetic.py','test_ordinary_arithmetic.py','check_arithmetic_metadata.py','arithmetic_observer.go')
DOCS=('schema.json','expansion.json','expressions.json','compilations.json','vectors.json','outcomes.json')
SCOPE='ordinary unbounded Nat algebra,balanced-pair Int algebra/equality/order independence,half-open logical folds and typed closed logical components; no source or complete Go elaboration proof'
PENDING=['connection of actual word denotations to this general Nat/Int library and remaining word/source arithmetic correctness',
    'source/storage/partial expressions,bounded definedness and independent elaborator/source correctness; all169 retained complete Go requests remain unelaborated',
    'remaining ordinary state/permission/frame/control/totality and complete Go statement/program/skeleton assembly',
    'frontend-envelope/policy/API/release roots,reviewed migrations,W04 freeze,W05 inactive publication/T01 gate and full T02–T11']


def catalogue():
    return {'schema':'mpk.go_practical.ordinary_arithmetic_draft.v0','module':MODULE,'heads':sorted(HEADS),
        'seed':SEED.relative_to(ROOT).as_posix(),'seed_sha256':sha(SEED.read_bytes()),
        'Nat':'genuine unbounded Std.Nat; recursive addition,multiplication,truncated subtraction,equality and order; checked induction and cancellation laws',
        'Int':'Bool->Nat balanced pair p-n; Equiv is genuine Nat cross-sum equality,never core equality of pair functions; operations and Bool observations respect representatives',
        'ranges':'half-open [lower,upper); count=truncated subtraction; checked zero,step,equal/reversed empty and representative independence under actual bounded FoldForall premise',
        'compiler':'independently infer existing normalized logical types,arity,payload and lexical Nat binder depth; reconstruct whole component; total closed component Defined=True is a value definition',
        'proof_boundary':'ordinary universal library theorems are checked; accepting closed Value/Defined definitions does not prove a Boolean clause true; ordinary_proof stays null',
        'literals':'exact canonical decimal input and binary Nat declaration DAG; private4096 declaration/decimal construction guard is pending,not a semantic integer bound',
        'model_boundary':'139 actual ordinary-AST observations;10 large inputs exceed private512 unary-model loop ceiling and remain model_resource_pending while native Go/big.Int observations and component typechecking succeed',
        'production_owners':{'ordinary':'crates/mpk-vc/src/go_practical_ordinary.rs (planned)','logical':'crates/mpk-vc/src/go_practical_contracts.rs (planned)'},
        'ordinary_library_checked':True,'word_library_connection_proved':False,'elaborator_correctness_proved':False,
        'source_evaluation_proved':False,'complete_go_elaboration':False,'profile_activated':False,'remaining':PENDING}


def documents():
    rows=expressions();vs=vectors()
    return dict(zip(DOCS,(catalogue(),plan(),rows,compilations(rows),vs,classify_vectors(vs))))


def cases():
    return [{'id':n,'expected':'accepted' if n in {'theory','closed-logical'} else 'rejected'} for n in ('theory','closed-logical')+MUTATIONS]
def case_plan(c,docs,closed):
    return docs['expansion.json'] if c['id']=='theory' else closed if c['id']=='closed-logical' else negative_plan(docs['expansion.json'],closed,c['id'])
def classify(c,obs):
    if c['expected']=='accepted':semantic_expectation(c,obs,declarations=c['declarations'],module=c['module']);return 'accepted_zero_axiom'
    r,g=obs['rust'],obs['go']
    require(r['exit_code']==g['exit_code']==1 and r['report'].get('error_code')=='KERNEL_CORE_CHECK' and g['report'].get('error_kind')=='core_check','paired ordinary core rejection,not resource/format rejection')
    require(json.loads(r['report']['error_detail'])['code']=='CORE_TYPE_MISMATCH' and (' inferred ' in g['report']['error_detail'] or g['report']['error_detail']=='lambda domain mismatch'),'paired ordinary type mismatch')
    return 'ordinary_type_mismatch'


def inputs():
    rows={x['path']:x['sha256'] for x in prior_inputs()}
    for p in [PRIOR/'observations.json',SEED]+[HERE/n for n in NEW]+[RECORD/n for n in DOCS]:rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    for p,h in rows.items():require(sha((ROOT/p).read_bytes())==h,'arithmetic/current predecessor pin: '+p)
    return [{'path':p,'sha256':h} for p,h in sorted(rows.items())]


def check_retained():
    docs=documents();r=strict_json((RECORD/'observations.json').read_bytes());closed=compiled_plan(docs['expansion.json'],docs['compilations.json'])
    for n,d in docs.items():require((RECORD/n).read_bytes()==canonical(d)+b'\n','complete arithmetic/logical reconstruction: '+n)
    require(r['scope']==SCOPE and r['pending']==PENDING and r['inputs']==inputs(),'ordinary arithmetic source/scope closure')
    actual=__import__('termination_foundation_build').files()
    require(len(actual)==len(r['mirrored_checker_inputs'])==77 and r['mirrored_checker_inputs']==[{'path':p,'sha256':sha(b)} for p,b in sorted(actual.items())],'unchanged joint77-source checker')
    require(len(r['cases'])==10,'complete paired arithmetic matrix')
    for x,c in zip(r['cases'],cases()):
        require(all(x['case'][k]==v for k,v in c.items()),'exact arithmetic case identity')
        cp=case_plan(c,docs,closed);raw=(RECORD/(c['id']+'.mpcert')).read_bytes()
        require(x['raw_sha256']==sha(raw) and x['case']['bytes']==len(raw) and x['ast_sha256']==sha(canonical(cp)+b'\n'),'full ordinary source/certificate bytes')
        require(x['classification']==classify(x['case'],x['observations']),'actual paired proof outcomes')
        require(x['case']['bytes']<=2097152 and x['case']['terms']<=131072 and x['case']['declarations']<=4096,'candidate proof artifact envelope')
        require(len(x['generations'])==2 and all(g['exit_code']==0 and g['stdout_sha256']==x['generations'][0]['stdout_sha256'] for g in x['generations']),'two exact ordinary emissions')
    require(r['outcomes']=={'accepted_zero_axiom':2,'ordinary_type_mismatch':8},'proof outcomes separate from model resource pending')
    rows=docs['expressions.json'];compiled=docs['compilations.json']
    require(len(rows)==len(compiled)==149,'closed expression coverage')
    for row,c in zip(rows,compiled):require(row['id']==c['id'],'ordered component identity');validate(row['expression'],c['program'])
    native_raw=(RECORD/'native.json').read_bytes();native=strict_json(native_raw);model=strict_json((RECORD/'model.json').read_bytes())
    require(native==expected(rows) and model==observations(rows,compiled,closed),'actual AST/Go/integer observations')
    require(Counter(x['status'] for x in model)=={'evaluated':139,'model_resource_pending':10},'exact private model ceiling')
    for m,n in zip(model,native):
        if m['status']=='evaluated':require({k:v for k,v in m.items() if k!='status'}==n,'actual evaluated AST agrees with independent native/oracle')
    require(len(r['native_runs'])==2 and all(x['exit_code']==0 and x['stdin_sha256']==sha(canonical(rows)) and x['stdout_sha256']==sha(native_raw) for x in r['native_runs']),'two exact independent Go observations')
    require(r['component_outcomes']==dict(Counter(x['classification'] for x in docs['outcomes.json']))=={'typed_logical_unproved':1,'configuration_rejection':19,'ordinary_component_pending':2},'typed false clauses and explicit pending')
    for f in (RECORD/'verification').glob('initial-*/diagnostic.json'):
        d=strict_json(f.read_bytes())
        for x in d['inputs']:require(sha((f.parent/x['path']).read_bytes())==x['sha256'],'preserved initial producer finding')
    for flag in ('word_library_connection_proved','elaborator_correctness_proved','source_evaluation_proved','complete_go_elaboration','profile_activated'):require(r[flag] is False,'logical proof evidence boundary: '+flag)
    return r


def main():
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group(required=True);g.add_argument('--write',action='store_true');g.add_argument('--check',action='store_true');args=p.parse_args()
    docs=documents();RECORD.mkdir(exist_ok=True);closed=compiled_plan(docs['expansion.json'],docs['compilations.json'])
    if args.write:
        for n,d in docs.items():(RECORD/n).write_bytes(canonical(d)+b'\n')
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/'host-toolchain.json').read_bytes()))
    with tempfile.TemporaryDirectory(prefix='mpk-arithmetic-replay-',dir='/private/tmp') as temporary:
        temp=Path(temporary);workspace=temp/'workspace';receipt=prepare(workspace)
        env={**os.environ,'CARGO_TARGET_DIR':str(TARGET),'CARGO_INCREMENTAL':'0','CARGO_PROFILE_RELEASE_DEBUG':'0','CARGO_PROFILE_RELEASE_CODEGEN_UNITS':'1'}
        build(['cargo','build','--release','--offline','--locked','--manifest-path',workspace/'Cargo.toml','-p','go-practical-closed-carriers'],env=env)
        rust=TARGET/'release/go-practical-closed-carriers';go=temp/'checker';native=temp/'observer'
        goenv={**os.environ,'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOTELEMETRY':'off','CGO_ENABLED':'0','GOFLAGS':'','GOEXPERIMENT':'','GOCACHE':'/private/tmp/mpk-go-practical-checker-cache'}
        build([DEFAULT_GOROOT/'bin/go','build','-trimpath','-buildvcs=false','-o',go,'./cmd/mpk-checker-ref'],cwd=workspace/'go-tools/mpk-checker-ref',env=goenv)
        build([DEFAULT_GOROOT/'bin/go','build','-trimpath','-buildvcs=false','-o',native,HERE/'arithmetic_observer.go'],env=goenv)
        binaries={'rust':sha(rust.read_bytes()),'go':sha(go.read_bytes()),'observer':sha(native.read_bytes())};rows=[];outputs={}
        for c in cases():
            cp=case_plan(c,docs,closed);ast=canonical(cp)+b'\n';wire=temp/'plan.json';wire.write_bytes(ast)
            files=[temp/(c['id']+str(i)+'.mpcert') for i in range(2)];runs=[timed([rust,'emit',SEED,wire,f]) for f in files]
            require(all(x['exit_code']==0 for x in runs) and files[0].read_bytes()==files[1].read_bytes(),'two byte-identical arithmetic emissions')
            counts=strict_json(runs[0]['stdout']);require(counts['added']==[d['name'] for d in cp['declarations']],'every ordinary arithmetic member emitted in order')
            case={**c,'module':cp['module'],**{k:counts[k] for k in ('bytes','terms','declarations')},'new_definitions':sum(d['kind']=='definition' for d in cp['declarations']),'new_theorems':sum(d['kind']=='theorem' for d in cp['declarations'])}
            obs={n:observe(exe,files[0]) for n,exe in (('rust',rust),('go',go))};classification=classify(case,obs)
            require(all(x['wall_seconds']<=60 and x['peak_rss_bytes']<=1073741824 for x in runs+list(obs.values())),'per-direct-proof-child resource envelope')
            raw=files[0].read_bytes();outputs[c['id']+'.mpcert']=raw
            rows.append({'case':case,'raw_sha256':sha(raw),'ast_sha256':sha(ast),'classification':classification,
                'generations':[{k:v for k,v in x.items() if k!='stdout'}|{'stdout_sha256':sha(x['stdout'].encode())} for x in runs],'observations':obs})
            print(c['id']+': '+classification,flush=True)
        native_runs=[];native_raw=[];qs=canonical(docs['expressions.json'])
        for _ in range(2):
            x=subprocess.run([native],input=qs,capture_output=True,timeout=60);require(x.returncode==0 and not x.stderr,'actual Go/big.Int logical observations')
            native_raw.append(x.stdout);native_runs.append({'exit_code':x.returncode,'stdin_sha256':sha(qs),'stdout_sha256':sha(x.stdout)})
        require(native_raw[0]==native_raw[1] and strict_json(native_raw[0])==expected(docs['expressions.json']),'two identical independent Go/integer outputs')
        model=observations(docs['expressions.json'],docs['compilations.json'],closed)
        require(before==inputs() and binaries=={'rust':sha(rust.read_bytes()),'go':sha(go.read_bytes()),'observer':sha(native.read_bytes())},'source and binary stability')
        for x in receipt['generated']:require(sha((workspace/x['path']).read_bytes())==x['sha256'],'unchanged private checker sources')
        report={'schema':'mpk.go_practical.ordinary_arithmetic_probe.v0','scope':SCOPE,'pending':PENDING,'inputs':before,'host_toolchain_inventory':inventory,'checker_binaries':binaries,
            'mirrored_checker_inputs':receipt['generated'],'cases':rows,'outcomes':dict(Counter(x['classification'] for x in rows)),
            'component_outcomes':dict(Counter(x['classification'] for x in docs['outcomes.json'])),'logical_heads':len(HEADS),'native_observations':149,'actual_AST_observations':139,'model_resource_pending':10,
            'native_runs':native_runs,'direct_child_parallelism':1,'proof_resource_scope':'60s/1GiB per direct proof child; Darwin measurement,not Linux release/process-tree evidence',
            'ordinary_library_checked':True,'word_library_connection_proved':False,'elaborator_correctness_proved':False,'source_evaluation_proved':False,'complete_go_elaboration':False,'profile_activated':False}
        if args.write:
            for n,raw in outputs.items():(RECORD/n).write_bytes(raw)
            (RECORD/'model.json').write_bytes(canonical(model)+b'\n');(RECORD/'native.json').write_bytes(native_raw[0]);(RECORD/'observations.json').write_bytes(json_bytes(report))
        old=check_retained();require(old['checker_binaries']==binaries and all((RECORD/n).read_bytes()==raw for n,raw in outputs.items()),'fresh complete arithmetic certificate/binary replay')
    print('PASS:2 zero-axiom ordinary sets,8 paired type mismatches,27 typed logical heads,149 Go/integer observations,139 actual AST results and10 private model-resource pending; W04 open',flush=True)


if __name__=='__main__':main()
