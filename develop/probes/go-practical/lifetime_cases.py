"""Independent expected native facts and typed full-root candidate traces."""
from storage_lifetimes import LiveWorld, Value, BOOL, BYTES, UINT64, Atom, ErrorValue, error_layout
from storage_views import INT, array, structure, slice_layout

EXPECTED = {
    "overwritten_reference": [9], "iteration_values": 499500,
    "positive_capacity_empty": {"values":[7],"capacity":2},
    "zero_access_tag": {"nil":False,"length":0,"capacity":0},
    "captured_assignment_target": {"current":2,"old":9},
    "array_descriptor_copy": [8,2],
    "error_result_payload": {"first":7,"nil":False,"message":"carried","data":[9,2,3,4]},
    "caller_scope_preserved": [88,2],
    "past_store_isolated": {"old":7,"changed":9,"new":11},
    "error_nil_and_concrete_zero": [True,False],
    "byte_data": "00ff", "unsigned_word": "18446744073709551615",
}


def world():
    w=LiveWorld();w.begin_scope("root",entry=(),result_token="root:return",result_types=())
    return w


def bound(w,name,data):
    d=w.allocate("eval:"+name,"root",INT,len(data))
    for i,v in enumerate(data):w.write(w.element_location(d,i),v)
    loc=w.declare("root",name,slice_layout(INT),d);w.release("eval:"+name)
    return loc,d


def model():
    out={};metrics={}
    w=world();loc,d=bound(w,"s",[7]);first=w.owner_slots[d.array.owner]
    w.write(loc,w.zero_value(slice_layout(INT)));w.collect()
    new=w.allocate("next","root",INT,1);w.write(w.element_location(new,0),9);w.write(loc,new);w.release("next")
    out["overwritten_reference"]=w.values(new);metrics["overwrite_reused_slot"]=first==w.owner_slots[new.array.owner]
    w=world();total=w.declare("root","sum",INT,0);peak=0
    for i in range(1000):
        scope="iteration:"+str(i);w.begin_scope(scope,"root");d=w.allocate("eval:"+str(i),scope,INT,1)
        w.declare(scope,"tmp",slice_layout(INT),d);w.release("eval:"+str(i));w.write(w.element_location(d,0),i)
        w.write(total,w.read(total)+w.values(d)[0]);peak=max(peak,len(w.slot_owners));w.end_scope(scope)
    out["iteration_values"]=w.read(total);metrics["iteration_peak_positive_slots"]=peak;metrics["iteration_allocations"]=len(w.slot_events)
    metrics["iteration_reused_one_slot"]=len({slot for _,slot in w.slot_events})==1
    w=world();loc,d=bound(w,"s",[0,0]);empty=w.reslice(d,0,0);w.write(loc,empty);w.collect()
    empty=w.append(empty,[7]);w.write(loc,empty);out["positive_capacity_empty"]={"values":w.values(empty),"capacity":empty.capacity}
    z=w.reslice(empty,0,0,0);w.write(loc,z);w.collect();out["zero_access_tag"]={"nil":z.nil,"length":z.length,"capacity":z.capacity}
    metrics["zero_access_retired_storage"]=d.array.owner not in w.owners
    w=world();s,old=bound(w,"s",[1]);_,new=bound(w,"t",[2]);w.hold_target("lhs:s0","root",w.element_location(old,0))
    w.write(s,new);w.collect();w.write(w.targets["lhs:s0"][1],9)
    out["captured_assignment_target"]={"current":w.values(new)[0],"old":w.values(old)[0]}
    w.release("lhs:s0");w.collect();metrics["captured_target_release_retires_old"]=old.array.owner not in w.owners
    w=world();a,da=bound(w,"a",[1]);b,db=bound(w,"b",[2]);t=array(slice_layout(INT),2)
    root=w.declare("root","bundle",t,[da,db]);copied=w.declare("root","copy",t,w.read(root))
    w.write(root,w.zero_value(t));w.write(a,w.zero_value(slice_layout(INT)));w.write(b,w.zero_value(slice_layout(INT)));w.collect()
    values=w.read(copied);w.write(w.element_location(values[0],0),8);out["array_descriptor_copy"]=[w.values(d)[0] for d in values]
    w=LiveWorld();w.begin_scope("caller");p=structure("carriedError",[("Data",slice_layout(INT))]);e=error_layout([("carriedError",p)])
    w.begin_scope("callee","caller",entry=(),result_token="call",result_types=(INT,e));a=w.declare("callee","a",array(INT,4),[1,2,3,4]);d=w.view(a)
    w.put_result("callee",[Value(INT,7),Value(e,ErrorValue("carriedError",{"Data":d}))]);w.end_scope("callee")
    n=w.project_result("call",0).data;error=w.project_result("call",1).data;root=w.declare("caller","err",e,error);w.release("call");w.collect()
    carried=w.read(root).payload["Data"];w.write(w.element_location(carried,0),9)
    out["error_result_payload"]={"first":n,"nil":False,"message":"carried","data":w.values(carried)}
    w=LiveWorld();w.begin_scope("caller");a=w.declare("caller","a",array(INT,2),[1,2]);d=w.view(a)
    w.begin_scope("callee","caller",entry=[Value(slice_layout(INT),d)],result_token="call",result_types=(slice_layout(INT),));w.put_result("callee",[Value(slice_layout(INT),d)]);w.end_scope("callee")
    returned=w.project_result("call",0).data;w.write(w.child(a,"index",0),88);out["caller_scope_preserved"]=w.values(returned)
    w=LiveWorld();w.begin_scope("caller");d=w.allocate("arg","caller",INT,1);w.write(w.element_location(d,0),7)
    w.begin_scope("callee","caller",entry=[Value(slice_layout(INT),d)],result_token="call",result_types=());snapshot=w.scopes["callee"].snapshot;w.release("arg")
    w.write(w.element_location(d,0),9);changed=w.values(d)[0];w.put_result("callee",[]);w.end_scope("callee")
    new=w.allocate("new","caller",INT,1);w.write(w.element_location(new,0),11)
    out["past_store_isolated"]={"old":snapshot.store.values(snapshot.inputs[0].data)[0],"changed":changed,"new":w.values(new)[0]}
    w=world();code=Atom("code","word",width=8,signed=False);e=error_layout([("code",code)]);w.register(e)
    out["error_nil_and_concrete_zero"]=[w.zero_value(e).case is None,ErrorValue("code",0).case is None]
    d=w.allocate("bytes","root",BYTES,1);w.write(w.element_location(d,0),b"\x00\xff");out["byte_data"]=w.values(d)[0].hex()
    d=w.allocate("word","root",UINT64,1);w.write(w.element_location(d,0),(1<<64)-1);out["unsigned_word"]=str(w.values(d)[0])
    return out,metrics
