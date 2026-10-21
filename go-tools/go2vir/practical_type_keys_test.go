//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"testing"
)

func TestPracticalTypeKeysRequireOriginalOwner(t *testing.T) {
	for _, owner := range []*practicalOriginalPackages{nil, {}} {
		if keys, err := owner.typeKeys("example.com/policy", practicalSourceSpan{}); err == nil || len(keys.canonical) != 0 {
			t.Fatal("unowned type components accepted", err)
		}
	}
}
func TestPracticalTypeKeysFromOriginalSource(t *testing.T) {
	text := []byte("package policy\r\nimport \"example.com/policy/helper\"\r\ntype Count uint16\r\ntype Alias = Count\r\ntype Other uint16\r\ntype Root struct {\r\n A struct { Value Alias }; B struct { Value Count }; E struct { Count }; D struct { Count Count }\r\n Tags struct { Raw byte \"\\xff\"; Replacement byte \"�\"; Empty byte }\r\n Names struct { É byte; ǅ byte; ᴬ byte; _ byte; _ byte }\r\n Words struct { I int; I8 int8; I16 int16; I32 int32; I64 int64; U uint; U8 uint8; U16 uint16; U32 uint32; U64 uint64; R rune; B byte; Flag bool; Text string }\r\n Zero [0]Other; Max [256]byte; Ref []Count; Empty struct{}; Err error; Foreign helper.Record\r\n}\r\nfunc Inspect(root Root, alias Alias, direct Count, imported helper.Record, values [2]byte, index uint32) {\r\n _ = root; _ = alias; _ = direct; _ = imported; _ = values[index]\r\n}\r\nfunc init(){panic(\"application must not execute\")}\r\n")
	files := map[string][]byte{"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"), "policy.go": text, "helper/helper.go": []byte("package helper\ntype Record struct { private uint16; Exported byte }\n")}
	owner, original := practicalStorageTestOwner(t, files)
	source := original["policy.go"]
	var first practicalTypeKeyComponents
	for n, expression := range []string{"root", "alias", "direct", "imported", "values[index]"} {
		t.Run(expression, func(t *testing.T) {
			span := practicalStorageTestSpan(t, source, expression)
			keys, err := owner.typeKeys("example.com/policy", span)
			if err != nil {
				t.Fatal(err)
			}
			data := map[string]any{"canonical_components": bytes.Clone(keys.canonical), "sources": []map[string]any{}, "source_typing_or_registered_context_proof": false}
			buffers := []map[string]any{}
			for _, buffer := range keys.sources {
				buffers = append(buffers, map[string]any{"object": buffer.object, "raw": buffer.sourceBytes()})
			}
			data["sources"] = buffers
			retainPracticalSyntaxEvidence(t, fmt.Sprintf("type-keys-%02d.json", n), data)
			if n == 0 {
				first.canonical = bytes.Clone(keys.canonical)
			}
			if n == 0 {
				parsed, err := decodeStrictJSON(keys.canonical)
				if err != nil {
					t.Fatal(err)
				}
				wire := parsed.(map[string]any)
				nodes := map[string]map[string]any{}
				builtins := map[string]string{}
				structs := []map[string]any{}
				for _, raw := range wire["types"].([]any) {
					row := raw.(map[string]any)
					id := row["id"].(string)
					nodes[id] = row
					if row["kind"] == "builtin" {
						builtins[row["name"].(string)] = id
					}
					if row["kind"] == "struct" {
						structs = append(structs, row)
					}
				}
				if builtins["int"] == builtins["int64"] || builtins["uint"] == builtins["uint64"] || builtins["byte"] != "" || builtins["rune"] != "" {
					t.Fatal("native names or scalar aliases corrupted type identity")
				}
				for _, name := range []string{"bool", "string", "error", "int", "int8", "int16", "int32", "int64", "uint", "uint8", "uint16", "uint32", "uint64"} {
					if builtins[name] == "" {
						t.Fatal("missing builtin", name)
					}
				}
				fields := map[string]map[string]any{}
				for _, row := range structs {
					for _, value := range row["fields"].([]any) {
						field := value.(map[string]any)
						fields[field["name"].(string)] = field
					}
				}
				if fields["A"]["type_id"] != fields["B"]["type_id"] || fields["E"]["type_id"] == fields["D"]["type_id"] {
					t.Fatal("transparent alias or embedded-field identity lost")
				}
				if fields["Raw"]["tag_bytes_hex"] != "ff" || fields["Replacement"]["tag_bytes_hex"] != hex.EncodeToString([]byte("�")) {
					t.Fatal("raw non-UTF8 tag bytes replaced")
				}
				if fields["É"]["unexported_package_id"] != nil || fields["ǅ"]["unexported_package_id"] == nil || fields["ᴬ"]["unexported_package_id"] == nil {
					t.Fatal("export status differs from pinned Go Unicode Lu")
				}
				if len(wire["packages"].([]any)) != 2 || len(wire["source_objects"].([]any)) != 3 || len(wire["aliases"].([]any)) != 1 {
					t.Fatal("incomplete original package/source/alias membership")
				}
			}
			clear(keys.canonical)
			for _, buffer := range keys.sources {
				clear(buffer.data)
			}
			clear(keys.nodeKeys)
			again, err := owner.typeKeys("example.com/policy", span)
			if err != nil || !bytes.Equal(again.canonical, data["canonical_components"].([]byte)) {
				t.Fatal("caller changed owned type components", err)
			}
		})
	}
	// Keys contain logical source IDs only. A native display string or physical
	// path cannot become an alternate nominal key input.
	if bytes.Contains(first.canonical, []byte("/private/")) || bytes.Contains(first.canonical, []byte("native_display")) {
		t.Fatal("physical/native display metadata entered type keys")
	}
	span := practicalStorageTestSpan(t, source, "root")
	span.end--
	if _, err := owner.typeKeys("example.com/policy", span); err == nil {
		t.Fatal("inexact physical expression accepted")
	}
	owner.Close()
	if _, err := owner.typeKeys("example.com/policy", practicalStorageTestSpan(t, source, "root")); err == nil {
		t.Fatal("closed owner supplied type components")
	}
}
