from unittest import TestCase, main
import json
from pathlib import Path
from structural_counters import Shape, Shapes, CounterError, CounterLimit, VALUE_UNITS, CURRENT_UNITS, ALL_UNITS, U64_MAX, checked_charge, measure_world, DESCRIPTOR_UNITS
from structural_counter_cases import boundary_cases, exact_weight_shapes, shape_observations, quote_type_weights, state_observations
from structural_limit_catalog import generate
from storage_lifetimes import LiveWorld, Value, Snapshot, CallEvidence, BYTES
from storage_views import INT, array, structure, slice_layout


class StructuralCounterTests(TestCase):
    @classmethod
    def setUpClass(cls): cls.states=state_observations()

    def test_catalog_and_all_isolated_boundaries(self):
        self.assertEqual(len(generate()["counters"]),64)
        rows=boundary_cases();self.assertEqual(len(rows),192)
        self.assertEqual(sum(r["outcome"]=="counter_within_limit" for r in rows),127)
        self.assertEqual(sum(r["outcome"]=="invalid_counter_input" for r in rows),1)

    def test_counter_preflight_does_not_wrap_or_commit_failure(self):
        current=U64_MAX
        with self.assertRaises(CounterLimit):checked_charge(current,1,U64_MAX,"bytes")
        self.assertEqual(current,U64_MAX)
        self.assertEqual(checked_charge(U64_MAX-1,1,U64_MAX,"bytes"),U64_MAX)
        for n in (-1,True,U64_MAX+1):
            with self.assertRaises(CounterError):checked_charge(0,n,10,"bytes")

    def test_exact_value_boundaries_with_real_array_struct_shapes(self):
        for n in (VALUE_UNITS-1,VALUE_UNITS,VALUE_UNITS+1):
            height,plain,weight=Shapes(exact_weight_shapes(n)).inspect("BudgetValue")
            self.assertEqual(weight,n);self.assertEqual(height,4);self.assertTrue(plain)

    def test_bound_interactions_do_not_guarantee_whole_value_fit(self):
        rows={r["id"]:r for r in shape_observations()}
        self.assertEqual(rows["both_maxima"]["true_weight_if_known"],4210945)
        self.assertFalse(rows["both_maxima"]["units_exact"])
        self.assertEqual(rows["both_maxima"]["weight"],VALUE_UNITS+1)
        self.assertEqual(rows["result_product"]["weight"],1052689)

    def test_huge_graph_is_saturated_without_value_expansion(self):
        rows=[Shape("string","string")]
        for i in range(16):rows.append(Shape(str(i),"array","string" if not i else str(i-1),256))
        self.assertEqual(Shapes(rows).inspect("15"),(16,True,VALUE_UNITS+1))

    def test_zero_array_retains_full_child_graph_validation(self):
        rows=[Shape("word","word")]
        for i in range(15):rows.append(Shape(str(i),"array","word" if not i else str(i-1),256))
        rows.append(Shape("empty","array","14",0))
        self.assertEqual(Shapes(rows).inspect("empty"),(16,True,1))
        with self.assertRaises(CounterError):Shapes([Shape("cycle","array","cycle",0)]).inspect("cycle")
        with self.assertRaises(CounterError):Shapes([Shape("bad","unsupported"),Shape("empty","array","bad",0)]).inspect("empty")

    def test_cached_shared_subtype_cannot_hide_a_deeper_path(self):
        rows=[Shape("word","word")]
        for i in range(16):rows.append(Shape(str(i),"array","word" if not i else str(i-1),1))
        rows.append(Shape("root","struct",fields=(("short","word"),("deep","15"))))
        shapes=Shapes(rows);shapes.inspect("15")
        with self.assertRaises(CounterLimit):shapes.inspect("root")

    def test_inherited_array_field_depth_boundaries(self):
        self.assertEqual(Shapes([Shape("w","word"),Shape("a","array","w",256)]).weight("a"),257)
        with self.assertRaises(CounterLimit):Shapes([Shape("w","word"),Shape("a","array","w",257)]).weight("a")
        with self.assertRaises(CounterLimit):Shapes([Shape("w","word"),Shape("s","struct",fields=tuple((str(i),"w") for i in range(65)))]).weight("s")

    def test_complete_error_payloads_count_and_reject_hidden_references(self):
        rows=[Shape("w","word"),Shape("s","slice","w"),Shape("ref","struct",fields=(("Data","s"),)),Shape("e","error",fields=(("Plain","w"),("Ref","ref"))),Shape("es","slice","e")]
        self.assertEqual(Shapes(rows).weight("e"),1+1+1+DESCRIPTOR_UNITS)
        with self.assertRaises(CounterError):Shapes(rows).inspect("es")
        with self.assertRaises(CounterError):Shapes([Shape("e","error",fields=(("Recursive","s"),)),Shape("s","struct",fields=(("Err","e"),))]).inspect("e")

    def test_original_quote_types_and_all_snapshot_costs(self):
        self.assertEqual(quote_type_weights()["inputs"],[257,37,1])
        active=self.states["quote_shaped_active_frame"]
        self.assertEqual(active["current"]["units"],1024);self.assertEqual(active["total_units"],1578)
        returned=self.states["quote_shaped_return_evidence"]
        self.assertEqual(returned["past_store_count"],2);self.assertEqual(returned["total_units"],1421)
        self.assertEqual(returned["violations"],[])

    def test_packing_does_not_erase_full_past_source_storage(self):
        self.assertEqual(self.states["packed_windows_current_cells"],4)
        self.assertEqual(self.states["packed_windows_post_source_cells"],256)
        self.assertEqual(self.states["packed_windows_keep_full_post_store"]["total_units"],416)

    def test_reuse_cost_is_live_carrier_not_ghost_allocation_history(self):
        r=self.states["repeated_allocation"]
        self.assertEqual((r["iterations"],r["sum"],r["peak_carrier_units"],r["final_carrier_units"]),(1000,499500,111,36))
        self.assertEqual(r["ghost_allocation_events"],1000);self.assertFalse(r["canonical_finite_id_proved"])

    def test_many_child_views_share_one_heap_owner_but_each_value_is_charged(self):
        w=LiveWorld();w.begin_scope("root");element=array(INT,4)
        d=w.allocate("new","root",element,128)
        views=[w.view(w.element_location(d,i)) for i in range(128)]
        w.declare("root","views",array(slice_layout(INT),128),views);w.release("new")
        r=measure_world(w)
        self.assertEqual(sum(o.kind=="heap" for o in w.owners.values()),1)
        self.assertEqual((r["largest_value_units"],r["total_units"]),(4737,5414))

    def test_descriptor_copies_charge_both_values_and_heap_once(self):
        w=LiveWorld();w.begin_scope("root");d=w.allocate("new","root",INT,1)
        w.declare("root","a",slice_layout(INT),d);w.declare("root","b",slice_layout(INT),d);w.release("new")
        r=measure_world(w)
        self.assertEqual(r["current"]["owners"],3);self.assertEqual(r["total_units"],146)

    def test_pending_operands_and_captured_target_roles_are_not_free(self):
        w=LiveWorld();w.begin_scope("root");d=w.allocate("rhs","root",INT,1)
        self.assertEqual(measure_world(w)["total_units"],42)
        w.hold_target("lhs","root",w.element_location(d,0));w.release("rhs");w.collect()
        self.assertEqual(measure_world(w)["total_units"],37)

    def test_shared_past_store_keeps_distinct_observation_packets(self):
        w=LiveWorld();w.register(INT);w.begin_scope("root",entry=(),result_token="call",result_types=())
        base=w.scopes["root"].snapshot
        extra=Snapshot(base.store,(Value(INT,7),),())
        w.packets["call"].evidence=CallEvidence("probe",base,extra,())
        r=measure_world(w)
        self.assertEqual(r["past_store_count"],1);self.assertEqual(r["past"][0]["observation_units"],3)

    def test_equal_past_contents_in_distinct_namespaces_are_charged_twice(self):
        w=LiveWorld();w.begin_scope("root",entry=(),result_token="call",result_types=())
        base=w.scopes["root"].snapshot;extra=Snapshot(w.store_snapshot(),(),())
        w.packets["call"].evidence=CallEvidence("probe",base,extra,())
        self.assertEqual(measure_world(w)["past_store_count"],2)

    def test_mutable_past_store_is_not_a_snapshot(self):
        w=LiveWorld();w.begin_scope("root");w.scopes["root"].snapshot=Snapshot(w,(),())
        with self.assertRaises(CounterError):measure_world(w)

    def test_heap_collection_is_charged_per_cell_and_against_store_total(self):
        w=LiveWorld();w.begin_scope("root")
        row=structure("Row",[("f"+str(i),BYTES) for i in range(64)])
        w.allocate("new","root",row,128)
        r=measure_world(w)
        self.assertEqual(r["largest_value_units"],16449)
        self.assertEqual(r["current"]["owner_units"],2105474)
        self.assertEqual(r["violations"],[]);self.assertTrue(r["units_exact"])

    def test_http_dependency_requires_a_distinct_practical_syntax_candidate(self):
        p=Path(__file__).resolve().parents[3]/"develop/migrations/go-practical/t01-w04/structural-limits/native-http-syntax.json"
        h=json.loads(p.read_text());nodes=h["total_syntax_nodes"]
        self.assertEqual(nodes,1417549)
        row=next(r for r in generate()["counters"] if r["id"]=="capture.typed_syntax_nodes_across_included_packages")
        self.assertEqual((row["origin_maximum"],row["maximum"]),(1000000,4194304))
        with self.assertRaises(CounterLimit):checked_charge(0,nodes,1000000,"old")
        self.assertEqual(checked_charge(0,nodes,row["maximum"],"new"),nodes)


if __name__=="__main__":main()
