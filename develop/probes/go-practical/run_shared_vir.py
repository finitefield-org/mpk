#!/usr/bin/env python3
"""Replay the shared-module draft against retained native semantics and hashes."""
import argparse
import os
import subprocess
import tempfile
from collections import Counter
from pathlib import Path
from shared_vir_vectors import bases,specs,altered,queries,baseline_queries,GO_FIXTURE,RUST_FIXTURE
from shared_vir import ModuleValidator,legacy_query,CONTRACT_FIELDS,VIR_FIELDS,RETAINED_CONTRACT_FIELDS,KINDS
from shared_project import SCOPE_SCHEMA,SCOPE_DOMAIN,SET_SCHEMA,SET_DOMAIN,CONTRACT_SCHEMA,CONTRACT_DOMAIN,OP_SCHEMA,OP_DOMAIN,BODY_SCHEMA,BODY_DOMAIN
from shared_native_build import build
from body_canonical import canonical,digest
from context_protocol import active_registry
from identity_catalog import SHARED
from run_vir_body import RECORD as PRIOR,TARGET as HASH_TARGET
from run_proof_pilot import ROOT,HERE,DEFAULT_GOROOT,json_bytes,sha,verify_host_toolchain
from run_structural_limits import timed_input
from check_scope_report import strict_json,require,exact
from operation_signatures import ROOT_FIELDS as OP_FIELDS
from vir_body import ROOT_FIELDS as BODY_FIELDS

RECORD=PRIOR.parent/'shared-vir'
DOCS=('schema.json','modules.json','vectors.json','outcomes.json','native-queries.json','baseline-queries.json')
NEW_PYTHON=('shared_project.py','shared_vir.py','shared_vir_vectors.py','shared_native_build.py','run_shared_vir.py','test_shared_vir.py','check_shared_vir_metadata.py')
SCOPE='closed five-profile shared VIR v3/contract v3 draft, whole multi-overlay/default-contract Go model closure and retained native importer/type/WP observations; no independent general source import, source/ordinary proof, complete artifact-family freeze or activation'


def catalogue():
    ids=[(SCOPE_SCHEMA,SCOPE_DOMAIN,'content_sha256'),(SET_SCHEMA,SET_DOMAIN,'content_sha256'),
         (CONTRACT_SCHEMA,CONTRACT_DOMAIN,'contract_hash'),(OP_SCHEMA,OP_DOMAIN,'operation_hash'),(BODY_SCHEMA,BODY_DOMAIN,'body_hash')]
    fields={SCOPE_SCHEMA:['schema','semantic_context','capture_context_id','source_overlay_object_id','source_object_id','function_decl_id','signature_id','error_universe_id','type_definitions','identity_nodes','identity_roots','bindings','inputs','results','headers','content_sha256'],
            SET_SCHEMA:['schema','semantic_context','capture_context','overlay_objects','selected_root_ids','functions','content_sha256'],
            CONTRACT_SCHEMA:['schema','semantic_context','source_scope_sha256','source_overlay_object_id','origin','capture_context_id','function_decl_id','signature_id','termination','requires','ensures','error_inputs','modifies','loops','entry_storage','pending_candidates','contract_hash'],
            OP_SCHEMA:list(OP_FIELDS),BODY_SCHEMA:list(BODY_FIELDS)}
    return {'schema':'mpk.go_practical.shared_vir_catalogue.v0','scope':SCOPE,
        'shared_roots':{'vir':{'schema':SHARED['vir'][0],'domain':SHARED['vir'][1],'fields':list(VIR_FIELDS),'hash_field':'vir_hash'},
                        'contract':{'schema':SHARED['contract'][0],'domain':SHARED['contract'][1],'fields':list(CONTRACT_FIELDS),'hash_field':'contract_hash'}},
        'new_go_identities':[{'schema':s,'domain':d,'fields':fields[s],'hash_field':h} for s,d,h in ids],
        'hash_rule':'exact schema and every required field except its own hash; shared-limit narrow canonical JSON; ASCII domain, NUL, canonical bytes; no implicit omitted/default members',
        'dispatch':[{'semantic_profile':p,'kind':k,'profile_contract':'mpk.profile.vir.'+k+'.v2'} for p,k in KINDS.items()],
        'retained_contract_fields':list(RETAINED_CONTRACT_FIELDS),
        'retained_program_fields':['kind','units'],'go_program_fields':['kind','capture_context','source_scopes','contracts','function_bodies'],
        'go_contract_fields':['kind','value'],
        'origin_union':[{'kind':'default','fields':['kind']},{'kind':'source_overlay','fields':['kind','object_id']}],
        'nested_shapes':'source/type/signature/binding/loop shapes and 70 expression arms unchanged from contract-wire; operation parameters/requirements unchanged from operation-signatures; seven body nodes/three terminators unchanged from vir-body; v2 bodies bind common contract v3 and v2 operations',
        'scope_change':'new source_scope root requires nullable source_overlay_object_id; no fabricated raw overlay for defaults; source_scope_set binds all actual overlay objects, selected roots and all body scopes',
        'whole_project':'private immutable fixture input rebuilt from original Go facts, raw overlays and explicit same-U set; all capture-dependent bindings/loops/types/operations/contracts/bodies share the one capture',
        'default_contract':'partial, no requires, true ensures, all same-U cases at every error input path, empty modifies; unproved candidate requiring actual source-body relation',
        'closure':'exact sorted unique body/contract/scope sets; DFS from every selected root; reject missing callees, cycles and unreachable extras; count all instruction nodes and regions across bodies',
        'retained_oracle':'unchanged production source files copied to an owned private workspace; one appended observation facade calls actual importer/projection/type-validation/WP; original Cargo dependency versions/checksums retained',
        'preservation_scope':'five fixture target contexts, with raw original Go/Rust64 roots and independently formed Rust32/C#/Java old-family inputs; full typed projection and all serialized VC data compared, not just acceptance or hashes',
        'owners':{'producer':'go-tools/go2vir/practical_lower.go (planned)',
                  'importer':'crates/mpk-vc/src/go_practical_artifacts.rs (planned)',
                  'shared_dispatch':'crates/mpk-vc/src/successor_source_artifacts.rs (migration review due)',
                  'vc':'crates/mpk-vc/src/go_practical_vc.rs (planned)',
                  'tests':'develop/probes/go-practical/test_shared_vir.py'},
        'remaining':['complete source-map/manifest/VC/policy/API/release roots and reviewed producer/consumer migrations',
                     'independent general Go source/call/control/lifetime/activation import and all ordinary source/operation/permission/frame/totality proofs',
                     'W04 completion, W05 inactive publication and cumulative T01 gate; all T02–T11 implementation and local release gates']}


def policy_from_outputs(qs,outputs):
    require(len(qs)==len(outputs),'complete retained oracle answers')
    cache={canonical(q):v for q,v in zip(qs,outputs)}
    def policy(q):
        key=canonical(q)
        if key not in cache:raise AssertionError('unobserved retained native query cannot become an assumed rejection')
        return cache[key]
    return policy


def evaluate(data,rows,policy):
    by_id={r['id']:r for r in data};out=[]
    for row in rows:
        base=by_id[row['base']]
        try:
            result=ModuleValidator(policy,base.get('project_spec')).validate(altered(row,base));classification=result['kind']
        except ValueError:result=None;classification='configuration_rejection'
        require((result is not None)==row['expected_accepted'],'shared VIR outcome: '+row['id'])
        out.append({'id':row['id'],'classification':classification,'result':result})
    return out


def check_preservation(data,qs,observed,baselines,baseline_observed):
    policy=policy_from_outputs(qs,observed);by_id={r['id']:r for r in data};summary=[]
    for row,old in zip(baselines,baseline_observed):
        new=by_id[row['id']]['module'];now=policy(legacy_query(new['semantic_context'],new['program']['units']))
        require(old['accepted'] is True and now['accepted'] is True,'retained native positive baseline')
        require(exact(old,now),'exact retained typed projection and VC-data equality: '+row['id'])
        summary.append({'id':row['id'],'projection_sha256':now['projection_sha256'],'vc_sha256':now['vc_sha256']})
    return summary


def hashes(data):
    out=[]
    def add(name,domain,value,omit):
        value={k:v for k,v in value.items() if k!=omit};raw=canonical(value)
        out.append({'id':name,'domain':domain,'raw_json_hex':raw.hex(),'accepted':True,'canonical_hex':raw.hex(),'digest':digest(domain,value)})
    for row in data:
        v=row['module'];name=row['id'];p=v['program'];add(name,SHARED['vir'][1],v,'vir_hash')
        if p['kind']=='go_practical':
            add(name+'-scope-set',SET_DOMAIN,p['source_scopes'],'content_sha256')
            for i,s in enumerate(p['source_scopes']['functions']):add(name+'-scope-'+str(i),SCOPE_DOMAIN,s,'content_sha256')
            for i,b in enumerate(p['function_bodies']):
                add(name+'-body-'+str(i),BODY_DOMAIN,b,'body_hash')
                for j,d in enumerate(b['operation_definitions']):add(name+'-op-'+str(i)+'-'+str(j),OP_DOMAIN,d,'operation_hash')
            cs=p['contracts']
            for i,c in enumerate(cs):add(name+'-normalized-'+str(i),CONTRACT_DOMAIN,c['contract']['value'],'contract_hash')
        else:cs=[f['contracts'] for u in p['units'] for f in u['functions']]
        for i,c in enumerate(cs):add(name+'-contract-'+str(i),SHARED['contract'][1],c,'contract_hash')
    return out


def expected_hashes(data):return [{k:v for k,v in x.items() if k not in {'domain','raw_json_hex'}} for x in hashes(data)]


def inputs():
    rows={p['path']:p['sha256'] for p in strict_json((PRIOR/'observations.json').read_bytes())['inputs']}
    paths=[PRIOR/'observations.json']+[HERE/n for n in NEW_PYTHON]+list((HERE/'shared-native').glob('*.rs'))+[GO_FIXTURE,RUST_FIXTURE,ROOT/'Cargo.toml']
    for name in ('mpk-core','mpk-cert','mpk-kernel','mpk-theory','mpk-vc'):
        paths += [ROOT/'crates'/name/'Cargo.toml']+[p for p in (ROOT/'crates'/name/'src').rglob('*') if p.is_file()]
    paths += [ROOT/'proofs/std'/n/('std-'+n+'.hex') for n in ('bool','eq','logic')]
    paths += [RECORD/n for n in DOCS]
    for p in paths:rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    for p,h in rows.items():require(sha((ROOT/p).read_bytes())==h,'shared/prior source pin: '+p)
    return [{'path':p,'sha256':h} for p,h in sorted(rows.items())]


def check_retained(data=None):
    data=bases() if data is None else data;rows=specs();qs=queries(rows,data);bs=baseline_queries()
    r=strict_json((RECORD/'observations.json').read_bytes());native=strict_json((RECORD/'retained-native.json').read_bytes());old=strict_json((RECORD/'baseline-native.json').read_bytes())
    outcomes=evaluate(data,rows,policy_from_outputs(qs,native));preservation=check_preservation(data,qs,native,bs,old)
    docs=dict(zip(DOCS,(catalogue(),data,rows,outcomes,qs,bs)))
    for name,value in docs.items():require((RECORD/name).read_bytes()==json_bytes(value),'reconstructed shared draft: '+name)
    require(r['scope']==SCOPE and r['inputs']==inputs() and r['preservation']==preservation,'shared scope/source/baseline closure')
    require(r['outcomes']==dict(Counter(x['classification'] for x in outcomes)) and r['hash_vectors']==len(hashes(data)),'shared complete outcomes/hash coverage')
    for name in ('retained','baseline','rust','go'):
        path=RECORD/({'retained':'retained-native','baseline':'baseline-native'}.get(name,name+'-hashes')+'.json');raw=path.read_bytes()
        require(len(r['native_runs'][name])==2 and all(x['exit_code']==0 and x['stdout_sha256']==sha(raw) for x in r['native_runs'][name]),'two complete identical native runs: '+name)
        if name in {'rust','go'}:require(exact(strict_json(raw),expected_hashes(data)),'shared full-limit canonical/hash equality')
    for k in ('shared_family_frozen','independent_source_import','dynamic_invocation_proofs_checked','ordinary_semantic_proofs_checked','profile_activated'):require(r[k] is False,'explicit shared boundary: '+k)
    return r


def main():
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group(required=True);g.add_argument('--write',action='store_true');g.add_argument('--check',action='store_true');args=p.parse_args()
    data=bases();rows=specs();qs=queries(rows,data);bs=baseline_queries();exe,audit=build()
    outputs={};runs={};env={**os.environ,'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOTELEMETRY':'off','CGO_ENABLED':'0','GOFLAGS':'','GOEXPERIMENT':'','GOCACHE':'/private/tmp/mpk-go-practical-checker-cache'}
    for name,q in (('retained',qs),('baseline',[x['query'] for x in bs])):
        observed=[timed_input([exe],json_bytes({'registry':active_registry(),'queries':q}),env) for _ in range(2)]
        require(observed[0][0]==observed[1][0],'two identical native importer/type/WP runs')
        outputs[name]=observed[0][0];runs[name]=[x for _,x in observed]
    outcomes=evaluate(data,rows,policy_from_outputs(qs,strict_json(outputs['retained'])))
    preservation=check_preservation(data,qs,strict_json(outputs['retained']),bs,strict_json(outputs['baseline']))
    docs=dict(zip(DOCS,(catalogue(),data,rows,outcomes,qs,bs)))
    if args.write:
        RECORD.mkdir(exist_ok=True)
        for name,value in docs.items():(RECORD/name).write_bytes(json_bytes(value))
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/'host-toolchain.json').read_bytes()))
    result=subprocess.run(['cargo','build','--release','--offline','--locked','--manifest-path',str(HERE/'body-hashes/Cargo.toml')],cwd=ROOT,env={**env,'CARGO_TARGET_DIR':str(HASH_TARGET)},capture_output=True,timeout=300)
    require(result.returncode==0 and b'warning:' not in result.stderr,'unchanged shared hash observer build')
    binaries={'retained':sha(exe.read_bytes())};payload=json_bytes([{k:x[k] for k in ('id','domain','raw_json_hex')} for x in hashes(data)])
    with tempfile.TemporaryDirectory(prefix='mpk-shared-hashes-',dir='/private/tmp') as temp:
        go=Path(temp)/'hashes';rust=HASH_TARGET/'release/go-practical-body-hashes'
        result=subprocess.run([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(go),str(HERE/'body_hashes.go')],cwd=ROOT,env=env,capture_output=True,timeout=300)
        require(result.returncode==0 and not result.stderr,'unchanged Go shared hash observer build')
        for name,bin_ in (('rust',rust),('go',go)):
            binaries[name]=sha(bin_.read_bytes());observed=[timed_input([bin_],payload,env) for _ in range(2)]
            require(observed[0][0]==observed[1][0] and exact(strict_json(observed[0][0]),expected_hashes(data)),'three-language shared canonical/hash agreement: '+name)
            outputs[name]=observed[0][0];runs[name]=[x for _,x in observed]
        require(before==inputs() and binaries=={'retained':sha(exe.read_bytes()),'rust':sha(rust.read_bytes()),'go':sha(go.read_bytes())},'shared source/native binary stability')
    report={'schema':'mpk.go_practical.shared_vir_probe.v0','scope':SCOPE,'inputs':before,'host_toolchain_inventory':inventory,'native_binaries':binaries,'native_runs':runs,'retained_source_audit':audit,
            'hash_vectors':len(hashes(data)),'outcomes':dict(Counter(x['classification'] for x in outcomes)),'preservation':preservation,'native_queries':len(qs),'direct_child_parallelism':1,
            'shared_family_frozen':False,'independent_source_import':False,'dynamic_invocation_proofs_checked':False,'ordinary_semantic_proofs_checked':False,'profile_activated':False}
    if args.write:
        for name,raw in outputs.items():(RECORD/({'retained':'retained-native','baseline':'baseline-native'}.get(name,name+'-hashes')+'.json')).write_bytes(raw)
        (RECORD/'observations.json').write_bytes(json_bytes(report))
    old=check_retained(data);require(old['native_binaries']==binaries and old['retained_source_audit']==audit,'reproduced shared native binaries/source bridge')
    print(f"PASS:{len(data)} modules/{len(rows)} cases,{len(qs)} actual retained queries,{len(hashes(data))} Python/Rust/Go hashes; exact typed projection/VC equality in five retained target contexts; Go model closure unproved",flush=True)


if __name__=='__main__':main()
