package main

import (
	"bytes"
	"os"
	"runtime/debug"
	"strings"
	"testing"
)

func practicalAnalysisTestSource(t *testing.T, name, kind string, raw []byte) practicalSourceBuffer {
	t.Helper()
	source, err := newPracticalSourceBuffer("analysis-test", name, kind, raw)
	if err != nil {
		t.Fatal(err)
	}
	return source
}

func TestPracticalAnalysisPinsOriginalFrontendManifestAndChecksums(t *testing.T) {
	manifestRaw, err := os.ReadFile("go.mod")
	if err != nil {
		t.Fatal(err)
	}
	sumsRaw, err := os.ReadFile("go.sum")
	if err != nil {
		t.Fatal(err)
	}
	manifest, err := parsePracticalModuleManifest(practicalAnalysisTestSource(t, "go.mod", "go_mod", manifestRaw), false)
	if err != nil {
		t.Fatal(err)
	}
	sums, err := parsePracticalModuleSums(practicalAnalysisTestSource(t, "go.sum", "go_sum", sumsRaw))
	if err != nil {
		t.Fatal(err)
	}
	if err := validatePracticalAnalysisManifest(manifest, sums); err != nil {
		t.Fatal(err)
	}
	for _, mutation := range []string{"version", "checksum", "replacement", "indirect"} {
		t.Run(mutation, func(t *testing.T) {
			modRaw, sumRaw := bytes.Clone(manifestRaw), bytes.Clone(sumsRaw)
			switch mutation {
			case "version":
				modRaw = bytes.Replace(modRaw, []byte("v0.36.0"), []byte("v0.36.1"), 1)
			case "checksum":
				sumRaw = bytes.Replace(sumRaw, []byte(practicalAnalysisPins[0].sum), []byte("h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="), 1)
			case "replacement":
				modRaw = append(modRaw, []byte("\nreplace golang.org/x/mod => ../caller\n")...)
			case "indirect":
				modRaw = bytes.Replace(modRaw, []byte(" // indirect"), nil, 1)
			}
			manifest, err := parsePracticalModuleManifest(practicalAnalysisTestSource(t, "go.mod", "go_mod", modRaw), false)
			if err != nil {
				t.Fatal(err)
			}
			sums, err := parsePracticalModuleSums(practicalAnalysisTestSource(t, "go.sum", "go_sum", sumRaw))
			if err != nil {
				t.Fatal(err)
			}
			if err := validatePracticalAnalysisManifest(manifest, sums); err == nil {
				t.Fatal("self-consistent but unregistered analysis build inputs accepted")
			}
		})
	}
}

func TestPracticalAnalysisVendorManifestRequiresCompleteExactOwners(t *testing.T) {
	raw, err := os.ReadFile("vendor/modules.txt")
	if err != nil {
		t.Fatal(err)
	}
	if packages, err := parsePracticalAnalysisVendorManifest(practicalAnalysisTestSource(t, "vendor/modules.txt", "metadata", raw)); err != nil || !packages["golang.org/x/tools/go/packages"] || !packages["golang.org/x/tools/go/ssa"] {
		t.Fatalf("original complete analysis packages rejected: %v", err)
	}
	for mutation, changed := range map[string][]byte{
		"wrong-version":      bytes.Replace(raw, []byte("v0.36.0"), []byte("v0.36.1"), 1),
		"missing-module":     raw[bytes.Index(raw, []byte("# golang.org/x/sync")):],
		"replacement":        bytes.Replace(raw, []byte("# golang.org/x/mod v0.27.0"), []byte("# golang.org/x/mod v0.27.0 => ../caller"), 1),
		"unknown-language":   bytes.Replace(raw, []byte("go 1.23.0"), []byte("go 1.25.0"), 1),
		"missing-annotation": bytes.Replace(raw, []byte("## explicit; go 1.23.0\n"), nil, 1),
		"empty-section":      bytes.Replace(raw, []byte("golang.org/x/sync/errgroup\n"), nil, 1),
		"duplicate-package":  append(bytes.Clone(raw), []byte("golang.org/x/tools/go/ssa\n")...),
		"crossed-package":    append(bytes.Clone(raw), []byte("golang.org/x/mod/module\n")...),
		"duplicate-module":   append(bytes.Clone(raw), []byte("# golang.org/x/tools v0.36.0\n## explicit; go 1.23.0\ngolang.org/x/tools/go/ssa\n")...),
		"unknown-module":     append(bytes.Clone(raw), []byte("# golang.org/x/caller v1.0.0\n## explicit; go 1.23.0\ngolang.org/x/caller\n")...),
	} {
		t.Run(mutation, func(t *testing.T) {
			if _, err := parsePracticalAnalysisVendorManifest(practicalAnalysisTestSource(t, "vendor/modules.txt", "metadata", changed)); err == nil {
				t.Fatal("incomplete or crossed analysis package authority accepted")
			}
		})
	}
}

func TestPracticalAnalysisGoSourceRequiresClosedEmbeddedInputs(t *testing.T) {
	ordinary := practicalAnalysisTestSource(t, "ordinary.go", "go_file", []byte("package p\nconst Example = `//go:embed payload`\n"))
	if err := validatePracticalAnalysisGoSource(ordinary); err != nil {
		t.Fatal(err)
	}
	// Directives follow imports and must be seen after an imports-only parse
	// would have stopped. String literals above remain ordinary source bytes.
	embedded := practicalAnalysisTestSource(t, "embedded.go", "go_file", []byte("package p\nimport _ \"embed\"\n//go:embed payload\nvar Data []byte\n"))
	if err := validatePracticalAnalysisGoSource(embedded); err == nil {
		t.Fatal("uncaptured embedded compiler bytes accepted")
	}
}

func practicalAnalysisTestBuildInfo() debug.BuildInfo {
	info := debug.BuildInfo{GoVersion: practicalCompilerRelease, Path: practicalFrontendModule, Main: debug.Module{Path: practicalFrontendModule, Version: "(devel)"}, Settings: []debug.BuildSetting{
		{Key: "GOOS", Value: "linux"}, {Key: "GOARCH", Value: "amd64"}, {Key: "GOAMD64", Value: "v1"}, {Key: "CGO_ENABLED", Value: "0"}, {Key: "-compiler", Value: "gc"}, {Key: "-buildmode", Value: "exe"}, {Key: "-trimpath", Value: "true"},
	}}
	for _, pin := range practicalAnalysisPins {
		info.Deps = append(info.Deps, &debug.Module{Path: pin.identity.Path, Version: pin.identity.Version})
	}
	return info
}

func TestPracticalAnalysisBuildInfoRequiresExactLinkedLibrariesAndFlags(t *testing.T) {
	valid := practicalAnalysisTestBuildInfo()
	if err := validatePracticalAnalysisBuildInfo(&valid, "linux_amd64"); err != nil {
		t.Fatal(err)
	}
	for _, mutation := range []string{"main-path", "main-version", "main-sum", "main-replacement", "dependency-count", "nil-dependency", "dependency-path", "dependency-version", "dependency-sum", "dependency-replacement", "duplicate-dependency", "trimpath", "race", "compiler", "experiment", "duplicate-setting"} {
		t.Run(mutation, func(t *testing.T) {
			info := practicalAnalysisTestBuildInfo()
			switch mutation {
			case "main-path":
				info.Main.Path = "caller.invalid/frontend"
			case "main-version":
				info.Main.Version = "caller"
			case "main-sum":
				info.Main.Sum = "caller"
			case "main-replacement":
				info.Main.Replace = &debug.Module{Path: "../caller"}
			case "dependency-count":
				info.Deps = info.Deps[:2]
			case "nil-dependency":
				info.Deps[0] = nil
			case "dependency-path":
				info.Deps[0].Path = "caller.invalid/analysis"
			case "dependency-version":
				info.Deps[0].Version = "v0.28.0"
			case "dependency-sum":
				info.Deps[0].Sum = strings.Repeat("0", 64)
			case "dependency-replacement":
				info.Deps[0].Replace = &debug.Module{Path: "../caller"}
			case "duplicate-dependency":
				info.Deps[1] = info.Deps[0]
			case "trimpath":
				info.Settings = info.Settings[:len(info.Settings)-1]
			case "race":
				info.Settings = append(info.Settings, debug.BuildSetting{Key: "-race", Value: "true"})
			case "compiler":
				info.GoVersion = "go1.25.1"
			case "experiment":
				info.Settings = append(info.Settings, debug.BuildSetting{Key: "GOEXPERIMENT", Value: "caller"})
			case "duplicate-setting":
				info.Settings = append(info.Settings, info.Settings[0])
			}
			if err := validatePracticalAnalysisBuildInfo(&info, "linux_amd64"); err == nil {
				t.Fatal("wrong analysis module/build metadata accepted")
			}
		})
	}
}
