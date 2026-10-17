"""Independent expected native observations and canonical-owner model traces."""
from storage_views import World, INT, Descriptor, array, structure, slice_layout

EXPECTED = {
    "child_write_parent_read": [1, 20, 3, 4],
    "parent_replace_existing_child": [9, 10, 11, 12],
    "sibling_preserved": [9, 10, 11, 12],
    "field_write_preserves_sibling": {"A": [1, 2, 3, 40], "Tag": 8},
    "whole_cell_replace_child": [5, 6, 7, 8],
    "array_value_copy_fresh_storage": [[100, 2, 3, 4], [1, 2, 3, 4]],
    "copy_write_original_preserved": [100, 2, 3, 4],
    "whole_local_assignment_existing_view": [9, 8, 7, 6],
    "overlap_copy_complete_elements": [[1, 2, 3, 4], [1, 2, 3, 4], [5, 6, 7, 8]],
    "overlap_copy_existing_child": [1, 2, 3, 4],
    "return_value_copy_and_shared_views": {"value": [99, 2, 3, 4], "first": [1, 22, 3, 4], "second": [22, 3, 4]},
    "zero_capacity_tag_and_count": {"nil": True, "nonnil": True, "copy": 0, "capacity": 0},
    "root_copy_values_and_descriptor_aliases": {"original_array": [1, 2], "copied_array": [77, 2], "original_view": [1, 22, 3, 44], "copied_view": [1, 22, 3, 44]},
    "distant_capped_windows": {"left": [10, 11], "right": [250, 251], "capacities": [2, 2]},
    "callee_returns_caller_array_view": [88, 2, 3, 4],
}


def model():
    out = {}
    row = array(INT, 4)
    w = World(); rows = w.create(array(row, 2), [[1, 2, 3, 4], [5, 6, 7, 8]], kind="heap")
    top = w.view(rows); inner = w.view(w.element_location(top, 0))
    w.write(w.element_location(inner, 1), 20); out["child_write_parent_read"] = w.read(w.element_location(top, 0))
    w.write(w.element_location(top, 0), [9, 10, 11, 12]); out["parent_replace_existing_child"] = w.values(inner)
    other = w.view(w.element_location(top, 1)); w.write(w.element_location(other, 2), 70)
    out["sibling_preserved"] = w.values(inner)
    cell = structure("cell", [("A", row), ("Tag", INT)])
    cs = w.create(array(cell, 1), [{"A": [1, 2, 3, 4], "Tag": 7}], kind="heap")
    cv = w.view(cs); cl = w.element_location(cv, 0); fv = w.view(w.child(cl, "field", "A"))
    w.write(w.child(cl, "field", "Tag"), 8); w.write(w.element_location(fv, 3), 40)
    out["field_write_preserves_sibling"] = w.read(cl)
    w.write(cl, {"A": [5, 6, 7, 8], "Tag": 9}); out["whole_cell_replace_child"] = w.values(fv)
    w = World(); a = w.create(array(row, 2), [[1, 2, 3, 4], [5, 6, 7, 8]])
    b = w.copy_local(a); av = w.view(w.child(a, "index", 0)); bv = w.view(w.child(b, "index", 0))
    w.write(w.element_location(av, 0), 100)
    out["array_value_copy_fresh_storage"] = [w.values(av), w.values(bv)]
    w.write(w.element_location(bv, 1), 200); out["copy_write_original_preserved"] = w.values(av)
    w.write(a, [[9, 8, 7, 6], [5, 4, 3, 2]]); out["whole_local_assignment_existing_view"] = w.values(av)
    w = World(); p = w.create(array(row, 3), [[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12]], kind="heap")
    pv = w.view(p); middle = w.view(w.element_location(pv, 1))
    w.copy_slice(w.reslice(pv, 1, 3), w.reslice(pv, 0, 2))
    out["overlap_copy_complete_elements"] = w.values(pv); out["overlap_copy_existing_child"] = w.values(middle)
    w = World(); original = w.create(row, [1, 2, 3, 4], scope="callee"); value = w.read(original)
    first, second = w.export_views([w.view(original), w.view(original, 1, 4)], ending_scope="callee")
    result_array = w.create(row, value, scope="caller"); w.write(w.child(result_array, "index", 0), 99); w.write(w.element_location(second, 0), 22)
    out["return_value_copy_and_shared_views"] = {"value": w.read(result_array), "first": w.values(first), "second": w.values(second)}
    nil = Descriptor("int", True, None, 0, 0, 0); empty = w.reslice(first, 0, 0, 0)
    out["zero_capacity_tag_and_count"] = {"nil": nil.nil, "nonnil": not empty.nil, "copy": w.copy_slice(empty, nil), "capacity": empty.capacity}
    root = structure("root", [("Arr", array(INT, 2)), ("View", slice_layout(INT))])
    r1 = w.create(root, {"Arr": [1, 2], "View": first}, scope="caller"); r2 = w.copy_local(r1, scope="caller")
    w.write(w.child(w.child(r2, "field", "Arr"), "index", 0), 77)
    w.write(w.element_location(w.read(w.child(r2, "field", "View")), 3), 44)
    out["root_copy_values_and_descriptor_aliases"] = {"original_array": w.read(w.child(r1, "field", "Arr")), "copied_array": w.read(w.child(r2, "field", "Arr")), "original_view": w.values(w.read(w.child(r1, "field", "View"))), "copied_view": w.values(w.read(w.child(r2, "field", "View")))}
    w = World(); large = w.create(array(INT, 256)); w.write(w.child(large, "index", 10), 10); w.write(w.child(large, "index", 250), 250)
    left, right = w.export_views([w.view(large, 10, 12, 12), w.view(large, 250, 252, 252)])
    w.write(w.element_location(left, 1), 11); w.write(w.element_location(right, 1), 251)
    out["distant_capped_windows"] = {"left": w.values(left), "right": w.values(right), "capacities": [left.capacity, right.capacity]}
    w = World(); caller = w.create(row, [1, 2, 3, 4], scope="caller")
    alias, = w.export_views([w.view(caller)], ending_scope="callee")
    w.write(w.child(caller, "index", 0), 88); out["callee_returns_caller_array_view"] = w.values(alias)
    return out
