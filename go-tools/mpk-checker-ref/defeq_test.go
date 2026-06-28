package mpkcheckerref

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestDefeqCoreNegativeFixturesMatchRust(t *testing.T) {
	fixtures := readNegativeDefeqFixtures(t)
	if len(fixtures) != 3 {
		t.Fatalf("negative defeq fixture count = %d, want 3", len(fixtures))
	}

	expectedConversions := map[string]string{
		"eta":               "eta",
		"proof-irrelevance": "proof_irrelevance",
		"theorem-unfolding": "theorem_unfolding",
	}

	seen := make(map[string]bool)
	for _, fixture := range fixtures {
		if fixture.expected != "reject" {
			t.Fatalf("%s: expected verdict = %q, want reject", fixture.id, fixture.expected)
		}
		if fixture.forbiddenConversion != expectedConversions[fixture.id] {
			t.Fatalf("%s: forbidden conversion = %q", fixture.id, fixture.forbiddenConversion)
		}

		accepted, err := runNegativeDefeqFixture(fixture.id)
		if err != nil {
			t.Fatalf("%s: defeq error = %v", fixture.id, err)
		}
		if accepted {
			t.Fatalf("%s: defeq accepted forbidden conversion", fixture.id)
		}
		seen[fixture.id] = true
	}

	for id := range expectedConversions {
		if !seen[id] {
			t.Fatalf("missing fixture id %s", id)
		}
	}
}

func TestDefeqReducesBetaZetaAndReducibleDefinitions(t *testing.T) {
	state := newCoreState()
	ty := sortParam(&state, "u")
	value := sortParam(&state, "v")
	body := state.terms.varTerm(0)
	lambda := state.terms.lam(ty, body)
	application := state.terms.app(lambda, []coreTermID{value})
	letTerm := state.terms.letTerm(ty, value, body)

	assertDefeq(t, &state, application, value, true)
	assertDefeq(t, &state, letTerm, value, true)

	zero := state.levels.zero()
	reducibleValue := state.terms.sort(zero)
	valueType := state.terms.sort(state.levels.succ(zero))
	global, err := state.env.registerDefinition("Core.ReducibleValue", valueType, reducibleValue, Reducible)
	if err != nil {
		t.Fatalf("register reducible definition: %v", err)
	}
	constant := state.terms.constant(global, nil)
	assertDefeq(t, &state, constant, reducibleValue, true)
}

func TestDefeqReducesGeneratedBoolRecursorIota(t *testing.T) {
	state := newCoreState()
	boolType := state.terms.sort(state.levels.zero())
	boolGlobal, err := state.env.registerInductive("Std.Bool", boolType)
	if err != nil {
		t.Fatalf("register Bool: %v", err)
	}
	boolTerm := state.terms.constant(boolGlobal, nil)
	falseGlobal, err := state.env.registerGenerated("Std.Bool.false", DeclConstructor, boolTerm, boolGlobal, true)
	if err != nil {
		t.Fatalf("register false: %v", err)
	}
	trueGlobal, err := state.env.registerGenerated("Std.Bool.true", DeclConstructor, boolTerm, boolGlobal, true)
	if err != nil {
		t.Fatalf("register true: %v", err)
	}
	recType := state.terms.pi(
		boolTerm,
		state.terms.pi(boolTerm, state.terms.pi(boolTerm, boolTerm)),
	)
	recGlobal, err := state.env.registerGenerated("Std.Bool.rec", DeclRecursor, recType, boolGlobal, true)
	if err != nil {
		t.Fatalf("register recursor: %v", err)
	}

	rec := state.terms.constant(recGlobal, nil)
	falseTerm := state.terms.constant(falseGlobal, nil)
	trueTerm := state.terms.constant(trueGlobal, nil)
	falseRedex := state.terms.app(rec, []coreTermID{falseTerm, trueTerm, falseTerm})
	trueRedex := state.terms.app(rec, []coreTermID{falseTerm, trueTerm, trueTerm})

	assertDefeq(t, &state, falseRedex, falseTerm, true)
	assertDefeq(t, &state, trueRedex, trueTerm, true)
}

func TestDefeqDoesNotUnfoldOpaqueDefinitionsOrTheorems(t *testing.T) {
	state := newCoreState()
	zero := state.levels.zero()
	value := state.terms.sort(zero)
	valueType := state.terms.sort(state.levels.succ(zero))

	opaque, err := state.env.registerDefinition("Core.OpaqueValue", valueType, value, Opaque)
	if err != nil {
		t.Fatalf("register opaque definition: %v", err)
	}
	opaqueConstant := state.terms.constant(opaque, nil)
	assertDefeq(t, &state, opaqueConstant, value, false)

	theorem, err := state.env.registerTheorem("Core.CheckedTheorem", valueType, value)
	if err != nil {
		t.Fatalf("register theorem: %v", err)
	}
	theoremConstant := state.terms.constant(theorem, nil)
	assertDefeq(t, &state, theoremConstant, value, false)
}

func TestDefeqFuelExhaustionIsDeterministic(t *testing.T) {
	state := newCoreState()
	term := sortParam(&state, "u")

	_, err := state.definitionallyEqualWithFuel(term, term, 0)
	if err == nil {
		t.Fatal("definitionallyEqualWithFuel() succeeded, want fuel error")
	}
	coreErr, ok := err.(*CoreCheckError)
	if !ok {
		t.Fatalf("error type = %T, want *CoreCheckError", err)
	}
	if coreErr.Kind != CoreCheckFuelExhausted {
		t.Fatalf("error kind = %s, want %s", coreErr.Kind, CoreCheckFuelExhausted)
	}

	letTerm := state.terms.letTerm(term, term, state.terms.varTerm(0))
	_, err = state.whnfWithFuel(letTerm, 0, false)
	if err == nil {
		t.Fatal("whnfWithFuel() succeeded, want fuel error")
	}
	coreErr, ok = err.(*CoreCheckError)
	if !ok {
		t.Fatalf("error type = %T, want *CoreCheckError", err)
	}
	if coreErr.Kind != CoreCheckFuelExhausted {
		t.Fatalf("error kind = %s, want %s", coreErr.Kind, CoreCheckFuelExhausted)
	}
}

type negativeDefeqFixture struct {
	id                  string
	forbiddenConversion string
	expected            string
}

func readNegativeDefeqFixtures(t *testing.T) []negativeDefeqFixture {
	t.Helper()
	paths := globFixtures(t, "fixtures/core-negative/*.fixture")
	fixtures := make([]negativeDefeqFixture, 0, len(paths))
	for _, path := range paths {
		contents, err := os.ReadFile(path)
		if err != nil {
			t.Fatalf("read %s: %v", path, err)
		}
		fixture := negativeDefeqFixture{}
		for _, rawLine := range strings.Split(string(contents), "\n") {
			line := strings.TrimSpace(rawLine)
			if line == "" || strings.HasPrefix(line, "#") {
				continue
			}
			key, value, ok := strings.Cut(line, ":")
			if !ok {
				t.Fatalf("%s: fixture line is not key/value: %q", path, line)
			}
			value = strings.TrimSpace(value)
			switch strings.TrimSpace(key) {
			case "id":
				fixture.id = value
			case "forbidden_conversion":
				fixture.forbiddenConversion = value
			case "expected":
				fixture.expected = value
			}
		}
		if fixture.id == "" || fixture.forbiddenConversion == "" || fixture.expected == "" {
			t.Fatalf("%s: incomplete fixture %#v", filepath.Base(path), fixture)
		}
		fixtures = append(fixtures, fixture)
	}
	return fixtures
}

func runNegativeDefeqFixture(id string) (bool, error) {
	switch id {
	case "eta":
		return runEtaRejectionFixture()
	case "proof-irrelevance":
		return runProofIrrelevanceRejectionFixture()
	case "theorem-unfolding":
		return runTheoremUnfoldingRejectionFixture()
	default:
		return false, newCoreError(CoreCheckInternalInvariant, "unknown defeq fixture "+id)
	}
}

func runEtaRejectionFixture() (bool, error) {
	state := newCoreState()
	ty := sortParam(&state, "u")
	function := state.terms.varTerm(0)
	liftedFunction := state.terms.varTerm(1)
	argument := state.terms.varTerm(0)
	etaBody := state.terms.app(liftedFunction, []coreTermID{argument})
	etaExpansion := state.terms.lam(ty, etaBody)
	return state.definitionallyEqual(etaExpansion, function)
}

func runProofIrrelevanceRejectionFixture() (bool, error) {
	state := newCoreState()
	zero := state.levels.zero()
	proofType := state.terms.sort(state.levels.succ(zero))
	first, err := state.env.registerAxiom("Core.ProofIrrelevance.first", proofType)
	if err != nil {
		return false, err
	}
	second, err := state.env.registerAxiom("Core.ProofIrrelevance.second", proofType)
	if err != nil {
		return false, err
	}
	firstProof := state.terms.constant(first, nil)
	secondProof := state.terms.constant(second, nil)
	return state.definitionallyEqual(firstProof, secondProof)
}

func runTheoremUnfoldingRejectionFixture() (bool, error) {
	state := newCoreState()
	zero := state.levels.zero()
	theoremType := state.terms.sort(state.levels.succ(zero))
	proof := state.terms.sort(zero)
	if err := state.check(proof, theoremType, nil); err != nil {
		return false, err
	}
	theorem, err := state.env.registerTheorem("Core.TheoremUnfolding.opaque", theoremType, proof)
	if err != nil {
		return false, err
	}
	theoremConstant := state.terms.constant(theorem, nil)
	return state.definitionallyEqual(theoremConstant, proof)
}

func sortParam(state *coreState, name string) coreTermID {
	return state.terms.sort(state.levels.param(name))
}

func assertDefeq(t *testing.T, state *coreState, lhs coreTermID, rhs coreTermID, want bool) {
	t.Helper()
	got, err := state.definitionallyEqual(lhs, rhs)
	if err != nil {
		t.Fatalf("definitionallyEqual() error = %v", err)
	}
	if got != want {
		t.Fatalf("definitionallyEqual() = %v, want %v", got, want)
	}
}
