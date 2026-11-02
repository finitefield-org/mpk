"""Policy corruption controls and real retained API state-transition trials."""
from proof_canonical import canonical,clone,digest
from policy_sources import source_rows,native_query,api_requests
from policy_roots import Policy,KEYS,HASH_FIELDS
from identity_catalog import SHARED
from check_scope_report import require

MUTATIONS=('same','extra-root','old-schema','reserved-csharp-schema','wrong-context','wrong-selection','wrong-policy-contract','request-link','success-link',
    'vir-link','map-link','manifest-link','proved-flag','registered-flag','ready-flag','source-access-flag','hash')


def specs():
    rows=[]
    for r in source_rows():
        id_=r['base']['id'];expected='ordinary_elaboration_pending' if 'project_spec' in r['base'] else 'checked_ordinary_vc' if id_ in {'rust64','rust32'} else 'assembly_error'
        for stage in ('scan','evidence','reproduction','receipt'):
            for m in MUTATIONS:rows.append({'id':id_+'-'+stage+'-'+m,'base':id_,'stage':stage,'mutation':m,'expected':expected if m=='same' else 'configuration_rejection'})
        for stage in ('scan','evidence','reproduction','receipt'):
            for m in ('lf','spaces','duplicate','invalid-utf8','two-values','bom'):rows.append({'id':id_+'-'+stage+'-'+m,'base':id_,'stage':stage,'mutation':m,'expected':'configuration_rejection'})
        for stage,names in (('scan',('claim-ready','helper-proof','capture-sha')),('evidence',('proof-swap','evidence-contract','vc-link','cert-link','property','external-proof','skip-go-member')),
            ('reproduction',('omit-step','wrong-evidence','caller-runner')),('receipt',('claim-checked','wrong-evidence','wrong-reproduction','ai-authority'))):
            for m in names:
                if m=='skip-go-member' and 'project_spec' not in r['base']:continue
                rows.append({'id':id_+'-'+stage+'-'+m,'base':id_,'stage':stage,'mutation':m,'expected':'configuration_rejection'})
    return rows


def reseal(stage,v):
    if stage!='scan':v[HASH_FIELDS[stage]]=digest(SHARED[KEYS[stage]][1],{k:x for k,x in v.items() if k!=HASH_FIELDS[stage]})
    return v


def altered(case,docs):
    stage,m=case['stage'],case['mutation'];v=clone(docs[stage]);raw=None
    if m=='same':return canonical(v)
    if m=='extra-root':v['extension']={}
    elif m in {'old-schema','reserved-csharp-schema'}:v['schema']='mpk.policy.'+stage+('.v2' if m=='old-schema' else '.v3')
    elif m=='wrong-context':v['semantic_context']['source_language']='foreign'
    elif m=='wrong-selection':v['selection']['value']['caller']=True
    elif m=='wrong-policy-contract':v['policy_contract']['value']['axiom_profile']='allow-external'
    elif m=='request-link':v['frontend_request_ref']['request_hash']='0'*64
    elif m=='success-link':v['frontend_success_ref']['success_hash']='0'*64
    elif m in {'vir-link','map-link','manifest-link'}:
        key,field={'vir-link':('vir','vir_hash'),'map-link':('source_map','source_map_hash'),'manifest-link':('frontend_manifest','source_manifest_hash')}[m];v['source_refs'][key][field]='0'*64
    elif m in {'proved-flag','registered-flag','ready-flag','source-access-flag'}:v[{'proved-flag':'proved','registered-flag':'registered_execution','ready-flag':'ready','source-access-flag':'source_access'}[m]]=True
    elif m=='hash':
        if stage=='scan':v['scan_hash']='0'*64
        else:v[HASH_FIELDS[stage]]='0'*64
        return canonical(v)
    elif m=='claim-ready':v['preparation']['readiness']='proof_ready'
    elif m=='helper-proof':v['preparation']['kernel_verdict']='accepted'
    elif m=='capture-sha':v['source_inputs']['caller_snapshot_sha256']='0'*64
    elif m=='proof-swap':v['proof']={'kind':'checked_retained_ordinary_vc','certificate':{'axiom_count':0}}
    elif m=='evidence-contract':v['evidence_contract']['value']['require_reference_checker']=False
    elif m=='vc-link':v['source_vc_ref']['vc_hash']='0'*64
    elif m=='cert-link':v['certificate_manifest_ref']['certificate_manifest_hash']='0'*64
    elif m=='property':v['properties'].append({'id':'caller','status':'verified'})
    elif m=='external-proof':v['proof']['caller_proof']=True
    elif m=='skip-go-member':v['proof']['unelaborated_request_ids'].pop()
    elif m=='omit-step':v['steps'].pop()
    elif m=='wrong-evidence':v['evidence_ref']['evidence_hash']='0'*64
    elif m=='caller-runner':v['runner_requirement']['kind']='caller_command'
    elif m=='claim-checked':v['status']='checked_ordinary_vc' if v['status']!='checked_ordinary_vc' else 'source_correctness_proved'
    elif m=='wrong-reproduction':v['reproduction_ref']['reproduction_hash']='0'*64
    elif m=='ai-authority':v['proof_authority']='ai_or_ci_status'
    elif m not in {'lf','spaces','duplicate','invalid-utf8','two-values','bom'}:raise AssertionError(m)
    reseal(stage,v);raw=canonical(v)
    if m=='lf':raw+=b'\n'
    elif m=='spaces':raw=b' '+raw
    elif m=='duplicate':raw=b'{"schema":"duplicate",'+raw[1:]
    elif m=='invalid-utf8':raw=b'"\xff"'
    elif m=='two-values':raw+=b'{}'
    elif m=='bom':raw=b'\xef\xbb\xbf'+raw
    return raw


def evaluate(native,rows=None):
    models={r['base']['id']:Policy(r,native) for r in source_rows()};docs={id_:p.build() for id_,p in models.items()};out=[]
    for c in specs() if rows is None else rows:
        try:result=models[c['base']].import_json(c['stage'],altered(c,docs[c['base']]));classification=result['kind']
        except (ValueError,TypeError,KeyError):result=None;classification='configuration_rejection'
        require(classification==c['expected'],'policy conformance: '+c['id']+': '+str(classification));out.append({'id':c['id'],'classification':classification,'result':result})
    return docs,out


CONTEXT_FIELDS=('api_profile','session_id','semantic_context','selection','source_ir_schema','source_ir_hash','source_manifest_schema','source_manifest_hash','input_set_hash','source_vc_schema','vc_hash')


def api_trials(row,generated):
    base=api_requests(row);ctx={k:clone(generated[k]) for k in CONTEXT_FIELDS};ctx['session_id']='@session'
    target=generated['vc']['functions'][0]['members'][0]['id'];requests=clone(base)
    requests += [{'operation':'vc/list','value':clone(ctx)},
        {'operation':'vc/start-proof','value':{**clone(ctx),'target':{'kind':'member','id':target}}},
        {'operation':'vc/attach-candidate','value':{**clone(ctx),'target_id':'@target','candidate_id':'missing','proof_root':4294967295}},
        {'operation':'vc/check-candidate','value':{**clone(ctx),'target_id':'@target','mode':'fail_fast_per_candidate','candidates':[{'candidate_id':'missing','proof_root':4294967295}]}}]
    for kind in ('context','selection','vc-hash','unknown-field','noncanonical','missing-lf','unknown-session','reimport','generate-again','unknown-target'):
        op={'operation':'vc/list','value':clone(ctx)}
        if kind=='context':op['value']['semantic_context']['semantic_profile']='caller.unregistered'
        elif kind=='selection':op['value']['selection']['value']['caller']=True
        elif kind=='vc-hash':op['value']['vc_hash']='0'*64
        elif kind=='unknown-field':op['value']['proved']=True
        elif kind in {'noncanonical','missing-lf'}:
            bound=clone(ctx);bound['session_id']=generated['session_id'];op['raw_hex']=(b' '+canonical(bound)+b'\n' if kind=='noncanonical' else canonical(bound)).hex()
        elif kind=='unknown-session':op['value']['session_id']='unknown'
        elif kind=='reimport':op=clone(base[1])
        elif kind=='generate-again':op=clone(base[2])
        else:op={'operation':'vc/start-proof','value':{**clone(ctx),'target':{'kind':'member','id':'missing'}}}
        op['id']=kind;requests.append(op)
    return native_query(row,'api',requests=requests)
