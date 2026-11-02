//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestPracticalStringConstantRequiresAnOriginalObservation(t *testing.T) {
	for _, owner := range []*practicalOriginalPackages{nil, {}} {
		if actual, err := owner.stringConstant("example.com/policy", practicalSourceSpan{}); err == nil || actual.source.object.ID != "" {
			t.Fatal("absent original package owner supplied a constant string", err)
		}
	}
	if carrier, err := (practicalStringConstant{}).carrier(); err == nil || carrier != (practicalStringCarrier{}) {
		t.Fatal("absent observation became a modeled empty string", err)
	}
}

// Reconstruct numeric bytes by folding all bits from the opposite direction
// to the producer. Check all 256 slots, including every padding bit.
func practicalStringTestCarrier(t *testing.T, carrier practicalStringCarrier, expected []byte) {
	t.Helper()
	var length uint64
	for bit := len(carrier.lengthWord) - 1; bit >= 0; bit-- {
		length *= 2
		if carrier.lengthWord[bit] {
			length++
		}
	}
	if length != uint64(len(expected)) {
		t.Fatalf("complete Word64 length differs: %d != %d", length, len(expected))
	}
	for slot := range carrier.byteSlots {
		var octet uint16
		for bit := len(carrier.byteSlots[slot]) - 1; bit >= 0; bit-- {
			octet *= 2
			if carrier.byteSlots[slot][bit] {
				octet++
			}
		}
		var want byte
		if slot < len(expected) {
			want = expected[slot]
		}
		if octet != uint16(want) {
			t.Fatalf("full byte/padding slot %d differs: %d != %d", slot, octet, want)
		}
	}
}

func practicalStringTestSpan(t *testing.T, object practicalSourceObject, source []byte, prefix, raw string) practicalSourceSpan {
	t.Helper()
	needle := []byte(prefix + raw)
	start := bytes.Index(source, needle)
	if start < 0 || bytes.Index(source[start+len(needle):], needle) >= 0 {
		t.Fatal("test needs one independent physical source locator", prefix, raw)
	}
	start += len(prefix)
	return practicalSourceSpan{sourceID: object.ID, start: start, end: start + len(raw)}
}

func practicalStringTestEvidence(value practicalStringConstant) map[string]any {
	return map[string]any{
		"package_path": value.packagePath, "module_source_id": value.moduleSourceID,
		"image_digest": value.imageDigest, "compiler_digest": value.compilerDigest,
		"source_object": value.source.object, "original_source": value.source.sourceBytes(),
		"span":                map[string]any{"source_id": value.span.sourceID, "start": value.span.start, "end": value.span.end},
		"original_expression": value.originalExpression(), "native_bytes": value.nativeBytes(), "native_type_display": value.nativeType,
		"ordinary_source_evaluation_or_nominal_identity_proof": false, "registered_context_or_program_proof": false,
	}
}

func TestPracticalStringConstantsOwnOriginalBytesAndCompleteFrozenCarrier(t *testing.T) {
	compiler := practicalPackageTestCompiler(t)
	roots, _ := practicalTestCaptureRoots(t)
	roots = append(roots, practicalSourceRoot{Capability: "original-go-sdk", Directory: compiler.selection.GoRoot})
	main, cache := practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}
	var alphabet []byte
	var escaped strings.Builder
	escaped.WriteByte('"')
	for octet := 0; octet < 256; octet++ {
		alphabet = append(alphabet, byte(octet))
		fmt.Fprintf(&escaped, "\\x%02x", octet)
	}
	escaped.WriteByte('"')
	cases := []struct {
		name, raw, nativeType string
		want                  []byte
	}{
		{"Empty", `""`, "untyped string", []byte{}},
		{"One", `"\xff"`, "untyped string", []byte{255}},
		{"Binary", `"\x00\xff\x80"`, "untyped string", []byte{0, 255, 128}},
		{"UTF8", `"é😀"`, "untyped string", []byte("é😀")},
		{"RawCRLF", "`left\r\nright`", "untyped string", []byte("left\nright")},
		{"AllBytes", escaped.String(), "untyped string", alphabet},
		{"Length255", `"` + strings.Repeat("b", 255) + `"`, "untyped string", bytes.Repeat([]byte{'b'}, 255)},
		{"Length257", `"` + strings.Repeat("c", 257) + `"`, "untyped string", bytes.Repeat([]byte{'c'}, 257)},
		{"Folded", `"pre" + helper.Tail`, "untyped string", []byte{'p', 'r', 'e', 0, 254}},
		{"Named", `Text("typed")`, "example.com/policy.Text", []byte("typed")},
		{"Alias", `AliasText("alias")`, "example.com/policy.AliasText", []byte("alias")},
		{"Other", `OtherText("typed")`, "example.com/policy.OtherText", []byte("typed")},
		{"Reference", `Named`, "example.com/policy.Text", []byte("typed")},
		{"Selector", `helper.Tail`, "untyped string", []byte{0, 254}},
		{"Parenthesized", `(Binary)`, "untyped string", []byte{0, 255, 128}},
		{"RuneConversion", `string(0xff)`, "string", []byte{195, 191}},
	}
	var policy strings.Builder
	policy.WriteString("package policy\r\nimport (\"example.com/policy/helper\"; \"unsafe\")\r\ntype Text string\r\ntype AliasText = Text\r\ntype OtherText string\r\n")
	for _, row := range cases {
		fmt.Fprintf(&policy, "const %s = %s\r\n", row.name, row.raw)
	}
	policy.WriteString("const Number = 42\r\nvar Dynamic = string([]byte{0, 255})\r\ntype Tagged struct { Field string `json:\"field\"` }\r\nfunc Runtime() string { return Dynamic }\r\nfunc UnusedConstant() string { return \"inside\\x00body\" }\r\nfunc Unused(c chan int, m map[int]int) int { return <-c + m[0] + int(unsafe.Sizeof(m)) }\r\nfunc init() { panic(\"application initialization must not execute\") }\r\n")
	manifest := []byte("module example.com/policy\ngo 1.25.0\n")
	policyBytes := []byte(policy.String())
	helper := []byte("package helper\nconst Tail = \"\\x00\\xfe\"\nfunc Unused(f func(int) int) int { return f(1) }\n")
	originalFiles := map[string][]byte{"go.mod": manifest, "policy.go": policyBytes, "helper/helper.go": helper}
	for name, raw := range originalFiles {
		writePracticalTestFile(t, filepath.Join(roots[0].Directory, name), raw, 0o600)
	}
	// Retain complete original inputs before executing any compiler command.
	retainPracticalSyntaxEvidence(t, "string-original-inputs.json", map[string]any{"roots": roots, "original_files": originalFiles})
	session := practicalTestSession(t, roots)
	discovery, err := session.discoverPracticalModules(main, cache, compiler.selection)
	if err != nil {
		var failure *practicalGoDiscoveryFailure
		if errors.As(err, &failure) {
			stdout, stderr := failure.originalOutput()
			retainPracticalSyntaxEvidence(t, "string-original-module-failure.json", map[string]any{"stdout": stdout, "stderr": stderr, "error": err.Error()})
		}
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "string-original-modules.json", map[string]any{"stdout": discovery.output, "stderr": discovery.observation.diagnostics, "compiler_sha256": discovery.compilerSHA256})
	if err := session.capturePracticalStandardLibrary(compiler, roots[2].Capability); err != nil {
		t.Fatal(err)
	}
	exports, err := session.capturePracticalPackageExports(compiler, discovery, roots[2].Capability, main, "example.com/policy")
	if err != nil {
		var failure *practicalGoDiscoveryFailure
		if errors.As(err, &failure) {
			exports.output, exports.diagnostics = failure.originalOutput()
		}
	}
	retainPracticalSyntaxEvidence(t, "string-original-exports.json", map[string]any{"stdout": exports.output, "stderr": exports.diagnostics, "error": fmt.Sprint(err)})
	if err != nil {
		t.Fatal(err)
	}
	sealed, err := session.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	// Full byte preimages are archived before their original roots and donor
	// buffers retire; later replays cannot stand in for these observations.
	retainPracticalStandardSnapshot(t, sealed.snapshot)
	retainPracticalStandardSnapshot(t, exports.snapshot)
	image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{sealed, exports.replay}, &compiler, roots[2].Capability)
	if err != nil {
		t.Fatal(err)
	}
	defer image.Close()
	for _, root := range roots {
		if err := removePracticalReplayImage(root.Directory); err != nil {
			t.Fatal(err)
		}
		if _, err := os.Lstat(root.Directory); !os.IsNotExist(err) {
			t.Fatal("original preparation path survived retirement", err)
		}
	}
	for _, snapshot := range []practicalSourceSnapshot{sealed.snapshot, exports.snapshot} {
		for _, buffer := range snapshot.buffers {
			clear(buffer.data)
		}
	}
	owner, err := image.originalPackages(main, cache, discovery, exports, roots[2].Capability)
	if err != nil {
		t.Fatal(err)
	}
	defer owner.Close()
	if err := image.Close(); err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "string-owned-universe.json", map[string]any{
		"module_stdout": owner.moduleOutput, "module_stderr": owner.moduleDiagnostics,
		"package_stdout": owner.packageOutput, "package_stderr": owner.packageDiagnostics,
		"image_digest": owner.imageDigest, "compiler_digest": owner.compilerDigest,
		"complete_source_typed_packages": len(owner.loader.packages), "complete_syntax_nodes": owner.loader.syntax.count,
		"original_source_objects": len(owner.inputs.snapshot.ids), "preparation_roots_and_image_retired": true,
		"registered_context_or_source_program_proof": false,
	})
	inventory, err := owner.inventory()
	if err != nil || len(inventory) != 3 {
		t.Fatal("original complete package universe changed", err)
	}
	object, helperObject := inventory[0].sources[0], inventory[1].sources[0]
	var retained []practicalStringConstant
	for _, row := range cases {
		t.Run(row.name, func(t *testing.T) {
			span := practicalStringTestSpan(t, object, policyBytes, "const "+row.name+" = ", row.raw)
			value, err := owner.stringConstant("example.com/policy", span)
			if err != nil {
				t.Fatal(err)
			}
			// Save the actual native observation before testing its modeled form.
			record := practicalStringTestEvidence(value)
			retainPracticalSyntaxEvidence(t, "string-"+row.name+"-native.json", record)
			if value.span != span || value.packagePath != "example.com/policy" || value.moduleSourceID != discovery.mainSourceID ||
				value.imageDigest != owner.imageDigest || value.compilerDigest != owner.compilerDigest || value.nativeType != row.nativeType ||
				!bytes.Equal(value.source.data, policyBytes) || !bytes.Equal(value.expression, []byte(row.raw)) || !bytes.Equal(value.value, row.want) {
				t.Fatal("original native value/type/source binding changed", value.nativeType)
			}
			carrier, failure := value.carrier()
			record["carrier_error"] = fmt.Sprint(failure)
			record["length_word"] = carrier.lengthWord
			record["byte_slots"] = carrier.byteSlots
			retainPracticalSyntaxEvidence(t, "string-"+row.name+"-carrier.json", record)
			if len(row.want) > 256 {
				var domain *practicalStringDomainError
				if !errors.As(failure, &domain) || domain.bytes != 257 || carrier != (practicalStringCarrier{}) || owner.inputs.failed != nil {
					t.Fatal("modeled-domain overflow lost its distinct full-value failure", failure)
				}
			} else {
				if failure != nil {
					t.Fatal(failure)
				}
				practicalStringTestCarrier(t, carrier, row.want)
				carrier.lengthWord[63] = true
				carrier.byteSlots[255][7] = !carrier.byteSlots[255][7]
				again, err := value.carrier()
				if err != nil {
					t.Fatal(err)
				}
				practicalStringTestCarrier(t, again, row.want)
			}
			clear(value.nativeBytes())
			clear(value.originalExpression())
			copied := value.originalSource()
			clear(copied.data)
			copied.object.ID = "caller-mutated-view"
			again, err := owner.stringConstant("example.com/policy", span)
			if err != nil || !bytes.Equal(again.nativeBytes(), row.want) || !bytes.Equal(value.nativeBytes(), row.want) ||
				!bytes.Equal(value.originalExpression(), []byte(row.raw)) || !bytes.Equal(value.originalSource().data, policyBytes) {
				t.Fatal("returned bytes/model changed the original owner or observation", err)
			}
			retained = append(retained, value)
		})
	}
	// Constants in full unused original bodies remain accessible without
	// executing initializers or replacing those bodies with signatures.
	span := practicalStringTestSpan(t, object, policyBytes, "func UnusedConstant() string { return ", `"inside\x00body"`)
	inside, err := owner.stringConstant("example.com/policy", span)
	if err != nil || !bytes.Equal(inside.nativeBytes(), []byte("inside\x00body")) || inside.nativeType != "string" {
		t.Fatal("unused complete-body constant changed", err)
	}
	retained = append(retained, inside)
	retainPracticalSyntaxEvidence(t, "string-unused-body-native.json", practicalStringTestEvidence(inside))
	valid := practicalStringTestSpan(t, object, policyBytes, "const Binary = ", `"\x00\xff\x80"`)
	for _, mutation := range []string{"source-metadata", "whole-source-byte", "expression-byte", "span", "source-role", "package", "image", "native-type"} {
		t.Run("damaged-"+mutation, func(t *testing.T) {
			candidate, err := owner.stringConstant("example.com/policy", valid)
			if err != nil {
				t.Fatal(err)
			}
			switch mutation {
			case "source-metadata":
				candidate.source.object.ID = strings.Repeat("f", 64)
			case "whole-source-byte":
				candidate.source.data[0] ^= 1 // outside the observed expression
			case "expression-byte":
				candidate.expression[0] ^= 1
			case "span":
				candidate.span.end--
			case "source-role":
				candidate.source.object.Kind = "auxiliary"
			case "package":
				candidate.packagePath = ""
			case "image":
				candidate.imageDigest = ""
			case "native-type":
				candidate.nativeType = ""
			}
			// Preserve the whole actual damaged input before conversion.
			record := practicalStringTestEvidence(candidate)
			retainPracticalSyntaxEvidence(t, "string-damaged-"+mutation+"-input.json", record)
			carrier, failure := candidate.carrier()
			var domain *practicalStringDomainError
			if failure == nil || errors.As(failure, &domain) || carrier != (practicalStringCarrier{}) {
				t.Fatal("damaged native observation became a value/domain failure", failure)
			}
			record["error"] = fmt.Sprint(failure)
			retainPracticalSyntaxEvidence(t, "string-damaged-"+mutation+"-result.json", record)
			again, err := owner.stringConstant("example.com/policy", valid)
			if err != nil || !bytes.Equal(again.originalSource().data, policyBytes) || !bytes.Equal(again.nativeBytes(), []byte{0, 255, 128}) {
				t.Fatal("owned observation mutated its original package owner", err)
			}
		})
	}
	controls := []struct {
		name, pkg string
		span      practicalSourceSpan
	}{
		{"unknown-package", "example.com/absent", valid},
		{"other-package", "example.com/policy/helper", valid},
		{"intrinsic-package", "unsafe", valid},
		{"other-source", "example.com/policy", practicalSourceSpan{sourceID: helperObject.ID, start: valid.start, end: valid.end}},
		{"unknown-source", "example.com/policy", practicalSourceSpan{sourceID: strings.Repeat("f", 64), start: valid.start, end: valid.end}},
		{"negative-start", "example.com/policy", practicalSourceSpan{sourceID: object.ID, start: -1, end: valid.end}},
		{"empty-span", "example.com/policy", practicalSourceSpan{sourceID: object.ID, start: valid.start, end: valid.start}},
		{"truncated-expression", "example.com/policy", practicalSourceSpan{sourceID: object.ID, start: valid.start, end: valid.end - 1}},
		{"extended-expression", "example.com/policy", practicalSourceSpan{sourceID: object.ID, start: valid.start, end: valid.end + 1}},
		{"beyond-source", "example.com/policy", practicalSourceSpan{sourceID: object.ID, start: len(policyBytes), end: len(policyBytes) + 1}},
		{"import-path", "example.com/policy", practicalStringTestSpan(t, object, policyBytes, "import (", `"example.com/policy/helper"`)},
		{"struct-tag", "example.com/policy", practicalStringTestSpan(t, object, policyBytes, "Field string ", "`json:\"field\"`")},
		{"integer-constant", "example.com/policy", practicalStringTestSpan(t, object, policyBytes, "const Number = ", "42")},
		{"runtime-conversion", "example.com/policy", practicalStringTestSpan(t, object, policyBytes, "var Dynamic = ", "string([]byte{0, 255})")},
		{"runtime-variable", "example.com/policy", practicalStringTestSpan(t, object, policyBytes, "func Runtime() string { return ", "Dynamic")},
	}
	for _, control := range controls {
		t.Run(control.name, func(t *testing.T) {
			actual, failure := owner.stringConstant(control.pkg, control.span)
			retainPracticalSyntaxEvidence(t, "string-reject-"+control.name+".json", map[string]any{
				"package_path": control.pkg, "source_id": control.span.sourceID, "start": control.span.start, "end": control.span.end,
				"error": fmt.Sprint(failure), "semantic_source_counterexample": false,
			})
			var domain *practicalStringDomainError
			if failure == nil || errors.As(failure, &domain) || actual.source.object.ID != "" || owner.inputs.failed != nil {
				t.Fatal("invalid observation became a value/domain/transaction failure", failure)
			}
			if _, err := owner.stringConstant("example.com/policy", valid); err != nil {
				t.Fatal("ordinary query error retired an intact owner", err)
			}
		})
	}
	// Every preexisting sticky layer prevents later native observations.
	sentinel := errors.New("original byte/type/syntax transaction failed")
	for _, layer := range []struct {
		name string
		at   *error
	}{{"input", &owner.inputs.failed}, {"types", &owner.loader.failed}, {"syntax", &owner.loader.syntax.failed}} {
		t.Run(layer.name+"-failed", func(t *testing.T) {
			*layer.at = sentinel
			_, failure := owner.stringConstant("example.com/policy", valid)
			*layer.at = nil
			if failure != sentinel {
				t.Fatal("failed original owner still supplied native observations", failure)
			}
		})
	}
	bad := object
	bad.ID = strings.Repeat("f", 64)
	_, failed := owner.originalBytes(bad)
	if failed == nil {
		t.Fatal("crossed original source identity accepted")
	}
	if _, err := owner.stringConstant("example.com/policy", valid); err != failed {
		t.Fatal("actual failed original byte transaction still supplied constants", err)
	}
	owner.Close()
	owner.Close()
	if _, err := owner.stringConstant("example.com/policy", valid); err == nil {
		t.Fatal("closed owner still supplied a constant")
	}
	for _, value := range retained {
		if _, err := value.originalSource().canonicalObject(); err != nil || !bytes.Equal(value.originalSource().data, policyBytes) {
			t.Fatal("returned observation lost its whole original source after retirement", err)
		}
		carrier, err := value.carrier()
		if len(value.nativeBytes()) > 256 {
			var domain *practicalStringDomainError
			if !errors.As(err, &domain) || domain.bytes != len(value.nativeBytes()) {
				t.Fatal("retired oversized observation lost its original domain failure", err)
			}
		} else {
			if err != nil {
				t.Fatal(err)
			}
			practicalStringTestCarrier(t, carrier, value.nativeBytes())
		}
	}
}
