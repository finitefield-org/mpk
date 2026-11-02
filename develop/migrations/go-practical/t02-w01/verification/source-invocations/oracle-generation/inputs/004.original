"""Explicit ordinary Sort0 terms and balanced heterogeneous products."""


def const(name):return {"kind":"const","name":name}
def var(name):return {"kind":"var","name":name}
def app(function,*arguments):return {"kind":"app","function":function,"arguments":list(arguments)} if arguments else function
def pi(name,domain,body):return {"kind":"pi","name":name,"domain":domain,"body":body}
def lam(name,domain,body):return {"kind":"lam","name":name,"domain":domain,"body":body}
def boolean():return const("Std.Bool")
def bit(value):return const("Std.Bool.true" if value else "Std.Bool.false")
def nat():return const("Std.Nat")
def eq(ty,left,right):return app(const("Std.Eq"),ty,left,right)
def refl(ty,value):return app(const("Std.Eq.refl"),ty,value)
def rec_type(left,right,tag):return app(const("Std.Bool.recType"),left,right,tag)


def bind(kind,args,body):
    for name,ty in reversed(args):body=(pi if kind=="pi" else lam)(name,ty,body)
    return body


def vector(bits,element,zero):
    args=[("a"+str(i),boolean()) for i in range(bits)]
    return bind("pi",args,element),bind("lam",args,zero)


def word(width):return vector((width-1).bit_length(),boolean(),bit(False))


def product(parts,fresh):
    """Empty product uses Bool with the separate canonical-false validity rule.

    A binary node is (tag:Bool)->Bool.recType A B tag. Its constructor is
    dependent Bool.cases, rather than a same-typed conditional on two values.
    """
    if not parts:return boolean(),bit(False)
    if len(parts)==1:return parts[0]
    middle=len(parts)//2
    left,lz=product(parts[:middle],fresh);right,rz=product(parts[middle:],fresh)
    name="p"+str(fresh[0]);fresh[0]+=1
    ty=pi(name,boolean(),rec_type(left,right,var(name)))
    motive=lam(name,boolean(),rec_type(left,right,var(name)))
    value=lam(name,boolean(),app(const("Std.Bool.cases"),motive,lz,rz,var(name)))
    return ty,value


def validate_expr(value,scope=()):
    """Closed AST/variable scope, not a replacement for either core checker."""
    if type(value) is not dict or type(value.get("kind")) is not str:raise ValueError("ordinary expression record")
    kind=value["kind"]
    fields={"sort0":{"kind"},"const":{"kind","name"},"var":{"kind","name"},"app":{"kind","function","arguments"},"pi":{"kind","name","domain","body"},"lam":{"kind","name","domain","body"}}
    if kind not in fields or set(value)!=fields[kind]:raise ValueError("closed ordinary AST; no axiom or inactive payload")
    if kind in {"const","var","pi","lam"} and (type(value["name"]) is not str or not value["name"]):raise ValueError("exact expression name")
    if kind=="var" and value["name"] not in scope:raise ValueError("no free variable")
    if kind=="app":
        if type(value["arguments"]) is not list or not value["arguments"]:raise ValueError("nonempty application")
        validate_expr(value["function"],scope)
        for a in value["arguments"]:validate_expr(a,scope)
    elif kind in {"pi","lam"}:
        validate_expr(value["domain"],scope);validate_expr(value["body"],scope+(value["name"],))


def pair_library(prefix,left,right):
    tag="tag";a="left";b="right";p="pair"
    ty=pi(tag,boolean(),rec_type(left,right,var(tag)))
    motive=lam(tag,boolean(),rec_type(left,right,var(tag)))
    constructor=bind("lam",[(a,left),(b,right)],lam(tag,boolean(),app(const("Std.Bool.cases"),motive,var(a),var(b),var(tag))))
    rows=[]
    def emit(name,kind,t,v):
        validate_expr(t);validate_expr(v);rows.append({"name":prefix+"."+name,"kind":kind,"ty":t,"value":v})
    emit("Make","definition",bind("pi",[(a,left),(b,right)],ty),constructor)
    for name,flag,result,arg in (("Left",False,left,a),("Right",True,right,b)):
        emit(name,"definition",pi(p,ty,result),lam(p,ty,app(var(p),bit(flag))))
        made=app(const(prefix+".Make"),var(a),var(b))
        projected=app(const(prefix+"."+name),made)
        args=[(a,left),(b,right)]
        emit(name+"OfMake","theorem",bind("pi",args,eq(result,projected,var(arg))),bind("lam",args,refl(result,var(arg))))
    return rows
