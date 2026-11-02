"""Closed policy scan/evidence/reproduction/receipt draft,with real proof bytes."""
from functools import lru_cache
from proof_canonical import canonical,parse,clone,digest,seal
from run_proof_pilot import sha
from identity_catalog import SHARED
from operation_signatures import exact,need
from policy_sources import source_rows,native_query,profile_envelope,FRONT
from frontend_vectors import make,facts
from run_frontend_wire import policy_from_outputs as frontend_policy
from proof_artifacts import ProofArtifacts
from proof_artifact_vectors import source_validator,base_query,policy_from_outputs as proof_policy
from run_proof_artifacts import RECORD as PROOF
from check_scope_report import strict_json

COMMON=('schema','semantic_context','selection','policy_contract','frontend_request_ref','frontend_success_ref','source_refs')
SCAN_FIELDS=COMMON+('source_inputs','frontend_status','frontend_phase','preparation')
EVIDENCE_FIELDS=COMMON+('evidence_contract','scan_ref','source_vc_ref','certificate_manifest_ref','proof','properties','helper_artifacts','evidence_hash')
REPRO_FIELDS=COMMON+('scan_ref','evidence_ref','steps','runner_requirement','reproduction_hash')
RECEIPT_FIELDS=COMMON+('scan_ref','evidence_ref','reproduction_ref','status','proof_authority','receipt_hash')
FIELDS={'scan':SCAN_FIELDS,'evidence':EVIDENCE_FIELDS,'reproduction':REPRO_FIELDS,'receipt':RECEIPT_FIELDS}
KEYS={'scan':'policy_scan','evidence':'policy_evidence','reproduction':'policy_reproduction','receipt':'policy_receipt'}
HASH_FIELDS={'evidence':'evidence_hash','reproduction':'reproduction_hash','receipt':'receipt_hash'}


def ref(family,value,field):return {'schema':SHARED[family][0],field:value[field]}
def scan_ref(value):return {'schema':SHARED['policy_scan'][0],'canonical_sha256':sha(canonical(value))}
def sealed(stage,body):
    key=KEYS[stage]
    return {'schema':SHARED[key][0],**body} if stage=='scan' else seal(SHARED[key][0],SHARED[key][1],body,HASH_FIELDS[stage])
def bytes_of(value):
    need(type(value) is list and all(type(x) is str and len(x)<=65536 for x in value),'bounded certificate transport chunks')
    return b''.join(bytes.fromhex(s) for s in value)


def paired_certificate(value,expected_manifest):
    """The kernel accepts many claims; source/statement binding remains separate."""
    need('setup_error' not in value,'actual source-free checker setup')
    raw=bytes_of(value['byte_chunks']);r=value['rust'];g=value['reference']
    need('execution_error' not in g and g['exit_code']==0 and g['stderr_hex']=='','actual reference execution,not timeout or protocol failure')
    report=g['stdout'];need(r['accepted'] is True and report['verdict']=='accepted' and r['axiom_count']==report['axiom_count']==0,'two actual zero-axiom acceptances')
    for key in ('certificate_hash','export_hash','axiom_report_hash'):need(r[key]==report['hashes'][key],'complete paired checker hash agreement')
    need(r['declarations']==report['declaration_count'],'paired complete declaration count')
    need(value['embedded_source_manifest_hex']==canonical(expected_manifest).hex(),'exact full common certificate manifest embedded in checked bytes')
    return {'format':'Certificate v0','module':report['module'],'canonical_sha256':sha(raw),'certificate_hash':r['certificate_hash'],
        'export_hash':r['export_hash'],'axiom_report_hash':r['axiom_report_hash'],'axiom_count':0,'declarations':r['declarations'],
        'source_manifest_canonical_sha256':sha(canonical(expected_manifest))}


@lru_cache(None)
def source_checked(id_):
    row=next(r for r in source_rows() if r['base']['id']==id_);b=row['base']
    fp=frontend_policy(strict_json((FRONT/'native-queries.json').read_bytes()),strict_json((FRONT/'retained-native.json').read_bytes()))
    a,r=make(b,fp);need(canonical(r)==canonical(row['request']),'complete independent frontend request')
    result=a.validate_process(r,canonical(row['success'])+b'\n',facts())
    pp=proof_policy(strict_json((PROOF/'native-queries.json').read_bytes()),strict_json((PROOF/'retained-native.json').read_bytes()))
    p=ProofArtifacts(source_validator(b),pp,base_query(b));p.validate_vc(b['module'],b['source_map'],b['manifest'],b['vc'])
    p.validate_certificate_manifest(b['module'],b['source_map'],b['manifest'],b['vc'],b['certificate_manifest'])
    if 'project_spec' not in b:p.validate_skeleton(b['module'],b['source_map'],b['manifest'],b['vc'],b['skeleton'])
    return result


def envelope(row,field):
    b=row['base'];c=b['module']['semantic_context'];kind=b['module']['program']['kind']
    if kind=='go_practical':
        value={'proof_authority':'certificate_only','recipe_profile_id':'mpk.go.practical.evidence_recipe.v1','require_reference_checker':True,'require_source_free_check':True} if field=='evidence' else {
            'axiom_profile':'zero-axiom','checker_profile':'mvp-strict','strategy_profile':'mpk.go.practical.policy_strategy.v1'}
        value['foundation_descriptor']=clone(c['foundation_descriptor'])
    else:value=profile_envelope(b,field)['value']
    return {'profile_entry_sha256':c['profile_entry_sha256'],'contract_id':'mpk.profile.'+field+'.'+kind+'.v2','value':clone(value)}


class Policy:
    def __init__(self,row,native):
        source_checked(row['base']['id']);self.row=clone(row);self.native=native;self.b=self.row['base'];self.go='project_spec' in self.b
        self.common={'semantic_context':clone(self.b['module']['semantic_context']),'selection':clone(row['request']['semantic_request']['selection']),
            'policy_contract':envelope(row,'policy'),'frontend_request_ref':ref('frontend_request',row['request'],'request_hash'),
            'frontend_success_ref':ref('frontend_success',row['success'],'success_hash'),
            'source_refs':{'vir':ref('vir',self.b['module'],'vir_hash'),'source_map':ref('source_map',self.b['source_map'],'source_map_hash'),
                'frontend_manifest':ref('frontend_manifest',self.b['manifest'],'source_manifest_hash')}}
        self.cached=None
    def helpers(self,old):
        rows=clone(old);contracts={f['id']:f['contracts']['contract_hash'] for u in self.b['module']['program'].get('units',[]) for f in u['functions']}
        for r in rows:
            if r['kind']=='verification_ir':r['schema']=SHARED['vir'][0];r['sha256']=self.b['module']['vir_hash']
            elif r['kind']=='vc':r['schema']=SHARED['vc'][0];r['sha256']=self.b['vc']['vc_hash']
            elif r['kind']=='contract':need(r['function_id'] in contracts,'same exact retained contract owner');r['contract_hash']=contracts[r['function_id']]
            else:need(r['kind']=='source','only actual generated frontend/VC helpers; CI/AI are not proof authority')
        return rows
    def build(self):
        if self.cached is not None:return clone(self.cached)
        b=self.b;v=b['vc'];scan_body={**self.common,'frontend_status':'ir-lowered','frontend_phase':'emission'}
        native_scan=None
        if self.go:
            m=b['manifest']['manifest'];plan=v['obligations'];ids=plan['unelaborated_request_ids']
            scan_body['source_inputs']={'kind':'go_practical','capture_context_id':m['capture_context']['id'],'input_inventory_ref':{'schema':m['input_inventory']['schema'],'content_sha256':m['input_inventory']['content_sha256']}}
            scan_body['preparation']={'kind':'go_model_unregistered','readiness':'ordinary_elaboration_pending','program_ref':{'schema':plan['schema'],'program_hash':plan['program_hash']},'unelaborated_request_ids':clone(ids)}
            proof={'kind':'go_ordinary_unelaborated','program_ref':clone(scan_body['preparation']['program_ref']),'unelaborated_request_ids':clone(ids)};properties=[];helpers=[]
        else:
            native_scan=self.native(native_query(self.row,'scan'));need(native_scan.get('accepted') is True,'actual retained policy scan')
            s=native_scan['scan'];scan_body['source_inputs']={'kind':'retained','input_set_hash':s['input_set_hash']}
            scan_body['preparation']={'kind':'retained_native','readiness':s['readiness'],'limit_profile':s['limit_profile'],
                'release_registry':clone(self.row['request']['release_registry']),'frontend':clone(s['frontend']),'toolchain':clone(s['toolchain']),
                'helper_artifacts':self.helpers(s['helper_artifacts']),'rejected_features':clone(s['rejected_features']),'diagnostics':clone(s['diagnostics'])}
            result=self.native(native_query(self.row,'verify'));properties=[];helpers=self.helpers(s['helper_artifacts'])
            if result.get('accepted') is not True:
                need('setup_error' not in result and result['error_code']=='SUCCESSOR_POLICY_CERTIFICATE_ASSEMBLY' and result['phase']=='certificate_assembly','actual retained assembly-phase failure,not unobserved query or source/ordinary falsehood')
                proof={'kind':'retained_assembly_error','phase':result['phase'],'code':result['error_code']}
            else:
                need(result['vc']['functions']==v['functions'] and result['skeleton']['theorem_declarations']==b['skeleton']['theorem_declarations'],'whole actual ordinary VC and statement interface equality')
                outcome=result['outcome'];e=result['evidence'];helpers=self.helpers(e['helper_artifacts']);properties=clone(e['properties'])
                if outcome['kind']=='checked':
                    checked=self.native({'mode':'certificate','byte_chunks':outcome['byte_chunks'],'source_manifest':b['certificate_manifest']})
                    certificate=paired_certificate(checked,b['certificate_manifest'])
                    need(certificate['export_hash']==outcome['rust']['export_hash'] and certificate['axiom_report_hash']==outcome['rust']['axiom_report_hash'],'source rebind leaves all ordinary interfaces/proofs/axiom reports unchanged')
                    declarations=e['trusted_evidence']['certificates'];need(len(declarations)==1 and declarations[0]['export_hash']==certificate['export_hash'],'one fully checked program certificate')
                    proof={'kind':'checked_retained_ordinary_vc','certificate':certificate,'checked_declarations':clone(declarations[0]['checked_declarations']),
                        'checker_verdicts':[{'checker':name,'checker_profile':'mvp-strict','verdict':'accepted','certificate_hash':certificate['certificate_hash']} for name in ('rust','reference')],
                        'source_binding':'complete common certificate manifest plus exact retained VC/statement projection; no general compiler/source-correctness proof'}
                elif outcome['kind']=='pending':proof={'kind':'retained_members_pending','missing_member_ids':clone(outcome['missing_member_ids']),'generated_declarations':outcome['declarations']}
                else:
                    need(outcome['kind']=='unaccepted','closed actual program outcome');proof={'kind':'retained_candidate_unaccepted','rust_verdict':outcome['rust_verdict'],'reference_verdict':outcome['reference_verdict']}
        scan=sealed('scan',scan_body)
        evidence=sealed('evidence',{**self.common,'evidence_contract':envelope(self.row,'evidence'),'scan_ref':scan_ref(scan),
            'source_vc_ref':ref('vc',v,'vc_hash'),'certificate_manifest_ref':ref('certificate_manifest',b['certificate_manifest'],'certificate_manifest_hash'),
            'proof':proof,'properties':properties,'helper_artifacts':helpers})
        steps=[{'operation':'validate_captured_frontend','frontend_request_ref':clone(self.common['frontend_request_ref']),'frontend_success_ref':clone(self.common['frontend_success_ref'])},
            {'operation':'regenerate_policy_scan','scan_ref':scan_ref(scan)},
            {'operation':'materialize_ordinary_vc','source_vc_ref':ref('vc',v,'vc_hash'),'stage':'unelaborated' if self.go else 'retained_projection'}]
        if proof['kind']=='checked_retained_ordinary_vc':steps.append({'operation':'check_identical_certificate_bytes','certificate':clone(proof['certificate']),'required_checkers':['rust','reference'],'required_axiom_count':0})
        repro=sealed('reproduction',{**self.common,'scan_ref':scan_ref(scan),'evidence_ref':ref('policy_evidence',evidence,'evidence_hash'),'steps':steps,
            'runner_requirement':{'kind':'registered_runner_required_for_application_replay','release_registry':clone(self.row['request']['release_registry']),
                'execution_observed_here':'source-bound private policy/API probe; no registered frontend execution or release activation'}})
        status={'go_ordinary_unelaborated':'ordinary_elaboration_pending','checked_retained_ordinary_vc':'checked_ordinary_vc','retained_members_pending':'proof_pending',
            'retained_assembly_error':'assembly_error','retained_candidate_unaccepted':'candidate_unaccepted'}[proof['kind']]
        receipt=sealed('receipt',{**self.common,'scan_ref':scan_ref(scan),'evidence_ref':ref('policy_evidence',evidence,'evidence_hash'),
            'reproduction_ref':ref('policy_reproduction',repro,'reproduction_hash'),'status':status,'proof_authority':'checked_certificate_for_exact_materialized_VC_only'})
        self.cached={'scan':scan,'evidence':evidence,'reproduction':repro,'receipt':receipt};return clone(self.cached)
    def validate(self,stage,value):
        exact(value,FIELDS[stage]);expected=self.build()[stage]
        need(canonical(value)==canonical(expected),'complete policy regeneration binds source,context,selection,helper roles,ordinary/proof state and own hash')
        return {'kind':expected.get('status',self.build()['receipt']['status']),'registered_frontend_execution':False,'source_semantics_proved':False,'profile_activated':False}
