// Independent native unsigned division and restoring-step observations.
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
	ID          string `json:"id"`
	Width       int    `json:"width"`
	Operation   string `json:"operation"`
	X           string `json:"x_hex"`
	Y           string `json:"y_hex"`
	Count       string `json:"count_hex"`
	SignedCount bool   `json:"signed_count"`
	Target      int    `json:"target_width"`
}
type stepQuery struct {
	ID    string `json:"id"`
	Width int    `json:"width"`
	R     string `json:"r_hex"`
	D     string `json:"d_hex"`
	Low   bool   `json:"low"`
}
type unsigned interface {
	~uint8 | ~uint16 | ~uint32 | ~uint64
}

func number(s string) uint64 {
	n, err := strconv.ParseUint(s, 16, 64)
	if err != nil {
		panic(err)
	}
	return n
}
func word[T unsigned](x T, w int) string { return fmt.Sprintf("%0*x", w/4, uint64(x)) }

func step[T unsigned](r, d T, low bool, w int) (T, map[string]any) {
	var digit T
	if low {
		digit = 1
	}
	high := r>>uint(w-1) != 0
	s := (r << 1) | digit
	sub := s - d
	var carry bool
	if w == 64 {
		_, c := bits.Add64(uint64(s), uint64(^d), 1)
		carry = c != 0
	} else {
		carry = (uint64(s)+uint64(^d)+1)>>uint(w) != 0
	}
	if carry != (s >= d) {
		panic("native carry disagreement")
	}
	decision := high || s >= d
	next := s
	if decision {
		next = sub
	}
	if r < d {
		left := new(big.Int).Lsh(new(big.Int).SetUint64(uint64(r)), 1)
		if low {
			left.Add(left, big.NewInt(1))
		}
		right := new(big.Int).SetUint64(uint64(next))
		if decision {
			right.Add(right, new(big.Int).SetUint64(uint64(d)))
		}
		if left.Cmp(right) != 0 || next >= d {
			panic("native restore arithmetic disagreement")
		}
	}
	return next, map[string]any{"prior_hex": word(r, w), "shifted_hex": word(s, w), "sub_hex": word(sub, w), "remainder_hex": word(next, w), "low": low, "high": high, "carry": carry, "decision": decision}
}
func divide[T unsigned](q query) (out map[string]any) {
	x, y := T(number(q.X)), T(number(q.Y))
	defer func() {
		if e := recover(); e != nil {
			if y != 0 {
				panic(e)
			}
			out = map[string]any{"id": q.ID, "kind": "runtime_safety_failure", "reason": "division_by_zero", "guard": false}
		}
	}()
	var result T
	if q.Operation == "UnsignedDiv" {
		result = x / y
	} else if q.Operation == "UnsignedRem" {
		result = x % y
	} else {
		panic("closed unsigned operation")
	}
	var r T
	dp, qp := new(big.Int), new(big.Int)
	stages := []map[string]any{}
	for i := 0; i < q.Width; i++ {
		low := (x>>uint(q.Width-1-i))&1 != 0
		var row map[string]any
		r, row = step(r, y, low, q.Width)
		dp.Lsh(dp, 1)
		if low {
			dp.Add(dp, big.NewInt(1))
		}
		qp.Lsh(qp, 1)
		if row["decision"].(bool) {
			qp.Add(qp, big.NewInt(1))
		}
		row["index"], row["dividend_prefix"], row["quotient_prefix"] = i, dp.String(), qp.String()
		stages = append(stages, row)
	}
	if qp.Uint64() != uint64(x/y) || r != x%y {
		panic("native restoring stages differ from Go operators")
	}
	return map[string]any{"id": q.ID, "kind": "word", "word_hex": word(result, q.Width), "guard": true, "stages": stages}
}
func restore[T unsigned](q stepQuery) map[string]any {
	r, d := T(number(q.R)), T(number(q.D))
	if r >= d {
		panic("step input outside stated domain")
	}
	_, out := step(r, d, q.Low, q.Width)
	out["id"] = q.ID
	return out
}
func main() {
	var input struct {
		Divisions []query     `json:"divisions"`
		Steps     []stepQuery `json:"steps"`
	}
	d := json.NewDecoder(os.Stdin)
	d.DisallowUnknownFields()
	if err := d.Decode(&input); err != nil {
		panic(err)
	}
	if err := d.Decode(new(any)); err != io.EOF {
		panic("trailing input")
	}
	divisions := []map[string]any{}
	steps := []map[string]any{}
	for _, q := range input.Divisions {
		var row map[string]any
		switch q.Width {
		case 8:
			row = divide[uint8](q)
		case 16:
			row = divide[uint16](q)
		case 32:
			row = divide[uint32](q)
		case 64:
			row = divide[uint64](q)
		default:
			panic("closed width")
		}
		divisions = append(divisions, row)
	}
	for _, q := range input.Steps {
		var row map[string]any
		switch q.Width {
		case 8:
			row = restore[uint8](q)
		case 16:
			row = restore[uint16](q)
		case 32:
			row = restore[uint32](q)
		case 64:
			row = restore[uint64](q)
		default:
			panic("closed width")
		}
		steps = append(steps, row)
	}
	if err := json.NewEncoder(os.Stdout).Encode(map[string]any{"divisions": divisions, "steps": steps}); err != nil {
		panic(err)
	}
}
