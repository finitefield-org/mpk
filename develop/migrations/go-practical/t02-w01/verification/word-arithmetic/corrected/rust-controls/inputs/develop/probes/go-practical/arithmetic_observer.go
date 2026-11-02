// Independent native typed Go multiplication/shifts and exact big integers.
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
	ID    string `json:"id"`
	Width int    `json:"width"`
	X     string `json:"x_hex"`
	Y     string `json:"y_hex"`
}

type unsigned interface {
	~uint8 | ~uint16 | ~uint32 | ~uint64
}
type signed interface {
	~int8 | ~int16 | ~int32 | ~int64
}

func signedProduct[T signed](x, y uint64) int64 { return int64(T(x) * T(y)) }

func observe[T unsigned](q query, signedResult int64) map[string]any {
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
	return map[string]any{"id": q.ID, "word_hex": format(uint64(result)), "go_signed_result": strconv.FormatInt(signedResult, 10), "unsigned_product": product.String(), "discarded_quotient": quotient.String(), "stages": stages}
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
			out[i] = observe[uint8](q, signedProduct[int8](x, y))
		case 16:
			out[i] = observe[uint16](q, signedProduct[int16](x, y))
		case 32:
			out[i] = observe[uint32](q, signedProduct[int32](x, y))
		case 64:
			out[i] = observe[uint64](q, signedProduct[int64](x, y))
		default:
			panic("closed word widths")
		}
	}
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
