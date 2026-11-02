"""Closed shared VIR/contract roots for the four retained profiles and practical Go."""
import copy
from collections import defaultdict
from body_canonical import canonical,digest,seal,parse
from identity_catalog import SHARED
from foundation_limits_vectors import model as protocol_model
from context_protocol import legacy_projection
from operation_signatures import ROOT_FIELDS as OP_FIELDS,CAPS,STATE,need,exact
from vir_body import BodyValidator,ROOT_FIELDS as BODY_FIELDS
from shared_project import (Project,ProjectModel,normalize_contract,BODY_SCHEMA,BODY_DOMAIN)

KINDS={'mpk.go.fixed.v0':'go_fixed','mpk.rust.checked.v0':'rust_checked','mpk.csharp.scalar.v0':'csharp_scalar',
       'mpk.java.scalar.v0':'java_scalar','mpk.go.practical.v1':'go_practical'}
CONTRACT_FIELDS=('schema','semantic_context','profile_contract','contract','contract_hash')
VIR_FIELDS=('schema','semantic_context','profile_contract','program','vir_hash')
RETAINED_CONTRACT_FIELDS=('kind','unit_id','function_id','requires','ensures','modifies','panic','termination','loops')


def profile_contract(context):
    profile=context['semantic_profile'];need(profile in KINDS,'one compiled profile')
    return 'mpk.profile.vir.'+KINDS[profile]+'.v2'


def contract(context,value):
    return seal(SHARED['contract'][0],SHARED['contract'][1],{'semantic_context':context,
        'profile_contract':profile_contract(context),'contract':value},'contract_hash')


def go_contract(scope):return contract(scope.context,{'kind':'go_practical','value':normalize_contract(scope)})


class ProjectBody(BodyValidator):
    def __init__(self,project,decl,definitions):
        self.m=ProjectModel(project,decl)
        need(type(definitions) is list and len(definitions)<=CAPS['shared.instructions_per_function'],'bounded concrete operation array')
        for d in definitions:
            exact(d,OP_FIELDS);expected=self.m.derive(d['op'],d['parameters'])
            need(canonical(d)==canonical(expected),'whole-capture concrete operation linkage')
        need([d['operation_hash'] for d in definitions]==sorted({d['operation_hash'] for d in definitions}),'canonical unique operation order')
        self.defs={d['operation_hash']:copy.deepcopy(d) for d in definitions}
        self.next_value=0;self.next_node=0;self.next_region=0;self.used_defs=set();self.sites=[]
        self.bindings={b['id']:b for b in self.m.scope.value['bindings']};self.declarations=set()

    def dependencies(self,d):
        n=int(d['op'][-2:]);rows=[]
        if n in {18,19,26}:
            for b in d['binding_requirements']:
                scope=self.m.project.scope(b['function_decl_id']);shared=go_contract(scope)
                node=next(x for x in scope.graph if x['id']==scope.declaration)
                rows.append({'function_decl_id':scope.declaration,'signature_id':scope.signature,'source_object_id':node['source_object_id'],
                    'start_byte':node['start_byte'],'end_byte':node['end_byte'],
                    'relation':{'kind':'source_body_and_contract','contract_schema':shared['schema'],'contract_hash':shared['contract_hash'],'origin':scope.origin}})
        elif n in {33,34}:rows=copy.deepcopy(d['binding_requirements'])
        return rows


def body(project,decl,graph,definitions):
    need(type(definitions) is list,'concrete definitions')
    for d in definitions:exact(d,OP_FIELDS);need(type(d['operation_hash']) is str,'operation hash')
    definitions=sorted(copy.deepcopy(definitions),key=lambda d:d['operation_hash'])
    v=ProjectBody(project,decl,definitions);analysis=v.function(graph);s=v.m.scope
    return seal(BODY_SCHEMA,BODY_DOMAIN,{'semantic_context':s.context,'capture_context_id':s.capture,'source_scope_sha256':s.value['content_sha256'],
        'function_decl_id':decl,'signature_id':s.signature,'contract_hash':go_contract(s)['contract_hash'],
        'operation_definitions':definitions,'entry_arguments':[{'binding_id':bid,'value_id':i} for i,bid in enumerate(s.inputs)],
        'body':graph,'obligation_sites':analysis['obligation_sites']},'body_hash')


def go_program(project,bodies):
    bodies=sorted(copy.deepcopy(bodies),key=lambda b:b['function_decl_id']);decls=[b['function_decl_id'] for b in bodies]
    return {'kind':'go_practical','capture_context':project.capture,'source_scopes':project.scope_set(decls),
            'contracts':[go_contract(project.scope(d)) for d in decls],'function_bodies':bodies}


def module(context,program):
    return seal(SHARED['vir'][0],SHARED['vir'][1],{'semantic_context':context,'profile_contract':profile_contract(context),'program':program},'vir_hash')


def check_contract_root(value,context,kind):
    exact(value,CONTRACT_FIELDS)
    need(value['schema']==SHARED['contract'][0] and canonical(value['semantic_context'])==canonical(context)
         and value['profile_contract']==profile_contract(context),'exact common contract/context/profile contract')
    payload=value['contract'];need(type(payload) is dict and payload.get('kind')==kind,'context-dispatched contract arm')
    if kind=='go_practical':exact(payload,{'kind','value'})
    else:exact(payload,RETAINED_CONTRACT_FIELDS)
    need(value['contract_hash']==digest(SHARED['contract'][1],{k:v for k,v in value.items() if k!='contract_hash'}),'complete shared contract hash')


def legacy_query(context,units):
    """Explicit private equivalence query; no v1 root is accepted as a v3 module."""
    value=copy.deepcopy(units);contracts={}
    for unit in value:
        exact(unit,{'id','name','type_decls','const_decls','functions'})
        need(type(unit['functions']) is list,'retained function array')
        for f in unit['functions']:
            exact(f,{'id','unit_id','name','params','results','locals','blocks','contracts','features_used'})
            check_contract_root(f['contracts'],context,KINDS[context['semantic_profile']]);payload=f['contracts']['contract']
            need(payload['unit_id']==unit['id']==f['unit_id'] and payload['function_id']==f['id'],'same retained unit/function')
            need(type(f['id']) is str and f['id'] not in contracts,'unique retained functions');contracts[f['id']]=f['contracts']['contract_hash']
    for unit in value:
        for f in unit['functions']:
            need(type(f['blocks']) is list,'retained blocks')
            for b in f['blocks']:
                need(type(b) is dict and type(b.get('instructions')) is list,'retained block shape')
                for op in b['instructions']:
                    if type(op) is dict and op.get('kind')=='CallStatic':
                        need(type(op.get('function')) is str and op['function'] in contracts and op.get('contract_hash')==contracts[op['function']],'new-family exact callee contract before oracle projection')
            payload=f['contracts']['contract'];f['contracts']={k:v for k,v in payload.items() if k!='kind'}
    return {'context':legacy_projection(context)['context'],'units':value}


class ModuleValidator:
    def __init__(self,legacy_policy,project_spec=None):
        self.contexts=protocol_model().contexts;self.legacy_policy=legacy_policy
        self.project_bytes=None if project_spec is None else canonical(project_spec)

    def validate(self,value):
        exact(value,VIR_FIELDS);need(value['schema']==SHARED['vir'][0],'exact shared VIR v3; no old/future root')
        context=value['semantic_context'];self.contexts.validate_context(context)
        need(value['profile_contract']==profile_contract(context),'compiled full profile-contract identity')
        need(value['vir_hash']==digest(SHARED['vir'][1],{k:v for k,v in value.items() if k!='vir_hash'}),'complete shared VIR hash')
        program=value['program'];need(type(program) is dict and program.get('kind')==KINDS[context['semantic_profile']],'profile arm dispatched only by validated context')
        if program['kind']!='go_practical':
            exact(program,{'kind','units'});need(type(program['units']) is list,'retained units')
            answer=self.legacy_policy(legacy_query(context,program['units']))
            need(answer['accepted'] is True,'actual retained importer and typed core importer')
            return {'kind':'retained_native','projection_sha256':answer['projection_sha256'],'vc_sha256':answer['vc_sha256']}
        exact(program,{'kind','capture_context','source_scopes','contracts','function_bodies'})
        need(self.project_bytes is not None,'captured project authority required')
        project=Project(parse(self.project_bytes));need(canonical(context)==canonical(next(iter(project.templates.values())).context),'same whole project context')
        need(type(program['function_bodies']) is list and program['function_bodies'],'complete Go body set')
        decls=[];edges={};nodes=regions=0
        for record in program['function_bodies']:
            exact(record,BODY_FIELDS);decl=record['function_decl_id'];need(type(decl) is str and decl in project.templates,'captured body declaration')
            expected=body(project,decl,record['body'],record['operation_definitions'])
            need(canonical(record)==canonical(expected),'complete whole-capture body reconstruction')
            v=ProjectBody(project,decl,record['operation_definitions']);v.function(record['body']);nodes+=v.next_node;regions+=v.next_region
            need(nodes<=CAPS['shared.instructions_per_module'] and regions<=CAPS['shared.blocks_per_module'],'whole module instruction/block counters')
            deps={d['function_decl_id'] for s in record['obligation_sites'] for d in s['dependencies'] if 'function_decl_id' in d}
            edges[decl]=deps;decls.append(decl)
        need(decls==sorted(set(decls)),'unique declaration-ordered Go bodies')
        need(canonical(program['capture_context'])==canonical(project.capture) and canonical(program['source_scopes'])==canonical(project.scope_set(decls)),'complete shared capture/scope set')
        need(canonical(program['contracts'])==canonical([go_contract(project.scope(d)) for d in decls]),'every body has its exact common contract')
        active=set();done=set()
        def visit(d):
            need(d in edges and d not in active,'complete acyclic source/body dependency closure')
            if d in done:return
            active.add(d)
            for child in sorted(edges[d]):visit(child)
            active.remove(d);done.add(d)
        for d in project.root_ids:visit(d)
        need(done==set(decls),'no omitted roots/dependencies or extra unreachable bodies')
        return {'kind':'go_typed_unproved','functions':len(decls),'instructions':nodes,'regions':regions}
