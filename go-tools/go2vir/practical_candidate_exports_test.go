//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"fmt"
	"reflect"
	"sort"
	"testing"
)

func practicalTestCandidateRequest(t *testing.T, roots []practicalCandidateSelector) *practicalCandidateRequest {
	t.Helper()
	sort.Slice(roots, func(i, j int) bool {
		left, leftErr := canonicalJSON(roots[i])
		right, rightErr := canonicalJSON(roots[j])
		if leftErr != nil || rightErr != nil {
			t.Fatal("candidate selector encoding", leftErr, rightErr)
		}
		return bytes.Compare(left, right) < 0
	})
	request, err := newPracticalCandidateRequest(roots)
	if err != nil {
		t.Fatal(err)
	}
	return request
}

func TestPracticalCandidatePackagePathsPreserveAllRoots(t *testing.T) {
	roots := make([]practicalCandidateSelector, 8192)
	for i := range roots {
		roots[i] = practicalCandidateSelector{Package: fmt.Sprintf("example.com/p%d", i%3), Function: fmt.Sprintf("F%04d", i)}
	}
	request := practicalTestCandidateRequest(t, roots)
	got, err := practicalCandidatePackagePaths(request)
	if err != nil || !reflect.DeepEqual(got, []string{"example.com/p0", "example.com/p1", "example.com/p2"}) {
		t.Fatalf("complete candidate packages: %v %v", got, err)
	}
	for i := range roots {
		roots[i] = practicalCandidateSelector{Package: fmt.Sprintf("example.com/p%04d", i), Function: "F"}
	}
	distinct := practicalTestCandidateRequest(t, roots)
	packages, err := practicalCandidatePackagePaths(distinct)
	if err != nil || len(packages) != len(roots) {
		t.Fatalf("distinct candidate packages: count=%d error=%v", len(packages), err)
	}
	for i, name := range packages {
		if name != fmt.Sprintf("example.com/p%04d", i) {
			t.Fatalf("candidate package %d changed: %q", i, name)
		}
	}
	changed := *request
	changed.request = bytes.Clone(request.request)
	changed.request[0] ^= 1
	if _, err := practicalCandidatePackagePaths(&changed); err == nil {
		t.Fatal("modified complete request was accepted")
	}
	if _, err := practicalCandidatePackagePaths(nil); err == nil {
		t.Fatal("missing request was accepted")
	}
	for _, name := range []string{"example.com/p...q", "all", "std", "cmd"} {
		pattern := practicalTestCandidateRequest(t, []practicalCandidateSelector{{Package: name, Function: "F"}})
		if _, err := practicalCandidatePackagePaths(pattern); err == nil {
			t.Fatalf("Go list pattern %q was treated as an exact package", name)
		}
	}
}

func TestPracticalCandidatePackageExportsKeepAllOriginalPackages(t *testing.T) {
	compiler := practicalTestRegisteredCompiler(t)
	additional := []byte("func OtherRoot(x uint16) uint16 { return x * 2 }\n")
	extra := []practicalCandidateExtraFile{{name: "second/policy.go", data: []byte("package second\nfunc Select(x uint16) uint16 { return x + 3 }\n")}}
	practicalCompilerInputsWithCandidateHook(t, compiler, "Root", additional, nil, extra,
		func(session *practicalCaptureSession, discovery practicalModuleDiscovery, main practicalModuleLocation, compiler practicalCompilerImage, sdk string) {
			baseline, err := session.capturePracticalPackageExports(compiler, discovery, sdk, main, "example.com/policy")
			if err != nil || len(baseline.packages) != 182 || len(baseline.snapshot.sourceObjects()) != 181 {
				t.Fatalf("complete original single-package baseline changed: packages=%d exports=%d error=%v", len(baseline.packages), len(baseline.snapshot.sourceObjects()), err)
			}
			baselineSources, err := copyPracticalSourceSnapshot(session)
			if err != nil || len(baselineSources.sourceObjects()) != 5516 {
				t.Fatalf("complete original source baseline changed: objects=%d error=%v", len(baselineSources.sourceObjects()), err)
			}
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
			if err := exports.requireCandidateOriginalObservation(request); err != nil {
				t.Fatal(err)
			}
			if exports.requireOriginalObservation() == nil {
				t.Fatal("multi-package observation was accepted as a single-package observation")
			}
			if !reflect.DeepEqual(exports.requestedPackages, []string{"example.com/c", "example.com/policy", "example.com/policy/second"}) || len(exports.packages) != 183 || len(exports.snapshot.sourceObjects()) != 182 {
				t.Fatalf("original package/export closure changed: packages=%d exports=%d roots=%v", len(exports.packages), len(exports.snapshot.sourceObjects()), exports.requestedPackages)
			}
			if exports.snapshot.contentSHA256 != practicalSnapshotDigest(exports.snapshot) || exports.replay.sealed == nil {
				t.Fatal("complete original export snapshot was not sealed")
			}
			sources, err := session.sealPracticalReplaySources()
			if err != nil || len(sources.snapshot.sourceObjects()) != 5517 || sources.snapshot.contentSHA256 != practicalSnapshotDigest(sources.snapshot) {
				t.Fatalf("complete original source snapshot changed: objects=%d error=%v", len(sources.snapshot.sourceObjects()), err)
			}
			for _, original := range baselineSources.sourceObjects() {
				before, exists := baselineSources.sourceBytes(original.ID)
				after, err := sources.snapshot.readSource(original.Capability, original.Path, original.Kind)
				if !exists || err != nil || after.object != original || !bytes.Equal(before, after.sourceBytes()) {
					t.Fatalf("original source bytes changed at %s/%s: %v", original.Capability, original.Path, err)
				}
			}
			candidatePackages := make(map[string]practicalCapturedPackageExport, len(exports.packages))
			for _, candidate := range exports.packages {
				candidatePackages[candidate.importPath] = candidate
			}
			for _, original := range baseline.packages {
				candidate, exists := candidatePackages[original.importPath]
				if !exists || original.intrinsic != candidate.intrinsic || (original.data == nil) != (candidate.data == nil) {
					t.Fatalf("original package disappeared: %s", original.importPath)
				}
				if original.data != nil {
					before, beforeOK := baseline.snapshot.sourceBytes(original.data.ID)
					after, afterOK := exports.snapshot.sourceBytes(candidate.data.ID)
					if !beforeOK || !afterOK || !bytes.Equal(before, after) {
						t.Fatalf("original compiler export bytes changed: %s", original.importPath)
					}
				}
			}
			mainSource, err := sources.snapshot.readSource("workspace/main", "policy.go", "go_file")
			original := []byte("package policy\nimport(a \"example.com/a\";c \"example.com/c\";alias \"example.com/alias\";\"net/http\")\nfunc Root(x uint16)uint16{return x+1}\nfunc Unused(r *http.Request,x int64)int64{go func(){select{}}();return a.Identity(c.Identity(alias.Identity(x)))}\nfunc init(){panic(\"original init must never execute\")}\n")
			if err != nil || !bytes.Equal(mainSource.sourceBytes(), append(original, additional...)) {
				t.Fatal("the complete original application source was not retained", err)
			}
			second, err := sources.snapshot.readSource("workspace/main", extra[0].name, "go_file")
			if err != nil || !bytes.Equal(second.sourceBytes(), extra[0].data) {
				t.Fatal("the independently selected original package was not retained", err)
			}
			rows, err := decodePracticalPackageExports(exports.output)
			if err != nil || validatePracticalExportRoots(rows, exports.requestedPackages) != nil {
				t.Fatal("original multi-package Go metadata did not match its request", err)
			}
			other := practicalTestCandidateRequest(t, []practicalCandidateSelector{{Package: "example.com/policy", Function: "Root"}})
			if exports.requireCandidateOriginalObservation(other) == nil {
				t.Fatal("a different complete request was accepted")
			}
			changed := exports
			changed.output = bytes.Clone(exports.output)
			changed.output[0] ^= 1
			if changed.requireCandidateOriginalObservation(request) == nil {
				t.Fatal("modified native output was accepted")
			}
			changed = exports
			changed.requestedPackages = append([]string(nil), exports.requestedPackages[:1]...)
			if changed.requireCandidateOriginalObservation(request) == nil {
				t.Fatal("a narrowed package set was accepted")
			}
			changed = exports
			observation := *exports.observation
			observation.arguments = append([]string(nil), observation.arguments...)
			observation.arguments[len(observation.arguments)-1] = "example.com/policy"
			changed.observation = &observation
			if changed.requireCandidateOriginalObservation(request) == nil {
				t.Fatal("modified native command arguments were accepted")
			}
			for i := range rows {
				if rows[i].ImportPath == "example.com/policy/second" {
					rows[i].DepOnly = true
					break
				}
			}
			if validatePracticalExportRoots(rows, exports.requestedPackages) == nil {
				t.Fatal("a selected package relabeled as a dependency was accepted")
			}
		})
}
