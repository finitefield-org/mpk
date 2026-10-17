"""Untrusted ordinary equality composition and Nat monoid normalization."""
from dataclasses import dataclass
from carrier_terms import const,var,app,pi,lam,bind,nat,boolean,bit,eq,refl
from proof_canonical import canonical

MODULE='Probe.GoPractical.Arithmetic'
N=MODULE+'.Nat.'
I=MODULE+'.Int.'


def call(name,*args):return app(const(name),*args)
def nc(name,*args):return call(N+name,*args)
def ic(name,*args):return call(I+name,*args)
def zero():return const('Std.Nat.zero')
def succ(n):return call('Std.Nat.succ',n)
def one():return succ(zero())
def add(a,b):return nc('Add',a,b)
def mul(a,b):return nc('Mul',a,b)
def induct(name,property_,base,step,value):return call('Std.Nat.induct',lam(name,nat(),property_),base,step,value)


@dataclass
class Equality:
    left:dict
    right:dict
    proof:dict
    ty:dict=None
    def __post_init__(self):
        if self.ty is None:self.ty=nat()
    def reverse(self):return Equality(self.right,self.left,call('Std.Eq.symm',self.ty,self.left,self.right,self.proof),self.ty)
    def then(self,other):
        assert self.ty==other.ty and self.right==other.left,'explicit equality endpoints'
        return Equality(self.left,other.right,call('Std.Eq.trans',self.ty,self.left,self.right,other.right,self.proof,other.proof),self.ty)
    def mapped(self,f,left,right,result_type=None):
        target=self.ty if result_type is None else result_type
        return Equality(left,right,call('Std.Eq.congr',self.ty,target,f,self.left,self.right,self.proof),target)


def reflexive(value,ty=None):return Equality(value,value,refl(nat() if ty is None else ty,value),ty)
def lemma(name,args,left,right):return Equality(left,right,nc(name,*args))
def on_left(e,b):return e.mapped(lam('_mapped',nat(),add(var('_mapped'),b)),add(e.left,b),add(e.right,b))
def on_right(a,e):return e.mapped(lam('_mapped',nat(),add(a,var('_mapped'))),add(a,e.left),add(a,e.right))
def on_succ(e):return e.mapped(const('Std.Nat.succ'),succ(e.left),succ(e.right))
def assoc(a,b,c):return lemma('AddAssoc',[a,b,c],add(add(a,b),c),add(a,add(b,c)))
def comm(a,b):return lemma('AddComm',[a,b],add(a,b),add(b,a))
def add_zero(a):return lemma('AddZero',[a],add(a,zero()),a)
def zero_add(a):return lemma('ZeroAdd',[a],add(zero(),a),a)
def add_succ(a,b):return lemma('AddSucc',[a,b],add(a,succ(b)),succ(add(a,b)))
def succ_add(a,b):return lemma('SuccAdd',[a,b],add(succ(a),b),succ(add(a,b)))
def split(node,name):
    return node['arguments'] if node.get('kind')=='app' and node['function']==const(N+name) and len(node['arguments'])==2 else None


class AddNormalizer:
    """Every reassociation/swap/identity is an application of a checked law."""
    def join(self,a,b):
        if a==zero():return zero_add(b)
        if b==zero():return add_zero(a)
        parts=split(a,'Add')
        if parts is None:return reflexive(add(a,b))
        x,y=parts
        return assoc(x,y,b).then(on_right(x,self.join(y,b)))
    def associate(self,node):
        parts=split(node,'Add')
        if parts is None:return reflexive(node)
        a,b=parts;x=self.associate(a);y=self.associate(b)
        return on_left(x,b).then(on_right(x.right,y)).then(self.join(x.right,y.right))
    def atoms(self,node):
        if node==zero():return []
        parts=split(node,'Add')
        return [node] if parts is None else [parts[0]]+self.atoms(parts[1])
    def value(self,atoms):
        if not atoms:return zero()
        return atoms[0] if len(atoms)==1 else add(atoms[0],self.value(atoms[1:]))
    def swap(self,atoms,index):
        assert 0<=index<len(atoms)-1
        if index:return on_right(atoms[0],self.swap(atoms[1:],index-1))
        a,b=atoms[:2]
        if len(atoms)==2:return comm(a,b)
        tail=self.value(atoms[2:])
        return assoc(a,b,tail).reverse().then(on_left(comm(a,b),tail)).then(assoc(b,a,tail))
    def normalize(self,node):
        e=self.associate(node);atoms=self.atoms(e.right)
        for end in range(len(atoms)-1,0,-1):
            for index in range(end):
                if canonical(atoms[index])>canonical(atoms[index+1]):
                    e=e.then(self.swap(atoms,index));atoms[index],atoms[index+1]=atoms[index+1],atoms[index]
        return e
    def prove(self,left,right):
        a=self.normalize(left);b=self.normalize(right)
        assert a.right==b.right,'same Nat additive polynomial'
        return a.then(b.reverse())


def mul_on_left(e,b):return e.mapped(lam('_mapped',nat(),mul(var('_mapped'),b)),mul(e.left,b),mul(e.right,b))
def mul_on_right(a,e):return e.mapped(lam('_mapped',nat(),mul(a,var('_mapped'))),mul(a,e.left),mul(a,e.right))
def mul_assoc(a,b,c):return lemma('MulAssoc',[a,b,c],mul(mul(a,b),c),mul(a,mul(b,c)))
def mul_comm(a,b):return lemma('MulComm',[a,b],mul(a,b),mul(b,a))


class ProductNormalizer(AddNormalizer):
    def join(self,a,b):
        if a==zero():return lemma('ZeroMul',[b],mul(a,b),zero())
        if b==zero():return lemma('MulZero',[a],mul(a,b),zero())
        if a==one():return lemma('OneMul',[b],mul(a,b),b)
        if b==one():return lemma('MulOne',[a],mul(a,b),a)
        parts=split(a,'Mul')
        if parts is None:return reflexive(mul(a,b))
        x,y=parts;return mul_assoc(x,y,b).then(mul_on_right(x,self.join(y,b)))
    def associate(self,node):
        parts=split(node,'Mul')
        if parts is None:return reflexive(node)
        a,b=parts;x=self.associate(a);y=self.associate(b)
        return mul_on_left(x,b).then(mul_on_right(x.right,y)).then(self.join(x.right,y.right))
    def atoms(self,node):
        if node==one():return []
        parts=split(node,'Mul');return [node] if parts is None else [parts[0]]+self.atoms(parts[1])
    def value(self,atoms):return one() if not atoms else atoms[0] if len(atoms)==1 else mul(atoms[0],self.value(atoms[1:]))
    def swap(self,atoms,index):
        if index:return mul_on_right(atoms[0],self.swap(atoms[1:],index-1))
        a,b=atoms[:2]
        if len(atoms)==2:return mul_comm(a,b)
        tail=self.value(atoms[2:]);return mul_assoc(a,b,tail).reverse().then(mul_on_left(mul_comm(a,b),tail)).then(mul_assoc(b,a,tail))


class PolynomialNormalizer:
    def __init__(self,save=lambda e:e):self.save=save
    def normalize(self,node):
        a=split(node,'Add');m=split(node,'Mul')
        if a:
            x,y=a;p=self.normalize(x);q=self.normalize(y)
            e=on_left(p,y).then(on_right(p.right,q));return self.save(e.then(self.sum_normalize(e.right)))
        if m:
            x,y=m;p=self.normalize(x);q=self.normalize(y)
            e=mul_on_left(p,y).then(mul_on_right(p.right,q));return self.save(e.then(self.distribute(p.right,q.right)))
        return reflexive(node)
    def sum_normalize(self,node):
        normalizer=AddNormalizer();e=normalizer.associate(node);atoms=normalizer.atoms(e.right)
        for end in range(len(atoms)-1,0,-1):
            for index in range(end):
                if canonical(atoms[index])>canonical(atoms[index+1]):
                    e=self.save(e.then(normalizer.swap(atoms,index)));atoms[index],atoms[index+1]=atoms[index+1],atoms[index]
        return e
    def distribute(self,a,b):
        aa=split(a,'Add');bb=split(b,'Add')
        if aa:
            x,y=aa;e=lemma('MulAddLeft',[x,y,b],mul(a,b),add(mul(x,b),mul(y,b)))
            p=self.distribute(x,b);q=self.distribute(y,b)
            e=e.then(on_left(p,mul(y,b))).then(on_right(p.right,q));return self.save(e.then(self.sum_normalize(e.right)))
        if bb:
            x,y=bb;e=lemma('MulAddRight',[a,x,y],mul(a,b),add(mul(a,x),mul(a,y)))
            p=self.distribute(a,x);q=self.distribute(a,y)
            e=e.then(on_left(p,mul(a,y))).then(on_right(p.right,q));return self.save(e.then(self.sum_normalize(e.right)))
        return ProductNormalizer().normalize(mul(a,b))
    def prove(self,left,right):
        a=self.normalize(left);b=self.normalize(right);assert a.right==b.right,'same Nat polynomial'
        return self.save(a.then(b.reverse()))
