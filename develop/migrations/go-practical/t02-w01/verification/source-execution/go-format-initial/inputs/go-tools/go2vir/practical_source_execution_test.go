//go:build go1.25 && (linux || darwin)

package main

import (
 "bytes"
 "encoding/json"
 "fmt"
 "go/ast"
 "go/types"
 "os"
 "os/exec"
 "path/filepath"
 "testing"
)

func TestPracticalSourceExecutionOriginalValues(t *testing.T) {
 evidence:=filepath.Join("..","..","develop","migrations","go-practical","t02-w01","verification","source-execution")
 read:=func(name string)[]byte{raw,err:=os.ReadFile(filepath.Join(evidence,name));if err!=nil{t.Fatal(err)};return raw}
 files:=map[string][]byte{
  "go.mod":[]byte("module example.com/policy\ngo 1.25.0\n"),
  "policy.go":read("program.go"),
  "helper/helper.go":[]byte("package helper\ntype Remote string\nfunc Text()Remote{return \"abc\"}\n"),
  "shadow/shadow.go":[]byte("package shadow\nfunc len(s string)int{return 1}\nfunc Length(s string)int{return len(s)}\n"),
 }
 owner,_:=practicalStorageTestOwner(t,files);defer owner.Close()
 retainPracticalSyntaxEvidence(t,"unit-inputs.json",practicalStringOperationInput(t,owner))
 roots:=[]practicalClosureRef{};rootRows:=[]any{};caseRows:=[]any{};entry:=[]types.Type{};declarations:=map[string]any{};var entryRow any
 for _,file:=range owner.loader.packages["example.com/policy"].files{
  for _,decl:=range file.Decls{
   if f,ok:=decl.(*ast.FuncDecl);ok&&f.Recv==nil{
    r:=practicalClosureTestRef(t,owner,"example.com/policy",f.Name.Name);roots=append(roots,r);rootRows=append(rootRows,practicalDispatchRef(r));if f.Name.Name=="Entry"{entryRow=practicalDispatchRef(r)}
   }
   if g,ok:=decl.(*ast.GenDecl);ok{for _,s:=range g.Specs{if ts,ok:=s.(*ast.TypeSpec);ok{
    span,_,err:=owner.loader.syntax.spanBytes(ts);if err!=nil{t.Fatal(err)};declarations["example.com/policy."+ts.Name.Name]=map[string]any{"source":span.sourceID,"start":span.start,"end":span.end}
   }}}
  }
 }
 for i:=0;i<70;i++{name:=fmt.Sprintf("Case%02d",i);caseRows=append(caseRows,declarations["example.com/policy."+name]);entry=append(entry,owner.loader.packages["example.com/policy"].types.Scope().Lookup(name).Type())}
 retainPracticalSyntaxEvidence(t,"unit-selection.json",map[string]any{"roots":rootRows,"cases":caseRows,"entry":entryRow,"declarations":declarations})
 retainPracticalSyntaxEvidence(t,"native-closure.json",practicalNativeErrorClosure(t,owner,roots,entry))
 // Execute fresh exact original application files. The observer is a separate
 // importing package, never a rewrite or stub of the selected application.
 directory:=t.TempDir();if out:=os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE");out!=""{directory=filepath.Join(out,"runtime-inputs");if err:=os.Mkdir(directory,0700);err!=nil{t.Fatal(err)}}
 complete:=map[string][]byte{}
 for name,raw:=range files{complete[name]=bytes.Clone(raw)}
 complete["cmd/observe/main.go"]=read("observer.go.txt");complete["vectors.json"]=read("vectors.json")
 for name,raw:=range complete{path:=filepath.Join(directory,name);if err:=os.MkdirAll(filepath.Dir(path),0700);err!=nil{t.Fatal(err)};if err:=os.WriteFile(path,raw,0600);err!=nil{t.Fatal(err)}}
 compiler:=practicalPackageTestCompiler(t)
 command:=exec.Command(filepath.Join(compiler.selection.GoRoot,"bin","go"),"run","./cmd/observe","vectors.json");command.Dir=directory
 command.Env=[]string{"PATH=/usr/bin:/bin","GOROOT="+compiler.selection.GoRoot,"GOTOOLCHAIN=local","GOENV=off","GOWORK=off","GOFLAGS=-mod=vendor","GOPROXY=off","GOSUMDB=off","CGO_ENABLED=0","GOMAXPROCS=1","GOCACHE="+os.Getenv("GOCACHE"),"GOMODCACHE="+os.Getenv("GOMODCACHE")}
 retainPracticalSyntaxEvidence(t,"runtime-prelaunch.json",map[string]any{"command":command.Args,"directory":directory,"environment":command.Env,"whole_original_files":complete,"compiler_sha256":owner.compilerDigest})
 var stdout,stderr bytes.Buffer;command.Stdout=&stdout;command.Stderr=&stderr;err:=command.Run()
 retainPracticalSyntaxEvidence(t,"runtime-result.json",map[string]any{"stdout":stdout.Bytes(),"stderr":stderr.Bytes(),"error":fmt.Sprint(err)})
 if err!=nil{t.Fatal(err,stderr.String())}
 if stderr.Len()!=0{t.Fatal(stderr.String())}
 var observations []map[string]any;if err:=json.Unmarshal(stdout.Bytes(),&observations);err!=nil{t.Fatal(err)}
 if len(observations)!=127{t.Fatal("incomplete native execution matrix",len(observations))}
 for _,row:=range observations{if row["fault"]=="unknown"{t.Fatal("unclassified native panic",row)}}
 retainPracticalSyntaxEvidence(t,"native-values.json",observations)
 for name,raw:=range complete{actual,err:=os.ReadFile(filepath.Join(directory,name));if err!=nil||!bytes.Equal(actual,raw){t.Fatal("native execution changed original source",name,err)}}
}
