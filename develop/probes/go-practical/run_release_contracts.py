#!/usr/bin/env python3
"""Record release v3 metadata design with actual old validators and raw hashes."""
import argparse,os,subprocess,tempfile
from pathlib import Path
from release_vectors import documents,hash_queries
from release_contracts import *
from release_native_build import build,sources,TARGET
from run_ai_contracts import inputs as prior_inputs,RECORD as PREVIOUS
from run_proof_pilot import HERE,DEFAULT_GOROOT,json_bytes,verify_host_toolchain
from run_structural_limits import timed_input
from check_scope_report import strict_json,require

RECORD=PREVIOUS.parent/'release-contracts'
NEW=('release_contracts.py','release_vectors.py','release_native_build.py','run_release_contracts.py','test_release_contracts.py','check_release_metadata.py','release_hashes.go')
DOCS=('schema.json','graph.json','vectors.json','hash-vectors.json')
SCOPE='inactive seven-root release metadata graph and readiness/activation contracts; actual unchanged four-language/five-target registry validation/selection and independent release canonical/content hashes; no new bundle/consumer build or release/native isolation proof'
PENDING=['ordinary-program/materialized-policy signatures,exact Go execution-host/budget closure and complete consumer/axiom-policy/historical-source-epoch migration review for W04',
 'actual v3 producers/resolver/materialization,reproducible Go frontend/consumer builds,all complete native Linux release/activation/rollback evidence in T07; metadata projections cannot run',
 'W05 inactive publication/cumulative T01 gate and full T02–T11 implementation,source/ordinary/application proof and release requirements']


def source_inputs():
    rows={r['path']:r['sha256']for r in prior_inputs()+sources()}
    for p in [HERE/n for n in NEW]+[PREVIOUS/'observations.json']:
        rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    return [{'path':p,'sha256':h}for p,h in sorted(rows.items())]
def inputs():return source_inputs()+[{'path':(RECORD/n).relative_to(ROOT).as_posix(),'sha256':sha((RECORD/n).read_bytes())}for n in DOCS]
def compact(row):return {k:v for k,v in row.items()if k not in ('family','index','domain')}


def check_retained():
    r=strict_json((RECORD/'observations.json').read_bytes());native=strict_json((RECORD/'native-legacy.json').read_bytes())[0];ds=documents(native)
    for n,v in ds.items():require((RECORD/n).read_bytes()==canonical(v)+b'\n','release graph/vector reconstruction '+n)
    require(r['scope']==SCOPE and r['pending']==PENDING and r['inputs']==inputs(),'release source/scope closure')
    for name in ('native-legacy','rust-hashes','go-hashes'):
        raw=(RECORD/(name+'.json')).read_bytes();require(len(r['runs'][name])==2 and all(x['exit_code']==0 and x['stdout_sha256']==sha(raw)for x in r['runs'][name]),'two exact complete native release observations')
    expected=[compact(h)for h in ds['hash-vectors.json']]
    require(strict_json((RECORD/'rust-hashes.json').read_bytes())==expected and strict_json((RECORD/'go-hashes.json').read_bytes())==expected,'independent compact canonical/domain hash agreement')
    for p in (RECORD/'verification').glob('initial-*/diagnostic.json'):
        for row in strict_json(p.read_bytes())['inputs']:require(sha((p.parent/row['path']).read_bytes())==row['sha256'],'initial release setup finding retained')
    for flag in ('new_frontend_executed','native_release_observed','Go_ordinary_elaborated','profile_activated','W04_frozen'):require(r[flag]is False,'metadata cannot imply execution/proof/activation')
    return r


def main():
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group(required=True);g.add_argument('--write',action='store_true');g.add_argument('--check',action='store_true');args=p.parse_args()
    before=source_inputs();cmd,audit=build();runs={};results={}
    observed=[timed_input(cmd,canonical([{'mode':'legacy'}]),os.environ.copy())for _ in range(2)]
    require(observed[0][0]==observed[1][0],'two actual old release validator/selector runs');results['native-legacy']=observed[0][0];runs['native-legacy']=[r for _,r in observed]
    native=strict_json(observed[0][0])[0];ds=documents(native);qs=hash_queries(ds['graph.json'],ds['hash-vectors.json']);payload=canonical(qs)
    observed=[timed_input(cmd,payload,os.environ.copy())for _ in range(2)];require(observed[0][0]==observed[1][0],'two actual Rust release hashes');results['rust-hashes']=observed[0][0];runs['rust-hashes']=[r for _,r in observed]
    inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/'host-toolchain.json').read_bytes()))
    env={**os.environ,'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOTELEMETRY':'off','CGO_ENABLED':'0','GOFLAGS':'','GOEXPERIMENT':'','GOCACHE':'/private/tmp/mpk-go-practical-checker-cache'}
    with tempfile.TemporaryDirectory(prefix='mpk-release-hashes-',dir='/private/tmp')as t:
        exe=Path(t)/'hashes';b=subprocess.run([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(exe),str(HERE/'release_hashes.go')],cwd=ROOT,env=env,capture_output=True,timeout=300)
        require(b.returncode==0 and not b.stderr,'independent Go release hash observer build')
        observed=[timed_input([exe],payload,env)for _ in range(2)];require(observed[0][0]==observed[1][0],'two independent Go release hashes');results['go-hashes']=observed[0][0];runs['go-hashes']=[r for _,r in observed]
    require(before==source_inputs(),'release inputs fixed throughout execution')
    if args.write:
        for n,v in ds.items():(RECORD/n).write_bytes(canonical(v)+b'\n')
        for n,raw in results.items():(RECORD/(n+'.json')).write_bytes(raw)
    report={'schema':'mpk.go_practical.release_contract_probe.v0','scope':SCOPE,'pending':PENDING,'inputs':inputs(),'native_audit':audit,'host_toolchain_inventory':inventory,'runs':runs,
        'cases':len(ds['vectors.json']),'roots':7,'retained_frontends':4,'retained_toolchains':4,'retained_tuples':5,'required_contexts':6,'hash_vectors':len(qs),
        'new_frontend_executed':False,'native_release_observed':False,'Go_ordinary_elaborated':False,'profile_activated':False,'W04_frozen':False}
    if args.write:(RECORD/'observations.json').write_bytes(json_bytes(report))
    check_retained();print('PASS:',report['cases'],'release cases,7 roots,5 actual retained selections,',len(qs),'independent hashes; no activation/W04 freeze',flush=True)


if __name__=='__main__':main()
