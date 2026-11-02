// Independent actual Go signed division ranges and minimum/-one correction.
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

func calculate[S signed](x, y uint64, remainder bool) (result S, panicked bool) {
	defer func() {
		if recover() != nil {
			panicked = true
		}
	}()
	if remainder {
		result = S(x) % S(y)
	} else {
		result = S(x) / S(y)
	}
	return
}
func bit(value bool) int64 {
	if value {
		return 1
	}
	return 0
}
func observe[U unsigned, S signed](q query, x, y uint64) map[string]any {
	ux, uy := U(x), U(y)
	if uint64(ux) != x || uint64(uy) != y {
		panic("original full word width")
	}
	quotient, qp := calculate[S](x, y, false)
	remainder, rp := calculate[S](x, y, true)
	defined := uy != 0
	if qp == defined || rp == defined {
		panic("real Go source fault/guard")
	}
	row := map[string]any{"id": q.ID, "operation": q.Operation, "defined": defined, "panicked": !defined}
	for _, key := range []string{"abs_x_hex", "abs_y_hex", "q_amount_hex", "r_amount_hex", "quotient_hex", "remainder_hex", "signed_x", "signed_y", "signed_quotient", "signed_remainder", "half", "modulus", "input_gap_x", "input_gap_y", "quotient_gap", "quotient_bounded", "remainder_strict_half", "negative_unit", "quotient_overflow", "minimum_minus_one", "quotient_no_overflow", "quotient_correction", "remainder_correction", "corrected_conservation", "exact_conservation", "minimum_results"} {
		row[key] = nil
	}
	if !defined {
		return row
	}
	ax, ay := ux, uy
	if S(ux) < 0 {
		ax = U(0) - ux
	}
	if S(uy) < 0 {
		ay = U(0) - uy
	}
	qa, ra := ax/ay, ax%ay
	modulus := new(big.Int).Lsh(big.NewInt(1), uint(q.Width))
	half := new(big.Int).Rsh(new(big.Int).Set(modulus), 1)
	gx := new(big.Int).Sub(new(big.Int).Set(half), new(big.Int).SetUint64(uint64(ax)))
	gy := new(big.Int).Sub(new(big.Int).Set(half), new(big.Int).SetUint64(uint64(ay)))
	gap := new(big.Int).Sub(new(big.Int).SetUint64(uint64(ay)), big.NewInt(1))
	gap.Mul(gap, new(big.Int).SetUint64(uint64(qa)))
	gap.Add(gap, new(big.Int).SetUint64(uint64(ra)))
	gap.Add(gap, gx)
	if new(big.Int).Add(new(big.Int).SetUint64(uint64(qa)), gap).Cmp(half) != 0 {
		panic("exact quotient gap")
	}
	h := U(1) << uint(q.Width-1)
	qBound, rBound := qa <= h, ra < h
	if !qBound || !rBound || gx.Sign() < 0 || gy.Sign() < 0 {
		panic("actual half bounds")
	}
	qFlag := (S(ux) < 0) != (S(uy) < 0)
	rFlag := S(ux) < 0
	correction := func(amount U, flag bool) int64 {
		positive := bit(S(amount) < 0)
		if flag {
			positive = bit(S(U(0)-amount) < 0)
		}
		return positive - bit(flag && amount != 0)
	}
	qc, rc := correction(qa, qFlag), correction(ra, rFlag)
	overflow := !qFlag && qa == h
	negativeUnit := S(uy) == S(-1)
	if negativeUnit != ((S(uy) < 0) && ay == 1) {
		panic("actual negative unit meaning")
	}
	minimum := ux == h && negativeUnit
	if overflow != minimum || qc != bit(minimum) || rc != 0 {
		panic("actual correction classification")
	}
	sx, sy := big.NewInt(int64(S(ux))), big.NewInt(int64(S(uy)))
	sq, sr := big.NewInt(int64(quotient)), big.NewInt(int64(remainder))
	correctedQ := new(big.Int).Add(new(big.Int).Set(sq), new(big.Int).Mul(new(big.Int).Set(modulus), big.NewInt(qc)))
	corrected := new(big.Int).Add(new(big.Int).Mul(new(big.Int).Set(sy), correctedQ), sr).Cmp(sx) == 0
	exact := new(big.Int).Add(new(big.Int).Mul(new(big.Int).Set(sy), sq), sr).Cmp(sx) == 0
	if !corrected || exact == minimum {
		panic("actual source signed reconstruction")
	}
	minimumResults := minimum && U(quotient) == h && remainder == 0
	if minimum && !minimumResults {
		panic("normal minimum/-one outputs")
	}
	format := func(v U) string { return fmt.Sprintf("%0*x", q.Width/4, uint64(v)) }
	for key, value := range map[string]any{
		"abs_x_hex": format(ax), "abs_y_hex": format(ay), "q_amount_hex": format(qa), "r_amount_hex": format(ra), "quotient_hex": format(U(quotient)), "remainder_hex": format(U(remainder)),
		"signed_x": sx.String(), "signed_y": sy.String(), "signed_quotient": sq.String(), "signed_remainder": sr.String(), "half": half.String(), "modulus": modulus.String(),
		"input_gap_x": gx.String(), "input_gap_y": gy.String(), "quotient_gap": gap.String(), "quotient_bounded": qBound, "remainder_strict_half": rBound,
		"negative_unit": negativeUnit, "quotient_overflow": overflow, "minimum_minus_one": minimum, "quotient_no_overflow": !minimum, "quotient_correction": qc, "remainder_correction": rc,
		"corrected_conservation": corrected, "exact_conservation": exact, "minimum_results": minimumResults,
	} {
		row[key] = value
	}
	return row
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
		panic("one original query document")
	}
	out := make([]map[string]any, len(rows))
	for i, q := range rows {
		if (q.Operation != "SignedDiv" && q.Operation != "SignedRem") || q.Target != 0 || q.SignedCount || q.Count != "0000000000000000" {
			panic("closed original signed queries")
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
			panic("original width")
		}
	}
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
