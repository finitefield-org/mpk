// Private original-Go observations; the observer is outside the proof closure.
package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"os"
	"strconv"
)

type stringOpInput struct {
	Bound       int    `json:"bound"`
	Left        string `json:"left"`
	Right       string `json:"right"`
	LeftLength  int    `json:"left_length"`
	RightLength int    `json:"right_length"`
	Lower       string `json:"lower"`
	Upper       string `json:"upper"`
	Index       string `json:"index"`
}
type stringOpOutput struct {
	SubstringGuard      bool   `json:"substring_guard"`
	SubstringDone       bool   `json:"substring_done"`
	SubstringPanicked   bool   `json:"substring_panicked"`
	SubstringBytes      string `json:"substring_bytes"`
	SubstringLength     string `json:"substring_length"`
	SubstringRead       bool   `json:"substring_read"`
	SubstringReadPanic  bool   `json:"substring_read_panicked"`
	SubstringValue      int    `json:"substring_value"`
	SubstringTailSame   bool   `json:"substring_tail_same"`
	RawSubstringLength  string `json:"raw_substring_length"`
	RawSubstringMapped  string `json:"raw_substring_mapped"`
	SubstringSourceRead bool   `json:"substring_source_read"`
	ConcatBytes         string `json:"concat_bytes"`
	ConcatLength        string `json:"concat_length"`
	ConcatBound         bool   `json:"concat_bound"`
	ConcatCarry         bool   `json:"concat_carry"`
	ConcatTailSame      bool   `json:"concat_tail_same"`
	ConcatLeftRead      bool   `json:"concat_left_read"`
	ConcatLeftValue     int    `json:"concat_left_value"`
	ConcatRightRead     bool   `json:"concat_right_read"`
	ConcatRightValue    int    `json:"concat_right_value"`
	ConcatRightMapped   string `json:"concat_right_mapped"`
	LeftSourceRead      bool   `json:"left_source_read"`
	RightSourceRead     bool   `json:"right_source_read"`
	InputBound          bool   `json:"input_bound"`
	InputUnchanged      bool   `json:"input_unchanged"`
	QueryCovered        bool   `json:"query_covered"`
}

func stringOpSlice(s string, lower, upper int64) (value string, done bool) {
	defer func() {
		if recover() != nil {
			done = false
		}
	}()
	return s[lower:upper], true
}
func stringOpRead(s string, i int64) (value int, done bool) {
	value = -1
	defer func() {
		if recover() != nil {
			done = false
		}
	}()
	return int(s[i]), true
}
func stringOpInt(s string) int64 {
	n, err := strconv.ParseInt(s, 10, 64)
	if err != nil {
		panic(err)
	}
	return n
}
func stringOpUint(n uint64) string { return strconv.FormatUint(n, 10) }
func main() {
	if len(os.Args) != 2 {
		panic("input JSON file required")
	}
	data, err := os.ReadFile(os.Args[1])
	if err != nil {
		panic(err)
	}
	var inputs []stringOpInput
	if err := json.Unmarshal(data, &inputs); err != nil {
		panic(err)
	}
	outputs := make([]stringOpOutput, 0, len(inputs))
	for _, row := range inputs {
		left, err := hex.DecodeString(row.Left)
		if err != nil {
			panic(err)
		}
		right, err := hex.DecodeString(row.Right)
		if err != nil {
			panic(err)
		}
		if row.Bound < 0 || row.LeftLength < 0 || row.LeftLength > len(left) || row.RightLength < 0 || row.RightLength > len(right) {
			panic("invalid byte backing")
		}
		l, h, i := stringOpInt(row.Lower), stringOpInt(row.Upper), stringOpInt(row.Index)
		beforeL, beforeR := bytes.Clone(left), bytes.Clone(right)
		a, b := string(left[:row.LeftLength]), string(right[:row.RightLength])
		ml, mr := bytes.Clone(left), bytes.Clone(right)
		for j := row.LeftLength; j < len(ml); j++ {
			ml[j] = 0
		}
		for j := row.RightLength; j < len(mr); j++ {
			mr[j] = 0
		}
		ma, mb := string(ml[:row.LeftLength]), string(mr[:row.RightLength])
		sub, done := stringOpSlice(a, l, h)
		maskedSub, maskedDone := stringOpSlice(ma, l, h)
		out := stringOpOutput{SubstringGuard: 0 <= l && l <= h && h <= int64(len(a)), SubstringDone: done, SubstringPanicked: !done, SubstringValue: -1,
			RawSubstringLength: stringOpUint(uint64(h) - uint64(l)), RawSubstringMapped: stringOpUint(uint64(l) + uint64(i)), InputBound: len(a) <= row.Bound && len(b) <= row.Bound}
		if done {
			out.SubstringBytes, out.SubstringLength = hex.EncodeToString([]byte(sub)), strconv.Itoa(len(sub))
			out.SubstringValue, out.SubstringRead = stringOpRead(sub, i)
			out.SubstringReadPanic = !out.SubstringRead
			out.SubstringTailSame = maskedDone && sub == maskedSub
		}
		_, out.SubstringSourceRead = stringOpRead(a, int64(uint64(l)+uint64(i)))
		combined := a + b
		out.ConcatBytes, out.ConcatLength = hex.EncodeToString([]byte(combined)), strconv.Itoa(len(combined))
		out.ConcatBound, out.ConcatCarry, out.ConcatTailSame = len(combined) <= row.Bound, uint64(len(a))+uint64(len(b)) < uint64(len(a)), combined == ma+mb
		out.ConcatLeftValue, out.ConcatLeftRead = stringOpRead(combined, i)
		mapped := uint64(len(a)) + uint64(i)
		out.ConcatRightMapped = stringOpUint(mapped)
		out.ConcatRightValue, out.ConcatRightRead = stringOpRead(combined, int64(mapped))
		_, out.LeftSourceRead = stringOpRead(a, i)
		_, out.RightSourceRead = stringOpRead(b, i)
		out.InputUnchanged = bytes.Equal(left, beforeL) && bytes.Equal(right, beforeR)
		padding := uint64(1)
		for padding < uint64(row.Bound) {
			padding *= 2
		}
		out.QueryCovered = uint64(i) == uint64(i)&(padding-1)
		outputs = append(outputs, out)
	}
	if err := json.NewEncoder(os.Stdout).Encode(outputs); err != nil {
		panic(err)
	}
}
