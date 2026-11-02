//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/json"
	"os"
	"path/filepath"
	"runtime"
	"testing"
)

// Add one additional hard link without changing any original source or the
// actual profile. Go resolves DEFAULT.PGO, even when a different native name
// refers to the very same underlying bytes.
func TestPracticalAnalysisCaseFoldHardLinkSpelling(t *testing.T) {
	frontend := restoreRegisteredAnalysisOriginal(t)
	profile, err := os.ReadFile(filepath.Join(runtime.GOROOT(), "src", "cmd", "compile", "default.pgo"))
	if err != nil {
		t.Fatal(err)
	}
	actual := filepath.Join(frontend, "DEFAULT.PGO")
	alias := filepath.Join(frontend, "AAA.PGO")
	if err := os.WriteFile(actual, profile, 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.Link(actual, alias); err != nil {
		t.Fatal(err)
	}
	lookup, err := os.Stat(filepath.Join(frontend, "default.pgo"))
	if err != nil {
		t.Fatal(err)
	}
	named, err := os.Stat(actual)
	if err != nil || !os.SameFile(lookup, named) {
		t.Fatal("the actual Go lookup does not resolve the native profile entry")
	}
	roots := practicalAnalysisTestRoots(t, frontend)
	session := practicalTestSession(t, roots)
	inputs, err := session.capturePracticalAnalysisForPGO(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, practicalAnalysisPGOAuto)
	if err != nil {
		t.Fatal(err)
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	wire, err := snapshot.snapshotInputs()
	if err != nil {
		t.Fatal(err)
	}
	selected := ""
	if inputs.defaultPGO != nil {
		selected = inputs.defaultPGO.object.Path
	}
	observation := map[string]any{"go_requested": "default.pgo", "actual_native_profile_name": "DEFAULT.PGO", "additional_hard_link_name": "AAA.PGO", "hard_link_same_file": true, "captured_native_profile_name": selected, "profile_sha256": sha256Hex(profile), "profile_bytes": len(profile), "source_objects": len(wire.sources), "build_objects": len(inputs.buildSources()), "source_to_binary_proof": false}
	if out := os.Getenv("MPK_PGO_CASEFOLD_OUTPUT"); out != "" {
		raw, err := json.MarshalIndent(observation, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(out, "observation.json"), append(raw, '\n'), 0o600); err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(out, "original-profile.raw"), profile, 0o600); err != nil {
			t.Fatal(err)
		}
	}
	if selected != "DEFAULT.PGO" {
		t.Fatalf("captured original PGO spelling %q, want the native name resolved by Go", selected)
	}
}
