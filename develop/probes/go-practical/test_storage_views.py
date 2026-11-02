from unittest import TestCase, main
from storage_views import World, INT, Descriptor, ZeroAccess, Location, Layout, array, structure, slice_layout, SourcePanic, ProfileDomain, PermissionFailure, ShapeFailure
from storage_view_cases import model, EXPECTED


class StorageViewTests(TestCase):
    def test_all_native_shaped_expected_cases(self): self.assertEqual(model(), EXPECTED)

    def test_many_typed_child_views_share_one_owner(self):
        w = World(); r = w.create(array(array(INT, 4), 128), kind="heap"); d = w.view(r)
        views = [w.view(w.element_location(d, i)) for i in range(128)]
        self.assertEqual(len({s.array.owner for s in views}), 1)
        w.write(w.element_location(views[-1], 3), 7)
        self.assertEqual(w.values(d)[-1][-1], 7)

    def test_borrowed_ancestor_forbids_child_write(self):
        w = World(); r = w.create(array(array(INT, 4), 1), kind="heap", access="borrowed")
        child = w.view(w.element_location(w.view(r), 0))
        with self.assertRaises(PermissionFailure): w.write(w.element_location(child, 0), 1)

    def test_positive_empty_tail_retains_accessible_capacity(self):
        w = World(); r = w.create(array(INT, 4), [1, 2, 3, 4], kind="heap")
        empty = w.view(r, 0, 0)
        self.assertEqual(empty.capacity, 4); self.assertEqual(w.values(w.reslice(empty, 0, 4)), [1, 2, 3, 4])

    def test_zero_access_is_typed_and_cannot_write(self):
        w = World(); r = w.create(array(INT, 1), kind="heap"); d = w.view(r, 0, 0, 0)
        self.assertEqual(d.array, ZeroAccess("int")); self.assertFalse(d.nil)
        with self.assertRaises(SourcePanic): w.element_location(d, 0)
        with self.assertRaises(ProfileDomain): w.append(d, [1])

    def test_zero_source_objects_do_not_consume_positive_owner_slots(self):
        w = World()
        for _ in range(5): w.create(array(INT, 0), kind="heap")
        w.create(array(INT, 1), kind="heap"); w.create(array(INT, 1), kind="heap")
        with self.assertRaises(ProfileDomain): w.create(array(INT, 1), kind="heap")

    def test_zero_count_copy_needs_no_write_permission(self):
        w = World(); r = w.create(array(INT, 2), [1, 2], kind="heap", access="borrowed")
        dest = w.view(r, 0, 0); nil = Descriptor("int", True, None, 0, 0, 0)
        self.assertEqual(w.copy_slice(dest, nil), 0); self.assertEqual(w.read(r), [1, 2])

    def test_same_values_do_not_coalesce_distinct_objects(self):
        w = World(); a = w.create(array(INT, 1), [0], kind="heap"); b = w.create(array(INT, 1), [0], kind="heap")
        w.write(w.element_location(w.view(a), 0), 9); self.assertEqual(w.values(w.view(b)), [0])

    def test_root_replacement_keeps_captured_array_path(self):
        w = World(); t = structure("R", [("A", array(INT, 2)), ("B", INT)])
        r = w.create(t); d = w.view(w.child(r, "field", "A")); w.write(r, {"A": [7, 8], "B": 9})
        self.assertEqual(w.values(d), [7, 8]); w.write(w.element_location(d, 0), 1)
        self.assertEqual(w.read(r), {"A": [1, 8], "B": 9})

    def test_descriptor_path_requires_explicit_owner_pivot(self):
        w = World(); a = w.create(array(INT, 2), kind="heap"); root = w.create(structure("R", [("S", slice_layout(INT))]), {"S": w.view(a)})
        with self.assertRaises(ShapeFailure): w.child(w.child(root, "field", "S"), "index", 0)
        w.write(w.element_location(w.read(w.child(root, "field", "S")), 0), 3)
        self.assertEqual(w.read(a), [3, 0])

    def test_exact_type_keys_and_complete_value_shape(self):
        w = World(); r = w.create(array(INT, 2))
        with self.assertRaises(ShapeFailure): w.write(r, [1])
        with self.assertRaises(ShapeFailure): w.create(Layout("int", "struct", fields=(("A", INT),)))

    def test_union_window_packing_preserves_overlap(self):
        w = World(); r = w.create(array(INT, 256), list(range(256)))
        a, b = w.export_views([w.view(r, 100, 104, 104), w.view(r, 102, 106, 106)])
        self.assertEqual(a.array, b.array); self.assertEqual((a.offset, b.offset), (0, 2))
        w.write(w.element_location(b, 0), 999); self.assertEqual(w.values(a), [100, 101, 999, 103])

    def test_child_export_uses_parent_owner_when_capacity_covers_it(self):
        w = World(); r = w.create(array(array(INT, 4), 2))
        top = w.view(r); child = w.view(w.element_location(top, 1)); parent, inner = w.export_views([top, child])
        self.assertEqual(parent.array.owner, inner.array.owner)
        w.write(w.element_location(parent, 1), [4, 5, 6, 7]); self.assertEqual(w.values(inner), [4, 5, 6, 7])

    def test_inaccessible_parent_cell_cannot_force_hidden_storage_bound(self):
        w = World(); r = w.create(array(array(INT, 4), 256))
        parent = w.view(r, 0, 128, 128); child = w.view(w.child(r, "index", 250))
        a, b = w.export_views([parent, child]); self.assertNotEqual(a.array.owner, b.array.owner)
        w.write(w.element_location(b, 1), 9); self.assertEqual(w.values(a)[0], [0, 0, 0, 0])

    def test_export_quota_failure_leaves_world_unchanged(self):
        w = World(); w.create(array(INT, 1), kind="heap"); w.create(array(INT, 1), kind="heap")
        local = w.create(array(INT, 2)); before = dict(w.owners); serial = w.serial
        with self.assertRaises(ProfileDomain): w.export_views([w.view(local)])
        self.assertEqual(w.owners, before); self.assertEqual(w.serial, serial)

    def test_promotion_cannot_launder_borrowed_permissions(self):
        w = World(); r = w.create(array(INT, 2), access="borrowed"); serial = w.serial
        with self.assertRaises(PermissionFailure): w.export_views([w.view(r)])
        self.assertEqual(w.serial, serial); self.assertEqual(w.owners[r.owner].access, "borrowed")

    def test_callee_exit_cannot_copy_a_still_live_caller_owner(self):
        w = World(); r = w.create(array(INT, 2), scope="caller"); d = w.view(r)
        returned, = w.export_views([d], ending_scope="callee")
        self.assertEqual(returned.array, d.array)
        w.write(w.element_location(d, 0), 9); self.assertEqual(w.values(returned), [9, 0])

    def test_ended_local_scope_is_not_a_second_live_store(self):
        w = World(); r = w.create(array(INT, 2)); d, = w.export_views([w.view(r)])
        with self.assertRaises(ShapeFailure): w.read(r)
        with self.assertRaises(ShapeFailure): w.create(array(INT, 1))
        w.write(w.element_location(d, 0), 7); self.assertEqual(w.values(d), [7, 0])

    def test_capacity_and_storage_overflow_are_domain_not_go_panic(self):
        w = World(); r = w.create(array(INT, 256))
        with self.assertRaises(ProfileDomain): w.view(r)
        with self.assertRaises(ProfileDomain): w.create(array(INT, 129), kind="heap")

    def test_source_bounds_reject_full_index_without_truncation(self):
        w = World(); d = w.view(w.create(array(INT, 2), kind="heap"))
        for index in (-1, 2, 1 << 64):
            with self.subTest(index=index), self.assertRaises(SourcePanic): w.element_location(d, index)
        with self.assertRaises(ShapeFailure): w.element_location(d, True)


if __name__ == "__main__": main()
