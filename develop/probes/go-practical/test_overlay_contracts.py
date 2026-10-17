import copy
from unittest import TestCase, main
from check_scope_report import strict_json
from overlay_contracts import BOOL, NAT, INT, Context, Expressions, Types, OverlayError, ProbeResourceLimit, normalize, OPERATORS
from overlay_vectors import e, nat, integer, inp, res, word, truth, local, base, selector, vectors, capture_id
from run_overlay_probes import RECORD, HERE, retained_check


class OverlayTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.facts = strict_json((RECORD / "native-facts.json").read_bytes())
        cls.scopes = next(f for f in cls.facts["functions"] if f["name"] == "Scopes")
        cls.types = Types(cls.facts, ["fixture/policy.RejectCode"])
        cls.ctx = Context("header", tuple(cls.scopes["inputs"]), tuple(cls.scopes["results"]), tuple(cls.scopes["loops"][0]["bindings"]), "for")

    def check_expr(self, value, expected, ctx=None):
        n, t = Expressions(self.types).check(value, ctx or self.ctx)
        self.assertEqual(t, expected)
        return n

    def reject_expr(self, value, reason, ctx=None):
        with self.assertRaisesRegex(OverlayError, reason):
            Expressions(self.types).check(value, ctx or self.ctx)

    def test_retained_actual_matrix(self):
        self.assertEqual(len(retained_check()["outcomes"]), 24)

    def test_physical_utf8_spans(self):
        source = (HERE / "overlay-fixture.go.txt").read_bytes()
        for f in self.facts["functions"]:
            self.assertTrue(source[f["start_byte"]:f["end_byte"]].startswith(b"func "))
            for l in f["loops"]:
                self.assertTrue(source[l["start_byte"]:l["end_byte"]].startswith(b"for "))
                self.assertEqual(source[l["end_byte"]-1:l["end_byte"]], b"}")
        self.assertIn("条件評価".encode(), source)

    def test_native_header_visibility(self):
        loops = self.scopes["loops"]
        self.assertEqual([l["ordinal"] for l in loops], [0, 1, 2, 3, 4])
        names = [{b["name"] for b in l["bindings"]} for l in loops]
        self.assertTrue(all("ambient" not in n for n in names))
        self.assertIn("i", names[0]); self.assertNotIn("j", names[0]); self.assertNotIn("after", names[0])
        self.assertIn("i", names[1]); self.assertIn("j", names[1])
        self.assertNotIn("index", names[2]); self.assertNotIn("value", names[2])
        self.assertIn("index", names[3]); self.assertIn("value", names[3]); self.assertIn("k", names[3])
        self.assertIn("index", names[4]); self.assertIn("value", names[4])
        totals = [next(b["id"] for b in l["bindings"] if b["name"] == "total") for l in loops]
        self.assertNotEqual(totals[0], totals[1]); self.assertEqual(totals[0], totals[2])

    def test_alias_retains_one_nominal_identity(self):
        n = self.check_expr(word("fixture/policy.Money", 7), "fixture/policy.Amount")
        self.assertEqual(n["type"], "fixture/policy.Amount")
        self.reject_expr(e("eq", word("fixture/policy.Amount", 7), word("fixture/policy.OtherAmount", 7)), "nominal/comparison")

    def test_word_overflow_is_configuration_failure(self):
        self.reject_expr(word("uint8", 256), "representability")
        self.check_expr(word("int64", -9223372036854775808), "int64")

    def test_signed_and_unsigned_bridges(self):
        for prefix, out in [("nat", NAT), ("int", INT)]:
            self.check_expr(e(prefix+".from_unsigned", word("uint8", 255)), out)
            self.check_expr(e(prefix+".from_signed", word("int64", -1)), out)
            self.reject_expr(e(prefix+".from_unsigned", word("int64", 1)), "bridge domain")
        # The negative Nat conversion is type-valid, with a definedness proof
        # still required. Type checking does not certify nonnegativity.

    def test_shift_full_count_type(self):
        self.check_expr(e("word.shl", word("uint8", 1), word("int64", 1000)), "uint8")
        self.check_expr(e("word.shr", word("int64", -1), word("int", -1)), "int64")
        n = self.check_expr(e("word.shl", word("byte", 1), word("uint64", 18446744073709551615)), "uint8")
        self.assertEqual(n["args"][1]["value"], "18446744073709551615")

    def test_error_equality_excluded(self):
        self.reject_expr(e("eq", local("err"), local("err")), "error/slice equality")
        self.check_expr(e("is_nil", local("err")), BOOL)

    def test_blank_field_is_not_an_observation(self):
        self.reject_expr(e("field", value=e("source.zero", type="fixture/policy.Blank"), field="_"), "nonpromoted field")

    def test_package_variable_cannot_be_a_header_local(self):
        self.reject_expr(local("ambient"), "live lexical header binding")

    def test_old_freezes_entire_observation(self):
        self.check_expr(e("old", value=e("element", inp(0), nat(0))), "int64")
        self.reject_expr(e("old", value=local("i")), "local unavailable")
        self.reject_expr(e("old", value=e("old", value=inp(0))), "nested entry")

    def test_predicate_is_logical_bool(self):
        self.reject_expr(e("bool.and", inp(2), truth()), "logical domain")
        self.check_expr(e("bool.and", e("bool.value", inp(2)), truth()), BOOL)

    def test_branch_types_checked_even_if_unreached(self):
        self.reject_expr(e("if", truth(), nat(0), integer(0)), "conditional branches")
        self.check_expr(e("if", truth(), nat(0), e("nat.from_signed", word("int64", -1))), NAT)

    def test_result_projection_phase_and_json_number(self):
        self.reject_expr(res(1), "only at same invocation")
        ctx = Context("ensures", self.ctx.inputs, self.ctx.results)
        self.check_expr(res(1), "error", ctx)
        self.reject_expr(res(True), "JSON integer", ctx)

    def test_bounded_quantifier_scopes_and_old(self):
        body = e("eq", e("old", value=e("element", inp(0), e("bound", name="j"))), word("int64", 0))
        self.check_expr(e("forall", name="j", lower=nat(0), upper=e("length", inp(0)), body=body), BOOL)
        self.reject_expr(e("bound", name="j"), "logical binder")
        nested = e("exists", name="j", lower=nat(0), upper=nat(1), body=truth())
        self.reject_expr(e("forall", name="j", lower=nat(0), upper=nat(1), body=nested), "fresh logical")

    def test_fresh_write_permission_cannot_be_asserted(self):
        c = base(); c["modifies"] = ["lines"]
        with self.assertRaisesRegex(OverlayError, "read-only"):
            normalize(c, self.facts, "a"*64, ["fixture/policy.RejectCode"])

    def test_nonplain_error_universe_cannot_hide_in_cells(self):
        c = next(c for c in vectors(self.facts) if c["id"] == "nonplain-global-universe-for-error-cells")
        with self.assertRaisesRegex(OverlayError, "plain backing cell"):
            normalize(c["overlay"], self.facts, "a"*64, c["universe"])

    def test_capture_dag_has_no_normalized_self_hash(self):
        c = vectors(self.facts)[0]; h = capture_id(c, self.facts["source_sha256"])
        n = normalize(c["overlay"], self.facts, h, c["universe"])
        self.assertEqual(n["capture_context_id"], h)
        self.assertNotIn("capture_context_id", c["overlay"])
        other = copy.deepcopy(c); other["overlay"]["termination"] = "partial"
        self.assertNotEqual(capture_id(other, self.facts["source_sha256"]), h)

    def test_private_resource_exhaustion_is_not_type_failure(self):
        with self.assertRaises(ProbeResourceLimit):
            Expressions(self.types).check(e("nat.literal", value="1"*4097), self.ctx)

    def test_every_closed_operator_has_a_typed_witness(self):
        a, b, yes = nat(1), integer(2), truth()
        signed, unsigned = word("int64", 4), word("uint8", 5)
        string = e("bytes.literal", type="string", value="00ff")
        record = e("source.zero", type="fixture/policy.Line")
        error = local("err")
        rows = [("bool.literal", yes, BOOL), ("nat.literal", a, NAT), ("int.literal", b, INT),
                ("word.literal", signed, "int64"), ("bytes.literal", string, "string"), ("source.zero", record, "fixture/policy.Line"),
                ("input", inp(0), "[]int64"), ("local", local("i"), "int"), ("old", e("old", value=inp(0)), "[]int64"),
                ("if", e("if", yes, a, a), NAT), ("word.convert", e("word.convert", type="uint8", value=signed), "uint8"),
                ("bool.value", e("bool.value", inp(2)), BOOL), ("field", e("field", value=record, field="Amount"), "fixture/policy.Amount"),
                ("length", e("length", inp(0)), NAT), ("capacity", e("capacity", inp(0)), NAT), ("is_nil", e("is_nil", inp(0)), BOOL),
                ("byte_at", e("byte_at", string, nat(0)), "uint8"), ("element", e("element", inp(0), nat(0)), "int64"),
                ("error.is", e("error.is", value=error, case="fixture/policy.RejectCode"), BOOL),
                ("error.payload", e("error.payload", value=error, case="fixture/policy.RejectCode"), "fixture/policy.RejectCode"),
                ("sequence.equal", e("sequence.equal", inp(0), inp(0)), BOOL)]
        for op in ("bool.not",): rows.append((op, e(op, yes), BOOL))
        rows += [("nat.to_int", e("nat.to_int", a), INT), ("int.neg", e("int.neg", b), INT)]
        for op in ("bool.and", "bool.or", "bool.implies"): rows.append((op, e(op, yes, yes), BOOL))
        for op in ("nat.add", "nat.mul", "nat.sub"): rows.append((op, e(op, a, a), NAT))
        for op in ("int.add", "int.sub", "int.mul"): rows.append((op, e(op, b, b), INT))
        for op in ("eq", "ne", "lt", "le", "gt", "ge"): rows.append((op, e(op, a, a), BOOL))
        for op in ("word.neg", "word.not"): rows.append((op, e(op, signed), "int64"))
        for op in ("word.add", "word.sub", "word.mul", "word.and", "word.or", "word.xor", "word.andnot", "word.div", "word.rem", "word.shl", "word.shr"):
            rows.append((op, e(op, signed, signed), "int64"))
        for op, val, out in [("nat.from_signed", signed, NAT), ("nat.from_unsigned", unsigned, NAT), ("int.from_signed", signed, INT), ("int.from_unsigned", unsigned, INT)]:
            rows.append((op, e(op, val), out))
        for op in ("bytes.prefix", "bytes.suffix", "bytes.concat"):
            rows.append((op, e(op, string, string), "string" if op == "bytes.concat" else BOOL))
        rows.append(("bytes.slice", e("bytes.slice", string, nat(0), nat(1)), "string"))
        rows += [("slice.view2", e("slice.view2", inp(0), nat(0), nat(1)), "[]int64"), ("slice.view3", e("slice.view3", inp(0), nat(0), nat(1), nat(1)), "[]int64")]
        for op, value, out in [("forall", yes, BOOL), ("exists", yes, BOOL), ("nat.sum", a, NAT), ("int.sum", b, INT)]:
            rows.append((op, e(op, name="j", lower=nat(0), upper=nat(1), body=value), out))
        observed = set()
        for op, value, out in rows:
            with self.subTest(operator=op): self.check_expr(value, out); observed.add(op)
        header = self.scopes["loops"][2]
        rctx = Context("header", self.ctx.inputs, self.ctx.results, tuple(header["bindings"]), "range")
        for op in ("range.index", "range.length"):
            self.check_expr(e(op), NAT, rctx); observed.add(op)
        self.check_expr(res(0), "fixture/policy.Amount", Context("ensures", self.ctx.inputs, self.ctx.results)); observed.add("result")
        self.check_expr(e("bound", name="j"), NAT, Context("header", self.ctx.inputs, self.ctx.results, variables=(("j", NAT),))); observed.add("bound")
        self.assertEqual(observed, OPERATORS)


if __name__ == "__main__": main()
