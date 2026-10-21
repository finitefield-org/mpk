//go:build go1.25 && (linux || darwin)

package main

import (
 "encoding/hex"
 "encoding/json"
 "fmt"
 "go/ast"
 "go/parser"
 "go/token"
 "go/types"
 "os"
 "path/filepath"
 "runtime"
 "sort"
 "strings"
 "testing"
)

type practicalAssignmentTypingCase struct {
 label, statement, setup, tail string
 outside, helper bool
}
func practicalAssignmentTypingCases() []practicalAssignmentTypingCase {
 cases:=[]practicalAssignmentTypingCase{}
 add:=func(statement string){cases=append(cases,practicalAssignmentTypingCase{label:statement,statement:statement})}
 for _,lhs:=range []string{"x","v","u","b","text","a","s","rec","err","_","(_)"} {
  for _,rhs:=range []string{"x","y","v","u","Other(1)","1","-1","65535","65536","1<<n","(1<<n)+65536","b","c","true","text","str","\"hi\"","a","[2]Word{1}","s","[]Word{1}","rec","Record{}","Code(1)","nil"} {add(lhs+"="+rhs)}
 }
 for _,lhs:=range []string{"x","v","u","b","text","a[0]","s[0]","rec.W"} {
  for _,op:=range []string{"+=","-=","*=","/=","%=","&=","|=","^=","&^=","<<=",">>="} {
   for _,rhs:=range []string{"1","0","-1","65536","x","v","u","1<<n","(1<<64)-1","b","\"x\""} {add(lhs+op+rhs)}
  }
 }
 for _,lhs:=range []string{"x","(x)","a[0]","s[0]","rec.W","rec.A[0]","rec.S[0]","GetSlice()[Index()]","[]Word{1}[0]","GetArray()[0]","[1]Word{1}[0]","GetRecord().W","str[0]","\"x\"[0]","C","1","x+y","Id(x)","_","(_)","b","text","a","s","err","global"} {
  add(lhs+"=1");add(lhs+"+=1");add(lhs+"++");add(lhs+"--")
 }
 for _,statement:=range []string{"x,y=y,x","x,x=1,2","n,s[n]=1,2","s[n],n=1,2","x,u=Pair(x)","_,_=Pair(x)","x,_=Pair(x)","_,u=Pair(x)","v,u=Pair(x)","x,v=Pair(x)","x=Pair(x)","x,y=1","x=1,2","x,y=Pair(x),1","x,y=1,Pair(x)","_=Void()","x,y+=1,2","x+=Pair(x)","_+=1","(_)++","x<<=-1","x<<=(1<<64)","x<<=u","x<<=(1<<64)-1","x<<=(1<<u)","x>>=-u","x/=0","x%=0","x/=1-1","text+=str","str+=text","text+=\"a\"","err=Code(1)","err=Good{}","err=error(Code(1))","err=nil","s=nil","global=x","C=x","rec._=x","missing=1","s[2]=1","a[2]=1","a[-1]=1","x+=float64(1)","x+=true"} {add(statement)}
 for _,statement:=range []string{"q:=1","q:=1<<n","q:=nil","x,q:=x+1,2","x,q:=Other(1),2","q,r:=Pair(x)","_,q:=Pair(x)","q,_:=Pair(x)","q,r:=Pair(x),1","q,q:=1,2","_,_:=Pair(x)","q:=Void()","q:=65536","q:=Code(1)","q:=s","q:=rec","q:=Good{}"} {
  tail:=";_=q"
  if strings.Contains(statement,"r:="){tail+=";_=r"}
  cases=append(cases,practicalAssignmentTypingCase{label:"short-"+statement,statement:statement,tail:tail})
 }
 for _,statement:=range []string{"x:=1","x,y:=1,2","x,x:=1,2","x,q:=x,1"} {
  tail:="";if strings.Contains(statement,",q"){tail=";_=q"}
  cases=append(cases,practicalAssignmentTypingCase{label:"short-reused-"+statement,statement:statement,tail:tail})
 }
 cases=append(cases,practicalAssignmentTypingCase{label:"short-shadow",statement:"x,q:=x,1",setup:"{",tail:";_=x;_=q}"})
 cases=append(cases,practicalAssignmentTypingCase{label:"shared-invalid-target-init",statement:"old=2",setup:"var old,bad uint8=1,256;",tail:";_=bad"})
 for _,statement:=range []string{"h.G=x","h.G=h.Id(v)","h.C=x","h.hidden=x","x=h.G","h.G,h.G=v,v"} {cases=append(cases,practicalAssignmentTypingCase{label:"import-"+statement,statement:statement,helper:true})}
 for _,statement:=range []string{"*ptr=1","ptr.N=1","m[0]=1","m[0]++","*ptr+=1","f+=1.0","f++"} {
  cases=append(cases,practicalAssignmentTypingCase{label:"outside-"+statement,statement:statement,setup:"var ptr *Word;var m map[int]Word;var f float64;_=ptr;_=m;_=f;",outside:true})
 }
 parts:=[]string{};args:=[]string{}
 for n:=0;n<200;n++{parts=append(parts,"x");args=append(args,"1")}
 add(strings.Join(parts,",")+"="+strings.Join(args,","))
 parts=nil;args=nil
 for n:=0;n<200;n++{parts=append(parts,fmt.Sprintf("q%d",n));args=append(args,"1")}
 tail:="";for _,name:=range parts{tail+=";_="+name}
 cases=append(cases,practicalAssignmentTypingCase{label:"short200",statement:strings.Join(parts,",")+":="+strings.Join(args,","),tail:tail})
 parts=nil;for n:=0;n<16;n++{parts=append(parts,"_")};add(strings.Join(parts,",")+"=Many16()")
 parts=append(parts,"_");cases=append(cases,practicalAssignmentTypingCase{label:"results17",statement:strings.Join(parts,",")+"=Many17()",outside:true})
 return cases
}
func TestPracticalOriginalAssignmentTypingReference(t *testing.T) {
 if runtime.Version()!=practicalCompilerRelease{t.Fatal(runtime.Version())}
 out:=os.Getenv("MPK_GO_ASSIGNMENT_TYPING_NATIVE_OUTPUT");if out==""{out=t.TempDir()}
 write:=func(path string,value any){raw,err:=json.MarshalIndent(value,"","  ");if err!=nil{t.Fatal(err)};f,err:=os.OpenFile(path,os.O_CREATE|os.O_EXCL|os.O_WRONLY,0o600);if err!=nil{t.Fatal(err)};if _,err=f.Write(append(raw,'\n'));err!=nil{t.Fatal(err)};if err=f.Close();err!=nil{t.Fatal(err)}}
 accepted,rejected:=0,0
 for index,c:=range practicalAssignmentTypingCases(){
 dir:=filepath.Join(out,fmt.Sprintf("case-%05d",index));if err:=os.Mkdir(dir,0o700);err!=nil{t.Fatal(err)}
 source:="package policy\n";helper:=""
 if c.helper {source+="import h \"example.com/scalar/helper\"\n";helper="package foreign\ntype Other uint16\nvar G uint16\nconst C uint16=1\nvar hidden uint16\nfunc Id(x uint16)uint16{return x}\n"}
 source+=`type Word uint16
type Other uint16
type Flag bool
type Text string
type Code uint16
func(Code)Error()string{return "code"}
type Good struct{}
func(Good)Error()string{return "good"}
type Record struct{W Word;A [2]Word;S []Word;_ Word}
const C Word=1
var global Word
func Id(x Word)Word{return x}
func Pair(x Word)(Word,uint64){return x,0}
func Void(){}
func GetSlice()[]Word{return nil}
func GetArray()[2]Word{return [2]Word{}}
func GetRecord()Record{return Record{}}
func Index()int{return 0}
`
 for _,arity:=range []int{16,17}{parts:=[]string{};for n:=0;n<arity;n++{parts=append(parts,fmt.Sprintf("r%d Word",n))};source+=fmt.Sprintf("func Many%d()(%s){return}\n",arity,strings.Join(parts,","))}
 source+="func Probe(x,y Word,v uint16,u uint64,n uint64,b Flag,c bool,text Text,str string,a [2]Word,s []Word,rec Record,err error){\n"+c.setup
 start:=len(source);source+=c.statement;end:=len(source);source+=c.tail+"\n}\n"
 cap:=fmt.Sprintf("assignment-typing/case-%05d",index)
		mod, err := newPracticalSourceBuffer(cap, "go.mod", "go_mod", []byte("module example.com/scalar\ngo 1.25.0\n"))
		if err != nil {
			t.Fatal(err)
		}
		root, err := newPracticalSourceBuffer(cap, "policy.go", "go_file", []byte(source))
		if err != nil {
			t.Fatal(err)
		}
		buffers := []practicalSourceBuffer{mod, root}
		names := map[string]string{"example.com/scalar": "policy"}
		fileIDs := map[string]string{"example.com/scalar": root.object.ID}
		if c.helper {
			h, err := newPracticalSourceBuffer(cap, "helper/helper.go", "go_file", []byte(helper))
			if err != nil {
				t.Fatal(err)
			}
			buffers = append(buffers, h)
			names["example.com/scalar/helper"] = "foreign"
			fileIDs["example.com/scalar/helper"] = h.object.ID
		}
		pkgs := map[string]map[string]any{}
		pkgIDs := map[string]string{}
		pkgRoots := []string{}
		for path, name := range names {
			p, err := practicalTypeKeySeal(practicalTypePackageDomain, map[string]any{"schema": practicalTypePackageSchema, "module_source_object_id": mod.object.ID, "import_path": path, "package_name": name, "source_objects": []string{fileIDs[path]}})
			if err != nil {
				t.Fatal(err)
			}
			id := p["id"].(string)
			pkgs[id] = p
			pkgIDs[path] = id
			pkgRoots = append(pkgRoots, id)
		}
		sort.Strings(pkgRoots)
		scalar, err := practicalTypeKeySeal(practicalSourceTypeDomain, map[string]any{"schema": practicalSourceTypeSchema, "kind": "builtin", "name": "bool", "width": nil, "signed": nil})
		if err != nil {
			t.Fatal(err)
		}
		objects := []practicalSourceObject{}
		originals := []map[string]any{}
		for _, b := range buffers {
			objects = append(objects, b.object)
			originals = append(originals, map[string]any{"object": b.object, "raw_hex": hex.EncodeToString(b.data)})
		}
		sort.Slice(objects, func(i, j int) bool { return objects[i].ID < objects[j].ID })
		components := map[string]any{"schema": practicalTypeComponentsSchema, "source_objects": objects, "packages": practicalTypeKeyRows(pkgs), "declarations": []any{}, "types": []any{scalar}, "underlying": []any{}, "aliases": []any{}, "package_roots": pkgRoots, "type_roots": []string{scalar["id"].(string)}}
		hash, err := hashTypedCanonicalJSON(practicalTypeComponentsDomain, components)
		if err != nil {
			t.Fatal(err)
		}
		components["content_sha256"] = hash
		canonical, err := canonicalJSON(components)
		if err != nil {
			t.Fatal(err)
		}

 write(filepath.Join(dir,"input.json"),map[string]any{"index":index,"label":c.label,"components_hex":hex.EncodeToString(canonical),"sources":originals,"query_source_object_id":root.object.ID,"statement_start_byte":start,"statement_end_byte":end,"outside_assignment_typing":c.outside,"body_state_or_program_proof":false})
		set := token.NewFileSet()
		infos := map[string]*types.Info{}
		files := map[string]*ast.File{}
		typed := practicalConstantReferenceImports{}
		var failure error
		for _, path := range []string{"example.com/scalar/helper", "example.com/scalar"} {
			if names[path] == "" {
				continue
			}
			raw := source
			if strings.HasSuffix(path, "/helper") {
				raw = helper
			}
			file, err := parser.ParseFile(set, path+".go", raw, parser.AllErrors|parser.SkipObjectResolution)
			if err != nil {
				failure = err
				break
			}
			info := &types.Info{Types: map[ast.Expr]types.TypeAndValue{}, Defs: map[*ast.Ident]types.Object{}, Uses: map[*ast.Ident]types.Object{}, Scopes: map[ast.Node]*types.Scope{}, Selections: map[*ast.SelectorExpr]*types.Selection{}}
			config := types.Config{Importer: typed, GoVersion: "go1.25", Sizes: types.SizesFor("gc", "amd64")}
			pkg, err := config.Check(path, set, []*ast.File{file}, info)
			if err != nil {
				failure = err
				break
			}
			typed[path] = pkg
			infos[path] = info
			files[path] = file
		}

 result:=map[string]any{"accepted":failure==nil,"body_state_or_program_proof":false}
 if failure!=nil{result["error"]=failure.Error();rejected++}else{
			var key func(types.Type) map[string]any
			key = func(typ types.Type) map[string]any {
				typ = types.Unalias(typ)
				var row map[string]any
				seal := func(fields map[string]any) map[string]any {
					fields["schema"] = practicalSourceTypeSchema
					r, e := practicalTypeKeySeal(practicalSourceTypeDomain, fields)
					if e != nil {
						t.Fatal(e)
					}
					return r
				}
				if named, ok := typ.(*types.Named); ok {
					obj := named.Obj()
					if obj.Pkg() == nil && obj.Name() == "error" {
						return seal(map[string]any{"kind": "builtin", "name": "error", "width": nil, "signed": nil})
					}
					path := obj.Pkg().Path()
					var spec *ast.TypeSpec
					ast.Inspect(files[path], func(n ast.Node) bool {
						if c, ok := n.(*ast.TypeSpec); ok && infos[path].Defs[c.Name] == obj {
							spec = c
						}
						return true
					})
					if spec == nil {
						t.Fatal("named origin", obj)
					}
					file := set.File(spec.Pos())
					decl, e := practicalTypeKeySeal(practicalTypeDeclarationDomain, map[string]any{"schema": practicalTypeDeclarationSchema, "package_id": pkgIDs[path], "source_object_id": fileIDs[path], "kind": "type", "start_byte": file.Offset(spec.Pos()), "end_byte": file.Offset(spec.End()), "name": obj.Name()})
					if e != nil {
						t.Fatal(e)
					}
					return seal(map[string]any{"kind": "nominal", "declaration_id": decl["id"]})
				}
				switch typ := typ.(type) {
				case *types.Basic:
					var width, signed any
					if typ.Info()&types.IsInteger != 0 {
						width = types.SizesFor("gc", "amd64").Sizeof(typ) * 8
						signed = typ.Info()&types.IsUnsigned == 0
					}
					row = map[string]any{"kind": "builtin", "name": types.Typ[typ.Kind()].Name(), "width": width, "signed": signed}
				case *types.Array:
					row = map[string]any{"kind": "array", "length": typ.Len(), "element": key(typ.Elem())["id"]}
				case *types.Slice:
					row = map[string]any{"kind": "slice", "element": key(typ.Elem())["id"]}
				case *types.Struct:
					fields := []map[string]any{}
					for i := 0; i < typ.NumFields(); i++ {
						f := typ.Field(i)
						var pkg any
						if !f.Exported() {
							pkg = pkgIDs[f.Pkg().Path()]
						}
						fields = append(fields, map[string]any{"name": f.Name(), "unexported_package_id": pkg, "type_id": key(f.Type())["id"], "tag_bytes_hex": hex.EncodeToString([]byte(typ.Tag(i))), "embedded": f.Embedded()})
					}
					row = map[string]any{"kind": "struct", "fields": fields}
				default:
					if !c.outside {
						t.Fatal("unmodeled native result", typ)
					}
					return nil
				}
				return seal(row)
			}

 info:=infos["example.com/scalar"];file:=files["example.com/scalar"]
 var query ast.Stmt
 ast.Inspect(file,func(n ast.Node)bool{if s,ok:=n.(ast.Stmt);ok{f:=set.File(s.Pos());if f.Offset(s.Pos())==start&&f.Offset(s.End())==end{if query!=nil{t.Fatal("ambiguous statement")};query=s}};return true})
 if query==nil{t.Fatal("missing statement",c.label)}
 var lhs,rhs []ast.Expr;var op token.Token
 switch s:=query.(type){case *ast.AssignStmt:lhs,rhs,op=s.Lhs,s.Rhs,s.Tok;case *ast.IncDecStmt:lhs,op=[]ast.Expr{s.X},s.Tok;default:t.Fatal("unexpected statement",c.label)}
 rhsTypes:=[]types.Type{}
 for _,x:=range rhs{typ:=info.TypeOf(x);if tuple,ok:=typ.(*types.Tuple);ok{for n:=0;n<tuple.Len();n++{rhsTypes=append(rhsTypes,tuple.At(n).Type())}}else{rhsTypes=append(rhsTypes,typ)}}
 targets:=[]map[string]any{};slots:=[]map[string]any{}
 for n,x:=range lhs{
  node:=x;for{p,ok:=node.(*ast.ParenExpr);if !ok{break};node=p.X}
  ident,isIdent:=node.(*ast.Ident);blank:=isIdent&&ident.Name=="_"
  typ:=info.TypeOf(x);var target any
  if blank{if n>=len(rhsTypes){t.Fatal("blank arity")};typ=types.Default(rhsTypes[n])}else{target=key(typ)}
  declares:=op==token.DEFINE&&isIdent&&ident.Name!="_"&&info.Defs[ident]!=nil
  f:=set.File(x.Pos());targets=append(targets,map[string]any{"start_byte":f.Offset(x.Pos()),"end_byte":f.Offset(x.End()),"type_key":target,"declares_variable":declares})
  slots=append(slots,map[string]any{"type_key":key(typ),"addressable":!blank,"constant":false,"untyped_nil":false})
 }
 calls:=[]map[string]any{}
 ast.Inspect(query,func(n ast.Node)bool{
  call,ok:=n.(*ast.CallExpr);if !ok{return true};fun:=call.Fun;for{p,ok:=fun.(*ast.ParenExpr);if !ok{break};fun=p.X}
  var obj types.Object
  switch f:=fun.(type){case *ast.Ident:obj=info.Uses[f];case *ast.SelectorExpr:if info.Selections[f]!=nil{return true};obj=info.Uses[f.Sel]}
  fn,ok:=obj.(*types.Func);if !ok{return true};sig:=fn.Type().(*types.Signature);if sig.Recv()!=nil{return true}
  path:=fn.Pkg().Path();var decl *ast.FuncDecl
  ast.Inspect(files[path],func(n ast.Node)bool{if d,ok:=n.(*ast.FuncDecl);ok&&infos[path].Defs[d.Name]==fn{decl=d};return true})
  if decl==nil{t.Fatal("missing call origin")};f:=set.File(call.Pos());df:=set.File(decl.Pos());var bs,be any;if decl.Body!=nil{bs,be=df.Offset(decl.Body.Pos()),df.Offset(decl.Body.End())}
  calls=append(calls,map[string]any{"start_byte":f.Offset(call.Pos()),"end_byte":f.Offset(call.End()),"declaration_source":fileIDs[path],"declaration_start":df.Offset(decl.Pos()),"declaration_end":df.Offset(decl.End()),"body_start":bs,"body_end":be})
  return true
 })
 sort.Slice(calls,func(i,j int)bool{if calls[i]["start_byte"].(int)==calls[j]["start_byte"].(int){return calls[i]["end_byte"].(int)<calls[j]["end_byte"].(int)};return calls[i]["start_byte"].(int)<calls[j]["start_byte"].(int)})
 result["operator"]=op.String();result["targets"]=targets;result["slots"]=slots;result["calls"]=calls;accepted++
 }
 write(filepath.Join(dir,"native.json"),result)
 }
 write(filepath.Join(out,"summary.json"),map[string]any{"cases":len(practicalAssignmentTypingCases()),"accepted":accepted,"rejected":rejected,"runtime":runtime.Version(),"sizes":"gc/amd64","applications_executed":false,"body_state_or_program_proof":false})
}
