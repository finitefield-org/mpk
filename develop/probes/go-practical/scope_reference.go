// Private original-Go type/runtime observations for W04 semantic decisions.
// Observer reflection, global counters and recovery are outside verified code.
package main

import (
	"encoding/json"
	"go/ast"
	"go/parser"
	"go/token"
	"go/types"
	"os"
	"reflect"
	"runtime"
	"sort"
	"strconv"
)

type sourceCase struct {
	ID     string `json:"id"`
	Source string `json:"source"`
}
type input struct {
	Sources []sourceCase `json:"sources"`
}
type typeFact struct {
	Name       string   `json:"name"`
	Alias      bool     `json:"alias"`
	Underlying string   `json:"underlying"`
	Fields     []string `json:"fields"`
	ValueError bool     `json:"value_error"`
	PtrError   bool     `json:"pointer_error"`
}
type callFact struct {
	Offset int    `json:"offset"`
	Kind   string `json:"kind"`
	Target string `json:"target"`
}
type sourceObservation struct {
	ID       string     `json:"id"`
	Status   string     `json:"status"`
	Errors   []string   `json:"errors"`
	Types    []typeFact `json:"types"`
	Calls    []callFact `json:"calls"`
	IntBits  int64      `json:"int_bits"`
	UintBits int64      `json:"uint_bits"`
}

func inspectSource(r sourceCase) sourceObservation {
	fset := token.NewFileSet()
	file, err := parser.ParseFile(fset, r.ID+".go", r.Source, 0)
	a := sourceObservation{ID: r.ID, Status: "ok", Errors: []string{}, Types: []typeFact{}, Calls: []callFact{}}
	if err != nil {
		a.Status, a.Errors = "source_parse_error", []string{err.Error()}
		return a
	}
	sizes := types.SizesFor("gc", "amd64")
	a.IntBits, a.UintBits = sizes.Sizeof(types.Typ[types.Int])*8, sizes.Sizeof(types.Typ[types.Uint])*8
	info := &types.Info{Types: map[ast.Expr]types.TypeAndValue{}, Uses: map[*ast.Ident]types.Object{}, Defs: map[*ast.Ident]types.Object{}, Selections: map[*ast.SelectorExpr]*types.Selection{}}
	config := types.Config{Sizes: sizes, GoVersion: "go1.25.0", Error: func(e error) { a.Errors = append(a.Errors, e.Error()) }}
	pkg, err := config.Check("mpk.private.w04/"+r.ID, fset, []*ast.File{file}, info)
	if err != nil {
		a.Status = "source_type_error"
		return a
	}
	errorInterface := types.Universe.Lookup("error").Type().Underlying().(*types.Interface)
	for _, name := range pkg.Scope().Names() {
		object, ok := pkg.Scope().Lookup(name).(*types.TypeName)
		if !ok {
			continue
		}
		t := object.Type()
		f := typeFact{Name: name, Alias: object.IsAlias(), Underlying: t.Underlying().String(), Fields: []string{}, ValueError: types.Implements(t, errorInterface), PtrError: types.Implements(types.NewPointer(t), errorInterface)}
		if s, ok := t.Underlying().(*types.Struct); ok {
			for i := 0; i < s.NumFields(); i++ {
				f.Fields = append(f.Fields, s.Field(i).Name()+":"+s.Field(i).Type().String())
			}
		}
		a.Types = append(a.Types, f)
	}
	ast.Inspect(file, func(n ast.Node) bool {
		call, ok := n.(*ast.CallExpr)
		if !ok {
			return true
		}
		fact := callFact{Offset: fset.Position(call.Pos()).Offset, Kind: "unknown", Target: ""}
		if tv := info.Types[call.Fun]; tv.IsType() {
			fact.Kind, fact.Target = "conversion", tv.Type.String()
		}
		switch f := call.Fun.(type) {
		case *ast.Ident:
			switch o := info.Uses[f].(type) {
			case *types.Func:
				fact.Kind, fact.Target = "static_function", o.FullName()
			case *types.Builtin:
				fact.Kind, fact.Target = "builtin", o.Name()
			}
		case *ast.SelectorExpr:
			if sel := info.Selections[f]; sel != nil {
				fact.Kind, fact.Target = "direct_method", sel.Obj().(*types.Func).FullName()
				if _, ok := sel.Recv().Underlying().(*types.Interface); ok {
					fact.Kind = "interface_invoke"
				}
			}
		}
		a.Calls = append(a.Calls, fact)
		return true
	})
	sort.Slice(a.Calls, func(i, j int) bool { return a.Calls[i].Offset < a.Calls[j].Offset })
	return a
}

type line struct {
	Amount int64
	SKU    string
}
type rows []line
type rootValue struct {
	Lines  rows
	Nested [2]rows
}
type code int64

func (code) Error() string { return "code" }

type flag bool

func (flag) Error() string { return "flag" }

type sliceError struct{ Values []int64 }

func (sliceError) Error() string { return "slice" }

type countedError int64

var errorCalls int

func (countedError) Error() string { errorCalls++; return "counted" }

func recovered(f func()) (panicked bool) {
	defer func() {
		if recover() != nil {
			panicked = true
		}
	}()
	f()
	return false
}
func observeRuntime() map[string]any {
	backing := rows{{Amount: -1, SKU: "\x80\x00"}, {Amount: 7, SKU: "x"}, {Amount: 11, SKU: "y"}}
	r := rootValue{Lines: backing[:1:3], Nested: [2]rows{backing[1:2:3], backing[:0:0]}}
	copyOfRoot := r
	copyOfRoot.Lines[0].Amount = 19
	rootSharing := r.Lines[0].Amount == 19 && backing[0].Amount == 19
	copyOfRoot.Lines = rows{{Amount: 23}}
	rebindingPreservesRoot := r.Lines[0].Amount == 19 && len(copyOfRoot.Lines) == 1
	arrayCopy := r.Nested
	arrayCopy[0][0].Amount = 31
	arraySharing := r.Nested[0][0].Amount == 31

	var nilRows rows
	zeroMade := make(rows, 0, 0)
	zeroView := backing[:0:0]
	zeroTail := backing[len(backing):len(backing):len(backing)]
	before := append(rows(nil), backing...)
	zeroObservations := []map[string]any{}
	for i, s := range []rows{nilRows, zeroMade, zeroView, zeroTail} {
		copyCount := copy(s, backing)
		a := append(s, rows{}...)
		resliced := s[:0:0]
		zeroObservations = append(zeroObservations, map[string]any{"kind": i, "nil": s == nil, "length": len(s), "capacity": cap(s), "copy_count": copyCount, "append_empty_nil": a == nil, "reslice_nil": resliced == nil, "index_panicked": recovered(func() { _ = s[0] }), "positive_reslice_panicked": recovered(func() { _ = s[:1] }), "named_append_type": reflect.TypeOf(a).Name()})
	}
	noWrites := reflect.DeepEqual(before, backing)
	zeroSourceCopyPreserved := copy(backing, zeroMade) == 0 && reflect.DeepEqual(before, backing)
	positiveEmpty := backing[:0:len(backing)]
	expanded := positiveEmpty[:cap(positiveEmpty)]
	positiveEmptyExposesTail := len(positiveEmpty) == 0 && len(expanded) == len(backing) && reflect.DeepEqual(expanded, backing)
	expanded[2].Amount = 47
	positiveEmptyShares := backing[2].Amount == 47
	grown := append(zeroMade, line{Amount: 41})
	zeroAppendSucceeded := len(grown) == 1 && grown[0].Amount == 41

	errorCalls = 0
	var boxed error = countedError(0)
	conversionCalls := errorCalls
	boxedNonNil := boxed != nil
	message := boxed.Error()
	invocationCalls := errorCalls
	var typedNil *code
	var pointerError error = typedNil
	valueError := error(code(0))
	var nilError error
	var noncomparable error = sliceError{Values: nil}
	comparisonPanicked := recovered(func() { _ = noncomparable == noncomparable })
	pointerCallPanicked := recovered(func() { _ = pointerError.Error() })
	closedMessages := []string{}
	for _, e := range []error{code(0), flag(false), sliceError{}} {
		closedMessages = append(closedMessages, e.Error())
	}
	return map[string]any{"root_copy_shares_backing": rootSharing, "root_descriptor_rebind_preserves_original": rebindingPreservesRoot, "root_array_copy_shares_referenced_backing": arraySharing, "zero_slices": zeroObservations, "zero_operations_preserve_all_backing": noWrites, "zero_source_copy_preserves_all_backing": zeroSourceCopyPreserved, "positive_capacity_empty_exposes_full_tail": positiveEmptyExposesTail, "positive_capacity_empty_shares_backing": positiveEmptyShares, "positive_append_to_zero_succeeds_in_go": zeroAppendSucceeded, "conversion_error_body_calls": conversionCalls, "invocation_error_body_calls": invocationCalls, "boxed_zero_non_nil": boxedNonNil, "counted_message": message, "typed_nil_interface_is_nil": pointerError == nil, "typed_nil_method_panics": pointerCallPanicked, "zero_value_error_is_nil": valueError == nil, "nil_interface_is_nil": nilError == nil, "slice_error_is_nil": noncomparable == nil, "error_comparison_panics": comparisonPanicked, "closed_messages": closedMessages, "extreme_amount_text": strconv.FormatInt(-1<<63, 10)}
}
func main() {
	f := os.Stdin
	if len(os.Args) == 2 {
		p, e := os.Open(os.Args[1])
		if e != nil {
			panic(e)
		}
		defer p.Close()
		f = p
	}
	var in input
	if e := json.NewDecoder(f).Decode(&in); e != nil {
		panic(e)
	}
	out := make([]sourceObservation, len(in.Sources))
	for i, r := range in.Sources {
		out[i] = inspectSource(r)
	}
	if e := json.NewEncoder(os.Stdout).Encode(map[string]any{"go_version": runtime.Version(), "target_sizes": "gc/amd64", "sources": out, "runtime": observeRuntime()}); e != nil {
		panic(e)
	}
}
