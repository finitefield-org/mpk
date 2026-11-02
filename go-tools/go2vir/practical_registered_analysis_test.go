//go:build go1.25 && (linux || darwin)

package main

import (
	"archive/tar"
	"bytes"
	"encoding/hex"
	"encoding/json"
	"io"
	"os"
	"path"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

type registeredAnalysisOriginalFile struct {
	Path   string `json:"path"`
	Bytes  uint64 `json:"bytes"`
	SHA256 string `json:"sha256"`
	Mode   uint32 `json:"mode"`
}

// Restore the complete registered source epoch from the unmodified retained
// archive. The receipt records the post-extraction file modes; the archive's
// own directory and file modes remain preserved in the exact archive bytes.
func restoreRegisteredAnalysisOriginal(t *testing.T) string {
	t.Helper()
	evidence := filepath.Join("..", "..", "develop", "migrations", "go-practical", "t02-w01", "verification", "installed-compiler")
	receiptRaw, err := os.ReadFile(filepath.Join(evidence, "original-go-source.json"))
	if err != nil {
		t.Fatal(err)
	}
	var receipt struct {
		BasisCommit   string                           `json:"basis_commit"`
		ArchiveSHA256 string                           `json:"archive_sha256"`
		SourceFiles   []registeredAnalysisOriginalFile `json:"source_files"`
	}
	if err := json.Unmarshal(receiptRaw, &receipt); err != nil {
		t.Fatal(err)
	}
	if receipt.BasisCommit != "ed67a4c388defe8babdbc55f29851552562121d9" || len(receipt.SourceFiles) != 269 {
		t.Fatal("original complete source receipt differs")
	}
	archive, err := os.ReadFile(filepath.Join(evidence, "original-go-source.tar"))
	if err != nil {
		t.Fatal(err)
	}
	if sha256Hex(archive) != receipt.ArchiveSHA256 {
		t.Fatal("original archive byte identity differs")
	}
	const oldRoot = "/private/tmp/mpk-installed-compiler-205-go-source/"
	expected := make(map[string]registeredAnalysisOriginalFile, len(receipt.SourceFiles))
	frontendCount := 0
	for _, file := range receipt.SourceFiles {
		if !strings.HasPrefix(file.Path, oldRoot) {
			t.Fatal("original receipt escapes its source epoch")
		}
		name := strings.TrimPrefix(file.Path, oldRoot)
		if name == "" || path.Clean(name) != name || strings.HasPrefix(name, "../") || strings.HasPrefix(name, "/") {
			t.Fatal("invalid original source path")
		}
		if _, exists := expected[name]; exists {
			t.Fatal("duplicate original source path")
		}
		expected[name] = file
		if strings.HasPrefix(name, "go-tools/go2vir/") {
			frontendCount++
		}
	}
	if frontendCount != 242 {
		t.Fatal("complete original frontend source count differs")
	}
	root := t.TempDir()
	archiveReader := tar.NewReader(bytes.NewReader(archive))
	directoryModes := map[string]os.FileMode{}
	seen := map[string]bool{}
	globalHeader := false
	for {
		header, err := archiveReader.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			t.Fatal(err)
		}
		if header.Typeflag == tar.TypeXGlobalHeader {
			if globalHeader || header.Name != "pax_global_header" || len(header.PAXRecords) != 1 || header.PAXRecords["comment"] != receipt.BasisCommit {
				t.Fatal("original archive global source identity differs")
			}
			globalHeader = true
			continue
		}
		name := header.Name
		if header.Typeflag == tar.TypeDir {
			name = strings.TrimSuffix(name, "/")
		}
		if name == "" || path.Clean(name) != name || strings.HasPrefix(name, "../") || strings.HasPrefix(name, "/") {
			t.Fatal("archive contains a noncanonical path")
		}
		file := filepath.Join(root, filepath.FromSlash(name))
		switch header.Typeflag {
		case tar.TypeDir:
			if _, duplicate := directoryModes[name]; duplicate {
				t.Fatal("duplicate archived directory")
			}
			directoryModes[name] = os.FileMode(header.Mode) & 0o777
			if err := os.MkdirAll(file, 0o700); err != nil {
				t.Fatal(err)
			}
		case tar.TypeReg, tar.TypeRegA:
			row, exists := expected[name]
			if !exists || seen[name] {
				t.Fatal("unexpected or duplicate original archive file")
			}
			raw, err := io.ReadAll(archiveReader)
			if err != nil || uint64(len(raw)) != row.Bytes || int64(len(raw)) != header.Size || sha256Hex(raw) != row.SHA256 {
				t.Fatalf("original archived byte mismatch: %s", name)
			}
			if err := os.MkdirAll(filepath.Dir(file), 0o700); err != nil {
				t.Fatal(err)
			}
			if err := os.WriteFile(file, raw, os.FileMode(row.Mode)); err != nil {
				t.Fatal(err)
			}
			if err := os.Chmod(file, os.FileMode(row.Mode)); err != nil {
				t.Fatal(err)
			}
			seen[name] = true
		default:
			t.Fatal("unexpected archived source file kind")
		}
	}
	if !globalHeader || len(seen) != len(expected) || len(directoryModes) != 61 {
		t.Fatal("original source archive is incomplete")
	}
	var directories []string
	for name := range directoryModes {
		directories = append(directories, name)
	}
	sort.Slice(directories, func(i, j int) bool { return len(directories[i]) > len(directories[j]) })
	for _, name := range directories {
		if err := os.Chmod(filepath.Join(root, filepath.FromSlash(name)), directoryModes[name]); err != nil {
			t.Fatal(err)
		}
	}
	for name, row := range expected {
		file := filepath.Join(root, filepath.FromSlash(name))
		raw, err := os.ReadFile(file)
		if err != nil {
			t.Fatal(err)
		}
		info, err := os.Stat(file)
		if err != nil || uint64(len(raw)) != row.Bytes || sha256Hex(raw) != row.SHA256 || info.Mode().Perm() != os.FileMode(row.Mode) {
			t.Fatalf("restored source differs: %s", name)
		}
	}
	frontend := filepath.Join(root, "go-tools", "go2vir")
	physical, err := filepath.EvalSymlinks(frontend)
	if err != nil {
		t.Fatal(err)
	}
	return physical
}

// This observes the original registered generation, rather than substituting
// the evolving checkout. Reproduction still requires the original build recipe
// and independent Rust/source-root validation.
func TestPracticalRegisteredAnalysisOriginalCapture(t *testing.T) {
	frontend := restoreRegisteredAnalysisOriginal(t)
	roots := practicalAnalysisTestRoots(t, frontend)
	session := practicalTestSession(t, roots)
	inputs, err := session.capturePracticalAnalysis(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability})
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
	build := inputs.buildSources()
	if len(inputs.sources) != 48 || len(inputs.vendor) != 133 || len(build) != 184 || len(wire.sources) != 196 || len(inputs.modules) != 3 {
		t.Fatal("complete registered source capture differs")
	}
	objects := make([]practicalSourceObject, 0, len(build))
	for i, source := range build {
		if i > 0 && build[i-1].object.Path >= source.object.Path {
			t.Fatal("original build paths are not unique and ordered")
		}
		raw, exists := snapshot.sourceBytes(source.object.ID)
		if !exists || !bytes.Equal(raw, source.sourceBytes()) {
			t.Fatal("build input differs from captured original bytes")
		}
		objects = append(objects, source.object)
	}
	if out := os.Getenv("MPK_GO_REGISTERED_ANALYSIS_OUTPUT"); out != "" {
		if err := os.MkdirAll(filepath.Join(out, "original-bytes"), 0o700); err != nil {
			t.Fatal(err)
		}
		type sourceRecord struct {
			Object  practicalSourceObject `json:"object"`
			RawFile string                `json:"raw_file"`
		}
		var records []sourceRecord
		for _, source := range wire.sources {
			raw := source.sourceBytes()
			name := source.object.RawSHA256 + ".raw"
			file := filepath.Join(out, "original-bytes", name)
			old, readErr := os.ReadFile(file)
			if os.IsNotExist(readErr) {
				if err := os.WriteFile(file, raw, 0o600); err != nil {
					t.Fatal(err)
				}
			} else if readErr != nil || !bytes.Equal(old, raw) {
				t.Fatal("source bytes disagree for one digest")
			}
			records = append(records, sourceRecord{source.object, name})
		}
		var archiveIDs []string
		for _, module := range inputs.modules {
			archiveIDs = append(archiveIDs, module.archive.source.object.ID)
		}
		packet := map[string]any{"snapshot_hex": hex.EncodeToString(wire.canonical), "source_ids": snapshot.ids, "original_sources": records, "manifest_source_id": inputs.vendorManifest.object.ID, "archive_source_ids": archiveIDs, "build_objects": objects, "original_frontend_go_files": len(inputs.sources), "original_vendor_files": len(inputs.vendor), "original_registered_tree_files": 242, "original_full_source_epoch_files": 269, "registered_source_binary_proved": false, "launcher_authority_claimed": false}
		raw, err := json.MarshalIndent(packet, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(out, "native.json"), append(raw, '\n'), 0o600); err != nil {
			t.Fatal(err)
		}
	}
}
