// Raw descriptor metadata and separate actual-Go construction/index observations.
package main

import (
	"encoding/json"
	"fmt"
	"math/bits"
	"os"
	"strconv"
)

type accessCell struct {
	Amount int64
	Tag    uint64
}
type accessInput struct {
	Profile  int     `json:"profile"`
	Storage  string  `json:"storage"`
	Offset   string  `json:"offset"`
	Length   string  `json:"length"`
	Capacity string  `json:"capacity"`
	Index    string  `json:"index"`
	Nil      bool    `json:"nil"`
	Region   bool    `json:"region"`
	Live     [2]bool `json:"live"`
}
type accessOutput struct {
	Valid                bool   `json:"valid"`
	SelectedLive         bool   `json:"selected_live"`
	Guard                bool   `json:"guard"`
	Extent               string `json:"extent"`
	ExtentCarry          bool   `json:"extent_carry"`
	Absolute             string `json:"absolute"`
	AbsoluteCarry        bool   `json:"absolute_carry"`
	PhysicalBound        bool   `json:"physical_bound"`
	PhysicalNonnegative  bool   `json:"physical_nonnegative"`
	StorageBound         bool   `json:"storage_bound"`
	Reconstructed        string `json:"reconstructed"`
	Covered              bool   `json:"covered"`
	CopyValid            bool   `json:"copy_valid"`
	FullValid            bool   `json:"full_valid"`
	NilGuard             bool   `json:"nil_guard"`
	Constructed          bool   `json:"constructed"`
	ConstructionPanicked bool   `json:"construction_panicked"`
	ReadDefined          bool   `json:"read_defined"`
	ReadPanicked         bool   `json:"read_panicked"`
	ActualValue          string `json:"actual_value"`
	ModeledValue         string `json:"modeled_value"`
}

func accessNumber(s string) int64 {
	n, err := strconv.ParseInt(s, 10, 64)
	if err != nil {
		panic(err)
	}
	return n
}
func accessValue(seed uint64, i int) accessCell {
	return accessCell{Amount: int64(seed ^ (uint64(i+1) * 0x9e3779b97f4a7c15)), Tag: (seed ^ 0xa5a5a5a5a5a5a5a5) + uint64(i)*0x100000001}
}
func accessText(v accessCell) string { return fmt.Sprintf("%016x:%016x", uint64(v.Amount), v.Tag) }
func accessValid(nilTag, live bool, o, l, k, n int64) bool {
	if nilTag {
		return l == 0 && k == 0
	}
	extent, carry := bits.Add64(uint64(o), uint64(k), 0)
	return live && o >= 0 && l >= 0 && k >= 0 && n >= 0 && uint64(l) <= uint64(k) && carry == 0 && extent <= uint64(n)
}
func accessConstruct(backing []accessCell, nilTag bool, o, l, k int64) (view []accessCell, constructed, panicked bool) {
	defer func() {
		if recover() != nil {
			view = nil
			constructed = false
			panicked = true
		}
	}()
	if nilTag {
		if l != 0 || k != 0 {
			return nil, false, false
		}
		return nil, true, false
	}
	return backing[o : o+l : o+k], true, false
}
func accessRead(view []accessCell, index int64) (value string, defined, panicked bool) {
	defer func() {
		if recover() != nil {
			value = ""
			defined = false
			panicked = true
		}
	}()
	return accessText(view[index]), true, false
}
func main() {
	if len(os.Args) != 2 {
		panic("input JSON file required")
	}
	raw, err := os.ReadFile(os.Args[1])
	if err != nil {
		panic(err)
	}
	var cases []accessInput
	if err = json.Unmarshal(raw, &cases); err != nil {
		panic(err)
	}
	outputs := make([]accessOutput, 0, len(cases))
	for _, row := range cases {
		switch row.Profile {
		case 0, 1, 8, 32, 127, 128, 129:
		default:
			panic("private profile workload required")
		}
		n, o, l, k, i := accessNumber(row.Storage), accessNumber(row.Offset), accessNumber(row.Length), accessNumber(row.Capacity), accessNumber(row.Index)
		if n < 0 || n > 257 {
			panic("empirical storage workload outside 0..257")
		}
		r := 0
		if row.Region {
			r = 1
		}
		selected := row.Live[r]
		extent, extentCarry := bits.Add64(uint64(o), uint64(k), 0)
		absolute, absoluteCarry := bits.Add64(uint64(o), uint64(i), 0)
		pad := 1
		for pad < row.Profile {
			pad *= 2
		}
		reconstructed := absolute & uint64(pad-1)
		seed := uint64(0)
		if row.Region {
			seed = 0x6a09e667f3bcc909
		}
		backing := make([]accessCell, int(n))
		for p := range backing {
			backing[p] = accessValue(seed, p)
		}
		view, constructed, constructionPanic := accessConstruct(backing, row.Nil, o, l, k)
		value, defined, readPanic := "", false, false
		if constructed {
			value, defined, readPanic = accessRead(view, i)
		}
		outputs = append(outputs, accessOutput{Valid: accessValid(row.Nil, selected, o, l, k, n), SelectedLive: selected,
			Guard: !row.Nil && i >= 0 && i < l, Extent: strconv.FormatUint(extent, 10), ExtentCarry: extentCarry != 0,
			Absolute: strconv.FormatUint(absolute, 10), AbsoluteCarry: absoluteCarry != 0, PhysicalBound: absolute < uint64(n), PhysicalNonnegative: int64(absolute) >= 0,
			StorageBound: uint64(n) <= uint64(row.Profile), Reconstructed: strconv.FormatUint(reconstructed, 10), Covered: absolute == reconstructed,
			CopyValid: accessValid(row.Nil, selected, o, l, k, n), FullValid: accessValid(row.Nil, selected, o, k, k, n), NilGuard: false,
			Constructed: constructed, ConstructionPanicked: constructionPanic, ReadDefined: defined, ReadPanicked: readPanic,
			ActualValue: value, ModeledValue: accessText(accessValue(seed, int(reconstructed)))})
	}
	if err = json.NewEncoder(os.Stdout).Encode(outputs); err != nil {
		panic(err)
	}
}
