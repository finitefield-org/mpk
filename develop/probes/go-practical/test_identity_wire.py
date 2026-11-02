import copy
import hashlib
from unittest import TestCase,main
from identity_catalog import generate,IDENTITY_SHAPES
from identity_vectors import fixture,hash_vectors,SOURCE
from identity_wire import IdentityError,canonical,parse,digest,seal,validate_node,validate_graph,resolve_alias
from identity_migration import proposal


class IdentityWireTests(TestCase):
    def setUp(self):self.f=fixture()

    def node(self,family):return next(n for n in self.f["nodes"] if n["schema"]==IDENTITY_SHAPES[family][0])

    def changed(self,family,patch,predicate=lambda n:True):
        mapping={};nodes=[];changed=False
        def remap(value):
            if type(value) is str:return mapping.get(value,value)
            if type(value) is list:return [remap(v) for v in value]
            if type(value) is dict:return {k:remap(v) for k,v in value.items()}
            return value
        for original in self.f["nodes"]:
            row=remap({k:v for k,v in original.items() if k not in {"schema","id"}})
            actual=next(k for k,(schema,_,_) in IDENTITY_SHAPES.items() if schema==original["schema"])
            if actual==family and predicate(original) and not changed:row.update(patch);changed=True
            n=seal(actual,row);mapping[original["id"]]=n["id"];nodes.append(n)
        return nodes,[mapping[r] for r in self.f["roots"]]

    def test_closed_identity_graph(self):
        self.assertEqual(self.f["result"]["identity_nodes"],26)
        self.assertEqual(self.f["result"]["roots"],7)
        self.assertIn("no registry/source",self.f["result"]["scope"])

    def test_new_names_and_domains_do_not_reuse_csharp_reservations(self):
        c=generate();ids=[r["identity"] for r in c["identities"] if r["identity"]];domains=[r["domain"] for r in c["identities"] if r["domain"]]
        self.assertEqual((len(ids),len(domains)),(101,41))
        self.assertFalse(set(ids)&set(c["reserved_csharp"]["identities"]));self.assertFalse(set(domains)&set(c["reserved_csharp"]["domains"]))
        self.assertNotIn("mpk.csharp.practical.v1",c["profiles_at_v1_activation"])

    def test_private_migration_routes_all_predecessor_files_without_claiming_review(self):
        p=proposal();self.assertEqual(len(p["files"]),143)
        self.assertTrue(all("remains required" in r["review_status"] for r in p["files"]))
        self.assertTrue(all("retain_reserved_inactive" in r["candidate_disposition"] for r in p["files"] if "csharp_practical" in r["path"]))

    def test_every_complete_node_preimage_is_in_hash_vectors(self):
        rows=hash_vectors();self.assertEqual(len(rows),44)
        self.assertEqual(sum(r["expected_accepted"] for r in rows),34)
        self.assertEqual([r["digest"] for r in rows[:26]],[n["id"] for n in self.f["nodes"]])

    def test_utf16_key_order_and_unicode_non_normalization(self):
        self.assertEqual(canonical({"\ue000":1,"😀":2}),'{"😀":2,"\ue000":1}'.encode())
        self.assertNotEqual(digest("MPK-TEST-1.0",{"name":"é"}),digest("MPK-TEST-1.0",{"name":"e\u0301"}))

    def test_strict_json_rejects_duplicates_floats_surrogates_and_extra_roots(self):
        for r in hash_vectors():
            if not r["expected_accepted"] and r["id"]!="invalid-domain":
                with self.subTest(case=r["id"]),self.assertRaises(IdentityError):parse(bytes.fromhex(r["raw_json_hex"]))

    def test_hash_is_domain_separated_with_no_lf_and_complete_fields(self):
        value={"a":1};expected=hashlib.sha256(b"MPK-TEST-1.0\0"+b'{"a":1}').hexdigest()
        self.assertEqual(digest("MPK-TEST-1.0",value),expected)
        self.assertNotEqual(expected,digest("MPK-TEST-2.0",value))
        n=copy.deepcopy(self.node("source_object"));n["bytes"]+=1
        with self.assertRaises(IdentityError):validate_node(n)

    def test_derived_contract_hash_cannot_enter_capture_record(self):
        n=copy.deepcopy(self.node("capture_context"));n["normalized_contract_sha256"]="a"*64
        with self.assertRaises(IdentityError):validate_node(n)

    def test_later_or_foreign_schema_and_unknown_fields_reject(self):
        original=self.node("source_object")
        for schema in ("mpk.go.source_object.v2","mpk.csharp.semantic_binding.v1"):
            n=copy.deepcopy(original);n["schema"]=schema
            with self.assertRaises(IdentityError):validate_node(n)
        n=copy.deepcopy(original);n["extra"]=None
        with self.assertRaises(IdentityError):validate_node(n)

    def test_source_paths_are_logical_and_raw_bytes_are_bound(self):
        for path in ("/tmp/policy.go","../policy.go","a//b","a\\b"):
            with self.assertRaises(IdentityError):self.changed("source_object",{"path":path})
        nodes,roots=self.changed("source_object",{"raw_sha256":hashlib.sha256(b"changed raw bytes").hexdigest()})
        self.assertNotEqual(validate_graph(nodes,roots)["capture_context_id"],self.f["result"]["capture_context_id"])

    def test_aliases_share_the_target_type_and_nominal_types_remain_distinct(self):
        alias=self.f["declarations"]["Money"];self.assertEqual(resolve_alias(alias,self.f["aliases"]),self.f["types"]["Amount"])
        self.assertNotEqual(self.f["types"]["Amount"],self.f["types"]["Other"])
        with self.assertRaises(IdentityError):resolve_alias(alias,{alias:alias})

    def test_alias_declaration_cannot_forge_a_distinct_nominal_type(self):
        nodes,roots=self.changed("source_type",{"declaration_id":self.f["declarations"]["Money"]},lambda n:n.get("kind")=="nominal")
        with self.assertRaises(IdentityError):validate_graph(nodes,roots)

    def test_full_word_identity_and_raw_struct_tag_bytes(self):
        with self.assertRaises(IdentityError):seal("source_type",{"kind":"builtin","name":"uint64","width":32,"signed":False})
        n=next(n for n in self.f["nodes"] if n.get("kind")=="struct")
        self.assertEqual(n["fields"][1]["tag_bytes_hex"],"ff")
        for f in n["fields"]:self.assertEqual(f["unexported_package_id"] is None,f["name"][0].isupper())

    def test_method_signature_and_ordered_results_bind_one_declaration(self):
        sig=self.node("signature")
        nodes,roots=self.changed("signature",{"declaration_id":self.f["declarations"]["Code"]})
        with self.assertRaises(IdentityError):validate_graph(nodes,roots)
        with self.assertRaises(IdentityError):seal("signature",{"declaration_id":sig["declaration_id"],"receiver":None,"params":[],"results":[self.f["types"]["int64"]]*17})

    def test_loop_ref_binds_physical_utf8_source_and_function(self):
        loop=self.node("loop_ref");raw=SOURCE.encode();self.assertTrue(raw[loop["start_byte"]:loop["end_byte"]].startswith(b"for "))
        self.assertNotEqual(loop["start_byte"],SOURCE.index("for _,v"))
        nodes,roots=self.changed("loop_ref",{"function_decl_id":self.f["declarations"]["Error"]})
        with self.assertRaises(IdentityError):validate_graph(nodes,roots)

    def test_loop_ref_cannot_reuse_a_different_capture_hash(self):
        nodes,roots=self.changed("loop_ref",{"capture_context_id":"0"*64})
        with self.assertRaises(IdentityError):validate_graph(nodes,roots)

    def test_package_and_module_source_membership_cannot_be_crossed(self):
        source=self.node("source_object")
        nodes,roots=self.changed("package_instance",{"objects":[source["id"]]})
        with self.assertRaises(IdentityError):validate_graph(nodes,roots)

    def test_missing_duplicate_and_unreachable_nodes_reject(self):
        nodes=self.f["nodes"]
        with self.assertRaises(IdentityError):validate_graph(nodes[1:],self.f["roots"])
        with self.assertRaises(IdentityError):validate_graph(nodes+[nodes[0]],self.f["roots"])
        extra=seal("source_type",{"kind":"builtin","name":"bool","width":None,"signed":None})
        with self.assertRaises(IdentityError):validate_graph(nodes+[extra],self.f["roots"])

    def test_error_universe_is_typed_unique_and_still_not_derived_by_this_probe(self):
        u=self.node("error_universe")
        with self.assertRaises(IdentityError):seal("error_universe",{"source_error_type_id":u["source_error_type_id"],"cases":u["cases"]*2})
        nodes,roots=self.changed("error_universe",{"source_error_type_id":self.f["types"]["int64"]})
        with self.assertRaises(IdentityError):validate_graph(nodes,roots)


if __name__=="__main__":main()
