"""Reconstructible common proof artifacts and actual retained regeneration queries."""
import copy
import json
from pathlib import Path
from proof_canonical import canonical,digest,parse
from capture_artifacts import Artifacts
from capture_artifact_vectors import bases as source_bases
from run_capture_artifacts import module_policy,RECORD as SOURCE_RECORD,baseline_queries as source_baselines
from run_shared_vir import policy_from_outputs as source_policy
from proof_artifacts import ProofArtifacts,vc,retained_payload,certificate_manifest,skeleton
from shared_project import Project
from go_obligations import program as go_program
from identity_catalog import SHARED
from run_proof_pilot import ROOT


def profile_envelope(context):
    language=context['source_language'];profile=context['semantic_profile']
    if language in ('csharp','java'):
        data=json.loads((ROOT/('develop/specs/vectors/'+language+'-profile-v0.json')).read_bytes())
        return next(r['envelope'] for r in data['profile_contracts'] if r['field']=='vc')
    suffix='go_fixed' if language=='go' else 'rust_checked'
    return {'profile_entry_sha256':context['profile_entry_sha256'],'contract_id':'mpk.profile.vc.'+suffix+'.v0',
        'value':{'contract_profile_id':'mpk.'+language+'.contract.v0','required_check_profile_id':profile,'verification_limit_profile_id':'mpk.verify.limits.v0'}}


def captured_policy():
    return source_policy(json.loads((SOURCE_RECORD/'native-queries.json').read_bytes()),json.loads((SOURCE_RECORD/'retained-native.json').read_bytes()))


def source_validator(base):return Artifacts(module_policy(),captured_policy(),base['authority'],base.get('project_spec'))


def base_query(base):
    if 'project_spec' in base:return None
    q=source_validator(base).query(base['module'],base['source_map'],base['manifest']);q['profile_contract']=profile_envelope(q['context']);return q


def generator_queries(data=None):
    data=source_bases() if data is None else data
    return [{'id':b['id'],'query':base_query(b)} for b in data if 'project_spec' not in b]


def policy_from_outputs(qs,outputs):
    if len(qs)!=len(outputs):raise AssertionError('complete native proof observations')
    cache={canonical(q):r for q,r in zip(qs,outputs)}
    def policy(q):
        key=canonical(q)
        if key not in cache:raise AssertionError('unobserved proof regeneration query cannot become assumed rejection/acceptance')
        return cache[key]
    return policy


def bases(generated,data=None):
    data=source_bases() if data is None else data;out=[]
    for b in data:
        row=copy.deepcopy(b);context=b['module']['semantic_context']
        if 'project_spec' in b:
            plan=go_program(b['module'],b['source_map'],b['manifest'],Project(b['project_spec']))
            payload={'kind':'go_practical','obligations':plan,'ordinary_expansion':{'kind':'unavailable'}}
            row['skeleton']=None
        else:
            native=generated[b['id']];assert native['accepted'] is True;payload=retained_payload(context,native)
        v=vc(b['module'],b['source_map'],b['manifest'],payload);row['vc']=v;row['certificate_manifest']=certificate_manifest(b['manifest'],v)
        if 'project_spec' not in b:row['skeleton']=skeleton(v,{'kind':v['kind'],'theorem_declarations':copy.deepcopy(generated[b['id']]['skeleton']['theorem_declarations'])})
        out.append(row)
    return out


def baseline_queries(data):
    out=[]
    for b in source_baselines(data):
        q=copy.deepcopy(b['query']);context=q['original']['vir']['semantic_context'] if 'original' in q else q['context']
        q['profile_contract']=profile_envelope(context);out.append({'id':b['id'],'query':q})
    return out


def specs():
    rows=[]
    for base in ('go_fixed','rust64','rust32','csharp','java','go-two-overlays','go-default-contracts','go_fixed_synthetic'):
        for stage in ('vc','certificate','skeleton'):
            if stage=='skeleton' and base.startswith('go-'):
                rows.append({'id':base+'-skeleton-pending','base':base,'stage':'skeleton_request','mutation':'same','expected':'ordinary_elaboration_pending'});continue
            for kind in ('same','extra-root','old-schema','wrong-context','wrong-profile-contract','wrong-source','bad-hash'):
                rows.append({'id':base+'-'+stage+'-'+kind,'base':base,'stage':stage,'mutation':kind,
                    'expected':('go_obligations_unelaborated' if stage=='vc' and base.startswith('go-') else 'retained_vc_generated' if stage=='vc' else 'certificate_manifest_linked_unproved' if stage=='certificate' else 'retained_skeleton_generated') if kind=='same' else 'configuration_rejection'})
    for base in ('go_fixed','rust64','csharp','java'):
        for kind in ('missing-member','altered-conclusion','altered-group','nested-extra'):
            rows.append({'id':base+'-vc-'+kind,'base':base,'stage':'vc','mutation':kind,'expected':'configuration_rejection'})
        rows.append({'id':base+'-skeleton-altered-statement','base':base,'stage':'skeleton','mutation':'altered-statement','expected':'configuration_rejection'})
    for kind in ('missing-request','caller-expansion','stale-request','missing-dependency','wrong-clause'):
        rows.append({'id':'go-'+kind,'base':'go-two-overlays','stage':'vc','mutation':kind,'expected':'configuration_rejection'})
    return rows


def reseal(value,stage):
    if stage=='skeleton':return value
    field='vc_hash' if stage=='vc' else 'certificate_manifest_hash';domain=SHARED['vc' if stage=='vc' else 'certificate_manifest'][1]
    value[field]=digest(domain,{k:v for k,v in value.items() if k!=field});return value


def altered(row,base):
    stage=row['stage'];kind=row['mutation'];value=copy.deepcopy(base['vc' if stage=='vc' else 'certificate_manifest' if stage=='certificate' else 'skeleton'])
    if kind=='same':return value
    if kind=='extra-root':value['verified']=True
    elif kind=='old-schema':value['schema']='mpk.vc.v2' if stage=='vc' else 'mpk.source_manifest.v1' if stage=='certificate' else 'mpk.vc.cert_skeleton.v2'
    elif kind=='wrong-context':value['semantic_context']['source_language']='unknown'
    elif kind=='wrong-profile-contract':value['profile_contract']='mpk.profile.vir.go_fixed.v2'
    elif kind=='wrong-source':value['source_ir_hash' if stage=='vc' else 'vc_hash' if stage=='certificate' else 'source_vc_hash']='0'*64
    elif kind=='bad-hash':
        if stage=='skeleton':value['source_vc_hash']='0'*64
        else:value['vc_hash' if stage=='vc' else 'certificate_manifest_hash']='0'*64
        return value
    elif kind in {'missing-member','altered-conclusion','altered-group','nested-extra'}:
        f=next(f for f in value['functions'] if f['members']);m=f['members'][0]
        if kind=='missing-member':f['members'].pop(0)
        elif kind=='altered-conclusion':m['conclusion']={'kind':'constant','name':'Std.Bool.false'}
        elif kind=='altered-group':f['groups'][0]['member_ids']=[]
        else:m['conclusion']['caller_verified']=True
    elif kind=='altered-statement':value['theorem_declarations'][0]['theorem_type']['body']={'kind':'constant','name':'Std.Bool.false'}
    elif kind=='caller-expansion':value['ordinary_expansion']={'kind':'expanded','ordinary_context_ref':{'schema':SHARED['program_assembly'][0],'assembly_sha256':'0'*64},'request_bindings':[]}
    else:
        plan=value['obligations'];requests=plan['functions'][-1]['requests']
        if kind=='missing-request':requests.pop()
        elif kind=='stale-request':requests[0]['body_hash']='0'*64
        elif kind=='missing-dependency':next(r for f in plan['functions'] for r in f['requests'] if r['dependencies'])['dependencies']=[]
        elif kind=='wrong-clause':next(r for f in plan['functions'] for r in f['requests'] if r['requirement']['kind']=='postcondition')['requirement']['expression']={'op':'bool.literal','type':{'kind':'logical','name':'Bool'},'payload':{'value':False}}
        else:raise AssertionError(kind)
        from go_obligations import REQUEST_DOMAIN,DOMAIN
        for f in plan['functions']:
            for r in f['requests']:r['request_hash']=digest(REQUEST_DOMAIN,{k:v for k,v in r.items() if k!='request_hash'})
        plan['program_hash']=digest(DOMAIN,{k:v for k,v in plan.items() if k!='program_hash'})
    return reseal(value,stage)


def native_queries(data,rows):
    by_id={b['id']:b for b in data};base_qs={b['id']:base_query(b) for b in data if 'project_spec' not in b}
    out={canonical(q):q for q in base_qs.values()}
    for row in rows:
        b=by_id[row['base']]
        if 'project_spec' in b:continue
        q=copy.deepcopy(base_qs[b['id']]);value=altered(row,b)
        if row['stage']=='vc' and 'functions' in value:q['vc_payload']={k:value[k] for k in ('functions','verification_limit_profile')}
        elif row['stage']=='skeleton' and 'theorem_declarations' in value:q['theorem_declarations']=value['theorem_declarations']
        else:continue
        out[canonical(q)]=q
    for original in base_qs.values():
        q=copy.deepcopy(original);q['profile_contract']['value']['required_check_profile_id']='caller.unregistered'
        out[canonical(q)]=q
    return [out[k] for k in sorted(out)]
