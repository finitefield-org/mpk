// Independent actual typed Go zero guards, signed division and remainder.
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

func allZero[U unsigned](x U, width int) bool {
	z := true
	for i := 0; i < width; i++ {
		z = z && ((x>>uint(i))&1 == 0)
	}
	return z
}
func calculate[S signed](x, y uint64, operation string) (result S, panicked bool) {
	defer func() {
		if recover() != nil {
			panicked = true
		}
	}()
	if operation == "SignedDiv" {
		result = S(x) / S(y)
	} else {
		result = S(x) % S(y)
	}
	return
}
func observe[U unsigned, S signed](q query, x, y uint64) map[string]any {
	ux, uy := U(x), U(y)
	if uint64(ux) != x || uint64(uy) != y {
		panic("original word width")
	}
	format := func(v U) string { return fmt.Sprintf("%0*x", q.Width/4, uint64(v)) }
	if q.Operation == "Neg" {
		return map[string]any{"kind": "zero", "id": q.ID, "word_hex": format(ux), "is_zero": ux == 0, "divide_defined": ux != 0, "zero_bits": allZero(ux, q.Width), "source_sign": S(ux) < 0}
	}
	result, panicked := calculate[S](x, y, q.Operation)
	defined := uy != 0
	if panicked == defined {
		panic("actual Go fault/guard mismatch")
	}
	var value any
	var signedResult any
	if defined {
		value = format(U(result))
		signedResult = strconv.FormatInt(int64(result), 10)
	}
	ay := uy
	if S(uy) < 0 {
		ay = U(-S(uy))
	}
	return map[string]any{"kind": "arithmetic", "id": q.ID, "operation": q.Operation, "defined": defined, "panicked": panicked, "value_hex": value, "go_signed_result": signedResult, "x_zero": ux == 0, "y_zero": uy == 0, "x_defined": ux != 0, "y_defined": uy != 0, "x_zero_bits": allZero(ux, q.Width), "y_zero_bits": allZero(uy, q.Width), "x_sign": S(ux) < 0, "y_sign": S(uy) < 0, "divisor_magnitude_zero": ay == 0}
}
func main() {
	d := json.NewDecoder(os.Stdin)
	d.DisallowUnknownFields()
	var rows []query
	if err := d.Decode(&rows); err != nil {
		panic(err)
	}
	var trailing any
	if err := d.Decode(&trailing); err != io.EOF {
		panic("one query document")
	}
	out := make([]map[string]any, len(rows))
	for i, q := range rows {
		if (q.Operation != "Neg" && q.Operation != "SignedDiv" && q.Operation != "SignedRem") || q.Target != 0 || q.SignedCount || q.Count != "0000000000000000" {
			panic("closed original queries")
		}
		x, err := strconv.ParseUint(q.X, 16, 64)
		if err != nil {
			panic(err)
		}
		y, err := strconv.ParseUint(q.Y, 16, 64)
		if err != nil {
			panic(err)
		}
		switch q.Width {
		case 8:
			out[i] = observe[uint8, int8](q, x, y)
		case 16:
			out[i] = observe[uint16, int16](q, x, y)
		case 32:
			out[i] = observe[uint32, int32](q, x, y)
		case 64:
			out[i] = observe[uint64, int64](q, x, y)
		default:
			panic("closed word width")
		}
	}
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
