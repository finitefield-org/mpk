"""Source-typed value/definedness specializations of the ordinary word library.

These are concrete ordinary definitions, not the source/evaluation/frame
theorems requested by the complete function VC. Nominal IDs are still checked
by the captured operation model before selecting a width-specific circuit.
"""
from word_foundation import MODULE,WIDTHS,Library,boolean_library,boolean_theorems,word_library,word_theorems,word,boolean,bit,var,const,call,literal
from shared_project import Project,ProjectModel
from shared_vir_vectors import project_spec
from operation_signatures import UNARY,ARITH,COMPARE,need,exact
from proof_canonical import canonical,clone

PURE_WORD_OPS={1,2,3,4,5,6,7,9}


class Bindings:
    def __init__(self,spec,decl):
        self.project=Project(clone(spec));self.model=ProjectModel(self.project,decl)
    def compile(self,operation):
        m=self.model;p=operation['parameters'];op=operation['op'];n=int(op[-2:])
        need(n in PURE_WORD_OPS,'word value/definedness component only; no implicit source/frame theorem')
        expected=m.derive(op,p);need(canonical(operation)==canonical(expected),'complete source/context/signature/operation reconstruction')
        source_ids=[r['source_type_id'] for r in operation['inputs']]
        input_rows=[m.integer(t) for t in source_ids]
        a=[var('argument'+str(i)) for i in range(len(input_rows))]
        args=[('argument'+str(i),word(r['width'])[0]) for i,r in enumerate(input_rows)]
        t=m.integer(p['from'] if n==3 else p['type']);width=t['width'];prefix=MODULE+'.W'+str(width)+'.'
        use=lambda name,*xs:call(prefix+name,*xs)
        condition=bit(True)
        if n==1:value=const(prefix+'Zero')
        elif n==2:value=literal(width,int(p['value']))
        elif n==3:
            target=m.integer(p['to'])['width'];value=use(('Signed' if t['signed'] else 'Unsigned')+'To'+str(target),a[0])
        elif n==4:value=a[0] if p['operator']=='plus' else use({'negate':'Neg','complement':'Not'}[p['operator']],a[0])
        elif n==5:value=use({'add':'Add','subtract':'Sub','multiply':'Mul','and':'And','or':'Or','xor':'Xor','and_not':'AndNot'}[p['operator']],*a)
        elif n==6:
            name=('Signed' if t['signed'] else 'Unsigned')+('Div' if p['operator']=='divide' else 'Rem')
            value=use(name,*a);condition=use('DivideDefined',a[1])
        elif n==7:
            count=input_rows[1];converted=call(MODULE+'.W'+str(count['width'])+'.'+('Signed' if count['signed'] else 'Unsigned')+'To64',a[1])
            name='ShiftLeft' if p['operator']=='left' else 'SignedShiftRight' if t['signed'] else 'ShiftRight'
            value=use(name,a[0],converted)
            if count['signed']:condition=use('SignedCountDefined',converted)
        else:
            sub=p['operator'];values=a if sub not in {'greater','greater_equal'} else list(reversed(a))
            name={'equal':'Equal','not_equal':'NotEqual','less':'Less','less_equal':'LessEqual','greater':'Less','greater_equal':'LessEqual'}[sub]
            if name not in {'Equal','NotEqual'}:name=('Signed' if t['signed'] else 'Unsigned')+name
            value=use(name,*values)
        output=operation['outputs'][0]['source_type_id'];r=m.root(output)
        result=boolean() if r['kind']=='bool' else word(r['width'])[0]
        prefix=MODULE+'.Binding.H'+operation['operation_hash']+'.'
        library=Library();library.emit(prefix+'Value',args,result,value);library.emit(prefix+'Defined',args,boolean(),condition)
        return {'schema':'mpk.go_practical.word_value_binding_draft.v0','semantic_context':operation['semantic_context'],
            'source_scope_sha256':operation['source_scope_sha256'],'operation_hash':operation['operation_hash'],
            'source_input_types':source_ids,'source_output_type':output,'value_name':prefix+'Value','definedness_name':prefix+'Defined',
            'declarations':library.rows,'source_evaluation_proved':False,'frame_permission_proved':False}
    def validate(self,operation,value):
        need(canonical(value)==canonical(self.compile(operation)),'complete ordinary value/definedness reconstruction; no caller formula or proof flag')


def fixture():
    spec=project_spec(False);p=Project(spec);decl=p.resolve({'package':'fixture/policy','function':'Scopes','receiver':None})
    b=Bindings(spec,decl);m=b.model
    ids=sorted({sid for sid in m.ids.values() if m.root(sid)['kind']=='word'});operations=[]
    def add(n,parameters):operations.append(m.derive('GP-OP-'+str(n).zfill(2),parameters))
    for sid in ids:
        row=m.integer(sid);width=row['width'];signed=row['signed']
        add(1,{'type':sid})
        for value in sorted({0,-(1<<(width-1)) if signed else 0,(1<<(width-(1 if signed else 0)))-1}):add(2,{'type':sid,'value':str(value)})
        for target in ids:add(3,{'from':sid,'to':target})
        for op in sorted(UNARY):add(4,{'type':sid,'operator':op})
        for op in sorted(ARITH):add(5,{'type':sid,'operator':op})
        for op in ('divide','remainder'):add(6,{'type':sid,'operator':op})
        for count in ids:
            for op in ('left','right'):add(7,{'type':sid,'count_type':count,'operator':op})
        for op in sorted(COMPARE):add(9,{'type':sid,'operator':op})
    operations.sort(key=lambda o:o['operation_hash'])
    values=[b.compile(o) for o in operations]
    return {'project_input':spec,'function_decl_id':decl,'source_word_types':ids,'operations':operations,'bindings':values}


def combined_plan(data):
    lib=Library();boolean_library(lib);boolean_theorems(lib)
    for width in WIDTHS:
        prefix=word_library(lib,width);word_theorems(lib,width,prefix)
    for binding in data['bindings']:
        for d in binding['declarations']:
            need(d['name'] not in lib.names,'unique source operation specialization');lib.names.add(d['name']);lib.rows.append(d)
    return {'module':MODULE+'.SourceWordValues','declarations':lib.rows}
