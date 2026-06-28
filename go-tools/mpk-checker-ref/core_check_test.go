package mpkcheckerref

import "testing"

func TestCheckCoreBasicCertificateFixturesMatchRustVerdicts(t *testing.T) {
	fixtures := []struct {
		path             string
		declarationCount int
	}{
		{"fixtures/cert-basic/zero-axiom.hex", 0},
		{"fixtures/cert-basic/one-theorem.hex", 1},
	}

	for _, fixture := range fixtures {
		fixture := fixture
		t.Run(fixture.path, func(t *testing.T) {
			cert, err := DecodeCertificate(readHexFixture(t, fixture.path))
			if err != nil {
				t.Fatalf("DecodeCertificate() error = %v", err)
			}

			report, err := CheckCore(cert)
			if err != nil {
				t.Fatalf("CheckCore() error = %v", err)
			}
			if report.DeclarationCount != fixture.declarationCount {
				t.Fatalf("declaration count = %d, want %d", report.DeclarationCount, fixture.declarationCount)
			}
		})
	}
}

func TestCheckCoreDependencyVerdictsMatchRust(t *testing.T) {
	t.Run("out_of_order", func(t *testing.T) {
		cert := oneTheoremCoreCertificate(0)
		cert.NameTable = []string{
			"Example.Driver.UsesFuture",
			"Example.Driver.Future",
		}
		cert.TermTable = append(cert.TermTable, TermNode{Tag: TermConst, A: 1})
		cert.Declarations = []Declaration{
			{Name: 0, Tag: DeclTheorem, Type: 1, Proof: 2},
			{Name: 1, Tag: DeclAxiom, Type: 1},
		}

		err := checkCoreError(t, cert)
		if err.Kind != CoreCheckOutOfOrderDependency {
			t.Fatalf("error kind = %s, want %s", err.Kind, CoreCheckOutOfOrderDependency)
		}
	})

	t.Run("missing_global", func(t *testing.T) {
		cert := oneTheoremCoreCertificate(0)
		cert.TermTable = append(cert.TermTable, TermNode{Tag: TermConst, A: 99})
		cert.Declarations[0] = Declaration{Name: 0, Tag: DeclTheorem, Type: 1, Proof: 2}

		err := checkCoreError(t, cert)
		if err.Kind != CoreCheckMissingGlobal {
			t.Fatalf("error kind = %s, want %s", err.Kind, CoreCheckMissingGlobal)
		}
	})
}

func TestCoreTermTypingMatchesRustInferCases(t *testing.T) {
	var state coreState
	state = newCoreState()

	zero := state.levels.zero()
	succZero := state.levels.succ(zero)
	sortZero := state.terms.sort(zero)
	sortSuccZero := state.terms.sort(succZero)

	inferred, err := state.infer(sortZero, nil)
	if err != nil {
		t.Fatalf("infer sort: %v", err)
	}
	if inferred != sortSuccZero {
		t.Fatalf("Sort 0 inferred term = %d, want %d", inferred, sortSuccZero)
	}

	domain := sortSuccZero
	argument := sortZero
	body := state.terms.varTerm(0)
	lambda := state.terms.lam(domain, body)
	application := state.terms.app(lambda, []coreTermID{argument})

	inferred, err = state.infer(application, nil)
	if err != nil {
		t.Fatalf("infer application: %v", err)
	}
	if inferred != domain {
		t.Fatalf("application inferred term = %d, want %d", inferred, domain)
	}

	letTerm := state.terms.letTerm(domain, argument, body)
	inferred, err = state.infer(letTerm, nil)
	if err != nil {
		t.Fatalf("infer let: %v", err)
	}
	if inferred != domain {
		t.Fatalf("let inferred term = %d, want %d", inferred, domain)
	}
}

func TestCoreCheckUsesReducibleDefinitionsForExpectedTypes(t *testing.T) {
	state := newCoreState()
	zero := state.levels.zero()
	succZero := state.levels.succ(zero)
	succSuccZero := state.levels.succ(succZero)
	term := state.terms.sort(zero)
	inferredType := state.terms.sort(succZero)
	definitionType := state.terms.sort(succSuccZero)

	global, err := state.env.registerDefinition(
		"Core.ExpectedType",
		definitionType,
		inferredType,
		Reducible,
	)
	if err != nil {
		t.Fatalf("register definition: %v", err)
	}
	expected := state.terms.constant(global, nil)

	if err := state.check(term, expected, nil); err != nil {
		t.Fatalf("check against reducible expected type: %v", err)
	}
}

func TestCoreTermTypingRejectsMismatchedApplicationArgument(t *testing.T) {
	state := newCoreState()
	zero := state.levels.zero()
	domainLevel := state.levels.succ(zero)
	domain := state.terms.sort(domainLevel)
	wrongArgument := state.terms.sort(domainLevel)
	body := state.terms.varTerm(0)
	lambda := state.terms.lam(domain, body)
	application := state.terms.app(lambda, []coreTermID{wrongArgument})

	_, err := state.infer(application, nil)
	if err == nil {
		t.Fatal("infer application succeeded, want error")
	}
	coreErr, ok := err.(*CoreCheckError)
	if !ok {
		t.Fatalf("error type = %T, want *CoreCheckError", err)
	}
	if coreErr.Kind != CoreCheckTypeMismatch {
		t.Fatalf("error kind = %s, want %s", coreErr.Kind, CoreCheckTypeMismatch)
	}
}

func oneTheoremCoreCertificate(proof uint32) *Certificate {
	return &Certificate{
		Module:     "Example.Driver.OneTheorem",
		NameTable:  []string{"Example.Driver.OneTheorem.sort0IsSort1"},
		LevelTable: []LevelNode{{Tag: LevelZero}, {Tag: LevelSucc, A: 0}},
		TermTable:  []TermNode{{Tag: TermSort, A: 0}, {Tag: TermSort, A: 1}},
		Declarations: []Declaration{
			{Name: 0, Tag: DeclTheorem, Type: 1, Proof: proof},
		},
	}
}

func checkCoreError(t *testing.T, cert *Certificate) *CoreCheckError {
	t.Helper()
	_, err := CheckCore(cert)
	if err == nil {
		t.Fatal("CheckCore() succeeded, want error")
	}
	coreErr, ok := err.(*CoreCheckError)
	if !ok {
		t.Fatalf("error type = %T, want *CoreCheckError", err)
	}
	return coreErr
}
