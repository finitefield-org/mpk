package main

import (
	"fmt"
	"sort"
	"unicode/utf8"
)

func buildSourceMap(module virModule, capture sourceCapture) (sourceMap, []byte, error) {
	inputs := make(map[string][]byte)
	for _, input := range capture.Inputs {
		if input.Kind == sourceInputKind {
			inputs[input.NormalizedPath] = input.Bytes
		}
	}
	entries := make([]sourceMapEntry, 0)
	for _, unit := range module.Units {
		for _, function := range unit.Functions {
			if err := validateSourceOrigin(function.origin, inputs, false); err != nil {
				return sourceMap{}, nil, err
			}
			entries = append(entries, sourceMapEntry{
				Reference: sourceMapReference{Kind: "function", UnitID: unit.ID, FunctionID: function.ID},
				Origin:    function.origin,
			})
			for _, block := range function.Blocks {
				for _, instruction := range block.Instructions {
					if err := validateSourceOrigin(instruction.origin, inputs, false); err != nil {
						return sourceMap{}, nil, err
					}
					entries = append(entries, sourceMapEntry{
						Reference: sourceMapReference{Kind: "instruction", UnitID: unit.ID, FunctionID: function.ID, Block: block.Label, Instruction: instruction.ID},
						Origin:    instruction.origin,
					})
				}
				if err := validateSourceOrigin(block.Terminator.origin, inputs, true); err != nil {
					return sourceMap{}, nil, err
				}
				entries = append(entries, sourceMapEntry{
					Reference: sourceMapReference{Kind: "terminator", UnitID: unit.ID, FunctionID: function.ID, Block: block.Label},
					Origin:    block.Terminator.origin,
				})
			}
		}
	}
	if len(entries) == 0 || len(entries) > maximumMapEntries {
		return sourceMap{}, nil, &artifactBuildError{code: "SOURCE_MAP_LIMIT_ENTRIES", message: "source map entry count is outside the profile"}
	}
	sort.Slice(entries, func(i, j int) bool { return sourceMapEntryLess(entries[i], entries[j]) })
	value := sourceMap{Schema: sourceMapSchema, SourceIRSchema: virSchema, SourceIRHash: module.VIRHash, Entries: entries, SourceMapHash: zeroSHA256()}
	strict, err := strictValueFromTyped(value)
	if err != nil {
		return sourceMap{}, nil, err
	}
	payload, err := withoutRootField(strict, "source_map_hash")
	if err != nil {
		return sourceMap{}, nil, err
	}
	digest, err := hashCanonicalJSON(sourceMapDomain, payload)
	if err != nil {
		return sourceMap{}, nil, err
	}
	value.SourceMapHash = digest
	canonical, err := canonicalJSON(value)
	if err != nil {
		return sourceMap{}, nil, err
	}
	if len(canonical) > maximumMapBytes {
		return sourceMap{}, nil, &artifactBuildError{code: "SOURCE_MAP_LIMIT_CANONICAL_BYTES", message: "source map exceeds the canonical byte limit"}
	}
	return value, canonical, nil
}

func validateSourceOrigin(origin sourceOrigin, inputs map[string][]byte, syntheticAllowed bool) error {
	switch origin.Kind {
	case "source":
		content, exists := inputs[origin.NormalizedPath]
		if !exists || origin.InputKind != sourceInputKind || origin.Start < 0 || origin.End <= origin.Start || origin.End > int64(len(content)) {
			return &artifactBuildError{code: "GO_SOURCE_MAP_ORIGIN", message: "source range is not covered by a captured source"}
		}
		if !utf8.Valid(content) || !utf8Boundary(content, int(origin.Start)) || !utf8Boundary(content, int(origin.End)) {
			return &artifactBuildError{code: "GO_SOURCE_MAP_ORIGIN", message: "source range is not a captured UTF-8 scalar range"}
		}
		if origin.Reason != "" {
			return &artifactBuildError{code: "GO_SOURCE_MAP_ORIGIN", message: "source origin contains a synthetic reason"}
		}
		return nil
	case "synthetic":
		if !syntheticAllowed || origin.InputKind != "" || origin.NormalizedPath != "" || origin.Start != 0 || origin.End != 0 {
			return &artifactBuildError{code: "GO_SOURCE_MAP_ORIGIN", message: "synthetic origin is not permitted here"}
		}
		switch origin.Reason {
		case "go.control_flow_join", "go.loop_backedge", "go.implicit_return":
			return nil
		default:
			return &artifactBuildError{code: "GO_SOURCE_MAP_ORIGIN", message: "synthetic reason is outside the Go profile"}
		}
	default:
		return &artifactBuildError{code: "GO_SOURCE_MAP_ORIGIN", message: "source origin is missing"}
	}
}

func utf8Boundary(content []byte, offset int) bool {
	return offset == 0 || offset == len(content) || offset > 0 && offset < len(content) && content[offset]&0xc0 != 0x80
}

func sourceMapEntryLess(left, right sourceMapEntry) bool {
	l, r := left.Reference, right.Reference
	if l.UnitID != r.UnitID {
		return l.UnitID < r.UnitID
	}
	if l.FunctionID != r.FunctionID {
		return l.FunctionID < r.FunctionID
	}
	rank := func(kind string) int {
		switch kind {
		case "function":
			return 0
		case "instruction":
			return 1
		default:
			return 2
		}
	}
	if rank(l.Kind) != rank(r.Kind) {
		return rank(l.Kind) < rank(r.Kind)
	}
	if blockNumber(l.Block) != blockNumber(r.Block) {
		return blockNumber(l.Block) < blockNumber(r.Block)
	}
	return instructionNumber(l.Instruction) < instructionNumber(r.Instruction)
}

func blockNumber(value string) int {
	var number int
	_, _ = fmt.Sscanf(value, "bb%d", &number)
	return number
}
func instructionNumber(value string) int {
	var number int
	_, _ = fmt.Sscanf(value, "t%d", &number)
	return number
}
