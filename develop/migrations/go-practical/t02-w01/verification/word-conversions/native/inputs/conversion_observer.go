// Typed Go conversion/count observations; math/big supplies independent balances.
package main

import (
	"encoding/json"
	"fmt"
	"io"
	"math/big"
	"os"
	"strconv"
	"strings"
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

func bit(b bool) int64 {
	if b {
		return 1
	}
	return 0
}
func shiftFault[T signed | unsigned](n T) (fault bool) {
	defer func() {
		if recover() != nil {
			fault = true
		}
	}()
	var one uint64 = 1
	_ = one << n
	return
}
func observe[U unsigned, S signed](q query, x uint64) map[string]any {
	u, s := U(x), S(x)
	if uint64(u) != x {
		panic("exact original width")
	}
	sg := strings.HasPrefix(q.Operation, "SignedTo")
	var r uint64
	switch q.Target {
	case 8:
		if sg {
			r = uint64(uint8(s))
		} else {
			r = uint64(uint8(u))
		}
	case 16:
		if sg {
			r = uint64(uint16(s))
		} else {
			r = uint64(uint16(u))
		}
	case 32:
		if sg {
			r = uint64(uint32(s))
		} else {
			r = uint64(uint32(u))
		}
	case 64:
		if sg {
			r = uint64(s)
		} else {
			r = uint64(u)
		}
	default:
		panic("target width")
	}
	ms := new(big.Int).Lsh(big.NewInt(1), uint(q.Width))
	mt := new(big.Int).Lsh(big.NewInt(1), uint(q.Target))
	source := new(big.Int).SetUint64(x)
	if sg {
		source.SetInt64(int64(s))
	}
	targetUnsigned := new(big.Int).SetUint64(r)
	targetSigned := new(big.Int).Set(targetUnsigned)
	targetNegative := (r >> uint(q.Target-1)) != 0
	if targetNegative {
		targetSigned.Sub(targetSigned, mt)
	}
	discard := uint64(0)
	var extension *big.Int
	if q.Width >= q.Target {
		discard = x >> uint(q.Target)
		balance := new(big.Int).Add(new(big.Int).Set(targetUnsigned), new(big.Int).Mul(new(big.Int).Set(mt), new(big.Int).SetUint64(discard)))
		if balance.Cmp(new(big.Int).SetUint64(x)) != 0 {
			panic("actual truncation")
		}
	} else {
		extension = new(big.Int).Sub(new(big.Int).Set(mt), ms)
		amount := new(big.Int).SetUint64(x)
		if sg && s < 0 {
			amount.Add(amount, extension)
		}
		if amount.Cmp(targetUnsigned) != 0 {
			panic("actual extension")
		}
	}
	cn := big.NewInt(0)
	if sg && s < 0 {
		cn.SetInt64(1)
		if q.Width >= q.Target {
			cn.Lsh(cn, uint(q.Width-q.Target))
		}
	}
	cu := new(big.Int).SetUint64(discard)
	cs := new(big.Int).Add(new(big.Int).Set(cu), big.NewInt(bit(targetNegative)))
	correctionUnsigned := new(big.Int).Sub(new(big.Int).Set(cu), cn)
	correctionSigned := new(big.Int).Sub(new(big.Int).Set(cs), cn)
	eq := func(value, correction *big.Int) bool {
		return new(big.Int).Add(new(big.Int).Set(value), new(big.Int).Mul(new(big.Int).Set(mt), correction)).Cmp(source) == 0
	}
	if !eq(targetUnsigned, correctionUnsigned) || !eq(targetSigned, correctionSigned) {
		panic("actual signed/unsigned corrected conversion")
	}
	exactSigned := sg && q.Width <= q.Target
	exactUnsigned := !sg && q.Width <= q.Target
	if exactSigned && targetSigned.Cmp(source) != 0 {
		panic("actual signed widening")
	}
	if exactUnsigned && targetUnsigned.Cmp(source) != 0 {
		panic("actual unsigned widening")
	}
	countWord := uint64(u)
	countFault := shiftFault(u)
	if sg {
		countWord = uint64(s)
		countFault = shiftFault(s)
	}
	countDefined := !sg || s >= 0
	if countFault == countDefined {
		panic("actual native negative shift guard")
	}
	if countDefined && countWord != x {
		panic("actual nonnegative count normalization")
	}
	var ext any
	if extension != nil {
		ext = extension.String()
	}
	return map[string]any{
		"id": q.ID, "operation": q.Operation, "result_hex": fmt.Sprintf("%0*x", q.Target/4, r),
		"source_value": source.String(), "target_unsigned": targetUnsigned.String(), "target_signed": targetSigned.String(),
		"source_modulus": ms.String(), "target_modulus": mt.String(), "discarded": strconv.FormatUint(discard, 10),
		"extension_mask": ext, "target_sign": targetNegative, "source_sign": s < 0,
		"correction_unsigned": correctionUnsigned.String(), "correction_signed": correctionSigned.String(),
		"unsigned_refinement": true, "signed_refinement": true, "exact_unsigned": exactUnsigned, "exact_signed": exactSigned,
		"count_word_hex": fmt.Sprintf("%016x", countWord), "count_defined": countDefined, "native_count_panicked": countFault,
	}
}
func main() {
	d := json.NewDecoder(os.Stdin)
	d.DisallowUnknownFields()
	var qs []query
	if err := d.Decode(&qs); err != nil {
		panic(err)
	}
	var trailing any
	if err := d.Decode(&trailing); err != io.EOF {
		panic("one original query document")
	}
	rows := make([]map[string]any, len(qs))
	for i, q := range qs {
		expected := fmt.Sprintf("UnsignedTo%d", q.Target)
		if strings.HasPrefix(q.Operation, "SignedTo") {
			expected = fmt.Sprintf("SignedTo%d", q.Target)
		}
		if q.Operation != expected || q.SignedCount || q.Count != "0000000000000000" || q.Y != strings.Repeat("0", q.Width/4) {
			panic("original complete conversion row")
		}
		x, e := strconv.ParseUint(q.X, 16, 64)
		if e != nil {
			panic(e)
		}
		switch q.Width {
		case 8:
			rows[i] = observe[uint8, int8](q, x)
		case 16:
			rows[i] = observe[uint16, int16](q, x)
		case 32:
			rows[i] = observe[uint32, int32](q, x)
		case 64:
			rows[i] = observe[uint64, int64](q, x)
		default:
			panic("source width")
		}
	}
	if err := json.NewEncoder(os.Stdout).Encode(rows); err != nil {
		panic(err)
	}
}
