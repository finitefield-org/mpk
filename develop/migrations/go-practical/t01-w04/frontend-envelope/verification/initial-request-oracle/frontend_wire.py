"""Closed frontend request/process draft; protocol validity is not run provenance."""
from proof_canonical import canonical,digest,seal,clone,parse
from identity_catalog import SHARED
from shared_vir import KINDS,VIR_FIELDS,CONTRACT_FIELDS,RETAINED_CONTRACT_FIELDS
from capture_artifacts import MAP_FIELDS,MAN_FIELDS,RETAINED_MAN_FIELDS,input_inventory
from capture_artifact_vectors import inventory
from context_protocol import legacy_projection
from foundation_limits_vectors import model as protocol_model
from operation_signatures import exact,need

MAX_STDOUT=268435456
MAX_STDERR=2097152
REQUEST_FIELDS=('schema','protocol','profile_contract','semantic_request','source_snapshot','release_registry','request_hash')
COMMON=('schema','protocol','profile_contract','request_ref','semantic_context','selection','status','phase','rejected_features','diagnostics')
SUCCESS_FIELDS=COMMON+('ir','source_map','source_manifest','success_hash')
DIAGNOSTIC_FIELDS=COMMON+('diagnostic_hash',)
PHASES=('capture','source','metadata','typecheck','subset','lowering','emission')


def contract(context):return 'mpk.profile.frontend.'+KINDS[context['semantic_profile']]+'.v2'
def retained_snapshot(captured):
    rows=inventory(captured);need(len({(r['kind'],r['normalized_path']) for r in rows})==len(rows),'unique independent available inputs')
    return {'kind':'retained','inputs':rows,'input_set_hash':digest('MPK-INPUT-SET-0.1',rows)}
def go_snapshot(project):return {'kind':'go_practical','objects':input_inventory(project)['objects']}
def request(semantic_request,snapshot,release):
    return seal(SHARED['frontend_request'][0],SHARED['frontend_request'][1],{'protocol':SHARED['frontend_cli'][0],
        'profile_contract':contract(semantic_request['semantic_context']),'semantic_request':clone(semantic_request),
        'source_snapshot':clone(snapshot),'release_registry':clone(release)},'request_hash')
def request_ref(r):return {'schema':r['schema'],'request_hash':r['request_hash']}
def response(r,status,phase,*,artifacts=None,rejected=(),diagnostics=()):
    success=status=='ir-lowered';key='frontend_success' if success else 'frontend_diagnostic';field='success_hash' if success else 'diagnostic_hash';s=r['semantic_request']
    value={'protocol':SHARED['frontend_cli'][0],'profile_contract':r['profile_contract'],'request_ref':request_ref(r),
        'semantic_context':clone(s['semantic_context']),'selection':clone(s['selection']),'status':status,'phase':phase,
        'rejected_features':clone(list(rejected)),'diagnostics':clone(list(diagnostics))}
    if success:
        need(artifacts is not None,'success requires complete source artifacts');vir,map_,manifest=artifacts
        value.update(ir={'schema':SHARED['vir'][0],'sha256':vir['vir_hash'],'value':clone(vir)},source_map=clone(map_),source_manifest=clone(manifest))
    else:need(artifacts is None,'diagnostic has no artifacts')
    return seal(SHARED[key][0],SHARED[key][1],value,field)


class ProtocolFailure(ValueError):
    def __init__(self,code):self.code=code;super().__init__(code)
def transport_guard(length,facts):
    exact(facts,('exit_code','signaled','stderr_observed_bytes'))
    need((facts['exit_code'] is None or type(facts['exit_code']) is int and -(1<<31)<=facts['exit_code']<(1<<31))
        and type(facts['signaled']) is bool and type(facts['stderr_observed_bytes']) is int and facts['stderr_observed_bytes']>=0,'trusted process fact types')
    if length>MAX_STDOUT or facts['stderr_observed_bytes']>MAX_STDERR:raise ProtocolFailure('FRONTEND_PROTOCOL_LIMIT')
    if facts['signaled'] or facts['exit_code'] is None:raise ProtocolFailure('FRONTEND_PROCESS_KILLED')
    if facts['exit_code']==2:raise ProtocolFailure('FRONTEND_PROTOCOL_UNEXPECTED_USAGE')


# Private new-Go codebook. Existing profiles use their actual compiled codebooks.
GO_CODES={}
def codes(names,status,phases,message):
    for name in names.split():GO_CODES['GO_'+name]={'status':status,'phases':list(phases),'message':message}
codes('CAPTURE SNAPSHOT_BINDING','frontend-error',('capture',),'Go capture failed closed')
codes('TOOLCHAIN MODULE_METADATA','frontend-error',('metadata',),'Go toolchain or module metadata is unavailable')
codes('SELECTION','frontend-error',('subset',),'Go function selection is invalid')
codes('RESOURCE INTERNAL OUTPUT_LIMIT DIAGNOSTIC_BUDGET','frontend-error',PHASES,'Go frontend failed closed')
codes('SOURCE_ENCODING SOURCE_PARSE','source-error',('source',),'Go source is invalid')
codes('SOURCE_MODULE','source-error',('metadata',),'Go module source is invalid')
codes('SOURCE_TYPE','source-error',('typecheck',),'Go source is invalid')
codes('SUBSET_TYPE','rejected',('typecheck',),'Go source is outside the profile')
codes('SUBSET_DECLARATION SUBSET_OPERATION SUBSET_CALL SUBSET_INITIALIZATION SUBSET_EFFECT SUBSET_CONTROL SUBSET_STORAGE CONTRACT_PARSE CONTRACT_SHAPE CONTRACT_TYPE CONTRACT_LINK','rejected',('subset',),'Go source or contract is outside the profile')
codes('LIMIT_CAPTURE','rejected',('capture',),'Go profile limit exceeded')
codes('LIMIT_SOURCE','rejected',('source',),'Go profile limit exceeded')
codes('LIMIT_CLOSURE','rejected',('subset',),'Go profile limit exceeded')
codes('LIMIT_OPERATIONS','rejected',('lowering',),'Go profile limit exceeded')
codes('LIMIT_ARTIFACT','rejected',('emission',),'Go profile limit exceeded')
codes('LOWERING_BINDING LOWERING_CHECKS','frontend-error',('lowering',),'Go lowering failed closed')
codes('SOURCE_MAP','frontend-error',('emission',),'Go source-map emission failed closed')


def legacy_units(vir,context):
    """Preserve native inner limits after projection,without an extra256-depth cap.

    Header/hash checks use the existing768-depth canonicalizer. Actual old
    typing,terms,diagnostics and source limits remain owned by the public API.
    """
    exact(vir['program'],('kind','units'));units=clone(vir['program']['units']);hashes={}
    need(type(units) is list,'retained unit array')
    for unit in units:
        exact(unit,('id','name','type_decls','const_decls','functions'));need(type(unit['functions']) is list,'retained function array')
        for f in unit['functions']:
            exact(f,('id','unit_id','name','params','results','locals','blocks','contracts','features_used'));c=f['contracts'];exact(c,CONTRACT_FIELDS)
            need(c['schema']==SHARED['contract'][0] and canonical(c['semantic_context'])==canonical(context)
                and c['profile_contract']=='mpk.profile.vir.'+KINDS[context['semantic_profile']]+'.v2','exact retained contract context')
            need(c['contract_hash']==digest(SHARED['contract'][1],{k:v for k,v in c.items() if k!='contract_hash'}),'complete retained contract hash')
            p=c['contract'];exact(p,RETAINED_CONTRACT_FIELDS)
            need(p['kind']==vir['program']['kind'] and p['unit_id']==unit['id']==f['unit_id'] and p['function_id']==f['id'],'same retained function/contract')
            need(type(f['id']) is str and f['id'] not in hashes,'unique retained function');hashes[f['id']]=c['contract_hash']
    for u in units:
        for f in u['functions']:
            need(type(f['blocks']) is list,'retained blocks')
            for b in f['blocks']:
                need(type(b) is dict and type(b.get('instructions')) is list,'retained block shape')
                for op in b['instructions']:
                    if type(op) is dict and op.get('kind')=='CallStatic':need(op.get('function') in hashes and op.get('contract_hash')==hashes[op['function']],'same-family call contract before projection')
            f['contracts']={k:v for k,v in f['contracts']['contract'].items() if k!='kind'}
    return units


class Frontend:
    def __init__(self,semantic_request,snapshot,authority,native,*,go_source=None,go_objects=None,go_declarations=None):
        protocol_model().contexts.validate_request(semantic_request)
        self.expected=canonical(request(semantic_request,snapshot,authority['release_registry']));self.authority=canonical(authority)
        self.context=clone(semantic_request['semantic_context']);self.selection=clone(semantic_request['selection']);self.native=native
        self.go_source=go_source;self.go_objects=clone(go_objects or {});self.go_declarations=clone(go_declarations or {})
        self.kind=KINDS[self.context['semantic_profile']]
    def validate_request(self,value):
        exact(value,REQUEST_FIELDS);need(canonical(value)==self.expected,'complete request reconstructed from independent semantic request,snapshot and release authority')
        return value
    def native_base(self,manifest=None):
        a=parse(self.authority);captured=a['captured_inputs']
        if manifest is not None:
            wanted={(r['kind'],r['normalized_path']):r for r in manifest['inputs']};available=retained_snapshot(captured)['inputs'];by_key={(r['kind'],r['normalized_path']):r for r in available}
            need(len(wanted)==len(manifest['inputs']) and all(k in by_key and by_key[k]==v for k,v in wanted.items()),'selected closure uses exact immutable available bytes')
            captured=[r for r in captured if (r['kind'],r['normalized_path']) in wanted]
        return {'context':legacy_projection(self.context)['context'],'selection':clone(self.selection),
            'expected_release_registry':a['expected_release_registry'],'captured_inputs':captured,'synthetic_permissions':a['synthetic_permissions']}
    def retained_project(self,vir,map_,manifest):
        self.header(vir,VIR_FIELDS,'vir','vir_hash','vir');need(vir['program']['kind']==self.kind,'context-dispatched retained program')
        self.header(map_,MAP_FIELDS,'source_map','source_map_hash','source_map');exact(map_['map'],('kind','entries'))
        need(map_['map']['kind']==self.kind and map_['source_ir_schema']==vir['schema'] and map_['source_ir_hash']==vir['vir_hash'],'same VIR/map')
        self.header(manifest,MAN_FIELDS,'frontend_manifest','source_manifest_hash','manifest');p=manifest['manifest'];exact(p,RETAINED_MAN_FIELDS)
        expected=parse(self.expected)
        need(p['kind']==self.kind and canonical(p['selection'])==canonical(self.selection) and canonical(p['release_registry'])==canonical(expected['release_registry']),'same request selection and independent release reference')
        need(manifest['vir_hash']==vir['vir_hash'] and manifest['source_map_hash']==map_['source_map_hash'],'same source artifacts')
        q=self.native_base(p);old={k:clone(v) for k,v in p.items() if k!='kind'};old['release_registry']=q['expected_release_registry']
        q.update(mode='project',units=legacy_units(vir,self.context),entries=clone(map_['map']['entries']),manifest=old)
        answer=self.native(q)
        if answer.get('accepted') is not True:raise ProtocolFailure(answer.get('error_code','FRONTEND_ARTIFACT_PROJECTION'))
        return clone(answer['value']),q
    def header(self,value,fields,family,field,role):
        exact(value,fields)
        need(value['schema']==SHARED[family][0] and canonical(value['semantic_context'])==canonical(self.context)
            and value['profile_contract']=='mpk.profile.'+role+'.'+self.kind+'.v2','full common artifact context/profile contract')
        need(value[field]==digest(SHARED[family][1],{k:v for k,v in value.items() if k!=field}),'complete common artifact hash')
    def go_issues(self,value):
        rejected,diagnostics=value['rejected_features'],value['diagnostics'];status,phase=value['status'],value['phase']
        need(type(rejected) is list and type(diagnostics) is list,'issue arrays')
        if len(rejected)+len(diagnostics)>1024:raise ProtocolFailure('FRONTEND_PROTOCOL_LIMIT')
        need((status=='ir-lowered' and phase=='emission' and not rejected and not diagnostics)
            or (status=='rejected' and rejected and not diagnostics) or (status in {'source-error','frontend-error'} and diagnostics and not rejected),'status/issue arm')
        total=0
        for rows in (rejected,diagnostics):
            order=[]
            for issue in rows:
                need(type(issue) is dict,'closed issue object')
                fields={'code','message'}|({'span'} if 'span' in issue else set())|({'function_decl_id'} if 'function_decl_id' in issue else set());exact(issue,fields)
                need(type(issue['code']) is str and issue['code'] in GO_CODES,'closed new-Go codebook');c=GO_CODES[issue['code']]
                need(status==c['status'] and phase in c['phases'] and issue['message']==c['message'],'fixed Go diagnostic status/phase/message')
                total+=len(issue['message'].encode());sid='';start=end=-1
                if 'span' in issue:
                    s=issue['span'];exact(s,('source_object_id','start_byte','end_byte'));sid=s['source_object_id'];start,end=s['start_byte'],s['end_byte']
                    need(type(sid) is str and sid in self.go_objects and type(start) is int and type(end) is int and 0<=start<=end<=self.go_objects[sid]['bytes'],'byte span including EOF over exact captured object; no ambiguous path')
                function=issue.get('function_decl_id','')
                if function:need(type(function) is str and function in self.go_declarations and (not sid or self.go_declarations[function]['source_object_id']==sid),'same captured Go declaration/span')
                elif 'function_decl_id' in issue:raise ValueError('nonempty captured function identity')
                order.append((sid,start,issue['code'],issue['message'],function,end))
            need(order==sorted(order),'canonical Go issue order')
        if total>2097152:raise ProtocolFailure('FRONTEND_PROTOCOL_LIMIT')
    def validate_process(self,r,stdout,facts):
        self.validate_request(r);transport_guard(len(stdout),facts)
        if not stdout:raise ProtocolFailure('FRONTEND_PROTOCOL_MISSING')
        if not stdout.endswith(b'\n'):raise ProtocolFailure('FRONTEND_PROTOCOL_TRUNCATED')
        raw=stdout[:-1];value=parse(raw);need(type(value) is dict and canonical(value)==raw,'one canonical UTF8 JSON object and exactly one LF')
        success=value.get('status')=='ir-lowered';fields=SUCCESS_FIELDS if success else DIAGNOSTIC_FIELDS;exact(value,fields)
        family,field=('frontend_success','success_hash') if success else ('frontend_diagnostic','diagnostic_hash')
        need(value['schema']==SHARED[family][0] and value['protocol']==SHARED['frontend_cli'][0] and value['profile_contract']==r['profile_contract'],'exact frontend generation/profile arm')
        need(value['request_ref']==request_ref(r) and canonical(value['semantic_context'])==canonical(self.context) and canonical(value['selection'])==canonical(self.selection),'complete request/context/selection equality')
        need(value[field]==digest(SHARED[family][1],{k:v for k,v in value.items() if k!=field}),'whole response preimage')
        if success:
            ir=value['ir'];exact(ir,('schema','sha256','value'));need(type(ir['value']) is dict and ir['schema']==SHARED['vir'][0] and ir['value'].get('schema')==ir['schema'] and ir['sha256']==ir['value'].get('vir_hash'),'full IR wrapper reference')
        if self.kind=='go_practical':
            self.go_issues(value);need(facts['exit_code']=={'ir-lowered':0,'frontend-error':1,'rejected':3,'source-error':4}[value['status']],'status/exit equality')
            if success:
                need(self.go_source is not None,'independent complete captured source authority');self.go_source.validate_manifest(value['ir']['value'],value['source_map'],value['source_manifest'])
                expected={x['id']:x for x in r['source_snapshot']['objects']}
                need(all(expected.get(x['id'])==x for x in value['source_manifest']['manifest']['input_inventory']['objects']),'Go captured closure is an exact subset of the available snapshot')
            return {'kind':'go_lowered_model_unregistered' if success else 'go_diagnostic_model_unregistered','registered_execution':False,'source_semantics_proved':False}
        if success:old,q=self.retained_project(value['ir']['value'],value['source_map'],value['source_manifest'])
        else:old={'schema':'mpk.frontend.cli.v1'};q=self.native_base()
        old.update({k:clone(value[k]) for k in ('status','phase','rejected_features','diagnostics')});old['semantic_context']=q['context'];old['selection']=q['selection']
        query={k:v for k,v in q.items() if k in ('context','selection','expected_release_registry','captured_inputs','synthetic_permissions')}
        query.update(mode='process',stdout_hex=(canonical(old)+b'\n').hex(),**facts)
        result=self.native(query)
        if result.get('accepted') is not True:raise ProtocolFailure(result.get('error_code','FRONTEND_PROTOCOL_REJECTED'))
        need(result['canonical_hex']==query['stdout_hex'],'actual public native validator preserves complete canonical envelope')
        return {'kind':'retained_process_protocol_valid' if success else 'retained_diagnostic_protocol_valid','registered_execution':False,'source_semantics_proved':False}
