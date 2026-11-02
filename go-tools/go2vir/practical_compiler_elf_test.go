//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"debug/elf"
	"encoding/binary"
	"encoding/hex"
	"encoding/json"
	"io"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
)

// Use the public ELF section reader to observe the actual pinned internal/zstd
// decoder, without modifying the SDK or importing/replacing internal packages.
func TestPracticalElfCompleteOriginalZstandardStreams(t *testing.T) {
	dir := os.Getenv("MPK_GO_ELF_ZSTD_OUTPUT")
	if dir == "" {
		t.Skip("original ELF Zstandard observations not requested")
	}
	if err := os.MkdirAll(dir, 0700); err != nil {
		t.Fatal(err)
	}
	paths, err := filepath.Glob(filepath.Join(runtime.GOROOT(), "src", "internal", "zstd", "testdata", "*.zst"))
	if err != nil || len(paths) != 3 {
		t.Fatalf("complete original streams: %v %v", paths, err)
	}
	rows := []any{}
	for _, path := range paths {
		raw, err := os.ReadFile(path)
		if err != nil {
			t.Fatal(err)
		}
		file := make([]byte, 152+len(raw))
		copy(file, []byte{'\x7f', 'E', 'L', 'F', 2, 1, 1})
		p := func(at int, value uint64, width int) {
			if width == 8 {
				binary.LittleEndian.PutUint64(file[at:], value)
			} else if width == 4 {
				binary.LittleEndian.PutUint32(file[at:], uint32(value))
			} else {
				binary.LittleEndian.PutUint16(file[at:], uint16(value))
			}
		}
		p(20, 1, 4)
		p(40, 64, 8)
		p(52, 64, 2)
		p(58, 64, 2)
		p(60, 1, 2)
		p(68, 1, 4)
		p(72, 0x800, 8)
		p(88, 128, 8)
		p(96, uint64(24+len(raw)), 8)
		p(128, 2, 4)
		p(144, 1, 8)
		copy(file[152:], raw)
		parsed, err := elf.NewFile(bytes.NewReader(file))
		if err != nil {
			t.Fatal(err)
		}
		decoded, err := io.ReadAll(parsed.Sections[0].Open())
		if err != nil {
			t.Fatal(err)
		}
		name := filepath.Base(path)
		if !strings.HasPrefix(sha256Hex(decoded), strings.Split(name, ".")[0]) {
			t.Fatal("original decompression identity:", name)
		}
		rows = append(rows, map[string]any{"name": name, "raw_hex": hex.EncodeToString(raw), "raw_sha256": sha256Hex(raw), "decoded_hex": hex.EncodeToString(decoded), "decoded_sha256": sha256Hex(decoded), "original_elf_hex": hex.EncodeToString(file)})
	}
	encoded, err := json.MarshalIndent(map[string]any{"go_version": runtime.Version(), "complete_original_streams": rows}, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "complete-original-zstandard.json"), append(encoded, '\n'), 0600); err != nil {
		t.Fatal(err)
	}
}
