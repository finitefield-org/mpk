//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"go/ast"
	"os"
	"strings"
	"testing"
)

func TestPracticalCandidateMultiPackageKeeps8192OriginalRoots(t *testing.T) {
	compiler := practicalTestRegisteredCompiler(t)
	var added, second bytes.Buffer
	second.WriteString("package second\nfunc Select(x uint16) uint16 { return x + 3 }\n")
	roots := []practicalCandidateSelector{
		{Package: "example.com/c", Function: "Identity"},
		{Package: "example.com/policy", Function: "Root"},
		{Package: "example.com/policy/second", Function: "Select"},
	}
	for i := 0; i < 4094; i++ {
		name := fmt.Sprintf("Main%04d", i)
		fmt.Fprintf(&added, "func %s(x uint16) uint16 { return x + 1 }\n", name)
		roots = append(roots, practicalCandidateSelector{Package: "example.com/policy", Function: name})
	}
	for i := 0; i < 4095; i++ {
		name := fmt.Sprintf("Second%04d", i)
		fmt.Fprintf(&second, "func %s(x uint16) uint16 { return x + 2 }\n", name)
		roots = append(roots, practicalCandidateSelector{Package: "example.com/policy/second", Function: name})
	}
	if len(roots) != 8192 {
		t.Fatal("the original candidate root boundary changed")
	}
	request := practicalTestCandidateRequest(t, roots)
	extra := []practicalCandidateExtraFile{{name: "second/policy.go", data: second.Bytes()}}
	practicalCompilerInputsWithCandidateHook(t, compiler, "Root", added.Bytes(), nil, extra,
		func(session *practicalCaptureSession, discovery practicalModuleDiscovery, main practicalModuleLocation, compiler practicalCompilerImage, sdk string) {
			cache := practicalModuleLocation{capability: "workspace/dep"}
			exports, err := session.capturePracticalCandidatePackageExports(compiler, discovery, sdk, main, request)
			if err != nil {
				t.Fatal(err)
			}
			sources, err := session.sealPracticalReplaySources()
			if err != nil || len(sources.snapshot.sourceObjects()) != 5517 || len(exports.snapshot.sourceObjects()) != 182 || len(exports.packages) != 183 {
				t.Fatal("the complete original application/SDK/export closure changed", err)
			}
			original := []byte("package policy\nimport(a \"example.com/a\";c \"example.com/c\";alias \"example.com/alias\";\"net/http\")\nfunc Root(x uint16)uint16{return x+1}\nfunc Unused(r *http.Request,x int64)int64{go func(){select{}}();return a.Identity(c.Identity(alias.Identity(x)))}\nfunc init(){panic(\"original init must never execute\")}\n")
			policy, err := sources.snapshot.readSource("workspace/main", "policy.go", "go_file")
			if err != nil || !bytes.Equal(policy.sourceBytes(), append(original, added.Bytes()...)) {
				t.Fatal("the original application source or added functions changed", err)
			}
			subpackage, err := sources.snapshot.readSource("workspace/main", "second/policy.go", "go_file")
			if err != nil || !bytes.Equal(subpackage.sourceBytes(), second.Bytes()) {
				t.Fatal("the complete independent package source changed", err)
			}
			image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{sources, exports.replay}, &compiler, sdk)
			if err != nil {
				t.Fatal(err)
			}
			defer image.Close()
			for _, root := range sources.origins {
				if err := removePracticalReplayImage(root.Directory); err != nil {
					t.Fatal(err)
				}
			}
			for _, snapshot := range []practicalSourceSnapshot{sources.snapshot, exports.snapshot} {
				for _, buffer := range snapshot.buffers {
					clear(buffer.data)
				}
			}
			owner, err := image.originalCandidatePackages(main, cache, discovery, exports, sdk, request)
			if err != nil {
				t.Fatal(err)
			}
			defer owner.Close()
			candidate, err := image.candidateCompilerInputs(owner, request)
			if err != nil {
				t.Fatal(err)
			}
			defer candidate.Close()
			refs := candidate.selection.rootRefs()
			if len(refs) != len(roots) || len(candidate.compiler.files) != len(compiler.candidate.ToolchainInventory.Files) {
				t.Fatal("candidate inputs lost original roots or compiler files")
			}
			// Compare each selector with independent physical token positions in
			// the actual original AST. Each source package is scanned once.
			physical := map[string]practicalClosureRef{}
			for _, name := range []string{"example.com/c", "example.com/policy", "example.com/policy/second"} {
				for _, file := range owner.loader.packages[name].files {
					for _, decl := range file.Decls {
						f, ok := decl.(*ast.FuncDecl)
						if !ok || f.Recv != nil {
							continue
						}
						tokenFile := owner.loader.syntax.fileSet.File(f.Pos())
						key := name + "\x00" + f.Name.Name
						physical[key] = practicalClosureRef{name, practicalSourceSpan{strings.TrimSuffix(tokenFile.Name(), ".go"), tokenFile.Offset(f.Pos()), tokenFile.Offset(f.End())}}
					}
				}
			}
			for i, root := range roots {
				if expected, ok := physical[root.Package+"\x00"+root.Function]; !ok || refs[i] != expected {
					t.Fatalf("candidate root %d crossed its original function %s.%s", i, root.Package, root.Function)
				}
			}
			if _, err := owner.selectionInputs(refs[:len(refs)-1]); err == nil {
				t.Fatal("a narrowed 8192-root request was accepted")
			}
			if err := image.Close(); err != nil {
				t.Fatal(err)
			}
			graph, err := candidate.moduleGraphInputs()
			if err != nil || !bytes.Equal(graph.modules.exports.canonical, candidate.compiler.exports.canonical) {
				t.Fatal("the full candidate module graph lost its original export owner", err)
			}
			if os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE") != "" {
				retainPracticalStandardSnapshot(t, owner.inputs.snapshot)
				packet := practicalCaptureInputEvidence(t, graph.modules.exports.capture)
				packet["export_inputs_hex"] = hex.EncodeToString(graph.modules.exports.canonical)
				packet["module_observations_hex"] = hex.EncodeToString(graph.modules.canonical)
				packet["module_graph_inputs_hex"] = hex.EncodeToString(graph.canonical)
				packet["candidate_request_hex"] = hex.EncodeToString(request.requestBytes())
				retainPracticalSyntaxEvidence(t, "candidate-multi-8192-inputs.json", packet)
			}
		})
}
