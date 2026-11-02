// Private native observations; application code is never executed by MPK.
package main

import (
	"encoding/json"
	"os"
)

type cell struct {
	A   [4]int
	Tag int
}

func returningArray() ([4]int, []int, []int) {
	a := [4]int{1, 2, 3, 4}
	return a, a[:], a[1:]
}

func distantWindows() ([]int, []int) {
	var a [256]int
	a[10], a[250] = 10, 250
	return a[10:12:12], a[250:252:252]
}

func identityView(s []int) []int { return s }

func main() {
	results := map[string]any{}
	rows := make([][4]int, 2)
	rows[0] = [4]int{1, 2, 3, 4}
	rows[1] = [4]int{5, 6, 7, 8}
	inner := rows[0][:]
	inner[1] = 20
	results["child_write_parent_read"] = rows[0]
	rows[0] = [4]int{9, 10, 11, 12}
	results["parent_replace_existing_child"] = append([]int(nil), inner...)
	rows[1][2] = 70
	results["sibling_preserved"] = append([]int(nil), inner...)

	cs := make([]cell, 1)
	cs[0] = cell{A: [4]int{1, 2, 3, 4}, Tag: 7}
	fieldView := cs[0].A[:]
	cs[0].Tag = 8
	fieldView[3] = 40
	results["field_write_preserves_sibling"] = cs[0]
	cs[0] = cell{A: [4]int{5, 6, 7, 8}, Tag: 9}
	results["whole_cell_replace_child"] = append([]int(nil), fieldView...)

	a := [2][4]int{{1, 2, 3, 4}, {5, 6, 7, 8}}
	b := a
	originalView, copiedView := a[0][:], b[0][:]
	originalView[0] = 100
	results["array_value_copy_fresh_storage"] = [2][4]int{a[0], b[0]}
	copiedView[1] = 200
	results["copy_write_original_preserved"] = a[0]
	a = [2][4]int{{9, 8, 7, 6}, {5, 4, 3, 2}}
	results["whole_local_assignment_existing_view"] = append([]int(nil), originalView...)

	p := make([][4]int, 3)
	p[0], p[1], p[2] = [4]int{1, 2, 3, 4}, [4]int{5, 6, 7, 8}, [4]int{9, 10, 11, 12}
	oldMiddle := p[1][:]
	copy(p[1:], p[:2])
	results["overlap_copy_complete_elements"] = p
	results["overlap_copy_existing_child"] = append([]int(nil), oldMiddle...)

	returnedValue, first, second := returningArray()
	returnedValue[0] = 99
	second[0] = 22
	results["return_value_copy_and_shared_views"] = map[string]any{"value": returnedValue, "first": append([]int(nil), first...), "second": append([]int(nil), second...)}

	var nilSlice []int
	zeroView := rows[0][0:0:0]
	zeroChild := [2][]int{nilSlice, zeroView}
	results["zero_capacity_tag_and_count"] = map[string]any{"nil": zeroChild[0] == nil, "nonnil": zeroChild[1] != nil, "copy": copy(zeroChild[1], zeroChild[0]), "capacity": cap(zeroChild[1])}

	// Copies of root aggregates preserve descriptor sharing, even when their
	// value-array fields acquire fresh storage.
	type root struct {
		Arr  [2]int
		View []int
	}
	r1 := root{Arr: [2]int{1, 2}, View: first}
	r2 := r1
	r2.Arr[0] = 77
	r2.View[3] = 44
	results["root_copy_values_and_descriptor_aliases"] = map[string]any{"original_array": r1.Arr, "copied_array": r2.Arr, "original_view": append([]int(nil), r1.View...), "copied_view": append([]int(nil), r2.View...)}
	left, right := distantWindows()
	left[1], right[1] = 11, 251
	results["distant_capped_windows"] = map[string]any{"left": left, "right": right, "capacities": [2]int{cap(left), cap(right)}}
	callerArray := [4]int{1, 2, 3, 4}
	callerAlias := identityView(callerArray[:])
	callerArray[0] = 88
	results["callee_returns_caller_array_view"] = callerAlias
	if err := json.NewEncoder(os.Stdout).Encode(results); err != nil {
		panic(err)
	}
}
