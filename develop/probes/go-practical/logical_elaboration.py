"""Closed logical component of the existing typed contract wire vocabulary.

This constructs values and unproved statements. It cannot authorize a source
binding,turn a false Boolean into a proof,or assemble a complete Go VC.
"""
import hashlib
import re
from arithmetic_terms import *
from ordinary_integers import integer
from ordinary_ranges import rc
from word_foundation import Library,choose,bc,Evaluator,Primitive
from contract_wire import NORMAL_OPS
from go_obligations import OrdinaryElaborationPending
from proof_canonical import canonical,clone

TYPES={'Bool':boolean,'Nat':nat,'Int':integer}
HEADS={'bool.literal','nat.literal','int.literal','bound','bool.not','bool.and','bool.or','bool.implies','nat.to_int','int.neg',
    'nat.add','nat.sub','nat.mul','int.add','int.sub','int.mul','eq','ne','lt','le','gt','ge','if','forall','exists','nat.sum','int.sum'}


def exact(value,fields):
    if type(value) is not dict or set(value)!=set(fields):raise ValueError('closed logical component fields')
def require(condition,reason):
    if not condition:raise ValueError(reason)
def type_ref(name):return {'kind':'logical','name':name}


class LogicalCompiler:
    def __init__(self,budget=4096):self.library=Library();self.literal_names={};self.budget=budget
    def literal(self,number):
        require(type(number) is int and number>=0,'ordinary Nat literal')
        if not number:return zero()
        value=zero();prefix=0
        for digit in bin(number)[2:]:
            prefix=2*prefix+int(digit)
            if prefix not in self.literal_names:
                if len(self.library.rows)>=self.budget:raise OrdinaryElaborationPending('private proof declaration construction budget; literal remains a valid mathematical value')
                name=MODULE+'.Literal.N'+hashlib.sha256(str(prefix).encode()).hexdigest()
                body=add(value,value);body=succ(body) if digit=='1' else body
                self.library.emit(name,[],nat(),body);self.literal_names[prefix]=name
            value=const(self.literal_names[prefix])
        return value
    def compile(self,node,scope=()):
        exact(node,('op','type','payload'));op=node['op'];p=node['payload']
        require(type(op) is str and op in NORMAL_OPS,'known contract expression head')
        if op not in HEADS:raise OrdinaryElaborationPending('source/word/storage expression requires the remaining registered elaborator: '+op)
        exact(node['type'],('kind','name'));require(node['type']['kind']=='logical' and node['type']['name'] in TYPES,'logical type arm')
        def visit(n):return self.compile(n,scope)
        if op in {'bool.literal','nat.literal','int.literal'}:
            exact(p,('value',));raw=p['value']
            if op=='bool.literal':require(type(raw) is bool,'exact Boolean literal');value=bit(raw);ty='Bool'
            else:
                require(type(raw) is str and re.fullmatch(r'0|-?[1-9][0-9]*' if op=='int.literal' else r'0|[1-9][0-9]*',raw) is not None,'canonical decimal literal')
                if len(raw)>4096:raise OrdinaryElaborationPending('private decimal construction budget,not a semantic integer bound')
                number=int(raw);magnitude=self.literal(abs(number));ty='Nat' if op=='nat.literal' else 'Int'
                value=magnitude if ty=='Nat' else ic('Make',zero(),magnitude) if number<0 else ic('FromNat',magnitude)
        elif op=='bound':
            exact(p,('depth',));depth=p['depth'];require(type(depth) is int and 0<=depth<len(scope),'lexical Nat binder depth')
            value=var(scope[-1-depth]);ty='Nat'
        elif op in {'forall','exists','nat.sum','int.sum'}:
            exact(p,('lower','upper','body'));lo,lot=visit(p['lower']);hi,hit=visit(p['upper']);require(lot==hit=='Nat','finite Nat interval')
            name='_logical_bound'+str(len(scope));body,bt=self.compile(p['body'],scope+(name,));ty='Nat' if op=='nat.sum' else 'Int' if op=='int.sum' else 'Bool'
            require(bt==ty,'logical range body type');tag={'forall':'Forall','exists':'Exists','nat.sum':'NatSum','int.sum':'IntSum'}[op]
            value=rc(tag,lo,hi,lam(name,nat(),body))
        else:
            exact(p,('args',));require(type(p['args']) is list,'ordered logical arguments');args=[visit(n) for n in p['args']]
            values=[v for v,_ in args];types=[t for _,t in args]
            if op=='if':
                require(len(args)==3 and types[0]=='Bool' and types[1]==types[2],'conditional branch type');ty=types[1]
                value=choose(values[0],values[1],values[2],TYPES[ty]())
            elif op in {'bool.not','int.neg','nat.to_int'}:
                expected='Bool' if op=='bool.not' else 'Int' if op=='int.neg' else 'Nat'
                require(types==[expected],'logical unary operand');ty='Bool' if op=='bool.not' else 'Int'
                value=bc('Not',*values) if ty=='Bool' else ic('Negate' if op=='int.neg' else 'FromNat',*values)
            elif op in {'bool.and','bool.or','bool.implies'}:
                require(types==['Bool','Bool'],'logical Boolean operands');ty='Bool'
                value=bc('And' if op=='bool.and' else 'Or',*values) if op!='bool.implies' else bc('Or',bc('Not',values[0]),values[1])
            elif op.startswith(('nat.','int.')):
                ty='Nat' if op.startswith('nat.') else 'Int';require(types==[ty,ty],'exact logical arithmetic operands')
                value=(nc if ty=='Nat' else ic)({'add':'Add','sub':'Sub','mul':'Mul'}[op.split('.')[1]],*values)
            else:
                require(op in {'eq','ne','lt','le','gt','ge'} and len(types)==2 and types[0]==types[1],'exact logical comparison operands');ty='Bool'
                if types[0]=='Bool':require(op in {'eq','ne'},'no Boolean order');value=bc('Equal',*values)
                else:
                    tag='Equal' if op in {'eq','ne'} else 'Less' if op in {'lt','gt'} else 'LessEqual'
                    value=(nc if types[0]=='Nat' else ic)(tag,*(list(reversed(values)) if op in {'gt','ge'} else values))
                if op=='ne':value=bc('Not',value)
        require(node['type']==type_ref(ty),'independently inferred logical result type')
        return value,ty
    def program(self,node):
        value,ty=self.compile(clone(node));name=MODULE+'.Closed.H'+hashlib.sha256(canonical(node)).hexdigest()
        if len(self.library.rows)+2>self.budget:raise OrdinaryElaborationPending('private component declaration budget; mathematical expression remains unproved')
        self.library.emit(name+'.Value',[],TYPES[ty](),value);self.library.emit(name+'.Defined',[],boolean(),bit(True))
        return {'schema':'mpk.go_practical.closed_logical_elaboration.v0','expression':clone(node),'type':type_ref(ty),'value_name':name+'.Value','definedness_name':name+'.Defined',
            'declarations':self.library.rows,'statement':eq(boolean(),const(name+'.Value'),bit(True)) if ty=='Bool' else None,
            'ordinary_proof':None,'source_evaluation_proved':False,'complete_go_elaboration':False}


def elaborate(node,budget=4096):return LogicalCompiler(budget).program(node)
def validate(node,value):require(canonical(elaborate(node))==canonical(value),'complete closed logical value/statement reconstruction; no caller proof or source status')


class ModelEvaluationPending(RuntimeError):pass


class NatPrimitive:
    def __init__(self,induction=False,args=()):self.induction=induction;self.args=args
    def __call__(self,arg):
        args=self.args+(arg,);arity=4 if self.induction else 3
        if len(args)<arity:return NatPrimitive(self.induction,args)
        base,step,n=args[1:] if self.induction else args
        require(type(n) is int and n>=0,'well-typed genuine Nat observation')
        if n>512:raise ModelEvaluationPending('private unary-AST observation ceiling; no semantic bound or proof result')
        for i in range(n):base=step(i)(base)
        return base


class ArithmeticEvaluator(Evaluator):
    def __init__(self,plan):
        super().__init__(plan);self.constants.update({'Std.Nat':object(),'Std.Nat.zero':0,'Std.Nat.succ':lambda n:n+1,
            'Std.Nat.rec':NatPrimitive(),'Std.Nat.induct':NatPrimitive(True)})
    def observe(self,value,ty):return value if ty!='Int' else value(False)-value(True)
