// Independent typed Go half ranges, lower bits and minimum magnitudes.
package main

import (
	"encoding/json"
	"fmt"
	"io"
	"math/big"
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

func observe[U unsigned, S signed](q query, x uint64) map[string]any {
	ux := U(x)
	if uint64(ux) != x {
		panic("original word width")
	}
	half := U(1) << uint(q.Width-1)
	low := ux & (half - 1)
	complement := (^low) & (half - 1)
	neg := -ux
	mag := ux
	sourceSign := S(ux) < 0
	if sourceSign {
		mag = U(-S(ux))
	}
	gap := low
	if !sourceSign {
		gap = complement + 1
	}
	absolute := new(big.Int).Abs(big.NewInt(int64(S(ux))))
	if absolute.Cmp(new(big.Int).SetUint64(uint64(mag))) != 0 {
		panic("typed Go and exact absolute value")
	}
	if uint64(low)+uint64(complement)+1 != uint64(half) || uint64(mag)+uint64(gap) != uint64(half) {
		panic("computed half gap")
	}
	minimum := ux == half
	if (mag == half) != minimum || (S(mag) < 0) != minimum || !sourceSign && mag >= half {
		panic("minimum/half/sign correspondence")
	}
	modulus := new(big.Int).Lsh(new(big.Int).SetUint64(uint64(half)), 1)
	format := func(v U) string { return fmt.Sprintf("%0*x", q.Width/4, uint64(v)) }
	decimal := func(v U) string { return strconv.FormatUint(uint64(v), 10) }
	return map[string]any{"id": q.ID, "word_hex": format(ux), "neg_hex": format(neg), "abs_hex": format(mag), "signed_x": strconv.FormatInt(int64(S(ux)), 10), "half": decimal(half), "modulus": modulus.String(), "low": decimal(low), "lower_complement": decimal(complement), "gap": decimal(gap), "source_sign": sourceSign, "negation_sign": S(neg) < 0, "magnitude_sign": S(mag) < 0, "unsigned_below_half": ux < half, "magnitude_half_bound": mag <= half, "magnitude_equals_half": mag == half, "input_is_minimum": minimum}
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
		if q.Operation != "Neg" || q.Target != 0 || q.SignedCount || q.Count != "0000000000000000" {
			panic("closed original half-range query")
		}
		x, err := strconv.ParseUint(q.X, 16, 64)
		if err != nil {
			panic(err)
		}
		switch q.Width {
		case 8:
			out[i] = observe[uint8, int8](q, x)
		case 16:
			out[i] = observe[uint16, int16](q, x)
		case 32:
			out[i] = observe[uint32, int32](q, x)
		case 64:
			out[i] = observe[uint64, int64](q, x)
		default:
			panic("closed word widths")
		}
	}
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
