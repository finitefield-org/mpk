"""Untrusted constructor for typed structured-body candidates and components."""
import copy
from vir_body import BodyValidator, body_record, STATE, source


class Builder:
    def __init__(self,model):
        self.m=model;self.next_value=0;self.next_node=0;self.next_region=0;self.defs={}

    def value(self):
        result=self.next_value;self.next_value+=1;return result

    def region(self,types,stack=(),captures=()):
        r={'id':self.next_region,'parameters':[{'id':self.value(),'type':copy.deepcopy(t)} for t in types],
           'captures':sorted(captures),'nodes':[],'terminator':None}
        self.next_region+=1
        return Region(self,r,tuple(stack))

    def definitions(self):return sorted(self.defs.values(),key=lambda d:d['operation_hash'])

    def function(self,make):
        ins=[source(self.m.id(b['type'])) for b in self.m.scope.function['inputs']]+[STATE]
        region=self.region(ins);make(region)
        return body_record(self.m,region.r,self.definitions())


class Region:
    def __init__(self,b,r,stack):
        self.b=b;self.r=r;self.stack=stack
        states=[p['id'] for p in r['parameters'] if p['type']['kind']=='state'];self.state=states[0] if states else None

    def anchor(self,loop=None,switch=False):
        if loop is not None:
            l=next(l for l in self.b.m.scope.function['loops'] if l['ordinal']==loop)
            return {'source_object_id':self.b.m.scope.source_id,'start_byte':l['start_byte'],'end_byte':l['end_byte'],'ordinal':loop}
        if self.stack:ref={k:v for k,v in self.stack[-1].items() if k!='kind'}
        else:
            f=self.b.m.scope.function;ref={'source_object_id':self.b.m.scope.source_id,'start_byte':f['start_byte'],'end_byte':f['end_byte'],'ordinal':0}
        if switch and self.stack:ref={**ref,'start_byte':ref['start_byte']+1,'end_byte':ref['end_byte']-1,'ordinal':self.b.next_node}
        return ref

    def node(self,kind,**fields):
        node={'kind':kind,'id':self.b.next_node,**fields};self.b.next_node+=1
        self.r['nodes'].append(node);return node

    def apply(self,n,parameters,inputs,children=None):
        definition=self.b.m.derive('GP-OP-'+str(n).zfill(2),parameters);self.b.defs[definition['operation_hash']]=definition
        outputs=[self.b.value() for _ in definition['outputs']]
        ref=self.anchor(parameters['loop'] if n in {51,52,53} else None,n==16)
        node=self.node('apply',definition_hash=definition['operation_hash'],inputs=list(inputs),outputs=outputs,regions=[],source_ref=ref)
        stack=self.stack
        if n in {16,51,52,53}:stack=stack+({'kind':'switch' if n==16 else 'loop',**ref},)
        for spec in definition['regions']:
            child=self.b.region(spec['inputs'],stack)
            if children is None:child.default(spec['outputs'])
            else:children(spec['role'],child,spec['outputs'])
            node['regions'].append({'role':spec['role'],'region':child.r})
        for ident,t in zip(outputs,definition['outputs']):
            if t['kind']=='state':self.state=ident
            elif t['kind']=='control':self.state=None
        return outputs

    def zero(self,t):return self.apply(1,{'type':t},[])[0]

    def address(self,binding,path=()):
        ident=self.b.value();self.node('address',root={'kind':'binding','binding_id':binding},path=list(path),state=self.state,output=ident,source_ref=self.anchor());return ident

    def slice_address(self,descriptor,index,path=()):
        ident=self.b.value();self.node('address',root={'kind':'slice','descriptor':descriptor,'index':index},path=list(path),state=self.state,output=ident,source_ref=self.anchor());return ident

    def load(self,loc):
        ident=self.b.value();self.node('load',location=loc,state=self.state,output=ident,source_ref=self.anchor());return ident

    def project(self,product,index,post_state=None):
        ident=self.b.value();self.node('project',product=product,index=index,post_state=post_state,output=ident,source_ref=self.anchor());return ident

    def values(self,values):
        ident=self.b.value();self.node('values',inputs=list(values),output=ident,source_ref=self.anchor());return ident

    def declare(self,binding,value):
        ident=self.b.value();self.node('declare',binding_id=binding,value=value,state=self.state,output=ident,source_ref=self.anchor());self.state=ident;return ident

    def scope(self,make,captures=()):
        ident=self.b.value();node=self.node('scope',state=self.state,output=ident,region=None,source_ref=self.anchor())
        child=self.b.region([STATE],self.stack,captures);make(child);node['region']=child.r;self.state=None;return ident

    def yield_(self,values):self.r['terminator']={'kind':'yield','values':list(values)}
    def normal(self):self.r['terminator']={'kind':'normal','state':self.state}

    def return_zero(self):
        types=self.b.m.fn_results;values=[self.zero(t) for t in types]
        product=self.apply(17,{'types':types},values)[0]
        control=self.apply(20,{},[product,self.state])[0];self.yield_([control])

    def resume(self,control,make,captures=()):
        continuation=self.b.region([STATE],self.stack,captures);make(continuation)
        self.r['terminator']={'kind':'resume','control':control,'continuation':continuation.r}

    def default(self,types):
        if types==[{'kind':'control','result_types':self.b.m.fn_results}]:self.normal();return
        values=[]
        for t in types:
            if t['kind']=='state':values.append(self.state)
            else:
                assert t['kind']=='source';values.append(self.zero(t['source_type_id']))
        self.yield_(values)


def component(model,n,parameters,stack=()):
    b=Builder(model)
    if n==18:
        call=model.derive('GP-OP-19',{'callee':parameters['callee']});r=b.region(call['inputs'],stack)
        product,state=r.apply(19,{'callee':parameters['callee']},[p['id'] for p in r.r['parameters']])
        result=r.apply(18,parameters,[product,state]);r.yield_(result);inputs=call['inputs']
    else:
        d=model.derive('GP-OP-'+str(n).zfill(2),parameters);inputs=d['inputs'];r=b.region(inputs,stack)
        results=r.apply(n,parameters,[p['id'] for p in r.r['parameters']]);r.yield_(results)
    d=model.derive('GP-OP-'+str(n).zfill(2),parameters);defs=b.definitions()
    v=BodyValidator(model,defs);result=v.region(r.r,inputs,d['outputs'],stack=tuple(stack))
    assert v.used_defs==set(v.defs)
    return {'op':d['op'],'inputs':inputs,'outputs':d['outputs'],'entry_control_stack':list(stack),
            'definitions':defs,'region':r.r,'obligation_sites':sorted(v.sites,key=lambda s:s['node_id']),
            'control_outcomes':sorted(result.controls),'scope':'private operation component query,not a function or source binding proof'}
