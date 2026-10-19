// Independent pinned-Go word addition and exact big-integer observations.
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
	ID       string `json:"id"`
	Width    int    `json:"width"`
	X        string `json:"x_hex"`
	Y        string `json:"y_hex"`
	Incoming bool   `json:"incoming"`
	Initial  string `json:"initial_nat"`
}

type unsigned interface {
	~uint8 | ~uint16 | ~uint32 | ~uint64
}
type signed interface {
	~int8 | ~int16 | ~int32 | ~int64
}

func unsignedAdd[T unsigned](x, y, c uint64) uint64 { return uint64(T(x) + T(y) + T(c)) }
func signedAdd[T signed](x, y, c uint64) int64      { return int64(T(x) + T(y) + T(c)) }
func number(s string) uint64 {
	n, err := strconv.ParseUint(s, 16, 64)
	if err != nil {
		panic(err)
	}
	return n
}
func add(xs ...*big.Int) *big.Int {
	n := new(big.Int)
	for _, x := range xs {
		n.Add(n, x)
	}
	return n
}
func observe(q query) map[string]any {
	x, y := number(q.X), number(q.Y)
	var c uint64
	if q.Incoming {
		c = 1
	}
	var result uint64
	var resultSigned int64
	switch q.Width {
	case 8:
		result, resultSigned = unsignedAdd[uint8](x, y, c), signedAdd[int8](x, y, c)
	case 16:
		result, resultSigned = unsignedAdd[uint16](x, y, c), signedAdd[int16](x, y, c)
	case 32:
		result, resultSigned = unsignedAdd[uint32](x, y, c), signedAdd[int32](x, y, c)
	case 64:
		result, resultSigned = unsignedAdd[uint64](x, y, c), signedAdd[int64](x, y, c)
	default:
		panic("closed word widths")
	}
	_, carry := bits.Add64(x, y, c)
	if q.Width < 64 {
		carry = (x + y + c) >> q.Width
	}
	lowMask := ^uint64(0) >> (65 - q.Width)
	into := ((x & lowMask) + (y & lowMask) + c) >> (q.Width - 1)
	initial, ok := new(big.Int).SetString(q.Initial, 10)
	if !ok || initial.Sign() < 0 {
		panic("nonnegative exact initial Nat")
	}
	power := new(big.Int).Lsh(big.NewInt(1), uint(q.Width))
	ux, uy, ur := new(big.Int).SetUint64(x), new(big.Int).SetUint64(y), new(big.Int).SetUint64(result)
	sign := func(value uint64) *big.Int {
		if value>>(q.Width-1) != 0 {
			return new(big.Int).Set(power)
		}
		return new(big.Int)
	}
	signedX := new(big.Int).Sub(new(big.Int).Set(ux), sign(x))
	signedY := new(big.Int).Sub(new(big.Int).Set(uy), sign(y))
	unsignedLeft := add(initial, ux, uy, new(big.Int).SetUint64(c))
	unsignedRight := add(initial, ur, new(big.Int).Mul(power, new(big.Int).SetUint64(carry)))
	signedLeft := add(unsignedLeft, sign(result))
	signedRight := add(initial, ur, sign(x), sign(y))
	return map[string]any{
		"id": q.ID, "word_hex": fmt.Sprintf("%0*x", q.Width/4, result),
		"carry": carry != 0, "sign_incoming": into != 0, "no_overflow": into == carry,
		"go_signed_result": strconv.FormatInt(resultSigned, 10),
		"unsigned_left":    unsignedLeft.String(), "unsigned_right": unsignedRight.String(),
		"signed_cross_left": signedLeft.String(), "signed_cross_right": signedRight.String(),
		"integer_sum": add(signedX, signedY, new(big.Int).SetUint64(c)).String(),
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
		panic("one query document")
	}
	out := make([]map[string]any, len(qs))
	for i, q := range qs {
		out[i] = observe(q)
	}
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
