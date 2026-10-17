"""Independent immutable source/capture roots and retained public-policy inputs."""
from functools import lru_cache
from proof_canonical import canonical,parse,clone
from frontend_vectors import source_bases
from frontend_wire import legacy_units
from context_protocol import legacy_projection
from run_proof_pilot import ROOT
from check_scope_report import strict_json,require

FRONT=ROOT/'develop/migrations/go-practical/t01-w04/frontend-envelope'


def profile_envelope(base,field):
    c=base['module']['semantic_context'];language=c['source_language'];kind=base['module']['program']['kind']
    if language in {'csharp','java'}:
        data=strict_json((ROOT/('develop/specs/vectors/'+language+'-profile-v0.json')).read_bytes())
        return clone(next(x['envelope'] for x in data['profile_contracts'] if x['field']==field))
    if field=='vc':value={'contract_profile_id':'mpk.'+language+'.contract.v0','required_check_profile_id':c['semantic_profile'],'verification_limit_profile_id':'mpk.verify.limits.v0'}
    elif field=='policy':value={'axiom_profile':'zero-axiom' if language=='go' else 'mvp-theory','checker_profile':'mvp-strict','strategy_profile':'payment-policy-alpha' if language=='go' else 'payment-policy-rust-alpha'}
    elif field=='evidence':value={'proof_authority':'certificate_only','recipe_profile_id':'mpk.'+language+'.evidence_recipe.v0','require_reference_checker':True,'require_source_free_check':True}
    else:raise AssertionError('closed profile-envelope role')
    return {'profile_entry_sha256':c['profile_entry_sha256'],'contract_id':'mpk.profile.'+field+'.'+kind+'.v0','value':value}


@lru_cache(None)
def source_rows():
    queries=strict_json((FRONT/'native-queries.json').read_bytes());outputs=strict_json((FRONT/'retained-native.json').read_bytes())
    requests=strict_json((FRONT/'requests.json').read_bytes());successes=strict_json((FRONT/'successes.json').read_bytes());out=[]
    for b in source_bases():
        row={'base':b,'request':requests[b['id']],'success':successes[b['id']]}
        if 'project_spec' not in b:
            c=legacy_projection(b['module']['semantic_context'])['context'];units=legacy_units(b['module'],b['module']['semantic_context']);a=b['authority']
            matching=[(q,v) for q,v in zip(queries,outputs) if q.get('mode')=='project' and v.get('accepted') is True and q['context']==c and q['units']==units
                and q['entries']==b['source_map']['map']['entries'] and q['synthetic_permissions']==a['synthetic_permissions']]
            require(len(matching)==1,'one actual source-bound retained frontend: '+b['id']);q,v=matching[0]
            require(q['captured_inputs']==a['captured_inputs'] and q['expected_release_registry']==a['expected_release_registry'],'independent available byte/release authority')
            row['native']={k:clone(q[k]) for k in ('captured_inputs','synthetic_permissions','expected_release_registry')}
            row['native']['frontend']=clone(v['value'])
            for field in ('vc','policy','evidence'):row['native'][field+'_contract']=profile_envelope(b,field)
        out.append(row)
    return out


def native_query(row,mode,**extra):
    require('native' in row,'Go practical has no installed policy/API dispatch')
    return {**clone(row['native']),'mode':mode,**extra}


def api_requests(row):
    f=row['native']['frontend'];c=f['semantic_context'];selection=f['selection'];vir=f['ir']['value'];m=f['source_manifest']
    common={'api_profile':'mpk.ai.api.v2','semantic_context':c,'selection':selection};sid={'session_id':'@session'}
    binding={**common,**sid,'source_ir_schema':vir['schema'],'source_ir_hash':vir['vir_hash'],'source_manifest_schema':m['schema'],
        'source_manifest_hash':m['source_manifest_hash'],'input_set_hash':m['input_set_hash']}
    return [{'operation':'session/start','value':{**common,'module_name':'Probe.Policy.'+row['base']['id'].replace('-','_'),'proof_profile':'mvp-strict'}},
        {'operation':'vir/import','value':{**common,**sid,'source_ir_schema':vir['schema'],'source_ir_hash':vir['vir_hash'],'vir':vir}},
        {'operation':'vc/generate','value':binding}]
