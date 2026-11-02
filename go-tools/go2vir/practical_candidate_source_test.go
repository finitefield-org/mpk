//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"testing"
)

func TestPracticalCandidateSourceRequiresCompleteRequestAndOwner(t *testing.T) {
	request, err := newPracticalCandidateRequest([]practicalCandidateSelector{{Package: "example.com/policy", Function: "Root"}})
	if err != nil {
		t.Fatal(err)
	}
	other, err := newPracticalCandidateRequest([]practicalCandidateSelector{{Package: "example.com/policy", Function: "Unused"}})
	if err != nil {
		t.Fatal(err)
	}
	if roots, err := practicalCandidateRequestRoots(request); err != nil || len(roots) != 1 || roots[0].Function != "Root" {
		t.Fatal("complete original request rejected", err)
	}
	mutated := func(field string) *practicalCandidateRequest {
		copy := *request
		switch field {
		case "registry":
			copy.registry = bytes.Clone(copy.registry)
			copy.registry[0] ^= 1
		case "context":
			copy.context = bytes.Clone(copy.context)
			copy.context[0] ^= 1
		case "selection":
			copy.selection = other.selectionBytes()
		case "request":
			copy.request = bytes.Clone(copy.request)
			copy.request[0] ^= 1
		}
		return &copy
	}
	for _, bad := range []*practicalCandidateRequest{nil, {}, mutated("registry"), mutated("context"), mutated("selection"), mutated("request")} {
		if roots, err := practicalCandidateRequestRoots(bad); err == nil || roots != nil {
			t.Fatal("incomplete, changed or crossed candidate request accepted", err)
		}
	}
	for _, owner := range []*practicalOriginalPackages{nil, {}, {closed: true}, {inputs: &practicalImageInputs{}, loader: &practicalSourceTypes{}}} {
		if selection, err := owner.candidateSourceSelection(request); err == nil || selection.request != nil || selection.roots != nil {
			t.Fatal("candidate request accepted without its original package owner", err)
		}
	}
	var image *practicalReplayImage
	if inputs, err := image.candidateCompilerInputs(nil, request); err == nil || inputs != nil {
		t.Fatal("candidate compiler accepted without its original image", err)
	}
	for _, inputs := range []*practicalCandidateCompilerInputs{nil, {}} {
		if graph, err := inputs.moduleGraphInputs(); err == nil || graph.canonical != nil {
			t.Fatal("candidate graph accepted without its original compiler owner", err)
		}
	}
}
