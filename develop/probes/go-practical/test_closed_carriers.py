"""Concrete specialization, context, alias, shape and ordinary-proof regressions."""
import copy
import json
import unittest
from closed_carriers import *
from carrier_cases import plain_error_candidate,CASES,plan
from foundation_limits_vectors import linked_contexts
from carrier_terms import validate_expr
from identity_wire import seal as source_seal


class ClosedCarrierTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.f,cls.i,cls.e=candidate();cls.context=cls.i["semantic_context"]

    def changed(self,change):
        t=copy.deepcopy(self.f["types"]);change(t);t["content_sha256"]=digest(TYPE_DOMAIN,{k:v for k,v in t.items() if k!="content_sha256"});return t

    def test_complete_instance_and_ordinary_member_counts(self):
        self.assertEqual(len(self.f["types"]["definitions"]),39);self.assertEqual(len(self.i["entries"]),42)
        self.assertEqual(sum(e["role"]=="source" for e in self.i["entries"]),38)
        self.assertEqual(sum(d["kind"]=="definition" for d in self.e["declarations"]),90)

    def test_aliases_preserve_the_target_instead_of_creating_an_instance(self):
        entries={e.get("source_type_id") for e in self.i["entries"]};nodes={n["id"]:n for n in self.f["types"]["identity_nodes"]}
        for a in self.f["types"]["aliases"]:
            self.assertIn(a["target_type_id"],entries);self.assertNotIn(a["declaration_id"],entries)
            self.assertEqual(nodes[a["declaration_id"]]["kind"],"alias_type")

    def test_nominal_identity_survives_equal_representations(self):
        a=self.f["ids"]["fixture/policy.Amount"];b=self.f["ids"]["fixture/policy.OtherAmount"]
        s=Specializer(self.f["types"],self.context)
        self.assertNotEqual(s.source(a)["id"],s.source(b)["id"])
        self.assertEqual(canonical(s.carrier(a)[0]),canonical(s.carrier(b)[0]))

    def test_every_source_integer_width_retains_all_bits(self):
        s=Specializer(self.f["types"],self.context)
        for row in self.f["types"]["definitions"]:
            if row["kind"]!="word":continue
            ty=s.carrier(row["id"])[0];depth=0
            while ty["kind"]=="pi":depth+=1;ty=ty["body"]
            self.assertEqual(1<<depth,row["width"])
            self.assertEqual(ty,const("Std.Bool"))

    def test_zero_and_nonpower_array_lengths_keep_distinct_identities(self):
        s=Specializer(self.f["types"],self.context);seen=set()
        for n,bits in ((0,None),(1,0),(3,2),(256,8)):
            key=self.f["ids"][f"fixture-array-{n}"];seen.add(s.source(key)["id"])
            ty,_=s.carrier(key);depth=0
            while ty["kind"]=="pi":depth+=1;ty=ty["body"]
            self.assertEqual(depth,bits or 0)
        self.assertEqual(len(seen),4)

    def test_string_keeps_full_length_and_arbitrary_byte_padding(self):
        s=Specializer(self.f["types"],self.context);key=self.f["ids"]["string"]
        self.assertEqual(s.source(key)["weight_units"],257)
        ty=s.carrier(key)[0];parts=ty["body"]["arguments"][:2]
        self.assertEqual(parts[0],word(64)[0]);self.assertEqual(parts[1],vector(8,*word(8))[0])

    def test_slice_owner_labels_remain_ghost_naturals_without_a_finite_id_claim(self):
        s=Specializer(self.f["types"],self.context);key=self.f["ids"]["[]fixture/policy.Line"]
        ty,zero=s.carrier(key);self.assertIn('Std.Nat',json.dumps(ty));self.assertIn('Std.Nat.zero',json.dumps(zero))
        self.assertEqual(s.source(key)["weight_units"],37)

    def test_global_nonplain_error_payload_excludes_error_backing_cells(self):
        s=Specializer(self.f["types"],self.context)
        self.assertFalse(s.source(self.f["ids"]["error"])["plain"])
        with self.assertRaises(ValueError):s.specialize([{"role":"source","source_type_id":self.f["ids"]["[]error"]}])

    def test_plain_error_universe_allows_error_cells_and_cannot_cross_instances(self):
        t,r,i,e=plain_error_candidate();s=Specializer(t,self.context)
        self.assertTrue(s.source(self.f["ids"]["error"])["plain"]);self.assertEqual(len(i["entries"]),43)
        with self.assertRaises(ValueError):validate_expansion(t,self.context,r,self.i,e)

    def test_every_error_case_needs_the_captured_signature(self):
        t=self.changed(lambda t:t.update(error_implementers=[]))
        with self.assertRaises(ValueError):Specializer(t,self.context)

    def test_zero_array_does_not_hide_a_recursive_named_element(self):
        amount=self.f["ids"]["fixture/policy.Amount"]
        def mutate(t):
            next(r for r in t["definitions"] if r["id"]==amount)["underlying"]=amount
            n=source_seal("source_type",{"kind":"array","length":0,"element":amount});t["identity_nodes"].append(n);t["identity_nodes"].sort(key=lambda n:n["id"])
            t["definitions"].append({"id":n["id"],"kind":"array","length":0,"element":amount});t["definitions"].sort(key=lambda r:r["id"])
        t=self.changed(mutate);key=next(r["id"] for r in t["definitions"] if r["kind"]=="array" and r["length"]==0 and r["element"]==amount)
        with self.assertRaises(ValueError):Specializer(t,self.context).source(key)

    def test_all_inactive_error_payload_edges_are_checked_for_cycles(self):
        payload=self.f["ids"]["fixture/policy.SliceError"];root=self.f["ids"]["fixture/policy.Root"]
        def mutate(t):next(r for r in t["definitions"] if r["id"]==payload)["underlying"]=root
        with self.assertRaises(ValueError):Specializer(self.changed(mutate),self.context).source(self.f["ids"]["error"])

    def test_joint_result_order_and_arity_are_explicit(self):
        a=self.f["ids"]["int64"];b=self.f["ids"]["string"];ids=[]
        for seq in ([a,b],[b,a]):
            i,_=Specializer(self.f["types"],self.context).specialize([{"role":"joint","result_types":seq}]);ids.append(i["roots"])
        self.assertNotEqual(*ids)
        with self.assertRaises(ValueError):Specializer(self.f["types"],self.context).specialize([{"role":"joint","result_types":[a]*17}])

    def test_fixed_go_cannot_request_practical_specialization(self):
        with self.assertRaises(ValueError):Specializer(self.f["types"],linked_contexts()["contexts"]["go_fixed"])

    def test_unknown_payload_and_rehashed_expansion_do_not_change_the_recipe(self):
        t=self.changed(lambda t:t.update(caller_adapter=[]))
        with self.assertRaises(ValueError):Specializer(t,self.context)
        e=copy.deepcopy(self.e);e["declarations"][1]["value"]=bit(True)
        e["content_sha256"]=digest(GO["foundation_expansion"][1],{k:v for k,v in e.items() if k!="content_sha256"})
        with self.assertRaises(ValueError):validate_expansion(self.f["types"],self.context,self.f["roots"],self.i,e)

    def test_ordinary_ast_rejects_axioms_free_variables_and_inactive_fields(self):
        for expr in ({"kind":"axiom","name":"caller"},{"kind":"var","name":"free"},{"kind":"sort0","levels":[1]}):
            with self.assertRaises(ValueError):validate_expr(expr)

    def test_zero_recipe_and_universal_projection_members_are_present(self):
        claims=[d for d in self.e["declarations"] if d["kind"]=="theorem"]
        self.assertEqual(len(claims),46)
        self.assertEqual(sum(d["name"].endswith("ZeroRecipe") for d in claims),42)
        for d in claims:validate_expr(d["ty"]);validate_expr(d["value"])

    def test_complete_closure_is_deterministic_without_root_order_authority(self):
        i,e=Specializer(self.f["types"],self.context).specialize(list(reversed(self.f["roots"])))
        self.assertEqual(canonical(i),canonical(self.i));self.assertEqual(canonical(e),canonical(self.e))
        with self.assertRaises(ValueError):Specializer(self.f["types"],self.context).specialize(self.f["roots"]+[self.f["roots"][0]])

    def test_candidate_identities_do_not_reuse_reserved_names(self):
        from identity_catalog import generate
        c=generate();ids={r["identity"] for r in c["identities"]};domains={r["domain"] for r in c["identities"]}
        for row in schemas()["new_identities"]:
            self.assertNotIn(row["schema"],ids|set(c["reserved_retired"]["identities"]))
            self.assertNotIn(row["domain"],domains|set(c["reserved_retired"]["domains"]))

    def test_both_actual_checkers_and_false_controls_remain_distinct_from_source_proofs(self):
        r=json.loads((RECORD/"observations.json").read_text())
        self.assertEqual(r["outcomes"],{"accepted_zero_axiom":2,"ordinary_core_type_mismatch":6,"legacy":2})
        for key in ("source_proofs_checked","complete_validity_proved","finite_ids_proved","production_checker_changed","profile_activated"):self.assertIs(r[key],False)


if __name__=="__main__":unittest.main()
