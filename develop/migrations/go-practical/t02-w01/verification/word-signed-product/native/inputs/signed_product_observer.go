// Independent typed signed multiplication, modular correction and exact integers.
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

func signedValue[T signed](x uint64) int64 { return int64(T(x)) }

func signedProduct[T signed](x, y uint64) int64 { return int64(T(x) * T(y)) }

func observe[T unsigned](q query, signedX, signedY, signedResult int64) map[string]any {
	x, err := strconv.ParseUint(q.X, 16, 64)
	if err != nil {
		panic(err)
	}
	y, err := strconv.ParseUint(q.Y, 16, 64)
	if err != nil {
		panic(err)
	}
	power := new(big.Int).Lsh(big.NewInt(1), uint(q.Width))
	product := new(big.Int).Mul(new(big.Int).SetUint64(x), new(big.Int).SetUint64(y))
	quotient := new(big.Int).Quo(new(big.Int).Set(product), power)
	format := func(v uint64) string { return fmt.Sprintf("%0*x", q.Width/4, v) }
	acc := T(0)
	stages := make([]map[string]any, q.Width)
	for i := 0; i < q.Width; i++ {
		shifted := T(x) << uint(i)
		selected := T(0)
		if (y>>uint(i))&1 != 0 {
			selected = shifted
		}
		_, carry := bits.Add64(uint64(acc), uint64(selected), 0)
		if q.Width < 64 {
			carry = (uint64(acc) + uint64(selected)) >> uint(q.Width)
		}
		acc += selected
		lost := new(big.Int).Lsh(new(big.Int).SetUint64(x), uint(i))
		lost.Quo(lost, power)
		stages[i] = map[string]any{"index": i, "left_hex": format(uint64(shifted)), "shift_discard": lost.String(), "carry": carry != 0, "acc_hex": format(uint64(acc))}
	}
	result := T(x) * T(y)
	if result != acc {
		panic("typed Go multiplication and native shift/add disagree")
	}
	signedExact := new(big.Int).Mul(big.NewInt(signedX), big.NewInt(signedY))
	positive := new(big.Int).Set(quotient)
	if signedResult < 0 {
		positive.Add(positive, big.NewInt(1))
	}
	if signedX < 0 && signedY < 0 {
		positive.Add(positive, power)
	}
	negative := new(big.Int)
	if signedY < 0 {
		negative.Add(negative, new(big.Int).SetUint64(x))
	}
	if signedX < 0 {
		negative.Add(negative, new(big.Int).SetUint64(y))
	}
	correction := new(big.Int).Sub(positive, negative)
	rebuilt := new(big.Int).Add(big.NewInt(signedResult), new(big.Int).Mul(power, correction))
	noOverflow := signedExact.Cmp(big.NewInt(signedResult)) == 0
	if rebuilt.Cmp(signedExact) != 0 || noOverflow != (positive.Cmp(negative) == 0) {
		panic("signed correction disagreement")
	}
	return map[string]any{"id": q.ID, "word_hex": format(uint64(result)), "go_signed_result": strconv.FormatInt(signedResult, 10), "unsigned_product": product.String(), "discarded_quotient": quotient.String(), "signed_x": strconv.FormatInt(signedX, 10), "signed_y": strconv.FormatInt(signedY, 10), "signed_product": signedExact.String(), "correction_positive": positive.String(), "correction_negative": negative.String(), "signed_correction": correction.String(), "no_overflow": noOverflow, "stages": stages}
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
		if q.Operation != "Mul" || q.Target != 0 {
			panic("closed native multiplication query")
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
			out[i] = observe[uint8](q, signedValue[int8](x), signedValue[int8](y), signedProduct[int8](x, y))
		case 16:
			out[i] = observe[uint16](q, signedValue[int16](x), signedValue[int16](y), signedProduct[int16](x, y))
		case 32:
			out[i] = observe[uint32](q, signedValue[int32](x), signedValue[int32](y), signedProduct[int32](x, y))
		case 64:
			out[i] = observe[uint64](q, signedValue[int64](x), signedValue[int64](y), signedProduct[int64](x, y))
		default:
			panic("closed word widths")
		}
	}
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
