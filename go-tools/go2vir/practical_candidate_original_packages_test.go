//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"os"
	"reflect"
	"slices"
	"strings"
	"testing"
)

func TestPracticalCandidateOriginalPackagesOwnWholeImage(t *testing.T) {
	compiler := practicalTestRegisteredCompiler(t)
	additional := []byte("func OtherRoot(x uint16) uint16 { return x * 2 }\n")
	extra := []practicalCandidateExtraFile{{name: "second/policy.go", data: []byte("package second\nfunc Select(x uint16) uint16 { return x + 3 }\n")}}
	practicalCompilerInputsWithCandidateHook(t, compiler, "Root", additional, nil, extra,
		func(session *practicalCaptureSession, discovery practicalModuleDiscovery, main practicalModuleLocation, compiler practicalCompilerImage, sdk string) {
			cache := practicalModuleLocation{capability: "workspace/dep"}
			request := practicalTestCandidateRequest(t, []practicalCandidateSelector{
				{Package: "example.com/c", Function: "Identity"},
				{Package: "example.com/policy", Function: "Root"},
				{Package: "example.com/policy", Function: "OtherRoot"},
				{Package: "example.com/policy/second", Function: "Select"},
			})
			exports, err := session.capturePracticalCandidatePackageExports(compiler, discovery, sdk, main, request)
			if err != nil {
				t.Fatal(err)
			}
			sources, err := session.sealPracticalReplaySources()
			if err != nil {
				t.Fatal(err)
			}
			objects := append(sources.snapshot.sourceObjects(), exports.snapshot.sourceObjects()...)
			if len(sources.snapshot.sourceObjects()) != 5517 || len(exports.snapshot.sourceObjects()) != 182 || len(exports.packages) != 183 {
				t.Fatal("complete original candidate inputs changed")
			}
			image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{sources, exports.replay}, &compiler, sdk)
			if err != nil {
				t.Fatal(err)
			}
			defer image.Close()
			// Only the test's original temporary copies are retired. The owner
			// must reconstruct from the sealed image, not those paths or buffers.
			for _, root := range sources.origins {
				if err := removePracticalReplayImage(root.Directory); err != nil {
					t.Fatal(err)
				}
				if _, err := os.Lstat(root.Directory); !os.IsNotExist(err) {
					t.Fatal("original preparation root survived retirement", err)
				}
			}
			for _, snapshot := range []practicalSourceSnapshot{sources.snapshot, exports.snapshot} {
				for _, buffer := range snapshot.buffers {
					clear(buffer.data)
				}
			}
			other := practicalTestCandidateRequest(t, []practicalCandidateSelector{{Package: "example.com/policy", Function: "Root"}})
			for _, wrong := range []*practicalCandidateRequest{nil, other} {
				if owner, err := image.originalCandidatePackages(main, cache, discovery, exports, sdk, wrong); err == nil || owner != nil {
					t.Fatal("candidate image accepted a missing or different complete request", err)
				}
			}
			if owner, err := image.originalPackages(main, cache, discovery, exports, sdk); err == nil || owner != nil {
				t.Fatal("legacy image owner accepted candidate exports", err)
			}
			changed := exports
			changed.snapshot.contentSHA256 = strings.Repeat("a", 64)
			if owner, err := image.originalCandidatePackages(main, cache, discovery, changed, sdk, request); err == nil || owner != nil || image.activeViews != 0 {
				t.Fatal("candidate image accepted a crossed export snapshot or leaked its view", err)
			}
			owner, err := image.originalCandidatePackages(main, cache, discovery, exports, sdk, request)
			if err != nil {
				t.Fatal(err)
			}
			defer owner.Close()
			if image.activeViews != 0 || owner.requestedPackage != "" || !reflect.DeepEqual(owner.requestedPackages, []string{"example.com/c", "example.com/policy", "example.com/policy/second"}) || !bytes.Equal(owner.candidateRequest, request.requestBytes()) {
				t.Fatal("candidate owner lost the complete request or retained its image view")
			}
			source, err := owner.sourceInputs()
			if err != nil || source.requestedPackage != "" || !reflect.DeepEqual(source.requestedPackages, owner.requestedPackages) || source.candidateRequestSHA256 == "" {
				t.Fatal("candidate source inputs lost complete request provenance", err)
			}
			if _, err := owner.candidateSourceSelection(other); err == nil || !strings.Contains(err.Error(), "different complete request") {
				t.Fatal("candidate owner accepted another request", err)
			}
			candidate, err := image.candidateCompilerInputs(owner, request)
			if err != nil {
				t.Fatal(err)
			}
			defer candidate.Close()
			refs := candidate.selection.rootRefs()
			if len(refs) != 4 {
				t.Fatalf("candidate source selection lost roots: %d", len(refs))
			}
			for _, ref := range []practicalClosureRef{
				practicalClosureTestRef(t, owner, "example.com/c", "Identity"),
				practicalClosureTestRef(t, owner, "example.com/policy", "Root"),
				practicalClosureTestRef(t, owner, "example.com/policy", "OtherRoot"),
				practicalClosureTestRef(t, owner, "example.com/policy/second", "Select"),
			} {
				if !slices.Contains(refs, ref) {
					t.Fatal("candidate source selection lost an original function")
				}
			}
			if _, err := owner.selectionInputs(refs[:3]); err == nil {
				t.Fatal("candidate owner accepted a narrowed physical selection")
			}
			changedRoots := append([]practicalClosureRef(nil), refs...)
			changedRoots[0] = practicalClosureTestRef(t, owner, "example.com/policy", "Unused")
			if _, err := owner.selectionInputs(changedRoots); err == nil {
				t.Fatal("candidate owner accepted a different physical root")
			}
			if err := image.Close(); err != nil {
				t.Fatal(err)
			}
			graph, err := candidate.moduleGraphInputs()
			if err != nil || !bytes.Equal(graph.modules.exports.canonical, candidate.compiler.exports.canonical) {
				t.Fatal("candidate graph lost the complete export owner", err)
			}
			if os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE") != "" {
				retainPracticalStandardSnapshot(t, owner.inputs.snapshot)
				packet := practicalCaptureInputEvidence(t, graph.modules.exports.capture)
				packet["export_inputs_hex"] = hex.EncodeToString(graph.modules.exports.canonical)
				packet["module_observations_hex"] = hex.EncodeToString(graph.modules.canonical)
				packet["module_graph_inputs_hex"] = hex.EncodeToString(graph.canonical)
				packet["candidate_request_hex"] = hex.EncodeToString(request.requestBytes())
				retainPracticalSyntaxEvidence(t, "candidate-multi-inputs.json", packet)
			}
			inventory, err := owner.inventory()
			if err != nil || len(inventory) != 183 || len(owner.loader.packages) != 182 {
				t.Fatal("candidate owner lost the original typed package closure", err)
			}
			for name, functions := range map[string][]string{
				"example.com/c":             {"Identity"},
				"example.com/policy":        {"Root", "OtherRoot", "Unused"},
				"example.com/policy/second": {"Select"},
			} {
				for _, function := range functions {
					if owner.loader.packages[name].types.Scope().Lookup(function) == nil {
						t.Fatalf("owned original function was lost: %s.%s", name, function)
					}
				}
			}
			for _, object := range objects {
				buffer, err := owner.originalBytes(object)
				if err != nil || buffer.object != object {
					t.Fatalf("owned original object changed: %s/%s: %v", object.Capability, object.Path, err)
				}
				if _, err := buffer.canonicalObject(); err != nil {
					t.Fatalf("owned original object bytes changed: %s/%s: %v", object.Capability, object.Path, err)
				}
			}
			originalRequest, originalOutput := bytes.Clone(owner.candidateRequest), bytes.Clone(owner.packageOutput)
			clear(request.request)
			clear(exports.output)
			if !bytes.Equal(owner.candidateRequest, originalRequest) || !bytes.Equal(owner.packageOutput, originalOutput) {
				t.Fatal("caller mutation changed the owned request or native output")
			}
			owner.Close()
			if _, err := owner.inventory(); err == nil || len(owner.candidateRequest) != 0 || len(owner.requestedPackages) != 0 {
				t.Fatal("closed candidate owner retained request or package authority")
			}
		})
}
