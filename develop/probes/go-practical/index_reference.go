// Private original-Go index observations; recovery is outside the proof closure.
package main

import (
	"encoding/json"
	"os"
	"strconv"
)

type indexInput struct {
	Capacity int    `json:"capacity"`
	Length   int    `json:"length"`
	Index    string `json:"index"`
}
type indexOutput struct {
	Bound         bool   `json:"bound"`
	SourceGuard   bool   `json:"source_guard"`
	MaskGuard     bool   `json:"mask_guard"`
	Covered       bool   `json:"covered"`
	ActualDefined bool   `json:"actual_defined"`
	ActualByte    int    `json:"actual_byte"`
	UnsignedIndex string `json:"unsigned_index"`
	LowIndex      string `json:"low_index"`
}

func observeIndex(value string, index int64) (observed int, defined bool) {
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
	raw, err := os.ReadFile(os.Args[1])
	if err != nil {
		panic(err)
	}
	var cases []indexInput
	if err := json.Unmarshal(raw, &cases); err != nil {
		panic(err)
	}
	outputs := make([]indexOutput, 0, len(cases))
	for _, row := range cases {
		if row.Length < 0 || row.Length > row.Capacity+1 || row.Capacity != 0 && row.Capacity != 1 && row.Capacity != 32 && row.Capacity != 256 {
			panic("invalid index workload")
		}
		index, err := strconv.ParseInt(row.Index, 10, 64)
		if err != nil {
			panic(err)
		}
		data := make([]byte, row.Length)
		for i := range data {
			data[i] = byte(i*31 + 17)
		}
		value := string(data)
		observed, defined := observeIndex(value, index)
		var low uint64
		if row.Capacity > 0 {
			low = uint64(index) & uint64(row.Capacity-1)
		}
		outputs = append(outputs, indexOutput{Bound: len(value) <= row.Capacity, SourceGuard: index >= 0 && index < int64(len(value)),
			MaskGuard: index >= 0 && uint64(index) < uint64(len(value)), Covered: row.Capacity > 0 && uint64(index) == low,
			ActualDefined: defined, ActualByte: observed, UnsignedIndex: strconv.FormatUint(uint64(index), 10), LowIndex: strconv.FormatUint(low, 10)})
	}
	if err := json.NewEncoder(os.Stdout).Encode(outputs); err != nil {
		panic(err)
	}
}
