// Native copy over fresh local arrays. Metadata modeling and source outcomes differ.
package main

import (
	"encoding/json"
	"fmt"
	"math/bits"
	"os"
	"strconv"
)

type copyCell struct {
	Amount int64
	Tag    uint64
}
type copyView struct {
	Offset   string `json:"offset"`
	Length   string `json:"length"`
	Capacity string `json:"capacity"`
	Nil      bool   `json:"nil"`
	Region   bool   `json:"region"`
}
type copyInput struct {
	Profile     int      `json:"profile"`
	Storage     string   `json:"storage"`
	Destination copyView `json:"destination"`
	Source      copyView `json:"source"`
	Index       string   `json:"index"`
	Live        [2]bool  `json:"live"`
}
type copyOutput struct {
	DestinationValid                bool        `json:"destination_valid"`
	SourceValid                     bool        `json:"source_valid"`
	Count                           string      `json:"count"`
	CountNonnegative                bool        `json:"count_nonnegative"`
	CountBounds                     bool        `json:"count_bounds"`
	CopiedGuard                     bool        `json:"copied_guard"`
	DestinationGuard                bool        `json:"destination_guard"`
	SourceGuard                     bool        `json:"source_guard"`
	Physical                        string      `json:"physical"`
	PhysicalCarry                   bool        `json:"physical_carry"`
	Relative                        string      `json:"relative"`
	Covered                         bool        `json:"covered"`
	StorageBound                    bool        `json:"storage_bound"`
	ModeledValue                    string      `json:"modeled_value"`
	DestinationConstructed          bool        `json:"destination_constructed"`
	SourceConstructed               bool        `json:"source_constructed"`
	DestinationConstructionPanicked bool        `json:"destination_construction_panicked"`
	SourceConstructionPanicked      bool        `json:"source_construction_panicked"`
	Copied                          bool        `json:"copied"`
	ActualCount                     string      `json:"actual_count"`
	MetadataPreserved               bool        `json:"metadata_preserved"`
	DestinationRead                 bool        `json:"destination_read"`
	SourceRead                      bool        `json:"source_read"`
	DestinationReadPanicked         bool        `json:"destination_read_panicked"`
	SourceReadPanicked              bool        `json:"source_read_panicked"`
	DestinationBefore               string      `json:"destination_before"`
	DestinationAfter                string      `json:"destination_after"`
	SourceBefore                    string      `json:"source_before"`
	SourceAfter                     string      `json:"source_after"`
	NaiveDestination                string      `json:"naive_destination"`
	Before                          [2][]string `json:"before"`
	After                           [2][]string `json:"after"`
}

func copyNumber(s string) int64 {
	n, e := strconv.ParseInt(s, 10, 64)
	if e != nil {
		panic(e)
	}
	return n
}
func copyText(v copyCell) string { return fmt.Sprintf("%016x:%016x", uint64(v.Amount), v.Tag) }
func copyValue(region bool, i uint64) copyCell {
	var seed uint64
	if region {
		seed = 0x6a09e667f3bcc909
	}
	return copyCell{int64(seed ^ ((i + 1) * 0x9e3779b97f4a7c15)), (seed ^ 0xa5a5a5a5a5a5a5a5) + i*0x100000001}
}
func copyRegion(v copyView) int {
	if v.Region {
		return 1
	}
	return 0
}
func copyValid(v copyView, n int64, live [2]bool) bool {
	o, l, k := copyNumber(v.Offset), copyNumber(v.Length), copyNumber(v.Capacity)
	if v.Nil {
		return l == 0 && k == 0
	}
	end, c := bits.Add64(uint64(o), uint64(k), 0)
	return live[copyRegion(v)] && o >= 0 && l >= 0 && k >= 0 && n >= 0 && uint64(l) <= uint64(k) && c == 0 && end <= uint64(n)
}
func copyConstruct(backing [2][]copyCell, v copyView) (s []copyCell, ok, panicked bool) {
	defer func() {
		if recover() != nil {
			s = nil
			ok = false
			panicked = true
		}
	}()
	o, l, k := copyNumber(v.Offset), copyNumber(v.Length), copyNumber(v.Capacity)
	if v.Nil {
		if l != 0 || k != 0 {
			return nil, false, false
		}
		return nil, true, false
	}
	return backing[copyRegion(v)][o : o+l : o+k], true, false
}
func copyRead(v []copyCell, i int64) (s string, ok, panicked bool) {
	defer func() {
		if recover() != nil {
			s = ""
			ok = false
			panicked = true
		}
	}()
	return copyText(v[i]), true, false
}
func copyBacking(n int) [2][]copyCell {
	var h [2][]copyCell
	for r := range h {
		h[r] = make([]copyCell, n)
		for i := range h[r] {
			h[r][i] = copyValue(r != 0, uint64(i))
		}
	}
	return h
}
func copyRows(h [2][]copyCell) [2][]string {
	var a [2][]string
	for r := range h {
		a[r] = make([]string, len(h[r]))
		for i, v := range h[r] {
			a[r][i] = copyText(v)
		}
	}
	return a
}
func main() {
	if len(os.Args) != 2 {
		panic("input path required")
	}
	raw, e := os.ReadFile(os.Args[1])
	if e != nil {
		panic(e)
	}
	var rows []copyInput
	if e = json.Unmarshal(raw, &rows); e != nil {
		panic(e)
	}
	out := make([]copyOutput, 0, len(rows))
	for _, r := range rows {
		switch r.Profile {
		case 0, 1, 8, 32, 127, 128, 129:
		default:
			panic("private capacity required")
		}
		n := copyNumber(r.Storage)
		if n < 0 || n > 257 {
			panic("private storage workload 0..257")
		}
		i := copyNumber(r.Index)
		d, s := r.Destination, r.Source
		dl, sl := uint64(copyNumber(d.Length)), uint64(copyNumber(s.Length))
		m := dl
		if sl < m {
			m = sl
		}
		o := copyNumber(d.Offset)
		physical, c := bits.Add64(uint64(o), uint64(i), 0)
		relative, _ := bits.Sub64(physical, uint64(o), 0)
		p := uint64(1)
		for p < uint64(r.Profile) {
			p *= 2
		}
		key := physical
		var value copyCell
		if relative < m {
			key, _ = bits.Add64(uint64(copyNumber(s.Offset)), relative, 0)
			value = copyValue(s.Region, key&(p-1))
		} else {
			value = copyValue(d.Region, key&(p-1))
		}
		a := copyOutput{DestinationValid: copyValid(d, n, r.Live), SourceValid: copyValid(s, n, r.Live), Count: strconv.FormatUint(m, 10), CountNonnegative: int64(m) >= 0, CountBounds: m <= dl && m <= sl,
			CopiedGuard: i >= 0 && uint64(i) < m, DestinationGuard: !d.Nil && i >= 0 && i < copyNumber(d.Length), SourceGuard: !s.Nil && i >= 0 && i < copyNumber(s.Length), Physical: strconv.FormatUint(physical, 10), PhysicalCarry: c != 0, Relative: strconv.FormatUint(relative, 10), Covered: physical == physical&(p-1), StorageBound: uint64(n) <= uint64(r.Profile), ModeledValue: copyText(value)}
		h := copyBacking(int(n))
		naive := copyBacking(int(n))
		a.Before = copyRows(h)
		dst, dc, dp := copyConstruct(h, d)
		src, sc, sp := copyConstruct(h, s)
		a.DestinationConstructed = dc
		a.SourceConstructed = sc
		a.DestinationConstructionPanicked = dp
		a.SourceConstructionPanicked = sp
		if dc {
			a.DestinationBefore, a.DestinationRead, a.DestinationReadPanicked = copyRead(dst, i)
		}
		if sc {
			a.SourceBefore, a.SourceRead, a.SourceReadPanicked = copyRead(src, i)
		}
		if dc && sc {
			dn, sn := len(dst), len(src)
			dk, sk := cap(dst), cap(src)
			dnNil, snNil := dst == nil, src == nil
			a.ActualCount = strconv.Itoa(copy(dst, src))
			a.Copied = true
			a.MetadataPreserved = dn == len(dst) && sn == len(src) && dk == cap(dst) && sk == cap(src) && dnNil == (dst == nil) && snNil == (src == nil)
			nd, _, _ := copyConstruct(naive, d)
			ns, _, _ := copyConstruct(naive, s)
			limit := len(nd)
			if len(ns) < limit {
				limit = len(ns)
			}
			for j := 0; j < limit; j++ {
				nd[j] = ns[j]
			}
			a.NaiveDestination, _, _ = copyRead(nd, i)
		}
		if dc {
			a.DestinationAfter, _, _ = copyRead(dst, i)
		}
		if sc {
			a.SourceAfter, _, _ = copyRead(src, i)
		}
		a.After = copyRows(h)
		out = append(out, a)
	}
	if e = json.NewEncoder(os.Stdout).Encode(out); e != nil {
		panic(e)
	}
}
