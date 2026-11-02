"""Common VC/certificate source references with profile-owned theorem programs."""
from proof_canonical import canonical,digest,seal,clone,parse
from capture_artifacts import Artifacts
from shared_project import Project
from shared_vir import KINDS
from identity_catalog import SHARED
from operation_signatures import need,exact
from go_obligations import program as go_program,require_materialized
from body_canonical import parse as source_parse

VC_COMMON=('schema','semantic_context','profile_contract','source_ir_schema','source_ir_hash','source_map_hash','source_manifest_schema','source_manifest_hash','kind','vc_hash')
RETAINED_VC_FIELDS=VC_COMMON+('verification_limit_profile','functions')
GO_VC_FIELDS=VC_COMMON+('obligations','ordinary_expansion')
CERT_FIELDS=('schema','semantic_context','profile_contract','source_manifest_schema','source_manifest_hash','vc_schema','vc_hash','certificate_manifest_hash')
SKELETON_FIELDS=('schema','semantic_context','profile_contract','source_vc_schema','source_vc_hash','kind','theorem_declarations')


def contract(context,field):return 'mpk.profile.'+field+'.'+KINDS[context['semantic_profile']]+'.v2'


def vc(vir,map_,manifest,payload):
    return seal(SHARED['vc'][0],SHARED['vc'][1],{'semantic_context':vir['semantic_context'],'profile_contract':contract(vir['semantic_context'],'vc'),
        'source_ir_schema':SHARED['vir'][0],'source_ir_hash':vir['vir_hash'],'source_map_hash':map_['source_map_hash'],
        'source_manifest_schema':SHARED['frontend_manifest'][0],'source_manifest_hash':manifest['source_manifest_hash'],**payload},'vc_hash')


def certificate_manifest(manifest,vc_):
    return seal(SHARED['certificate_manifest'][0],SHARED['certificate_manifest'][1],{'semantic_context':manifest['semantic_context'],
        'profile_contract':contract(manifest['semantic_context'],'manifest'),'source_manifest_schema':SHARED['frontend_manifest'][0],
        'source_manifest_hash':manifest['source_manifest_hash'],'vc_schema':SHARED['vc'][0],'vc_hash':vc_['vc_hash']},'certificate_manifest_hash')


def retained_payload(context,generated):
    v=generated['vc'];return {'kind':KINDS[context['semantic_profile']],'verification_limit_profile':v['verification_limit_profile'],'functions':clone(v['functions'])}


def skeleton(vc_,payload):
    return {'schema':SHARED['vc_skeleton'][0],'semantic_context':vc_['semantic_context'],'profile_contract':vc_['profile_contract'],
        'source_vc_schema':SHARED['vc'][0],'source_vc_hash':vc_['vc_hash'],**payload}


class ProofArtifacts:
    def __init__(self,source_validator,native_policy,base_query):
        self.source=source_validator;self.policy=native_policy;self.query_bytes=canonical(base_query)
        self.cached_source=None;self.expected_vc=None;self.native=None

    def generated(self,vir,map_,manifest):
        key=(canonical(vir),canonical(map_),canonical(manifest))
        if key!=self.cached_source:
            self.source.validate_manifest(vir,map_,manifest);context=vir['semantic_context'];kind=vir['program']['kind']
            if kind=='go_practical':
                p=Project(source_parse(self.source.project_bytes));payload={'kind':kind,'obligations':go_program(vir,map_,manifest,p),
                    'ordinary_expansion':{'kind':'unavailable'}}
                self.native=None
            else:
                query=parse(self.query_bytes);expected=self.source.query(vir,map_,manifest)
                expected['profile_contract']=query['profile_contract']
                need(canonical(expected)==self.query_bytes,'native generator source exactly matches validated common artifacts and independent authority')
                self.native=clone(self.policy(query));need(self.native['accepted'] is True,'actual retained public VC generation/import')
                payload=retained_payload(context,self.native)
            self.expected_vc=canonical(vc(vir,map_,manifest,payload));self.cached_source=key
        return self.expected_vc

    def validate_vc(self,vir,map_,manifest,value):
        exact(value,GO_VC_FIELDS if vir['program']['kind']=='go_practical' else RETAINED_VC_FIELDS);need(value['schema']==SHARED['vc'][0],'one common VC generation')
        expected=self.generated(vir,map_,manifest)
        need(canonical(value)==expected,'full common VC context/source/program/hash regeneration')
        return {'kind':'go_obligations_unelaborated' if vir['program']['kind']=='go_practical' else 'retained_vc_generated'}

    def expected_skeleton(self,vc_):
        if vc_['kind']=='go_practical':
            require_materialized(vc_['obligations'])
            raise ValueError('registered Go ordinary context and exact statement bindings are required')
        need(self.native is not None and self.native['accepted'] is True,'validated retained VC source')
        return skeleton(vc_,{'kind':vc_['kind'],'theorem_declarations':clone(self.native['skeleton']['theorem_declarations'])})

    def validate_skeleton(self,vir,map_,manifest,vc_,value):
        exact(value,SKELETON_FIELDS)
        need(value['schema']==SHARED['vc_skeleton'][0] and value['source_vc_schema']==SHARED['vc'][0]
            and value['source_vc_hash']==vc_['vc_hash'] and value['kind']==vc_['kind']
            and value['profile_contract']==vc_['profile_contract'] and canonical(value['semantic_context'])==canonical(vc_['semantic_context']),'exact skeleton context/profile/source reference')
        self.validate_vc(vir,map_,manifest,vc_)
        need(canonical(value)==canonical(self.expected_skeleton(vc_)),'complete source-bound skeleton regeneration')
        return {'kind':'retained_skeleton_generated'}

    def validate_certificate_manifest(self,vir,map_,manifest,vc_,value):
        exact(value,CERT_FIELDS);self.validate_vc(vir,map_,manifest,vc_)
        need(canonical(value)==canonical(certificate_manifest(manifest,vc_)),'acyclic exact frontend-manifest/VC/certificate-manifest linkage')
        return {'kind':'certificate_manifest_linked_unproved'}
