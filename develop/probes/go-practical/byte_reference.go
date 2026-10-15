// Private ordinary-Go observations; no source-to-certificate claim.
package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"os"
	"strconv"
)

type byteInput struct {
	Capacity int    `json:"capacity"`
	Left     string `json:"left"`
	Right    string `json:"right"`
	LeftLen  int    `json:"left_length"`
	RightLen int    `json:"right_length"`
	Index    string `json:"index"`
}
type byteOutput struct {
	Equal          bool `json:"equal"`
	Less           bool `json:"less"`
	MaskedEqual    bool `json:"masked_equal"`
	MaskedLess     bool `json:"masked_less"`
	RawEqual       bool `json:"raw_equal"`
	RawLess        bool `json:"raw_less"`
	IndexDefined   bool `json:"index_defined"`
	IndexByte      int  `json:"index_byte"`
	LeftLength     int  `json:"left_length"`
	RightLength    int  `json:"right_length"`
	InputUnchanged bool `json:"input_unchanged"`
}

// This observer is outside the proof closure. It records real Go index
// panics as well as successful reads, without admitting recovery in v1.
func observeByteIndex(value string, index int64) (observed int, defined bool) {
	observed = -1
	defer func() {
		if recover() != nil {
			defined = false
		}
	}()
	observed = int(value[index])
	defined = true
	return
}

func main() {
	if len(os.Args) != 2 {
		panic("input JSON file required")
	}
	data, err := os.ReadFile(os.Args[1])
	if err != nil {
		panic(err)
	}
	var inputs []byteInput
	if err := json.Unmarshal(data, &inputs); err != nil {
		panic(err)
	}
	outputs := make([]byteOutput, 0, len(inputs))
	for _, row := range inputs {
		left, err := hex.DecodeString(row.Left)
		if err != nil {
			panic(err)
		}
		right, err := hex.DecodeString(row.Right)
		if err != nil {
			panic(err)
		}
		storage := row.Capacity
		if storage == 0 {
			storage = 1
		}
		if len(left) != storage || len(right) != storage || row.LeftLen < 0 || row.LeftLen > row.Capacity || row.RightLen < 0 || row.RightLen > row.Capacity {
			panic("invalid bounded input")
		}
		index, err := strconv.ParseInt(row.Index, 10, 64)
		if err != nil {
			panic(err)
		}
		beforeLeft, beforeRight := bytes.Clone(left), bytes.Clone(right)
		a, b := string(left[:row.LeftLen]), string(right[:row.RightLen])
		maskedLeft, maskedRight := bytes.Clone(left), bytes.Clone(right)
		for i := row.LeftLen; i < len(maskedLeft); i++ {
			maskedLeft[i] = 0
		}
		for i := row.RightLen; i < len(maskedRight); i++ {
			maskedRight[i] = 0
		}
		ma, mb := string(maskedLeft[:row.LeftLen]), string(maskedRight[:row.RightLen])
		observed, defined := observeByteIndex(a, index)
		outputs = append(outputs, byteOutput{Equal: a == b, Less: a < b, MaskedEqual: ma == mb, MaskedLess: ma < mb,
			RawEqual: string(left) == string(right), RawLess: string(left) < string(right), IndexDefined: defined, IndexByte: observed,
			LeftLength: len(a), RightLength: len(b), InputUnchanged: bytes.Equal(left, beforeLeft) && bytes.Equal(right, beforeRight)})
	}
	if err := json.NewEncoder(os.Stdout).Encode(outputs); err != nil {
		panic(err)
	}
}
