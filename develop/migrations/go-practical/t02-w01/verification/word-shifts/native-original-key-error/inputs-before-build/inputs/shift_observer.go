// Actual typed full-count Go shifts, including negative-count panics.
package main

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"strconv"
)

type query struct {
	Operation   string `json:"operation"`
	Count       string `json:"count_hex"`
	SignedCount bool   `json:"signed_count"`
	Target      int    `json:"target_width"`
	ID          string `json:"id"`
	Width       int    `json:"width"`
	X           string `json:"x_hex"`
	Y           string `json:"y_hex"`
}
type unsigned interface {
	~uint8 | ~uint16 | ~uint32 | ~uint64
}
type signed interface {
	~int8 | ~int16 | ~int32 | ~int64
}

func shift[U unsigned, S signed, C unsigned | signed](op string, x U, c C) (r uint64, left any, fault bool) {
	defer func() {
		if recover() != nil {
			fault = true
			left = nil
			r = 0
		}
	}()
	switch op {
	case "ShiftLeft":
		r = uint64(x << c)
		s := uint64(U(S(x) << c))
		if s != r {
			panic("signed and unsigned left bits")
		}
		left = s
	case "ShiftRight":
		r = uint64(x >> c)
	case "SignedShiftRight":
		r = uint64(U(S(x) >> c))
	default:
		panic("shift operation")
	}
	return
}
func observe[U unsigned, S signed](q query, x, c uint64) map[string]any {
	if uint64(U(x)) != x {
		panic("exact width")
	}
	d := 0
	for (1 << d) < q.Width {
		d++
	}
	var r uint64
	var left any
	var fault bool
	if q.SignedCount {
		r, left, fault = shift[U, S](q.Operation, U(x), int64(c))
	} else {
		r, left, fault = shift[U, S](q.Operation, U(x), c)
	}
	defined := !q.SignedCount || int64(c) >= 0
	if fault == defined {
		panic("actual count guard")
	}
	var result any
	var signedLeft any
	var stages any
	if defined {
		result = fmt.Sprintf("%0*x", q.Width/4, r)
		if left != nil {
			signedLeft = fmt.Sprintf("%0*x", q.Width/4, left.(uint64))
		}
		xs := U(x)
		values := []string{fmt.Sprintf("%0*x", q.Width/4, xs)}
		for i := 0; i < d; i++ {
			if c&(1<<i) != 0 {
				switch q.Operation {
				case "ShiftLeft":
					xs = xs << uint(1<<i)
				case "ShiftRight":
					xs = xs >> uint(1<<i)
				case "SignedShiftRight":
					xs = U(S(xs) >> uint(1<<i))
				}
			}
			values = append(values, fmt.Sprintf("%0*x", q.Width/4, xs))
		}
		stages = values
	}
	var excess any
	if c >= uint64(q.Width) {
		excess = strconv.FormatUint(c-uint64(q.Width), 10)
	}
	return map[string]any{"id": q.ID, "operation": q.Operation, "count_defined": defined, "native_count_panicked": fault, "count_high": c>>d != 0, "low_count": strconv.FormatUint(c&uint64(q.Width-1), 10), "high_value": strconv.FormatUint(c>>d, 10), "large_count_excess": excess, "result_hex": result, "signed_left_result_hex": signedLeft, "stage_hex": stages}
}
func main() {
	d := json.NewDecoder(os.Stdin)
	d.DisallowUnknownFields()
	var qs []query
	if err := d.Decode(&qs); err != nil {
		panic(err)
	}
	var tail any
	if err := d.Decode(&tail); err != io.EOF {
		panic("one original query document")
	}
	rows := make([]map[string]any, len(qs))
	for i, q := range qs {
		if q.Target != 0 {
			panic("original shift target")
		}
		x, e := strconv.ParseUint(q.X, 16, 64)
		if e != nil {
			panic(e)
		}
		c, e := strconv.ParseUint(q.Count, 16, 64)
		if e != nil {
			panic(e)
		}
		switch q.Width {
		case 8:
			rows[i] = observe[uint8, int8](q, x, c)
		case 16:
			rows[i] = observe[uint16, int16](q, x, c)
		case 32:
			rows[i] = observe[uint32, int32](q, x, c)
		case 64:
			rows[i] = observe[uint64, int64](q, x, c)
		default:
			panic("width")
		}
	}
	if e := json.NewEncoder(os.Stdout).Encode(rows); e != nil {
		panic(e)
	}
}
