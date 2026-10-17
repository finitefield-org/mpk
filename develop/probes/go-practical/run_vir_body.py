#!/usr/bin/env python3
"""Replay structured typed body candidates and full-limit canonical observers."""
import argparse
import copy
import os
import subprocess
import tempfile
from collections import Counter
from pathlib import Path
from vir_body_vectors import model,components,functions,vector_specs,alter,catalogue
from vir_body import ROOT,HERE,RECORD,DOMAIN,validate_body,BodyValidator
from operation_signatures import Signatures
from contract_scope import Scope
from body_canonical import canonical,parse,digest
from run_proof_pilot import DEFAULT_GOROOT,json_bytes,sha,verify_host_toolchain
from run_structural_limits import timed_input
from check_scope_report import strict_json,require,exact

PRIOR=RECORD.parent/'operation-signatures/observations.json'
TARGET=Path('/private/tmp/mpk-go-practical-body-hash-target')
NEW_PYTHON=('body_canonical.py','vir_body.py','vir_body_build.py','vir_body_vectors.py','test_vir_body.py','run_vir_body.py','check_vir_body_metadata.py')
DOCS=('schema.json','components.json','functions.json','vectors.json','outcomes.json')
SCOPE='typed Go function-body and56 component graph proposals with static SSA/state/control/capture/invocation provenance; no complete shared VIR/source/ordinary semantic proof or activation'


def boundaries():
    rows=[]
    def add(name,raw,expected):rows.append({'id':name,'raw_json_hex':raw.hex(),'expected_accepted':expected})
    for n in (64,65,256,257):add('depth-'+str(n),b'['*n+b'0'+b']'*n,n<=256)
    for n in (256,257):add('empty-depth-'+str(n),b'['*n+b']'*n,n<=256)
    add('duplicate',b'{"a":0,"a":1}',False);add('unsafe-integer',b'9007199254740992',False)
    add('unicode-order',b'{"\\ue000":0,"\\ud83d\\ude00":1}',True);add('unpaired',b'"\\ud800"',False)
    return rows


def corpus():
    m=model();cs=components(m);fs=functions(m);specs=vector_specs();out=[]
    for spec in specs:
        row=fs[spec['base']];mm=m if row['model']=='scopes' else Signatures(Scope(row['raw'],row['universe']))
        try:validate_body(mm,alter(spec,fs,m));result='typed_unproved'
        except ValueError:result='configuration_rejection'
        require(result==spec['expected'],'body classification: '+spec['id']);out.append({'id':spec['id'],'classification':result})
    schema=catalogue();schema['canonical_boundaries']=boundaries()
    return dict(zip(DOCS,(schema,cs,fs,specs,out)))


def inputs():
    prior=strict_json(PRIOR.read_bytes());rows={p['path']:p['sha256'] for p in prior['inputs']}
    paths=[PRIOR]+[HERE/n for n in NEW_PYTHON]+[HERE/'body_hashes.go']+[p for p in (HERE/'body-hashes').rglob('*') if p.is_file()]+[RECORD/n for n in DOCS]
    for p in paths:rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    for path,h in rows.items():require(sha((ROOT/path).read_bytes())==h,'retained/current source pin: '+path)
    return [{'path':p,'sha256':h} for p,h in sorted(rows.items())]


def hashes(docs):
    rows=[]
    def add(name,domain,value):
        raw=canonical(value);rows.append({'id':name,'domain':domain,'raw_json_hex':raw.hex(),'accepted':True,'canonical_hex':raw.hex(),'digest':digest(domain,value)})
    for r in docs['functions.json']:add(r['id'],DOMAIN,{k:v for k,v in r['record'].items() if k!='body_hash'})
    for r in docs['components.json']:add('component-'+r['op'],'MPK-GO-IDENTITY-PILOT-1.0',r)
    for row in boundaries():
        raw=bytes.fromhex(row['raw_json_hex']);item={'id':row['id'],'domain':DOMAIN,'raw_json_hex':row['raw_json_hex'],'accepted':row['expected_accepted']}
        try:value=parse(raw);encoded=canonical(value);item.update(canonical_hex=encoded.hex(),digest=digest(DOMAIN,value));ok=True
        except ValueError:ok=False
        require(ok==row['expected_accepted'],'registered canonical boundary');rows.append(item)
    return rows


def expected(docs):return [{k:v for k,v in r.items() if k not in {'domain','raw_json_hex'}} for r in hashes(docs)]


def check_retained(docs=None):
    docs=corpus() if docs is None else docs;r=strict_json((RECORD/'observations.json').read_bytes())
    require(r['scope']==SCOPE and r['inputs']==inputs(),'complete body scope and input pins')
    for name,value in docs.items():require((RECORD/name).read_bytes()==json_bytes(value),'canonical body data: '+name)
    require(r['outcomes']==dict(Counter(x['classification'] for x in docs['outcomes.json'])) and r['hash_vectors']==69,'complete body outcomes and hashes')
    for lang in ('rust','go'):
        raw=(RECORD/(lang+'-hashes.json')).read_bytes();require(exact(strict_json(raw),expected(docs)),'complete body canonical/digest agreement')
        require(len(r['native_runs'][lang])==2 and all(x['exit_code']==0 and x['stdout_sha256']==sha(raw) for x in r['native_runs'][lang]),'two exact native outputs')
    for key in ('shared_family_frozen','independent_source_import','dynamic_invocation_proofs_checked','ordinary_semantic_proofs_checked','profile_activated'):require(r[key] is False,'body scope boundary: '+key)
    return r


def main():
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group(required=True);g.add_argument('--write',action='store_true');g.add_argument('--check',action='store_true');args=p.parse_args()
    docs=corpus()
    if args.write:
        RECORD.mkdir(exist_ok=True)
        for name,value in docs.items():(RECORD/name).write_bytes(json_bytes(value))
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/'host-toolchain.json').read_bytes()))
    build=subprocess.run(['cargo','build','--release','--offline','--locked','--manifest-path',str(HERE/'body-hashes/Cargo.toml')],cwd=ROOT,env={**os.environ,'CARGO_TARGET_DIR':str(TARGET)},capture_output=True,timeout=300)
    require(build.returncode==0 and b'warning:' not in build.stderr,'new body observer build: '+build.stderr.decode())
    rust=TARGET/'release/go-practical-body-hashes';native={};runs={}
    payload=json_bytes([{k:r[k] for k in ('id','domain','raw_json_hex')} for r in hashes(docs)])
    with tempfile.TemporaryDirectory(prefix='mpk-vir-body-',dir='/private/tmp') as temp:
        go=Path(temp)/'hashes';env={**os.environ,'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOTELEMETRY':'off','CGO_ENABLED':'0','GOFLAGS':'','GOEXPERIMENT':'','GOCACHE':'/private/tmp/mpk-go-practical-checker-cache'}
        build=subprocess.run([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(go),str(HERE/'body_hashes.go')],cwd=ROOT,env=env,capture_output=True,timeout=300)
        require(build.returncode==0 and not build.stderr,'new Go body observer build')
        binaries={'rust':sha(rust.read_bytes()),'go':sha(go.read_bytes())}
        for lang,exe in (('rust',rust),('go',go)):
            observed=[timed_input([exe],payload,env) for _ in range(2)]
            require(observed[0][0]==observed[1][0] and exact(strict_json(observed[0][0]),expected(docs)),'full-limit body canonical/hash agreement: '+lang)
            native[lang]=observed[0][0];runs[lang]=[x for _,x in observed]
        require(before==inputs() and binaries=={'rust':sha(rust.read_bytes()),'go':sha(go.read_bytes())},'body source/binary stability')
    report={'schema':'mpk.go_practical.vir_body_probe.v0','scope':SCOPE,'pending':docs['schema.json']['remaining'],'inputs':before,'host_toolchain_inventory':inventory,'native_binaries':binaries,'native_runs':runs,'hash_vectors':69,'outcomes':dict(Counter(x['classification'] for x in docs['outcomes.json'])),'direct_child_parallelism':1,
            'shared_family_frozen':False,'independent_source_import':False,'dynamic_invocation_proofs_checked':False,'ordinary_semantic_proofs_checked':False,'profile_activated':False}
    if args.write:
        for lang,raw in native.items():(RECORD/(lang+'-hashes.json')).write_bytes(raw)
        (RECORD/'observations.json').write_bytes(json_bytes(report))
    old=check_retained(docs);require(old['native_binaries']==binaries,'reproduced body observer binaries')
    print('PASS:56 typed components,3 function candidates,20 graph cases (3 unproved/17 rejected),69 Python/Rust/Go hashes with256-depth boundaries; no shared/source/dynamic/ordinary completion',flush=True)


if __name__=='__main__':main()
