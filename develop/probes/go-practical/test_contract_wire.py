"""Typed wire crossings, lexical namespaces and complete original-byte binding."""
import copy
import json
import unittest
from contract_scope import Scope,source_overlay,validate_scope,canonical,digest,GO
from contract_wire import WireExpressions,context,normalize_contract,validate_contract
from contract_wire_vectors import source_cases,typed_witnesses
from contract_catalog import generate,preflight
from overlay_vectors import e,inp,res,local,nat,word
from overlay_contracts import ProbeResourceLimit
from run_contract_wire import observed


class ContractWireTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.rows=source_cases();cls.s,cls.witnesses=typed_witnesses();cls.codec=WireExpressions(cls.s)

    def test_all_twenty_four_source_outcomes_remain_explicit(self):
        rows,accepted,_=observed();self.assertEqual(len(rows),24);self.assertEqual(len(accepted),6)
        self.assertEqual(sum(r["classification"]=="configuration_rejection" for r in rows),18)
        self.assertEqual(sum(len(a["contract"]["pending_candidates"]) for a in accepted),3)

    def test_all_seventy_exact_source_and_normalized_heads_roundtrip(self):
        cat=generate();self.assertEqual(len(cat["source_arms"]),70);self.assertEqual(len(cat["normalized_arms"]),70)
        for row in self.witnesses:
            c=context(self.s,row["phase"],row["ordinal"]);source=self.codec.decode(row["wire"],c,row["ordinal"])
            self.assertEqual(self.codec.encode(source,c,row["ordinal"]),row["wire"])

    def test_receiver_alias_resolves_to_same_decl_and_signature(self):
        row=next(c for c in self.rows if c["id"]=="value-receiver-alias");raw=copy.deepcopy(row["raw"])
        a=Scope(raw,row["universe"]);raw["function"]["receiver"]["type_name"]="RejectCode";b=Scope(raw,row["universe"])
        self.assertEqual(a.declaration,b.declaration);self.assertEqual(a.signature,b.signature)
        self.assertNotEqual(a.capture,b.capture) # Original overlay bytes still differ.

    def test_whitespace_and_escapes_change_raw_capture_without_changing_value(self):
        row=self.rows[0];raw=row["raw"];pretty=(json.dumps(raw,ensure_ascii=True,indent=4)+"\n").encode()
        a=Scope(raw,row["universe"]);b=Scope(raw,row["universe"],raw_bytes=pretty)
        self.assertNotEqual(a.overlay_id,b.overlay_id);self.assertNotEqual(a.capture,b.capture)
        self.assertNotEqual(normalize_contract(a)["contract_hash"],normalize_contract(b)["contract_hash"])

    def test_duplicate_original_json_members_never_disappear(self):
        row=self.rows[0];raw=canonical(row["raw"]);duplicate=raw.replace(b'"termination":"total"',b'"termination":"partial","termination":"total"')
        with self.assertRaises(ValueError):Scope(row["raw"],row["universe"],raw_bytes=duplicate)

    def test_binding_ids_are_typed_full_capture_function_and_span_preimages(self):
        for b in self.s.value["bindings"]:
            self.assertEqual(b["capture_context_id"],self.s.capture);self.assertEqual(b["function_decl_id"],self.s.declaration)
            self.assertLess(b["start_byte"],b["end_byte"])
            self.assertIn(b["source_type_id"],self.s.names)
        self.assertEqual(len({b["id"] for b in self.s.value["bindings"]}),15)

    def test_header_shadowing_does_not_accept_same_name_from_other_scope(self):
        outer=self.codec.encode(local("total"),context(self.s,"header",0),0)
        inner=self.codec.encode(local("total"),context(self.s,"header",1),1)
        self.assertNotEqual(outer["payload"]["binding_id"],inner["payload"]["binding_id"])
        bad=copy.deepcopy(inner);bad["payload"]["binding_id"]=outer["payload"]["binding_id"]
        with self.assertRaises(ValueError):self.codec.decode(bad,context(self.s,"header",1),1)

    def test_local_is_not_result_and_source_type_match_cannot_cross_namespace(self):
        local_result=self.codec.encode(local("total"),context(self.s,"header",0),0)
        result=self.codec.encode(res(0),context(self.s,"ensures"))
        self.assertEqual(local_result["type"],result["type"])
        self.assertEqual(local_result["payload"]["binding_id"],result["payload"]["binding_id"])
        with self.assertRaises(ValueError):self.codec.decode(result,context(self.s,"header",0),0)

    def test_full_old_observation_differs_from_old_descriptor_only(self):
        full=e("old",value=e("element",inp(0),nat(0)))
        descriptor=e("element",e("old",value=inp(0)),nat(0));c=context(self.s,"header",0)
        a=self.codec.encode(full,c,0);b=self.codec.encode(descriptor,c,0)
        self.assertEqual(a["payload"]["value"]["payload"]["namespace"],"entry")
        self.assertEqual(b["payload"]["namespace"],"header")
        bad=copy.deepcopy(a);bad["payload"]["value"]["payload"]["namespace"]="header"
        with self.assertRaises(ValueError):self.codec.decode(bad,c,0)

    def test_result_and_local_and_range_are_unavailable_under_old(self):
        for source,c,o in ((e("old",value=res(0)),context(self.s,"ensures"),None),(e("old",value=local("i")),context(self.s,"header",0),0),(e("old",value=e("range.index")),context(self.s,"header",2),2)):
            with self.assertRaises(ValueError):self.codec.encode(source,c,o)

    def test_range_reference_is_bound_to_the_exact_current_loop(self):
        c=context(self.s,"header",2);n=self.codec.encode(e("range.index"),c,2)
        n["payload"]["loop_ref_id"]=self.s.loop_ids[4]
        with self.assertRaises(ValueError):self.codec.decode(n,c,2)

    def test_bound_variables_use_depth_and_cannot_escape_their_binder(self):
        value=e("forall",name="j",lower=nat(0),upper=nat(2),body=e("forall",name="k",lower=nat(0),upper=nat(1),body=e("eq",e("bound",name="j"),e("bound",name="k"))))
        c=context(self.s,"header",0);n=self.codec.encode(value,c,0);args=n["payload"]["body"]["payload"]["body"]["payload"]["args"]
        self.assertEqual([a["payload"]["depth"] for a in args],[1,0]);self.codec.decode(n,c,0)
        with self.assertRaises(ValueError):self.codec.decode(args[0],c,0)

    def test_nominal_source_types_do_not_become_equal_through_same_width(self):
        with self.assertRaises(ValueError):self.codec.encode(e("eq",word("fixture/policy.Amount",7),word("fixture/policy.OtherAmount",7)),context(self.s,"header",0),0)
        value=self.codec.encode(word("fixture/policy.Money",7),context(self.s,"header",0),0)
        self.assertEqual(value["type"]["source_type_id"],self.s.ids["fixture/policy.Amount"])

    def test_source_bool_is_distinct_from_logical_bool(self):
        with self.assertRaises(ValueError):self.codec.encode(e("bool.and",inp(2),e("bool.literal",value=True)),context(self.s,"header",0),0)
        self.codec.encode(e("bool.value",inp(2)),context(self.s,"header",0),0)

    def test_full_word_decimal_survives_and_partiality_is_not_type_falsity(self):
        value=e("word.shl",word("uint8",1),word("uint64",18446744073709551615));c=context(self.s,"header",0)
        n=self.codec.encode(value,c,0);self.assertEqual(n["payload"]["args"][1]["payload"]["value"],"18446744073709551615")
        self.codec.encode(e("nat.from_signed",word("int64",-1)),c,0)
        members={r["op"]:r["definedness_members"] for r in generate()["normalized_arms"]}
        self.assertTrue(members["nat.from_signed"])

    def test_nested_error_paths_use_capacity_and_preserve_case_ids(self):
        row=next(c for c in self.rows if c["id"]=="nested-and-tail-error-entry-sets");s=Scope(row["raw"],row["universe"]);n=normalize_contract(s)
        self.assertEqual(n["error_inputs"][1]["path"][0]["extent"],"capacity")
        self.assertEqual(n["error_inputs"][0]["case_type_ids"],[s.ids["fixture/policy.RejectCode"]])

    def test_registered_count_depth_limits_are_separate_from_probe_resource(self):
        preflight([e("bool.literal",value=True)]*1024)
        with self.assertRaises(ValueError):preflight([e("bool.literal",value=True)]*1025)
        value=e("bool.literal",value=True)
        for _ in range(32):value=e("bool.not",value)
        preflight([value])
        with self.assertRaises(ValueError):preflight([e("bool.not",value)])
        with self.assertRaises(ProbeResourceLimit):self.codec.encode(e("nat.literal",value="1"*4097),context(self.s,"header",0),0)

    def test_unknown_inactive_payloads_and_type_hints_reject(self):
        c=context(self.s,"header",0);n=self.codec.encode(nat(0),c,0)
        for mutate in (lambda v:v.update(verified=True),lambda v:v["payload"].update(args=[]),lambda v:v.update(type={"kind":"logical","name":"Int"})):
            bad=copy.deepcopy(n);mutate(bad)
            with self.assertRaises(ValueError):self.codec.decode(bad,c,0)

    def test_rehashed_contract_or_mutable_scope_cannot_change_source_meaning(self):
        row=self.rows[0];s=Scope(row["raw"],row["universe"]);n=normalize_contract(s)
        n["entry_storage"]["input_access"]="writable";n["contract_hash"]=digest(GO["normalized_contract"][1],{k:v for k,v in n.items() if k!="contract_hash"})
        with self.assertRaises(ValueError):validate_contract(s,n)
        s.raw["ensures"]=[e("bool.literal",value=False)]
        with self.assertRaises(ValueError):normalize_contract(s)

    def test_catalogue_closes_three_roots_and_all_preimage_fields(self):
        cat=generate();self.assertEqual(len(cat["roots"]),3)
        for r in cat["roots"]:
            self.assertEqual(set(r["field_types"]),set(r["required_fields"]));self.assertEqual(r["optional_fields"],[])
            self.assertEqual(r["hash_preimage_fields"],[k for k in r["required_fields"] if k!=r["hash_field"]])


if __name__=="__main__":unittest.main()
