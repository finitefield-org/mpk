"""All56 body components, full function candidates and static-flow corruptions."""
import copy
from operation_signature_vectors import model,parameters,enclosing
from operation_signatures import Signatures,STATE,source
from contract_scope import Scope
from contract_wire_vectors import source_cases
from vir_body import BodyValidator,validate_body,canonical,digest,DOMAIN
from vir_body_build import Builder,component


def components(m):
    p=parameters(m)
    return [component(m,n,p[n],[enclosing(m)] if n in {54,55} else ()) for n in range(1,57)]


def loop_function(m):
    b=Builder(m)
    i=next(binding for binding in m.scope.value['bindings'] if binding['name']=='i')
    flag=m.scope.inputs[2];xs=m.scope.inputs[0];I=m.id('int');B=m.id('bool')
    def function(r):
        def for_regions(role,c,outputs):
            if role=='init':
                zero=c.zero(I);state=c.declare(i['id'],zero);c.yield_([state])
            elif role=='test':
                index=c.load(c.address(i['id']));sl=c.load(c.address(xs))
                length=c.apply(37,{'type':m.id('[]int64'),'size':'length'},[sl])[0]
                cond=c.apply(9,{'type':I,'operator':'less'},[index,length])[0];c.yield_([cond,c.state])
            elif role=='post':
                loc=c.address(i['id']);one=c.apply(2,{'type':I,'value':'1'},[])[0]
                state=c.apply(22,{'type':I,'rhs_type':I,'operator':'add'},[loc,one,c.state])[0];c.yield_([state])
            else:
                cond=c.load(c.address(flag))
                def branches(branch,k,types):
                    op=54 if branch=='then' else 55
                    control=k.apply(op,{'enclosing':list(k.stack)},[k.state])[0];k.yield_([control])
                control=c.apply(15,{'condition_type':B},[cond,c.state],branches)[0];c.yield_([control])
        control=r.apply(51,{'loop':0,'condition_type':B,'has_test':True},[r.state],for_regions)[0]
        r.resume(control,lambda c:c.return_zero())
    return b.function(function)


def functions(m):
    rows=[{'id':'straight-return-candidate','model':'scopes','record':Builder(m).function(lambda r:r.return_zero())},
          {'id':'loop-abrupt-candidate','model':'scopes','record':loop_function(m)}]
    source=next(r for r in source_cases() if r['id']=='nested-and-tail-error-entry-sets')
    other=Signatures(Scope(source['raw'],source['universe']))
    rows.append({'id':'empty-original-function-candidate','model':'input-errors','raw':other.scope.raw,
                 'universe':list(other.universe),'record':Builder(other).function(lambda r:r.normal())})
    return rows


def nodes(region):
    for node in region['nodes']:
        yield node
        if node['kind']=='apply':
            for child in node['regions']:yield from nodes(child['region'])
        elif node['kind']=='scope':yield from nodes(node['region'])
    if region['terminator']['kind']=='resume':yield from nodes(region['terminator']['continuation'])


def reseal(record):
    record['body_hash']=digest(DOMAIN,{k:v for k,v in record.items() if k!='body_hash'});return record


def vector_specs():
    return [{'id':name,'base':base,'expected':expected} for name,base,expected in (
        ('straight-valid',0,'typed_unproved'),('loop-valid',1,'typed_unproved'),('empty-valid',2,'typed_unproved'),
        ('unknown-root',0,'configuration_rejection'),('wrong-context',0,'configuration_rejection'),
        ('forward-use',0,'configuration_rejection'),('duplicate-value',0,'configuration_rejection'),
        ('skip-value-id',0,'configuration_rejection'),('wrong-result-order',0,'configuration_rejection'),
        ('omit-obligation-site',0,'configuration_rejection'),('alter-access-phase',1,'configuration_rejection'),
        ('unused-definition',0,'configuration_rejection'),('foreign-definition',0,'configuration_rejection'),
        ('state-rollback',1,'configuration_rejection'),('swapped-regions',1,'configuration_rejection'),
        ('wrong-header-binding',1,'configuration_rejection'),('forged-target-stack',1,'configuration_rejection'),
        ('normal-result-function-exit',0,'configuration_rejection'),('instruction-after-control',0,'configuration_rejection'),
        ('wrong-source-object',0,'configuration_rejection'))]


def alter(spec,records,m):
    record=copy.deepcopy(records[spec['base']]['record']);name=spec['id']
    if spec['expected']=='typed_unproved':return record
    ns=list(nodes(record['body']))
    if name=='unknown-root':record['trusted']=True
    elif name=='wrong-context':record['semantic_context']['source_language']='rust'
    elif name=='forward-use':ns[-2]['inputs'][0]=ns[-1]['outputs'][0]
    elif name=='duplicate-value':ns[1]['outputs'][0]=ns[0]['outputs'][0]
    elif name=='skip-value-id':ns[0]['outputs'][0]+=1
    elif name=='wrong-result-order':ns[-2]['inputs'].reverse()
    elif name=='omit-obligation-site':record['obligation_sites'].pop()
    elif name=='alter-access-phase':next(s for s in record['obligation_sites'] if s['access'])['access'][0]['state_ref']={'kind':'current','value':0}
    elif name=='unused-definition':record['operation_definitions'].append(m.derive('GP-OP-02',{'type':m.id('int'),'value':'7'}));record['operation_definitions'].sort(key=lambda d:d['operation_hash'])
    elif name=='foreign-definition':record['operation_definitions'][0]['semantic_context']['source_language']='rust'
    elif name=='state-rollback':next(n for n in ns if n['kind']=='apply' and n['inputs'] and any(s['node_id']==n['id'] and s['access'] for s in record['obligation_sites']))['inputs'][-1]=record['body']['parameters'][-1]['id']
    elif name=='swapped-regions':record['body']['nodes'][0]['regions'][0:2]=reversed(record['body']['nodes'][0]['regions'][0:2])
    elif name=='wrong-header-binding':next(n for n in ns if n['kind']=='address')['root']['binding_id']='0'*64
    elif name=='forged-target-stack':
        d=next(d for d in record['operation_definitions'] if d['op']=='GP-OP-54');old=d['operation_hash'];new=m.derive('GP-OP-54',{'enclosing':[enclosing(m,1)]})
        record['operation_definitions']=[new if x['operation_hash']==old else x for x in record['operation_definitions']];record['operation_definitions'].sort(key=lambda d:d['operation_hash'])
        next(n for n in ns if n.get('definition_hash')==old)['definition_hash']=new['operation_hash']
    elif name=='normal-result-function-exit':record['body']['terminator']={'kind':'normal','state':record['body']['parameters'][-1]['id']}
    elif name=='instruction-after-control':record['body']['nodes'].append(copy.deepcopy(ns[0]))
    elif name=='wrong-source-object':ns[0]['source_ref']['source_object_id']='0'*64
    else:raise AssertionError(name)
    return reseal(record)


def catalogue():
    from vir_body import SCHEMA,DOMAIN,ROOT_FIELDS,NODE_FIELDS,REGION_FIELDS,TERM_FIELDS
    fields={'schema':'literal mpk.go.function_body.v1','semantic_context':'complete validated Go practical context.v3',
            'capture_context_id':'same complete capture id','source_scope_sha256':'complete reconstructed contract-scope hash',
            'function_decl_id':'same selected function id','signature_id':'same concrete signature id',
            'contract_hash':'complete normalized contract hash','operation_definitions':'hash-sorted unique complete concrete operation definitions',
            'entry_arguments':'signature-ordered binding_id/value_id records','body':'one closed root region',
            'obligation_sites':'node-preorder complete derived requirement sites,not proofs','body_hash':'lowercase sha256'}
    assert set(fields)==set(ROOT_FIELDS)
    return {'schema':'mpk.go_practical.vir_body_draft.v0','body_schema':SCHEMA,'hash_domain':DOMAIN,
            'root_required_fields':list(ROOT_FIELDS),'optional_fields':[],'unknown_fields':'reject',
            'root_field_types':fields,
            'hash_preimage_fields':[x for x in ROOT_FIELDS if x!='body_hash'],
            'node_arms':{k:list(v) for k,v in NODE_FIELDS.items()},'region_required_fields':list(REGION_FIELDS),
            'terminator_arms':{k:list(v) for k,v in TERM_FIELDS.items()},
            'apply_region_fields':['role','region'],'parameter_fields':['id','type'],
            'source_ref_fields':['source_object_id','start_byte','end_byte','ordinal'],
            'address_root_arms':{'binding':['kind','binding_id'],'slice':['kind','descriptor','index']},
            'address_path_arms':{'field':['kind','owner_type_id','ordinal'],'array':['kind','owner_type_id','index']},
            'obligation_site_fields':['node_id','source_ref','members','access','dependencies'],
            'access_site_fields':['operand','mode','state_ref','location','checks'],
            'access_state_arms':{'current':['kind','value'],'assignment_prefix':['kind','entry','target_ordinal']},
            'source_relation_fields':['function_decl_id','signature_id','source_object_id','start_byte','end_byte','relation'],
            'relation_arms':{'source_body':['kind'],'overlay':['kind','contract_hash']},
            'counter_mapping':{'shared.locals':'all explicit SSA definitions,including region arguments,state/control/location/product handles',
                               'shared.blocks_per_function':'all regions,including resume continuations',
                               'shared.block_parameters':'parameters plus explicit captures in each region',
                               'shared.instructions_per_block':'all apply/address/load/project/values nodes in the region',
                               'shared.instructions_per_function':'all nodes across all regions',
                               'shared.json_nesting':'actual JSON containers,root at depth1; no64-depth pilot cap'},
            'producer':'go-tools/go2vir/practical_lower.go','consumer':'crates/mpk-vc/src/go_practical_artifacts.rs',
            'vc_owner':'crates/mpk-vc/src/go_practical_vc.rs','test_owner':'crates/mpk-vc/tests/go_practical_artifacts.rs',
            'remaining':['complete shared VIR five-profile wrapper,source-map/manifests/VC/policy/API/release roots and reviewed producer/consumer migration',
                         'independent project/source/call/control/lifetime/activation reconstruction and all ordinary source/operation/permission/frame/totality proofs; W05/T01 and full T02–T11']}
