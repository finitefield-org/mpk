"""Closed source-bound theorem requests; ordinary elaboration is still required."""
import copy
from proof_canonical import canonical,digest,seal
from shared_project import Project,normalize_contract
from shared_vir import go_contract
from operation_signatures import need

SCHEMA='mpk.go.obligation_program.v1';DOMAIN='MPK-GO-OBLIGATION-PROGRAM-1.0'
REQUEST_SCHEMA='mpk.go.theorem_request.v1';REQUEST_DOMAIN='MPK-GO-THEOREM-REQUEST-1.0'
FAMILIES=('function_source_binding','entry_representation','contract_definedness','postcondition','function_frame',
 'partial_correctness','function_totality','node_member','access_member','call_precondition','call_relation',
 'call_totality','adapter_binding','region_control','loop_initialization','loop_preservation','loop_exit',
 'loop_variant_domain','loop_variant_decrease')
FUNCTION_FAMILIES={'function_source_binding','entry_representation','function_frame','partial_correctness','function_totality'}


class OrdinaryElaborationPending(RuntimeError):pass


def body_regions(region):
    yield region
    for node in region['nodes']:
        if node['kind']=='apply':
            for r in node['regions']:yield from body_regions(r['region'])
        elif node['kind']=='scope':yield from body_regions(node['region'])
    if region['terminator']['kind']=='resume':yield from body_regions(region['terminator']['continuation'])


def local_requirements(site,node):
    """All existing concrete members/access points,without free predicate text."""
    out=[]
    for m in site['members']:
        out.append({'kind':'node_member','node_id':node['id'],'node_kind':node['kind'],
            'member_category':m['category'],'member_ordinal':m['ordinal']})
    for ordinal,access in enumerate(site['access']):out.append({'kind':'access_member','node_id':node['id'],'access_ordinal':ordinal,'access':access})
    return out


def program(vir,map_,manifest,project):
    context=vir['semantic_context'];bodies={b['function_decl_id']:b for b in vir['program']['function_bodies']}
    contracts={d:normalize_contract(project.scope(d)) for d in bodies};built={};active=set()
    def function(decl):
        if decl in built:return built[decl]
        need(decl not in active,'acyclic Go theorem dependency closure');active.add(decl)
        body=bodies[decl];contract=contracts[decl];scope=project.scope(decl)
        callees=sorted({d['function_decl_id'] for s in body['obligation_sites'] for d in s['dependencies'] if 'function_decl_id' in d})
        callee_requests={d:function(d)['requests'] for d in callees};requests=[]
        def add(kind,parameters=None,origin=None,dependencies=()):
            need(kind in FAMILIES,'closed Go theorem request family')
            requirement={'kind':kind,**({} if parameters is None else parameters)}
            origin={'kind':'map','reference':{'kind':'function','function_decl_id':decl}} if origin is None else origin
            r=seal(REQUEST_SCHEMA,REQUEST_DOMAIN,{'semantic_context':context,'source_ir_hash':vir['vir_hash'],
                'source_scope_sha256':scope.value['content_sha256'],'function_decl_id':decl,'body_hash':body['body_hash'],
                'contract_hash':go_contract(scope)['contract_hash'],'requirement':requirement,'origin':origin,
                'dependencies':sorted(set(dependencies))},'request_hash')
            requests.append(r);return r['request_hash']
        def callee_ids(d,kinds):return [r['request_hash'] for r in callee_requests[d] if r['requirement']['kind'] in kinds]
        add('function_source_binding',dependencies=[x for d in callees for x in callee_ids(d,{'function_source_binding'})])
        add('entry_representation');add('function_frame');add('partial_correctness')
        if contract['termination']=='total':add('function_totality')
        def clause(kind,index,expr,loop=None):
            ref={'kind':kind,'ordinal':index,'loop_ref_id':loop}
            origin={'kind':'contract','contract_hash':go_contract(scope)['contract_hash'],'source_overlay_object_id':scope.overlay_id,'clause':ref}
            add('contract_definedness',{'clause':ref,'expression':expr},origin)
            if kind=='ensures':add('postcondition',{'clause':ref,'expression':expr},origin)
        for kind in ('requires','ensures'):
            for index,expr in enumerate(contract[kind]):clause(kind,index,expr)
        for loop in contract['loops']:
            ref=loop['loop_ref_id']
            for index,expr in enumerate(loop['invariants']):clause('loop_invariant',index,expr,ref)
            for kind in ('loop_initialization','loop_preservation','loop_exit'):add(kind,{'loop_ref_id':ref})
            if loop['variant']['kind']=='candidate':
                expr=loop['variant']['expression'];clause('loop_variant',0,expr,ref)
                for kind in ('loop_variant_domain','loop_variant_decrease'):add(kind,{'loop_ref_id':ref,'expression':expr})
        nodes={n['id']:n for r in body_regions(body['body']) for n in r['nodes']}
        for region in body_regions(body['body']):
            add('region_control',{'region_id':region['id'],'terminator_kind':region['terminator']['kind']},
                {'kind':'map','reference':{'kind':'terminator','function_decl_id':decl,'region_id':region['id']}})
        for site in body['obligation_sites']:
            node=nodes[site['node_id']];origin={'kind':'map','reference':{'kind':'instruction','function_decl_id':decl,'node_id':node['id']}}
            for requirement in local_requirements(site,node):add(requirement['kind'],{k:v for k,v in requirement.items() if k!='kind'},origin)
            for dependency in site['dependencies']:
                if 'function_decl_id' in dependency:
                    d=dependency['function_decl_id'];parameters={'node_id':node['id'],'callee_decl_id':d,'dependency':dependency}
                    add('call_precondition',parameters,origin,callee_ids(d,{'entry_representation','contract_definedness'}))
                    add('call_relation',parameters,origin,callee_ids(d,{'function_source_binding','function_frame','partial_correctness','postcondition'}))
                    if contract['termination']=='total':
                        # Conditional at the caller's actual invocation arguments;
                        # an unannotated partial callee is not silently made total.
                        add('call_totality',parameters,origin,callee_ids(d,{'function_source_binding'}))
                else:add('adapter_binding',{'node_id':node['id'],'dependency':dependency},origin)
        active.remove(decl)
        built[decl]={'function_decl_id':decl,'signature_id':scope.signature,'body_hash':body['body_hash'],
            'contract_hash':go_contract(scope)['contract_hash'],'callees':callees,'requests':requests}
        return built[decl]
    for d in sorted(bodies):function(d)
    functions=[built[d] for d in sorted(built)];all_ids=[r['request_hash'] for f in functions for r in f['requests']]
    need(len(all_ids)==len(set(all_ids)),'unique concrete Go theorem requests')
    pending=[{'function_decl_id':d,**row} for d in sorted(contracts) for row in contracts[d]['pending_candidates']]
    # This prototype has no registered complete source/operation elaborator.
    # Missing definitions are derived data and cannot be supplied as proof facts.
    return seal(SCHEMA,DOMAIN,{'semantic_context':context,'source_ir_hash':vir['vir_hash'],'source_map_hash':map_['source_map_hash'],
        'source_manifest_hash':manifest['source_manifest_hash'],'foundation_descriptor':context['foundation_descriptor'],
        'functions':functions,'unelaborated_request_ids':sorted(all_ids),'pending_candidates':pending},'program_hash')


def require_materialized(value):
    if value['unelaborated_request_ids'] or value['pending_candidates']:
        raise OrdinaryElaborationPending('Go ordinary source/operation/goal elaboration remains unavailable; no certificate skeleton')
