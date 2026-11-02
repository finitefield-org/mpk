// Actual two/three-index operations; raw model metadata stays separate.
package main

import (
	"encoding/json"
	"fmt"
	"math/bits"
	"os"
	"strconv"
)

type rangeCell struct {
	Amount int64
	Tag    uint64
}
type rangeInput struct {
	Kind     string  `json:"kind"`
	Profile  int     `json:"profile"`
	Storage  string  `json:"storage"`
	Offset   string  `json:"offset"`
	Length   string  `json:"length"`
	Capacity string  `json:"capacity"`
	Low      string  `json:"low"`
	High     string  `json:"high"`
	Maximum  string  `json:"maximum"`
	Index    string  `json:"index"`
	Nil      bool    `json:"nil"`
	Region   bool    `json:"region"`
	Live     [2]bool `json:"live"`
}
type rangeOutput struct {
	ValidOld             bool   `json:"valid_old"`
	Guard                bool   `json:"guard"`
	NewOffset            string `json:"new_offset"`
	OffsetCarry          bool   `json:"offset_carry"`
	NewLength            string `json:"new_length"`
	LengthBorrow         bool   `json:"length_borrow"`
	NewCapacity          string `json:"new_capacity"`
	CapacityBorrow       bool   `json:"capacity_borrow"`
	ValidNew             bool   `json:"valid_new"`
	StorageBound         bool   `json:"storage_bound"`
	Constructed          bool   `json:"constructed"`
	ConstructionPanicked bool   `json:"construction_panicked"`
	Resliced             bool   `json:"resliced"`
	ReslicePanicked      bool   `json:"reslice_panicked"`
	ActualNil            bool   `json:"actual_nil"`
	ActualLength         string `json:"actual_length"`
	ActualCapacity       string `json:"actual_capacity"`
	NewGuard             bool   `json:"new_guard"`
	Mapped               string `json:"mapped"`
	MappedCarry          bool   `json:"mapped_carry"`
	Physical             string `json:"physical"`
	PhysicalCarry        bool   `json:"physical_carry"`
	Covered              bool   `json:"covered"`
	Reconstructed        string `json:"reconstructed"`
	ModeledValue         string `json:"modeled_value"`
	ReadDefined          bool   `json:"read_defined"`
	ReadPanicked         bool   `json:"read_panicked"`
	ActualValue          string `json:"actual_value"`
	FullDefined          bool   `json:"full_defined"`
	FullPanicked         bool   `json:"full_panicked"`
	FullValue            string `json:"full_value"`
	OriginalDefined      bool   `json:"original_defined"`
	OriginalPanicked     bool   `json:"original_panicked"`
	OriginalValue        string `json:"original_value"`
	Replacement          string `json:"replacement"`
	AliasAfter           string `json:"alias_after"`
	FullAfter            string `json:"full_after"`
}

func rangeNumber(s string) int64 {
	n, e := strconv.ParseInt(s, 10, 64)
	if e != nil {
		panic(e)
	}
	return n
}
func rangeValue(seed uint64, i uint64) rangeCell {
	return rangeCell{int64(seed ^ ((i + 1) * 0x9e3779b97f4a7c15)), (seed ^ 0xa5a5a5a5a5a5a5a5) + i*0x100000001}
}
func rangeText(v rangeCell) string { return fmt.Sprintf("%016x:%016x", uint64(v.Amount), v.Tag) }
func rangeValid(nilTag, live bool, o, l, k, n int64) bool {
	if nilTag {
		return l == 0 && k == 0
	}
	end, c := bits.Add64(uint64(o), uint64(k), 0)
	return live && o >= 0 && l >= 0 && k >= 0 && n >= 0 && uint64(l) <= uint64(k) && c == 0 && end <= uint64(n)
}
func rangeConstruct(backing []rangeCell, nilTag bool, o, l, k int64) (v []rangeCell, ok, panicked bool) {
	defer func() {
		if recover() != nil {
			v = nil
			ok = false
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
func rangeView(parent []rangeCell, three bool, lo, hi, m int64) (v []rangeCell, ok, panicked bool) {
	defer func() {
		if recover() != nil {
			v = nil
			ok = false
			panicked = true
		}
	}()
	if three {
		return parent[lo:hi:m], true, false
	}
	return parent[lo:hi], true, false
}
func rangeRead(view []rangeCell, i int64) (value string, ok, panicked bool) {
	defer func() {
		if recover() != nil {
			value = ""
			ok = false
			panicked = true
		}
	}()
	return rangeText(view[i]), true, false
}
func main() {
	if len(os.Args) != 2 {
		panic("input JSON path required")
	}
	raw, err := os.ReadFile(os.Args[1])
	if err != nil {
		panic(err)
	}
	var inputs []rangeInput
	if err = json.Unmarshal(raw, &inputs); err != nil {
		panic(err)
	}
	out := make([]rangeOutput, 0, len(inputs))
	for _, r := range inputs {
		switch r.Profile {
		case 0, 1, 8, 32, 127, 128, 129:
		default:
			panic("private probe capacity required")
		}
		if r.Kind != "two" && r.Kind != "three" {
			panic("two/three source operation required")
		}
		n, o, l, k := rangeNumber(r.Storage), rangeNumber(r.Offset), rangeNumber(r.Length), rangeNumber(r.Capacity)
		lo, hi, m, i := rangeNumber(r.Low), rangeNumber(r.High), rangeNumber(r.Maximum), rangeNumber(r.Index)
		if n < 0 || n > 257 {
			panic("empirical storage workload outside 0..257")
		}
		if r.Kind == "two" {
			m = k
		}
		region := 0
		if r.Region {
			region = 1
		}
		live := r.Live[region]
		no, oc := bits.Add64(uint64(o), uint64(lo), 0)
		nl, lb := bits.Sub64(uint64(hi), uint64(lo), 0)
		nk, kb := bits.Sub64(uint64(m), uint64(lo), 0)
		mapped, mc := bits.Add64(uint64(lo), uint64(i), 0)
		physical, pc := bits.Add64(uint64(o), mapped, 0)
		pad := 1
		for pad < r.Profile {
			pad *= 2
		}
		reconstructed := physical & uint64(pad-1)
		seed := uint64(0)
		if r.Region {
			seed = 0x6a09e667f3bcc909
		}
		backing := make([]rangeCell, int(n))
		for p := range backing {
			backing[p] = rangeValue(seed, uint64(p))
		}
		parent, constructed, constructPanic := rangeConstruct(backing, r.Nil, o, l, k)
		var view []rangeCell
		resliced, reslicePanic := false, false
		if constructed {
			view, resliced, reslicePanic = rangeView(parent, r.Kind == "three", lo, hi, m)
		}
		actualLen, actualCap := "", ""
		actualNil := false
		value, defined, readPanic := "", false, false
		fullValue, fullDefined, fullPanic := "", false, false
		originalValue, originalDefined, originalPanic := "", false, false
		replacement, aliasAfter, fullAfter := "", "", ""
		if resliced {
			actualLen, actualCap = strconv.Itoa(len(view)), strconv.Itoa(cap(view))
			actualNil = view == nil
			value, defined, readPanic = rangeRead(view, i)
		}
		if constructed {
			full := parent[:cap(parent)]
			fullValue, fullDefined, fullPanic = rangeRead(full, int64(mapped))
			originalValue, originalDefined, originalPanic = rangeRead(parent, int64(mapped))
			if defined && fullDefined {
				next := rangeValue(seed^0x510e527fade682d1, physical)
				replacement = rangeText(next)
				full[int64(mapped)] = next
				aliasAfter, _, _ = rangeRead(view, i)
				fullAfter, _, _ = rangeRead(full, int64(mapped))
			}
		}
		guard := lo >= 0 && lo <= hi && hi <= m && m <= k
		newGuard := !r.Nil && i >= 0 && i < int64(nl)
		out = append(out, rangeOutput{ValidOld: rangeValid(r.Nil, live, o, l, k, n), Guard: guard,
			NewOffset: strconv.FormatUint(no, 10), OffsetCarry: oc != 0, NewLength: strconv.FormatUint(nl, 10), LengthBorrow: lb != 0, NewCapacity: strconv.FormatUint(nk, 10), CapacityBorrow: kb != 0,
			ValidNew: rangeValid(r.Nil, live, int64(no), int64(nl), int64(nk), n), StorageBound: uint64(n) <= uint64(r.Profile),
			Constructed: constructed, ConstructionPanicked: constructPanic, Resliced: resliced, ReslicePanicked: reslicePanic, ActualNil: actualNil, ActualLength: actualLen, ActualCapacity: actualCap, NewGuard: newGuard,
			Mapped: strconv.FormatUint(mapped, 10), MappedCarry: mc != 0, Physical: strconv.FormatUint(physical, 10), PhysicalCarry: pc != 0, Covered: physical == reconstructed, Reconstructed: strconv.FormatUint(reconstructed, 10), ModeledValue: rangeText(rangeValue(seed, reconstructed)),
			ReadDefined: defined, ReadPanicked: readPanic, ActualValue: value, FullDefined: fullDefined, FullPanicked: fullPanic, FullValue: fullValue, OriginalDefined: originalDefined, OriginalPanicked: originalPanic, OriginalValue: originalValue,
			Replacement: replacement, AliasAfter: aliasAfter, FullAfter: fullAfter})
	}
	if err = json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
