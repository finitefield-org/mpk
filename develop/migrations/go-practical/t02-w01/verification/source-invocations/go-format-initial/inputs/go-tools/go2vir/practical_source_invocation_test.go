//go:build go1.25 && (linux || darwin)

package main

import (
 "fmt"
 "go/ast"
 "go/types"
 "os"
 "path/filepath"
 "testing"
)

func TestPracticalSourceInvocationOriginalSignatures(t *testing.T){
 root:=filepath.Join("..","..","develop","migrations","go-practical","t02-w01","verification")
 read:=func(path string)[]byte{b,err:=os.ReadFile(filepath.Join(root,path));if err!=nil{t.Fatal(err)};return b}
 for _,name:=range []string{"pure","wide"}{t.Run(name,func(t *testing.T){
  if out:=os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE");out!=""{out=filepath.Join(out,name);if err:=os.Mkdir(out,0700);err!=nil{t.Fatal(err)};t.Setenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE",out)}
  policy:=append(read("source-execution/program.go"),read("source-invocations/additions.go.txt")...)
  owner,_:=practicalStorageTestOwner(t,map[string][]byte{"go.mod":[]byte("module example.com/policy\ngo 1.25.0\n"),"policy.go":policy,"helper/helper.go":[]byte("package helper\ntype Remote string\nfunc Text()Remote{return \"abc\"}\n"),"shadow/shadow.go":[]byte("package shadow\nfunc len(s string)int{return 1}\nfunc Length(s string)int{return len(s)}\n")});defer owner.Close()
  retainPracticalSyntaxEvidence(t,"unit-inputs.json",practicalStringOperationInput(t,owner))
  roots:=[]practicalClosureRef{};rootRows:=[]any{};cases:=[]any{};entry:=[]types.Type{};var entryRow any;declarations:=map[string]any{}
  for _,file:=range owner.loader.packages["example.com/policy"].files{for _,decl:=range file.Decls{
   if f,ok:=decl.(*ast.FuncDecl);ok&&f.Recv==nil{r:=practicalClosureTestRef(t,owner,"example.com/policy",f.Name.Name);roots=append(roots,r);rootRows=append(rootRows,practicalDispatchRef(r));if f.Name.Name=="Entry"{entryRow=practicalDispatchRef(r)}}
   if g,ok:=decl.(*ast.GenDecl);ok{for _,s:=range g.Specs{if ts,ok:=s.(*ast.TypeSpec);ok{span,_,err:=owner.loader.syntax.spanBytes(ts);if err!=nil{t.Fatal(err)};declarations["example.com/policy."+ts.Name.Name]=map[string]any{"source":span.sourceID,"start":span.start,"end":span.end}}}}
  }}
  if name=="wide"{for i:=0;i<70;i++{n:=fmt.Sprintf("Case%02d",i);cases=append(cases,declarations["example.com/policy."+n]);entry=append(entry,owner.loader.packages["example.com/policy"].types.Scope().Lookup(n).Type())}}
  retainPracticalSyntaxEvidence(t,"unit-selection.json",map[string]any{"name":name,"roots":rootRows,"cases":cases,"entry":entryRow,"declarations":declarations})
  closure:=practicalNativeErrorClosure(t,owner,roots,entry);retainPracticalSyntaxEvidence(t,"native-closure.json",closure)
  bodies:=map[string]bool{};for _,r:=range closure["bodies"].([]map[string]any){bodies[fmt.Sprint(r["source"],":",r["start"],":",r["end"])]=true}
  // Only calls in the independently selected complete native body set. The
  // Go type facts below are comparison output, never Rust constructor input.
  observations:=[]any{}
  for _,loaded:=range owner.loader.packages{for _,file:=range loaded.files{for _,decl:=range file.Decls{
   f,ok:=decl.(*ast.FuncDecl);if !ok{continue};caller,_,err:=owner.loader.syntax.spanBytes(f);if err!=nil{t.Fatal(err)};if !bodies[fmt.Sprint(caller.sourceID,":",caller.start,":",caller.end)]{continue}
   ast.Inspect(f.Body,func(n ast.Node)bool{
    call,ok:=n.(*ast.CallExpr);if !ok{return true};object,selected:=practicalClosureFunction(loaded.info,call.Fun);if _,ok:=object.(*types.Func);!ok{return true}
    signature,ok:=loaded.info.TypeOf(call.Fun).(*types.Signature);if !ok{t.Fatal("native call signature missing")}
    inputTypes:=[]types.Type{};groups:=[]struct{node ast.Expr;receiver bool}{}
    if selected!=nil&&selected.Kind()==types.MethodVal{receiver:=practicalClosureUnparen(call.Fun).(*ast.SelectorExpr).X;inputTypes=append(inputTypes,loaded.info.TypeOf(receiver));groups=append(groups,struct{node ast.Expr;receiver bool}{receiver,true})}
    for i:=0;i<signature.Params().Len();i++{inputTypes=append(inputTypes,signature.Params().At(i).Type())}
    for i,a:=range call.Args{groups=append(groups,struct{node ast.Expr;receiver bool}{a,selected!=nil&&selected.Kind()==types.MethodExpr&&i==0})}
    inputs:=[]string{};results:=[]string{};for _,ty:=range inputTypes{inputs=append(inputs,practicalNativeBoxType(t,owner,ty))};for i:=0;i<signature.Results().Len();i++{results=append(results,practicalNativeBoxType(t,owner,signature.Results().At(i).Type()))}
    arguments:=[]any{};position:=0
    for _,g:=range groups{span,_,err:=owner.loader.syntax.spanBytes(g.node);if err!=nil{t.Fatal(err)};ty:=loaded.info.TypeOf(g.node);typesList:=[]types.Type{ty};if tuple,ok:=ty.(*types.Tuple);ok{typesList=nil;for i:=0;i<tuple.Len();i++{typesList=append(typesList,tuple.At(i).Type())}}
     values:=[]any{};for i,ty:=range typesList{nilValue:=loaded.info.Types[g.node].IsNil();if basic,ok:=ty.(*types.Basic);ok&&(basic.Info()&types.IsUntyped)!=0{ty=inputTypes[position]};values=append(values,map[string]any{"projection":i,"parameter":position,"source_type":practicalNativeBoxType(t,owner,ty),"parameter_type":inputs[position],"nil":nilValue});position++}
     arguments=append(arguments,map[string]any{"start":span.start,"end":span.end,"receiver":g.receiver,"values":values})
    }
    span,_,err:=owner.loader.syntax.spanBytes(call);if err!=nil{t.Fatal(err)};observations=append(observations,map[string]any{"source":span.sourceID,"start":span.start,"end":span.end,"caller_start":caller.start,"caller_end":caller.end,"inputs":inputs,"results":results,"arguments":arguments});return true
   })
  }}}
  retainPracticalSyntaxEvidence(t,"native-calls.json",observations)
 })}
}
