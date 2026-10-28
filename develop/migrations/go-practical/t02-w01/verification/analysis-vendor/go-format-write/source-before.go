package main

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

// All controls retain the full original vendor manifest unless explicitly
// testing a missing section. The input is not registered-context authority.
func TestPracticalAnalysisVendorDifferential(t *testing.T) {
	raw, err := os.ReadFile("vendor/modules.txt")
	if err != nil {
		t.Fatal(err)
	}
	type observation struct {
		Name      string   `json:"name"`
		Kind      string   `json:"kind"`
		SHA256    string   `json:"sha256"`
		Bytes     int      `json:"bytes"`
		SourceID  string   `json:"source_id"`
		Accepted  bool     `json:"accepted"`
		Packages  []string `json:"packages"`
	}
	var rows []observation
	out := os.Getenv("MPK_GO_ANALYSIS_VENDOR_OUTPUT")
	if out != "" {
		if err := os.MkdirAll(filepath.Join(out, "original-bytes"), 0o700); err != nil {
			t.Fatal(err)
		}
	}
	check := func(name, kind string, data []byte, want bool) map[string]bool {
		t.Helper()
		source, err := newPracticalSourceBuffer("analysis-frontend", "vendor/modules.txt", kind, data)
		if err != nil {
			t.Fatalf("%s source: %v", name, err)
		}
		packages, err := parsePracticalAnalysisVendorManifest(source)
		if (err == nil) != want {
			t.Fatalf("%s: accepted=%v, want %v: %v", name, err == nil, want, err)
		}
		var names []string
		for name := range packages {
			names = append(names, name)
		}
		sort.Strings(names)
		digest := sha256.Sum256(data)
		hash := hex.EncodeToString(digest[:])
		if out != "" {
			file := filepath.Join(out, "original-bytes", hash+".raw")
			old, readErr := os.ReadFile(file)
			if os.IsNotExist(readErr) {
				if err := os.WriteFile(file, data, 0o600); err != nil {
					t.Fatal(err)
				}
			} else if readErr != nil || !bytes.Equal(old, data) {
				t.Fatal("source digest aliases differing original bytes")
			}
		}
		rows = append(rows, observation{name, kind, hash, len(data), source.object.ID, err == nil, names})
		return packages
	}
	base := check("original-complete-vendor-manifest", "metadata", raw, true)
	replace := func(old, new string) []byte {
		t.Helper()
		if !bytes.Contains(raw, []byte(old)) {
			t.Fatalf("original control anchor %q missing", old)
		}
		return bytes.Replace(raw, []byte(old), []byte(new), 1)
	}
	for _, mutation := range []struct {
		name string
		data []byte
	}{
		{"wrong-version", replace("v0.36.0", "v0.36.1")},
		{"missing-module", raw[bytes.Index(raw, []byte("# golang.org/x/sync")):]},
		{"replacement", replace("# golang.org/x/mod v0.27.0", "# golang.org/x/mod v0.27.0 => ../caller")},
		{"unknown-language", replace("go 1.23.0", "go 1.25.0")},
		{"missing-annotation", replace("## explicit; go 1.23.0\n", "")},
		{"empty-section", replace("golang.org/x/sync/errgroup\n", "")},
		{"duplicate-package", append(bytes.Clone(raw), []byte("golang.org/x/tools/go/ssa\n")...)},
		{"crossed-package", append(bytes.Clone(raw), []byte("golang.org/x/mod/module\n")...)},
		{"duplicate-module", append(bytes.Clone(raw), []byte("# golang.org/x/tools v0.36.0\n## explicit; go 1.23.0\ngolang.org/x/tools/go/ssa\n")...)},
		{"unknown-module", append(bytes.Clone(raw), []byte("# golang.org/x/caller v1.0.0\n## explicit; go 1.23.0\ngolang.org/x/caller\n")...)},
		{"duplicate-annotation", replace("## explicit; go 1.23.0\n", "## explicit; go 1.23.0\n## explicit; go 1.23.0\n")},
		{"annotation-before-header", append([]byte("## explicit; go 1.23.0\n"), raw...)},
	} {
		check(mutation.name, "metadata", mutation.data, false)
	}
	check("wrong-source-kind", "go_file", raw, false)
	check("header-fields-unicode-whitespace", "metadata", replace("# golang.org/x/mod v0.27.0", "#\tgolang.org/x/mod\u2003v0.27.0"), true)
	check("all-crlf", "metadata", bytes.ReplaceAll(raw, []byte("\n"), []byte("\r\n")), true)
	check("unicode-trim-space", "metadata", append([]byte("\u00a0\u3000\n"), raw...), true)
	for _, name := range []string{".private", "a..b", "a+b", "a~x", "trailing-"} {
		check("valid-path-"+name, "metadata", append(bytes.Clone(raw), []byte("golang.org/x/tools/"+name+"\n")...), true)
	}
	for _, name := range []string{"CON", "nul.txt", "dir~1.ext", "abc.", ".", "..", "a//b", "a/b/", "a b", "a%b", "あ"} {
		check("invalid-path-"+name, "metadata", append(bytes.Clone(raw), []byte("golang.org/x/tools/"+name+"\n")...), false)
	}
	invalidUTF8 := append(bytes.Clone(raw), []byte("golang.org/x/tools/")...)
	invalidUTF8 = append(invalidUTF8, 0xff, '\n')
	check("invalid-utf8", "metadata", invalidUTF8, false)
	var expanded strings.Builder
	expanded.Write(raw)
	for index := 0; index < maximumManifestInputs-len(base); index++ {
		fmt.Fprintf(&expanded, "golang.org/x/tools/testing/f%05d\n", index)
	}
	atLimit := []byte(expanded.String())
	packages := check("complete-original-plus-maximum-packages", "metadata", atLimit, true)
	if len(packages) != maximumManifestInputs {
		t.Fatalf("maximum package count: %d", len(packages))
	}
	check("complete-original-plus-one-over-maximum", "metadata", append(bytes.Clone(atLimit), []byte("golang.org/x/tools/testing/over_limit\n")...), false)
	if out != "" {
		data, err := json.MarshalIndent(rows, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(out, "native.json"), append(data, '\n'), 0o600); err != nil {
			t.Fatal(err)
		}
	}
}
