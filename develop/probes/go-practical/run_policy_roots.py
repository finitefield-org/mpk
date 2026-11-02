#!/usr/bin/env python3
"""Replay common policy roots,actual retained APIs and exact-source certificates."""
import argparse
import os
import subprocess
import tempfile
from pathlib import Path
from collections import Counter
from policy_roots import Policy,KEYS,FIELDS,HASH_FIELDS,paired_certificate,bytes_of
from policy_sources import source_rows,native_query,api_requests,FRONT
from policy_cases import specs,evaluate,reseal,api_trials
from policy_native_build import build,linked_inputs,TARGET
from proof_canonical import canonical,parse,clone,digest
from identity_catalog import SHARED
from context_protocol import active_registry
from run_frontend_wire import inputs as prior_inputs
from run_proof_pilot import ROOT,HERE,DEFAULT_GOROOT,sha,json_bytes,verify_host_toolchain
from check_scope_report import require,strict_json
from run_structural_limits import timed_input

RECORD=FRONT.parent/'policy-api'
NEW=('policy_sources.py','policy_roots.py','policy_cases.py','policy_native_build.py','run_policy_roots.py','test_policy_roots.py','check_policy_metadata.py','policy_raw_hashes.go')
DOCS=('schema.json','bundles.json','vectors.json','outcomes.json','native-queries.json','api-trials.json','kernel-witnesses.json','native-imports.json')
SCOPE='private four common policy roots,exact common-manifest binding of real dual-checked retained ordinary certificates,actual seven retained API operations and source/proof readiness boundaries; no Go ordinary elaboration,registered frontend execution or activation'
PENDING=['complete Go ordinary statement/program/assembly/skeleton context and source/elaborator/word/state/permission/frame/control/definedness/totality proofs;169 Go requests remain unelaborated',
    'new APIv4/AI-explanation and release roots,all reviewed producer/consumer migrations and general profile-preservation/activation review; observed retained assembly interface failures remain failures',
    'W04 freeze,W05 inactive publication and cumulative T01 gate; full T02–T11 implementation,application proofs and required native local Linux release gates']


def catalogue():
    return {'schema':'mpk.go_practical.policy_roots_draft.v0','scope':SCOPE,'roots':[{'schema':SHARED[KEYS[s]][0],'domain':SHARED[KEYS[s]][1],'fields':list(FIELDS[s]),'hash_field':HASH_FIELDS.get(s)} for s in KEYS],
        'canonical':'existing narrow JCS depth768,safe integral metadata,1MiB decoded strings,256MiB JSON budget; policy inputs have no LF; scan canonical_sha256 is raw SHA256 of the whole canonical value,not a newly invented public scan domain',
        'hashes':'evidence/reproduction/receipt use the distinct reserved v4 domain,NUL,and every root field except own hash; source refs use exact shared context-selected schema/hash pairs',
        'graph':'captured frontend request/success and complete source artifacts -> scan -> evidence -> reproduction -> receipt; certificate manifest references source VC/frontend graph,never downstream evidence or receipt',
        'authority':'complete immutable independent source authority plus actual native observations; IDs,caller flags,AI/CI/helper documents,checker JSON or a valid unrelated certificate cannot grant source-proof authority',
        'source_validation':'complete preceding frontend/model and VC/certificate-manifest/retained-skeleton independent validation; full actual retained VC functions and statement declarations compare before source rebind',
        'proof_arms':{'checked_retained_ordinary_vc':'actual retained public policy assembly; replace only Certificate v0 opaque source-manifest payload with the entire validated common certificate manifest; run unchanged Rust and embedded Go source-free checkers on identical resulting bytes; exact export/axiom interface hashes remain unchanged',
            'retained_assembly_error':'actual public certificate-assembly phase/code; no certificate/proof verdict or source rejection invented',
            'retained_members_pending':'actual assembler missing-member set; no checked certificate',
            'retained_candidate_unaccepted':'actual candidate checker verdict pair; never a source compiler verdict',
            'go_ordinary_unelaborated':'complete program hash and exact unelaborated request set; no ordinary target bindings,skeleton or certificate claimed'},
        'source_binding_counterexample':'the two kernels accept a real Rust certificate stamped with either Go common certificate manifest; policy still rejects a checked-Go claim because its complete ordinary statement/program context is unelaborated',
        'helpers':'retained source/raw-contract inputs keep meaning; normalized contract,VIR and VC refs lift to exact new-family source values; helpers remain non-authoritative',
        'profile_contracts':'retained compiled values remain exact; new-family v2 contract ID and entry hash; proposed Go strategy/recipe v1 additionally bind the complete existing private foundation descriptor and require zero axioms,both strict source-free checkers',
        'reproduction':'typed exact source/scan/VC/check steps; registered runner and resolved release authority required for application replay; no source frontend,application replay or release activation is observed by this probe',
        'native_API':'all seven existing context-aware operations exercised; generated/listed/imported targets are helper_only; failures must leave mutation_count unchanged; these are existing APIv2 observations,not publication of APIv4 roots or routes',
        'execution':'pinned cached Rust1.95 Linux amd64 container,local arm64 engine/emulation,network none,read-only repository,unprivileged UID; not native Linux release or checker process-tree resource evidence',
        'owners':{'policy':'crates/mpk-cli/src/go_practical_policy.rs (planned)','program':'crates/mpk-cli/src/go_practical_program.rs (planned)',
            'API':'crates/mpk-api/src/go_practical_api.rs (planned)','release':'registered runner/resolver T07; no activation here'},'remaining':PENDING}


def policy_from_outputs(qs,outs):
    require(len(qs)==len(outs) and len({canonical(q) for q in qs})==len(qs),'complete unique native policy observations');cache={canonical(q):o for q,o in zip(qs,outs)}
    def native(q):
        k=canonical(q)
        if k not in cache:raise AssertionError('unobserved policy/API/checker query cannot become assumed acceptance or rejection')
        return clone(cache[k])
    return native


def scan_import_tests(scan):
    rows=[{'id':'same','value':clone(scan),'expected':True}]
    for name in ('extra','readiness','source','context','policy','helper'):
        v=clone(scan)
        if name=='extra':v['proved']=True
        elif name=='readiness':v['readiness']='verified'
        elif name=='source':v['source_ir_hash']='0'*64
        elif name=='context':v['semantic_context']['source_language']='foreign'
        elif name=='policy':v['policy_contract']['value']['axiom_profile']='external'
        else:v['helper_artifacts'].append({'kind':'ci_status','id':'ci','status':'success'})
        rows.append({'id':name,'value':v,'expected':False})
    rows.append({'id':'noncanonical','value':clone(scan),'raw_hex':(canonical(scan)+b'\n').hex(),'expected':False});return rows


def native_imports(native):
    out=[]
    for row in source_rows():
        if 'native' not in row:continue
        s=native(native_query(row,'scan'))['scan']
        for t in scan_import_tests(s):
            q=native_query(row,'scan',candidate=t['value'],**({'candidate_hex':t['raw_hex']} if 'raw_hex' in t else {}));r=native(q)
            require(r.get('accepted') is t['expected'],'actual scan import '+row['base']['id']+':'+t['id']);out.append({'id':row['base']['id']+'-scan-'+t['id'],'query':q,'result':r})
        v=native(native_query(row,'verify'))
        if v.get('accepted') is not True:continue
        tests=[]
        for name in ('same','extra','source','certificate','verdict','property','noncanonical'):
            e=clone(v['evidence'])
            if name=='extra':e['ready']=True
            elif name=='source':e['source_ir_hash']='0'*64
            elif name=='certificate':e['trusted_evidence']['certificates'][0]['certificate_hash']='0'*64
            elif name=='verdict':e['trusted_evidence']['checker_verdicts'][0]['verdict']='caller_accepted'
            elif name=='property':e['properties'][0]['status']='caller_verified'
            t={'id':name,'stage':'evidence','value':e}
            if name=='noncanonical':t['raw_hex']=(canonical(e)+b'\n').hex()
            tests.append(t)
        q=native_query(row,'verify',imports=tests);r=native(q);require(r.get('accepted') is True,'actual checked run for evidence import')
        require([x['accepted'] for x in r['imports']]==[True]+[False]*6,'actual evidence source/proof/transport import controls');out.append({'id':row['base']['id']+'-evidence-imports','query':q,'result':r['imports']})
    return out


def api_observations(native):
    rows=[]
    expected_errors=['AI_API_V2_UNKNOWN_PROOF','AI_API_V2_CANDIDATE_UNKNOWN']+['AI_API_V2_CONTEXT_MISMATCH']*3+['AI_API_V2_SHAPE','AI_API_V2_CANONICAL_TRANSPORT','AI_API_V2_JSON_INVALID','AI_API_V2_SCALAR','AI_API_V2_SESSION_STATE','AI_API_V2_SESSION_STATE','AI_API_V2_TARGET_UNKNOWN']
    for row in source_rows():
        if 'native' not in row:continue
        first=native(native_query(row,'api',requests=api_requests(row)));require(first.get('accepted') is True and all(x['accepted'] for x in first['operations']),'actual API bound source generation')
        q=api_trials(row,first['operations'][2]['value']);r=native(q);require(r.get('accepted') is True and len(r['operations'])==17 and r['mutations']==4,'all seven API operations/state sequence')
        ops=r['operations'];require([x['accepted'] for x in ops]==[True]*5+[False]*12,'actual API positive/negative states')
        require([x['error_code'] for x in ops[5:]]==expected_errors,'exact public API rejection classes')
        require(all(x['mutation_before']==x['mutation_after'] for x in ops[5:]),'all actual failures are atomic')
        require(all(x['value']['helper_only'] is True for x in ops[:5]),'API helper response cannot grant source-proof authority')
        rows.append({'id':row['base']['id'],'query':q,'result':r})
    return rows


def kernel_witnesses(native,docs):
    rust=next(r for r in source_rows() if r['base']['id']=='rust64');v=native(native_query(rust,'verify'));chunks=v['outcome']['byte_chunks'];out=[]
    for row in source_rows():
        if 'project_spec' not in row['base']:continue
        q={'mode':'certificate','byte_chunks':chunks,'source_manifest':row['base']['certificate_manifest']};r=native(q);c=paired_certificate(r,row['base']['certificate_manifest'])
        value=clone(docs[row['base']['id']]['evidence']);value['proof']=clone(docs['rust64']['evidence']['proof']);value['proof']['certificate']=c;reseal('evidence',value)
        try:Policy(row,native).validate('evidence',value)
        except ValueError:pass
        else:raise AssertionError('a metadata-stamped kernel certificate proved unelaborated Go')
        out.append({'id':row['base']['id'],'query':q,'kernel_certificate':c,'classification':'kernel_accepted_go_claim_rejected','forged_evidence':value})
    raw=bytearray(bytes_of(chunks));raw[0]^=1;q={'mode':'certificate','byte_chunks':[bytes(raw[i:i+32768]).hex() for i in range(0,len(raw),32768)]};r=native(q)
    require(r['rust']['accepted'] is False and r['reference'].get('exit_code')==1 and r['reference']['stdout']['verdict']=='rejected','both actual checkers reject corrupted format')
    out.append({'id':'corrupted-magic','query':q,'classification':'format_rejection_not_ordinary_falsehood','result':r});return out


def hashes(docs):
    rows=[]
    for id_,bundle in docs.items():
        for stage,value in bundle.items():
            preimage={k:v for k,v in value.items() if k!=HASH_FIELDS.get(stage)};raw=canonical(preimage);domain=SHARED[KEYS[stage]][1] or 'MPK-GO-IDENTITY-PILOT-1.0'
            rows.append({'id':id_+'-'+stage,'raw_json_hex':raw.hex(),'domain':domain,'accepted':True,'canonical_hex':raw.hex(),'canonical_sha256':sha(raw),'digest':digest(domain,preimage),'public_scan_domain':None if stage=='scan' else domain})
    from run_proof_artifacts import boundaries
    for b in boundaries()+[{'id':'bad-domain','raw_json_hex':'30','accepted':False}]:
        domain='invalid' if b['id']=='bad-domain' else SHARED['policy_evidence'][1];r={**b,'domain':domain}
        try:v=parse(bytes.fromhex(b['raw_json_hex']));raw=canonical(v);r.update(canonical_hex=raw.hex(),canonical_sha256=sha(raw),digest=digest(domain,v));ok=True
        except ValueError:ok=False
        require(ok==b['accepted'],'policy canonical/domain boundary');rows.append(r)
    return rows


def source_inputs():
    rows={x['path']:x['sha256'] for x in prior_inputs()}
    for x in linked_inputs():rows[x['path']]=x['sha256']
    paths=[FRONT/'observations.json']+[HERE/n for n in NEW]+[p for p in (HERE/'policy-native').rglob('*') if p.is_file()]
    for p in paths:rows[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
    return [{'path':p,'sha256':h} for p,h in sorted(rows.items())]
def inputs():return source_inputs()+[{'path':(RECORD/n).relative_to(ROOT).as_posix(),'sha256':sha((RECORD/n).read_bytes())} for n in DOCS]


def documents(native,qs,prepared=None):
    docs,out=evaluate(native) if prepared is None else prepared
    return dict(zip(DOCS,(catalogue(),docs,specs(),out,qs,api_observations(native),kernel_witnesses(native,docs),native_imports(native))))


def check_retained():
    r=strict_json((RECORD/'observations.json').read_bytes());qs=strict_json((RECORD/'native-queries.json').read_bytes());native=policy_from_outputs(qs,strict_json((RECORD/'native.json').read_bytes()));ds=documents(native,qs)
    for n,v in ds.items():require((RECORD/n).read_bytes()==canonical(v)+b'\n','exact policy/source/proof/API reconstruction: '+n)
    require(r['scope']==SCOPE and r['pending']==PENDING and r['inputs']==inputs(),'policy scope/source identity')
    require(r['cases']==len(ds['vectors.json']) and r['outcomes']==dict(Counter(x['classification'] for x in ds['outcomes.json'])),'all policy statuses and controls')
    hs=hashes(ds['bundles.json']);expected=[{k:v for k,v in h.items() if k in {'id','accepted','canonical_hex','canonical_sha256','digest'}} for h in hs]
    require(strict_json((RECORD/'rust-hashes.json').read_bytes())==expected,'full actual Rust policy preimages/raw/domain hashes')
    go=strict_json((RECORD/'go-hashes.json').read_bytes());require(go==[{k:v for k,v in h.items() if k!='canonical_sha256'} for h in expected],'independent Go policy canonical/domain hashes')
    require(strict_json((RECORD/'go-raw-hashes.json').read_bytes())==[{k:h[k] for k in ('id','canonical_sha256')} for h in expected if h['accepted']],'independent Go raw canonical identities')
    for name in ('native','rust-hashes','go-hashes','go-raw-hashes'):
        raw=(RECORD/(name+'.json')).read_bytes();require(len(r['runs'][name])==2 and all(x['exit_code']==0 and x['stdout_sha256']==sha(raw) for x in r['runs'][name]),'two exact complete policy/API/checker/hash observations')
    for p in (RECORD/'verification').glob('initial-*/diagnostic.json'):
        for x in strict_json(p.read_bytes())['inputs']:require(sha((p.parent/x['path']).read_bytes())==x['sha256'],'initial policy probe finding preserved')
    for flag in ('Go_ordinary_elaborated','registered_frontend_executed','source_semantics_proved','profile_activated','W04_frozen'):require(r[flag] is False,'policy evidence/readiness boundary')
    return r


def main():
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group(required=True);g.add_argument('--write',action='store_true');g.add_argument('--check',action='store_true');args=p.parse_args()
    before=source_inputs();cmd,audit=build();cache={};qs=[];outs=[]
    def collect(q):
        key=canonical(q)
        if key not in cache:
            raw,_=timed_input(cmd,canonical({'registry':active_registry(),'queries':[q]}),os.environ.copy());v=strict_json(raw)[0]
            require('setup_error' not in v,'observer setup is not semantic rejection: '+str(v.get('setup_error')));cache[key]=v;qs.append(q);outs.append(v)
        return clone(cache[key])
    prepared=evaluate(collect);ds=documents(collect,qs,prepared);require(before==source_inputs(),'all source bytes fixed throughout native discovery')
    if args.write:
        RECORD.mkdir(exist_ok=True)
        for n,v in ds.items():(RECORD/n).write_bytes(canonical(v)+b'\n')
    pinned=inputs();hs=hashes(ds['bundles.json']);hash_queries=[{'mode':'hash',**{k:h[k] for k in ('id','raw_json_hex','domain')}} for h in hs]
    results={};runs={};binaries={'linux_observer':audit['binary_sha256']};inventory=verify_host_toolchain(DEFAULT_GOROOT,strict_json((HERE/'host-toolchain.json').read_bytes()))
    for name,queries in (('native',qs),('rust-hashes',hash_queries)):
        observed=[timed_input(cmd,canonical({'registry':active_registry(),'queries':queries}),os.environ.copy()) for _ in range(2)];require(observed[0][0]==observed[1][0],'two complete Linux policy/API/kernel runs reproduce exactly');results[name]=observed[0][0];runs[name]=[x for _,x in observed]
    require(strict_json(results['native'])==outs,'full native replay equals all discovered observations')
    with tempfile.TemporaryDirectory(prefix='mpk-policy-hashes-',dir='/private/tmp') as temp:
        env={**os.environ,'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOTELEMETRY':'off','CGO_ENABLED':'0','GOFLAGS':'','GOEXPERIMENT':'','GOCACHE':'/private/tmp/mpk-go-practical-checker-cache'}
        for name,source in (('go-hashes','proof_hashes.go'),('go-raw-hashes','policy_raw_hashes.go')):
            payload=[{k:h[k] for k in ('id','raw_json_hex','domain')} for h in hs] if name=='go-hashes' else [{k:h[k] for k in ('id','canonical_hex')} for h in strict_json(results['go-hashes']) if h['accepted']]
            exe=Path(temp)/name;x=subprocess.run([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(exe),str(HERE/source)],cwd=ROOT,env=env,capture_output=True,timeout=300);require(x.returncode==0 and not x.stderr,'independent Go policy hash observer build')
            observed=[timed_input([exe],canonical(payload),env) for _ in range(2)];require(observed[0][0]==observed[1][0],'two exact Go policy hashes');results[name]=observed[0][0];runs[name]=[x for _,x in observed];binaries[name]=sha(exe.read_bytes())
        require(pinned==inputs() and sha((TARGET/'debug/go-practical-policy-native').read_bytes())==binaries['linux_observer'],'policy source/binary stability')
    report={'schema':'mpk.go_practical.policy_roots_probe.v0','scope':SCOPE,'pending':PENDING,'inputs':pinned,'public_source_audit':audit,'binaries':binaries,'runs':runs,'host_toolchain_inventory':inventory,
        'cases':len(ds['vectors.json']),'native_queries':len(qs),'hash_vectors':len(hs),'outcomes':dict(Counter(x['classification'] for x in ds['outcomes.json'])),'API_operations':102,'API_failures_atomic':72,
        'rebound_zero_axiom_certificates':2,'kernel_valid_go_claims_rejected':2,'retained_assembly_errors':4,'Go_unelaborated_requests':169,
        'Go_ordinary_elaborated':False,'registered_frontend_executed':False,'source_semantics_proved':False,'profile_activated':False,'W04_frozen':False,
        'resource_scope':'Darwin Docker-client proxy rusage/timing only; no native/checker process-tree resource or Linux release evidence; unchanged embedded reference checker retains its own runtime limits'}
    if args.write:
        for n,raw in results.items():(RECORD/(n+'.json')).write_bytes(raw)
        (RECORD/'observations.json').write_bytes(json_bytes(report))
    old=check_retained();require(old['binaries']==binaries and old['public_source_audit']==audit,'fresh actual policy checker/binary/build identity')
    print('PASS:',report['cases'],'policy cases,',len(qs),'actual Linux queries,',len(hs),'hashes,102 API operations/72 atomic failures,2 source-rebound dual-zero-axiom certificates and2 rejected Go proof claims; W04 open',flush=True)


if __name__=='__main__':main()
