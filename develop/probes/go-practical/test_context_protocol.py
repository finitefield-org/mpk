"""Cross-family, authority and retained-policy regressions for the private draft."""
import unittest
from context_protocol import Protocol, ContextError, registration_fixture, go_selection as check_selection, raw_request, legacy_projection, schemas, same
from context_vectors import cases, contexts, selections, go_root, go_selection, assign, rehash_registry, legacy_queries
from identity_catalog import SHARED
from identity_wire import canonical, digest, parse
from run_context_protocol import RECORD, policy_from_outputs, observe, outcomes
from check_scope_report import strict_json


class ContextProtocolTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.rows=cases();cls.by_id={r["id"]:r for r in cls.rows};cls.contexts=contexts();cls.selections=selections()
        cls.outputs=strict_json((RECORD/"legacy-observations.json").read_bytes())
        cls.policy=staticmethod(policy_from_outputs(legacy_queries(),cls.outputs));cls.protocol=Protocol(cls.policy)

    def test_exact_corpus_expected_outcomes(self):
        observed=[observe(self.protocol,r) for r in self.rows]
        self.assertEqual(len(observed),152)
        self.assertEqual(sum(r["result"]!="configuration_rejected" for r in observed),24)
        self.assertEqual(outcomes(observed)["context"],{"context_candidate":6,"configuration_rejected":61})

    def test_actual_legacy_policy_keeps_both_rust_targets(self):
        for name in ("go_fixed","rust64","rust32","csharp","java"):
            c=self.contexts[name];s=self.selections[name]
            self.assertEqual(self.policy(legacy_projection(c,s)),{"context_accepted":True,"selection_accepted":True})
        self.assertEqual(self.contexts["rust32"]["semantic_parameters"]["value"]["pointer_width"],32)

    def test_actual_legacy_parameter_rejections(self):
        for r in self.rows:
            if r["id"].startswith(("legacy-extra-params-","legacy-params-")):
                with self.subTest(id=r["id"]):self.assertFalse(self.policy(legacy_projection(r["value"]))["context_accepted"])

    def test_actual_legacy_selection_rejections(self):
        for r in self.rows:
            if r["id"].startswith("legacy-selection-"):
                c=r["value"]["semantic_context"];s=r["value"]["selection"]
                with self.subTest(id=r["id"]):self.assertEqual(self.policy(legacy_projection(c,s)),{"context_accepted":True,"selection_accepted":False})

    def test_rehashed_registry_cannot_add_authority(self):
        registry=registration_fixture()["registry"]
        changed=rehash_registry(assign(registry,["profiles",0,"contracts","vir"],"mpk.profile.vir.caller.v2"))
        self.assertEqual(changed["registry_sha256"],digest(SHARED["registry"][1],{k:v for k,v in changed.items() if k!="registry_sha256"}))
        with self.assertRaises(ContextError):self.protocol.validate_registry(changed)

    def test_language_only_dispatch_rejects_fixed_entry(self):
        practical=self.contexts["go_practical"];fixed=self.contexts["go_fixed"]
        self.assertEqual(practical["source_language"],fixed["source_language"])
        with self.assertRaises(ContextError):self.protocol.validate_context(assign(practical,["profile_entry_sha256"],fixed["profile_entry_sha256"]))

    def test_all_context_fields_are_required(self):
        c=self.contexts["go_practical"]
        for field in c:
            altered={k:v for k,v in c.items() if k!=field}
            with self.subTest(field=field),self.assertRaises(ContextError):self.protocol.validate_context(altered)

    def test_foundation_is_profile_bound_and_required_even_null(self):
        foundation=self.contexts["go_practical"]["foundation_descriptor"]
        for name,c in self.contexts.items():
            if name=="go_practical":continue
            self.assertIsNone(c["foundation_descriptor"])
            with self.subTest(profile=name),self.assertRaises(ContextError):self.protocol.validate_context(assign(c,["foundation_descriptor"],foundation))
        with self.assertRaises(ContextError):self.protocol.validate_context(assign(self.contexts["go_practical"],["foundation_descriptor"],None))

    def test_foundation_hash_does_not_admit_a_caller_bundle(self):
        c=self.contexts["go_practical"]
        with self.assertRaises(ContextError):self.protocol.validate_context(assign(c,["foundation_descriptor","definitions"],[]))
        self.assertIs(registration_fixture()["foundation_fixture"]["complete_definition_bundle"],False)

    def test_exact_go_parameters_and_no_numeric_override(self):
        for r in self.rows:
            if r["id"].startswith("context-reject-") and "semantic_parameters" in r["value"]:
                if not same(r["value"]["semantic_parameters"],self.contexts["go_practical"]["semantic_parameters"]):
                    with self.subTest(id=r["id"]),self.assertRaises(ContextError):self.protocol.validate_context(r["value"])
        with self.assertRaises(ContextError):self.protocol.validate_context(assign(self.contexts["go_practical"],["semantic_parameters","value","limits"],{}))

    def test_raw_method_and_alias_selectors_need_later_source_resolution(self):
        for name in ("request-go-method","request-go-receiver-alias-syntax","request-go-unicode-name"):
            request=self.by_id[name]["value"];self.protocol.validate_request(request)
        # Distinct raw names can resolve to one method. This layer cannot decide that.
        roots=[go_root(receiver={"package":"example.test/quote","type_name":name}) for name in ("Ledger","LedgerAlias")]
        check_selection(go_selection(sorted(roots,key=canonical)))
        self.assertTrue(any("alias resolution" in item for item in schemas()["remaining"]))

    def test_raw_roots_unique_and_canonically_sorted(self):
        check_selection(go_selection(sorted([go_root("A"),go_root("B")],key=canonical)))
        for roots in ([go_root(),go_root()],[go_root("B"),go_root("A")]):
            with self.assertRaises(ContextError):check_selection(go_selection(roots))

    def test_explicit_root_count_boundaries(self):
        roots=sorted([go_root("F"+str(i)) for i in range(8192)],key=canonical)
        check_selection(go_selection(roots))
        with self.assertRaises(ContextError):check_selection(go_selection(sorted(roots+[go_root("Beyond")],key=canonical)))
        with self.assertRaises(ContextError):check_selection(go_selection([]))

    def test_package_init_and_method_init_are_distinct(self):
        check_selection(go_selection([go_root("init",{"package":"example.test/quote","type_name":"Ledger"})]))
        for name in ("init","_","return"):
            with self.subTest(name=name),self.assertRaises(ContextError):check_selection(go_selection([go_root(name)]))

    def test_valid_distinct_contexts_cannot_link(self):
        left=self.contexts["rust32"];right=self.contexts["rust64"]
        self.protocol.validate_context(left);self.protocol.validate_context(right)
        with self.assertRaises(ContextError):self.protocol.linkage(left,right)
        self.assertFalse(same({"revision":1},{"revision":True}))

    def test_request_hash_binds_complete_context_and_selection(self):
        c=self.contexts["go_practical"];original=raw_request(c,go_selection());changed=raw_request(c,go_selection([go_root("Another")]))
        self.protocol.validate_request(original);self.protocol.validate_request(changed)
        self.assertNotEqual(original["request_sha256"],changed["request_sha256"])
        changed["request_sha256"]=original["request_sha256"]
        with self.assertRaises(ContextError):self.protocol.validate_request(changed)

    def test_unknown_nested_fields_do_not_hide_in_a_new_request_hash(self):
        c=self.contexts["go_practical"];selection=assign(go_selection(),["value","roots",0,"unchecked"],True)
        with self.assertRaises(ContextError):self.protocol.validate_request(raw_request(c,selection))

    def test_strict_wire_rejects_duplicate_and_untyped_number_forms(self):
        for r in self.rows:
            if r["kind"]=="wire":
                with self.subTest(id=r["id"]),self.assertRaises(ValueError):parse(bytes.fromhex(r["value"]))

    def test_unobserved_legacy_policy_is_fatal_not_an_accepted_fallback(self):
        query=legacy_projection(self.contexts["rust64"]);query["context"]["semantic_parameters"]["value"]["caller_added"]=True
        with self.assertRaises(AssertionError):self.policy(query)

    def test_all_six_root_shapes_have_complete_preimage_fields(self):
        roots=schemas()["roots"];self.assertEqual(len(roots),6)
        for root in roots:
            with self.subTest(schema=root["schema"]):
                self.assertEqual(set(root["field_types"]),set(root["required_fields"]))
                self.assertEqual(root["optional_fields"],[])
                self.assertEqual(root["hash_preimage_fields"],[k for k in root["required_fields"] if k!=root["hash_field"]])


if __name__=="__main__":unittest.main()
