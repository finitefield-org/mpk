// Independent typed Go negation and magnitudes, including the minimum value.
package main

import (
	"encoding/json"
	"fmt"
	"io"
	"math/big"
	"math/bits"
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
	sx := S(ux)
	sn := -sx
	neg := -ux
	abs := ux
	if sx < 0 {
		abs = U(-sx)
	}
	if U(sn) != neg {
		panic("typed signed and unsigned negations disagree")
	}
	format := func(v U) string { return fmt.Sprintf("%0*x", q.Width/4, uint64(v)) }
	_, co := bits.Add64(uint64(^ux), 0, 1)
	if q.Width < 64 {
		co = (uint64(^ux) + 1) >> uint(q.Width)
	}
	carry := uint64(1)
	stages := make([]map[string]any, q.Width)
	for i := 0; i < q.Width; i++ {
		a := (uint64(^ux) >> uint(i)) & 1
		sum := a + carry
		out := sum >> 1
		result := (uint64(neg) >> uint(i)) & 1
		if result != sum&1 {
			panic("native negation and adder stage disagree")
		}
		stages[i] = map[string]any{"index": i, "complement_bit": a != 0, "carry_in": carry != 0, "result_bit": result != 0, "carry_out": out != 0}
		carry = out
	}
	if carry != co {
		panic("native full-width and bit carries disagree")
	}
	cp, cn := 0, 0
	if sx < 0 {
		cp++
	}
	if sn < 0 {
		cp++
	}
	if ux != 0 {
		cn++
	}
	correction := cp - cn
	exact := new(big.Int).Neg(big.NewInt(int64(sx)))
	absolute := new(big.Int).Abs(big.NewInt(int64(sx)))
	modulus := new(big.Int).Lsh(big.NewInt(1), uint(q.Width))
	rebuilt := new(big.Int).Add(big.NewInt(int64(sn)), new(big.Int).Mul(modulus, big.NewInt(int64(correction))))
	noOverflow := exact.Cmp(big.NewInt(int64(sn))) == 0
	if rebuilt.Cmp(exact) != 0 || noOverflow != (cp == cn) || absolute.Cmp(new(big.Int).SetUint64(uint64(abs))) != 0 {
		panic("mathematical negation or magnitude disagreement")
	}
	return map[string]any{"id": q.ID, "neg_hex": format(neg), "abs_hex": format(abs), "signed_x": strconv.FormatInt(int64(sx), 10), "go_signed_negation": strconv.FormatInt(int64(sn), 10), "exact_negation": exact.String(), "absolute_value": absolute.String(), "carry": co != 0, "input_zero": ux == 0, "negation_zero": neg == 0, "magnitude_zero": abs == 0, "input_defined": ux != 0, "negation_defined": neg != 0, "magnitude_defined": abs != 0, "correction_positive": cp, "correction_negative": cn, "signed_correction": strconv.Itoa(correction), "no_overflow": noOverflow, "stages": stages}
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
			panic("closed original negation query")
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
