"""Retained24 source cases plus typed witnesses covering all70 expression heads."""
import copy
import json
from contract_scope import Scope,source_overlay,FACTS
from contract_wire import normalize_contract,validate_contract,context,WireExpressions
from overlay_vectors import vectors,e,nat,integer,truth,inp,res,word,local
from overlay_contracts import OPERATORS


def source_cases():
    facts=json.loads(FACTS.read_text());rows=[]
    for c in vectors(facts):
        raw=source_overlay(c["overlay"])
        if c["id"]=="old-schema":raw["schema"]="mpk.go.contract.v0"
        rows.append({"id":c["id"],"raw":raw,"universe":c["universe"],"expected":c["expected"]})
    return rows


def typed_witnesses():
    facts=json.loads(FACTS.read_text());base=next(c for c in source_cases() if c["id"]=="missing-invariants-partial")
    scope=Scope(base["raw"],base["universe"]);codec=WireExpressions(scope)
    a,b,yes=nat(1),integer(2),truth();signed,unsigned=word("int64",4),word("uint8",5)
    string=e("bytes.literal",type="string",value="00ff");record=e("source.zero",type="fixture/policy.Line");error=local("err")
    values={"bool.literal":yes,"nat.literal":a,"int.literal":b,"word.literal":signed,"bytes.literal":string,"source.zero":record,"input":inp(0),"local":local("i"),"old":e("old",value=inp(0)),"if":e("if",yes,a,a),"word.convert":e("word.convert",type="uint8",value=signed),"bool.value":e("bool.value",inp(2)),"field":e("field",value=record,field="Amount"),"length":e("length",inp(0)),"capacity":e("capacity",inp(0)),"is_nil":e("is_nil",inp(0)),"byte_at":e("byte_at",string,nat(0)),"element":e("element",inp(0),nat(0)),"error.is":e("error.is",value=error,case="fixture/policy.RejectCode"),"error.payload":e("error.payload",value=error,case="fixture/policy.RejectCode"),"sequence.equal":e("sequence.equal",inp(0),inp(0)),"bool.not":e("bool.not",yes),"nat.to_int":e("nat.to_int",a),"int.neg":e("int.neg",b)}
    for ops,value in ((["bool.and","bool.or","bool.implies"],yes),(["nat.add","nat.mul","nat.sub"],a),(["int.add","int.sub","int.mul"],b),(["eq","ne","lt","le","gt","ge"],a)):
        for op in ops:values[op]=e(op,value,value)
    for op in ("word.neg","word.not"):values[op]=e(op,signed)
    for op in ("word.add","word.sub","word.mul","word.and","word.or","word.xor","word.andnot","word.div","word.rem","word.shl","word.shr"):values[op]=e(op,signed,signed)
    for op,value in (("nat.from_signed",signed),("nat.from_unsigned",unsigned),("int.from_signed",signed),("int.from_unsigned",unsigned)):values[op]=e(op,value)
    for op in ("bytes.prefix","bytes.suffix","bytes.concat"):values[op]=e(op,string,string)
    values["bytes.slice"]=e("bytes.slice",string,nat(0),nat(1));values["slice.view2"]=e("slice.view2",inp(0),nat(0),nat(1));values["slice.view3"]=e("slice.view3",inp(0),nat(0),nat(1),nat(1))
    for op,body in (("forall",yes),("exists",yes),("nat.sum",a),("int.sum",b)):values[op]=e(op,name="j",lower=nat(0),upper=nat(1),body=body)
    values["result"]=res(0);values["range.index"]=e("range.index");values["range.length"]=e("range.length")
    values["bound"]=e("forall",name="j",lower=nat(0),upper=nat(1),body=e("eq",e("bound",name="j"),nat(0)))
    assert set(values)==OPERATORS
    rows=[]
    for op,value in sorted(values.items()):
        phase="ensures" if op=="result" else "header";ordinal=None if phase=="ensures" else 2 if op.startswith("range.") else 0
        ctx=context(scope,phase,ordinal);wire=codec.encode(value,ctx,ordinal);raw=codec.decode(wire,ctx,ordinal)
        assert codec.encode(raw,ctx,ordinal)==wire
        rows.append({"op":op,"phase":phase,"ordinal":ordinal,"source":value,"wire":wire})
    return scope,rows
