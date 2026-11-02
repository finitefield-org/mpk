//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/constant"
	"go/parser"
	"go/scanner"
	"go/token"
	"go/types"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strings"
	"testing"
)

// Independent native oracle only: no original program is executed or admitted.
// All source bytes and object records are retained before scanner.Init/Scan.
func TestPracticalLexOriginalGoReference(t *testing.T) {
	input,output:=os.Getenv("MPK_GO_LEX_INPUT"),os.Getenv("MPK_GO_LEX_OUTPUT")
	fullCorpus:=input!=""
	if output==""{output=t.TempDir()}
	if runtime.Version()!="go1.25.0"{t.Fatal("lexical oracle requires the exact pinned Go1.25.0 runtime")}
	write:=func(path string,raw []byte){t.Helper();if err:=os.MkdirAll(filepath.Dir(path),0o700);err!=nil{t.Fatal(err)};f,err:=os.OpenFile(path,os.O_WRONLY|os.O_CREATE|os.O_EXCL,0o600);if err!=nil{t.Fatal(err)};_,e:=f.Write(raw);c:=f.Close();if e!=nil||c!=nil{t.Fatal(e,c)}}
	read:=func(path string)[]byte{t.Helper();raw,err:=os.ReadFile(path);if err!=nil{t.Fatal(err)};return raw}
	if input==""{input=t.TempDir();write(filepath.Join(input,"scanner_test.go"),read(filepath.Join(runtime.GOROOT(),"src/go/scanner/scanner_test.go")));write(filepath.Join(input,"manifest.json"),[]byte(`{"sources":[],"scanner_test":"scanner_test.go"}`))}
	var manifest struct{Sources []string `json:"sources"`; ScannerTest string `json:"scanner_test"`}
	if err:=json.Unmarshal(read(filepath.Join(input,"manifest.json")),&manifest);err!=nil{t.Fatal(err)}
	type entry struct{label string; object,raw []byte}
	cases:=[]entry{}
	for _,id:=range manifest.Sources {
		object:=read(filepath.Join(input,"sources",id+".json"));raw:=read(filepath.Join(input,"sources",id+".raw"));buffer,err:=decodePracticalSourceBuffer(object,raw);if err!=nil||buffer.object.ID!=id||buffer.object.Kind!="go_file"{t.Fatal("invalid original source case",id,err)}
		cases=append(cases,entry{"original/"+id,object,raw})
	}
	// Import every constant string expression in the pinned scanner's own test
	// source. Evaluation is go/constant only, with no imports or function calls.
	sdkTest:=read(filepath.Join(input,manifest.ScannerTest));write(filepath.Join(output,"scanner-test-source.go"),sdkTest);fset:=token.NewFileSet();file,err:=parser.ParseFile(fset,"scanner_test.go",sdkTest,0);if err!=nil{t.Fatal(err)}
	values:=map[string]bool{}
	ast.Inspect(file,func(node ast.Node)bool{
		expression,ok:=node.(ast.Expr);if !ok{return true}
		switch expression.(type){case *ast.BasicLit,*ast.BinaryExpr:default:return true}
		start,end:=fset.Position(expression.Pos()).Offset,fset.Position(expression.End()).Offset
		value,err:=types.Eval(token.NewFileSet(),nil,token.NoPos,string(sdkTest[start:end]));if err==nil&&value.Value!=nil&&value.Value.Kind()==constant.String{values[constant.StringVal(value.Value)]=true}
		return true
	})
	stringsFromSDK:=make([]string,0,len(values));for value:=range values{stringsFromSDK=append(stringsFromSDK,value)};sort.Strings(stringsFromSDK)
	add:=func(label string,raw []byte){t.Helper();buffer,err:=newPracticalSourceBuffer("lexical-cases",label+".go","go_file",raw);if err!=nil{t.Fatal(err)};object,err:=buffer.canonicalObject();if err!=nil{t.Fatal(err)};cases=append(cases,entry{label,object,raw})}
	for n,value:=range stringsFromSDK{add(fmt.Sprintf("pinned-scanner-literal/%04d",n),[]byte(value))}
	for n,value:=range [][]byte{
		nil,[]byte("\xef\xbb\xbfpackage π\r\nvar ᴬǅÉ０ byte\r\n"),[]byte("x/* a\r\n b */ y//end"),[]byte("x/*no-newline*/++\n"),[]byte("x\r\n//line ../../elsewhere.go:100:20\r\ny\n"),[]byte("/*line C:/abs:4:2*/x"),
		[]byte("0b019i 078i 0X_1.F_Fp-2i .123e2 0x1e+2"),[]byte("'世' '\\xff' \"\\xFF\\uFFFD\" `raw\r\nbytes`"),
		[]byte("//line x:0\npackage x"),[]byte("//line x:1:0\nx"),[]byte("/*line x:1073741825*/x"),[]byte("//line x:18446744073709551616\nx"),[]byte("\x00"),[]byte("\xff\xfeX\x00"),[]byte("\xfe\xff"),[]byte("\xef\xbb\xbf\xef\xbb\xbf"),[]byte("\"a\x80b\""),[]byte("x\f y"),[]byte("x\u00a0y"),[]byte("/**\r/"),
	}{add(fmt.Sprintf("extra/%03d",n),value)}
	// Every pair of actual operator spellings tests longest-match boundaries.
	operators:=[]string{"+","-","*","/","%","&","|","^","<<",">>","&^","+=","-=","*=","/=","%=","&=","|=","^=","<<=",">>=","&^=","&&","||","<-","++","--","==","<",">","=","!","!=","<=",">=",":=","...","(",")","[","]","{","}",",",";",".",":","~"}
	for i,a:=range operators{for j,b:=range operators{add(fmt.Sprintf("operator-pairs/%02d-%02d",i,j),[]byte(a+b))}}
	accepted,rejected,tokens:=0,0,0
	for n,entry:=range cases{
		dir:=filepath.Join(output,fmt.Sprintf("case-%05d",n));write(filepath.Join(dir,"source.raw"),entry.raw);write(filepath.Join(dir,"source.json"),entry.object)
		var errors []map[string]any
		set:=token.NewFileSet();physical:=set.AddFile("original.go",-1,len(entry.raw));var lexer scanner.Scanner
		lexer.Init(physical,entry.raw,func(position token.Position,message string){errors=append(errors,map[string]any{"offset":position.Offset,"message":message})},0)
		rows:=make([][3]any,0)
		for{pos,kind,literal:=lexer.Scan();rows=append(rows,[3]any{physical.Offset(pos),kind.String(),hex.EncodeToString([]byte(literal))});if kind==token.EOF{break}}
		tokens+=len(rows);if lexer.ErrorCount==0{accepted++}else{rejected++}
		record:=map[string]any{"label":entry.label,"accepted":lexer.ErrorCount==0,"tokens":rows,"errors":errors,"scanner_errors":lexer.ErrorCount,"source_typing_or_program_authority":false}
		raw,err:=json.Marshal(record);if err!=nil{t.Fatal(err)};write(filepath.Join(dir,"native.json"),raw)
	}
	summary:=map[string]any{"cases":len(cases),"original_sources":len(manifest.Sources),"pinned_scanner_constant_strings":len(stringsFromSDK),"accepted":accepted,"rejected":rejected,"native_tokens":tokens,"runtime":runtime.Version(),"native_scanner_source_not_replaced":true,"application_executed":false}
	raw,err:=json.Marshal(summary);if err!=nil{t.Fatal(err)};write(filepath.Join(output,"summary.json"),raw);t.Log(string(raw))
	if fullCorpus&&len(manifest.Sources)<5000||len(stringsFromSDK)<500||accepted==0||rejected==0{t.Fatal("incomplete original lexical corpus",strings.Join(manifest.Sources[:min(1,len(manifest.Sources))],","))}
}
