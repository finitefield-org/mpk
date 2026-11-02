package main

import (
	"bytes"
	"fmt"
	"os"
	"testing"
)

func TestPracticalCandidateRequestMatchesFrozenGoRustContext(t *testing.T) {
	roots := []practicalCandidateSelector{{Package: "example.test/quote", Function: "Quote"}}
	request, err := newPracticalCandidateRequest(roots)
	if err != nil {
		t.Fatal(err)
	}
	want, err := os.ReadFile("../../crates/mpk-vc/tests/fixtures/go-practical-candidate-request.json")
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(append(request.requestBytes(), '\n'), want) {
		t.Fatal("Go producer differs from the frozen independent candidate request")
	}
	registry, err := decodeStrictJSON(request.registryBytes())
	if err != nil || registry.(map[string]jsonValue)["registry_sha256"] != practicalCandidateRegistryHash {
		t.Fatal("candidate registry is not the complete pinned Go/Rust registry", err)
	}
	selection, err := decodeStrictJSON(request.selectionBytes())
	if err != nil {
		t.Fatal(err)
	}
	selectionHash, err := hashCanonicalJSON("MPK-GO-PRACTICAL-SELECTION-1.0", selection)
	if err != nil || selectionHash != "198d3169e9678149f2fa72f3c415c041e3f9d6d379eab0a0d53e12d392064059" {
		t.Fatal("selection hash differs from the independent frozen preimage", err)
	}
	before := request.requestBytes()
	roots[0].Package = "caller-mutated"
	copyOut := request.requestBytes()
	copyOut[0] ^= 1
	if !bytes.Equal(request.requestBytes(), before) {
		t.Fatal("caller mutation changed an owned candidate request")
	}
}

func TestPracticalCandidateSelectionRetainsExactRawRules(t *testing.T) {
	root := practicalCandidateSelector{Package: "example.test/quote", Function: "Quote"}
	valid := [][]practicalCandidateSelector{
		{root},
		{{Package: "example.test//quote", Function: "見積"}},
		{{Package: root.Package, Function: "init", Receiver: &practicalCandidateReceiver{Package: root.Package, TypeName: "LedgerAlias"}}},
	}
	for _, roots := range valid {
		if _, err := newPracticalCandidateRequest(roots); err != nil {
			t.Fatalf("valid frozen raw selector rejected: %v", err)
		}
	}
	invalid := []struct {
		name  string
		roots []practicalCandidateSelector
	}{
		{"empty", nil},
		{"empty package", []practicalCandidateSelector{{Function: "Quote"}}},
		{"host path", []practicalCandidateSelector{{Package: "/tmp/quote", Function: "Quote"}}},
		{"backslash", []practicalCandidateSelector{{Package: "quote\\sub", Function: "Quote"}}},
		{"nul", []practicalCandidateSelector{{Package: "quote\x00sub", Function: "Quote"}}},
		{"invalid utf8", []practicalCandidateSelector{{Package: string([]byte{0xff}), Function: "Quote"}}},
		{"blank", []practicalCandidateSelector{{Package: root.Package, Function: "_"}}},
		{"keyword", []practicalCandidateSelector{{Package: root.Package, Function: "return"}}},
		{"package init", []practicalCandidateSelector{{Package: root.Package, Function: "init"}}},
		{"pointer receiver", []practicalCandidateSelector{{Package: root.Package, Function: "Quote", Receiver: &practicalCandidateReceiver{Package: root.Package, TypeName: "*Ledger"}}}},
		{"foreign receiver", []practicalCandidateSelector{{Package: root.Package, Function: "Quote", Receiver: &practicalCandidateReceiver{Package: "other", TypeName: "Ledger"}}}},
		{"duplicate", []practicalCandidateSelector{root, root}},
		{"reverse", []practicalCandidateSelector{{Package: root.Package, Function: "B"}, {Package: root.Package, Function: "A"}}},
	}
	for _, test := range invalid {
		t.Run(test.name, func(t *testing.T) {
			if _, err := newPracticalCandidateRequest(test.roots); err == nil {
				t.Fatal("invalid frozen raw selector was accepted")
			}
		})
	}
}

func TestPracticalCandidateSelectionExactRootLimitAndReceiverOwnership(t *testing.T) {
	roots := make([]practicalCandidateSelector, 8192)
	for i := range roots {
		roots[i] = practicalCandidateSelector{Package: "example.test/quote", Function: fmt.Sprintf("F%05d", i)}
	}
	if _, err := newPracticalCandidateRequest(roots); err != nil {
		t.Fatalf("the exact 8192-root limit was rejected: %v", err)
	}
	if _, err := newPracticalCandidateRequest(append(roots, practicalCandidateSelector{Package: "example.test/quote", Function: "Z"})); err == nil {
		t.Fatal("8193 roots were accepted")
	}
	receiver := &practicalCandidateReceiver{Package: "example.test/quote", TypeName: "Ledger"}
	request, err := newPracticalCandidateRequest([]practicalCandidateSelector{{Package: receiver.Package, Function: "Quote", Receiver: receiver}})
	if err != nil {
		t.Fatal(err)
	}
	before := request.requestBytes()
	receiver.TypeName = "Mutated"
	if !bytes.Equal(request.requestBytes(), before) {
		t.Fatal("caller mutation changed the owned receiver")
	}
}
