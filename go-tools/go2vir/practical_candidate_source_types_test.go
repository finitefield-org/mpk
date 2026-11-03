//go:build go1.25 && (linux || darwin)

package main

import "testing"

func TestPracticalCandidateSourceTypesBindWholeOriginalPackageClosure(t *testing.T) {
	compiler := practicalTestRegisteredCompiler(t)
	additional := []byte("func OtherRoot(x uint16) uint16 { return x * 2 }\n")
	extra := []practicalCandidateExtraFile{{name: "second/policy.go", data: []byte("package second\nfunc Select(x uint16) uint16 { return x + 3 }\n")}}
	practicalCompilerInputsWithCandidateHook(t, compiler, "Root", additional, nil, extra,
		func(session *practicalCaptureSession, discovery practicalModuleDiscovery, main practicalModuleLocation, compiler practicalCompilerImage, sdk string) {
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
			if len(sources.snapshot.sourceObjects()) != 5517 || len(exports.packages) != 183 || len(exports.snapshot.sourceObjects()) != 182 {
				t.Fatal("complete original source/export closure changed")
			}
			loader, err := newPracticalSourceTypesWithStandardLibrary(sources.snapshot, discovery, compiler, sdk)
			if err != nil {
				t.Fatal(err)
			}
			if _, err := loader.typecheck("example.com/policy"); err != nil {
				t.Fatal(err)
			}
			if loader.packages["example.com/policy/second"] != nil || exports.bindCandidateOriginalSourceTypes(loader, discovery, compiler, sdk, request) == nil {
				t.Fatal("candidate binding accepted a missing selected source package")
			}
			if _, err := loader.typecheck("example.com/policy/second"); err != nil {
				t.Fatal(err)
			}
			if len(loader.packages) != 182 {
				t.Fatalf("complete typed original closure changed: %d", len(loader.packages))
			}
			if err := exports.bindCandidateOriginalSourceTypes(loader, discovery, compiler, sdk, request); err != nil {
				t.Fatal(err)
			}
			for name, functions := range map[string][]string{
				"example.com/c":             {"Identity"},
				"example.com/policy":        {"Root", "OtherRoot", "Unused"},
				"example.com/policy/second": {"Select"},
			} {
				for _, function := range functions {
					if loader.packages[name].types.Scope().Lookup(function) == nil {
						t.Fatalf("original typed function was lost: %s.%s", name, function)
					}
				}
			}
			if exports.bindOriginalSourceTypes(loader, discovery, compiler, sdk) == nil {
				t.Fatal("candidate exports were accepted by the single-package binder")
			}
			if exports.bindCandidateOriginalSourceTypes(&practicalSourceTypes{}, discovery, compiler, sdk, request) == nil {
				t.Fatal("candidate exports accepted an incomplete source loader")
			}
			other := practicalTestCandidateRequest(t, []practicalCandidateSelector{{Package: "example.com/policy", Function: "Root"}})
			if exports.bindCandidateOriginalSourceTypes(loader, discovery, compiler, sdk, other) == nil {
				t.Fatal("candidate source binding accepted a different request")
			}
			changed := exports
			changed.packages = append([]practicalCapturedPackageExport(nil), exports.packages...)
			for i := range changed.packages {
				if changed.packages[i].importPath == "example.com/policy/second" {
					changed.packages[i].sources = nil
				}
			}
			if changed.bindCandidateOriginalSourceTypes(loader, discovery, compiler, sdk, request) == nil {
				t.Fatal("candidate source binding accepted incomplete package sources")
			}
		})
}
