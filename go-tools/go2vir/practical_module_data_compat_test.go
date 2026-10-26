//go:build go1.25 && (linux || darwin)

package main

import (
	"archive/zip"
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"testing"

	"golang.org/x/mod/sumdb/dirhash"
)

func TestPracticalModuleDataZIPCompatibilityOracle(t *testing.T) {
	practicalContractTestOutput(t, "module-data-compat")
	identity := practicalArchiveFixtureIdentity
	rows := []any{}
	observe := func(name string, raw []byte) {
		sum := "h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
		if reader, err := zip.NewReader(bytes.NewReader(raw), int64(len(raw))); err == nil {
			names := []string{}
			last := map[string]*zip.File{}
			for _, file := range reader.File {
				names = append(names, file.Name)
				last[file.Name] = file
			}
			if value, err := dirhash.Hash1(names, func(name string) (io.ReadCloser, error) { return last[name].Open() }); err == nil {
				sum = value
			}
		}
		source, err := newPracticalSourceBuffer("archives", "module.zip", "auxiliary", raw)
		if err != nil {
			t.Fatal(err)
		}
		parsed, err := parsePracticalModuleArchive(identity, source, sum, 131072)
		rows = append(rows, map[string]any{"case": name, "path": identity.Path, "version": identity.Version, "object": source.object, "raw": raw, "sum": sum, "remaining": 131072, "accepted": err == nil, "error": fmt.Sprint(err), "members": parsed.memberInventory(), "namespace_entries": parsed.namespaceEntries})
	}
	stored := practicalArchiveTestZIP(t, practicalArchiveTestEntries(identity, practicalArchiveFixtureFiles(t)), zip.Store)
	central := bytes.Index(stored, []byte{'P', 'K', 1, 2})
	if central < 0 {
		t.Fatal("original central directory missing")
	}
	for _, name := range []string{"no-descriptor-crc-zero", "no-descriptor-crc-wrong", "unix-symlink-attributes", "descriptor-crc-wrong", "unsupported-method"} {
		raw := bytes.Clone(stored)
		switch name {
		case "no-descriptor-crc-zero", "no-descriptor-crc-wrong":
			binary.LittleEndian.PutUint16(raw[central+8:], binary.LittleEndian.Uint16(raw[central+8:])&^8)
			crc := uint32(0)
			if name == "no-descriptor-crc-wrong" {
				crc = 1
			}
			binary.LittleEndian.PutUint32(raw[central+16:], crc)
		case "unix-symlink-attributes":
			binary.LittleEndian.PutUint16(raw[central+4:], (3<<8)|20)
			binary.LittleEndian.PutUint32(raw[central+38:], (0120000|0777)<<16)
		case "descriptor-crc-wrong":
			reader, err := zip.NewReader(bytes.NewReader(raw), int64(len(raw)))
			if err != nil {
				t.Fatal(err)
			}
			at, err := reader.File[0].DataOffset()
			if err != nil {
				t.Fatal(err)
			}
			raw[at+int64(reader.File[0].CompressedSize64)+4] ^= 1
		case "unsupported-method":
			binary.LittleEndian.PutUint16(raw[central+10:], 99)
		}
		observe(name, raw)
	}
	// A descriptor without the optional signature; subsequent local offsets and
	// the original central directory position retain their exact meanings.
	reader, err := zip.NewReader(bytes.NewReader(stored), int64(len(stored)))
	if err != nil {
		t.Fatal(err)
	}
	start, err := reader.File[0].DataOffset()
	if err != nil {
		t.Fatal(err)
	}
	descriptor := int(start) + int(reader.File[0].CompressedSize64)
	if binary.LittleEndian.Uint32(stored[descriptor:]) != 0x08074b50 {
		t.Fatal("original descriptor signature missing")
	}
	unsigned := append(bytes.Clone(stored[:descriptor]), stored[descriptor+4:]...)
	newCentral := central - 4
	for at := newCentral; at+46 <= len(unsigned) && binary.LittleEndian.Uint32(unsigned[at:]) == 0x02014b50; {
		offset := binary.LittleEndian.Uint32(unsigned[at+42:])
		if int(offset) > descriptor {
			binary.LittleEndian.PutUint32(unsigned[at+42:], offset-4)
		}
		at += 46 + int(binary.LittleEndian.Uint16(unsigned[at+28:])) + int(binary.LittleEndian.Uint16(unsigned[at+30:])) + int(binary.LittleEndian.Uint16(unsigned[at+32:]))
	}
	binary.LittleEndian.PutUint32(unsigned[len(unsigned)-22+16:], uint32(newCentral))
	observe("unsigned-descriptor", unsigned)

	// Original ZIP64 member fields, independent of the separate ZIP64 end case.
	for _, name := range []string{"zip64-member", "zip64-member-short", "zip64-negative-compressed-size"} {
		raw := bytes.Clone(stored)
		n := int(binary.LittleEndian.Uint16(raw[central+28:]))
		e := int(binary.LittleEndian.Uint16(raw[central+30:]))
		at := central + 46 + n + e
		extra := make([]byte, 28)
		binary.LittleEndian.PutUint16(extra, 1)
		binary.LittleEndian.PutUint16(extra[2:], 24)
		binary.LittleEndian.PutUint64(extra[4:], uint64(binary.LittleEndian.Uint32(raw[central+24:])))
		binary.LittleEndian.PutUint64(extra[12:], uint64(binary.LittleEndian.Uint32(raw[central+20:])))
		binary.LittleEndian.PutUint64(extra[20:], uint64(binary.LittleEndian.Uint32(raw[central+42:])))
		if name == "zip64-member-short" {
			extra = extra[:12]
			binary.LittleEndian.PutUint16(extra[2:], 8)
		}
		if name == "zip64-negative-compressed-size" {
			binary.LittleEndian.PutUint64(extra[4:], 0)
			binary.LittleEndian.PutUint64(extra[12:], ^uint64(0))
			binary.LittleEndian.PutUint16(raw[central+8:], binary.LittleEndian.Uint16(raw[central+8:])&^8)
			binary.LittleEndian.PutUint32(raw[central+16:], 0)
		}
		binary.LittleEndian.PutUint32(raw[central+20:], ^uint32(0))
		binary.LittleEndian.PutUint32(raw[central+24:], ^uint32(0))
		binary.LittleEndian.PutUint32(raw[central+42:], ^uint32(0))
		binary.LittleEndian.PutUint16(raw[central+30:], uint16(e+len(extra)))
		raw = append(append(bytes.Clone(raw[:at]), extra...), raw[at:]...)
		end := len(raw) - 22
		binary.LittleEndian.PutUint32(raw[end+12:], binary.LittleEndian.Uint32(raw[end+12:])+uint32(len(extra)))
		observe(name, raw)
	}
	directory := practicalArchiveTestZIP(t, practicalArchiveTestEntries(identity, []practicalArchiveTestEntry{{name: "directory/"}}), zip.Deflate)
	for _, name := range []string{"opaque-compressed-directory-metadata", "directory-uncompressed-data"} {
		raw := bytes.Clone(directory)
		at := bytes.Index(raw, []byte{'P', 'K', 1, 2})
		binary.LittleEndian.PutUint32(raw[at+20:], 7)
		binary.LittleEndian.PutUint16(raw[at+10:], 99)
		binary.LittleEndian.PutUint32(raw[at+16:], 123)
		if name == "directory-uncompressed-data" {
			binary.LittleEndian.PutUint32(raw[at+24:], 1)
		}
		observe(name, raw)
	}
	retainPracticalSyntaxEvidence(t, "original-zip-compatibility.json", map[string]any{"cases": rows, "whole_original_inputs_retained": true, "original_scope_not_narrowed_to_stricter_zip_dispositions": true})
}
