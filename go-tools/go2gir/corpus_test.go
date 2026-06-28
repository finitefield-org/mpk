package main

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

type goBasicCorpus struct {
	Schema   string                `json:"schema"`
	Positive []goBasicPositiveCase `json:"positive"`
	Negative []goBasicNegativeCase `json:"negative"`
}

type goBasicPositiveCase struct {
	Name string `json:"name"`
	Path string `json:"path"`
}

type goBasicNegativeCase struct {
	Name    string `json:"name"`
	Path    string `json:"path"`
	Feature string `json:"feature"`
	Reason  string `json:"reason"`
}

func TestGoBasicCorpusPositiveFixturesLowerToGIR(t *testing.T) {
	corpus := readGoBasicCorpus(t)
	corpusRoot := goBasicCorpusRoot(t)
	if corpus.Schema != "mpk.go_basic_corpus.v0" {
		t.Fatalf("schema = %q, want mpk.go_basic_corpus.v0", corpus.Schema)
	}
	if len(corpus.Positive) == 0 {
		t.Fatal("positive corpus is empty")
	}

	for _, tt := range corpus.Positive {
		t.Run(tt.Name, func(t *testing.T) {
			result := runGoBasicCorpusPackage(t, corpusRoot, tt.Path)
			if result.GIR == nil {
				t.Fatal("GIR missing")
			}
			if result.GIR.GIRHash == "" {
				t.Fatal("GIR hash missing")
			}
			if result.GIREmission == nil {
				t.Fatal("GIR emission missing")
			}
			if result.SourceManifest == nil {
				t.Fatal("source manifest missing")
			}
			if len(result.Packages) != 1 {
				t.Fatalf("package count = %d, want 1: %+v", len(result.Packages), result.Packages)
			}
			if len(result.SourceManifest.SourceFiles) == 0 {
				t.Fatal("source manifest files missing")
			}
		})
	}
}

func TestGoBasicCorpusNegativeFixturesRejectWithReasons(t *testing.T) {
	corpus := readGoBasicCorpus(t)
	corpusRoot := goBasicCorpusRoot(t)
	if len(corpus.Negative) == 0 {
		t.Fatal("negative corpus is empty")
	}

	for _, tt := range corpus.Negative {
		t.Run(tt.Name, func(t *testing.T) {
			var stdout bytes.Buffer
			var stderr bytes.Buffer

			restoreWorkingDir := chdirForTest(t, corpusRoot)
			exitCode := run([]string{"./" + filepath.ToSlash(tt.Path)}, &stdout, &stderr)
			restoreWorkingDir()
			if exitCode != 1 {
				t.Fatalf("exit code = %d, want 1; stdout=%s stderr=%s", exitCode, stdout.String(), stderr.String())
			}
			if stderr.Len() != 0 {
				t.Fatalf("stderr = %q, want empty", stderr.String())
			}

			var result cliResult
			if err := json.Unmarshal(stdout.Bytes(), &result); err != nil {
				t.Fatalf("decode stdout: %v\n%s", err, stdout.String())
			}
			if result.Status != "rejected" {
				t.Fatalf("status = %q, want rejected", result.Status)
			}
			if result.SSA != nil {
				t.Fatalf("SSA = %+v, want nil for rejected package", result.SSA)
			}
			if result.GIR != nil || result.GIREmission != nil || result.SourceManifest != nil {
				t.Fatalf("GIR output = %+v / %+v / %+v, want nil for rejected package", result.GIR, result.GIREmission, result.SourceManifest)
			}
			if !hasRejectedFeature(result.RejectedFeatures, tt.Feature, tt.Reason) {
				t.Fatalf("rejected features = %+v, want %q / %q", result.RejectedFeatures, tt.Feature, tt.Reason)
			}
		})
	}
}

func readGoBasicCorpus(t *testing.T) goBasicCorpus {
	t.Helper()

	content, err := os.ReadFile(filepath.Join(goBasicCorpusRoot(t), "manifest.json"))
	if err != nil {
		t.Fatalf("read Go basic corpus manifest: %v", err)
	}
	var corpus goBasicCorpus
	if err := json.Unmarshal(content, &corpus); err != nil {
		t.Fatalf("decode Go basic corpus manifest: %v", err)
	}
	return corpus
}

func runGoBasicCorpusPackage(t *testing.T, corpusRoot string, path string) cliResult {
	t.Helper()

	restoreWorkingDir := chdirForTest(t, corpusRoot)
	defer restoreWorkingDir()
	return runSuccessfulPackage(t, "./"+filepath.ToSlash(path))
}

func goBasicCorpusRoot(t *testing.T) string {
	t.Helper()

	root, err := filepath.Abs(filepath.FromSlash("../../fixtures/go-basic"))
	if err != nil {
		t.Fatalf("resolve Go basic corpus root: %v", err)
	}
	return root
}

func chdirForTest(t *testing.T, dir string) func() {
	t.Helper()

	previousDir, err := os.Getwd()
	if err != nil {
		t.Fatalf("get working directory: %v", err)
	}
	if err := os.Chdir(dir); err != nil {
		t.Fatalf("change working directory to %q: %v", dir, err)
	}
	return func() {
		t.Helper()
		if err := os.Chdir(previousDir); err != nil {
			t.Fatalf("restore working directory to %q: %v", previousDir, err)
		}
	}
}
