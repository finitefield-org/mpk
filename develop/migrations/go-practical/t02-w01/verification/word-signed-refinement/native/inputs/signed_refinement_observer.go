// Independent typed Go source words and arbitrary sign-selection corrections.
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
func bit(b bool) int64 {
	if b {
		return 1
	}
	return 0
}
func observe[U unsigned, S signed](q query, x, y uint64) []map[string]any {
	ux, uy := U(x), U(y)
	if uint64(ux) != x || uint64(uy) != y {
		panic("original full word width")
	}
	format := func(v U) string { return fmt.Sprintf("%0*x", q.Width/4, uint64(v)) }
	makeRow := func(kind string, flag bool, amount U, result S) map[string]any {
		negated := U(0) - amount
		selected := amount
		if flag {
			selected = negated
		}
		if selected != U(result) {
			panic("actual typed source/sign-selection correspondence")
		}
		cp := bit(S(amount) < 0)
		if flag {
			cp = bit(S(negated) < 0)
		}
		cn := bit(flag && amount != 0)
		mathematical := new(big.Int).SetUint64(uint64(amount))
		if flag {
			mathematical.Neg(mathematical)
		}
		modulus := new(big.Int).Lsh(big.NewInt(1), uint(q.Width))
		corrected := new(big.Int).Mul(modulus, big.NewInt(cp-cn))
		corrected.Add(corrected, big.NewInt(int64(result)))
		if mathematical.Cmp(corrected) != 0 {
			panic("independent exact signed correction")
		}
		return map[string]any{
			"kind": kind, "id": q.ID, "operation": q.Operation, "flag": flag,
			"defined": true, "panicked": false, "amount_hex": format(amount),
			"negated_amount_hex": format(negated), "result_hex": format(U(result)),
			"mathematical_result": mathematical.String(), "go_signed_result": strconv.FormatInt(int64(result), 10),
			"correction_positive": cp, "correction_negative": cn, "correction": cp - cn,
		}
	}
	if q.Operation == "Neg" {
		return []map[string]any{
			makeRow("selection", false, ux, S(ux)),
			makeRow("selection", true, ux, S(U(0)-ux)),
		}
	}
	flag := S(ux) < 0
	if q.Operation == "SignedDiv" {
		flag = flag != (S(uy) < 0)
	}
	result, panicked := calculate[S](x, y, q.Operation)
	defined := uy != 0
	if panicked == defined {
		panic("actual native fault/guard mismatch")
	}
	if !defined {
		return []map[string]any{{
			"kind": "arithmetic", "id": q.ID, "operation": q.Operation, "flag": flag,
			"defined": false, "panicked": true, "amount_hex": nil,
			"negated_amount_hex": nil, "result_hex": nil, "mathematical_result": nil,
			"go_signed_result": nil, "correction_positive": nil, "correction_negative": nil, "correction": nil,
		}}
	}
	ax, ay := ux, uy
	if S(ux) < 0 {
		ax = U(0) - ux
	}
	if S(uy) < 0 {
		ay = U(0) - uy
	}
	var amount U
	if q.Operation == "SignedDiv" {
		amount = ax / ay
	} else {
		amount = ax % ay
	}
	row := makeRow("arithmetic", flag, amount, result)
	qx, rx := new(big.Int), new(big.Int)
	qx.QuoRem(big.NewInt(int64(S(ux))), big.NewInt(int64(S(uy))), rx)
	mathematical := qx
	if q.Operation == "SignedRem" {
		mathematical = rx
	}
	if mathematical.String() != row["mathematical_result"] {
		panic("independent unbounded signed quotient/remainder")
	}
	return []map[string]any{row}
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
	out := []map[string]any{}
	for _, q := range rows {
		if (q.Operation != "Neg" && q.Operation != "SignedDiv" && q.Operation != "SignedRem") || q.Target != 0 || q.SignedCount || q.Count != "0000000000000000" {
			panic("closed original query rows")
		}
		x, err := strconv.ParseUint(q.X, 16, 64)
		if err != nil {
			panic(err)
		}
		y, err := strconv.ParseUint(q.Y, 16, 64)
		if err != nil {
			panic(err)
		}
		var result []map[string]any
		switch q.Width {
		case 8:
			result = observe[uint8, int8](q, x, y)
		case 16:
			result = observe[uint16, int16](q, x, y)
		case 32:
			result = observe[uint32, int32](q, x, y)
		case 64:
			result = observe[uint64, int64](q, x, y)
		default:
			panic("closed original word width")
		}
		out = append(out, result...)
	}
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
