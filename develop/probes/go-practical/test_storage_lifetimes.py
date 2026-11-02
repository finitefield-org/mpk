import copy
from unittest import TestCase, main
from storage_lifetimes import LiveWorld, Value, BOOL, BYTES, UINT64, Atom, ErrorValue, error_layout
from storage_views import INT, Layout, Descriptor, Location, array, structure, slice_layout, ShapeFailure, ProfileDomain, PermissionFailure


class LifetimeTests(TestCase):
    def world(self):
        w=LiveWorld(); w.begin_scope("f",entry=(),result_token="result:f",result_types=())
        return w

    def allocate_bound(self,w,name,length=1,element=INT):
        d=w.allocate("eval:"+name,"f",element,length)
        loc=w.declare("f",name,slice_layout(element),d); w.release("eval:"+name)
        return loc,d

    def test_reuses_slot_with_new_semantic_identity(self):
        w=self.world(); loc,old=self.allocate_bound(w,"s"); first=w.owner_slots[old.array.owner]
        w.write(loc,w.zero_value(slice_layout(INT)));w.collect()
        new=w.allocate("next","f",INT,1)
        self.assertEqual(first,w.owner_slots[new.array.owner]);self.assertNotEqual(old.array.owner,new.array.owner)
        with self.assertRaises(ShapeFailure):w.validate(old)

    def test_two_live_owners_block_third(self):
        w=self.world();self.allocate_bound(w,"a");self.allocate_bound(w,"b")
        with self.assertRaises(ProfileDomain):w.allocate("third","f",INT,1)

    def test_evaluation_result_is_a_root_before_assignment(self):
        w=self.world();a=w.allocate("rhs:a","f",INT,1);b=w.allocate("rhs:b","f",INT,1)
        w.collect();self.assertEqual(w.live_mask("int"),(True,True))
        with self.assertRaises(ProfileDomain):w.allocate("rhs:c","f",INT,1)
        w.release("rhs:a");w.collect();self.assertNotIn(a.array.owner,w.owners);self.assertIn(b.array.owner,w.owners)

    def test_captured_target_keeps_old_backing_after_rebind(self):
        w=self.world();s,old=self.allocate_bound(w,"s");t,new=self.allocate_bound(w,"t")
        w.hold_target("lhs:s0","f",w.element_location(old,0));w.write(s,new);w.collect()
        with self.assertRaises(ProfileDomain):w.allocate("third","f",INT,1)
        w.write(w.targets["lhs:s0"][1],9);self.assertEqual(w.values(old),[9]);self.assertEqual(w.values(new),[0])
        w.release("lhs:s0");w.collect();self.assertNotIn(old.array.owner,w.owners)

    def test_zero_length_positive_capacity_remains_live(self):
        w=self.world();s,d=self.allocate_bound(w,"s",2);empty=w.reslice(d,0,0)
        w.write(s,empty);w.collect();self.assertIn(d.array.owner,w.owners)
        w.write(s,w.reslice(empty,0,0,0));w.collect();self.assertNotIn(d.array.owner,w.owners)

    def test_array_value_copy_retains_descriptor_roots(self):
        w=self.world();_,a=self.allocate_bound(w,"a");_,b=self.allocate_bound(w,"b")
        t=array(slice_layout(INT),2); original=w.declare("f","bundle",t,[a,b]); copied=w.declare("f","copy",t,w.read(original))
        for name in ("a","b"):w.write(w.variable("f",name),w.zero_value(slice_layout(INT)))
        w.write(original,w.zero_value(t));w.collect();self.assertEqual(w.live_mask("int"),(True,True))
        w.write(copied,w.zero_value(t));w.collect();self.assertEqual(w.live_mask("int"),(False,False))

    def test_complete_error_payload_holds_owner(self):
        w=self.world();s,d=self.allocate_bound(w,"s")
        payload=structure("RefError",[("Data",slice_layout(INT))]);e=error_layout([("RefError",payload)])
        err=w.declare("f","err",e,ErrorValue("RefError",{"Data":d}))
        w.write(s,w.zero_value(slice_layout(INT)));w.collect();self.assertIn(d.array.owner,w.owners)
        w.write(err,ErrorValue());w.collect();self.assertNotIn(d.array.owner,w.owners)

    def test_global_nonplain_error_cannot_hide_in_slice(self):
        w=self.world();payload=structure("RefError",[("Data",slice_layout(INT))]);e=error_layout([("RefError",payload)])
        w.register(e)
        with self.assertRaises(ShapeFailure):w.register(slice_layout(e))
        with self.assertRaises(ShapeFailure):w.register(Layout("smaller-error","error",fields=()))

    def test_plain_error_bool_bytes_and_uint64_cells(self):
        w=self.world();code=Atom("Code","word",width=8,signed=False);e=error_layout([("Code",code)])
        for i,t in enumerate((BOOL,BYTES,UINT64,e)):
            d=w.allocate("value:"+str(i),"f",t,1);w.collect();self.assertIn(d.array.owner,w.owners)
        self.assertEqual(w.values(w.packets["value:1"].items[0].data),[b""])
        u=w.packets["value:2"].items[0].data;w.write(w.element_location(u,0),(1<<64)-1)
        self.assertEqual(w.values(u),[(1<<64)-1])

    def test_entry_constant_pins_after_parameter_reassignment(self):
        w=LiveWorld();w.begin_scope("caller");d=w.allocate("arg","caller",INT,1)
        w.begin_scope("callee","caller",entry=[Value(slice_layout(INT),d)],result_token="call",result_types=())
        w.release("arg");p=w.declare("callee","p",slice_layout(INT),d);w.write(p,w.zero_value(slice_layout(INT)))
        w.collect();self.assertIn(d.array.owner,w.owners)
        w.put_result("callee",[]);w.end_scope("callee");self.assertNotIn(d.array.owner,w.owners)

    def test_snapshot_uses_past_store_after_current_slot_reuse(self):
        w=LiveWorld();w.begin_scope("caller");d=w.allocate("arg","caller",INT,1);w.write(w.element_location(d,0),7)
        w.begin_scope("callee","caller",entry=[Value(slice_layout(INT),d)],result_token="call",result_types=())
        snapshot=w.scopes["callee"].snapshot;w.release("arg");w.put_result("callee",[]);w.end_scope("callee")
        new=w.allocate("next","caller",INT,1);w.write(w.element_location(new,0),11)
        self.assertEqual(snapshot.store.values(snapshot.inputs[0].data),[7]);self.assertEqual(w.values(new),[11])
        with self.assertRaises(PermissionFailure):snapshot.store.write(snapshot.store.element_location(d,0),99)
        with self.assertRaises(ShapeFailure):w.validate(d)

    def test_post_input_frame_is_kept_in_same_invocation_evidence(self):
        w=LiveWorld();w.begin_scope("caller");d=w.allocate("arg","caller",INT,1);w.write(w.element_location(d,0),7)
        w.begin_scope("callee","caller",entry=[Value(slice_layout(INT),d)],result_token="call",result_types=(INT,));w.release("arg")
        w.put_result("callee",[Value(INT,3)]);w.end_scope("callee");e=w.packets["call"].evidence
        self.assertNotIn(d.array.owner,w.owners);self.assertEqual(e.invocation,"callee")
        self.assertEqual(e.entry.store.values(e.entry.inputs[0].data),[7]);self.assertEqual(e.post.store.values(e.post.inputs[0].data),[7])
        self.assertEqual(e.post.outputs[0].data,3)
        new=w.allocate("next","caller",INT,1);w.write(w.element_location(new,0),11)
        self.assertEqual(e.post.store.values(e.post.inputs[0].data),[7])

    def test_result_packet_pins_all_projections_until_consumed(self):
        w=LiveWorld();w.begin_scope("caller");w.begin_scope("callee","caller",entry=(),result_token="call",result_types=(INT,slice_layout(INT)))
        a=w.declare("callee","a",array(INT,2),[1,2]);d=w.view(a)
        w.put_result("callee",[Value(INT,7),Value(slice_layout(INT),d)])
        with self.assertRaises(ShapeFailure):w.project_result("call",0)
        w.end_scope("callee");self.assertEqual(w.project_result("call",0).data,7)
        escaped=w.project_result("call",1).data;w.collect();self.assertIn(escaped.array.owner,w.owners)
        var=w.declare("caller","s",slice_layout(INT),escaped);w.release("call");w.collect();self.assertIn(escaped.array.owner,w.owners)
        w.write(var,w.zero_value(slice_layout(INT)));w.collect();self.assertNotIn(escaped.array.owner,w.owners)

    def test_materialized_projection_survives_joint_packet_consumption(self):
        w=LiveWorld();w.begin_scope("caller");w.begin_scope("callee","caller",entry=(),result_token="call",result_types=(slice_layout(INT),))
        a=w.declare("callee","a",array(INT,1),[6]);w.put_result("callee",[Value(slice_layout(INT),w.view(a))]);w.end_scope("callee")
        value=w.materialize_result("call",0,"projected:0");w.release("call");w.collect()
        self.assertEqual(w.values(value.data),[6]);w.release("projected:0");w.collect();self.assertNotIn(value.data.array.owner,w.owners)

    def test_result_signature_and_invocation_tokens_are_closed(self):
        w=self.world();r=w.declare("f","x",INT);w.hold_target("occupied","f",r)
        with self.assertRaises(ShapeFailure):w.begin_scope("bad","f",result_token="occupied")
        with self.assertRaises(ShapeFailure):w.begin_scope("wide","f",result_token="wide:return",result_types=(INT,)*17)

    def test_root_error_in_joint_result_is_promoted_recursively(self):
        w=LiveWorld();w.begin_scope("caller");p=structure("RefError",[("Data",slice_layout(INT))]);e=error_layout([("RefError",p)])
        w.begin_scope("callee","caller",entry=(),result_token="call",result_types=(e,))
        a=w.declare("callee","a",array(INT,2),[3,4]);d=w.view(a)
        w.put_result("callee",[Value(e,ErrorValue("RefError",{"Data":d}))]);w.end_scope("callee")
        error=w.project_result("call",0).data;escaped=error.payload["Data"]
        self.assertNotEqual(escaped.array.owner,a.owner);self.assertEqual(w.values(escaped),[3,4])
        with self.assertRaises(ShapeFailure):w.read(a)

    def test_caller_storage_is_not_promoted_at_callee_exit(self):
        w=LiveWorld();w.begin_scope("caller");a=w.declare("caller","a",array(INT,2),[1,2]);d=w.view(a)
        w.begin_scope("callee","caller",entry=[Value(slice_layout(INT),d)],result_token="call",result_types=(slice_layout(INT),))
        w.put_result("callee",[Value(slice_layout(INT),d)]);w.end_scope("callee")
        returned=w.project_result("call",0).data;self.assertEqual(returned.array,d.array)
        w.write(w.element_location(d,0),9);self.assertEqual(w.values(returned),[9,2])

    def test_block_escape_is_derived_from_parent_binding(self):
        w=self.world();s=w.declare("f","s",slice_layout(INT));w.begin_scope("block","f")
        a=w.declare("block","a",array(INT,2),[1,2]);w.write(s,w.view(a));w.end_scope("block")
        d=w.read(s);self.assertNotEqual(d.array.owner,a.owner);self.assertEqual(w.values(d),[1,2])

    def test_return_transition_is_atomic_on_live_quota_failure(self):
        w=LiveWorld();w.begin_scope("caller")
        for n in ("a","b"):
            d=w.allocate("e:"+n,"caller",INT,1);w.declare("caller",n,slice_layout(INT),d);w.release("e:"+n)
        w.begin_scope("callee","caller",entry=(),result_token="call",result_types=(slice_layout(INT),))
        a=w.declare("callee","a",array(INT,1));w.put_result("callee",[Value(slice_layout(INT),w.view(a))])
        before=copy.deepcopy((w.owners,w.packets,w.slot_owners,w.ended_scopes,w.serial))
        scopes={k:(s.parent,copy.deepcopy(s.variables),copy.deepcopy(s.entry),s.result_token,s.snapshot) for k,s in w.scopes.items()}
        with self.assertRaises(ProfileDomain):w.end_scope("callee")
        self.assertEqual((w.owners,w.packets,w.slot_owners,w.ended_scopes,w.serial),before)
        self.assertEqual(set(w.scopes),set(scopes))
        for k,s in w.scopes.items():
            self.assertEqual((s.parent,s.variables,s.entry,s.result_token),scopes[k][:4]);self.assertIs(s.snapshot,scopes[k][4])

    def test_dying_local_references_do_not_block_escape_promotion(self):
        w=LiveWorld();w.begin_scope("caller");w.begin_scope("callee","caller",entry=(),result_token="call",result_types=(slice_layout(INT),))
        for n in ("dead:a","dead:b"):
            d=w.allocate(n,"callee",INT,1);w.declare("callee",n,slice_layout(INT),d);w.release(n)
        a=w.declare("callee","a",array(INT,1),[8]);w.put_result("callee",[Value(slice_layout(INT),w.view(a))]);w.end_scope("callee")
        self.assertEqual(w.values(w.project_result("call",0).data),[8]);self.assertEqual(len(w.slot_owners),1)

    def test_no_escape_list_or_live_mask_can_be_supplied(self):
        w=self.world()
        with self.assertRaises(TypeError):w.collect(skip_scope="f")
        with self.assertRaises(ShapeFailure):w.export_views([],"f")
        with self.assertRaises(ShapeFailure):w.create(array(INT,1),kind="heap")

    def test_scope_exit_requires_complete_evaluation_and_stack(self):
        w=self.world();w.begin_scope("block","f")
        with self.assertRaises(ShapeFailure):w.end_scope("f")
        d=w.allocate("rhs","block",INT,1)
        with self.assertRaises(ShapeFailure):w.end_scope("block")
        w.release("rhs");w.end_scope("block");self.assertNotIn(d.array.owner,w.owners)

    def test_one_result_assignment_for_one_invocation(self):
        w=self.world();w.put_result("f",[])
        with self.assertRaises(ShapeFailure):w.put_result("f",[])
        w.end_scope("f")
        with self.assertRaises(ShapeFailure):w.begin_scope("f")

    def test_many_iterations_reuse_one_live_slot(self):
        w=self.world();total=0;peak=0
        for i in range(100):
            scope="iteration:"+str(i);w.begin_scope(scope,"f");d=w.allocate("eval:"+str(i),scope,INT,1)
            w.declare(scope,"tmp",slice_layout(INT),d);w.release("eval:"+str(i));w.write(w.element_location(d,0),i)
            total+=w.values(d)[0];peak=max(peak,len(w.slot_owners));w.end_scope(scope)
        self.assertEqual(total,4950);self.assertEqual(peak,1);self.assertEqual({slot for _,slot in w.slot_events},{("int",0)})


if __name__=="__main__":main()
