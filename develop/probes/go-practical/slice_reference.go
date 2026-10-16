// Actual Go slice observations; panic recovery stays outside the proof closure.
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"
)

type sliceCell struct {
	Amount int64
	Tag    uint64
}
type sliceInput struct {
	Storage  int    `json:"storage"`
	Offset   int    `json:"offset"`
	Length   int    `json:"length"`
	Capacity int    `json:"capacity"`
	Index    string `json:"index"`
	Nil      bool   `json:"nil"`
	Seed     string `json:"seed"`
}
type sliceOutput struct {
	Nil                    bool   `json:"nil"`
	Length                 int    `json:"length"`
	Capacity               int    `json:"capacity"`
	CopyMetadata           bool   `json:"copy_metadata"`
	FullMetadata           bool   `json:"full_metadata"`
	NilEmptyDistinct       bool   `json:"nil_empty_distinct"`
	SourceDefined          bool   `json:"source_defined"`
	ReadDefined            bool   `json:"read_defined"`
	WriteDefined           bool   `json:"write_defined"`
	Before                 string `json:"before"`
	AliasAfter             string `json:"alias_after"`
	WrongFreshView         string `json:"wrong_fresh_view"`
	FullDefined            bool   `json:"full_defined"`
	FullBefore             string `json:"full_before"`
	FullAfter              string `json:"full_after"`
	OtherRegionFrame       bool   `json:"other_region_frame"`
	SameRegionFrame        bool   `json:"same_region_frame"`
	ArrayCopyPreserved     bool   `json:"array_copy_preserved"`
	ArrayDiffersAfterWrite bool   `json:"array_differs_after_write"`
}

func cellText(value sliceCell) string {
	return fmt.Sprintf("%016x:%016x", uint64(value.Amount), value.Tag)
}
func cellAt(seed uint64, position int) sliceCell {
	return sliceCell{Amount: int64(seed ^ (uint64(position+1) * 0x9e3779b97f4a7c15)), Tag: (seed ^ 0xa5a5a5a5a5a5a5a5) + uint64(position)*0x100000001}
}
func sliceRead(values []sliceCell, index int64) (value string, defined bool) {
	defer func() {
		if recover() != nil {
			value = ""
			defined = false
		}
	}()
	return cellText(values[index]), true
}
func sliceWrite(values []sliceCell, index int64, value sliceCell) (defined bool) {
	defer func() {
		if recover() != nil {
			defined = false
		}
	}()
	values[index] = value
	return true
}
func main() {
	if len(os.Args) != 2 {
		panic("input JSON file required")
	}
	raw, err := os.ReadFile(os.Args[1])
	if err != nil {
		panic(err)
	}
	var cases []sliceInput
	if err = json.Unmarshal(raw, &cases); err != nil {
		panic(err)
	}
	outputs := make([]sliceOutput, 0, len(cases))
	for _, row := range cases {
		switch row.Storage {
		case 0, 1, 8, 32, 127, 128, 129:
		default:
			panic("private storage workload required")
		}
		if row.Offset < 0 || row.Length < 0 || row.Capacity < row.Length || row.Offset+row.Capacity > row.Storage || row.Nil && (row.Offset != 0 || row.Length != 0 || row.Capacity != 0) {
			panic("invalid source slice descriptor")
		}
		index, err := strconv.ParseInt(row.Index, 10, 64)
		if err != nil {
			panic(err)
		}
		seed, err := strconv.ParseUint(row.Seed, 10, 64)
		if err != nil {
			panic(err)
		}
		var backing, other [256]sliceCell
		for i := 0; i < row.Storage; i++ {
			backing[i] = cellAt(seed, i)
			other[i] = cellAt(seed^0x6a09e667f3bcc909, i)
		}
		beforeBacking, beforeOther, arrayCopy := backing, other, backing
		view := backing[row.Offset : row.Offset+row.Length : row.Offset+row.Capacity]
		wrongFresh := other[row.Offset : row.Offset+row.Length : row.Offset+row.Capacity]
		if row.Nil {
			view = nil
			wrongFresh = nil
		}
		copied := view
		full := view[:cap(view)]
		before, readDefined := sliceRead(view, index)
		fullBefore, fullDefined := sliceRead(full, index)
		replacement := sliceCell{Amount: -9223372036854775808, Tag: 0xffffffffffffffff}
		writeDefined := sliceWrite(copied, index, replacement)
		aliasAfter, _ := sliceRead(view, index)
		wrongAfter, _ := sliceRead(wrongFresh, index)
		fullAfter, _ := sliceRead(full, index)
		frame := true
		for i := range backing {
			if writeDefined && i == row.Offset+int(index) {
				continue
			}
			if backing[i] != beforeBacking[i] {
				frame = false
			}
		}
		var nilSlice []sliceCell
		empty := make([]sliceCell, 0, row.Storage)
		outputs = append(outputs, sliceOutput{Nil: view == nil, Length: len(view), Capacity: cap(view),
			CopyMetadata:     (copied == nil) == (view == nil) && len(copied) == len(view) && cap(copied) == cap(view),
			FullMetadata:     (full == nil) == (view == nil) && len(full) == cap(view) && cap(full) == cap(view),
			NilEmptyDistinct: nilSlice == nil && empty != nil && len(nilSlice) == 0 && len(empty) == 0,
			SourceDefined:    view != nil && index >= 0 && index < int64(len(view)), ReadDefined: readDefined, WriteDefined: writeDefined,
			Before: before, AliasAfter: aliasAfter, WrongFreshView: wrongAfter, FullDefined: fullDefined, FullBefore: fullBefore, FullAfter: fullAfter,
			OtherRegionFrame: other == beforeOther, SameRegionFrame: frame, ArrayCopyPreserved: arrayCopy == beforeBacking, ArrayDiffersAfterWrite: arrayCopy != backing})
	}
	if err = json.NewEncoder(os.Stdout).Encode(outputs); err != nil {
		panic(err)
	}
}
