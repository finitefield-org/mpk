package mpkcheckerref

import (
	"math/rand"
	"testing"
)

func TestCoreTermInterningPreservesLinearFirstID(t *testing.T) {
	var arena coreTermArena
	var linear []coreTermNode
	check := func(node coreTermNode) {
		t.Helper()
		want := coreTermID(len(linear))
		for i, existing := range linear {
			if coreTermNodeEqual(existing, node) {
				want = coreTermID(i)
				break
			}
		}
		if int(want) == len(linear) {
			linear = append(linear, node)
		}
		if got := arena.intern(node); got != want {
			t.Fatalf("intern(%+v) = %d, linear first ID = %d", node, got, want)
		}
	}
	for _, node := range []coreTermNode{
		{}, {Levels: []coreLevelID{}, Arguments: []coreTermID{}},
		{A: 1}, {B: 1}, {C: 1}, {A: 1 << 24},
		{Levels: []coreLevelID{0}}, {Arguments: []coreTermID{0}},
		{Levels: []coreLevelID{1, 2}}, {Levels: []coreLevelID{2, 1}},
		{Arguments: []coreTermID{1, 2}}, {Arguments: []coreTermID{2, 1}},
		{Levels: []coreLevelID{1}, Arguments: []coreTermID{2}},
		{Levels: []coreLevelID{1, 2}},
	} {
		check(node)
	}
	random := rand.New(rand.NewSource(1))
	for i := 0; i < 4000; i++ {
		node := coreTermNode{
			Tag: TermTag(random.Intn(8)), A: random.Uint32(),
			B: random.Uint32(), C: random.Uint32(),
		}
		for j := random.Intn(5); j > 0; j-- {
			node.Levels = append(node.Levels, coreLevelID(random.Uint32()))
		}
		for j := random.Intn(5); j > 0; j-- {
			node.Arguments = append(node.Arguments, coreTermID(random.Uint32()))
		}
		check(node)
		check(linear[random.Intn(len(linear))])
		check(node)
	}
	if len(arena.nodes) != len(linear) {
		t.Fatalf("arena stores %d nodes, linear stores %d", len(arena.nodes), len(linear))
	}
}

func TestCoreTermInterningRetainsFirstPrepopulatedID(t *testing.T) {
	node := coreTermNode{Tag: TermConst, A: 3, Levels: []coreLevelID{1, 2}}
	arena := coreTermArena{nodes: []coreTermNode{node, node}}
	if got := arena.intern(node); got != 0 {
		t.Fatalf("duplicate prepopulated term ID = %d, want first ID 0", got)
	}
	if got := arena.intern(coreTermNode{Tag: TermVar, A: 1}); got != 2 {
		t.Fatalf("new prepopulated term ID = %d, want 2", got)
	}
}

func TestCoreTermConstructorsOwnIndexedSliceFields(t *testing.T) {
	var arena coreTermArena
	levels := []coreLevelID{1, 2}
	constant := arena.constant(3, levels)
	levels[0] = 4
	if got := arena.constant(3, []coreLevelID{1, 2}); got != constant {
		t.Fatalf("caller changed indexed constant: ID %d, want %d", got, constant)
	}
	arguments := []coreTermID{constant, arena.varTerm(0)}
	application := arena.app(constant, arguments)
	arguments[0] = arguments[1]
	if got := arena.app(constant, []coreTermID{constant, arena.varTerm(0)}); got != application {
		t.Fatalf("caller changed indexed application: ID %d, want %d", got, application)
	}
}
