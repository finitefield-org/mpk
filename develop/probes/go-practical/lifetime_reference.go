// Native data observations; no machine address or GC claim is made.
package main

import (
	"encoding/hex"
	"encoding/json"
	"os"
	"strconv"
)

type carriedError struct{ Data []int }

func (carriedError) Error() string { return "carried" }

type code uint8

func (code) Error() string { return "code" }

func carry() (int, error) {
	a := [4]int{1, 2, 3, 4}
	return 7, carriedError{Data: a[:]}
}
func identity(s []int) []int { return s }

func main() {
	out := map[string]any{}
	s := make([]int, 1)
	s[0] = 7
	s = nil
	s = make([]int, 1)
	s[0] = 9
	out["overwritten_reference"] = append([]int(nil), s...)
	total := 0
	for i := 0; i < 1000; i++ {
		tmp := make([]int, 1)
		tmp[0] = i
		total += tmp[0]
	}
	out["iteration_values"] = total
	s = make([]int, 2)
	empty := s[:0]
	s = nil
	empty = append(empty, 7)
	out["positive_capacity_empty"] = map[string]any{"values": empty, "capacity": cap(empty)}
	z := empty[:0:0]
	out["zero_access_tag"] = map[string]any{"nil": z == nil, "length": len(z), "capacity": cap(z)}
	s = []int{1}
	t := []int{2}
	observedOld := s // observer-only alias to show the destination's actual write
	s, s[0] = t, 9
	out["captured_assignment_target"] = map[string]any{"current": s[0], "old": observedOld[0]}
	a, b := []int{1}, []int{2}
	root := [2][]int{a, b}
	copyRoot := root
	root = [2][]int{}
	a, b = nil, nil
	copyRoot[0][0] = 8
	out["array_descriptor_copy"] = [2]int{copyRoot[0][0], copyRoot[1][0]}
	n, err := carry()
	// A type assertion here is observer instrumentation, outside verified v1.
	payload := err.(carriedError)
	payload.Data[0] = 9
	out["error_result_payload"] = map[string]any{"first": n, "nil": err == nil, "message": err.Error(), "data": payload.Data}
	caller := [2]int{1, 2}
	returned := identity(caller[:])
	caller[0] = 88
	out["caller_scope_preserved"] = returned
	before := []int{7}
	oldData := append([]int(nil), before...)
	before[0] = 9
	current := before[0]
	before = nil
	after := []int{11}
	out["past_store_isolated"] = map[string]any{"old": oldData[0], "changed": current, "new": after[0]}
	var nilError error
	var zeroError error = code(0)
	out["error_nil_and_concrete_zero"] = [2]bool{nilError == nil, zeroError == nil}
	bytes := []string{string([]byte{0, 255})}
	out["byte_data"] = hex.EncodeToString([]byte(bytes[0]))
	words := []uint64{^uint64(0)}
	out["unsigned_word"] = strconv.FormatUint(words[0], 10)
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
