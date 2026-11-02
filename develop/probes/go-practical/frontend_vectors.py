"""Source-bound frontend requests and process conformance cases, not executions."""
from functools import lru_cache
from pathlib import Path
from proof_canonical import canonical,clone,digest,parse
from frontend_wire import Frontend,ProtocolFailure,GO_CODES,request,response,retained_snapshot,go_snapshot
from capture_artifacts import project_nodes
from capture_artifact_vectors import altered as alter_source
from context_protocol import raw_request
from identity_catalog import SHARED
from run_proof_pilot import ROOT
from check_scope_report import strict_json,require

SOURCE=ROOT/'develop/migrations/go-practical/t01-w04/proof-artifacts/artifacts.json'


@lru_cache(None)
def source_bases():
    # Existing immutable source/model bundles, not a frontend invocation.
    return strict_json(SOURCE.read_bytes())


@lru_cache(None)
def go_authorities(id_):
    from shared_project import Project
    from proof_artifact_vectors import source_validator
    b=next(b for b in source_bases() if b['id']==id_);p=Project(b['project_spec']);nodes=project_nodes(p)
    return p,source_validator(b),{i:n for i,n in nodes.items() if n['schema']=='mpk.go.source_object.v1'},\
        {i:n for i,n in nodes.items() if n['schema']=='mpk.go.declaration.v1' and n.get('kind') in {'function','method'}}


def make(base,native,*,extra_available=False):
    authority=clone(base['authority']);c=base['module']['semantic_context'];s=base['manifest']['manifest']['selection']
    if 'project_spec' in base:
        p,source,objects,declarations=go_authorities(base['id']);snapshot=go_snapshot(p)
        a=Frontend(raw_request(c,s),snapshot,authority,native,go_source=source,go_objects=objects,go_declarations=declarations)
    else:
        if extra_available:authority['captured_inputs'].append({'kind':'source','normalized_path':'unused/staged.txt','raw_hex':b'unused staged bytes\n'.hex()})
        snapshot=retained_snapshot(authority['captured_inputs']);a=Frontend(raw_request(c,s),snapshot,authority,native)
    return a,parse(a.expected)


def artifacts(base):return base['module'],base['source_map'],base['manifest']
def facts(exit_code=0,signaled=False,stderr_observed_bytes=0):return dict(exit_code=exit_code,signaled=signaled,stderr_observed_bytes=stderr_observed_bytes)


def issue(base,status):
    lang=base['module']['semantic_context']['source_language']
    if lang=='csharp':return {'code':{'source-error':'CSHARP_SOURCE_PARSE','rejected':'CSHARP_SUBSET_OPERATION','frontend-error':'CSHARP_FRONTEND_INTERNAL'}[status],
        'message':{'source-error':'C# source is invalid','rejected':'C# source is outside the frozen profile','frontend-error':'C# frontend failed closed'}[status]}
    if lang=='java':return {'code':{'source-error':'JAVA_SOURCE_PARSE','rejected':'JAVA_SUBSET_OPERATION','frontend-error':'JAVA_FRONTEND_INTERNAL'}[status],
        'message':{'source-error':'Java source is invalid','rejected':'Java source is outside the frozen profile','frontend-error':'Java frontend failed closed'}[status]}
    return {'code':'NATIVE_SOURCE_TEST','message':'Private process protocol observation'}


def specs():
    rows=[]
    def add(b,kind,expected='configuration_rejection',**kw):rows.append({'id':b+'-'+kind+(('-'+kw['code']+'-'+kw['phase']) if 'code' in kw else ''),'base':b,'mutation':kind,'expected':expected,**kw})
    for b in source_bases():
        id_=b['id'];go='project_spec' in b;success='go_lowered_model_unregistered' if go else 'retained_process_protocol_valid'
        add(id_,'same',success)
        for k in ('request-schema','request-protocol','request-profile','request-context','request-selection','request-release','request-snapshot','request-extra','bad-request-hash',
                  'old-schema','future-schema','wrong-protocol','wrong-profile','wrong-context','wrong-selection','wrong-request','extra-root','ready-flag','proof-flag','registered-flag','bad-hash',
                  'wrapper-schema','wrapper-hash','wrapper-extra','missing-map','missing-manifest','nonempty-success','wrong-phase','wrong-exit','map-link','manifest-input','manifest-release',
                  'empty','missing-lf','extra-lf','noncanonical','duplicate','invalid-utf8','bom','two-values','killed','no-exit','usage','stderr-limit'):
            add(id_,k,{'empty':'FRONTEND_PROTOCOL_MISSING','missing-lf':'FRONTEND_PROTOCOL_TRUNCATED','killed':'FRONTEND_PROCESS_KILLED','no-exit':'FRONTEND_PROCESS_KILLED','usage':'FRONTEND_PROTOCOL_UNEXPECTED_USAGE','stderr-limit':'FRONTEND_PROTOCOL_LIMIT'}.get(k,'configuration_rejection'))
        if not go:
            add(id_,'unused-staged',success)
            for status in ('frontend-error','rejected','source-error'):add(id_,'diagnostic-'+status,'retained_diagnostic_protocol_valid')
            for k in ('diag-artifact','diag-bad-status','diag-bad-exit','diag-empty','diag-unsorted','diag-host-posix','diag-host-uri','diag-host-drive','diag-host-unc','diag-too-many','diag-span-empty','diag-function-object'):
                add(id_,k,'FRONTEND_PROTOCOL_LIMIT' if k=='diag-too-many' else 'configuration_rejection')
        else:
            for code,c in sorted(GO_CODES.items()):
                for phase in c['phases']:add(id_,'go-code','go_diagnostic_model_unregistered',code=code,phase=phase)
            for k in ('go-eof','go-function-span'):add(id_,k,'go_diagnostic_model_unregistered')
            for k in ('go-unknown-code','go-custom-message','go-issue-extra','go-span-object','go-span-negative','go-span-past-end','go-span-bool','go-span-path','go-span-function','go-function-unknown','go-function-list','go-too-many','go-wrong-code-phase','go-wrong-code-status'):
                add(id_,k,'FRONTEND_PROTOCOL_LIMIT' if k=='go-too-many' else 'configuration_rejection')
    for b in ('csharp','java'):
        for k in ('diag-unknown-code','diag-foreign-code','diag-custom-message','diag-code-phase'):add(b,k)
    add('csharp','diag-generic-code','retained_diagnostic_protocol_valid')
    return rows


def reseal(v,request_=False):
    family,field=('frontend_request','request_hash') if request_ else ('frontend_success','success_hash') if v.get('status')=='ir-lowered' else ('frontend_diagnostic','diagnostic_hash')
    v[field]=digest(SHARED[family][1],{k:x for k,x in v.items() if k!=field});return v


def altered(row,b,a,r):
    k=row['mutation'];r=clone(r);v=response(r,'ir-lowered','emission',artifacts=artifacts(b));f=facts();raw=None
    if k.startswith('request-') or k=='bad-request-hash':
        if k=='request-schema':r['schema']='mpk.frontend.request.v1'
        elif k=='request-protocol':r['protocol']='mpk.frontend.cli.v1'
        elif k=='request-profile':r['profile_contract']='mpk.profile.frontend.java.v2'
        elif k=='request-context':r['semantic_request']['semantic_context']['source_language']='foreign'
        elif k=='request-selection':r['semantic_request']['selection']['value']['caller']=True
        elif k=='request-release':r['release_registry']['registry_sha256']='0'*64
        elif k=='request-snapshot':r['source_snapshot']['objects' if 'project_spec' in b else 'inputs'].pop()
        elif k=='request-extra':r['caller_verified']=True
        else:r['request_hash']='0'*64
        if k!='bad-request-hash':reseal(r,True)
    elif k.startswith('diagnostic-') or k.startswith('diag-'):
        status=k.removeprefix('diagnostic-') if k.startswith('diagnostic-') else 'source-error';i=issue(b,status)
        v=response(r,status,'source' if status=='source-error' else 'subset' if status=='rejected' else 'metadata',**({'rejected':[i]} if status=='rejected' else {'diagnostics':[i]}));f=facts({'frontend-error':1,'rejected':3,'source-error':4}[status])
        if k=='diag-artifact':v['ir']={'schema':'mpk.vir.v3'}
        elif k=='diag-bad-status':v['status']='accepted'
        elif k=='diag-bad-exit':f['exit_code']=0
        elif k=='diag-empty':v['diagnostics']=[]
        elif k=='diag-unsorted':v['diagnostics']=[dict(i,message='Zulu'),dict(i,message='Alpha')]
        elif k.startswith('diag-host-'):v['diagnostics'][0]['message']={'posix':'at /private/source.go','uri':'at file:///private/source.go','drive':'at C:\\private\\source.go','unc':'at \\\\server\\source.go'}[k.removeprefix('diag-host-')]
        elif k=='diag-too-many':v['diagnostics']=[i]*1025
        elif k=='diag-span-empty':v['diagnostics'][0]['span']={'normalized_path':'source.go','start':0,'end':0}
        elif k=='diag-function-object':v['diagnostics'][0]['function_id']={}
        elif k=='diag-unknown-code':v['diagnostics'][0]['code']=('CSHARP_' if b['id']=='csharp' else 'JAVA_')+'UNREGISTERED'
        elif k=='diag-foreign-code':v['diagnostics'][0]['code']='JAVA_SOURCE_PARSE' if b['id']=='csharp' else 'CSHARP_SOURCE_PARSE'
        elif k=='diag-generic-code':v['diagnostics'][0]['code']='GENERIC_SOURCE_TEST'
        elif k=='diag-custom-message':v['diagnostics'][0]['message']='untrusted compiler prose'
        elif k=='diag-code-phase':v['phase']='lowering'
        reseal(v)
    elif k.startswith('go-'):
        code=row.get('code','GO_SOURCE_PARSE');c=GO_CODES[code];status=c['status'];phase=row.get('phase',c['phases'][0]);i={'code':code,'message':c['message']}
        if k not in {'go-code','go-too-many'}:
            sid=next(iter(a.go_objects));n=a.go_objects[sid];i['span']={'source_object_id':sid,'start_byte':n['bytes'],'end_byte':n['bytes']}
        if k in {'go-function-span','go-span-function'}:
            fn=next(iter(a.go_declarations));d=a.go_declarations[fn];i['function_decl_id']=fn;i['span']={'source_object_id':d['source_object_id'],'start_byte':d['start_byte'],'end_byte':d['end_byte']}
        if k=='go-unknown-code':i['code']='GO_UNREGISTERED'
        elif k=='go-custom-message':i['message']='compiler leaked /private/source.go'
        elif k=='go-issue-extra':i['ready']=True
        elif k=='go-span-object':i['span']['source_object_id']='0'*64
        elif k=='go-span-negative':i['span']['start_byte']=-1
        elif k=='go-span-past-end':i['span']['end_byte']+=1
        elif k=='go-span-bool':i['span']['start_byte']=False
        elif k=='go-span-path':i['span']={'normalized_path':'source.go','start':0,'end':1}
        elif k=='go-span-function':i['span']['source_object_id']=next(s for s in a.go_objects if s!=d['source_object_id'])
        elif k=='go-function-unknown':i['function_decl_id']='0'*64
        elif k=='go-function-list':i['function_decl_id']=['bad']
        elif k=='go-wrong-code-phase':phase='emission'
        elif k=='go-wrong-code-status':status='rejected'
        items=[i]*(1025 if k=='go-too-many' else 1)
        v=response(r,status,phase,**({'rejected':items} if status=='rejected' else {'diagnostics':items}));f=facts({'source-error':4,'frontend-error':1,'rejected':3}[status])
    elif k in {'same','unused-staged'}:pass
    elif k=='old-schema':v['schema']='mpk.frontend.cli.v1'
    elif k=='future-schema':v['schema']='mpk.frontend.success.v4'
    elif k=='wrong-protocol':v['protocol']='mpk.frontend.cli.v1'
    elif k=='wrong-profile':v['profile_contract']='mpk.profile.frontend.foreign.v2'
    elif k=='wrong-context':v['semantic_context']['source_language']='foreign'
    elif k=='wrong-selection':v['selection']['value']['caller']=True
    elif k=='wrong-request':v['request_ref']['request_hash']='0'*64
    elif k in {'extra-root','ready-flag','proof-flag','registered-flag'}:v[{'extra-root':'caller','ready-flag':'ready','proof-flag':'proved','registered-flag':'registered_execution'}[k]]=True
    elif k=='bad-hash':v['success_hash']='0'*64
    elif k=='wrapper-schema':v['ir']['schema']='mpk.vir.v1'
    elif k=='wrapper-hash':v['ir']['sha256']='0'*64
    elif k=='wrapper-extra':v['ir']['ready']=True
    elif k=='missing-map':del v['source_map']
    elif k=='missing-manifest':del v['source_manifest']
    elif k=='nonempty-success':v['diagnostics']=[issue(b,'source-error')]
    elif k=='wrong-phase':v['phase']='typecheck'
    elif k=='wrong-exit':f['exit_code']=1
    elif k=='map-link':v['source_map']=alter_source({'stage':'map','mutation':'wrong-vir'},b)
    elif k in {'manifest-input','manifest-release'}:v['source_manifest']=alter_source({'stage':'manifest','mutation':('inventory-sha' if 'project_spec' in b else 'input-sha') if k=='manifest-input' else 'release-ref'},b)
    elif k=='killed':f['signaled']=True
    elif k=='no-exit':f['exit_code']=None
    elif k=='usage':f['exit_code']=2
    elif k=='stderr-limit':f['stderr_observed_bytes']=2097153
    elif k not in {'empty','missing-lf','extra-lf','noncanonical','duplicate','invalid-utf8','bom','two-values'}:raise AssertionError(k)
    if k not in {'bad-hash'}:reseal(v)
    raw=canonical(v)+b'\n'
    if k=='empty':raw=b''
    elif k=='missing-lf':raw=raw[:-1]
    elif k=='extra-lf':raw+=b'\n'
    elif k=='noncanonical':raw=b' '+raw
    elif k=='duplicate':raw=b'{"status":"ir-lowered",'+raw[1:]
    elif k=='invalid-utf8':raw=b'"\xff"\n'
    elif k=='bom':raw=b'\xef\xbb\xbf'+raw
    elif k=='two-values':raw+=b'{}\n'
    return r,raw,f


def classify(a,r,raw,f):
    try:return a.validate_process(r,raw,f),None
    except ProtocolFailure as e:return None,e.code
    except (ValueError,TypeError,KeyError):return None,'configuration_rejection'


def evaluate(native,rows=None):
    data=source_bases();by_id={b['id']:b for b in data};validators={b['id']:make(b,native) for b in data};out=[]
    for row in specs() if rows is None else rows:
        b=by_id[row['base']];a,r=make(b,native,extra_available=True) if row['mutation']=='unused-staged' else validators[b['id']]
        r,raw,f=altered(row,b,a,r);result,error=classify(a,r,raw,f);kind=result['kind'] if result else error
        # Closed shape/hash errors are one configuration class; native transport
        # resource/process failures keep their separate observed codes.
        if row['expected']=='configuration_rejection' and error and error not in {'FRONTEND_PROCESS_KILLED','FRONTEND_PROTOCOL_LIMIT','FRONTEND_PROTOCOL_UNEXPECTED_USAGE'}:kind='configuration_rejection'
        require(kind==row['expected'],'frontend outcome '+row['id']+': '+str(kind))
        out.append({'id':row['id'],'classification':kind,'native_error_code':error if error!='configuration_rejection' else None,'result':result})
    return out
