package main

import (
	"bytes"
	"crypto/sha256"
	"debug/buildinfo"
	"encoding/hex"
	"encoding/json"
	"os"
	"path/filepath"
	"runtime/debug"
	"testing"
)

// These are native Go observations, not a source-to-binary or context proof.
func TestPracticalAnalysisBuildInfoDifferential(t *testing.T) {
	type observation struct {
		Name     string           `json:"name"`
		Host     string           `json:"host"`
		Info     *debug.BuildInfo `json:"info"`
		Accepted bool             `json:"accepted"`
	}
	var rows []observation
	check := func(name, host string, info *debug.BuildInfo, want bool) {
		t.Helper()
		got := validatePracticalAnalysisBuildInfo(info, host) == nil
		if got != want {
			t.Fatalf("%s: Go validation=%v, want %v", name, got, want)
		}
		rows = append(rows, observation{name, host, info, got})
	}

	file := filepath.Join("..", "..", "develop", "migrations", "go-practical", "t02-w01", "verification", "analysis-build-info", "registered-go2vir.elf")
	raw, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}
	digest := sha256.Sum256(raw)
	if len(raw) != 9004378 || hex.EncodeToString(digest[:]) != "ccc4a984d41b5da7b3a6838cbfaeee3eb23f675a2e4d7fb5d4f9fcd7e597d538" {
		t.Fatal("registered original frontend bytes differ")
	}
	registered, err := buildinfo.Read(bytes.NewReader(raw))
	if err != nil {
		t.Fatal(err)
	}
	check("original-registered-frontend", "linux_amd64", registered, true)
	check("original-valid-facts", "linux_amd64", ptrAnalysisBuildInfo(), true)

	type mutation struct {
		name string
		apply func(*debug.BuildInfo)
	}
	mutations := []mutation{
		{"main-path", func(i *debug.BuildInfo) { i.Main.Path = "caller.invalid/frontend" }},
		{"main-version", func(i *debug.BuildInfo) { i.Main.Version = "caller" }},
		{"main-sum", func(i *debug.BuildInfo) { i.Main.Sum = "caller" }},
		{"main-replacement", func(i *debug.BuildInfo) { i.Main.Replace = &debug.Module{Path: "../caller"} }},
		{"dependency-count", func(i *debug.BuildInfo) { i.Deps = i.Deps[:2] }},
		{"nil-dependency", func(i *debug.BuildInfo) { i.Deps[0] = nil }},
		{"dependency-path", func(i *debug.BuildInfo) { i.Deps[0].Path = "caller.invalid/analysis" }},
		{"dependency-version", func(i *debug.BuildInfo) { i.Deps[0].Version = "v0.28.0" }},
		{"dependency-sum", func(i *debug.BuildInfo) { i.Deps[0].Sum = "caller" }},
		{"dependency-replacement", func(i *debug.BuildInfo) { i.Deps[0].Replace = &debug.Module{Path: "../caller"} }},
		{"duplicate-dependency", func(i *debug.BuildInfo) { i.Deps[1] = i.Deps[0] }},
		{"trimpath-missing", func(i *debug.BuildInfo) { i.Settings = i.Settings[:len(i.Settings)-1] }},
		{"trimpath-false", func(i *debug.BuildInfo) { i.Settings[len(i.Settings)-1].Value = "false" }},
		{"compiler", func(i *debug.BuildInfo) { i.GoVersion = "go1.25.1" }},
		{"frontend-path", func(i *debug.BuildInfo) { i.Path = "caller.invalid/frontend" }},
		{"experiment", func(i *debug.BuildInfo) { i.Settings = append(i.Settings, debug.BuildSetting{Key: "GOEXPERIMENT", Value: "caller"}) }},
		{"duplicate-setting", func(i *debug.BuildInfo) { i.Settings = append(i.Settings, i.Settings[0]) }},
		{"wrong-os", func(i *debug.BuildInfo) { i.Settings[0].Value = "darwin" }},
		{"wrong-amd64", func(i *debug.BuildInfo) { i.Settings[2].Value = "v3" }},
		{"cgo", func(i *debug.BuildInfo) { i.Settings[3].Value = "1" }},
		{"backend", func(i *debug.BuildInfo) { i.Settings[4].Value = "gccgo" }},
		{"buildmode", func(i *debug.BuildInfo) { i.Settings[5].Value = "pie" }},
	}
	for _, m := range mutations {
		info := ptrAnalysisBuildInfo()
		m.apply(info)
		check(m.name, "linux_amd64", info, false)
	}
	for _, flag := range []string{"-asan", "-msan", "-race", "-cover", "-tags", "-gcflags", "-asmflags", "-gccgoflags", "-ldflags", "-pgo", "GOFIPS140"} {
		info := ptrAnalysisBuildInfo()
		info.Settings = append(info.Settings, debug.BuildSetting{Key: flag, Value: "enabled"})
		check("forbidden-flag-"+flag, "linux_amd64", info, false)
	}
	info := ptrAnalysisBuildInfo()
	for index, pin := range practicalAnalysisPins {
		info.Deps[index].Sum = pin.sum
	}
	info.Deps[0], info.Deps[2] = info.Deps[2], info.Deps[0]
	info.Settings = append(info.Settings, debug.BuildSetting{Key: "-race"})
	check("valid-checksums-order-and-empty-flag", "linux_amd64", info, true)
	info = ptrAnalysisBuildInfo()
	info.Settings[0].Value = "darwin"
	info.Settings[1].Value = "arm64"
	check("valid-darwin-host", "darwin_arm64", info, true)
	check("unknown-host", "linux_arm64", ptrAnalysisBuildInfo(), false)

	if out := os.Getenv("MPK_GO_ANALYSIS_BUILD_INFO_OUTPUT"); out != "" {
		if err := os.MkdirAll(out, 0o700); err != nil {
			t.Fatal(err)
		}
		data, err := json.MarshalIndent(rows, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(out, "native.json"), append(data, '\n'), 0o600); err != nil {
			t.Fatal(err)
		}
	}
}

func ptrAnalysisBuildInfo() *debug.BuildInfo {
	info := practicalAnalysisTestBuildInfo()
	return &info
}
