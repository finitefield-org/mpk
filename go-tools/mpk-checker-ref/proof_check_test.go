package mpkcheckerref

import "testing"

func TestCheckProofNodesCoreBootstrapFixturesMatchRust(t *testing.T) {
	report, err := CheckProofNodesWithProfile(bootstrapProofCertificate(), ProofCheckCoreBootstrap)
	if err != nil {
		t.Fatalf("CheckProofNodesWithProfile() error = %v", err)
	}
	if report.ProofNodeCount != 7 {
		t.Fatalf("proof node count = %d, want 7", report.ProofNodeCount)
	}
}

func TestCheckProofNodesStructuralFixturesMatchRust(t *testing.T) {
	report, err := CheckProofNodes(structuralProofCertificate())
	if err != nil {
		t.Fatalf("CheckProofNodes() error = %v", err)
	}
	if report.ProofNodeCount != 7 {
		t.Fatalf("proof node count = %d, want 7", report.ProofNodeCount)
	}
}

func TestCheckProofNodesProfileRejectionsMatchRust(t *testing.T) {
	t.Run("core_bootstrap_rejects_structural_node", func(t *testing.T) {
		cert := bootstrapProofCertificate()
		cert.ProofNodeTable = append(cert.ProofNodeTable, ProofNode{
			Tag:          ProofLetProof,
			Value:        2,
			BodyProof:    0,
			ExpectedType: 0,
		})

		err := checkProofError(t, cert, ProofCheckCoreBootstrap)
		if err.Kind != ProofCheckUnsupportedProofNodeKind {
			t.Fatalf("error kind = %s, want %s", err.Kind, ProofCheckUnsupportedProofNodeKind)
		}
	})

	t.Run("structural_profile_rejects_theory_node", func(t *testing.T) {
		cert := structuralProofCertificate()
		cert.TheoryCertificates = append(cert.TheoryCertificates, TheoryCertificate{Format: "dummy"})
		cert.ProofNodeTable = append(cert.ProofNodeTable, ProofNode{
			Tag:               ProofTheory,
			TheoryCertificate: 0,
			ExpectedType:      2,
		})

		err := checkProofError(t, cert, ProofCheckMvpStructural)
		if err.Kind != ProofCheckUnsupportedProofNodeKind {
			t.Fatalf("error kind = %s, want %s", err.Kind, ProofCheckUnsupportedProofNodeKind)
		}
	})
}

func TestCheckProofNodesGeneratedConstructorVerdictMatchesRust(t *testing.T) {
	cert := structuralProofCertificate()
	cert.Declarations[1].Generated = false

	err := checkProofError(t, cert, ProofCheckMvpStructural)
	if err.Kind != ProofCheckUnsupportedProofNodeKind {
		t.Fatalf("error kind = %s, want %s", err.Kind, ProofCheckUnsupportedProofNodeKind)
	}
}

func TestCheckProofNodesRejectsBadExactNode(t *testing.T) {
	cert := bootstrapProofCertificate()
	cert.ProofNodeTable = []ProofNode{{
		Tag:          ProofExact,
		Term:         2,
		ExpectedType: 1,
	}}

	err := checkProofError(t, cert, ProofCheckMvpStructural)
	if err.Kind != ProofCheckCoreCheck {
		t.Fatalf("error kind = %s, want %s", err.Kind, ProofCheckCoreCheck)
	}
}

func bootstrapProofCertificate() *Certificate {
	defeqWitness := uint32(0)
	return &Certificate{
		Module:    "Example.ProofBootstrap",
		NameTable: []string{"Example.ProofBootstrap.x"},
		LevelTable: []LevelNode{
			{Tag: LevelZero},
			{Tag: LevelSucc, A: 0},
		},
		TermTable: []TermNode{
			{Tag: TermSort, A: 0},
			{Tag: TermSort, A: 1},
			{Tag: TermConst, A: 0},
			{Tag: TermVar, A: 0},
			{Tag: TermLam, A: 0, B: 3},
			{Tag: TermPi, A: 0, B: 0},
		},
		ProofNodeTable: []ProofNode{
			{Tag: ProofExact, Term: 2, ExpectedType: 0},
			{Tag: ProofExact, Term: 4, ExpectedType: 5},
			{Tag: ProofApply, FunctionProof: 1, ArgumentProofs: []uint32{0}, ExpectedType: 0},
			{Tag: ProofExact, Term: 3, ExpectedType: 0},
			{Tag: ProofIntro, DomainType: 0, BodyProof: 3, ExpectedType: 5},
			{Tag: ProofRefl, Term: 2, ExpectedType: 0},
			{Tag: ProofConv, Proof: 0, ExpectedType: 0, DefeqWitness: &defeqWitness},
		},
		Declarations: []Declaration{
			{Name: 0, Tag: DeclAxiom, Type: 0},
		},
	}
}

func structuralProofCertificate() *Certificate {
	return &Certificate{
		Module: "Example.ProofStructural",
		NameTable: []string{
			"Example.ProofStructural.Bool",
			"Example.ProofStructural.Bool.false",
			"Example.ProofStructural.Bool.true",
			"Example.ProofStructural.Bool.rec",
		},
		LevelTable: []LevelNode{
			{Tag: LevelZero},
			{Tag: LevelSucc, A: 0},
		},
		TermTable: []TermNode{
			{Tag: TermSort, A: 0},
			{Tag: TermSort, A: 1},
			{Tag: TermConst, A: 0},
			{Tag: TermConst, A: 1},
			{Tag: TermConst, A: 2},
			{Tag: TermPi, A: 2, B: 2},
			{Tag: TermPi, A: 2, B: 5},
			{Tag: TermPi, A: 2, B: 6},
			{Tag: TermVar, A: 0},
		},
		ProofNodeTable: []ProofNode{
			{Tag: ProofConstructor, Constructor: 1, ExpectedType: 2},
			{Tag: ProofConstructor, Constructor: 2, ExpectedType: 2},
			{Tag: ProofRecursor, Recursor: 3, Motive: 2, MinorProofs: []uint32{0, 1}, MajorProof: 0, ExpectedType: 2},
			{Tag: ProofLetProof, Value: 3, BodyProof: 4, ExpectedType: 2},
			{Tag: ProofExact, Term: 8, ExpectedType: 2},
			{Tag: ProofRewrite, EqProof: 1, TargetProof: 0, ExpectedType: 2},
			{Tag: ProofEqRec, Motive: 2, EqProof: 1, BaseProof: 0, ExpectedType: 2},
		},
		Declarations: []Declaration{
			{Name: 0, Tag: DeclInductive, Type: 0},
			{Name: 1, Tag: DeclConstructor, Type: 2, Inductive: 0, Generated: true},
			{Name: 2, Tag: DeclConstructor, Type: 2, Inductive: 0, Generated: true},
			{Name: 3, Tag: DeclRecursor, Type: 7, Inductive: 0, Generated: true},
		},
	}
}

func checkProofError(t *testing.T, cert *Certificate, profile ProofCheckProfile) *ProofCheckError {
	t.Helper()
	_, err := CheckProofNodesWithProfile(cert, profile)
	if err == nil {
		t.Fatal("CheckProofNodesWithProfile() succeeded, want error")
	}
	proofErr, ok := err.(*ProofCheckError)
	if !ok {
		t.Fatalf("error type = %T, want *ProofCheckError", err)
	}
	return proofErr
}
