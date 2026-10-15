// Private differential observer. This tool is not admitted Go-profile source.
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"
)

type input struct {
	Width       int    `json:"width"`
	Limit       string `json:"limit"`
	Accumulator string `json:"accumulator"`
	Item        string `json:"item"`
}

type observation struct {
	Remaining              string `json:"remaining"`
	Result                 string `json:"result"`
	LimitNonnegative       bool   `json:"limit_nonnegative"`
	AccumulatorNonnegative bool   `json:"accumulator_nonnegative"`
	AccumulatorWithinLimit bool   `json:"accumulator_within_limit"`
	ItemNonnegative        bool   `json:"item_nonnegative"`
	ItemWithinRemaining    bool   `json:"item_within_remaining"`
	RemainingNonnegative   bool   `json:"remaining_nonnegative"`
	ResultNonnegative      bool   `json:"result_nonnegative"`
	ResultWithinLimit      bool   `json:"result_within_limit"`
	AddGuard               bool   `json:"add_guard"`
	Continues              bool   `json:"continues"`
}

func observe[T int8 | int16 | int32 | int64](limit, accumulator, item T) observation {
	remaining := limit - accumulator
	result := accumulator + item
	return observation{
		Remaining: strconv.FormatInt(int64(remaining), 10), Result: strconv.FormatInt(int64(result), 10),
		LimitNonnegative: limit >= 0, AccumulatorNonnegative: accumulator >= 0,
		AccumulatorWithinLimit: accumulator <= limit, ItemNonnegative: item >= 0,
		ItemWithinRemaining: item <= remaining, RemainingNonnegative: remaining >= 0,
		ResultNonnegative: result >= 0, ResultWithinLimit: result <= limit,
		AddGuard:  (accumulator < 0) != (item < 0) || (result < 0) == (accumulator < 0),
		Continues: limit >= 0 && accumulator >= 0 && accumulator <= limit && item >= 0 && item <= remaining,
	}
}

func main() {
	if len(os.Args) != 2 {
		panic("input JSON file required")
	}
	data, err := os.ReadFile(os.Args[1])
	if err != nil {
		panic(err)
	}
	var inputs []input
	if err := json.Unmarshal(data, &inputs); err != nil {
		panic(err)
	}
	result := make([]observation, 0, len(inputs))
	for _, row := range inputs {
		values := make([]int64, 0, 3)
		for _, value := range []string{row.Limit, row.Accumulator, row.Item} {
			n, err := strconv.ParseInt(value, 10, row.Width)
			if err != nil {
				panic(err)
			}
			values = append(values, n)
		}
		l, a, x := values[0], values[1], values[2]
		switch row.Width {
		case 8:
			result = append(result, observe(int8(l), int8(a), int8(x)))
		case 16:
			result = append(result, observe(int16(l), int16(a), int16(x)))
		case 32:
			result = append(result, observe(int32(l), int32(a), int32(x)))
		case 64:
			result = append(result, observe(l, a, x))
		default:
			panic("only actual Go signed widths are observed")
		}
	}
	if err := json.NewEncoder(os.Stdout).Encode(result); err != nil {
		panic(fmt.Sprint(err))
	}
}
