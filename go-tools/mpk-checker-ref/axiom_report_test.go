package mpkcheckerref

import (
	"os"
	"path/filepath"
	"testing"
)

func TestBuildAxiomReportFixtureMatchesRust(t *testing.T) {
	report, err := BuildAxiomReport(axiomReportFixtureCertificate())
	if err != nil {
		t.Fatalf("BuildAxiomReport() error = %v", err)
	}

	actual := renderAxiomReportFixture(report)
	expected, err := os.ReadFile(filepath.Join(repoRoot(), "fixtures/cert-axiom-report/basic-report.txt"))
	if err != nil {
		t.Fatalf("read expected report: %v", err)
	}
	if actual != string(expected) {
		t.Fatalf("report fixture mismatch\nactual:\n%s\nexpected:\n%s", actual, string(expected))
	}
}

func TestBuildAxiomReportDependencyVerdictsMatchRust(t *testing.T) {
	report, err := BuildAxiomReport(axiomReportFixtureCertificate())
	if err != nil {
		t.Fatalf("BuildAxiomReport() error = %v", err)
	}

	coreAxiom := findAxiomReportEntry(t, report, "Example.AxiomReport.ax")
	if !uint32SlicesEqual(coreAxiom.DirectDependentDeclarations, []uint32{0, 1}) {
		t.Fatalf("core axiom direct dependents = %v, want [0 1]", coreAxiom.DirectDependentDeclarations)
	}
	if !uint32SlicesEqual(coreAxiom.TransitiveDependentDeclarations, []uint32{0, 1, 2}) {
		t.Fatalf("core axiom transitive dependents = %v, want [0 1 2]", coreAxiom.TransitiveDependentDeclarations)
	}

	theoremDependencies := findDeclarationAxiomDependencies(t, report, "Example.AxiomReport.thm")
	if len(theoremDependencies.DirectAxiomDependencies) != 0 {
		t.Fatalf("theorem direct axiom dependencies = %v, want []", theoremDependencies.DirectAxiomDependencies)
	}
	if len(theoremDependencies.TransitiveAxiomDependencies) != 1 {
		t.Fatalf("theorem transitive axiom dependencies = %v, want one entry", theoremDependencies.TransitiveAxiomDependencies)
	}
}

func TestBuildAxiomReportTheoryPrimitiveVerdictMatchesRust(t *testing.T) {
	report, err := BuildAxiomReport(axiomReportFixtureCertificate())
	if err != nil {
		t.Fatalf("BuildAxiomReport() error = %v", err)
	}

	theory := findAxiomReportEntry(t, report, "Example.AxiomReport.theory")
	if theory.Category != AxiomCategoryBuiltinTheory {
		t.Fatalf("theory category = %s, want %s", theory.Category, AxiomCategoryBuiltinTheory)
	}
	if !uint32SlicesEqual(theory.DirectDependentDeclarations, []uint32{3, 4}) {
		t.Fatalf("theory direct dependents = %v, want [3 4]", theory.DirectDependentDeclarations)
	}
	if !uint32SlicesEqual(theory.TransitiveDependentDeclarations, []uint32{3, 4}) {
		t.Fatalf("theory transitive dependents = %v, want [3 4]", theory.TransitiveDependentDeclarations)
	}
}

func TestCheckAxiomReportVerifiesEmbeddedReportAndHash(t *testing.T) {
	cert := axiomReportFixtureCertificate()
	report, err := BuildAxiomReport(cert)
	if err != nil {
		t.Fatalf("BuildAxiomReport() error = %v", err)
	}
	cert.AxiomReport = report
	cert.Hashes.AxiomReportHash = AxiomReportHash(report)

	checked, err := CheckAxiomReport(cert)
	if err != nil {
		t.Fatalf("CheckAxiomReport() error = %v", err)
	}
	if !axiomReportsEqual(checked, report) {
		t.Fatalf("checked report does not match rebuilt report")
	}
}

func TestCheckAxiomReportRejectsMismatches(t *testing.T) {
	t.Run("report", func(t *testing.T) {
		cert := axiomReportFixtureCertificate()
		report, err := BuildAxiomReport(cert)
		if err != nil {
			t.Fatalf("BuildAxiomReport() error = %v", err)
		}
		cert.AxiomReport = report
		cert.AxiomReport.Summary.TotalAxiomCount++
		cert.Hashes.AxiomReportHash = AxiomReportHash(cert.AxiomReport)

		reportErr := checkAxiomReportError(t, cert)
		if reportErr.Kind != AxiomReportMismatch {
			t.Fatalf("error kind = %s, want %s", reportErr.Kind, AxiomReportMismatch)
		}
	})

	t.Run("hash", func(t *testing.T) {
		cert := axiomReportFixtureCertificate()
		report, err := BuildAxiomReport(cert)
		if err != nil {
			t.Fatalf("BuildAxiomReport() error = %v", err)
		}
		cert.AxiomReport = report
		cert.Hashes.AxiomReportHash[0] = 0xff

		reportErr := checkAxiomReportError(t, cert)
		if reportErr.Kind != AxiomReportHashMismatch {
			t.Fatalf("error kind = %s, want %s", reportErr.Kind, AxiomReportHashMismatch)
		}
	})
}

func TestBasicFixtureAxiomReportsMatchRust(t *testing.T) {
	for _, fixture := range []string{"zero-axiom", "one-theorem"} {
		fixture := fixture
		t.Run(fixture, func(t *testing.T) {
			cert, err := DecodeCertificate(readHexFixture(t, "fixtures/cert-basic/"+fixture+".hex"))
			if err != nil {
				t.Fatalf("DecodeCertificate() error = %v", err)
			}
			if _, err := CheckAxiomReport(cert); err != nil {
				t.Fatalf("CheckAxiomReport() error = %v", err)
			}
		})
	}
}

func TestBuildAxiomReportRejectsFutureDeclarationReferences(t *testing.T) {
	cert := axiomReportFixtureCertificate()
	cert.Declarations[1] = Declaration{
		Name:         1,
		Tag:          DeclDef,
		Type:         0,
		Value:        4,
		Reducibility: Reducible,
	}

	err := buildAxiomReportError(t, cert)
	if err.Kind != AxiomReportFutureDeclarationReference {
		t.Fatalf("error kind = %s, want %s", err.Kind, AxiomReportFutureDeclarationReference)
	}
}

func axiomReportFixtureCertificate() *Certificate {
	return &Certificate{
		Module: "Example.AxiomReport",
		NameTable: []string{
			"Example.AxiomReport.ax",
			"Example.AxiomReport.def",
			"Example.AxiomReport.thm",
			"Example.AxiomReport.theory",
			"Example.AxiomReport.usesTheory",
		},
		LevelTable: []LevelNode{
			{Tag: LevelZero},
		},
		TermTable: []TermNode{
			{Tag: TermSort, A: 0},
			{Tag: TermConst, A: 0},
			{Tag: TermConst, A: 1},
			{Tag: TermVar, A: 0},
			{Tag: TermConst, A: 3},
		},
		Declarations: []Declaration{
			{Name: 0, Tag: DeclAxiom, Type: 0},
			{Name: 1, Tag: DeclDef, Type: 0, Value: 1, Reducibility: Reducible},
			{Name: 2, Tag: DeclTheorem, Type: 2, Proof: 3},
			{Name: 3, Tag: DeclTheoryPrimitive, Type: 0},
			{Name: 4, Tag: DeclTheorem, Type: 4, Proof: 3},
		},
	}
}

func renderAxiomReportFixture(report AxiomReport) string {
	var output string
	output += "summary core=" + formatUint64(report.Summary.CoreAxiomCount) +
		" builtin=" + formatUint64(report.Summary.BuiltinTheoryAxiomCount) +
		" go=" + formatUint64(report.Summary.GoSemanticsAxiomCount) +
		" external=" + formatUint64(report.Summary.ExternalAxiomCount) +
		" total=" + formatUint64(report.Summary.TotalAxiomCount) + "\n"
	output += "entries\n"
	for index, entry := range report.Entries {
		output += formatUint64(uint64(index)) +
			" category=" + string(entry.Category) +
			" name=" + entry.Name +
			" origin=" + entry.OriginModule +
			" type_hash=" + HashHex(entry.TypeHash) +
			" declaration_hash=" + HashHex(entry.DeclarationHash) +
			" direct=" + formatUint32Slice(entry.DirectDependentDeclarations) +
			" transitive=" + formatUint32Slice(entry.TransitiveDependentDeclarations) + "\n"
	}
	output += "declarations\n"
	for _, dependencies := range report.DeclarationDependencies {
		output += dependencies.DeclarationName +
			" declaration_hash=" + HashHex(dependencies.DeclarationHash) +
			" direct=" + formatUint32Slice(dependencies.DirectAxiomDependencies) +
			" transitive=" + formatUint32Slice(dependencies.TransitiveAxiomDependencies) + "\n"
	}
	return output
}

func formatUint32Slice(values []uint32) string {
	if len(values) == 0 {
		return "[]"
	}
	output := "["
	for index, value := range values {
		if index > 0 {
			output += ", "
		}
		output += formatUint64(uint64(value))
	}
	return output + "]"
}

func findAxiomReportEntry(t *testing.T, report AxiomReport, name string) AxiomReportEntry {
	t.Helper()
	for _, entry := range report.Entries {
		if entry.Name == name {
			return entry
		}
	}
	t.Fatalf("missing axiom report entry %s", name)
	return AxiomReportEntry{}
}

func findDeclarationAxiomDependencies(t *testing.T, report AxiomReport, name string) DeclarationAxiomDependencies {
	t.Helper()
	for _, dependencies := range report.DeclarationDependencies {
		if dependencies.DeclarationName == name {
			return dependencies
		}
	}
	t.Fatalf("missing declaration axiom dependencies %s", name)
	return DeclarationAxiomDependencies{}
}

func buildAxiomReportError(t *testing.T, cert *Certificate) *AxiomReportCheckError {
	t.Helper()
	_, err := BuildAxiomReport(cert)
	if err == nil {
		t.Fatal("BuildAxiomReport() succeeded, want error")
	}
	reportErr, ok := err.(*AxiomReportCheckError)
	if !ok {
		t.Fatalf("error type = %T, want *AxiomReportCheckError", err)
	}
	return reportErr
}

func checkAxiomReportError(t *testing.T, cert *Certificate) *AxiomReportCheckError {
	t.Helper()
	_, err := CheckAxiomReport(cert)
	if err == nil {
		t.Fatal("CheckAxiomReport() succeeded, want error")
	}
	reportErr, ok := err.(*AxiomReportCheckError)
	if !ok {
		t.Fatalf("error type = %T, want *AxiomReportCheckError", err)
	}
	return reportErr
}
