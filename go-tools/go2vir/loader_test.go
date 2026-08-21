package main

import (
	"bytes"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
)

func TestLoaderUsesSnapshotPinnedToolchainAndHostileEnvironmentCannotChangeManifest(t *testing.T) {
	request, candidate := buildTestLauncherSelection(t)
	if err := validateLowerRequest(request); err != nil {
		t.Fatalf("fresh test candidate request is invalid: %v", err)
	}
	selection, err := validateLauncherSelection(request, candidate)
	if err != nil {
		t.Fatalf("validate fresh unregistered test candidate: %v", err)
	}

	originalRoot := copyPreflightFixture(t)
	capture, err := captureSourceTree(originalRoot, request)
	if err != nil {
		t.Fatalf("capture loader fixture: %v", err)
	}
	snapshot, err := buildSourceSnapshot(t.TempDir(), capture)
	if err != nil {
		t.Fatalf("build loader snapshot: %v", err)
	}
	defer snapshot.Close()
	writeTestFile(t, filepath.Join(originalRoot, "identity.go"), []byte("package corrupted\n"), 0o600)

	setHostileGoEnvironment(t, "first")
	first, err := loadCapturedPackages(capture, snapshot, selection)
	if err != nil {
		if failure, ok := err.(*frontendFailure); ok && failure.cause != nil {
			t.Fatalf("load captured packages under first hostile environment: %v (private cause: %v)", err, failure.cause)
		}
		t.Fatalf("load captured packages under first hostile environment: %v", err)
	}
	assertLoadedFixture(t, first)
	firstManifest, firstBytes, err := buildSourceManifest(request, capture, first, selection,
		"374dbbcc0c9454bf29c0117c02f1bbdc0424df970297af9fe4560512d40d0690",
		"f66b38fcdba7dd4b6250269c566d5599c2b1821a69370f67aa961fdb5893b6f9",
	)
	if err != nil {
		t.Fatalf("build first manifest: %v", err)
	}

	setHostileGoEnvironment(t, "second")
	second, err := loadCapturedPackages(capture, snapshot, selection)
	if err != nil {
		t.Fatalf("load captured packages under second hostile environment: %v", err)
	}
	assertLoadedFixture(t, second)
	secondManifest, secondBytes, err := buildSourceManifest(request, capture, second, selection,
		firstManifest.VIRHash, firstManifest.SourceMapHash,
	)
	if err != nil {
		t.Fatalf("build second manifest: %v", err)
	}
	if !reflect.DeepEqual(firstManifest, secondManifest) || !bytes.Equal(firstBytes, secondBytes) {
		t.Fatal("hostile ambient environment changed canonical manifest bytes")
	}
	if bytes.Contains(firstBytes, []byte(originalRoot)) || bytes.Contains(firstBytes, []byte(snapshot.Root)) || bytes.Contains(firstBytes, []byte(selection.GoRoot)) {
		t.Fatal("manifest leaked an original, snapshot, or toolchain path")
	}

	mutatedRequest := request
	mutatedRequest.ToolchainDistributionSHA256 = strings.Repeat("0", 64)
	if _, err := validateLauncherSelection(mutatedRequest, candidate); err == nil {
		t.Fatal("launcher/toolchain identity mismatch was accepted")
	}
}

func TestLoaderEnvironmentIsClosedAndSorted(t *testing.T) {
	selection := validatedLauncherSelection{GoRoot: "/private/toolchain/go", GoExecutable: "/private/toolchain/go/bin/go"}
	filesystem := loaderFilesystem{
		goBuild:   "/private/cache/go-build",
		goMod:     "/private/cache/go-mod",
		goPath:    "/private/gopath",
		home:      "/private/empty/home",
		temporary: "/private/tmp",
	}
	environment := pinnedLoaderEnvironment(selection, filesystem)
	if !sortStringsAreStrict(environment) {
		t.Fatalf("loader environment is not sorted: %v", environment)
	}
	if len(environment) != 30 {
		t.Fatalf("loader environment has %d entries, want 30", len(environment))
	}
	want := map[string]string{
		"CGO_ENABLED": "0", "GOOS": "linux", "GOARCH": "amd64", "GOAMD64": "v1",
		"GOPROXY": "off", "GOSUMDB": "off", "GOTOOLCHAIN": "local", "GOWORK": "off",
		"GOROOT": selection.GoRoot, "PATH": filepath.Dir(selection.GoExecutable),
	}
	for key, value := range want {
		if !containsEnvironmentEntry(environment, key+"="+value) {
			t.Fatalf("loader environment lacks %s=%s", key, value)
		}
	}
	for _, forbidden := range []string{"USER=", "LOGNAME=", "SHELL=", "HTTP_PROXY=", "GITHUB_TOKEN="} {
		for _, entry := range environment {
			if strings.HasPrefix(entry, forbidden) {
				t.Fatalf("loader environment inherited forbidden entry %q", entry)
			}
		}
	}
}

func assertLoadedFixture(t *testing.T, loaded packageLoadResult) {
	t.Helper()
	if len(loaded.Packages) != 2 {
		t.Fatalf("loaded package count = %d, want 2", len(loaded.Packages))
	}
	want := map[string][]string{
		"example.com/mpk/vector":        {"identity.go"},
		"example.com/mpk/vector/helper": {"helper/helper.go"},
	}
	for _, packageValue := range loaded.Packages {
		if expected, exists := want[packageValue.PackagePath]; !exists || !reflect.DeepEqual(packageValue.CompiledGoFiles, expected) || !reflect.DeepEqual(packageValue.GoFiles, expected) {
			t.Fatalf("loaded package inventory = %#v", packageValue)
		}
	}
}

func setHostileGoEnvironment(t *testing.T, suffix string) {
	t.Helper()
	values := map[string]string{
		"GOENV": "/hostile/goenv-" + suffix, "GOPROXY": "https://proxy.invalid/" + suffix,
		"GONOSUMDB": "*", "GOPRIVATE": "*", "GOWORK": "/hostile/go.work",
		"HOME": "/hostile/home-" + suffix, "LANG": "hostile_" + suffix, "LC_ALL": "hostile_" + suffix,
		"TZ": "Hostile/" + suffix, "GITHUB_TOKEN": "sentinel-" + suffix,
	}
	for key, value := range values {
		t.Setenv(key, value)
	}
}

func sortStringsAreStrict(values []string) bool {
	for index := 1; index < len(values); index++ {
		if values[index-1] >= values[index] {
			return false
		}
	}
	return true
}

func containsEnvironmentEntry(values []string, wanted string) bool {
	for _, value := range values {
		if value == wanted {
			return true
		}
	}
	return false
}

func TestLoaderRejectsAmbientModuleDependencyBeforeGoRuns(t *testing.T) {
	root := copyPreflightFixture(t)
	writeTestFile(t, filepath.Join(root, "identity.go"), []byte("package vector\n\nimport \"example.net/external\"\n"), 0o600)
	_, err := captureSourceTree(root, preflightRequest())
	assertFrontendFailure(t, err, "rejected", "GO_SUBSET_IMPORT")
	if _, statErr := os.Stat(filepath.Join(root, "go.sum")); statErr != nil {
		t.Fatalf("ambient dependency rejection changed source inputs: %v", statErr)
	}
}
