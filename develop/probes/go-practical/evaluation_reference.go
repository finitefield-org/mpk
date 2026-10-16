// Private Go evaluation observations. Counters/recovery are observer-only.
package main

import (
	"encoding/json"
	"os"
)

type pair struct{ A, B int }
type byteSlice []byte
type text string
type sliceFailure struct{ Values []int }

func (sliceFailure) Error() string { return "failure" }

type errorHolder struct{ Error error }
type root struct {
	V int
	S []int
}
type row struct {
	ID   string `json:"id"`
	N    int    `json:"n"`
	Flag bool   `json:"flag"`
}

var trace []int

func mark(id, value int) int { trace = append(trace, id); return value }
func joint() (int, int)      { trace = append(trace, 1); return 7, 9 }
func arrayCall() [3]int      { trace = append(trace, 1); return [3]int{1, 2, 3} }
func panicMark() int         { trace = append(trace, 1); panic("observed") }
func catch(f func()) (panicked bool) {
	defer func() {
		if recover() != nil {
			panicked = true
		}
	}()
	f()
	return false
}

func observe(r row) map[string]any {
	trace = []int{}
	result := map[string]any{}
	switch r.ID {
	case "parallel-index":
		s := []int{3, 5, 7}
		i := 0
		i, s[i] = 1, 2
		result["i"], result["s"] = i, s
	case "duplicate-target":
		s := []int{3}
		s[0], s[0] = 1, 2
		result["s"] = s
	case "slice-rebind-target":
		a, b := []int{1, 2}, []int{3, 4}
		s := a
		s, s[0] = b, 9
		result["s"], result["old"] = s, a
	case "array-rebind-target":
		a, b := [2]int{1, 2}, [2]int{3, 4}
		a, a[0] = b, 9
		result["a"], result["b"] = a, b
	case "array-view-assignment":
		a, b := [2]int{1, 2}, [2]int{3, 4}
		view := a[:]
		a = b
		b[0] = 8
		result["a"], result["b"], result["view"] = a, b, view
	case "struct-rebind-target":
		a := root{V: 1}
		a, a.V = root{V: 3}, 9
		result["v"] = a.V
	case "struct-slice-rebind-target":
		old := []int{1, 2}
		a := root{V: 1, S: old}
		a, a.S[0] = root{V: 3, S: []int{3, 4}}, 9
		result["v"], result["s"], result["old"] = a.V, a.S, old
	case "cell-field-merge":
		s := []pair{{1, 2}}
		s[0].A, s[0].B = s[0].B, s[0].A
		result["a"], result["b"] = s[0].A, s[0].B
	case "joint-call-once":
		a, b := joint()
		result["a"], result["b"] = a, b
	case "discarded-result":
		a, _ := joint()
		result["a"] = a
	case "forwarded-return":
		a, b := func() (int, int) { return joint() }()
		result["a"], result["b"] = a, b
	case "compound-once":
		s := []int{10}
		s[mark(1, 0)] += mark(2, 7)
		result["s"] = s
	case "assignment-calls-before-store-panic":
		s := []int{1, 2}
		panicked := catch(func() { s[0], s[2] = mark(1, 7), mark(2, 9) })
		result["s"], result["panicked"] = s, panicked
	case "rhs-panic-before-writes":
		s := []int{1, 2}
		panicked := catch(func() { s[0], s[1] = mark(2, 7), panicMark() })
		result["s"], result["panicked"] = s, panicked
	case "short-circuit":
		a := r.Flag && mark(1, 1) == 1
		b := r.Flag || mark(2, 1) == 1
		result["a"], result["b"] = a, b
	case "switch-case-order":
		v := 0
		switch mark(1, 2) {
		case mark(2, 1), mark(3, 2):
			v = 7
		case mark(4, 2):
			v = 9
		default:
			v = 11
		}
		result["v"] = v
	case "switch-default-first":
		v := 0
		switch mark(1, 2) {
		default:
			v = 11
		case mark(2, 2):
			v = 7
		}
		result["v"] = v
	case "switch-break-target":
		total, iterations := 0, 0
		for i := 0; i < r.N; i++ {
			switch i % 2 {
			case 0:
				total += 10
				break
			default:
				total++
			}
			total += 100
			iterations++
		}
		result["total"], result["iterations"] = total, iterations
	case "switch-continue-target":
		total, posts := 0, 0
		for i := 0; i < r.N; func() { i++; posts++ }() {
			switch i % 2 {
			case 0:
				continue
			}
			total++
		}
		result["total"], result["posts"] = total, posts
	case "nested-loop-break":
		outer, inner := 0, 0
		for i := 0; i < r.N; i++ {
			for j := 0; j < 3; j++ {
				inner++
				break
			}
			outer++
		}
		result["outer"], result["inner"] = outer, inner
	case "range-array-copy":
		a := [3]int{1, 2, 3}
		seen := make([]int, 0, 3)
		for i, v := range a {
			if i == 0 {
				a[1] = 20
			}
			seen = append(seen, v)
		}
		result["seen"], result["a"] = seen, a
	case "range-array-descriptor-copy":
		s := []int{1, 2}
		a := [2][]int{s, s}
		seen := make([]int, 0, 2)
		for i, v := range a {
			if i == 0 {
				a[1] = []int{9}
				s[0] = 7
			}
			seen = append(seen, v[0])
		}
		result["seen"], result["rebound"] = seen, a[1][0]
	case "range-slice-capture":
		s := []int{1, 2, 3}
		seen := make([]int, 0, 3)
		for i, v := range s {
			if i == 0 {
				s[1] = 20
				s = s[:1]
			}
			seen = append(seen, v)
		}
		result["seen"], result["s"] = seen, s
	case "range-slice-nil":
		var s []int
		n := 0
		for range s {
			n++
		}
		result["iterations"] = n
	case "range-assignment-target":
		s := []int{3, 5, 7}
		i := 2
		for i, s[i] = range s {
			break
		}
		result["i"], result["s"] = i, s
	case "range-array-call-once":
		sum := 0
		for i := range arrayCall() {
			sum += i
		}
		result["sum"] = sum
	case "range-blank-second-variable":
		sum := 0
		for i, _ := range arrayCall() {
			sum += i
		}
		result["sum"] = sum
	case "range-no-element-read":
		a := [3]int{1, 2, 3}
		sum := 0
		for i := range a {
			sum += i
		}
		result["sum"] = sum
	case "array-size-call":
		n, c := len(arrayCall()), cap(arrayCall())
		result["length"], result["capacity"] = n, c
	case "early-return-skips-post":
		value, posts := func() (int, int) {
			posts := 0
			for i := 0; i < 1; func() { i++; posts++ }() {
				return 7, posts
			}
			return 9, posts
		}()
		result["value"], result["posts"] = value, posts
	case "named-return-zero":
		value, ok := func() (value int, ok bool) { return }()
		result["value"], result["ok"] = value, ok
	case "byte-string-copy":
		dst := make(byteSlice, 3)
		n := copy(dst, text("\x00\x80xq"))
		result["count"], result["bytes"] = n, []int{int(dst[0]), int(dst[1]), int(dst[2])}
	case "byte-string-append":
		dst := make(byteSlice, 1, 4)
		dst[0] = 7
		var appended byteSlice = append(dst, text("\x00\x80x")...)
		result["bytes"], result["capacity"] = []int{int(appended[0]), int(appended[1]), int(appended[2]), int(appended[3])}, cap(appended)
	case "integer-defined-edges":
		minimum := int64(-1 << 63)
		negativeOne := int64(-1)
		count := uint64(^uint64(0))
		result["div"], result["rem"] = minimum/negativeOne, minimum%negativeOne
		result["left"], result["unsigned_right"], result["signed_right"] = uint64(1)<<count, uint64(1)>>count, negativeOne>>count
	case "aggregate-error-equality":
		a := errorHolder{Error: sliceFailure{Values: []int{1}}}
		result["panicked"] = catch(func() { _ = a == a })
	default:
		panic("unknown private observation")
	}
	result["trace"] = append([]int{}, trace...)
	return result
}
func main() {
	var rows []row
	if e := json.NewDecoder(os.Stdin).Decode(&rows); e != nil {
		panic(e)
	}
	results := make([]map[string]any, len(rows))
	for i, row := range rows {
		results[i] = observe(row)
	}
	if e := json.NewEncoder(os.Stdout).Encode(results); e != nil {
		panic(e)
	}
}
