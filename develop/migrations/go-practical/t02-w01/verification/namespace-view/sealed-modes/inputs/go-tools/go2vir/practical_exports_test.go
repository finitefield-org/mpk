package main

import (
	"testing"
)

func TestPracticalExportMetadataRejectsAmbiguousFailedOrGeneratedPackages(t *testing.T) {
	valid := `{"ImportPath":"example.com/app","Name":"policy","Match":["example.com/app"]}`
	rows, err := decodePracticalPackageExports([]byte(valid))
	if err != nil || len(rows) != 1 {
		t.Fatal("original native package metadata did not decode", err)
	}
	for _, raw := range []string{
		"", "[]", valid + valid,
		`{"ImportPath":"example.com/app","ImportPath":"other","Name":"policy"}`,
		`{"ImportPath":"../other","Name":"policy"}`,
		`{"ImportPath":"example.com/app","Name":"policy","ForTest":"example.com/app"}`,
		`{"ImportPath":"example.com/app","Name":"policy","Incomplete":true}`,
		`{"ImportPath":"example.com/app","Name":"policy","BinaryOnly":true}`,
		`{"ImportPath":"example.com/app","Name":"policy","Error":{"Err":"failed"}}`,
		`{"ImportPath":"example.com/app","Name":"policy","DepsErrors":[{"Err":"failed"}]}`,
		`{"ImportPath":"example.com/app","Name":"policy","CgoFiles":["generated.go"]}`,
	} {
		t.Run(raw, func(t *testing.T) {
			if _, err := decodePracticalPackageExports([]byte(raw)); err == nil {
				t.Fatal("ambiguous, missing or generated package acquired original export metadata")
			}
		})
	}
}

func TestPracticalExportRootRetainsActualCommandSelection(t *testing.T) {
	rows := []practicalGoPackageExport{
		{ImportPath: "fmt", DepOnly: true},
		{ImportPath: "example.com/app", Match: []string{"example.com/app"}},
	}
	if err := validatePracticalExportRoot(rows, "example.com/app"); err != nil {
		t.Fatal(err)
	}
	if err := validatePracticalExportRoot(rows, "fmt"); err == nil {
		t.Fatal("original dependency was relabeled as the command's selected application")
	}
	rows[0].DepOnly = false
	if err := validatePracticalExportRoot(rows, "example.com/app"); err == nil {
		t.Fatal("extra selected root acquired original command authority")
	}
}
