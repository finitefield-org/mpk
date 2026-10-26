//go:build go1.25 && (linux || darwin)

package main

import (
	"archive/zip"
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"sort"
	"strings"
	"testing"
	"unicode"
	"unicode/utf8"

	"golang.org/x/mod/module"
	"golang.org/x/mod/sumdb/dirhash"
)

func TestPracticalModuleDataOriginalOracle(t *testing.T) {
	practicalContractTestOutput(t, "module-data-oracle")
	folds := [][2]uint32{}
	var table strings.Builder
	table.WriteString("// Generated from the complete pinned Go1.25 Unicode simple-fold observation.\n// Native producer, complete pairs and all scalar comparisons are retained.\npub(super) const PAIRS: &[(u32,u32)] = &[\n")
	for r := rune(0); r <= unicode.MaxRune; r++ {
		if !utf8.ValidRune(r) {
			continue
		}
		fold := []rune(foldPracticalArchiveName(string(r)))[0]
		if fold != r {
			folds = append(folds, [2]uint32{uint32(r), uint32(fold)})
			fmt.Fprintf(&table, "(%d,%d),\n", r, fold)
		}
	}
	table.WriteString("];\n")
	retainPracticalSyntaxEvidence(t, "original-simple-fold.json", map[string]any{"unicode_version": unicode.Version, "pairs": folds, "rust_source": table.String(), "all_original_scalar_values_observed": true})

	const sum = "h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
	sums := []any{}
	addSum := func(name, kind string, raw []byte) {
		source, err := newPracticalSourceBuffer("sums", "go.sum", kind, raw)
		if err != nil {
			t.Fatal(err)
		}
		parsed, err := parsePracticalModuleSums(source)
		rows := []any{}
		if err == nil {
			keys := []module.Version{}
			for key := range parsed.entries {
				keys = append(keys, key)
			}
			sort.Slice(keys, func(i, j int) bool {
				if keys[i].Path != keys[j].Path {
					return keys[i].Path < keys[j].Path
				}
				return keys[i].Version < keys[j].Version
			})
			for _, key := range keys {
				rows = append(rows, map[string]any{"path": key.Path, "version": key.Version, "sum": parsed.entries[key]})
			}
		}
		sums = append(sums, map[string]any{"case": name, "object": source.object, "raw": raw, "accepted": err == nil, "error": fmt.Sprint(err), "entries": rows})
	}
	paths := []string{"example.com/dep", "example.com/Foo", "example.com/a..b", "example.com/x/v2", "example.com/x/v10", "example.com/x/v0", "example.com/x/v1", "example.com/x/v01", "example.com/x/v1.2", "example.com/x/vx", "example.com/x/v", "example.com/Con", "example.com/com1.txt", "example.com/a~1", "example.com/a~1.txt", "example.com/a~b", "example.com/a+b", "example.com/.dot", "example.com/a.", "example.com/a//b", "example.com/a/", "Example.com/dep", "-example.com/dep", "example/dep", "../dep", "gopkg.in/name.v0", "gopkg.in/name.v0-unstable", "gopkg.in/name.v1", "gopkg.in/name.v2", "gopkg.in/name.v2-unstable", "gopkg.in/name.v01", "gopkg.in/name", "example.com/σ"}
	versions := []string{"v0.0.0", "v1.2.3", "v2.0.0", "v10.0.0", "v2.0.0+incompatible", "v0.0.0-20161208181325-20d25e280405", "v1.2.3-rc.1", "v1.2.3-0", "v1.2.3-01", "v1.2.3-pre-foo+incompatible", "v1.2.3+metadata", "v1.2", "v1", "v01.2.3", "v1.2.03", "v1.2.3-pre..x", "v1.2.3-", "v1.2.3+incompatible+incompatible", "v9999999999999999999999.0.0+incompatible"}
	for _, path := range paths {
		for _, version := range versions {
			for _, suffix := range []string{"", "/go.mod"} {
				addSum(path+"@"+version+suffix, "go_sum", []byte(path+" "+version+suffix+" "+sum+"\n"))
			}
		}
	}
	prefix := "example.com/dep v1.0.0 "
	for name, raw := range map[string]string{"empty": "", "identical-duplicate": prefix + sum + "\n" + prefix + sum + "\n", "conflict": prefix + sum + "\n" + prefix + "h1:RNha+Ovptpv+50nN2NH8md8DuZZwwlLUzexiDm4kE8g=\n", "extra-field": prefix + sum + " extra", "missing-field": prefix, "algorithm": prefix + strings.Replace(sum, "h1:", "h2:", 1), "short": prefix + "h1:YQ==", "base64": prefix + "h1:" + strings.Repeat("!", 43) + "=", "noncanonical-base64": prefix + strings.TrimSuffix(sum, "A=") + "B=", "bad-suffix": "example.com/dep v1.0.0/go.mod/go.mod " + sum, "binary": string([]byte{255, 0})} {
		addSum(name, "go_sum", []byte(raw))
	}
	for r := rune(0); r <= unicode.MaxRune; r++ {
		if unicode.IsSpace(r) {
			addSum(fmt.Sprintf("space-%x", r), "go_sum", []byte("example.com/dep"+string(r)+"v1.0.0"+string(r)+sum))
		}
	}
	addSum("wrong-role", "go_mod", []byte(prefix+sum))
	retainPracticalSyntaxEvidence(t, "original-module-sums.json", map[string]any{"cases": sums, "whole_original_inputs_retained": true})

	archives := []any{}
	identity := practicalArchiveFixtureIdentity
	prefixZIP := identity.Path + "@" + identity.Version + "/"
	addZIP := func(name string, raw []byte, remaining int) {
		expected := sum
		if reader, err := zip.NewReader(bytes.NewReader(raw), int64(len(raw))); err == nil {
			names := []string{}
			last := map[string]*zip.File{}
			for _, file := range reader.File {
				names = append(names, file.Name)
				last[file.Name] = file
			}
			if observed, err := dirhash.Hash1(names, func(name string) (io.ReadCloser, error) { return last[name].Open() }); err == nil {
				expected = observed
			}
		}
		source, err := newPracticalSourceBuffer("archives", "module.zip", "auxiliary", raw)
		if err != nil {
			t.Fatal(err)
		}
		parsed, err := parsePracticalModuleArchive(identity, source, expected, remaining)
		archives = append(archives, map[string]any{"case": name, "path": identity.Path, "version": identity.Version, "object": source.object, "raw": raw, "sum": expected, "remaining": remaining, "accepted": err == nil, "error": fmt.Sprint(err), "members": parsed.memberInventory(), "namespace_entries": parsed.namespaceEntries})
	}
	fixture := practicalArchiveTestEntries(identity, practicalArchiveFixtureFiles(t))
	stored := practicalArchiveTestZIP(t, fixture, zip.Store)
	compressed := practicalArchiveTestZIP(t, fixture, zip.Deflate)
	addZIP("stored", stored, 131072)
	addZIP("deflated", compressed, 131072)
	addZIP("self-extract-prefix", append([]byte("whole original self extracting prefix\n"), compressed...), 131072)
	addZIP("trailing-data", append(bytes.Clone(compressed), []byte("whole trailing bytes")...), 131072)
	addZIP("empty", practicalArchiveTestZIP(t, nil, zip.Store), 0)
	addZIP("opaque-directories", practicalArchiveTestZIP(t, practicalArchiveTestEntries(identity, []practicalArchiveTestEntry{{name: ""}, {name: "folder/"}, {name: "folder/"}, {name: "folder/empty"}, {name: strings.Repeat("a", 2048) + ".bin", data: []byte{255, 0, 1}}}), zip.Deflate), 5)
	for name, paths := range map[string][]string{"escape": {"../escape.go"}, "wrong-prefix": {"example.com/other@v1.0.0/file.go"}, "dot-segment": {prefixZIP + "x/../file.go"}, "absolute": {prefixZIP + "/file.go"}, "backslash": {prefixZIP + "x\\file.go"}, "duplicate-file": {prefixZIP + "file.go", prefixZIP + "file.go"}, "case-file": {prefixZIP + "File.go", prefixZIP + "file.go"}, "case-parent": {prefixZIP + "A/x.go", prefixZIP + "a/y.go"}, "simple-fold": {prefixZIP + "σ/x.go", prefixZIP + "ς/y.go"}, "file-directory": {prefixZIP + "a", prefixZIP + "a/x.go"}, "nested-manifest": {prefixZIP + "nested/go.mod"}, "manifest-case": {prefixZIP + "Go.Mod"}} {
		files := []practicalArchiveTestEntry{}
		for _, path := range paths {
			files = append(files, practicalArchiveTestEntry{name: path, data: []byte("whole original bytes")})
		}
		addZIP(name, practicalArchiveTestZIP(t, files, zip.Store), 131072)
	}
	reader, err := zip.NewReader(bytes.NewReader(stored), int64(len(stored)))
	if err != nil {
		t.Fatal(err)
	}
	dataOffset, err := reader.File[0].DataOffset()
	if err != nil {
		t.Fatal(err)
	}
	for _, name := range []string{"body", "crc", "declared-size", "protocol-size", "truncated", "local-header"} {
		raw := bytes.Clone(stored)
		central := bytes.Index(raw, []byte{'P', 'K', 1, 2})
		switch name {
		case "body":
			raw[dataOffset] ^= 1
		case "crc":
			raw[central+16] ^= 1
		case "declared-size":
			binary.LittleEndian.PutUint32(raw[central+24:], 1)
		case "protocol-size":
			binary.LittleEndian.PutUint32(raw[central+24:], practicalGoModuleZIPBytes+1)
		case "truncated":
			raw = raw[:len(raw)-8]
		case "local-header":
			raw[0] ^= 1
		}
		addZIP(name, raw, 131072)
	}
	// ZIP64 end/locator for the same complete original central directory.
	end := len(compressed) - 22
	zip64 := bytes.Clone(compressed[:end])
	record := make([]byte, 56)
	binary.LittleEndian.PutUint32(record, 0x06064b50)
	binary.LittleEndian.PutUint64(record[4:], 44)
	binary.LittleEndian.PutUint64(record[24:], 2)
	binary.LittleEndian.PutUint64(record[32:], 2)
	binary.LittleEndian.PutUint64(record[40:], uint64(binary.LittleEndian.Uint32(compressed[end+12:])))
	binary.LittleEndian.PutUint64(record[48:], uint64(binary.LittleEndian.Uint32(compressed[end+16:])))
	zip64 = append(zip64, record...)
	locator := make([]byte, 20)
	binary.LittleEndian.PutUint32(locator, 0x07064b50)
	binary.LittleEndian.PutUint64(locator[8:], uint64(end))
	binary.LittleEndian.PutUint32(locator[16:], 1)
	zip64 = append(zip64, locator...)
	ending := bytes.Clone(compressed[end:])
	binary.LittleEndian.PutUint16(ending[10:], 65535)
	zip64 = append(zip64, ending...)
	addZIP("zip64-end", zip64, 131072)
	// Full expanded protocol boundary, streamed rather than allocating 500 MiB.
	var big bytes.Buffer
	writer := zip.NewWriter(&big)
	file, err := writer.CreateHeader(&zip.FileHeader{Name: prefixZIP + "opaque.bin", Method: zip.Deflate})
	if err != nil {
		t.Fatal(err)
	}
	chunk := make([]byte, 65536)
	for left := practicalGoModuleZIPBytes; left > 0; left -= len(chunk) {
		if _, err := file.Write(chunk[:min(left, len(chunk))]); err != nil {
			t.Fatal(err)
		}
	}
	if err := writer.Close(); err != nil {
		t.Fatal(err)
	}
	addZIP("full-expanded-500MiB", big.Bytes(), 131072)
	deep := []practicalArchiveTestEntry{}
	remaining := 131072
	for branch := 0; remaining > 0; branch++ {
		parts := min(remaining, 32000)
		name := string(rune('a'+branch)) + "/" + strings.Repeat("d/", parts-2) + "f"
		deep = append(deep, practicalArchiveTestEntry{name: prefixZIP + name})
		remaining -= parts
	}
	rawDeep := practicalArchiveTestZIP(t, deep, zip.Store)
	addZIP("full-namespace-131072", rawDeep, 131072)
	addZIP("namespace-one-short", rawDeep, 131071)
	retainPracticalSyntaxEvidence(t, "original-module-archives.json", map[string]any{"cases": archives, "whole_original_inputs_retained": true, "full_expanded_protocol_bytes": practicalGoModuleZIPBytes, "full_namespace_bound": 131072})
}
