#!/usr/bin/env python3
"""Replay common VC/certificate links and source-bound Go theorem demands."""
import argparse
import os
import subprocess
import tempfile
from collections import Counter
from pathlib import Path
from proof_artifact_vectors import (bases,source_bases,generator_queries,baseline_queries,specs,native_queries,base_query,
    source_validator,altered,policy_from_outputs)
from proof_artifacts import ProofArtifacts,RETAINED_VC_FIELDS,GO_VC_FIELDS,CERT_FIELDS,SKELETON_FIELDS
from proof_components import fragments
from go_obligations import SCHEMA as PLAN_SCHEMA,DOMAIN as PLAN_DOMAIN,REQUEST_SCHEMA,REQUEST_DOMAIN,FAMILIES,OrdinaryElaborationPending
from proof_native_build import build
from proof_canonical import canonical,digest,parse
from run_capture_artifacts import RECORD as PRIOR
from identity_catalog import SHARED
from context_protocol import active_registry
from run_proof_pilot import ROOT,HERE,DEFAULT_GOROOT,json_bytes,sha,verify_host_toolchain
from run_structural_limits import timed_input
from check_scope_report import require,strict_json,exact

RECORD=PRIOR.parent/'proof-artifacts'
DOCS=('schema.json','artifacts.json','components.json','vectors.json','outcomes.json','native-queries.json','baseline-queries.json')
NEW_PYTHON=('proof_canonical.py','proof_artifacts.py','proof_artifact_vectors.py','go_obligations.py','proof_components.py','proof_native_build.py','run_proof_artifacts.py','test_proof_artifacts.py','check_proof_artifact_metadata.py')
SCOPE='common VC v4/certificate-source links,actual retained VC/skeleton regeneration and closed source-bound Go theorem-demand draft; Go ordinary proposition expansion/final skeleton and all source/ordinary proofs remain due; no activation'


def boundaries():
    rows=[]
    for depth in (256,257,767,768,769):rows.append({'id':'depth-'+str(depth),'raw_json_hex':(b'['*depth+b'0'+b']'*depth).hex(),'accepted':depth<=768})
    for depth in (768,769):rows.append({'id':'empty-depth-'+str(depth),'raw_json_hex':(b'['*depth+b']'*depth).hex(),'accepted':depth<=768})
    for name,raw,accepted in (('duplicate',b'{"a":0,"a":1}',False),('unsafe-integer',b'9007199254740992',False),('unicode-order',b'{"\\ue000":0,"\\ud83d\\ude00":1}',True),('unpaired',b'"\\ud800"',False)):
        rows.append({'id':name,'raw_json_hex':raw.hex(),'accepted':accepted})
    return rows


def catalogue():
    return {'schema':'mpk.go_practical.proof_artifact_catalogue.v0','scope':SCOPE,
        'roots':[{'schema':SHARED['vc'][0],'domain':SHARED['vc'][1],'retained_fields':list(RETAINED_VC_FIELDS),'go_fields':list(GO_VC_FIELDS),'hash_field':'vc_hash'},
                 {'schema':SHARED['certificate_manifest'][0],'domain':SHARED['certificate_manifest'][1],'fields':list(CERT_FIELDS),'hash_field':'certificate_manifest_hash'},
                 {'schema':SHARED['vc_skeleton'][0],'domain':None,'fields':list(SKELETON_FIELDS),'hash_field':None},
                 {'schema':PLAN_SCHEMA,'domain':PLAN_DOMAIN,'hash_field':'program_hash'},
                 {'schema':REQUEST_SCHEMA,'domain':REQUEST_DOMAIN,'hash_field':'request_hash'}],
        'dispatch':'full validated context/source and compiled profile vc/manifest contract v2; exact context-selected closed root fieldset; no caller extensions',
        'hash_rule':'all required root fields including schema/context/source/theorem demand or ordinary functions,except own hash; domain,NUL,narrow JCS bytes',
        'transport':{'input_json_bytes':268435456,'canonical_json_bytes':268435456,'json_container_depth':768,'decoded_string_bytes':1048576},
        'depth_preservation':'retained functions and theorem_declarations remain directly at root,so valid old VC/skeleton terms gain no JSON container; unchanged canonicalizer maximum768 remains authoritative',
        'retained_regeneration':'actual public generate/import_successor_vc and emit/import_successor_vc_skeleton under fully validated old-family source/profile envelope; compare complete regenerated functions/declarations,not just native accepted flag',
        'certificate_dag':'VIR/map -> frontend manifest -> VC -> certificate manifest; certificate root references exact frontend schema/hash and validated common VC schema/hash; VC never references certificate manifest',
        'skeleton_boundary':'skeleton is an exact deterministic VC-derived object without own public hash; private pilot digest vectors cover bytes; it contains statement declarations,not proofs',
        'go_request_families':list(FAMILIES),
        'go_request_meaning':'registered theorem demand over all reachable dynamic activations under exact explicit entry/domain/contract assumptions; references entire captured body/source/contract,not one sample execution; no arbitrary predicates or proof-status flags',
        'go_operation_meaning':'member category/ordinal resolves only through the exact typed body site/concrete operation definition; access demands retain full current/prefix-state and location/check recipes; body/source hashes pin all type,operand,state,control and invocation data',
        'go_function_meaning':'always source-body refinement,entry representation,frame,partial correctness,all requires/ensures definedness and each requested postcondition; zero-node functions still require source/function goals',
        'go_loop_meaning':'source-contract loop invariants have definedness,initialization,preservation,exit; candidate ranks have domain and continuing-backedge decrease; pending total variants stay pending; region-control demands cover all normal/abrupt paths and resume continuations',
        'go_call_meaning':'exact same-U source-body/contract dependencies and caller-side precondition/relation/conditional call-totality demands; default partial contracts supply no callee theorem or global termination promise; adapters require exact registered source relation',
        'go_ordinary_boundary':'current private foundation has no complete registered source/operation theorem elaborator; all request IDs are unelaborated and Go skeleton emission raises OrdinaryElaborationPending; a caller expanded/ready flag cannot select a trusted context',
        'go_remaining_interface':'final registered ordinary-context expansion,statement/term bindings and Go skeleton representation remain unfrozen W04 work; typed demand requests are not ordinary propositions or discharged proofs',
        'components':'56 project-bound local operation fragments yield every existing member/access demand; fragments are not complete source functions or application VCs',
        'canonical_boundaries':boundaries(),
        'owners':{'generator':'crates/mpk-vc/src/go_practical_vc.rs (planned)','ordinary_elaboration':'crates/mpk-vc/src/go_practical_ordinary.rs (planned)',
                  'importer':'crates/mpk-vc/src/go_practical_artifacts.rs (planned)','shared_vc':'crates/mpk-vc/src/successor_vc.rs (migration review due)',
                  'tests':'develop/probes/go-practical/test_proof_artifacts.py'},
        'remaining':['complete registered Go ordinary theorem signatures/definitions/statement expansion and Go skeleton/program-assembly closure',
                     'frontend-envelope/policy/API/release roots and complete reviewed producer/consumer migrations',
                     'independent general source/AST/control/lifetime/activation correspondence and all ordinary operation/permission/frame/totality proofs; W04/W05/T01 and full T02–T11']}


def evaluate(data,rows,policy):
    by_id={b['id']:b for b in data};validators={b['id']:ProofArtifacts(source_validator(b),policy,base_query(b)) for b in data};out=[]
    for row in rows:
        b=by_id[row['base']];a=validators[b['id']];v=altered(row,b)
        try:
            args=(b['module'],b['source_map'],b['manifest'])
            if row['stage']=='vc':result=a.validate_vc(*args,v)
            elif row['stage']=='certificate':result=a.validate_certificate_manifest(*args,b['vc'],v)
            elif row['stage']=='skeleton':result=a.validate_skeleton(*args,b['vc'],v)
            else:a.validate_vc(*args,b['vc']);a.expected_skeleton(b['vc']);raise AssertionError('pending Go skeleton emitted')
            classification=result['kind']
        except OrdinaryElaborationPending:result=None;classification='ordinary_elaboration_pending'
        except ValueError:result=None;classification='configuration_rejection'
        require(classification==row['expected'],'proof artifact classification: '+row['id'])
        out.append({'id':row['id'],'classification':classification,'result':result})
    return out


def preservation(data,policy,bs,old):
    by_id={b['id']:b for b in data};rows=[];require(len(bs)==len(old)==5,'five complete retained proof baselines')
    for b,r in zip(bs,old):
        now=policy(base_query(by_id[b['id']]))
        require(r['accepted'] is True and exact(now,r),'full actual VC/skeleton/certificate-source equality: '+b['id'])
        rows.append({'id':b['id'],'vc_sha256':sha(canonical(now['vc'])),'skeleton_sha256':sha(canonical(now['skeleton'])),'certificate_manifest_sha256':sha(canonical(now['certificate_manifest']))})
    return rows


def hashes(data,components):
    out=[]
    def add(name,domain,value,omit=None):
        value={k:v for k,v in value.items() if k!=omit};raw=canonical(value)
        out.append({'id':name,'domain':domain,'raw_json_hex':raw.hex(),'accepted':True,'canonical_hex':raw.hex(),'digest':digest(domain,value)})
    for b in data:
        add(b['id']+'-vc',SHARED['vc'][1],b['vc'],'vc_hash');add(b['id']+'-certificate-source',SHARED['certificate_manifest'][1],b['certificate_manifest'],'certificate_manifest_hash')
        if b['skeleton'] is not None:add(b['id']+'-private-skeleton','MPK-GO-IDENTITY-PILOT-1.0',b['skeleton'])
        if 'obligations' in b['vc']:
            plan=b['vc']['obligations'];add(b['id']+'-go-program',PLAN_DOMAIN,plan,'program_hash')
            for f in plan['functions']:
                for i,r in enumerate(f['requests']):add(b['id']+'-request-'+f['function_decl_id']+'-'+str(i),REQUEST_DOMAIN,r,'request_hash')
    for c in components:add('fragment-'+c['op'],'MPK-GO-IDENTITY-PILOT-1.0',c)
    for b in boundaries():
        row={**b,'domain':SHARED['vc'][1]}
        try:value=parse(bytes.fromhex(b['raw_json_hex']));raw=canonical(value);row.update(canonical_hex=raw.hex(),digest=digest(row['domain'],value));ok=True
        except ValueError:ok=False
        require(ok==b['accepted'],'registered proof canonical boundary');out.append(row)
    return out


def expected_hashes(data,components):return [{k:v for k,v in x.items() if k not in {'domain','raw_json_hex'}} for x in hashes(data,components)]


def inputs():
    old=strict_json((PRIOR/'observations.json').read_bytes());rows={r['path']:r['sha256'] for r in old['inputs']}
    paths=[PRIOR/'observations.json']+[HERE/n for n in NEW_PYTHON]+[HERE/'proof_hashes.go']+[p for p in (HERE/'proof-native').rglob('*') if p.is_file()]+[RECORD/n for n in DOCS]
    for p in paths:rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    for p,h in rows.items():require(sha((ROOT/p).read_bytes())==h,'proof/current predecessor input pin: '+p)
    return [{'path':p,'sha256':h} for p,h in sorted(rows.items())]


def check_retained():
    r=strict_json((RECORD/'observations.json').read_bytes());qs=strict_json((RECORD/'native-queries.json').read_bytes());native=strict_json((RECORD/'retained-native.json').read_bytes());policy=policy_from_outputs(qs,native)
    source=source_bases();data=bases({b['id']:policy(base_query(b)) for b in source if 'project_spec' not in b},source);cs=fragments();rows=specs();bs=baseline_queries(data)
    expected_qs=native_queries(data,rows);require(exact(qs,expected_qs),'complete proof native query coverage')
    old=strict_json((RECORD/'baseline-native.json').read_bytes());p=preservation(data,policy,bs,old);out=evaluate(data,rows,policy)
    docs=dict(zip(DOCS,(catalogue(),data,cs,rows,out,qs,bs)))
    for n,v in docs.items():require((RECORD/n).read_bytes()==json_bytes(v),'reconstructed proof artifact data: '+n)
    require(r['scope']==SCOPE and r['inputs']==inputs() and r['preservation']==p,'proof scope/source/baseline closure')
    require(r['outcomes']==dict(Counter(x['classification'] for x in out)) and r['hash_vectors']==len(hashes(data,cs)),'complete proof classification/hash counts')
    for name in ('retained','baseline','rust','go'):
        raw=(RECORD/({'retained':'retained-native','baseline':'baseline-native'}.get(name,name+'-hashes')+'.json')).read_bytes()
        require(len(r['native_runs'][name])==2 and all(x['exit_code']==0 and x['stdout_sha256']==sha(raw) for x in r['native_runs'][name]),'two identical complete proof native runs')
        if name in ('rust','go'):require(exact(strict_json(raw),expected_hashes(data,cs)),'full proof canonical/hash equality')
    for key in ('shared_family_frozen','go_ordinary_terms_materialized','go_skeleton_emitted','source_proofs_checked','ordinary_semantic_proofs_checked','profile_activated'):require(r[key] is False,'explicit proof stage boundary: '+key)
    return r


def main():
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group(required=True);g.add_argument('--write',action='store_true');g.add_argument('--check',action='store_true');args=p.parse_args()
    source=source_bases();initial=generator_queries(source);exe,audit=build();env={**os.environ,'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOTELEMETRY':'off','CGO_ENABLED':'0','GOFLAGS':'','GOEXPERIMENT':'','GOCACHE':'/private/tmp/mpk-go-practical-checker-cache'}
    raw,_=timed_input([exe],json_bytes({'registry':active_registry(),'queries':[r['query'] for r in initial]}),env)
    generated={r['id']:v for r,v in zip(initial,strict_json(raw))};require(all(v['accepted'] is True for v in generated.values()),'all retained public VC bases generated')
    data=bases(generated,source);cs=fragments();rows=specs();qs=native_queries(data,rows);bs=baseline_queries(data);outputs={};runs={}
    for name,q in (('retained',qs),('baseline',[r['query'] for r in bs])):
        observed=[timed_input([exe],json_bytes({'registry':active_registry(),'queries':q}),env) for _ in range(2)]
        require(observed[0][0]==observed[1][0],'two identical public VC/skeleton/source observations');outputs[name]=observed[0][0];runs[name]=[x for _,x in observed]
    policy=policy_from_outputs(qs,strict_json(outputs['retained']));out=evaluate(data,rows,policy);preserved=preservation(data,policy,bs,strict_json(outputs['baseline']))
    docs=dict(zip(DOCS,(catalogue(),data,cs,rows,out,qs,bs)))
    if args.write:
        RECORD.mkdir(exist_ok=True)
        for name,value in docs.items():(RECORD/name).write_bytes(json_bytes(value))
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/'host-toolchain.json').read_bytes()))
    hash_rows=hashes(data,cs);hash_input=[{k:r[k] for k in ('id','domain','raw_json_hex')} for r in hash_rows];binaries={'retained':sha(exe.read_bytes())}
    observed=[timed_input([exe],json_bytes({'registry':active_registry(),'queries':hash_input}),env) for _ in range(2)]
    require(observed[0][0]==observed[1][0] and exact(strict_json(observed[0][0]),expected_hashes(data,cs)),'unchanged Rust canonicalizer/full768 proof hash agreement')
    outputs['rust']=observed[0][0];runs['rust']=[x for _,x in observed]
    with tempfile.TemporaryDirectory(prefix='mpk-proof-hashes-',dir='/private/tmp') as temp:
        go=Path(temp)/'hashes';p=subprocess.run([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(go),str(HERE/'proof_hashes.go')],cwd=ROOT,env=env,capture_output=True,timeout=300)
        require(p.returncode==0 and not p.stderr,'new independent Go768 hash observer build')
        binaries['go']=sha(go.read_bytes());observed=[timed_input([go],json_bytes(hash_input),env) for _ in range(2)]
        require(observed[0][0]==observed[1][0] and exact(strict_json(observed[0][0]),expected_hashes(data,cs)),'independent Go proof canonical/hash equality')
        outputs['go']=observed[0][0];runs['go']=[x for _,x in observed]
        require(before==inputs() and binaries=={'retained':sha(exe.read_bytes()),'go':sha(go.read_bytes())},'proof source/native binary stability')
    report={'schema':'mpk.go_practical.proof_artifact_probe.v0','scope':SCOPE,'inputs':before,'host_toolchain_inventory':inventory,'native_binaries':binaries,'native_runs':runs,'public_source_audit':audit,
        'hash_vectors':len(hash_rows),'outcomes':dict(Counter(x['classification'] for x in out)),'native_queries':len(qs),'local_fragments':len(cs),'local_demands':sum(len(r['requirements']) for c in cs for r in c['local_requirements']),
        'go_requests':sum(len(f['requests']) for b in data if 'obligations' in b['vc'] for f in b['vc']['obligations']['functions']),'preservation':preserved,'direct_child_parallelism':1,
        'shared_family_frozen':False,'go_ordinary_terms_materialized':False,'go_skeleton_emitted':False,'source_proofs_checked':False,'ordinary_semantic_proofs_checked':False,'profile_activated':False}
    if args.write:
        for name,raw in outputs.items():(RECORD/({'retained':'retained-native','baseline':'baseline-native'}.get(name,name+'-hashes')+'.json')).write_bytes(raw)
        (RECORD/'observations.json').write_bytes(json_bytes(report))
    old=check_retained();require(old['native_binaries']==binaries and old['public_source_audit']==audit,'fresh proof native source/binary reproduction')
    print(f"PASS:8 proof bundles/{len(rows)} cases,{len(qs)} actual native queries,{len(hash_rows)} Python/Rust/Go hashes,56 local fragments/416 demands,169 Go theorem requests; five retained contexts exact; Go ordinary elaboration/skeleton remains pending",flush=True)


if __name__=='__main__':main()
