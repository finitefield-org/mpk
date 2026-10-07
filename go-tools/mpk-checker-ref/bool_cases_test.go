package mpkcheckerref

import "testing"

func TestBoolCasesCertificatesMatchRust(t *testing.T) {
	for _, name := range []string{"right-identity", "constructor-false", "constructor-true", "open-motive", "conjunction-left", "conjunction-right", "wrong-branch", "wrong-conjunction-left", "wrong-conjunction-right", "wrong-motive-universe", "wrong-interface", "nongenerated", "extra-constructor", "extra-constructor-after-cases", "renamed", "wrong-case-levels", "wrong-major-levels", "wrong-case-levels-open", "wrong-major-levels-open", "later-constructor-value-levels", "legacy-prior-constructor-value-levels", "legacy-prior-theorem-proof-levels", "prior-constructor-value-levels", "prior-family-type-levels", "prior-lambda-body-levels", "prior-let-value-levels", "prior-opaque-value-levels", "prior-other-family-levels", "prior-pi-domain-levels", "prior-theorem-proof-levels", "unused-constructor-levels"} {
		t.Run(name, func(t *testing.T) {
			certificate, err := DecodeCertificate(readHexFixture(t, "fixtures/core-bool-cases/"+name+".hex"))
			if err != nil {
				t.Fatal(err)
			}
			_, err = CheckCoreDeclarations(certificate)
			good := name == "right-identity" || name == "constructor-false" || name == "constructor-true" || name == "open-motive" || name == "conjunction-left" || name == "conjunction-right" || name == "legacy-prior-constructor-value-levels" || name == "legacy-prior-theorem-proof-levels" || name == "unused-constructor-levels" || name == "prior-other-family-levels"
			if (err == nil) != good {
				t.Fatalf("accepted = %v, want %v: %v", err == nil, good, err)
			}
		})
	}
}

func TestBoolCasesClosesConstructorRegistrationWithoutChangingLegacyFamilies(t *testing.T) {
	state, family, _, _ := boolCasesTestState(t)
	ty, err := state.boolCasesType(family)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := state.env.registerGenerated("Test.Bool.cases", DeclRecursor, ty, family, true); err != nil {
		t.Fatal(err)
	}
	before := len(state.env.declarations)
	for _, generated := range []bool{false, true} {
		_, err := state.env.register("Test.Bool.extra", coreDeclaration{
			tag: DeclConstructor, ty: state.terms.constant(family, nil), inductive: family, generated: generated,
		})
		if err == nil || len(state.env.declarations) != before {
			t.Fatal("late constructor registration did not reject without changing the environment")
		}
	}
	legacy, legacyFamily, _, _ := boolCasesTestState(t)
	if _, err := legacy.env.registerGenerated("Test.Bool.extra", DeclConstructor, legacy.terms.constant(legacyFamily, nil), legacyFamily, true); err != nil {
		t.Fatalf("legacy constructor registration changed: %v", err)
	}
}

func boolCasesTestState(t *testing.T) (coreState, coreGlobalID, coreGlobalID, coreGlobalID) {
	t.Helper()
	state := newCoreState()
	family, err := state.env.registerInductive("Test.Bool", state.terms.sort(state.levels.zero()))
	if err != nil {
		t.Fatal(err)
	}
	boolean := state.terms.constant(family, nil)
	no, err := state.env.registerGenerated("Test.Bool.false", DeclConstructor, boolean, family, true)
	if err != nil {
		t.Fatal(err)
	}
	yes, err := state.env.registerGenerated("Test.Bool.true", DeclConstructor, boolean, family, true)
	if err != nil {
		t.Fatal(err)
	}
	return state, family, no, yes
}

func TestBoolCasesOpenMotiveStaysDependentAndNeutral(t *testing.T) {
	state, family, noGlobal, yesGlobal := boolCasesTestState(t)
	ty, err := state.boolCasesType(family)
	if err != nil {
		t.Fatal(err)
	}
	global, err := state.env.registerGenerated("Test.Bool.cases", DeclRecursor, ty, family, true)
	if err != nil {
		t.Fatal(err)
	}
	boolean := state.terms.constant(family, nil)
	sort := state.terms.sort(state.levels.zero())
	context := coreLocalContext(nil).withBinder(state.terms.pi(boolean, sort))
	context = context.withBinder(state.terms.app(state.terms.varTerm(0), []coreTermID{state.terms.constant(noGlobal, nil)}))
	context = context.withBinder(state.terms.app(state.terms.varTerm(1), []coreTermID{state.terms.constant(yesGlobal, nil)}))
	context = context.withBinder(boolean)
	p, no, yes, major := state.terms.varTerm(3), state.terms.varTerm(2), state.terms.varTerm(1), state.terms.varTerm(0)
	application := state.terms.app(state.terms.constant(global, nil), []coreTermID{p, no, yes, major})
	wanted := state.terms.app(p, []coreTermID{major})
	inferred, err := state.infer(application, context)
	if err != nil || inferred != wanted {
		t.Fatalf("dependent result = %v, want %v: %v", inferred, wanted, err)
	}
	assertDefeq(t, &state, application, no, false)
	assertDefeq(t, &state, application, yes, false)
}

func TestBoolCasesRejectsMalformedFamilyBeforeReduction(t *testing.T) {
	state, family, _, _ := boolCasesTestState(t)
	ty, err := state.boolCasesType(family)
	if err != nil {
		t.Fatal(err)
	}
	if err := state.checkBoolCasesDeclaration("Test.Bool.cases", ty, family, false); err == nil {
		t.Fatal("nongenerated cases accepted")
	}
	boolean := state.terms.constant(family, nil)
	if err := state.checkBoolCasesDeclaration("Test.Bool.cases", boolean, family, true); err == nil {
		t.Fatal("wrong interface accepted")
	}
	if _, err := state.env.registerGenerated("Test.Bool.extra", DeclConstructor, boolean, family, false); err != nil {
		t.Fatal(err)
	}
	if err := state.checkBoolCasesDeclaration("Test.Bool.cases", ty, family, true); err == nil {
		t.Fatal("extra constructor accepted")
	}
}

func TestBoolCasesRejectsUniverseArgumentsWithoutNeedingAnIotaRedex(t *testing.T) {
	state, family, no, yes := boolCasesTestState(t)
	legacy := state.terms.constant(no, []coreLevelID{state.levels.zero()})
	if _, err := state.infer(legacy, nil); err != nil {
		t.Fatalf("legacy universe-argument handling changed: %v", err)
	}
	ty, err := state.boolCasesType(family)
	if err != nil {
		t.Fatal(err)
	}
	cases, err := state.env.registerGenerated("Test.Bool.cases", DeclRecursor, ty, family, true)
	if err != nil {
		t.Fatal(err)
	}
	for _, global := range []coreGlobalID{family, no, yes, cases} {
		constant := state.terms.constant(global, []coreLevelID{state.levels.zero()})
		if _, err := state.infer(constant, nil); err == nil {
			t.Fatalf("universe arguments accepted for global %v", global)
		}
	}
}
