"""One immutable multi-overlay fixture capture and sidecar-optional function scopes."""
import copy
import json
from contract_scope import Scope,FACTS,BINDING_SCHEMA,BINDING_DOMAIN,source_overlay,need,exact
from overlay_contracts import Types,SOURCE_SCHEMA
from identity_wire import seal as identity_seal,validate_graph,path
from closed_carriers import TYPE_DOMAIN,Specializer
from body_canonical import canonical,parse,digest,seal
from operation_signatures import Signatures,SCHEMA as OLD_OP_SCHEMA
from contract_wire import WireExpressions,context,check_source_budget

SCOPE_SCHEMA='mpk.go.source_scope.v1';SCOPE_DOMAIN='MPK-GO-SOURCE-SCOPE-1.0'
SET_SCHEMA='mpk.go.source_scope_set.v1';SET_DOMAIN='MPK-GO-SOURCE-SCOPE-SET-1.0'
CONTRACT_SCHEMA='mpk.go.practical.contract.v2';CONTRACT_DOMAIN='MPK-GO-PRACTICAL-CONTRACT-2.0'
OP_SCHEMA='mpk.go.operation_definition.v2';OP_DOMAIN='MPK-GO-OPERATION-DEFINITION-2.0'
BODY_SCHEMA='mpk.go.function_body.v2';BODY_DOMAIN='MPK-GO-FUNCTION-BODY-2.0'


def overlay_shape(value):
    """Structural guard independent of the predecessor's whole-overlay hash."""
    exact(value,{'schema','source','function','termination','requires','ensures','error_inputs','loops','modifies'})
    need(value['schema']=='mpk.go.practical.source_overlay.v1','one source overlay generation')
    need(type(value['source']) is str and type(value['termination']) is str and value['termination'] in {'partial','total'},'source path/termination mode')
    for key in ('requires','ensures','error_inputs','loops','modifies'):need(type(value[key]) is list,'source clause/entry/loop arrays')
    for row in value['error_inputs']:
        exact(row,{'input','path','cases'})
        need(type(row['path']) is list and all(type(p) is dict for p in row['path']) and type(row['cases']) is list,'typed entry path and case set')
    for row in value['loops']:
        exact(row,{'selector','invariants','variant'});exact(row['selector'],{'kind','start_byte','end_byte','ordinal'})
        need(type(row['invariants']) is list and type(row['variant']) is dict,'source invariant/variant shape')
        v=row['variant']
        if v.get('kind')=='absent':exact(v,{'kind'})
        else:exact(v,{'kind','expression'});need(v['kind']=='candidate','closed source variant')


def default_overlay(f,facts,universe):
    types=Types(facts,universe);entries=[]
    for i,b in enumerate(f['inputs']):
        entries += [{'input':i,'path':list(p),'cases':list(types.universe)} for p in types.errors(b['type'])]
    old={'schema':SOURCE_SCHEMA,'source':facts['path'],'function':{'name':f['name'],**({'receiver':f['receiver']} if f.get('receiver') else {})},
         'termination':'partial','requires':[],'ensures':[{'op':'bool.literal','value':True}],
         'error_inputs':entries,'loops':[],'modifies':[]}
    return source_overlay(old)


class Project:
    def __init__(self,spec):
        exact(spec,{'schema','roots','universe','overlays'});need(spec['schema']=='mpk.go.module_model_input.v0','private complete project input')
        self.spec_bytes=canonical(spec);self.spec=copy.deepcopy(spec);self.facts=json.loads(FACTS.read_bytes())
        need(type(spec['universe']) is list and all(type(x) is str for x in spec['universe']) and spec['universe']==sorted(set(spec['universe'])),'canonical explicit private error universe')
        self.universe=tuple(spec['universe']);self.templates={};self.selectors={};self.by_selector={}
        for f in self.facts['functions']:
            raw=default_overlay(f,self.facts,self.universe);s=Scope(raw,self.universe)
            self.templates[s.declaration]=s;self.selectors[s.declaration]=raw['function'];self.by_selector[canonical(raw['function'])]=s.declaration
        need(type(spec['roots']) is list and spec['roots'],'nonempty selected roots')
        resolved=[self.resolve(r) for r in spec['roots']]
        need(len(resolved)<=8192 and len(resolved)==len(set(resolved)) and spec['roots']==sorted(spec['roots'],key=canonical),'canonical selectors; unique alias-resolved roots')
        self.root_ids=sorted(resolved)
        need(type(spec['overlays']) is list and len(spec['overlays'])<=32768,'bounded raw overlay set')
        self.overlays={};objects=[];paths=[]
        for row in spec['overlays']:
            exact(row,{'path','raw_hex'});path(row['path']);need(type(row['raw_hex']) is str,'raw overlay bytes')
            try:raw=bytes.fromhex(row['raw_hex'])
            except ValueError as e:raise ValueError('raw overlay hex') from e
            need(len(raw)<=1048576,'per-overlay input size');value=parse(raw)
            need(type(value) is dict and type(value.get('function')) is dict,'source overlay selector')
            decl=self.resolve(value['function']);need(decl not in self.overlays,'one actual overlay per alias-resolved function')
            # Validate complete source fields without an old64-depth whole-overlay hash.
            overlay_shape(value)
            obj=identity_seal('source_object',{'capability':'overlays','path':row['path'],'kind':'overlay',
                 'raw_sha256':__import__('hashlib').sha256(raw).hexdigest(),'bytes':len(raw)})
            self.overlays[decl]={'raw':raw,'value':value,'object':obj,'path':row['path']};objects.append(obj);paths.append(row['path'])
        need(paths==sorted(set(paths)),'unique canonical overlay path order')
        first=next(iter(self.templates.values()));nodes=first.value['type_definitions']['identity_nodes']
        old=next(n for n in nodes if n['id']==first.capture)
        original_objects=[x for x in old['objects'] if x!=first.overlay_id]
        self.capture=identity_seal('capture_context',{k:v for k,v in old.items() if k not in {'schema','id'}}|
            {'objects':sorted(original_objects+[o['id'] for o in objects])})
        self.objects=sorted(objects,key=lambda o:o['id']);self.scopes={}
        # Declarations/signatures do not depend on capture. All loops do.
        self.function_nodes={};self.loop_nodes={}
        for t in self.templates.values():
            for n in t.graph:
                if n['schema'] in {'mpk.go.declaration.v1','mpk.go.signature.v1'}:self.function_nodes[n['id']]=copy.deepcopy(n)
                elif n['schema']=='mpk.go.loop_ref.v1':
                    new=identity_seal('loop_ref',{k:v for k,v in n.items() if k not in {'schema','id'}}|{'capture_context_id':self.capture['id']})
                    self.loop_nodes[new['id']]=new

    def resolve(self,selector):
        exact(selector,{'package','function','receiver'});value=copy.deepcopy(selector)
        need(value['package']=='fixture/policy','private native fixture package')
        if value['receiver'] is not None:
            exact(value['receiver'],{'package','type_name'});need(value['receiver']['package']==value['package'],'same receiver package')
            need(type(value['receiver']['type_name']) is str,'receiver identifier')
            name='fixture/policy.'+value['receiver']['type_name'];name=self.facts['aliases'].get(name,name)
            value['receiver']['type_name']=name.split('.')[-1]
        key=canonical(value);need(key in self.by_selector,'exact captured function/value receiver')
        return self.by_selector[key]

    def scope(self,decl):
        need(type(decl) is str and decl in self.templates,'known captured function declaration')
        if decl not in self.scopes:self.scopes[decl]=ProjectScope(self,decl)
        return self.scopes[decl]

    def fresh(self):return Project(parse(self.spec_bytes))

    def scope_set(self,declarations):
        need(declarations==sorted(set(declarations)) and set(self.root_ids)<=set(declarations),'complete ordered source scope set')
        scopes=[self.scope(d).value for d in declarations]
        return seal(SET_SCHEMA,SET_DOMAIN,{'semantic_context':scopes[0]['semantic_context'],'capture_context':self.capture,
            'overlay_objects':self.objects,'selected_root_ids':self.root_ids,'functions':scopes},'content_sha256')


class ProjectScope(Scope):
    def __init__(self,project,decl):
        seed=project.templates[decl];self.__dict__.update(copy.deepcopy(seed.__dict__));self.project=project
        old_capture,old_overlay=self.capture,self.overlay_id;self.capture=project.capture['id']
        overlay=project.overlays.get(decl);self.overlay_id=overlay['object']['id'] if overlay else None
        self.raw=copy.deepcopy(overlay['value'] if overlay else seed.raw)
        self.raw_bytes=overlay['raw'] if overlay else None;self.overlay_path=overlay['path'] if overlay else None
        self.origin={'kind':'source_overlay','object_id':self.overlay_id} if overlay else {'kind':'default'}
        defs=copy.deepcopy(seed.value['type_definitions'])
        nodes=[n for n in defs['identity_nodes'] if n['id'] not in {old_capture,old_overlay}]+project.objects+[project.capture]
        defs=seal(defs['schema'],TYPE_DOMAIN,{k:v for k,v in defs.items() if k not in {'schema','content_sha256'}}|{'identity_nodes':sorted(nodes,key=lambda n:n['id'])},'content_sha256')
        old_bindings=copy.deepcopy(self.bindings);replacement={};self.bindings={};self.native_bindings={}
        for native,b in old_bindings.items():
            new=seal(BINDING_SCHEMA,BINDING_DOMAIN,{k:v for k,v in b.items() if k not in {'schema','id'}}|{'capture_context_id':self.capture},'id')
            replacement[b['id']]=new['id'];self.bindings[native]=new;self.native_bindings[new['id']]=seed.native_bindings[b['id']]
        self.inputs=[replacement[b] for b in seed.inputs];self.results=[replacement[b] for b in seed.results]
        self.header_ids={ordinal:sorted(replacement[b] for b in ids) for ordinal,ids in seed.header_ids.items()}
        self.loop_ids={n['function_preorder_ordinal']:n['id'] for n in project.loop_nodes.values() if n['function_decl_id']==decl}
        graph={n['id']:n for n in nodes};graph.update(project.function_nodes);graph.update(project.loop_nodes)
        roots=set(seed.graph_roots)-{old_capture,old_overlay}-set(seed.loop_ids.values())|{self.capture}|set(project.function_nodes)|set(project.loop_nodes)
        self.graph=sorted(graph.values(),key=lambda n:n['id']);self.graph_roots=sorted(roots);validate_graph(self.graph,self.graph_roots)
        self.value=seal(SCOPE_SCHEMA,SCOPE_DOMAIN,{'semantic_context':self.context,'capture_context_id':self.capture,
            'source_overlay_object_id':self.overlay_id,'source_object_id':self.source_id,'function_decl_id':decl,'signature_id':self.signature,
            'error_universe_id':defs['error_universe_id'],'type_definitions':defs,'identity_nodes':self.graph,'identity_roots':self.graph_roots,
            'bindings':sorted(self.bindings.values(),key=lambda b:b['id']),'inputs':self.inputs,'results':self.results,
            'headers':[{'loop_ref_id':self.loop_ids[o],'bindings':self.header_ids[o]} for o in self.loop_ids]},'content_sha256')
        self._immutable=self.snapshot()

    def snapshot(self):
        fields=('raw','facts','function','context','ids','names','capture','overlay_id','source_id','declaration','signature','bindings','native_bindings','inputs','results','value')
        return canonical({k:getattr(self,k) for k in fields}|{'loop_ids':{str(k):v for k,v in self.loop_ids.items()},
            'header_ids':{str(k):v for k,v in self.header_ids.items()},'types':self.types.rows,'aliases':self.types.aliases,
            'universe':list(self.types.universe),'origin':self.origin,'raw_hex':None if self.raw_bytes is None else self.raw_bytes.hex(),'overlay_path':self.overlay_path})

    def assert_integrity(self):need(self.snapshot()==self._immutable,'immutable assembled source scope');return True


class ProjectModel(Signatures):
    def __init__(self,project,decl):
        self.project=project;self.scope=project.scope(decl);self.scope.assert_integrity();self.universe=project.universe
        self.defs=self.scope.value['type_definitions'];self.specializer=Specializer(self.defs,self.scope.context)
        self.rows=self.specializer.rows;self.nodes=self.specializer.nodes;self.ids=self.scope.ids
        self.table=json.loads(__import__('operation_signatures').TABLE.read_bytes());self.operations={r['id']:r for r in self.table['operations']}
        self.callees={n['declaration_id']:n for n in self.scope.graph if n['schema']=='mpk.go.signature.v1'}
        self.fn_results=[self.id(t['type']) for t in self.scope.function['results']]
        for b in self.scope.function['inputs']+self.scope.function['results']:self.root(self.id(b['type']))

    def derive(self,op,p):
        candidate=super().derive(op,p)
        return seal(OP_SCHEMA,OP_DOMAIN,{k:v for k,v in candidate.items() if k not in {'schema','operation_hash'}},'operation_hash')


def normalize_contract(scope):
    """Same closed70 expression semantics; new origin/null scope and full JSON limits."""
    scope.assert_integrity();raw=scope.raw
    need(all(type(raw[k]) is list for k in ('requires','ensures','loops','error_inputs','modifies')),'closed contract lists')
    need(len(raw['requires'])<=64 and 1<=len(raw['ensures'])<=64 and len(raw['loops'])<=1024,'contract counts')
    exprs=list(raw['requires'])+list(raw['ensures'])
    for loop in raw['loops']:
        exact(loop,{'selector','invariants','variant'});need(type(loop['invariants']) is list and 1<=len(loop['invariants'])<=64,'invariant count')
        exprs+=loop['invariants'];v=loop['variant'];need(type(v) is dict,'source variant object')
        if v.get('kind')=='candidate':exact(v,{'kind','expression'});exprs.append(v['expression'])
        else:exact(v,{'kind'});need(v['kind']=='absent','closed source variant')
    check_source_budget(exprs);old=scope.legacy_normalized();wire=WireExpressions(scope)
    requires=[wire.encode(e,context(scope,'requires')) for e in raw['requires']]
    ensures=[wire.encode(e,context(scope,'ensures')) for e in raw['ensures']]
    supplied={r['selector']['ordinal']:r for r in raw['loops']};loops=[]
    for row in old['loops']:
        o=row['loop_ref']['function_preorder_ordinal'];r=supplied.get(o);ctx=context(scope,'header',o)
        inv=[wire.encode(e,ctx,o) for e in r['invariants']] if r else [wire.encode({'op':'bool.literal','value':True},ctx,o)]
        variant=row['variant']
        if variant['kind']=='candidate':
            source_expr=r['variant']['expression'] if r and r['variant']['kind']=='candidate' else {'op':'nat.sub','args':[{'op':'range.length'},{'op':'range.index'}]}
            variant={'kind':'candidate','origin':variant['origin'],'expression':wire.encode(source_expr,ctx,o)}
        loops.append({'loop_ref_id':scope.loop_ids[o],'invariants':inv,'invariant_origin':row['invariant_origin'],'variant':variant})
    entries=[]
    for row in old['error_inputs']:
        t=scope.function['inputs'][row['input']]['type'];p=[]
        for step in row['path']:
            base=scope.types.base(t);owner=scope.ids[scope.types.resolve(t)]
            if step=='elements':p.append({'kind':'elements','owner_type_id':owner,'extent':'capacity' if base['kind']=='slice' else 'length'});t=base['element']
            else:
                ordinal=int(step.split(':')[1]);p.append({'kind':'field','owner_type_id':owner,'ordinal':ordinal});t=base['fields'][ordinal]['type']
        entries.append({'input_binding_id':scope.inputs[row['input']],'path':p,'case_type_ids':sorted(scope.ids[t] for t in row['cases'])})
    return seal(CONTRACT_SCHEMA,CONTRACT_DOMAIN,{'semantic_context':scope.context,'source_scope_sha256':scope.value['content_sha256'],
        'source_overlay_object_id':scope.overlay_id,'origin':scope.origin,'capture_context_id':scope.capture,'function_decl_id':scope.declaration,
        'signature_id':scope.signature,'termination':raw['termination'],'requires':requires,'ensures':ensures,'error_inputs':entries,'modifies':[],
        'loops':loops,'entry_storage':old['entry_storage'],'pending_candidates':[{'loop_ref_id':scope.loop_ids[p['ordinal']],'reason':p['reason']} for p in old['pending_candidates']]},'contract_hash')
