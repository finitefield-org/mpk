package main

import "testing"

func TestGoProfileV0VectorsAreExhaustivelyAccountedFor(t *testing.T) {
	vectors := loadStrictObjectFile(t, repoPath("develop/specs/vectors/go-vir-profile-v0.json"))
	if stringField(t, vectors, "schema") != "mpk.go.vir_profile.conformance.v0" || stringField(t, vectors, "spec_profile") != goSemanticProfile {
		t.Fatal("Go profile vector identity drifted")
	}
	owners := stringArrayField(t, vectors, "owner_tests")
	for _, wanted := range []string{"go-tools/go2vir/profile_v0_test.go", "go-tools/go2vir/profile_v0_vectors_test.go", "crates/mpk-vc/tests/go_profile_vectors.rs"} {
		if !containsString(owners, wanted) {
			t.Fatalf("Go profile vector lacks owner %s", wanted)
		}
	}
	groups := []string{"profile_cases", "capture_cases", "source_cases", "operation_cases", "contract_cases", "loop_call_cases", "diagnostic_cases", "limit_cases"}
	wantCounts := map[string]int{"profile_cases": 6, "capture_cases": 27, "source_cases": 22, "operation_cases": 17, "contract_cases": 21, "loop_call_cases": 10, "diagnostic_cases": 7, "limit_cases": 20}
	visited := make(map[string]bool)
	for _, group := range groups {
		cases := arrayField(t, vectors, group)
		if len(cases) != wantCounts[group] {
			t.Fatalf("%s count = %d, want %d", group, len(cases), wantCounts[group])
		}
		for _, raw := range cases {
			value := asObject(t, raw, group)
			id := stringField(t, value, "id")
			if visited[id] {
				t.Fatalf("duplicate Go profile vector ID %s", id)
			}
			visited[id] = true
			expect := objectField(t, value, "expect")
			outcome := stringField(t, expect, "outcome")
			if outcome == "" {
				t.Fatalf("%s has no outcome", id)
			}
			if outcome != "accepted" {
				if code, exists := optionalString(expect, "code"); exists && code == "" {
					t.Fatalf("%s has an empty rejection code", id)
				}
			}
			if _, sourceCase := value["source"]; sourceCase && group != "source_cases" && group != "operation_cases" {
				t.Fatalf("%s places source text in the wrong group", id)
			}
		}
	}
	if len(visited) != 130 {
		t.Fatalf("visited %d Go profile vectors, want 130", len(visited))
	}
}
