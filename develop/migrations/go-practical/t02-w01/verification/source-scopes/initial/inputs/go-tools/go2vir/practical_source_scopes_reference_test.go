//go:build go1.25 && (linux || darwin)

package main

import (
 "bytes"
 "go/ast"
 "go/types"
 "sort"
 "testing"
)

// Original compiler facts are reference observations. The Rust consumer must
// reconstruct lexical scopes from whole bytes rather than import these maps.
func TestPracticalOriginalSourceScopesReference(t *testing.T) {
 files:=map[string][]byte{
  "go.mod":[]byte("module example.com/policy\ngo 1.25.0\n"),
  "policy.go":[]byte(`package policy
import (
 alias "example.com/policy/helper"
 "example.com/policy/oddpath"
)
type Root struct { A alias.Record; B domain.Record }
type Amount uint16
const ( First = iota; Second; Third = Second + 1 )
var Shared Amount
func Probe(root Root, x uint16) (out uint16) {
 _ = root
 const Outer = 7
 type Local = Amount
 { const Outer = 9; var int = Outer; _ = int; var typed Local; _ = typed }
 x, fresh := x+1, Outer
 _ = fresh
 if x := x+1; x > 0 { _ = x } else if y := x; y > 0 { _ = y }
 for i:=0; i<2; i++ { _=i; const Inner=1; _=Inner }
 for k,v := range []uint16{1,2} { _=k; _=v }
 switch x:=x; x { case 1: type Case=Amount; var c Case; _=c; default: const C=8; _=C }
 switch z:=any(root).(type) { case Root: _=z; default: _=z }
 select { case v,ok := <- make(chan uint16): _=v; _=ok; default: }
 fn := func(c uint16) (r uint16) { return c }; _=fn
 _=Shared; _=OtherFile; _=alias.Value; _=domain.Value; _=Third
 return x
}
type Box[T any] struct{ V T }
func (b Box[U]) Method(v U) U { var out U = v; return out }
func Generic[T any](x T) T { return x }
func init(){panic("application must not execute")}
`),
 "other.go":[]byte("package policy\r\n//line imaginary.go:900\r\nconst OtherFile = 4\r\ntype É uint16\r\nfunc Unicode(é É) É {return é}\r\n"),
 "dot.go":[]byte("package policy\nimport . `example.com/policy/helper`\nfunc Dot(){var record Record; _=record; _=Value; _=Call()}\n"),
 "helper/helper.go":[]byte("package helper\ntype Record struct{Value uint16}\nconst Value=5\nfunc Call() uint16{return Value}\n"),
 "oddpath/odd.go":[]byte("package domain\ntype Record struct{Value uint16}\nconst Value=6\n"),
 }
 owner,original:=practicalStorageTestOwner(t,files)
 source:=original["policy.go"]
 keys,err:=owner.typeKeys("example.com/policy",practicalStorageTestSpan(t,source,"root"))
 if err!=nil {t.Fatal(err)}
 buffers:=[]map[string]any{}
 for _,b:=range keys.sources {buffers=append(buffers,map[string]any{"object":b.object,"whole_original_bytes":b.sourceBytes()})}
 retainPracticalSyntaxEvidence(t,"scope-components.json",map[string]any{"canonical_components":bytes.Clone(keys.canonical),"sources":buffers,"source_typing_or_context_or_program_authority":false})
 owner.mutex.Lock()
 defer owner.mutex.Unlock()
 observations:=[]map[string]any{}
 for _,path:=range []string{"example.com/policy","example.com/policy/helper","example.com/policy/oddpath"} {
  pkg:=owner.loader.packages[path]
  if pkg==nil {t.Fatal("missing original package",path)}
  nonlexical:=map[*ast.Ident]bool{}
  for _,file:=range pkg.files {ast.Inspect(file,func(n ast.Node)bool{
   switch n:=n.(type){case *ast.SelectorExpr:nonlexical[n.Sel]=true;case *ast.BranchStmt:if n.Label!=nil {nonlexical[n.Label]=true}}
   return true
  })}
  for ident,obj:=range pkg.info.Uses {
   if nonlexical[ident] {continue}
   if field,ok:=obj.(*types.Var);ok&&field.IsField(){continue}
   if _,ok:=obj.(*types.Label);ok{continue}
   kind:=""
   switch obj:=obj.(type){
    case *types.TypeName:kind="type";if _,ok:=obj.Type().(*types.TypeParam);ok {kind="type_parameter"}
    case *types.Const:kind="constant"
    case *types.Var:kind="variable"
    case *types.Func:kind="function"
    case *types.PkgName:kind="import"
    case *types.Builtin:kind="builtin_function"
    case *types.Nil:kind="builtin_nil"
    default:t.Fatalf("unhandled original object %T",obj)
   }
   use,_,err:=owner.loader.syntax.spanBytes(ident)
   if err!=nil {t.Fatal(err)}
   row:=map[string]any{"source_object_id":use.sourceID,"start_byte":use.start,"end_byte":use.end,"name":ident.Name,"kind":kind,"definition_source_object_id":nil,"definition_start_byte":nil}
   if obj.Pkg()==nil {
    switch kind {case "type":row["kind"]="builtin_type";case "constant":row["kind"]="builtin_constant"}
   } else {
    found:=false
    for _,parsed:=range owner.loader.syntax.files {
     file:=owner.loader.syntax.fileSet.File(obj.Pos())
     if file!=nil&&owner.loader.syntax.fileSet.File(parsed.file.Pos())==file {row["definition_source_object_id"]=parsed.source.object.ID;row["definition_start_byte"]=file.Offset(obj.Pos());found=true;break}
    }
    if !found {t.Fatal("original definition location missing",ident.Name,obj.Pos())}
   }
   observations=append(observations,row)
  }
 }
 sort.Slice(observations,func(i,j int)bool{
  a,b:=observations[i],observations[j];if a["source_object_id"]!=b["source_object_id"] {return a["source_object_id"].(string)<b["source_object_id"].(string)}
  return a["start_byte"].(int)<b["start_byte"].(int)
 })
 if len(observations)<100 {t.Fatal("incomplete lexical observation",len(observations))}
 retainPracticalSyntaxEvidence(t,"scope-uses.json",map[string]any{"uses":observations,"source_typing_or_context_or_program_authority":false})
 t.Logf("original lexical uses %d",len(observations))
}
