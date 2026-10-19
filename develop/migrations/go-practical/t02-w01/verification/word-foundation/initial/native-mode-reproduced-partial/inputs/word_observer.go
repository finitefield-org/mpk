// Independent pinned-Go observations. This is not a trusted frontend or proof.
package main

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"strconv"
	"strings"
)

type query struct {
	ID          string `json:"id"`
	Width       int    `json:"width"`
	Operation   string `json:"operation"`
	X           string `json:"x_hex"`
	Y           string `json:"y_hex"`
	Count       string `json:"count_hex"`
	SignedCount bool   `json:"signed_count"`
	Target      int    `json:"target_width"`
}

type integer interface {
	~uint8 | ~uint16 | ~uint32 | ~uint64 | ~int8 | ~int16 | ~int32 | ~int64
}

func number(s string) uint64 {
	v, err := strconv.ParseUint(s, 16, 64)
	if err != nil {
		panic(err)
	}
	return v
}

func execute[T integer](q query) (out map[string]any) {
	defer func() {
		if failure := recover(); failure != nil {
			out = map[string]any{"id": q.ID, "kind": "runtime_safety_failure", "detail": fmt.Sprint(failure)}
		}
	}()
	x, y := T(number(q.X)), T(number(q.Y))
	count := number(q.Count)
	var result T
	var comparison bool
	boolean := false
	width := q.Width
	switch q.Operation {
	case "Add":
		result = x + y
	case "Sub":
		result = x - y
	case "Mul":
		result = x * y
	case "Neg":
		result = -x
	case "Not":
		result = ^x
	case "And":
		result = x & y
	case "Or":
		result = x | y
	case "Xor":
		result = x ^ y
	case "AndNot":
		result = x &^ y
	case "Equal":
		boolean, comparison = true, x == y
	case "NotEqual":
		boolean, comparison = true, x != y
	case "UnsignedLess", "SignedLess":
		boolean, comparison = true, x < y
	case "UnsignedLessEqual", "SignedLessEqual":
		boolean, comparison = true, x <= y
	case "UnsignedDiv", "SignedDiv":
		result = x / y
	case "UnsignedRem", "SignedRem":
		result = x % y
	case "ShiftLeft":
		if q.SignedCount {
			result = x << int64(count)
		} else {
			result = x << count
		}
	case "ShiftRight", "SignedShiftRight":
		if q.SignedCount {
			result = x >> int64(count)
		} else {
			result = x >> count
		}
	default:
		if !strings.Contains(q.Operation, "To") || q.Target != 8 && q.Target != 16 && q.Target != 32 && q.Target != 64 {
			panic("unregistered observation operation")
		}
		// Actual Go conversions first use the selected signed/unsigned source T.
		width = q.Target
		switch width {
		case 8:
			return bits(q.ID, width, uint64(uint8(x)))
		case 16:
			return bits(q.ID, width, uint64(uint16(x)))
		case 32:
			return bits(q.ID, width, uint64(uint32(x)))
		case 64:
			return bits(q.ID, width, uint64(x))
		}
	}
	if boolean {
		return map[string]any{"id": q.ID, "kind": "boolean", "value": comparison}
	}
	return bits(q.ID, width, uint64(result))
}

func bits(id string, width int, value uint64) map[string]any {
	mask := ^uint64(0) >> (64 - width)
	return map[string]any{"id": id, "kind": "word", "value_hex": fmt.Sprintf("%0*x", width/4, value&mask)}
}

func observe(q query) map[string]any {
	signed := strings.HasPrefix(q.Operation, "Signed")
	switch q.Width {
	case 8:
		if signed {
			return execute[int8](q)
		}
		return execute[uint8](q)
	case 16:
		if signed {
			return execute[int16](q)
		}
		return execute[uint16](q)
	case 32:
		if signed {
			return execute[int32](q)
		}
		return execute[uint32](q)
	case 64:
		if signed {
			return execute[int64](q)
		}
		return execute[uint64](q)
	}
	panic("closed word widths")
}

func main() {
	decoder := json.NewDecoder(os.Stdin)
	decoder.DisallowUnknownFields()
	var queries []query
	if err := decoder.Decode(&queries); err != nil {
		panic(err)
	}
	var trailing any
	if err := decoder.Decode(&trailing); err != io.EOF {
		panic("one complete query document")
	}
	out := make([]map[string]any, len(queries))
	for i, q := range queries {
		out[i] = observe(q)
	}
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
