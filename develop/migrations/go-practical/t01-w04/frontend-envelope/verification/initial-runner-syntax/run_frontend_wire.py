#!/usr/bin/env python3
"""Replay closed common frontend roots against actual public retained APIs."""
import argparse
import os
import subprocess
import tempfile
from collections import Counter
from pathlib import Path
from frontend_wire import REQUEST_FIELDS,COMMON,SUCCESS_FIELDS,DIAGNOSTIC_FIELDS,MAX_STDOUT,MAX_STDERR,GO_CODES,transport_guard,ProtocolFailure,request,response
from frontend_vectors import source_bases,make,artifacts,facts,specs,evaluate,altered,canonical,clone,parse,digest
from frontend_native_build import build,linked_inputs
from run_word_arithmetic import inputs as prior_inputs,RECORD as PRIOR
from run_proof_pilot import ROOT,HERE,DEFAULT_GOROOT,sha,json_bytes,verify_host_toolchain
from context_protocol import active_registry
from identity_catalog import SHARED
from check_scope_report import require,strict_json
from run_structural_limits import timed_input

RECORD=PRIOR.parent/'frontend-envelope'
NEW=('frontend_wire.py','frontend_vectors.py','frontend_native_build.py','run_frontend_wire.py','test_frontend_wire.py','check_frontend_metadata.py')
DOCS=('schema.json','requests.json','successes.json','vectors.json','outcomes.json','native-queries.json','limits.json','preservation.json')
SCOPE='private common request/success/diagnostic/CLI draft; actual unchanged retained process/source validators; source-bound unregistered Go model diagnostics and lowering fixtures; no frontend execution,source proof or profile activation'
PENDING=['policy/API/release roots and complete reviewed producer/consumer migrations; W04 freeze and W05 inactive publication/cumulative T01 gate',
    'remaining word/source/partial/definedness/elaborator,state/permission/frame/control/totality proofs and complete Go statement/program/skeleton assembly;169 complete Go requests remain unelaborated',
    'full T02–T11 implementation,application proofs and local Linux release gates before activation']


def catalogue():
    return {'schema':'mpk.go_practical.frontend_envelope_draft.v0','scope':SCOPE,
        'roots':[{'schema':SHARED[k][0],'domain':SHARED[k][1],'fields':list(f),'hash_field':h} for k,f,h in
            (('frontend_request',REQUEST_FIELDS,'request_hash'),('frontend_success',SUCCESS_FIELDS,'success_hash'),('frontend_diagnostic',DIAGNOSTIC_FIELDS,'diagnostic_hash'))]+
            [{'schema':SHARED['frontend_cli'][0],'domain':None,'meaning':'exact protocol tag in every request and response; dispatch to the closed success or diagnostic root; never an additional nested artifact wrapper'}],
        'hash_rule':'SHA256(domain ASCII,NUL,UTF8 narrow JCS of every root field including schema except own hash); IR sha256 is the complete validated common VIR hash',
        'request_binding':'semantic request validates complete private registered context and selection; request independently binds available immutable source snapshot and release authority before lowering; response copies exact request_ref/context/selection',
        'snapshots':{'retained':{'fields':['kind','inputs','input_set_hash'],'input_fields':['kind','normalized_path','size_bytes','sha256'],'hash_domain':'MPK-INPUT-SET-0.1','closure':'success manifest selects exact subset of staged available bytes; unused staging remains in request'},
            'go_practical':{'fields':['kind','objects'],'object_schema':'mpk.go.source_object.v1','closure':'exact capability/path/kind/bytes/raw-hash source objects; success read set subset of available objects and whole independent ProjectSpec validation; no returned capture ID is required to form request'}},
        'success':'ir-lowered/emission/exit0,empty rejected_features and diagnostics; exact complete ir wrapper/source-map/frontend-manifest; artifacts and independent captured-byte authority validated before protocol acceptance',
        'diagnostic':'no IR/map/manifest fields; frontend-error exit1,rejected exit3,source-error exit4; exit2 usage and killed/no exit are process/protocol failures,not source or proof verdicts',
        'transport':{'stdout_bytes':MAX_STDOUT,'stderr_observed_bytes':MAX_STDERR,'canonical_depth':768,'decoded_string_bytes':1048576,'issues':1024,'message_total_bytes':2097152,
            'encoding':'one canonical UTF8 narrow JSON object plus exactly one LF; duplicate keys,unsafe metadata numbers,BOM,invalid UTF8,extra data and unknown fields reject'},
        'retained_preservation':'project into unchanged mpk.frontend.cli.v1 and call public validate_successor_frontend_process; old native256-depth/node/string/source/diagnostic limits and compiled codebooks remain authoritative; common wrapper uses768 and imposes no extra256-depth precheck',
        'retained_codebook_boundary':'C# prefixed closed codes and its existing generic-code behavior are preserved; Java independently requires its closed codebook; no old normative C# namespace is widened',
        'go_issue':{'fields':['code','message'],'optional_fields':['span','function_decl_id'],'span_fields':['source_object_id','start_byte','end_byte'],
            'bounds':'exact captured source object; integer0<=start<=end<=bytes including EOF; optional function declaration resolves independently and agrees with span source object',
            'order':['source_object_id or empty','start_byte or -1','code','message','function_decl_id or empty','end_byte or -1'],
            'messages':'fixed public codebook messages; no raw compiler paths,prose,stdout or stderr; invalid UTF8 source can still have byte spans; model diagnostic does not assert compiler observation'},
        'go_codebook':GO_CODES,'evidence_boundary':{'registered_execution':False,'source_semantics_proved':False,'frontend_invoked':False,'profile_activated':False},
        'owners':{'common_protocol':'crates/mpk-cli/src/successor_frontend_protocol.rs (migration review due)',
            'go_protocol':'crates/mpk-cli/src/go_practical_frontend_protocol.rs (planned)','go_source':'crates/mpk-vc/src/go_practical_artifacts.rs (planned)',
            'runner':'registered runner/release resolver in T07; request hashes and literal release references are not run authentication'},'remaining':PENDING}


def policy_from_outputs(qs,outputs):
    require(len(qs)==len(outputs) and len({canonical(q) for q in qs})==len(qs),'unique complete frontend native observations')
    cache={canonical(q):v for q,v in zip(qs,outputs)}
    def native(q):
        key=canonical(q);require(key in cache,'unobserved frontend query cannot grant acceptance or assumed rejection')
        return clone(cache[key])
    return native


def limit_queries(native):
    b=source_bases()[0];a,_=make(b,native);q=a.native_base();q.update(mode='process',stdout_hex='',**facts())
    out=[{'id':'compiled-limits','query':{'mode':'limits'},'expected':{'stdout_bytes':MAX_STDOUT,'stderr_bytes':MAX_STDERR}}]
    for n in (MAX_STDOUT,MAX_STDOUT+1):out.append({'id':'stdout-'+str(n),'query':{**q,'padding_bytes':n},'expected':{'accepted':False,'error_code':'FRONTEND_PROTOCOL_TRUNCATED' if n==MAX_STDOUT else 'FRONTEND_PROTOCOL_LIMIT'}})
    for n in (MAX_STDERR,MAX_STDERR+1):out.append({'id':'stderr-'+str(n),'query':{**q,'stderr_observed_bytes':n},'expected':{'accepted':False,'error_code':'FRONTEND_PROTOCOL_MISSING' if n==MAX_STDERR else 'FRONTEND_PROTOCOL_LIMIT'}})
    for id_,f,code in (('killed-empty',facts(signaled=True),'FRONTEND_PROCESS_KILLED'),('missing-exit',facts(None),'FRONTEND_PROCESS_KILLED'),('usage-empty',facts(2),'FRONTEND_PROTOCOL_UNEXPECTED_USAGE'),
        ('limit-before-killed',facts(None,True,MAX_STDERR+1),'FRONTEND_PROTOCOL_LIMIT')):
        out.append({'id':id_,'query':{**q,**f},'expected':{'accepted':False,'error_code':code}})
    return out


def preservation(native):
    old=strict_json((PRIOR.parent/'capture-artifacts/baseline-native.json').read_bytes());baseline=strict_json((PRIOR.parent/'capture-artifacts/baseline-queries.json').read_bytes())
    by_id={b['id']:b for b in source_bases()};rows=[]
    for key,wanted in zip(baseline,old):
        b=by_id[key['id']];a,_=make(b,native);v,_=a.retained_project(*artifacts(b))
        require(wanted['accepted'] is True and v['source_map']==wanted['source_map'] and v['source_manifest']==wanted['manifest'],'complete independent retained source baseline: '+b['id'])
        require(v['ir']['value']['vir_hash']==wanted['manifest']['vir_hash'],'same complete retained VIR content hash')
        rows.append({'id':b['id'],'vir_sha256':v['ir']['sha256'],'source_map_sha256':sha(canonical(v['source_map'])),'manifest_sha256':sha(canonical(v['source_manifest']))})
    require(len(rows)==5,'five independently retained context/target baselines');return rows


def documents(native,qs):
    requests={};successes={}
    for b in source_bases():
        _,r=make(b,native);requests[b['id']]=r;successes[b['id']]=response(r,'ir-lowered','emission',artifacts=artifacts(b))
    limits=limit_queries(native)
    for l in limits:require(native(l['query'])==l['expected'],'actual compiled process boundary: '+l['id'])
    return dict(zip(DOCS,(catalogue(),requests,successes,specs(),evaluate(native),qs,limits,preservation(native))))


def hashes(native):
    rows=[]
    def add(id_,domain,v,omit):
        v={k:x for k,x in v.items() if k!=omit};raw=canonical(v);rows.append(dict(id=id_,domain=domain,raw_json_hex=raw.hex(),accepted=True,canonical_hex=raw.hex(),digest=digest(domain,v)))
    for b in source_bases():
        a,r=make(b,native);add(b['id']+'-request',SHARED['frontend_request'][1],r,'request_hash');add(b['id']+'-success',SHARED['frontend_success'][1],response(r,'ir-lowered','emission',artifacts=artifacts(b)),'success_hash')
        for row in specs():
            if row['base']!=b['id'] or row['expected'] not in {'go_diagnostic_model_unregistered','retained_diagnostic_protocol_valid'}:continue
            _,raw,_=altered(row,b,a,r);add(row['id'],SHARED['frontend_diagnostic'][1],parse(raw[:-1]),'diagnostic_hash')
    from run_proof_artifacts import boundaries
    for b in boundaries()+[{'id':'bad-domain','raw_json_hex':'30','accepted':False}]:
        domain='invalid' if b['id']=='bad-domain' else SHARED['frontend_request'][1];row={**b,'domain':domain}
        try:v=parse(bytes.fromhex(row['raw_json_hex']));raw=canonical(v);row.update(canonical_hex=raw.hex(),digest=digest(domain,v));ok=True
        except ValueError:ok=False
        require(ok==row['accepted'],'frontend canonical/domain boundary');rows.append(row)
    return rows


def inputs():
    rows={r['path']:r['sha256'] for r in prior_inputs()}
    for r in linked_inputs():rows[r['path']]=r['sha256']
    paths=[PRIOR/'observations.json']+[HERE/n for n in NEW]+[p for p in (HERE/'frontend-native').rglob('*') if p.is_file()]+[RECORD/n for n in DOCS]
    for p in paths:rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    for p,h in rows.items():require(sha((ROOT/p).read_bytes())==h,'frontend and unchanged predecessor source pin: '+p)
    return [{'path':p,'sha256':h} for p,h in sorted(rows.items())]


def check_retained():
    r=strict_json((RECORD/'observations.json').read_bytes());qs=strict_json((RECORD/'native-queries.json').read_bytes());native=policy_from_outputs(qs,strict_json((RECORD/'retained-native.json').read_bytes()));docs=documents(native,qs)
    for n,v in docs.items():require((RECORD/n).read_bytes()==canonical(v)+b'\n','complete frontend reconstruction: '+n)
    require(r['scope']==SCOPE and r['pending']==PENDING and r['inputs']==inputs() and r['outcomes']==dict(Counter(x['classification'] for x in docs['outcomes.json'])),'frontend source/scope/case closure')
    hs=hashes(native);expected=[{k:v for k,v in h.items() if k not in {'domain','raw_json_hex'}} for h in hs]
    require(r['native_queries']==len(qs) and r['hash_vectors']==len(hs),'complete actual query/hash coverage')
    for name in ('retained','rust','go'):
        raw=(RECORD/('retained-native.json' if name=='retained' else name+'-hashes.json')).read_bytes()
        require(len(r['native_runs'][name])==2 and all(x['exit_code']==0 and x['stdout_sha256']==sha(raw) for x in r['native_runs'][name]),'two exact frontend native observations')
        if name!='retained':require(strict_json(raw)==expected,'full Python/Rust/Go frontend preimages')
    for f in (RECORD/'verification').glob('initial-*/diagnostic.json'):
        for x in strict_json(f.read_bytes())['inputs']:require(sha((f.parent/x['path']).read_bytes())==x['sha256'],'preserved initial frontend finding')
    for f in ('frontend_invoked','registered_execution','source_semantics_proved','shared_family_frozen','profile_activated'):require(r[f] is False,'frontend protocol/model evidence boundary')
    return r


def main():
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group(required=True);g.add_argument('--write',action='store_true');g.add_argument('--check',action='store_true');args=p.parse_args()
    exe,audit=build();cache={};qs=[];observations=[]
    def collect(q):
        key=canonical(q)
        if key not in cache:
            raw,_=timed_input([exe],canonical({'registry':active_registry(),'queries':[q]}),os.environ.copy());v=strict_json(raw)[0]
            cache[key]=v;qs.append(q);observations.append(v)
        return clone(cache[key])
    # Discover only queries actually required by closed validation. Absence is
    # never interpreted as a negative. Re-execute the entire set twice below.
    evaluate(collect);preservation(collect)
    for l in limit_queries(collect):require(collect(l['query'])==l['expected'],'actual process boundary '+l['id'])
    docs=documents(collect,qs);print('constructed',len(docs['vectors.json']),'cases /',len(qs),'native queries',flush=True)
    if args.write:
        RECORD.mkdir(exist_ok=True)
        for n,v in docs.items():(RECORD/n).write_bytes(canonical(v)+b'\n')
    before=inputs();inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/'host-toolchain.json').read_bytes()));outputs={};runs={};binaries={'retained':sha(exe.read_bytes())}
    for name,queries in (('retained',qs),('rust',[{k:h[k] for k in ('id','domain','raw_json_hex')} for h in hashes(collect))):
        observed=[timed_input([exe],canonical({'registry':active_registry(),'queries':queries}),os.environ.copy()) for _ in range(2)]
        require(observed[0][0]==observed[1][0],'two byte-identical public frontend observations');outputs[name]=observed[0][0];runs[name]=[x for _,x in observed]
    require(strict_json(outputs['retained'])==observations,'fresh complete native replay agrees with query discovery')
    expected=[{k:v for k,v in h.items() if k not in {'domain','raw_json_hex'}} for h in hashes(collect)];require(strict_json(outputs['rust'])==expected,'Rust/frontend hash agreement')
    with tempfile.TemporaryDirectory(prefix='mpk-frontend-hashes-',dir='/private/tmp') as temp:
        go=Path(temp)/'hashes';env={**os.environ,'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOTELEMETRY':'off','CGO_ENABLED':'0','GOFLAGS':'','GOEXPERIMENT':'','GOCACHE':'/private/tmp/mpk-go-practical-checker-cache'}
        p=subprocess.run([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(go),str(HERE/'proof_hashes.go')],cwd=ROOT,env=env,capture_output=True,timeout=300);require(p.returncode==0 and not p.stderr,'pinned Go frontend hash observer build')
        binaries['go']=sha(go.read_bytes());observed=[timed_input([go],canonical([{k:h[k] for k in ('id','domain','raw_json_hex')} for h in hashes(collect)]),env) for _ in range(2)]
        require(observed[0][0]==observed[1][0] and strict_json(observed[0][0])==expected,'independent Go/full frontend hash agreement');outputs['go']=observed[0][0];runs['go']=[x for _,x in observed]
        require(before==inputs() and binaries=={'retained':sha(exe.read_bytes()),'go':sha(go.read_bytes())},'frontend source/binary stability')
    report={'schema':'mpk.go_practical.frontend_envelope_probe.v0','scope':SCOPE,'pending':PENDING,'inputs':before,'host_toolchain_inventory':inventory,'public_source_audit':audit,'native_binaries':binaries,'native_runs':runs,
        'cases':len(docs['vectors.json']),'native_queries':len(qs),'hash_vectors':len(expected),'outcomes':dict(Counter(x['classification'] for x in docs['outcomes.json'])),'retained_context_baselines':5,'retained_success_bundles':6,'go_model_bundles':2,'go_codebook_entries':len(GO_CODES),
        'frontend_invoked':False,'registered_execution':False,'source_semantics_proved':False,'shared_family_frozen':False,'profile_activated':False,'direct_child_parallelism':1}
    if args.write:
        for name,raw in outputs.items():(RECORD/('retained-native.json' if name=='retained' else name+'-hashes.json')).write_bytes(raw)
        (RECORD/'observations.json').write_bytes(json_bytes(report))
    retained=check_retained();require(retained['native_binaries']==binaries and retained['public_source_audit']==audit,'fresh public frontend build/source reproduction')
    print('PASS:',report['cases'],'frontend cases;',len(qs),'actual native queries;',len(expected),'Python/Rust/Go hashes; W04 open',flush=True)


if __name__=='__main__':main()
