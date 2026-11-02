// Private original-Go observations of value structs in actual slice views.
package main

import (
	"encoding/hex"
	"encoding/json"
	"os"
	"reflect"
	"strconv"
	"strings"
)

type encodedCell struct {
	Amount string `json:"amount"`
	SKU    string `json:"sku"`
}
type namedLine struct {
	Amount int64
	SKU    string
}
type input struct {
	B           int           `json:"byte_bound"`
	S           int           `json:"slice_bound"`
	Padding     int           `json:"padding"`
	Backing     []encodedCell `json:"backing"`
	Frame       encodedCell   `json:"frame"`
	Replacement string        `json:"replacement"`
	Nil         bool          `json:"nil"`
	Offset      string        `json:"offset"`
	Length      string        `json:"length"`
	Capacity    string        `json:"capacity"`
	Index       string        `json:"index"`
	Pattern     string        `json:"pattern"`
	ByteIndex   string        `json:"byte_index"`
}
type output struct {
	Valid          bool   `json:"descriptor_valid"`
	ProfileInput   bool   `json:"profile_input"`
	Read           bool   `json:"line_read"`
	Panicked       bool   `json:"line_panicked"`
	Absolute       string `json:"absolute"`
	FiniteAbsolute string `json:"finite_absolute"`
	Amount         string `json:"amount"`
	SKU            string `json:"sku"`
	Length         int    `json:"source_length"`
	Prefix         bool   `json:"prefix"`
	Suffix         bool   `json:"suffix"`
	PatternRead    bool   `json:"pattern_read"`
	PatternByte    int    `json:"pattern_byte"`
	PrefixRead     bool   `json:"prefix_read"`
	PrefixByte     int    `json:"prefix_byte"`
	SuffixRead     bool   `json:"suffix_read"`
	SuffixByte     int    `json:"suffix_byte"`
	FramePreserved bool   `json:"frame_sku_preserved"`
	FrameAmount    string `json:"frame_amount"`
	InputUnchanged bool   `json:"input_unchanged"`
}

func number(s string) int64 {
	n, err := strconv.ParseInt(s, 10, 64)
	if err != nil {
		panic(err)
	}
	return n
}
func decode(s string) string {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic(err)
	}
	return string(b)
}
func cell(c encodedCell) namedLine { return namedLine{number(c.Amount), decode(c.SKU)} }
func byteAt(s string, i int64) (defined bool, value int) {
	value = -1
	defer func() {
		if recover() != nil {
			defined = false
			value = -1
		}
	}()
	return true, int(s[i])
}
func lineAt(lines []namedLine, i int64) (value namedLine, panicked bool) {
	defer func() {
		if recover() != nil {
			panicked = true
		}
	}()
	value = lines[i]
	return
}
func observe(r input) output {
	backing := make([]namedLine, len(r.Backing))
	for i, c := range r.Backing {
		backing[i] = cell(c)
	}
	before := append([]namedLine(nil), backing...)
	frame := cell(r.Frame)
	updated := frame
	updated.Amount = number(r.Replacement)
	o, l, k, i, j := number(r.Offset), number(r.Length), number(r.Capacity), number(r.Index), number(r.ByteIndex)
	pattern := decode(r.Pattern)
	valid := l >= 0 && k >= l && o >= 0 && o <= int64(len(backing)) && k <= int64(len(backing))-o
	if r.Nil {
		valid = l == 0 && k == 0
	}
	profile := valid && len(backing) <= r.S && len(pattern) <= r.B && len(frame.SKU) <= r.B
	for _, c := range backing {
		profile = profile && len(c.SKU) <= r.B
	}
	absolute := uint64(o) + uint64(i)
	a := output{Valid: valid, ProfileInput: profile, Absolute: strconv.FormatUint(absolute, 10), FiniteAbsolute: strconv.FormatUint(absolute&uint64(r.Padding-1), 10), Amount: "0", PatternByte: -1, PrefixByte: -1, SuffixByte: -1, FramePreserved: updated.SKU == frame.SKU, FrameAmount: strconv.FormatInt(updated.Amount, 10)}
	if valid {
		var view []namedLine
		if !r.Nil {
			view = backing[o : o+l : o+k]
		}
		v, panicked := lineAt(view, i)
		a.Panicked = panicked
		a.Read = !panicked
		if !panicked {
			a.Amount, a.SKU, a.Length = strconv.FormatInt(v.Amount, 10), hex.EncodeToString([]byte(v.SKU)), len(v.SKU)
			a.Prefix, a.Suffix = strings.HasPrefix(v.SKU, pattern), strings.HasSuffix(v.SKU, pattern)
			a.PatternRead, a.PatternByte = byteAt(pattern, j)
			a.PrefixRead, a.PrefixByte = byteAt(v.SKU, j)
			a.SuffixRead, a.SuffixByte = byteAt(v.SKU, int64(uint64(len(v.SKU))-uint64(len(pattern))+uint64(j)))
		}
	}
	a.InputUnchanged = reflect.DeepEqual(before, backing) || len(backing) == 0
	return a
}
func main() {
	stream := os.Stdin
	if len(os.Args) == 2 {
		f, err := os.Open(os.Args[1])
		if err != nil {
			panic(err)
		}
		defer f.Close()
		stream = f
	}
	var rows []input
	if err := json.NewDecoder(stream).Decode(&rows); err != nil {
		panic(err)
	}
	outputs := make([]output, len(rows))
	for i, r := range rows {
		outputs[i] = observe(r)
	}
	if err := json.NewEncoder(os.Stdout).Encode(outputs); err != nil {
		panic(err)
	}
}
