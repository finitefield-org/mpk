"""Unbounded finite-range logical folds with exact half-open interval rules."""
from arithmetic_terms import *
from ordinary_integers import integer,equiv
from word_foundation import bc,EvaluationProof

R=MODULE+'.Range.'
def rc(name,*args):return call(R+name,*args)


class Ranges:
    def __init__(self,lib):self.lib=lib
    def boolean_observations(self):
        prefix=MODULE+'.Bool.'
        def split(names,environment,proposition,leaf):
            if not names:return leaf(environment)
            name=names[0];rest=names[1:];env={**environment,name:var(name),**{n:var(n) for n in rest}}
            motive=lam(name,boolean(),bind('pi',[(n,boolean()) for n in rest],proposition(env)))
            return lam(name,boolean(),call('Std.Bool.cases',motive,split(rest,{**environment,name:bit(False)},proposition,leaf),split(rest,{**environment,name:bit(True)},proposition,leaf),var(name)))
        for name,selected in (('AndLeft','a'),('AndRight','b'),('EqualReflect',None)):
            def proposition(e):
                premise=eq(boolean(),bc('Equal' if selected is None else 'And',e['a'],e['b']),bit(True))
                return pi('_h',premise,eq(boolean(),e['a'],e['b']) if selected is None else eq(boolean(),e[selected],bit(True)))
            def leaf(e):
                a,b=e['a'],e['b'];premise=eq(boolean(),bc('Equal' if selected is None else 'And',a,b),bit(True))
                if selected is not None:proof=refl(boolean(),bit(True)) if e[selected]==bit(True) else var('_h')
                elif a==b:proof=refl(boolean(),a)
                else:
                    actual=bc('Equal',a,b);normal=EvaluationProof(self.lib.rows).equation(actual,bit(False))
                    evidence=Equality(actual,bit(False),normal,boolean()).reverse().then(Equality(actual,bit(True),var('_h'),boolean()))
                    proof=evidence.proof if a==bit(False) else evidence.reverse().proof
                return lam('_h',premise,proof)
            env={'a':var('a'),'b':var('b')};ty=bind('pi',[('a',boolean()),('b',boolean())],proposition(env))
            self.lib.emit(prefix+name,[],ty,split(['a','b'],{},proposition,leaf),'theorem')
    def build(self):
        self.boolean_observations()
        lower,count,upper=var('lower'),var('count'),var('upper');f=var('body');index,acc=var('_index'),var('_acc')
        for name,ty,initial in (('Forall',boolean(),bit(True)),('Exists',boolean(),bit(False)),('NatSum',nat(),zero()),('IntSum',integer(),ic('Zero'))):
            fn=pi('_argument',nat(),ty);args=[('lower',nat()),('count',nat()),('body',fn)]
            def combine(a,b):
                return bc('And' if name=='Forall' else 'Or',a,b) if name in {'Forall','Exists'} else add(a,b) if name=='NatSum' else ic('Add',a,b)
            step=lam('_index',nat(),lam('_acc',ty,combine(acc,app(f,add(lower,index)))))
            body=induct('_index',ty,initial,step,count)
            self.lib.emit(R+'Fold'+name,args,ty,body)
            self.lib.emit(R+name,[('lower',nat()),('upper',nat()),('body',fn)],ty,rc('Fold'+name,lower,nc('Sub',upper,lower),f))
            folded=rc('Fold'+name,lower,count,f);next_=rc('Fold'+name,lower,succ(count),f);value=combine(folded,app(f,add(lower,count)))
            self.lib.emit(R+name+'Zero',[('lower',nat()),('body',fn)],eq(ty,rc('Fold'+name,lower,zero(),f),initial),refl(ty,initial),'theorem')
            self.lib.emit(R+name+'Step',args,eq(ty,next_,value),refl(ty,value),'theorem')
            sub=nc('Sub',lower,lower)
            p=Equality(sub,zero(),nc('SubSame',lower)).mapped(lam('_count',nat(),rc('Fold'+name,lower,var('_count'),f)),rc('Fold'+name,lower,sub,f),rc('Fold'+name,lower,zero(),f),ty)
            q=Equality(rc('Fold'+name,lower,zero(),f),initial,rc(name+'Zero',lower,f),ty)
            self.lib.emit(R+name+'SameBounds',[('lower',nat()),('body',fn)],eq(ty,rc(name,lower,lower,f),initial),p.then(q).proof,'theorem')
            empty=eq(boolean(),nc('Less',lower,upper),bit(False));length=nc('Sub',upper,lower)
            p=Equality(length,zero(),nc('EmptyLength',lower,upper,var('_empty'))).mapped(lam('_count',nat(),rc('Fold'+name,lower,var('_count'),f)),rc('Fold'+name,lower,length,f),rc('Fold'+name,lower,zero(),f),ty)
            self.lib.emit(R+name+'Empty',[('lower',nat()),('upper',nat()),('body',fn),('_empty',empty)],eq(ty,rc(name,lower,upper,f),initial),p.then(q).proof,'theorem')
            # Pointwise representative/equality independence for every prefix.
            g=var('other');point=equiv(app(f,index),app(g,index)) if name=='IntSum' else eq(ty,app(f,index),app(g,index))
            point_type=pi('_index',nat(),point);h=var('_pointwise');ih=var('_ih')
            left=rc('Fold'+name,lower,count,f);right=rc('Fold'+name,lower,count,g)
            relation=lambda a,b:equiv(a,b) if name=='IntSum' else eq(ty,a,b)
            at_n=lambda source:rc('Fold'+name,lower,index,source)
            prop=relation(at_n(f),at_n(g));position=add(lower,index);cf,cg=app(f,position),app(g,position)
            if name=='IntSum':
                base=ic('Reflexive',initial);step_proof=ic('AddRespects',at_n(f),cf,at_n(g),cg,ih,app(h,position))
            else:
                base=refl(ty,initial);previous=Equality(at_n(f),at_n(g),ih,ty);current=Equality(cf,cg,app(h,position),ty)
                first=previous.mapped(lam('_left',ty,combine(var('_left'),cf)),combine(at_n(f),cf),combine(at_n(g),cf))
                second=current.mapped(lam('_right',ty,combine(at_n(g),var('_right'))),combine(at_n(g),cf),combine(at_n(g),cg))
                step_proof=first.then(second).proof
            proof=induct('_index',prop,base,lam('_index',nat(),lam('_ih',prop,step_proof)),count)
            self.lib.emit(R+name+'Respects',args+[('other',fn),('_pointwise',point_type)],relation(left,right),proof,'theorem')
            # A bounded premise is enough: no equality outside the actual
            # visited interval becomes an extra function-entry assumption.
            comparator=(lambda a,b:ic('Equal',a,b)) if name=='IntSum' else (lambda a,b:bc('Equal',a,b)) if ty==boolean() else (lambda a,b:nc('Equal',a,b))
            predicate=lam('_element',nat(),comparator(app(f,var('_element')),app(g,var('_element'))))
            all_equal=lambda n:rc('FoldForall',lower,n,predicate)
            prop=pi('_bounded',eq(boolean(),all_equal(index),bit(True)),relation(at_n(f),at_n(g)))
            previous_true=call(MODULE+'.Bool.AndLeft',all_equal(index),comparator(cf,cg),var('_bounded'))
            current_true=call(MODULE+'.Bool.AndRight',all_equal(index),comparator(cf,cg),var('_bounded'))
            reflected=ic('EqualReflect',cf,cg,current_true) if name=='IntSum' else call(MODULE+'.Bool.EqualReflect',cf,cg,current_true) if ty==boolean() else nc('EqualReflect',cf,cg,current_true)
            past=app(var('_previous'),previous_true)
            if name=='IntSum':
                step_value=ic('AddRespects',at_n(f),cf,at_n(g),cg,past,reflected)
                base_value=ic('Reflexive',initial)
            else:
                p=Equality(at_n(f),at_n(g),past,ty);q=Equality(cf,cg,reflected,ty)
                e=p.mapped(lam('_left',ty,combine(var('_left'),cf)),combine(at_n(f),cf),combine(at_n(g),cf))
                e=e.then(q.mapped(lam('_right',ty,combine(at_n(g),var('_right'))),combine(at_n(g),cf),combine(at_n(g),cg)))
                step_value=e.proof;base_value=refl(ty,initial)
            base=lam('_bounded',eq(boolean(),all_equal(zero()),bit(True)),base_value)
            step=lam('_index',nat(),lam('_previous',prop,lam('_bounded',eq(boolean(),all_equal(succ(index)),bit(True)),step_value)))
            proof=induct('_index',prop,base,step,count)
            result=pi('_bounded',eq(boolean(),all_equal(count),bit(True)),relation(left,right))
            self.lib.emit(R+name+'BoundedRespects',args+[('other',fn)],result,proof,'theorem')
