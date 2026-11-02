"""Typed Go structured SSA body proposal; no source/ordinary semantic proof."""
import copy
from dataclasses import dataclass
from operation_signatures import Signatures, STATE, source, location, uint, need, exact, ROOT, HERE, CAPS, ROOT_FIELDS as OP_ROOT_FIELDS
from contract_scope import Scope
from contract_wire import normalize_contract
from body_canonical import canonical,digest,seal

RECORD=ROOT/"develop/migrations/go-practical/t01-w04/vir-body"
SCHEMA="mpk.go.function_body.v1"
DOMAIN="MPK-GO-FUNCTION-BODY-1.0"
ROOT_FIELDS=("schema","semantic_context","capture_context_id","source_scope_sha256","function_decl_id",
             "signature_id","contract_hash","operation_definitions","entry_arguments","body","obligation_sites","body_hash")
NODE_FIELDS={
    "apply":("kind","id","definition_hash","inputs","outputs","regions","source_ref"),
    "address":("kind","id","root","path","state","output","source_ref"),
    "load":("kind","id","location","state","output","source_ref"),
    "project":("kind","id","product","index","post_state","output","source_ref"),
    "values":("kind","id","inputs","output","source_ref"),
    "declare":("kind","id","binding_id","value","state","output","source_ref"),
    "scope":("kind","id","state","output","region","source_ref"),
}
REGION_FIELDS=("id","parameters","captures","nodes","terminator")
TERM_FIELDS={"yield":("kind","values"),"normal":("kind","state"),"resume":("kind","control","continuation")}


@dataclass
class Value:
    type: dict
    origin: object=None


@dataclass
class Result:
    values: list
    controls: set
    bindings: set


class BodyValidator:
    def __init__(self,model,definitions):
        # Build from raw buffers again: mutable producer objects are not authority.
        self.m=Signatures(Scope(__import__('json').loads(model.raw_bytes),model.universe,model.overlay_path,model.raw_bytes))
        need(type(definitions) is list and len(definitions)<=CAPS['shared.instructions_per_function'],"bounded complete concrete definition array")
        # This model is already freshly reconstructed; avoid rebuilding it a second time.
        for d in definitions:
            exact(d,OP_ROOT_FIELDS)
            expected=self.m.derive(d['op'],d['parameters'])
            need(canonical(d)==canonical(expected),"independently reconstructed complete concrete definition")
        need([d['operation_hash'] for d in definitions]==sorted({d['operation_hash'] for d in definitions}),"canonical unique operation definitions")
        self.defs={d['operation_hash']:copy.deepcopy(d) for d in definitions}
        self.next_value=0;self.next_node=0;self.next_region=0;self.used_defs=set();self.sites=[]
        self.bindings={b['id']:b for b in self.m.scope.value['bindings']}
        self.declarations=set()

    def type(self,t):
        need(type(t) is dict and type(t.get('kind')) is str,"closed operation type")
        k=t['kind']
        if k in {'source','location'}:exact(t,{'kind','source_type_id'});self.m.root(t['source_type_id'])
        elif k in {'joint','control','values'}:
            field='types' if k=='values' else 'result_types';exact(t,{'kind',field})
            need(type(t[field]) is list and len(t[field])<=(CAPS['capture.typed_syntax_nodes_across_included_packages'] if k=='values' else 16),"ordered concrete type product")
            for v in t[field]:self.m.root(v)
            if k=='control':need(t[field]==self.m.fn_results,"same-function control result shape")
        else:need(k=='state',"closed six operation types");exact(t,{'kind'})

    def allocate(self,ident,ty,origin=None):
        need(type(ident) is int and ident==self.next_value,"canonical globally unique SSA value preorder")
        self.type(ty);self.next_value+=1;need(self.next_value<=CAPS['shared.locals'],"all explicit SSA bindings count toward function locals")
        return Value(copy.deepcopy(ty),origin)

    def anchor(self,ref,stack):
        exact(ref,{'source_object_id','start_byte','end_byte','ordinal'})
        need(ref['source_object_id']==self.m.scope.source_id,"same captured source object")
        start,end,ordinal=(uint(ref[k]) for k in ('start_byte','end_byte','ordinal'))
        f=self.m.scope.function;need(f['start_byte']<=start<end<=f['end_byte'],"physical same-function candidate source span")
        if stack:need(stack[-1]['start_byte']<=start<end<=stack[-1]['end_byte'],"nested control source bounds")
        return copy.deepcopy(ref)

    def target(self,d,anchor):
        n=int(d['op'][-2:])
        if n==16:return {'kind':'switch',**anchor}
        ordinal=d['parameters']['loop'];loop=next(l for l in self.m.scope.function['loops'] if l['ordinal']==ordinal)
        ref={'kind':'loop','source_object_id':self.m.scope.source_id,'start_byte':loop['start_byte'],'end_byte':loop['end_byte'],'ordinal':ordinal}
        need({k:v for k,v in ref.items() if k!='kind'}==anchor,"loop anchor matches exact captured selector")
        return ref

    def site(self,node,members,access=(),dependencies=()):
        self.sites.append({'node_id':node['id'],'source_ref':copy.deepcopy(node['source_ref']),
                           'members':copy.deepcopy(members),'access':copy.deepcopy(list(access)),
                           'dependencies':copy.deepcopy(list(dependencies))})

    def dependencies(self,d):
        n=int(d['op'][-2:]);rows=[]
        if n in {18,19,26}:
            for b in d['binding_requirements']:
                decl=self.m.scope.graph
                node=next(x for x in decl if x['id']==b['function_decl_id'])
                contract={'kind':'source_body'}
                if node['id']==self.m.scope.declaration:
                    contract={'kind':'overlay','contract_hash':normalize_contract(self.m.scope)['contract_hash']}
                rows.append({'function_decl_id':node['id'],'signature_id':b['signature_id'],
                             'source_object_id':node['source_object_id'],'start_byte':node['start_byte'],'end_byte':node['end_byte'],
                             'relation':contract})
        elif n in {33,34}:rows=copy.deepcopy(d['binding_requirements'])
        return rows

    def region(self,r,expected_inputs,expected_outputs,outer=None,stack=(),live=None):
        exact(r,REGION_FIELDS);need(type(r['id']) is int and r['id']==self.next_region,"canonical region preorder")
        self.next_region+=1
        need(self.next_region<=CAPS['shared.blocks_per_function'],"all regions/continuations count toward function blocks")
        need(type(r['parameters']) is list and len(r['parameters'])==len(expected_inputs),"exact region parameter arity")
        need(type(r['captures']) is list and all(type(i) is int for i in r['captures']) and r['captures']==sorted(set(r['captures'])),"sorted unique lexical captures")
        outer={} if outer is None else outer;env={};captured=set(r['captures']);used_captures=set()
        active={b['id'] for b in self.bindings.values() if b['kind']!='local'} if live is None else set(live)
        need(len(r['parameters'])+len(captured)<=CAPS['shared.block_parameters'],"explicit arguments plus captures count toward block parameters")
        for ident in captured:
            need(ident in outer and outer[ident].type['kind']!='control',"only dominated noncontrol captures")
            env[ident]=outer[ident]
        current=None;pending=None
        for p,ty in zip(r['parameters'],expected_inputs):
            exact(p,{'id','type'});need(canonical(p['type'])==canonical(ty),"independently derived region parameter type")
            env[p['id']]=self.allocate(p['id'],ty,{'kind':'parameter'})
            if ty['kind']=='state':need(current is None,"one current state parameter");current=p['id']
        def get(ident,ty=None):
            need(type(ident) is int and ident in env,"dominating same-region or explicitly captured SSA value")
            if ident in captured:used_captures.add(ident)
            v=env[ident]
            if ty is not None:need(canonical(v.type)==canonical(ty),"exact nominal/concrete operand type")
            return v
        def state(ident):get(ident,STATE);need(ident==current,"current state cannot be replayed,rolled back or borrowed from sibling path")
        need(type(r['nodes']) is list and len(r['nodes'])<=CAPS['shared.instructions_per_block'],"bounded region nodes")
        for node in r['nodes']:
            need(pending is None,"control result must be propagated or resumed before further execution")
            need(type(node) is dict and type(node.get('kind')) is str and node['kind'] in NODE_FIELDS,"closed body node union")
            k=node['kind'];exact(node,NODE_FIELDS[k]);need(type(node['id']) is int and node['id']==self.next_node,"canonical node preorder")
            self.next_node+=1;need(self.next_node<=CAPS['shared.instructions_per_function'],"complete function node count")
            anchor=self.anchor(node['source_ref'],stack)
            if k=='apply':
                need(type(node['definition_hash']) is str and node['definition_hash'] in self.defs,"resolved concrete operation definition")
                d=self.defs[node['definition_hash']];self.used_defs.add(node['definition_hash']);n=int(d['op'][-2:])
                need(type(node['inputs']) is list and len(node['inputs'])==len(d['inputs']),"complete operand arity")
                values=[get(i,t) for i,t in zip(node['inputs'],d['inputs'])]
                state_ids=[i for i,t in zip(node['inputs'],d['inputs']) if t['kind']=='state']
                if n==18:
                    origin=values[0].origin
                    need(type(origin) is dict and origin.get('kind')=='invocation' and origin['callee']==d['parameters']['callee'] and origin['post_state']==node['inputs'][1],"projection uses exact same static invocation and its immutable post-state")
                else:
                    for ident in state_ids:state(ident)
                if n in {54,55}:need(canonical(d['parameters']['enclosing'])==canonical(list(stack)),"abrupt target stack reconstructed from enclosing graph")
                need(type(node['outputs']) is list and len(node['outputs'])==len(d['outputs']),"complete result arity")
                output_values=[self.allocate(i,t) for i,t in zip(node['outputs'],d['outputs'])]
                need(type(node['regions']) is list and len(node['regions'])==len(d['regions']),"all guarded/control regions present")
                child_stack=stack+(self.target(d,anchor),) if n in {16,51,52,53} else stack
                children=[];child_live=set(active)
                for child,spec in zip(node['regions'],d['regions']):
                    exact(child,{'role','region'});need(child['role']==spec['role'],"ordered exact region role")
                    result=self.region(child['region'],spec['inputs'],spec['outputs'],env,child_stack,child_live)
                    if n==51 and child['role']=='init':child_live=result.bindings
                    children.append((child['role'],result))
                    for ident in child['region']['captures']:
                        if ident in captured:used_captures.add(ident)
                controls=set()
                if n==20:controls={'return'}
                elif n in {54,55}:
                    target=d['binding_requirements'][0]['target'];controls={('break:' if n==54 else 'continue:')+canonical(target).hex()}
                elif n in {15,16,51,52,53}:
                    for _,result in children:controls.update(result.controls)
                    if n==16:
                        tag='break:'+canonical(child_stack[-1]).hex()
                        if tag in controls:controls.remove(tag);controls.add('normal')
                        if d['parameters']['default_index'] is None:controls.add('normal')
                    elif n in {51,52,53}:
                        target=canonical(child_stack[-1]).hex();has_break='break:'+target in controls
                        controls.discard('break:'+target);controls.discard('continue:'+target);controls.discard('normal')
                        if n!=51 or d['parameters']['has_test'] or has_break:controls.add('normal')
                for ident,value in zip(node['outputs'],output_values):env[ident]=value
                if n in {19,26}:
                    callee=d['parameters']['callee'] if n==19 else 'closed-error:'+self.m.defs['error_universe_id']
                    env[node['outputs'][0]].origin={'kind':'invocation','node_id':node['id'],'callee':callee,'post_state':node['outputs'][1]}
                elif n==17:env[node['outputs'][0]].origin={'kind':'pack','values':list(node['inputs'])}
                elif n==18:env[node['outputs'][0]].origin={'kind':'projection','invocation':copy.deepcopy(values[0].origin),'index':d['parameters']['index']}
                for ident,ty in zip(node['outputs'],d['outputs']):
                    if ty['kind']=='state':current=ident
                    if ty['kind']=='control':env[ident].origin={'kind':'control','controls':controls};pending=ident;current=None
                access=[]
                if n in {12,21,22,42}:
                    mode='view' if n==42 else 'read_write' if n==22 else 'write'
                    for j,(ident,ty) in enumerate(zip(node['inputs'],d['inputs'])):
                        if ty['kind']=='location':
                            point={'kind':'current','value':state_ids[0]}
                            if n==21:
                                target=[i for i,t in enumerate(d['parameters']['targets']) if t is not None][j]
                                point={'kind':'assignment_prefix','entry':state_ids[0],'target_ordinal':target}
                            access.append({'operand':j,'mode':mode,'state_ref':point,'location':copy.deepcopy(get(ident).origin),'checks':['full_index_bounds_at_access','same_snapshot_validity','permission_and_frame']})
                self.site(node,d['obligations'],access,self.dependencies(d))
            elif k=='address':
                state(node['state']);root=node['root'];need(type(root) is dict and type(root.get('kind')) is str,"closed location root")
                if root['kind']=='binding':
                    exact(root,{'kind','binding_id'});bid=root['binding_id'];need(type(bid) is str and bid in self.bindings,"captured lexical variable binding")
                    b=self.bindings[bid]
                    need(bid in active,"local declaration must dominate this lexical scope")
                    ty=b['source_type_id']
                else:
                    exact(root,{'kind','descriptor','index'});need(root['kind']=='slice',"no arbitrary pointer/root")
                    sv=get(root['descriptor']);need(sv.type['kind']=='source',"frozen source slice descriptor")
                    shape=self.m.root(sv.type['source_type_id'],{'slice'});self.index(get(root['index']));ty=shape['element']
                need(type(node['path']) is list and len(node['path'])<=CAPS['shared.aggregate_type_nesting'],"bounded typed location path")
                for step in node['path']:
                    need(type(step) is dict and type(step.get('kind')) is str,"closed typed path step")
                    shape=self.m.root(ty);need(step.get('owner_type_id')==ty,"exact nominal location owner")
                    if step['kind']=='field':
                        exact(step,{'kind','owner_type_id','ordinal'});need(shape['kind']=='struct',"direct field path")
                        ordinal=uint(step['ordinal'],len(shape['fields'])-1)
                        need(self.m.nodes[self.m.underlying(ty)]['fields'][ordinal]['name']!='_',"no blank field observation")
                        ty=shape['fields'][ordinal]['type']
                    else:
                        exact(step,{'kind','owner_type_id','index'});need(step['kind']=='array' and shape['kind']=='array',"slice crossings require a separately frozen descriptor root")
                        self.index(get(step['index']));ty=shape['element']
                origin={'kind':'location','root':copy.deepcopy(root),'path':copy.deepcopy(node['path']),'captured_state':node['state']}
                env[node['output']]=self.allocate(node['output'],location(ty),origin)
                self.site(node,[{'category':'ordinary','ordinal':0,'predicate':'location_source_binding_and_lexical_lifetime'}])
            elif k=='load':
                state(node['state']);lv=get(node['location']);need(lv.type['kind']=='location',"internal location load")
                env[node['output']]=self.allocate(node['output'],source(lv.type['source_type_id']))
                self.site(node,[{'category':'ordinary','ordinal':0,'predicate':'complete_value_read_source_binding'}],
                          [{'operand':0,'mode':'read','state_ref':{'kind':'current','value':node['state']},'location':copy.deepcopy(lv.origin),'checks':['full_index_bounds_at_access','initialized_live_location','same_snapshot_validity','read_permission']}])
            elif k=='project':
                product=get(node['product']);need(product.type['kind']=='joint',"one complete ordered product")
                index=uint(node['index'],len(product.type['result_types'])-1);origin=product.origin
                need(type(origin) is dict and origin.get('kind') in {'invocation','pack'},"derived product provenance")
                if origin['kind']=='invocation':
                    get(node['post_state'],STATE);need(node['post_state']==origin['post_state'],"same invocation post-state,including closed error dispatch")
                else:need(node['post_state'] is None,"ordinary pack has no invented invocation state")
                env[node['output']]=self.allocate(node['output'],source(product.type['result_types'][index]),{'kind':'projection','product':node['product'],'index':index})
                self.site(node,[{'category':'ordinary','ordinal':0,'predicate':'ordered_product_projection_and_same_invocation'}])
            elif k=='values':
                need(type(node['inputs']) is list and len(node['inputs'])<=CAPS['capture.typed_syntax_nodes_across_included_packages'],"ordered frozen RHS value sequence")
                vals=[get(ident) for ident in node['inputs']];need(all(v.type['kind']=='source' for v in vals),"complete source RHS values")
                ty={'kind':'values','types':[v.type['source_type_id'] for v in vals]}
                env[node['output']]=self.allocate(node['output'],ty,{'kind':'values','values':list(node['inputs'])})
                self.site(node,[{'category':'ordinary','ordinal':0,'predicate':'freeze_complete_RHS_values_once_before_ordered_stores'}])
            elif k=='declare':
                state(node['state']);bid=node['binding_id']
                need(type(bid) is str and bid in self.bindings and self.bindings[bid]['kind']=='local' and bid not in active and bid not in self.declarations,"one captured local declaration per static source binding")
                get(node['value'],source(self.bindings[bid]['source_type_id']));active.add(bid);self.declarations.add(bid)
                env[node['output']]=self.allocate(node['output'],STATE);current=node['output']
                self.site(node,[{'category':'ordinary','ordinal':0,'predicate':'local_declaration_initialization_and_activation_lifetime'}])
            else:
                state(node['state']);control={'kind':'control','result_types':self.m.fn_results}
                output=self.allocate(node['output'],control)
                child=self.region(node['region'],[STATE],[control],env,stack,active)
                for ident in node['region']['captures']:
                    if ident in captured:used_captures.add(ident)
                output.origin={'kind':'control','controls':child.controls};env[node['output']]=output;pending=node['output'];current=None
                self.site(node,[{'category':'ordinary','ordinal':0,'predicate':'lexical_scope_normal_and_abrupt_lifetime_relation'}])
        term=r['terminator'];need(type(term) is dict and type(term.get('kind')) is str and term['kind'] in TERM_FIELDS,"closed body terminator")
        k=term['kind'];exact(term,TERM_FIELDS[k]);controls=set();outputs=[]
        if k=='yield':
            need(type(term['values']) is list and len(term['values'])==len(expected_outputs),"exact region yield arity")
            for ident,ty in zip(term['values'],expected_outputs):
                v=get(ident,ty);outputs.append(v)
                if ty['kind']=='state':state(ident)
                if ty['kind']=='control':need(ident==pending,"propagate exact pending control");controls.update(v.origin['controls'])
        elif k=='normal':
            need(expected_outputs==[{'kind':'control','result_types':self.m.fn_results}] and pending is None,"normal control exit only")
            state(term['state']);controls={'normal'}
        else:
            need(expected_outputs==[{'kind':'control','result_types':self.m.fn_results}] and term['control']==pending and pending is not None,"only resume the exact pending control")
            v=get(term['control']);continuation=self.region(term['continuation'],[STATE],expected_outputs,env,stack,active)
            for ident in term['continuation']['captures']:
                if ident in captured:used_captures.add(ident)
            controls=set(v.origin['controls'])-{'normal'}
            if 'normal' in v.origin['controls']:controls.update(continuation.controls)
        need(used_captures==captured,"no unused or hidden lexical capture")
        return Result(outputs,controls,active)

    def index(self,v):
        need(v.type['kind']=='source',"full source integer index")
        self.m.integer(v.type['source_type_id'])

    def function(self,body):
        canonical(body) # Full JSON nesting/size guards precede recursive region analysis.
        ins=[source(self.m.id(b['type'])) for b in self.m.scope.function['inputs']]+[STATE]
        result=self.region(body,ins,[{'kind':'control','result_types':self.m.fn_results}])
        need(not any(c.startswith(('break:','continue:')) for c in result.controls),"no abrupt target escapes its handler")
        need(not self.m.fn_results or 'normal' not in result.controls,"all normal function exits establish ordered results")
        need(self.used_defs==set(self.defs),"exact reachable concrete operation definition set")
        return {'obligation_sites':sorted(self.sites,key=lambda x:x['node_id']),'controls':sorted(result.controls)}


def body_record(model,body,definitions):
    need(type(definitions) is list and len(definitions)<=CAPS['shared.instructions_per_function'],"bounded concrete definition array before sorting")
    for d in definitions:
        exact(d,OP_ROOT_FIELDS)
        h=d['operation_hash'];need(type(h) is str and len(h)==64 and all(c in '0123456789abcdef' for c in h),"canonical definition digest before sorting")
    definitions=sorted(copy.deepcopy(definitions),key=lambda d:d['operation_hash'])
    validator=BodyValidator(model,definitions);analysis=validator.function(body);s=validator.m.scope
    return seal(SCHEMA,DOMAIN,{'semantic_context':s.context,'capture_context_id':s.capture,
        'source_scope_sha256':s.value['content_sha256'],'function_decl_id':s.declaration,'signature_id':s.signature,
        'contract_hash':normalize_contract(s)['contract_hash'],'operation_definitions':definitions,
        'entry_arguments':[{'binding_id':bid,'value_id':i} for i,bid in enumerate(s.inputs)],
        'body':body,'obligation_sites':analysis['obligation_sites']},'body_hash')


def validate_body(model,record):
    exact(record,ROOT_FIELDS)
    expected=body_record(model,record['body'],record['operation_definitions'])
    need(canonical(record)==canonical(expected),"complete body/context/source/contract/SSA/region/obligation/hash linkage")
    return True
