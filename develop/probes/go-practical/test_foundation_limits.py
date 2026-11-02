"""Authority, raw-byte and cross-artifact regressions for private packaging."""
import copy
import unittest
from pathlib import Path
from unittest.mock import patch
from foundation_limits_protocol import (FoundationError, RECORD, ROOT, package, reference, schema_draft, core_interfaces)
from foundation_limits_vectors import model, cases, apply_row, linked_contexts, hash_vectors
from context_protocol import ContextError
from context_vectors import contexts, assign, rehash_registry
from identity_catalog import GO, SHARED
from identity_wire import canonical, digest
from structural_limit_catalog import generate
from run_foundation_limits import observations


class FoundationLimitsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.p=model();cls.data=cls.p.compiled

    def test_full_expected_configuration_corpus(self):
        observed=observations();self.assertEqual(len(observed),102)
        self.assertEqual(sum(x["result"]=="configuration_candidate" for x in observed),16)

    def test_all_sixty_four_distinct_numeric_dimensions_retained(self):
        actual=self.data["limits"]["counters"]
        self.assertEqual(len(actual),64)
        self.assertEqual({c["id"]:c["maximum"] for c in actual},{c["id"]:c["maximum"] for c in generate()["counters"]})
        self.assertEqual([c["id"] for c in actual],sorted({c["id"] for c in actual}))

    def test_limits_units_separate_bytes_bits_wall_and_carrier(self):
        rows={c["id"]:c for c in self.data["limits"]["counters"]}
        expected={"semantic.string_bytes":"bytes","semantic.source_word_bits":"bits","weighted.value_units":"abstract_carrier_units","proof.construction_wall_ns":"monotonic_nanoseconds","shared.aggregate_type_nesting":"depth"}
        self.assertEqual({k:rows[k]["unit"] for k in expected},expected)
        self.assertEqual(rows["capture.typed_syntax_nodes_across_included_packages"]["maximum"],4194304)

    def test_resource_and_domain_overflow_cannot_be_reported_as_success(self):
        rows=self.data["limits"]["counters"]
        self.assertEqual({c["exceeded"] for c in rows if c["category"]=="proof_resources"},{"proof_pending_resource"})
        self.assertEqual({c["exceeded"] for c in rows if c["category"] in {"semantic_domain","carrier_accounting"}},{"domain_obligation_unproved"})

    def test_rehashed_maximum_does_not_create_a_registered_limit(self):
        changed=assign(self.data["limits"],["counters",0,"maximum"],999999999)
        changed["content_sha256"]=digest(GO["limits"][1],{k:v for k,v in changed.items() if k!="content_sha256"})
        with self.assertRaises(FoundationError):self.p.validate_record("limits",changed)

    def test_weights_include_inactive_payloads_and_complete_byte_arrays(self):
        weights=self.data["limits"]["weights"]
        self.assertEqual(weights["byte_string"],257)
        self.assertEqual(weights["closed_error"],"1+sum(W(each global-U payload))")
        for key in ("closed_error","byte_string"):
            changed=copy.deepcopy(self.data["limits"]);changed["weights"][key]=1
            changed["content_sha256"]=digest(GO["limits"][1],{k:v for k,v in changed.items() if k!="content_sha256"})
            with self.assertRaises(FoundationError):self.p.validate_record("limits",changed)

    def test_joint_core_interfaces_match_exact_retained_source_closure(self):
        interfaces=core_interfaces();self.assertEqual([i["name"] for i in interfaces],["Std.Nat.induct","Std.Bool.recType"])
        self.assertEqual(len(self.data["definitions"]["mirrored_checker_inputs"]),77)
        for interface in interfaces:
            for m in interface["sources"]:
                from run_proof_pilot import sha
                self.assertEqual(sha((ROOT/m["path"]).read_bytes()),m["sha256"])

    def test_changing_type_selection_or_iota_after_rehash_rejects(self):
        for row in cases():
            if row["id"] in {"definitions-universe","definitions-equation","definitions-type-selection","definitions-guard"}:
                with self.subTest(id=row["id"]),self.assertRaises(FoundationError):apply_row(row,self.p)

    def test_recipe_inventory_covers_all_logical_operations_and_common_proofs(self):
        defs=self.data["definitions"];families=defs["operation_families"]
        self.assertEqual(len(families),8)
        ids=[op["id"] for f in families for op in f["operations"]]
        self.assertEqual(sorted(ids),[f"GP-OP-{i:02}" for i in range(1,57)])
        self.assertEqual(len(defs["common_required_members"]),6)
        self.assertEqual(defs["representation"],"operation_recipe_inventory_draft")

    def test_source_callable_and_caller_extension_lists_are_empty(self):
        for kind in ("definitions","descriptor"):
            for key in ("source_callable_members","caller_extension_points"):
                self.assertEqual(self.data[kind][key],[])
                changed=assign(self.data[kind],[key],["caller.Helper"])
                with self.assertRaises(FoundationError):self.p.validate_record(kind,changed)

    def test_descriptor_binds_exact_limits_and_definition_refs(self):
        desc=self.data["descriptor"]
        self.assertEqual(desc["limits_ref"],reference(self.data["limits"]))
        self.assertEqual(desc["definitions_ref"],reference(self.data["definitions"]))
        self.assertEqual(self.data["definitions"]["limits_ref"],desc["limits_ref"])

    def test_descriptor_graph_has_no_self_context_or_release_member(self):
        members=self.data["descriptor"]["members"]
        self.assertEqual([Path(m["path"]).name for m in members],["definitions.json","limits.json","semantic-rules.md"])
        self.assertNotIn("profile_registry",self.data["descriptor"])
        self.assertNotIn("release",self.data["descriptor"])

    def test_full_member_bytes_and_final_lf_are_bound(self):
        members=self.data["descriptor"]["members"]
        provided={m["path"]:(ROOT/m["path"]).read_bytes() for m in members}
        self.p.validate_package(self.data,provided)
        for key,raw in provided.items():
            self.assertTrue(raw.endswith(b"\n"))
            changed={**provided,key:raw[:-1]}
            with self.subTest(path=key),self.assertRaises(FoundationError):self.p.validate_package(self.data,changed)

    def test_member_set_cannot_grow_or_omit(self):
        provided={m["path"]:(ROOT/m["path"]).read_bytes() for m in self.data["descriptor"]["members"]}
        for changed in ({**provided,"caller.json":b"{}\n"},{k:v for k,v in provided.items() if k!=next(iter(provided))}):
            with self.assertRaises(FoundationError):self.p.validate_package(self.data,changed)

    def test_registered_member_symlink_is_rejected_before_read(self):
        with patch.object(Path,"is_symlink",return_value=True),patch.object(Path,"read_bytes",side_effect=AssertionError("must not read")):
            with self.assertRaises(FoundationError):self.p.validate_members(self.data["descriptor"])

    def test_fresh_registry_contexts_keep_all_old_policies(self):
        cs=self.p.contextual_cases()
        for c in cs.values():self.p.contexts.validate_context(c)
        self.assertEqual(cs["rust32"]["semantic_parameters"]["value"]["pointer_width"],32)
        self.assertEqual(cs["rust64"]["semantic_parameters"]["value"]["pointer_width"],64)
        self.assertEqual(sum(c["foundation_descriptor"] is None for c in cs.values()),5) # Rust has two target contexts.

    def test_prior_private_generation_cannot_mix_with_new_registered_refs(self):
        old=contexts();new=self.p.contextual_cases()
        for name in old:
            with self.subTest(name=name),self.assertRaises(ContextError):self.p.contexts.validate_context(old[name])
        self.assertNotEqual(old["go_practical"]["foundation_descriptor"],new["go_practical"]["foundation_descriptor"])

    def test_rehashed_full_registry_with_stale_foundation_still_rejects(self):
        registry=copy.deepcopy(self.p.contexts.compiled);old=contexts()["go_practical"]["foundation_descriptor"]
        next(e for e in registry["profiles"] if e["semantic_profile"]=="mpk.go.practical.v1")["foundation_descriptor"]=old
        registry=rehash_registry(registry)
        with self.assertRaises(ContextError):self.p.contexts.validate_registry(registry)

    def test_all_three_own_hashes_cover_every_active_member(self):
        for root in schema_draft()["roots"]:
            self.assertEqual(root["hash_preimage_fields"],[k for k in root["required_fields"] if k!="content_sha256"])
        for v in hash_vectors():self.assertEqual(bytes.fromhex(v["raw_json_hex"]),bytes.fromhex(v["canonical_hex"]))
        self.assertEqual(len(hash_vectors()),21)

    def test_new_go_context_and_request_hashes_bind_new_limits_and_descriptor(self):
        linked=linked_contexts();go=linked["contexts"]["go_practical"]
        self.assertEqual(go["semantic_parameters"]["value"]["limits_sha256"],self.data["limits"]["content_sha256"])
        self.assertEqual(go["foundation_descriptor"],reference(self.data["descriptor"]))
        for request in linked["requests"].values():self.p.contexts.validate_request(request)


if __name__=="__main__":unittest.main()
