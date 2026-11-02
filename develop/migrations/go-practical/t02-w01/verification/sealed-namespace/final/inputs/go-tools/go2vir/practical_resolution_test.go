package main

import (
	"bytes"
	"context"
	"strings"
	"testing"
)

func TestPracticalResolutionDecodesPinnedGoStreamAndRetainsMetadata(t *testing.T) {
	raw := []byte(`{"Path":"example.com/app","Main":true,"Dir":"/original/app","GoMod":"/original/app/go.mod","GoVersion":"1.25.0"}
{"Path":"example.com/dep","Version":"v1.0.0","GoVersion":"1.22","Origin":{"VCS":"git","Hash":"opaque"},"Indirect":true,"GoModSum":"original h1"}`)
	rows, err := decodePracticalGoModules(raw)
	if err != nil || len(rows) != 2 || !rows[0].Main || rows[1].Version != "v1.0.0" || !bytes.Contains(rows[1].Origin, []byte("opaque")) {
		t.Fatalf("original Go stream or opaque provenance differs: %v", err)
	}
}

func TestPracticalResolutionRejectsMalformedAmbiguousAndFailedGoOutput(t *testing.T) {
	for name, raw := range map[string]string{
		"empty": "", "array": "[]", "empty-path": "{}", "trailing": `{"Path":"a"} trailing`,
		"duplicate-field": `{"Path":"a","Path":"b"}`, "duplicate-module": `{"Path":"a"}{"Path":"a"}`,
		"unknown-Go-field": `{"Path":"a","CallerAuthority":true}`, "Go-error": `{"Path":"a","Error":{"Err":"offline input missing"}}`,
		"replacement-error":   `{"Path":"a","Replace":{"Path":"b","Error":{"Err":"missing"}}}`,
		"chained-replacement": `{"Path":"a","Replace":{"Path":"b","Replace":{"Path":"c"}}}`,
	} {
		t.Run(name, func(t *testing.T) {
			if _, err := decodePracticalGoModules([]byte(raw)); err == nil {
				t.Fatal("invalid Go module output accepted")
			}
		})
	}
	if _, err := decodePracticalGoModules(bytes.Repeat([]byte(" "), maximumCandidateBytes+1)); err == nil {
		t.Fatal("unbounded module output accepted")
	}
}

func TestPracticalResolutionOutputBoundCancelsAndRetainsOnlyBoundedBytes(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	output := practicalBoundedOutput{maximum: 2, cancel: cancel}
	if n, err := output.Write([]byte("ab")); err != nil || n != 2 {
		t.Fatal("inclusive Go output bound rejected")
	}
	if n, err := output.Write([]byte("c")); err == nil || n != 0 || output.String() != "ab" || ctx.Err() == nil {
		t.Fatal("Go output overflow did not cancel before retaining excess bytes")
	}
}

func TestPracticalResolutionRequiresExactCompilerBeforeInputReads(t *testing.T) {
	session := &practicalCaptureSession{}
	selection := validatedLauncherSelection{Target: targetIdentity{ID: goTarget, PointerWidth: goPointerWidth}}
	if _, err := session.discoverPracticalModules(practicalModuleLocation{}, practicalModuleLocation{}, selection); err == nil || !strings.Contains(err.Error(), "exact validated") || len(session.files) != 0 {
		t.Fatal("missing pinned compiler identity allowed source reads")
	}
}
