//go:build go1.25 && (linux || darwin)

package main

import (
    "encoding/json"
    "os"
    "os/exec"
    "path/filepath"
    "runtime"
    "strings"
    "testing"
)

func TestPracticalAnalysisCaseFoldActualProfile(t *testing.T) {
    frontend := restoreRegisteredAnalysisOriginal(t)
    data, err := os.ReadFile(filepath.Join(runtime.GOROOT(), "src", "cmd", "compile", "default.pgo"))
    if err != nil { t.Fatal(err) }
    if err := os.WriteFile(filepath.Join(frontend, "DEFAULT.PGO"), data, 0o644); err != nil { t.Fatal(err) }
    if _, err := os.Stat(filepath.Join(frontend, "default.pgo")); err != nil { t.Fatalf("case-insensitive alias was not resolved: %v", err) }
    roots := practicalAnalysisTestRoots(t, frontend)
    session := practicalTestSession(t, roots)
    inputs, err := session.capturePracticalAnalysisForPGO(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, practicalAnalysisPGOAuto)
    if err != nil { t.Fatal(err) }
    snapshot, err := session.seal(); if err != nil { t.Fatal(err) }
    wire, err := snapshot.snapshotInputs(); if err != nil { t.Fatal(err) }
    args := []string{"build", "-n", "-a", "-mod=vendor", "-trimpath", "-buildvcs=false", "-ldflags=-buildid=", "-o", filepath.Join(t.TempDir(), "go2vir"), "."}
    command := exec.Command(filepath.Join(runtime.GOROOT(), "bin", "go"), args...)
    command.Dir = frontend
    command.Env = []string{"PATH=" + filepath.Join(runtime.GOROOT(), "bin") + ":/usr/bin:/bin", "HOME=" + t.TempDir(), "GOROOT=" + runtime.GOROOT(), "GOCACHE=" + os.Getenv("GOCACHE"), "GOENV=off", "GOFLAGS=", "GOWORK=off", "GOPROXY=off", "GOSUMDB=off", "GOTOOLCHAIN=local", "CGO_ENABLED=0", "GOOS=linux", "GOARCH=amd64", "GOAMD64=v1"}
    plan, err := command.CombinedOutput(); if err != nil { t.Fatalf("actual Go plan: %v\n%s", err, plan) }
    uses := strings.Contains(string(plan), "default.pgo")
    observation := map[string]any{"actual_root_entry": "DEFAULT.PGO", "go_stat_lowercase_resolves": true, "go_plan_uses_profile": uses, "profile_captured": inputs.defaultPGO != nil, "source_objects": len(wire.sources), "build_objects": len(inputs.buildSources()), "profile_bytes": len(data), "profile_sha256": sha256Hex(data), "source_to_binary_proof": false}
    out := os.Getenv("MPK_PGO_CASEFOLD_OUTPUT")
    if out != "" {
        raw, err := json.MarshalIndent(observation, "", "  "); if err != nil { t.Fatal(err) }
        if err := os.WriteFile(filepath.Join(out, "observation.json"), append(raw, '\n'), 0o600); err != nil { t.Fatal(err) }
        if err := os.WriteFile(filepath.Join(out, "plan.log"), plan, 0o600); err != nil { t.Fatal(err) }
        if err := os.WriteFile(filepath.Join(out, "original-profile.raw"), data, 0o600); err != nil { t.Fatal(err) }
    }
    if !uses { t.Fatal("Go did not use the actual resolved default profile") }
    if inputs.defaultPGO == nil { t.Fatal("automatic PGO omitted a profile resolved by the actual filesystem") }
}
