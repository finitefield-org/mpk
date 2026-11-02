#!/usr/bin/env python3
"""Replay common capture artifacts against actual retained public validators."""
import argparse
import copy
import os
import subprocess
import tempfile
from collections import Counter
from pathlib import Path
from capture_artifact_vectors import bases,specs,queries,altered,release_fixture,retained_data,GO_DIR,RUST_DIR,GO_ROOT,RUST_ROOT
from capture_artifacts import Artifacts,MAP_FIELDS,MAN_FIELDS,RETAINED_MAN_FIELDS,GO_MAN_FIELDS,INVENTORY_SCHEMA,INVENTORY_DOMAIN,MAP_CANONICAL_MAX,MAP_ENTRIES_MAX,MAN_CANONICAL_MAX
from capture_native_build import build
from shared_vir_vectors import baseline_queries as vir_baselines
from run_shared_vir import RECORD as PRIOR,policy_from_outputs
from run_vir_body import TARGET as HASH_TARGET
from body_canonical import canonical,digest
from context_protocol import active_registry
from identity_catalog import SHARED
from run_proof_pilot import ROOT,HERE,DEFAULT_GOROOT,json_bytes,sha,verify_host_toolchain
from run_structural_limits import timed_input
from check_scope_report import strict_json,require,exact

RECORD=PRIOR.parent/'capture-artifacts'
DOCS=('schema.json','artifacts.json','vectors.json','outcomes.json','native-queries.json','baseline-queries.json')
NEW_PYTHON=('capture_artifacts.py','capture_artifact_vectors.py','capture_native_build.py','run_capture_artifacts.py','test_capture_artifacts.py','check_capture_artifact_metadata.py')
SCOPE='closed common source-map/frontend-manifest and Go input-inventory draft with exact captured-byte/model linkage and retained public native validation; no general source import, AST/ordinary proof, registered release/tool provenance, certificate manifest or activation'


def catalogue():
    return {'schema':'mpk.go_practical.capture_artifact_catalogue.v0','scope':SCOPE,
        'roots':[{'schema':SHARED['source_map'][0],'domain':SHARED['source_map'][1],'fields':list(MAP_FIELDS),'hash_field':'source_map_hash'},
                 {'schema':SHARED['frontend_manifest'][0],'domain':SHARED['frontend_manifest'][1],'fields':list(MAN_FIELDS),'hash_field':'source_manifest_hash'},
                 {'schema':INVENTORY_SCHEMA,'domain':INVENTORY_DOMAIN,'fields':['schema','semantic_context','capture_context_id','objects','content_sha256'],'hash_field':'content_sha256'}],
        'hash_rule':'all exact required fields,including schema and complete context/payload,except own hash; narrow canonical JSON,ASCII domain,NUL,canonical bytes',
        'dispatch':'validated complete common VIR/context first; exact compiled source_map/manifest profile contract v2 and matching closed five-profile payload arm; no language-only or caller extension dispatch',
        'map_limits':{'canonical_json_bytes':MAP_CANONICAL_MAX,'entries':MAP_ENTRIES_MAX,'input_json_bytes':268435456,'json_nesting':256,'decoded_string_bytes':1048576},
        'manifest_limits':{'canonical_json_bytes':MAN_CANONICAL_MAX,'input_json_bytes':268435456,'json_nesting':256,'decoded_string_bytes':1048576,'inputs':32768,'units':256},
        'retained_map_fields':['kind','entries'],'go_map_fields':['kind','capture_context_id','entries'],
        'go_references':[{'kind':'function','fields':['kind','function_decl_id']},
                         {'kind':'instruction','fields':['kind','function_decl_id','node_id']},
                         {'kind':'terminator','fields':['kind','function_decl_id','region_id']}],
        'go_origin_fields':['kind','source_object_id','start_byte','end_byte'],'go_origin_kind':'captured_span',
        'go_map_rule':'one entry per function,every one of seven body node arms and every region terminator,including scope children and resume continuations; function/declaration order then function/instruction/terminator and numeric IDs; exact body anchors,terminators inherit enclosing anchor; nonempty UTF8 byte ranges in captured go_source objects',
        'locator_boundary':'byte/source-object locators do not prove AST,lowering,source execution or ordinary semantics; broad inherited spans remain visible as such',
        'retained_manifest_fields':list(RETAINED_MAN_FIELDS),'go_manifest_fields':list(GO_MAN_FIELDS),
        'go_inventory_rule':'all original source objects in capture-ID order,full unchanged source_object v1 records,complete raw bytes/size/SHA check,one ASCII-case-stable path per capability; real overlays only,all capture/source/contract/manifest bounds retained',
        'go_manifest_rule':'exact selection and captured build/module/package/inventory/context/limit/immutable external provenance authority; original authority bytes rebuilt independently of caller-mutated artifact values',
        'release_reference_fixture':release_fixture(),
        'provenance_boundary':'historical fixture tool/frontend identity payloads are untrusted shape/linkage samples; private release reference fixture is incomplete and uninstalled; no observed practical frontend,bundle registry,registered tool provenance or release eligibility is claimed',
        'retained_validation':'actual unchanged public successor VIR,map and frontend-manifest imports; exact captured bytes and external release/synthetic-permission authority; native validator retains profile-specific Java rules; old roots are explicit observation projections,never accepted as common roots',
        'preservation_boundary':'original Go/Rust64 roots and independently formed Rust32/C#/Java fixture baselines compare complete validated old map/manifest values in five target contexts; additional authorized synthetic Go case; no universal preservation/source theorem',
        'certificate_stage':'not accepted through frontend root; certificate_manifest v3 and VC v4 closure/binding remain separate work',
        'owners':{'producer':'go-tools/go2vir/practical_capture.go and practical_lower.go (planned)',
                  'importer':'crates/mpk-vc/src/go_practical_artifacts.rs (planned)',
                  'shared_dispatch':'crates/mpk-vc/src/successor_source_artifacts.rs (migration review due)',
                  'tests':'develop/probes/go-practical/test_capture_artifacts.py'},
        'remaining':['certificate-manifest/VC/policy/API/frontend-envelope/release roots and reviewed producer/consumer migrations',
                     'independent general source capture/import,actual AST/control/lifetime/activation correspondence and all ordinary source/operation/permission/frame/totality proofs',
                     'W04 completion,W05 inactive publication/cumulative T01 gate and all T02–T11 implementation/release gates']}


def module_policy():
    return policy_from_outputs(strict_json((PRIOR/'native-queries.json').read_bytes()),strict_json((PRIOR/'retained-native.json').read_bytes()))


def evaluate(data,rows,policy):
    by_id={x['id']:x for x in data};mp=module_policy()
    validators={b['id']:Artifacts(mp,policy,b['authority'],b.get('project_spec')) for b in data};out=[]
    for row in rows:
        b=by_id[row['base']];a=validators[b['id']];value=altered(row,b)
        try:
            result=a.validate_map(b['module'],value) if row['stage']=='map' else a.validate_manifest(b['module'],b['source_map'],value)
            classification=result['kind']
        except ValueError:result=None;classification='configuration_rejection'
        require((result is not None)==row['expected_accepted'],'capture artifact outcome: '+row['id'])
        out.append({'id':row['id'],'classification':classification,'result':result})
    return out


def baseline_queries(data):
    by_id={x['id']:x for x in data};out=[]
    for base in vir_baselines():
        name=base['id'];b=by_id[name];authority=b['authority'];q=copy.deepcopy(base['query'])
        q.update(stage='manifest',captured_inputs=authority['captured_inputs'],synthetic_permissions=[],expected_release_registry=authority['expected_release_registry'])
        if 'original_vir' in q:
            folder=GO_DIR if name=='go_fixed' else RUST_DIR
            q['original']={'vir':q.pop('original_vir'),'source_map':strict_json((folder/'source-map.json').read_bytes()),'manifest':strict_json((folder/'source-manifest.frontend.json').read_bytes())}
        else:
            # These baseline payloads originate from the old fixtures/independent
            # old VIR inputs,not from a serializer/deserializer round-trip.
            view={'id':name,'module':{'semantic_context':b['module']['semantic_context'],'program':{'kind':b['module']['program']['kind'],'units':q['units']}}}
            payload,entries,_=retained_data(view);payload.pop('kind');payload['release_registry']=authority['expected_release_registry']
            q.update(entries=entries,manifest=payload)
        out.append({'id':name,'query':q})
    return out


def preservation(data,qs,observed,bs,old):
    policy=policy_from_outputs(qs,observed);by_id={x['id']:x for x in data};mp=module_policy();rows=[]
    require(len(bs)==len(old)==5,'complete five-context source artifact baselines')
    for b,r in zip(bs,old):
        v=by_id[b['id']];a=Artifacts(mp,policy,v['authority']);q=a.query(v['module'],v['source_map'],v['manifest']);now=policy(q)
        require(r['accepted'] is True and exact(now,r),'complete retained map/manifest equality: '+b['id'])
        rows.append({'id':b['id'],'validated_map_sha256':sha(canonical(now['source_map'])),'validated_manifest_sha256':sha(canonical(now['manifest']))})
    return rows


def hashes(data):
    rows=[]
    def add(name,domain,value,field=None):
        value={k:v for k,v in value.items() if k!=field};raw=canonical(value)
        rows.append({'id':name,'domain':domain,'raw_json_hex':raw.hex(),'accepted':True,'canonical_hex':raw.hex(),'digest':digest(domain,value)})
    for b in data:
        add(b['id']+'-map',SHARED['source_map'][1],b['source_map'],'source_map_hash')
        add(b['id']+'-frontend',SHARED['frontend_manifest'][1],b['manifest'],'source_manifest_hash')
        if 'project_spec' in b:add(b['id']+'-inventory',INVENTORY_DOMAIN,b['manifest']['manifest']['input_inventory'],'content_sha256')
    add('private-external-release-reference-fixture',SHARED['bundle_registry'][1],release_fixture()['value'])
    return rows


def expected_hashes(data):return [{k:v for k,v in r.items() if k not in {'domain','raw_json_hex'}} for r in hashes(data)]


def inputs():
    rows={p['path']:p['sha256'] for p in strict_json((PRIOR/'observations.json').read_bytes())['inputs']}
    paths=[PRIOR/'observations.json']+[HERE/n for n in NEW_PYTHON]+[p for p in (HERE/'capture-native').rglob('*') if p.is_file()]
    paths += [GO_DIR/n for n in ('source-map.json','source-manifest.frontend.json')]+[RUST_DIR/n for n in ('source-map.json','source-manifest.frontend.json')]
    paths += [GO_ROOT/'go.mod',GO_ROOT/'positive/arith/arith.go',RUST_ROOT/'src/lib.rs']+list((RUST_ROOT/'contracts').glob('*.json'))
    paths += [ROOT/'crates/mpk-vc/tests/successor_vc.rs']+[RECORD/n for n in DOCS]
    for p in paths:rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    for p,h in rows.items():require(sha((ROOT/p).read_bytes())==h,'retained/current capture source pin: '+p)
    return [{'path':p,'sha256':h} for p,h in sorted(rows.items())]


def check_retained(data=None):
    data=bases() if data is None else data;rows=specs();qs=queries(data,rows);bs=baseline_queries(data)
    r=strict_json((RECORD/'observations.json').read_bytes());native=strict_json((RECORD/'retained-native.json').read_bytes());old=strict_json((RECORD/'baseline-native.json').read_bytes())
    outcomes=evaluate(data,rows,policy_from_outputs(qs,native));p=preservation(data,qs,native,bs,old)
    docs=dict(zip(DOCS,(catalogue(),data,rows,outcomes,qs,bs)))
    for name,value in docs.items():require((RECORD/name).read_bytes()==json_bytes(value),'reconstructed capture proposal: '+name)
    require(r['scope']==SCOPE and r['inputs']==inputs() and r['preservation']==p,'capture scope/source/preservation closure')
    require(r['outcomes']==dict(Counter(x['classification'] for x in outcomes)) and r['hash_vectors']==len(hashes(data)),'complete capture outcomes/hash coverage')
    for name in ('retained','baseline','rust','go'):
        raw=(RECORD/({'retained':'retained-native','baseline':'baseline-native'}.get(name,name+'-hashes')+'.json')).read_bytes()
        require(len(r['native_runs'][name])==2 and all(x['exit_code']==0 and x['stdout_sha256']==sha(raw) for x in r['native_runs'][name]),'two complete identical capture native runs')
        if name in {'rust','go'}:require(exact(strict_json(raw),expected_hashes(data)),'complete capture canonical/digest agreement')
    for k in ('shared_family_frozen','general_source_import','ast_correspondence_proofs_checked','ordinary_semantic_proofs_checked','registered_provenance_checked','certificate_manifest_checked','profile_activated'):require(r[k] is False,'explicit capture scope: '+k)
    return r


def main():
    parser=argparse.ArgumentParser(description=__doc__);group=parser.add_mutually_exclusive_group(required=True);group.add_argument('--write',action='store_true');group.add_argument('--check',action='store_true');args=parser.parse_args()
    data=bases();rows=specs();qs=queries(data,rows);bs=baseline_queries(data);exe,audit=build()
    env={**os.environ,'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOTELEMETRY':'off','CGO_ENABLED':'0','GOFLAGS':'','GOEXPERIMENT':'','GOCACHE':'/private/tmp/mpk-go-practical-checker-cache'}
    outputs={};runs={}
    for name,q in (('retained',qs),('baseline',[x['query'] for x in bs])):
        observed=[timed_input([exe],json_bytes({'registry':active_registry(),'queries':q}),env) for _ in range(2)]
        require(observed[0][0]==observed[1][0],'identical public source validator observations')
        outputs[name]=observed[0][0];runs[name]=[x for _,x in observed]
    outcomes=evaluate(data,rows,policy_from_outputs(qs,strict_json(outputs['retained'])));p=preservation(data,qs,strict_json(outputs['retained']),bs,strict_json(outputs['baseline']))
    docs=dict(zip(DOCS,(catalogue(),data,rows,outcomes,qs,bs)))
    if args.write:
        RECORD.mkdir(exist_ok=True)
        for name,value in docs.items():(RECORD/name).write_bytes(json_bytes(value))
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/'host-toolchain.json').read_bytes()))
    result=subprocess.run(['cargo','build','--release','--offline','--locked','--manifest-path',str(HERE/'body-hashes/Cargo.toml')],cwd=ROOT,env={**env,'CARGO_TARGET_DIR':str(HASH_TARGET)},capture_output=True,timeout=300)
    require(result.returncode==0 and b'warning:' not in result.stderr,'unchanged full-limit Rust hash observer build')
    binaries={'retained':sha(exe.read_bytes())};payload=json_bytes([{k:x[k] for k in ('id','domain','raw_json_hex')} for x in hashes(data)])
    with tempfile.TemporaryDirectory(prefix='mpk-capture-hashes-',dir='/private/tmp') as temp:
        go=Path(temp)/'hashes';rust=HASH_TARGET/'release/go-practical-body-hashes'
        result=subprocess.run([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(go),str(HERE/'body_hashes.go')],cwd=ROOT,env=env,capture_output=True,timeout=300)
        require(result.returncode==0 and not result.stderr,'unchanged full-limit Go hash observer build')
        for name,bin_ in (('rust',rust),('go',go)):
            binaries[name]=sha(bin_.read_bytes());observed=[timed_input([bin_],payload,env) for _ in range(2)]
            require(observed[0][0]==observed[1][0] and exact(strict_json(observed[0][0]),expected_hashes(data)),'capture Python/Rust/Go canonical/hash agreement')
            outputs[name]=observed[0][0];runs[name]=[x for _,x in observed]
        require(before==inputs() and binaries=={'retained':sha(exe.read_bytes()),'rust':sha(rust.read_bytes()),'go':sha(go.read_bytes())},'capture source/binary stability')
    report={'schema':'mpk.go_practical.capture_artifact_probe.v0','scope':SCOPE,'inputs':before,'host_toolchain_inventory':inventory,'native_binaries':binaries,'native_runs':runs,'public_source_audit':audit,
        'hash_vectors':len(hashes(data)),'outcomes':dict(Counter(x['classification'] for x in outcomes)),'native_queries':len(qs),'preservation':p,'direct_child_parallelism':1,
        'shared_family_frozen':False,'general_source_import':False,'ast_correspondence_proofs_checked':False,'ordinary_semantic_proofs_checked':False,'registered_provenance_checked':False,'certificate_manifest_checked':False,'profile_activated':False}
    if args.write:
        for name,raw in outputs.items():(RECORD/({'retained':'retained-native','baseline':'baseline-native'}.get(name,name+'-hashes')+'.json')).write_bytes(raw)
        (RECORD/'observations.json').write_bytes(json_bytes(report))
    old=check_retained(data);require(old['native_binaries']==binaries and old['public_source_audit']==audit,'reproduced capture public native binaries/sources')
    print(f'PASS:{len(data)} artifact triples/{len(rows)} cases,{len(qs)} actual public native queries,{len(hashes(data))} Python/Rust/Go hashes; exact retained map/manifest equality in five target contexts; Go source/provenance/ordinary proofs remain due',flush=True)


if __name__=='__main__':main()
