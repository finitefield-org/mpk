package main

import (
	"archive/zip"
	"bytes"
	"encoding/binary"
	"encoding/json"
	"io"
	"os"
	"strings"
	"testing"
	"time"

	"golang.org/x/mod/module"
	"golang.org/x/mod/sumdb/dirhash"
)

var practicalArchiveFixtureIdentity = module.Version{Path: "example.com/pracdep", Version: "v1.0.0"}

const practicalArchiveFixtureSum = "h1:MTsAUxsuzB4cUq0dDw78AH8ovfJ05oBPSyUb+5y8IPA="

type practicalArchiveTestEntry struct {
	name string
	data []byte
	mode os.FileMode
}

func practicalArchiveFixtureFiles(t *testing.T) []practicalArchiveTestEntry {
	t.Helper()
	raw, err := os.ReadFile("../../develop/probes/go-practical/module-cases.json")
	if err != nil {
		t.Fatal(err)
	}
	var fixture struct {
		Dependency struct{ Files map[string]string }
	}
	if err := json.Unmarshal(raw, &fixture); err != nil {
		t.Fatal(err)
	}
	return []practicalArchiveTestEntry{{name: "go.mod", data: []byte(fixture.Dependency.Files["go.mod"])}, {name: "dep.go", data: []byte(fixture.Dependency.Files["dep.go"])}}
}

func practicalArchiveTestZIP(t *testing.T, entries []practicalArchiveTestEntry, method uint16) []byte {
	t.Helper()
	var buffer bytes.Buffer
	writer := zip.NewWriter(&buffer)
	for _, entry := range entries {
		header := &zip.FileHeader{Name: entry.name, Method: method, Modified: time.Date(2020, 1, 2, 3, 4, 5, 0, time.UTC)}
		if entry.mode != 0 {
			header.SetMode(entry.mode)
		}
		file, err := writer.CreateHeader(header)
		if err != nil {
			t.Fatal(err)
		}
		if _, err := file.Write(entry.data); err != nil {
			t.Fatal(err)
		}
	}
	if err := writer.Close(); err != nil {
		t.Fatal(err)
	}
	return buffer.Bytes()
}

func practicalArchiveTestEntries(identity module.Version, files []practicalArchiveTestEntry) []practicalArchiveTestEntry {
	entries := make([]practicalArchiveTestEntry, len(files))
	for i, file := range files {
		entries[i] = file
		entries[i].name = identity.Path + "@" + identity.Version + "/" + file.name
	}
	return entries
}

func practicalArchiveTestSum(t *testing.T, raw []byte) string {
	t.Helper()
	reader, err := zip.NewReader(bytes.NewReader(raw), int64(len(raw)))
	if err != nil {
		t.Fatal(err)
	}
	names := []string{}
	files := map[string]*zip.File{}
	for _, file := range reader.File {
		names = append(names, file.Name)
		files[file.Name] = file
	}
	sum, err := dirhash.Hash1(names, func(name string) (io.ReadCloser, error) { return files[name].Open() })
	if err != nil {
		t.Fatal(err)
	}
	return sum
}

func practicalArchiveTestBuffer(t *testing.T, raw []byte) practicalSourceBuffer {
	t.Helper()
	buffer, err := newPracticalSourceBuffer("archives", "module.zip", "auxiliary", raw)
	if err != nil {
		t.Fatal(err)
	}
	return buffer
}

func TestPracticalArchiveMatchesIndependentGoChecksumAndRawIdentity(t *testing.T) {
	// The h1 value is independently observed by pinned Go in T01-W02.
	files := practicalArchiveFixtureFiles(t)
	stored := practicalArchiveTestZIP(t, practicalArchiveTestEntries(practicalArchiveFixtureIdentity, files), zip.Store)
	files[0], files[1] = files[1], files[0]
	files[0].mode = os.ModeSymlink | 0o777 // ZIP attributes are not filesystem links.
	compressed := practicalArchiveTestZIP(t, practicalArchiveTestEntries(practicalArchiveFixtureIdentity, files), zip.Deflate)
	var firstID string
	for index, raw := range [][]byte{stored, compressed} {
		buffer := practicalArchiveTestBuffer(t, raw)
		archive, err := parsePracticalModuleArchive(practicalArchiveFixtureIdentity, buffer, practicalArchiveFixtureSum, 2)
		if err != nil {
			t.Fatal(err)
		}
		if archive.sum != practicalArchiveFixtureSum || len(archive.members) != 2 || archive.namespaceEntries != 2 {
			t.Fatal("original Go module checksum or complete namespace differs")
		}
		if index == 0 {
			firstID = archive.source.object.ID
		} else if firstID == archive.source.object.ID {
			t.Fatal("different original ZIP encodings lost their raw source identity")
		}
		for _, file := range files {
			kind := "go_file"
			if file.name == "go.mod" {
				kind = "go_mod"
			}
			original, err := newPracticalSourceBuffer("original-module", file.name, kind, file.data)
			if err != nil || archive.verifyContent(file.name, original) != nil {
				t.Fatal("archive does not bind complete original file bytes")
			}
		}
		inventory := archive.memberInventory()
		if inventory[0].Path != "dep.go" || inventory[1].Path != "go.mod" {
			t.Fatal("archive inventory is not in exact source path order")
		}
		inventory[0].RawSHA256 = "caller mutation"
		if archive.memberInventory()[0].RawSHA256 == "caller mutation" {
			t.Fatal("returned inventory aliases private archive metadata")
		}
	}
}

func TestPracticalArchiveRejectsInvalidOriginalNamespaces(t *testing.T) {
	prefix := practicalArchiveFixtureIdentity.Path + "@" + practicalArchiveFixtureIdentity.Version + "/"
	cases := map[string][]string{
		"escape": {"../escape.go"}, "wrong-prefix": {"example.com/other@v1.0.0/file.go"},
		"dot-segment": {prefix + "x/../file.go"}, "absolute": {prefix + "/file.go"},
		"backslash": {prefix + "x\\file.go"}, "duplicate-file": {prefix + "file.go", prefix + "file.go"},
		"case-file": {prefix + "File.go", prefix + "file.go"}, "case-parent": {prefix + "A/x.go", prefix + "a/y.go"},
		"unicode-simple-fold": {prefix + "σ/x.go", prefix + "ς/y.go"},
		"file-directory":      {prefix + "a", prefix + "a/x.go"},
		"nested-manifest":     {prefix + "nested/go.mod"}, "manifest-case": {prefix + "Go.Mod"},
	}
	for name, paths := range cases {
		t.Run(name, func(t *testing.T) {
			entries := []practicalArchiveTestEntry{}
			for _, path := range paths {
				entries = append(entries, practicalArchiveTestEntry{name: path, data: []byte("original")})
			}
			raw := practicalArchiveTestZIP(t, entries, zip.Store)
			// Even a matching content checksum cannot authorize an unsafe namespace.
			if _, err := parsePracticalModuleArchive(practicalArchiveFixtureIdentity, practicalArchiveTestBuffer(t, raw), practicalArchiveTestSum(t, raw), 100); err == nil {
				t.Fatal("invalid original module namespace accepted")
			}
		})
	}
}

func TestPracticalArchiveRejectsCorruptBodyCRCAndSizes(t *testing.T) {
	raw := practicalArchiveTestZIP(t, practicalArchiveTestEntries(practicalArchiveFixtureIdentity, practicalArchiveFixtureFiles(t)), zip.Store)
	reader, err := zip.NewReader(bytes.NewReader(raw), int64(len(raw)))
	if err != nil {
		t.Fatal(err)
	}
	dataOffset, err := reader.File[0].DataOffset()
	if err != nil {
		t.Fatal(err)
	}
	central := bytes.Index(raw, []byte{'P', 'K', 1, 2})
	if central < 0 {
		t.Fatal("missing central directory in original fixture")
	}
	for _, name := range []string{"body", "crc", "declared-size", "protocol-size", "truncated"} {
		t.Run(name, func(t *testing.T) {
			changed := bytes.Clone(raw)
			switch name {
			case "body":
				changed[dataOffset] ^= 1
			case "crc":
				changed[central+16] ^= 1
			case "declared-size":
				binary.LittleEndian.PutUint32(changed[central+24:], uint32(reader.File[0].UncompressedSize64+1))
			case "protocol-size":
				binary.LittleEndian.PutUint32(changed[central+24:], practicalGoModuleZIPBytes+1)
			case "truncated":
				changed = changed[:len(changed)-5]
			}
			if _, err := parsePracticalModuleArchive(practicalArchiveFixtureIdentity, practicalArchiveTestBuffer(t, changed), practicalArchiveFixtureSum, 100); err == nil {
				t.Fatal("corrupt bytes, CRC or header size accepted")
			}
		})
	}
	if _, err := parsePracticalModuleArchive(practicalArchiveFixtureIdentity, practicalArchiveTestBuffer(t, raw), "h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=", 100); err == nil {
		t.Fatal("stale archive checksum accepted")
	}
	// Go HashZip selects the last payload for a repeated directory name. A
	// shadowed nonempty directory still owes Go's original ZIP validation.
	// Build regular headers, then replace equal-length names: Go's ZIP writer
	// correctly refuses to manufacture nonempty directory entries directly.
	directories := practicalArchiveTestEntries(practicalArchiveFixtureIdentity, []practicalArchiveTestEntry{{name: "dirX", data: []byte("first")}, {name: "dirX"}})
	shadowed := practicalArchiveTestZIP(t, directories, zip.Store)
	prefix := practicalArchiveFixtureIdentity.Path + "@" + practicalArchiveFixtureIdentity.Version + "/"
	shadowed = bytes.ReplaceAll(shadowed, []byte(prefix+"dirX"), []byte(prefix+"dir/"))
	expected := practicalArchiveTestSum(t, shadowed)
	if _, err := parsePracticalModuleArchive(practicalArchiveFixtureIdentity, practicalArchiveTestBuffer(t, shadowed), expected, 10); err == nil {
		t.Fatal("invalid shadowed directory payload bypassed ZIP validation")
	}

}

func TestPracticalArchivePreservesOpaqueAssetsAndDirectoryHeaders(t *testing.T) {
	files := []practicalArchiveTestEntry{{name: ""}, {name: "folder/"}, {name: "folder/"}, {name: "folder/empty", data: nil}, {name: strings.Repeat("a", 2048) + ".bin", data: []byte{0xff, 0, 1}}}
	raw := practicalArchiveTestZIP(t, practicalArchiveTestEntries(practicalArchiveFixtureIdentity, files), zip.Deflate)
	archive, err := parsePracticalModuleArchive(practicalArchiveFixtureIdentity, practicalArchiveTestBuffer(t, raw), practicalArchiveTestSum(t, raw), 5)
	if err != nil {
		t.Fatal(err)
	}
	if len(archive.members) != 2 || archive.namespaceEntries != 5 {
		t.Fatalf("directory/root/repeated headers or exact opaque names lost: %d, %d", len(archive.members), archive.namespaceEntries)
	}
	member, err := newPracticalSourceBuffer("original", "folder/empty", "auxiliary", nil)
	if err != nil || archive.verifyContent("folder/empty", member) != nil {
		t.Fatal("zero-byte original member was lost")
	}
	if archive.verifyContent("missing", member) == nil {
		t.Fatal("uncaptured archive member was synthesized")
	}
	changed, err := newPracticalSourceBuffer("original", "folder/empty", "auxiliary", []byte{0})
	if err != nil || archive.verifyContent("folder/empty", changed) == nil {
		t.Fatal("changed original content accepted")
	}
}

func TestPracticalArchiveBoundsImplicitNamespaceAndEmptyArchive(t *testing.T) {
	raw := practicalArchiveTestZIP(t, practicalArchiveTestEntries(practicalArchiveFixtureIdentity, []practicalArchiveTestEntry{{name: "deep/member"}}), zip.Store)
	for _, remaining := range []int{1, 2} {
		archive, err := parsePracticalModuleArchive(practicalArchiveFixtureIdentity, practicalArchiveTestBuffer(t, raw), practicalArchiveTestSum(t, raw), remaining)
		if remaining == 1 && err == nil || remaining == 2 && (err != nil || archive.namespaceEntries != 2) {
			t.Fatal("implicit namespace parent did not consume the inclusive entry bound")
		}
	}
	empty := practicalArchiveTestZIP(t, nil, zip.Store)
	archive, err := parsePracticalModuleArchive(practicalArchiveFixtureIdentity, practicalArchiveTestBuffer(t, empty), practicalArchiveTestSum(t, empty), 0)
	if err != nil || len(archive.members) != 0 || archive.namespaceEntries != 0 {
		t.Fatal("valid empty original module archive rejected")
	}
}

func TestPracticalArchiveRequiresExactIdentityAndSourceRole(t *testing.T) {
	raw := practicalArchiveTestZIP(t, practicalArchiveTestEntries(practicalArchiveFixtureIdentity, practicalArchiveFixtureFiles(t)), zip.Store)
	for _, identity := range []module.Version{{Path: "example.com/pracdep", Version: "v1.0"}, {Path: "example.com/pracdep", Version: "v1.0.1"}, {Path: "example.com/other", Version: "v1.0.0"}} {
		if _, err := parsePracticalModuleArchive(identity, practicalArchiveTestBuffer(t, raw), practicalArchiveFixtureSum, 100); err == nil {
			t.Fatal("noncanonical or crossed module archive identity accepted")
		}
	}
	wrongRole, err := newPracticalSourceBuffer("archives", "module.zip", "metadata", raw)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := parsePracticalModuleArchive(practicalArchiveFixtureIdentity, wrongRole, practicalArchiveFixtureSum, 100); err == nil {
		t.Fatal("wrong original ZIP source role accepted")
	}
	buffer := practicalArchiveTestBuffer(t, raw)
	buffer.data[0] ^= 1
	if _, err := parsePracticalModuleArchive(practicalArchiveFixtureIdentity, buffer, practicalArchiveFixtureSum, 100); err == nil {
		t.Fatal("corrupted owned archive buffer accepted")
	}
}
