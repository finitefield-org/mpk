import copy
from unittest import TestCase, main
from admission_closure import classify, catalog, PENDING
from admission_catalog import generate
from admission_vectors import CASES
from run_source_admission import native_records, expected


class SourceAdmissionTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.rows = {c["id"]: (c, n) for c, (_, n) in zip(CASES, native_records())}

    def result(self, identity):
        c, n = self.rows[identity]
        return classify(c["source"], n, c["root"])

    def test_all_original_source_vectors(self):
        for c, n in self.rows.values():
            with self.subTest(case=c["id"]): expected(c, classify(c["source"], n, c["root"]))

    def test_catalog_covers_pinned_ast_and_builtin_universe(self):
        self.assertEqual(catalog(), generate())
        self.assertEqual(len(catalog()["ast_nodes"]), 56)
        self.assertEqual({r["name"] for r in catalog()["builtins"] if r["disposition"] == "conditional_candidate"}, {"append","cap","copy","len","make"})

    def test_unselected_map_and_initialization_do_not_enter_proof(self):
        r = self.result("unselected-map-and-init")
        self.assertEqual([f["name"] for f in r["closure"]], ["Root"])
        self.assertEqual(r["typecheck_only"], ["init","Other"])
        self.assertEqual(r["pending"], PENDING)

    def test_error_conversion_uses_signature_without_calling_body(self):
        r = self.result("error-box-signature-only")
        self.assertEqual(r["status"], "candidate_unproved")
        self.assertEqual(r["typecheck_only"], ["fixture/policy.Code.Error"])
        called = self.result("called-error-panic")
        self.assertTrue(any(d["code"] == "BUILTIN_panic" and d["path"] == ["Root","fixture/policy.Code.Error"] for d in called["diagnostics"]))

    def test_hidden_ambient_access_has_transitive_path(self):
        r = self.result("helper-ambient")
        self.assertTrue(any(d["code"] == "AMBIENT_STATE" and d["path"] == ["Root","Helper"] for d in r["diagnostics"]))

    def test_recursive_and_false_branch_calls_are_retained(self):
        r = self.result("call-cycle")
        self.assertTrue(any(d["code"] == "CALL_CYCLE" and d["path"] == ["Root","Helper","Root"] for d in r["diagnostics"]))
        r = self.result("unreachable-cycle")
        self.assertTrue(any(d["code"] == "CALL_CYCLE" and d["path"] == ["Root","Helper","Helper"] for d in r["diagnostics"]))

    def test_generic_call_retains_its_static_body_before_rejection(self):
        r = self.result("generic-function")
        self.assertEqual([f["name"] for f in r["closure"]], ["Root","Helper"])
        self.assertTrue(any(d["code"] == "GENERIC_CALL" for d in r["diagnostics"]))

    def test_methods_use_qualified_receiver_declaration(self):
        for identity in ("direct-value-method","method-expression"):
            self.assertEqual([f["name"] for f in self.result(identity)["closure"]], ["Root","fixture/policy.Code.Add"])

    def test_original_unselected_type_error_is_not_pruned(self):
        r = self.result("unselected-type-error")
        self.assertEqual(r["status"], "type_error"); self.assertEqual(r["closure"], [])
        self.assertTrue(r["diagnostics"])

    def test_utf8_spans_are_physical_bytes(self):
        c, n = self.rows["utf8-physical-span"]
        fn = n["functions"][0]
        self.assertEqual(fn["start_byte"], c["source"].encode().index(b"func Root"))
        self.assertNotEqual(fn["start_byte"], c["source"].index("func Root"))
        self.assertTrue(any(c["source"].encode()[r["start_byte"]:r["end_byte"]] == "額".encode() for r in fn["nodes"]))

    def test_effective_language_table_is_retained(self):
        self.assertEqual({n["language"] for _, n in self.rows.values()}, {"go1.22","go1.23","go1.24","go1.25"})

    def test_source_crossing_cannot_reuse_native_facts(self):
        c, n = self.rows["ordinary-domain"]
        with self.assertRaises(ValueError): classify(c["source"]+"\n", n, c["root"])

    def test_wrong_target_or_unknown_ast_cannot_be_classified(self):
        c, original = self.rows["ordinary-domain"]
        for field, value in (("int_bits",32),("language","go1.26")):
            n = copy.deepcopy(original); n[field] = value
            with self.assertRaises(ValueError): classify(c["source"],n,c["root"])
        n = copy.deepcopy(original);n["functions"][0]["nodes"][0]["kind"] = "FutureSyntax"
        with self.assertRaises(ValueError): classify(c["source"],n,c["root"])

    def test_callee_and_selected_root_bindings_must_exist(self):
        c, original = self.rows["parenthesized-call"]
        n = copy.deepcopy(original);n["functions"][-1]["calls"] = ["function:missing"]
        with self.assertRaises(ValueError): classify(c["source"],n,c["root"])
        with self.assertRaises(ValueError): classify(c["source"],original,"Missing")
        c, n = self.rows["unselected-map-and-init"]
        with self.assertRaises(ValueError): classify(c["source"],n,"init")

    def test_candidate_is_never_source_or_claim_acceptance(self):
        for identity in ("fresh-slice-builtins","error-entry-needs-universe","nested-control"):
            r = self.result(identity)
            self.assertEqual(r["status"],"candidate_unproved"); self.assertEqual(r["pending"],PENDING)


if __name__ == "__main__": main()
