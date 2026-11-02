//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"testing"
	"time"
)

// The external local driver launches the independent read-only processes.
// Keep the original preparation owner and baseline alive until their actual
// complete outputs/caches have been consumed by this Go process.
func comparePracticalOwnedNativeRounds(t *testing.T, output string, image *practicalReplayImage, discovery practicalModuleDiscovery, exports practicalPackageExports) {
	t.Helper()
	baseline, err := image.originalReplayBaseline(discovery, exports)
	if err != nil {
		t.Fatal(err)
	}
	defer baseline.Close()
	if err := os.WriteFile(filepath.Join(output, "baseline-ready"), []byte(image.contentSHA256+"\n"), 0o600); err != nil {
		t.Fatal(err)
	}
	deadline := time.Now().Add(20 * time.Minute)
	if testDeadline, exists := t.Deadline(); exists && testDeadline.Before(deadline) {
		deadline = testDeadline.Add(-5 * time.Second)
	}
	for _, round := range []string{"replay-1", "replay-2"} {
		directory := filepath.Join(output, "handoff", round)
		var status struct {
			ModuleExitCode  int `json:"module_exit_code"`
			PackageExitCode int `json:"package_exit_code"`
		}
		for {
			raw, err := os.ReadFile(filepath.Join(directory, "ready.json"))
			if err == nil {
				if len(raw) > maximumContractBytes || json.Unmarshal(raw, &status) != nil {
					t.Fatal("invalid actual native driver status")
				}
				break
			}
			if !os.IsNotExist(err) || time.Now().After(deadline) {
				t.Fatal("actual independent native execution did not return its complete result", err)
			}
			time.Sleep(50 * time.Millisecond)
		}
		read := func(name string, maximum int) []byte {
			t.Helper()
			info, err := os.Lstat(filepath.Join(directory, name))
			if err != nil || !info.Mode().IsRegular() || info.Size() > int64(maximum) {
				t.Fatal("native handoff observation is missing, special or oversized", name, err)
			}
			raw, err := os.ReadFile(filepath.Join(directory, name))
			if err != nil || len(raw) > maximum {
				t.Fatal("native handoff observation changed or exceeded its bound", name, err)
			}
			return raw
		}
		modules, packages := read("modules.stdout.log", maximumCandidateBytes), read("packages.stdout.log", maximumCandidateBytes)
		moduleDiagnostics, packageDiagnostics := read("modules.stderr.log", maximumContractBytes), read("packages.stderr.log", maximumContractBytes)
		var comparison practicalReplayComparison
		var failure error
		if status.ModuleExitCode != 0 || status.PackageExitCode != 0 {
			failure = fmt.Errorf("actual native commands failed: modules=%d packages=%d", status.ModuleExitCode, status.PackageExitCode)
		} else {
			comparison, failure = baseline.compare(modules, packages, moduleDiagnostics, packageDiagnostics, "/mpk-work/build",
				practicalSourceRoot{Capability: "actual-" + round + "-exports", Directory: filepath.Join(directory, "work", "build")})
		}
		if comparison.actual.contentSHA256 != "" {
			retainPracticalStandardSnapshot(t, comparison.actual)
		}
		record := map[string]any{
			"round": round, "modules": comparison.modules, "packages": comparison.packages, "exports": comparison.exports,
			"export_bytes": comparison.bytes, "image_digest": baseline.imageDigest, "error": fmt.Sprint(failure),
			"module_output": modules, "package_output": packages, "module_diagnostics": moduleDiagnostics, "package_diagnostics": packageDiagnostics,
			"actual_archive_objects": comparison.actual.sourceObjects(), "module_exit_code": status.ModuleExitCode, "package_exit_code": status.PackageExitCode,
			"comparison_is_kernel_execution_attestation": false, "registered_context_or_source_program_proof": false,
		}
		retainPracticalSyntaxEvidence(t, round+".owned-go-comparison.json", record)
		raw, err := json.MarshalIndent(record, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(directory, "go-comparison.json"), append(raw, '\n'), 0o600); err != nil {
			t.Fatal(err)
		}
		if failure != nil {
			t.Fatal(failure)
		}
	}
}
