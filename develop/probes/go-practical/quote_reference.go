// Private original-Go observer. Only the arithmetic prefix model is proved.
package main

import (
	"encoding/json"
	"os"
	"strconv"
)

type Amount int64
type Line struct{ Amount Amount }
type Decision struct{ Total Amount }
type RejectCode uint8

const (
	BadInput RejectCode = iota + 1
	LimitExceeded
)

func (RejectCode) Error() string { return "order rejected" }

func Quote(customerID string, lines []Line, limit Amount) (Decision, error) {
	if customerID == "" || limit < 0 {
		return Decision{}, BadInput
	}
	total := Amount(0)
	for _, line := range lines {
		if line.Amount < 0 {
			return Decision{}, BadInput
		}
		if line.Amount > limit-total {
			return Decision{}, LimitExceeded
		}
		total += line.Amount
	}
	return Decision{Total: total}, nil
}

type quoteInput struct {
	CustomerEmpty bool     `json:"customer_empty"`
	NilLines      bool     `json:"nil_lines"`
	Limit         string   `json:"limit"`
	Amounts       []string `json:"amounts"`
}
type quoteObservation struct {
	Total          string `json:"total"`
	Code           uint8  `json:"code"`
	Length         int    `json:"length"`
	InputUnchanged bool   `json:"input_unchanged"`
	NilPreserved   bool   `json:"nil_preserved"`
}

func main() {
	if len(os.Args) != 2 {
		panic("input JSON file required")
	}
	data, err := os.ReadFile(os.Args[1])
	if err != nil {
		panic(err)
	}
	var inputs []quoteInput
	if err := json.Unmarshal(data, &inputs); err != nil {
		panic(err)
	}
	results := make([]quoteObservation, 0, len(inputs))
	for _, row := range inputs {
		limit, err := strconv.ParseInt(row.Limit, 10, 64)
		if err != nil {
			panic(err)
		}
		var lines []Line
		if !row.NilLines {
			lines = make([]Line, len(row.Amounts))
		} else if len(row.Amounts) != 0 {
			panic("nonempty nil slice")
		}
		for i, amount := range row.Amounts {
			x, err := strconv.ParseInt(amount, 10, 64)
			if err != nil {
				panic(err)
			}
			lines[i] = Line{Amount: Amount(x)}
		}
		before := append([]Line(nil), lines...)
		nilBefore := lines == nil
		customerID := "x"
		if row.CustomerEmpty {
			customerID = ""
		}
		decision, failure := Quote(customerID, lines, Amount(limit))
		var code uint8
		if failure != nil {
			code = uint8(failure.(RejectCode))
		}
		unchanged := len(lines) == len(before)
		for i := range before {
			unchanged = unchanged && lines[i] == before[i]
		}
		results = append(results, quoteObservation{Total: strconv.FormatInt(int64(decision.Total), 10), Code: code,
			Length: len(lines), InputUnchanged: unchanged, NilPreserved: nilBefore == (lines == nil)})
	}
	if err := json.NewEncoder(os.Stdout).Encode(results); err != nil {
		panic(err)
	}
}
