// Independent typed signed/unsigned equality observations, never proof authority.
package main

import (
	"encoding/json"
	"io"
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

type signed interface {
	~int8 | ~int16 | ~int32 | ~int64
}
type unsigned interface {
	~uint8 | ~uint16 | ~uint32 | ~uint64
}

func number(text string, width int) uint64 {
	if len(text) != width/4 {
		panic("complete original word payload required")
	}
	n, err := strconv.ParseUint(text, 16, width)
	if err != nil {
		panic(err)
	}
	return n
}

func execute[S signed, U unsigned](q query) map[string]any {
	a, b := number(q.X, q.Width), number(q.Y, q.Width)
	x, y := S(a), S(b)
	u, v := U(a), U(b)
	var signedValue, unsignedValue bool
	if q.Operation == "Equal" {
		signedValue, unsignedValue = x == y, u == v
	} else if q.Operation == "NotEqual" {
		signedValue, unsignedValue = x != y, u != v
	} else {
		panic("closed equality operations")
	}
	return map[string]any{"id": q.ID, "kind": "boolean", "operation": q.Operation,
		"width": q.Width, "value": signedValue, "unsigned_value": unsignedValue,
		"signed_x": strconv.FormatInt(int64(x), 10), "signed_y": strconv.FormatInt(int64(y), 10)}
}

func main() {
	decoder := json.NewDecoder(os.Stdin)
	decoder.DisallowUnknownFields()
	var queries []query
	if err := decoder.Decode(&queries); err != nil {
		panic(err)
	}
	var trailing any
	if err := decoder.Decode(&trailing); err != io.EOF {
		panic("one original query document")
	}
	out := []map[string]any{}
	for _, q := range queries {
		if q.Operation != "Equal" && q.Operation != "NotEqual" {
			continue
		}
		var row map[string]any
		switch q.Width {
		case 8:
			row = execute[int8, uint8](q)
		case 16:
			row = execute[int16, uint16](q)
		case 32:
			row = execute[int32, uint32](q)
		case 64:
			row = execute[int64, uint64](q)
		default:
			panic("closed full widths")
		}
		out = append(out, row)
	}
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
