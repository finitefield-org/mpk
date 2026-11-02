//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"reflect"
	"runtime"
	"sort"
	"strings"
	"testing"
)

// Canonical preorder with alphabetic field roles, preserving each list order.
// Object resolution, comments and derived File imports/version metadata are
// separate obligations. Physical spans use the original-token corrections.
func practicalParseReference(t *testing.T,source practicalSourceBuffer)(map[string]any){
	t.Helper();set:=token.NewFileSet();file,err:=parser.ParseFile(set,"original.go",source.data,parser.AllErrors|parser.SkipObjectResolution)
	if err!=nil{return map[string]any{"accepted":false,"error":err.Error(),"source_typing_or_program_authority":false}}
	physical:=set.File(file.Pos());ends,err:=practicalPhysicalTokenEnds(physical,source.data);if err!=nil{t.Fatal(err)}
	type edge struct{Role string `json:"role"`;Targets []int `json:"targets"`}
	type node struct{Kind string `json:"kind"`;Start int `json:"start"`;End int `json:"end"`;Attrs map[string]string `json:"attrs"`;Edges []edge `json:"edges"`}
	type pending struct{kind string; children map[string][]ast.Node}
	getChildren:=func(n ast.Node)map[string][]ast.Node{
		result:=map[string][]ast.Node{};v:=reflect.ValueOf(n).Elem();typ:=v.Type()
		for i:=0;i<v.NumField();i++{
			name:=typ.Field(i).Name
			switch name{case "Doc","Comment","Comments","Obj","Scope","Unresolved","Imports":continue}
			value:=v.Field(i);add:=func(value reflect.Value){if !value.IsValid()||!value.CanInterface(){return};if value.Kind()==reflect.Interface&&value.IsNil()||value.Kind()==reflect.Pointer&&value.IsNil(){return};if child,ok:=value.Interface().(ast.Node);ok{result[name]=append(result[name],child)}}
			if value.Kind()==reflect.Slice{for j:=0;j<value.Len();j++{add(value.Index(j))}}else{add(value)}
		};return result
	}
	seen:=map[ast.Node]int{};order:=[]ast.Node{};todo:=[]ast.Node{file};children:=map[ast.Node]map[string][]ast.Node{}
	for len(todo)>0{n:=todo[len(todo)-1];todo=todo[:len(todo)-1];if _,ok:=seen[n];ok{continue};seen[n]=len(order);order=append(order,n);child:=getChildren(n);children[n]=child;roles:=make([]string,0,len(child));for role:=range child{roles=append(roles,role)};sort.Strings(roles);for i:=len(roles)-1;i>=0;i--{list:=child[roles[i]];for j:=len(list)-1;j>=0;j--{todo=append(todo,list[j])}}}
	rows:=make([]node,0,len(order))
	for _,n:=range order{
		span,err:=practicalPhysicalNodeSpan(physical,source,n,ends);if err!=nil{t.Fatal(err)};attrs:=map[string]string{}
		switch n:=n.(type){
		case *ast.Ident:attrs["Name"]=n.Name
		case *ast.BasicLit:attrs["Kind"]=n.Kind.String();attrs["ValueHex"]=hex.EncodeToString([]byte(n.Value))
		case *ast.GenDecl:attrs["Tok"]=n.Tok.String();attrs["Grouped"]=fmt.Sprint(n.Lparen.IsValid())
		case *ast.TypeSpec:attrs["Alias"]=fmt.Sprint(n.Assign.IsValid())
		case *ast.UnaryExpr:attrs["Op"]=n.Op.String()
		case *ast.BinaryExpr:attrs["Op"]=n.Op.String()
		case *ast.ChanType:attrs["Dir"]=fmt.Sprint(int(n.Dir))
		case *ast.StructType:attrs["Incomplete"]=fmt.Sprint(n.Incomplete)
		case *ast.InterfaceType:attrs["Incomplete"]=fmt.Sprint(n.Incomplete)
		case *ast.CompositeLit:attrs["Incomplete"]=fmt.Sprint(n.Incomplete)
		case *ast.CallExpr:attrs["Ellipsis"]=fmt.Sprint(n.Ellipsis.IsValid())
		case *ast.SliceExpr:attrs["Slice3"]=fmt.Sprint(n.Slice3)
		case *ast.IncDecStmt:attrs["Tok"]=n.Tok.String()
		case *ast.AssignStmt:attrs["Tok"]=n.Tok.String()
		case *ast.BranchStmt:attrs["Tok"]=n.Tok.String()
		case *ast.EmptyStmt:attrs["Implicit"]=fmt.Sprint(n.Implicit)
		case *ast.RangeStmt:attrs["Tok"]=n.Tok.String()
		}
		child:=children[n];roles:=make([]string,0,len(child));for role:=range child{roles=append(roles,role)};sort.Strings(roles);edges:=[]edge{};for _,role:=range roles{ids:=[]int{};for _,target:=range child[role]{ids=append(ids,seen[target])};edges=append(edges,edge{role,ids})}
		rows=append(rows,node{reflect.TypeOf(n).Elem().Name(),span.start,span.end,attrs,edges})
	}
	return map[string]any{"accepted":true,"nodes":rows,"root":0,"source_typing_or_program_authority":false}
}

func TestPracticalParseOriginalGoReference(t *testing.T){
	if runtime.Version()!="go1.25.0"{t.Fatal("exact pinned parser required")}
	input,output:=os.Getenv("MPK_GO_PARSE_INPUT"),os.Getenv("MPK_GO_PARSE_OUTPUT");full:=input!="";if output==""{output=t.TempDir()}
	write:=func(p string,raw []byte){t.Helper();if err:=os.MkdirAll(filepath.Dir(p),0o700);err!=nil{t.Fatal(err)};f,err:=os.OpenFile(p,os.O_WRONLY|os.O_CREATE|os.O_EXCL,0o600);if err!=nil{t.Fatal(err)};_,e:=f.Write(raw);c:=f.Close();if e!=nil||c!=nil{t.Fatal(e,c)}}
	read:=func(p string)[]byte{t.Helper();raw,err:=os.ReadFile(p);if err!=nil{t.Fatal(err)};return raw}
	type entry struct{label string;source practicalSourceBuffer}
	cases:=[]entry{}
	if full{
		var manifest struct{Sources []string `json:"sources"`};if err:=json.Unmarshal(read(filepath.Join(input,"manifest.json")),&manifest);err!=nil{t.Fatal(err)}
		for _,id:=range manifest.Sources{source,err:=decodePracticalSourceBuffer(read(filepath.Join(input,"sources",id+".json")),read(filepath.Join(input,"sources",id+".raw")));if err!=nil||source.object.ID!=id{t.Fatal("original source membership",err)};cases=append(cases,entry{"original/"+id,source})}
	}
	add:=func(label string,raw []byte){t.Helper();source,err:=newPracticalSourceBuffer("parse-cases",label+".go","go_file",raw);if err!=nil{t.Fatal(err)};cases=append(cases,entry{label,source})}
	testdata:=filepath.Join(runtime.GOROOT(),"src/go/parser/testdata");err:=filepath.WalkDir(testdata,func(p string,d os.DirEntry,err error)error{if err!=nil{return err};if !d.IsDir(){ext:=filepath.Ext(p);if ext==".src"||ext==".go2"||ext==".go"{relative,err:=filepath.Rel(testdata,p);if err!=nil{return err};add("pinned-parser/"+filepath.ToSlash(relative),read(p))}};return nil});if err!=nil{t.Fatal(err)}
	for n,raw:=range []string{
		"package p", "package _", "package p; import ( . \"x\"; _ `y` ); var A,B int = 1,2; const ( X = iota; Y )",
		"package p; type ( A = int; B [1+2]A; C []B; D struct { A; P *B; X,Y map[string][]int `json:\"x\"` }; E interface { Error() string; ~int | ~string; D } )",
		"package p; type A[T any, P ~int|~int32] struct{ X T; Y P }; type B[T *int,] = A[T,int]",
		"package p; func F[T any](x,y T,z ...int)(a T,b error){ var c = func(v T)T{return v}; if q:=c(x); true { a=q } else if false { a=y } else { a=x }; return }",
		"package p; func F(ch chan int,x any){ L: for i:=0;i<3;i++ { if i==1 {continue L}; break }; for range 3 {}; for k,v:=range []int{1,2}{ _,_=k,v }; switch a:=x; b:=a.(type) {case int,string: _=b;default:}; select {case v:=<-ch: _=v; case ch<-1: default:}; go F(ch,x); defer F(ch,x); goto L }",
		"package p; func F(a []int,x any) { _=a[:]; _=a[1:2:3]; _=x.(int); _=[][]int{{1,2},{3}}; _=map[int]struct{X int}{1:{X:2}}; _=(*T)(nil); _=(<-chan<-chan int)(nil) }; type T int",
		"package p\r\n//line ../../invented.go:900:20\r\nconst X = `a\r\nb\rc`\r\ntype T struct{ X int `a\r\nb` }\r\n",
		"package p; type N[P*T] byte", "package p; type N[P*T,] byte", "package p; type N[P([]T)] byte", "package p; type N[P *E|F|~G] byte",
		"package p; func F(){ L: }; func G(){ ; ; return }",
		"package p; func F(x int){ switch { case true: fallthrough; default: return }; for ;true; {} }",
		"", "package", "package p; func F( {", "package p; type T struct{ (X) }", "package p; import 3", "package p; var x", "package p; type T [3,]int", "package p; func F(a ...int,b int){}", "package p; func F(){ if true\n{} }", "package p; func F(){ _=x[1::3] }", "package p; func F(){ go x }", "package p; func F(){ ~T }", "package p; var x = (T){1}", "package p; type T[P] struct{}",
	}{add(fmt.Sprintf("extra/%03d",n),[]byte(raw))}
	accepted,rejected,nodes:=0,0,0;kinds:=map[string]bool{}
	for n,entry:=range cases{
		dir:=filepath.Join(output,fmt.Sprintf("case-%05d",n));object,err:=entry.source.canonicalObject();if err!=nil{t.Fatal(err)};write(filepath.Join(dir,"source.json"),object);write(filepath.Join(dir,"source.raw"),entry.source.sourceBytes())
		value:=practicalParseReference(t,entry.source);value["label"]=entry.label;if value["accepted"].(bool){accepted++;v:=reflect.ValueOf(value["nodes"]);nodes+=v.Len();for i:=0;i<v.Len();i++{kinds[v.Index(i).FieldByName("Kind").String()]=true}}else{rejected++};raw,err:=json.Marshal(value);if err!=nil{t.Fatal(err)};write(filepath.Join(dir,"native.json"),raw)
	}
	summary:=map[string]any{"cases":len(cases),"accepted":accepted,"rejected":rejected,"nodes":nodes,"node_kinds":len(kinds),"runtime":runtime.Version(),"applications_executed":false,"original_context_or_typing_proof":false};raw,err:=json.Marshal(summary);if err!=nil{t.Fatal(err)};write(filepath.Join(output,"summary.json"),raw);t.Log(string(raw))
	if full&&len(cases)<2934||len(kinds)<40{t.Fatal("incomplete source syntax corpus",strings.Join([]string{fmt.Sprint(len(cases)),fmt.Sprint(len(kinds))},","))}
}
