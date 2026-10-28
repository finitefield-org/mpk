//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
)

// This additional test keeps the entire restored original source epoch and
// the actual registered SDK profile; the filesystem decides if the spelling
// aliases Go's default.pgo lookup.
func TestPracticalAnalysisCaseSensitiveAndFoldedPGO(t *testing.T) {
	frontend := restoreRegisteredAnalysisOriginal(t)
	profile, err := os.ReadFile(filepath.Join(runtime.GOROOT(), "src", "cmd", "compile", "default.pgo"))
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(frontend, "DEFAULT.PGO"), profile, 0o644); err != nil {
		t.Fatal(err)
	}
	_, err = os.Stat(filepath.Join(frontend, "default.pgo"))
	aliases := err == nil
	if !aliases && !os.IsNotExist(err) {
		t.Fatal(err)
	}
	if aliases != (os.Getenv("MPK_CASE_FOLD_EXPECTED") == "1") {
		t.Fatalf("unexpected original filesystem name resolution: %v", err)
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
	if aliases {
		if inputs.defaultPGO == nil || inputs.defaultPGO.object.Path != "DEFAULT.PGO" || !bytes.Equal(inputs.defaultPGO.sourceBytes(), profile) || len(wire.sources) != 197 || len(inputs.buildSources()) != 185 {
			t.Fatal("case-folded profile lookup lost original bytes or original spelling")
		}
	} else if inputs.defaultPGO != nil || len(wire.sources) != 196 || len(inputs.buildSources()) != 184 {
		t.Fatal("case-sensitive Go ignores the differently spelled file")
	}
	args := []string{"build", "-n", "-a", "-mod=vendor", "-trimpath", "-buildvcs=false", "-ldflags=-buildid=", "-o", filepath.Join(t.TempDir(), "go2vir"), "."}
	command := exec.Command(filepath.Join(runtime.GOROOT(), "bin", "go"), args...)
	command.Dir = frontend
	command.Env = []string{"PATH=" + filepath.Join(runtime.GOROOT(), "bin") + ":/usr/bin:/bin", "HOME=" + t.TempDir(), "GOROOT=" + runtime.GOROOT(), "GOCACHE=" + os.Getenv("GOCACHE"), "GOENV=off", "GOFLAGS=", "GOWORK=off", "GOPROXY=off", "GOSUMDB=off", "GOTOOLCHAIN=local", "CGO_ENABLED=0", "GOOS=linux", "GOARCH=amd64", "GOAMD64=v1"}
	plan, err := command.CombinedOutput()
	if err != nil {
		t.Fatalf("actual Go plan: %v\n%s", err, plan)
	}
	if strings.Contains(string(plan), "default.pgo") != aliases {
		t.Fatal("actual Go plan disagrees with original filesystem lookup")
	}
	out := os.Getenv("MPK_PGO_CASEFOLD_OUTPUT")
	if out == "" {
		return
	}
	if err := os.MkdirAll(filepath.Join(out, "original-bytes"), 0o700); err != nil {
		t.Fatal(err)
	}
	var rows []struct {
		Object  practicalSourceObject `json:"object"`
		RawFile string                `json:"raw_file"`
	}
	for _, source := range wire.sources {
		name := source.object.RawSHA256 + ".raw"
		file := filepath.Join(out, "original-bytes", name)
		old, err := os.ReadFile(file)
		if os.IsNotExist(err) {
			if err := os.WriteFile(file, source.sourceBytes(), 0o600); err != nil {
				t.Fatal(err)
			}
		} else if err != nil || !bytes.Equal(old, source.sourceBytes()) {
			t.Fatal("original bytes differ")
		}
		rows = append(rows, struct {
			Object  practicalSourceObject `json:"object"`
			RawFile string                `json:"raw_file"`
		}{source.object, name})
	}
	var archives []string
	for _, item := range inputs.modules {
		archives = append(archives, item.archive.source.object.ID)
	}
	var objects []practicalSourceObject
	for _, item := range inputs.buildSources() {
		objects = append(objects, item.sourceObject())
	}
	packet := map[string]any{"snapshot_hex": hex.EncodeToString(wire.canonical), "source_ids": snapshot.ids, "original_sources": rows, "manifest_source_id": inputs.vendorManifest.object.ID, "archive_source_ids": archives, "build_objects": objects, "actual_root_entry": "DEFAULT.PGO", "go_lookup_resolves": aliases, "go_plan_uses_profile": aliases, "profile_sha256": sha256Hex(profile), "profile_bytes": len(profile), "original_sources_and_modes_preserved": true, "launcher_authority_claimed": false}
	raw, err := json.MarshalIndent(packet, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(out, "native.json"), append(raw, '\n'), 0o600); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(out, "plan.log"), plan, 0o600); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(out, "original-profile.raw"), profile, 0o600); err != nil {
		t.Fatal(err)
	}
}
