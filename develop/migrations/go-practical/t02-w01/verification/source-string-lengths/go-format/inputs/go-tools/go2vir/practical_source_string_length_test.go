//go:build go1.25 && (linux || darwin)

package main

import (
	"go/ast"
	"go/types"
 "fmt"
 "strings"
	"testing"
)

// Native observations remain comparison data. The Rust constructor consumes
// original source buffers and derives its own complete value/error domain.
func TestPracticalSourceStringLengthOriginalTypes(t *testing.T) {
	files := map[string][]byte{
		"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"),
		"helper/helper.go": []byte(`package helper
type Remote string
`),
		"policy.go": []byte(`package policy
import "example.com/policy/helper"
type Named string
type Chained Named
type Alias = Named
type Name struct { Value Named }
type Chain struct { Value Chained }
type Text struct { Plain string; Named Named; Chain Chained; Imported helper.Remote; Empty [0]bool; Flag bool }
type Code int64
func(Name)Error()string{return ""}
func(Chain)Error()string{return ""}
func(Text)Error()string{return ""}
func(Code)Error()string{return ""}
func Entry(e error)string{return e.Error()}
func Length(s string,n Named,c Chained,h helper.Remote)int{return len(s)+len(n)+len(c)+len(h)}
`),
	}
 var extra strings.Builder
 for i:=0;i<70;i++{fmt.Fprintf(&extra,"type Wide%02d int64\nfunc(Wide%02d)Error()string{return \"\"}\n",i,i)}
 files["policy.go"]=append(files["policy.go"],[]byte(extra.String())...)

	owner, _ := practicalStorageTestOwner(t, files)
	defer owner.Close()
	retainPracticalSyntaxEvidence(t, "dispatch-inputs.json", practicalBoxingInput(t, owner))
	declarations := map[string]any{}
	for path, loaded := range owner.loader.packages {
		if path != "example.com/policy" && path != "example.com/policy/helper" {
			continue
		}
		for _, file := range loaded.files {
			for _, decl := range file.Decls {
				if g, ok := decl.(*ast.GenDecl); ok {
					for _, spec := range g.Specs {
						if ts, ok := spec.(*ast.TypeSpec); ok {
							span, _, err := owner.loader.syntax.spanBytes(ts)
							if err != nil {
								t.Fatal(err)
							}
							declarations[path+"."+ts.Name.Name] = map[string]any{"source": span.sourceID, "start": span.start, "end": span.end}
						}
					}
				}
			}
		}
	}
	retainPracticalSyntaxEvidence(t, "dispatch-declarations.json", declarations)
	root := practicalClosureTestRef(t, owner, "example.com/policy", "Entry")
 lengthRoot:=practicalClosureTestRef(t,owner,"example.com/policy","Length")
 wide:=[]string{};for i:=0;i<70;i++{wide=append(wide,fmt.Sprintf("Wide%02d",i))}
 cases:=[][]string{{},{"Name","Chain","Text","Code"},wide}
 caseNames:=[]string{"source-length-empty","source-length-small","source-length-wide"}
 for index,names:=range cases{
  entry:=[]types.Type{}
  for _,name:=range names{entry=append(entry,owner.loader.packages["example.com/policy"].types.Scope().Lookup(name).Type())}
  retainPracticalSyntaxEvidence(t,fmt.Sprintf("dispatch-%02d-input.json",index),map[string]any{"name":caseNames[index],"roots":[]any{map[string]any{"name":"Entry","ref":practicalDispatchRef(root)},map[string]any{"name":"Length","ref":practicalDispatchRef(lengthRoot)}},"entry_cases":names,"expected_rejection":""})
  retainPracticalSyntaxEvidence(t,fmt.Sprintf("dispatch-%02d-actual.json",index),practicalNativeErrorClosure(t,owner,[]practicalClosureRef{root,lengthRoot},entry))
 }
 observations:=[]any{}
 for _,file:=range owner.loader.packages["example.com/policy"].files{
  for _,decl:=range file.Decls{
   function,ok:=decl.(*ast.FuncDecl);if !ok||function.Name.Name!="Length"{continue}
   ast.Inspect(function.Body,func(n ast.Node)bool{
    call,ok:=n.(*ast.CallExpr);if !ok{return true}
    ident,ok:=call.Fun.(*ast.Ident);if !ok||ident.Name!="len"{return true}
    info:=owner.loader.packages["example.com/policy"].info
    if !types.Identical(info.TypeOf(call),types.Typ[types.Int])||info.Types[call].Value!=nil{t.Fatalf("not actual dynamic int len: %v",info.Types[call])}
    span,_,err:=owner.loader.syntax.spanBytes(call);if err!=nil{t.Fatal(err)}
    observations=append(observations,map[string]any{"source":span.sourceID,"start":span.start,"end":span.end,"operand_type":types.TypeString(info.TypeOf(call.Args[0]),nil),"result_type":"int","constant":false})
    return true
   })
  }
 }
 if len(observations)!=4{t.Fatalf("incomplete length observations: %d",len(observations))}
 retainPracticalSyntaxEvidence(t,"length-call-types.json",observations)
 retainPracticalSyntaxEvidence(t,"dispatch-controls.json",map[string]any{"cases":3,"native_dynamic_len_calls":4,"error_cases":[]int{0,4,70},"source_runtime_proof":false})
}
