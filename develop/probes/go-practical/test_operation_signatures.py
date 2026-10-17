"""Concrete type/control/call/check boundaries; typed candidates are not source proofs."""
import copy
import unittest
from operation_signature_vectors import model, parameters, definitions, enclosing, mutate, reseal, catalogue
from operation_signatures import STATE, COMMON
from contract_scope import Scope
from operation_signatures import Signatures


class OperationSignatureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.m=model();cls.ps=parameters(cls.m);cls.ds=definitions(cls.m)

    def derive(self,n,**updates):
        p=copy.deepcopy(self.ps[n]);p.update(updates);return self.m.derive("GP-OP-"+str(n).zfill(2),p)

    def test_all56_signatures_and_six_common_members(self):
        self.assertEqual(len(self.ds),56);self.m.validate_all(self.ds)
        for d in self.ds:self.assertEqual([o["predicate"] for o in d["obligations"][:6]],list(COMMON))

    def test_every_rehashed_signature_and_missing_member_rejects(self):
        records=[mutate(d,kind) for kind in ("extra_parameter","alter_signature","omit_obligation") for d in self.ds]
        self.assertEqual(self.m.classify(records),[False]*168)

    def test_all_parameter_records_are_closed(self):
        for d in self.ds:
            p=copy.deepcopy(d["parameters"]);p["caller_adapter"]=True
            with self.assertRaises(ValueError):self.m.derive(d["op"],p)
        missing=copy.deepcopy(self.ds[0]);missing.pop('op')
        extra={**self.ds[0],'verified':True}
        later={**self.ds[0],'schema':'mpk.go.operation_definition.v2'}
        self.assertEqual(self.m.classify([None,True,[],{},missing,extra,later]),[False]*7)

    def test_nominal_operands_cannot_cross_by_equal_width(self):
        d=copy.deepcopy(self.ds[4]);d["inputs"][1]["source_type_id"]=self.m.id("fixture/policy.OtherAmount")
        with self.assertRaises(ValueError):self.m.validate(reseal(d))
        with self.assertRaises(ValueError):self.derive(21,targets=[self.m.id("fixture/policy.Amount")],types=[self.m.id("fixture/policy.OtherAmount")])
        self.derive(3,to=self.m.id("fixture/policy.OtherAmount"))

    def test_all_source_integer_bits_and_representability(self):
        self.assertEqual(self.ds[1]["parameters"]["value"],"18446744073709551615")
        for t,value in (("int8","-128"),("uint8","255")):self.derive(2,type=self.m.id(t),value=value)
        for t,value in (("int8","-129"),("uint8","256"),("int8","-0"),("uint64",True)):
            with self.assertRaises(ValueError):self.derive(2,type=self.m.id(t),value=value)

    def test_shift_count_is_full_source_type_and_no_width_limit(self):
        self.assertEqual(self.ds[6]["inputs"][1]["source_type_id"],self.m.id("uint64"))
        checks=[o["predicate"] for o in self.ds[6]["obligations"] if o["category"]=="runtime_safety"]
        self.assertEqual(checks,["signed shift count >= 0"])

    def test_compound_division_shift_and_write_checks_are_not_lost(self):
        for op in ("divide","remainder","left","right"):
            d=self.derive(22,operator=op,rhs_type=self.m.id("fixture/policy.Amount"))
            checks=[o for o in d["obligations"] if o["category"]=="runtime_safety"]
            self.assertEqual(len(checks),1);self.assertTrue(any(o["category"]=="permission" for o in d["obligations"]))

    def test_short_circuit_rhs_is_a_guarded_state_region(self):
        d=self.ds[7];self.assertEqual([r["role"] for r in d["regions"]],["guarded_rhs"])
        self.assertEqual(d["outputs"][-1],STATE)
        unary=self.derive(8,operator="not");self.assertFalse(unary["regions"]);self.assertEqual(len(unary["inputs"]),1)

    def test_switch_keeps_case_group_order_and_default_position(self):
        d=self.ds[15];self.assertEqual([r["role"] for r in d["regions"]],['case_test:0:0','case_test:0:1','case_body:0','case_test:1:0','case_body:1','default'])
        self.assertEqual(d["parameters"]["default_index"],0)
        for arities in ([0],[-1],[True]):
            with self.assertRaises(ValueError):self.derive(16,case_arities=arities)
        self.derive(16,type=self.m.id("fixture/policy.LineArray"))
        with self.assertRaises(ValueError):self.derive(16,type=self.m.id("fixture/policy.Root"))

    def test_result_products0_and16_are_ordered_and17_rejects(self):
        t=self.m.id("int64")
        self.assertEqual(self.derive(17,types=[])["outputs"][0]["result_types"],[])
        self.derive(17,types=[t]*16)
        with self.assertRaises(ValueError):self.derive(17,types=[t]*17)

    def test_parallel_values_are_not_limited_to_call_result_arity(self):
        t=self.m.id("int64");d=self.derive(21,types=[t]*32,targets=[None,t]*16)
        self.assertEqual(d["inputs"][-2]["kind"],"values");self.assertEqual(len(d["inputs"][-2]["types"]),32)

    def test_static_call_and_projection_keep_receiver_and_joint_post_state(self):
        d=self.ds[18];self.assertEqual(d["inputs"][0]["source_type_id"],self.m.id("fixture/policy.RejectCode"))
        self.assertEqual(d["outputs"][0]["kind"],"joint");self.assertEqual(d["outputs"][1],STATE)
        p=self.ds[17];self.assertEqual(p["inputs"][0],d["outputs"][0]);self.assertEqual(p["inputs"][1],STATE)
        with self.assertRaises(ValueError):self.derive(18,index=1)
        with self.assertRaises(ValueError):self.derive(19,callee="0"*64)

    def test_return_preserves_exact_selected_function_product(self):
        self.assertEqual(self.ds[19]["inputs"][0]["result_types"],self.m.fn_results)
        self.assertEqual(self.ds[19]["outputs"][0]["result_types"],self.m.fn_results)

    def test_error_dispatch_uses_every_global_case_and_boxing_no_call(self):
        d=self.ds[25];self.assertEqual(len(d["binding_requirements"]),1)
        self.assertEqual({r["case_type_id"] for r in d["binding_requirements"]},set(self.m.specializer.universe["cases"]))
        self.assertFalse(self.ds[23]["binding_requirements"])
        with self.assertRaises(ValueError):self.derive(24,type=self.m.id("fixture/policy.Amount"))
        u=("fixture/policy.RejectCode","fixture/policy.SliceError")
        with self.assertRaises(ValueError):Signatures(Scope(self.m.scope.raw,u)) # []error backing is non-Plain in this U.
        raw=copy.deepcopy(self.m.scope.raw);raw['function']['function']='Quote';raw['loops']=[];raw['error_inputs']=[]
        other=Signatures(Scope(raw,u));self.assertEqual(len(other.derive('GP-OP-26',{})['binding_requirements']),2)

    def test_complete_bytes_and_literal_domain_are_preserved(self):
        self.derive(27,bytes_hex="00ff"*128)
        for value in ("00"*257,"FF","0",1):
            with self.assertRaises(ValueError):self.derive(27,bytes_hex=value)

    def test_adapters_pin_both_exact_original_source_files(self):
        for d,name in ((self.ds[32],"HasPrefix"),(self.ds[33],"HasSuffix")):
            b=d["binding_requirements"][0];self.assertEqual(b["symbol"],"strings."+name);self.assertEqual(len(b["sources"]),2)
            bad=copy.deepcopy(d);bad["binding_requirements"][0]["symbol"]="caller."+name
            with self.assertRaises(ValueError):self.m.validate(reseal(bad))

    def test_noaccess_and_copy_permissions_remain_obligations(self):
        for n in (45,46,47,48,49,50):self.assertTrue(any(o["category"]=="permission" for o in self.ds[n-1]["obligations"]))
        self.assertTrue(any("NoAccess" in o["predicate"] for o in self.ds[44]["obligations"]))

    def test_array_views_need_plain_cells_and_exact_slice_element(self):
        with self.assertRaises(ValueError):self.derive(42,slice_type=self.m.id("[]int64"))
        with self.assertRaises(ValueError):self.derive(39,type=self.m.id("fixture/policy.Root"))
        self.assertNotEqual(len(self.derive(42,max_type=None)["inputs"]),len(self.derive(42,max_type=self.m.id("uint64"))["inputs"]))

    def test_array_range_can_omit_evaluation_only_without_second_value(self):
        d=self.derive(52,evaluation="constant_length",value=False);self.assertEqual(d["inputs"],[STATE])
        with self.assertRaises(ValueError):self.derive(52,evaluation="constant_length",value=True)
        self.assertEqual(len(self.ds[52]["inputs"]),2) # Slice descriptor capture always remains.

    def test_missing_for_test_does_not_run_a_test_region(self):
        d=self.derive(51,has_test=False,condition_type=self.m.id("bool"));self.assertEqual([r["role"] for r in d["regions"]],["init","body","post"])
        self.assertEqual(d["regions"][-1]["outputs"],[STATE])

    def test_break_and_continue_resolve_different_innermost_targets(self):
        # Supplied private syntactic context, not a parsed switch source fact.
        loop=enclosing(self.m);switch={**loop,"kind":"switch","start_byte":loop['start_byte']+1,"end_byte":loop['end_byte']-1,"ordinal":999}
        self.assertEqual(self.derive(54,enclosing=[loop,switch])["binding_requirements"][0]["target"]["kind"],"switch")
        self.assertEqual(self.derive(55,enclosing=[loop,switch])["binding_requirements"][0]["target"]["kind"],"loop")
        with self.assertRaises(ValueError):self.derive(55,enclosing=[switch])
        with self.assertRaises(ValueError):self.derive(54,enclosing=[loop,loop])

    def test_unprovable_append_domain_is_not_a_new_syntax_limit(self):
        self.derive(48,count=129)
        self.assertTrue(any(o["category"]=="profile_domain" for o in self.ds[47]["obligations"]))
        d=copy.deepcopy(self.ds[31]);next(o for o in d["obligations"] if o["category"]=="profile_domain")["category"]="runtime_safety"
        with self.assertRaises(ValueError):self.m.validate(reseal(d))

    def test_source_context_mixing_and_mutable_maps_reject(self):
        d=copy.deepcopy(self.ds[0]);d["semantic_context"]["source_language"]="rust"
        with self.assertRaises(ValueError):self.m.validate(reseal(d))
        m=model();good=m.derive("GP-OP-05",self.ps[5]);m.rows[self.m.id("fixture/policy.Amount")]["kind"]="bool"
        bad=copy.deepcopy(good);bad["inputs"]=[STATE]
        with self.assertRaises(ValueError):m.validate(reseal(bad))

    def test_catalogue_has_all_owner_and_parameter_pairs(self):
        c=catalogue(self.m);self.assertEqual(len(c["operations"]),56)
        for row in c["operations"]:
            self.assertEqual(set(row["parameter_required_fields"]),set(row["parameter_field_types"]))
            self.assertTrue(all(row[k] for k in ('producer','consumer','vc_owner','test_owner')))


if __name__=="__main__":unittest.main()
