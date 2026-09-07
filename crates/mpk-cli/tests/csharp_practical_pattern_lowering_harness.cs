using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
namespace Mpk.CSharp2Vir;
internal static class PracticalPatternLoweringHarness
{
    public static int Main(string[] args)
    {
        string stage="setup";
        try {
            var references=Directory.EnumerateFiles(Path.Combine(args[0],"ref","net10.0"),"*.dll").OrderBy(p=>p,StringComparer.Ordinal).Select(p=>MetadataReference.CreateFromFile(p)).ToImmutableArray<MetadataReference>();
            string integer=PracticalIdentity.PrimitiveId("i32");
            string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{integer,PracticalIdentity.ClosedInstanceId("bounded_sequence",integer),PracticalIdentity.PrimitiveId("string")},integer);
            var cases=new List<(string Id,string Body,bool Accept)> {
                ("constant","return n switch {0=>10,1=>20,_=>30};",true),
                ("relational","return n switch {<0=>-1,>=1 and <=2=>1,not (3 or 4)=>2,_=>3};",true),
                ("guard_order","int i=0;int r=n switch {var x when ++i==1 && x==0=>10,var y when ++i==2=>y,_=>30};return r*10+i;",true),
                ("guard_fallthrough","int i=0;return n switch {var x when ++i<0=>x,_=>i};",true),
                ("once","int result=n++ switch {0=>10,_=>20};return result+n;",true),
                ("nonexhaustive","return n switch {0=>10};",true),
                ("bool_exhaustive","return (n>0) switch {true=>1,false=>0};",true),
                ("statement","int r=0;switch(n){case 0:case 1:r=10;break;case int x when x>2:r=x;break;default:r=30;break;}return r;",true),
                ("default_first","switch(n){default:return 30;case 0:return 10;case 1:return 20;}",true),
                ("statement_unmatched","int r=3;switch(n){case 0:r=1;break;}return r;",true),
                ("nested_control","int sum=0;for(int i=0;i<n;i++){switch(i){case 0:continue;case 1:break;default:sum+=i;break;}sum++;}return sum;",true),
                ("nested_switch","switch(n){case 0:switch(a.Length){case 0:return 1;default:break;}break;default:break;}return 2;",true),
                ("is_binding","if(n is >0 and var x)return x;return -1;",true),
                ("null","string? text=n==0?null:s;return text switch {null=>1,not null=>2};",true),
                ("nullable","int? value=n==0?null:n;return value switch {null=>0,int x=>x};",true),
                ("string","return s switch {\"\"=>0,\"A😀日本\"=>1,_=>2};",true),
                ("list","return a switch {[]=>0,[var x,>0,_,_,3]=>x,[4,1,3]=>9,_=>10};",true),
                ("list_logical","return a is not [] and [>0,1,3] ? 1:0;",true),
                ("nullable_list","int[]? value=n==0?null:a;return value switch {null=>1,[]=>2,_=>3};",true),
                ("array_binding","int[] x=new int[2];return x switch {var alias=>alias.Length};",true),
                ("property","Box b=new Box(n);return b switch {Value: >0} ? 1:0;",false),
                ("property_claim","Box b=new Box(n);return b switch { {Value: >0}=>1,_=>0};",true),
                ("field","Box b=new Box(n);return b is {Stored:var x} ? x:0;",true),
                ("type","Box? b=n==0?null:new Box(n);return b switch {Box x=>x.Stored,null=>0};",true),
                ("slice","return a is [..] ? 1:0;",false),
                ("positional","Box b=new Box(n);return b is Box(0) ? 1:0;",false),
                ("goto_case","switch(n){case 0:goto case 1;case 1:return 1;default:return 2;}",false),
                ("goto_default","switch(n){case 0:goto default;default:return 2;}",false),
                ("fallthrough","switch(n){case 0:n++;case 1:return 1;default:return 2;}",false),
                ("overlap","return n switch {_=>0,1=>1};",false),
                ("binding_scope","return n switch {int x when x>0=>x,_=>x};",false),
                ("identity","object x=n;return x is int y?y:0;",false),
                ("unsupported_list","return s is ['a']?1:0;",false),
                ("enum","Kind value=n==0?Kind.A:Kind.B;return value switch {Kind.A=>0,Kind.B=>1};",true),
                ("alias_write","int[] x=new int[2];if(x is var alias && alias.Length>0){x[0]=1;}return x[0];",false),
                ("guard_alias_write","int[] x=new int[2];return x switch {var alias when (x[0]=1)>0=>alias[0],_=>0};",false),
                ("getter_effect","Box b=new Box(n);return b is {Value:0}?1:0;",false),
                ("default_guard_effect","int i=0;switch(n){default:return i;case var x when ++i<0:return x;}",true),
                ("default_guard_alias","int[] x=new int[2];int[] alias=new int[2];switch(n){default:x[0]=1;break;case var unused when (alias=x).Length<0:return unused;}return alias[0];",false),
                ("array_property","return a is {Length:>2}?1:0;",true),
                ("string_property","return s is {Length:>0}?1:0;",true),
                ("single_list","return a is [var x]?x:0;",true),
                ("guard_throw","return n switch {var x when 10/x>1=>1,_=>0};",true),
                ("governing_throw","return a[n] switch {0=>1,_=>0};",true),
                ("list_guard_write","int[] x=new int[1];return x switch {[0] when (x[0]=1)==0=>0,[1]=>1,_=>2};",false),
                ("list_statement_guard_write","int[] x=new int[1];switch(x){case [0] when (x[0]=1)==0:return 0;case [1]:return 1;default:return 2;}",false),
                ("list_body_write","int[] x=new int[1];return x switch {[0]=>(x[0]=1),_=>2};",true),
                ("double_nan","double x=n==0?0.0/0.0:1.0;return x switch {(0.0/0.0)=>10,>0.0=>1,_=>0};",true),
                ("float_nan","float x=n==0?0.0f/0.0f:1.0f;return x switch {(0.0f/0.0f)=>10,>0.0f=>1,_=>0};",true),
                ("decimal","decimal x=n==0?0.5m:1.0m;return x switch {0.5m=>1,>0.75m=>2,_=>0};",true),
                ("array_arm","int[] x=n switch {0=>new int[1],_=>new int[2]};return x.Length;",true),
                ("getter_missing","Box b=new Box(n);return b is {Value:0}?1:0;",false),
            };
            var rows=new List<object>();var failures=new List<string>();
            foreach(var item in cases) {
                stage=item.Id;
                string prefix=item.Body.Contains("Box",StringComparison.Ordinal)?"public sealed class Box{public readonly int Stored;public Box(int value){Stored=value;}"+(item.Body.Contains("Value",StringComparison.Ordinal)?"public int Value=>Stored;":"")+"}":"";
                if(item.Id=="enum")prefix="public enum Kind{A=0,B=1}";
                if(item.Id=="getter_effect")prefix="public sealed class Box{public int Stored;public Box(int value){Stored=value;}public int Value=>++Stored;}";
                string source="namespace Business;"+prefix+"public static class Entry{public static int Run(int n,int[] a,string s){"+item.Body+"}}\n";
                var input=new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source));
                var selection=new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"data",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>());
                try {
                    byte[] bytes=CSharpPracticalLoopLowering.Capture(selection,new[]{input},references,allowPatterns:true,totalGetters:item.Id=="property_claim"?new[]{PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Box"),"get_Value",Array.Empty<string>(),integer)}:Array.Empty<string>());
                    if(!item.Accept)throw new Exception("unexpected_accept");
                    if(!bytes.SequenceEqual(CSharpPracticalLoopLowering.Capture(selection,new[]{input},references,allowPatterns:true,totalGetters:item.Id=="property_claim"?new[]{PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Box"),"get_Value",Array.Empty<string>(),integer)}:Array.Empty<string>())))throw new Exception("determinism");
                    if(item.Id is "guard_order" or "list" or "nonexhaustive") {
                        var changed=JsonNode.Parse(bytes)!;
                        var nodes=changed["functions"]![0]!["nodes"]!.AsArray();
                        if(item.Id=="guard_order") {
                            var branch=nodes.First(n=>n!["kind"]!.GetValue<string>()=="branch")!;
                            var a=branch["successors"]![0]!.GetValue<string>();var b=branch["successors"]![1]!.GetValue<string>();
                            branch["successors"]=new JsonArray(b,a);
                        } else if(item.Id=="list")nodes.First(n=>n!["operation"]!.GetValue<string>()=="pattern_bind")!["slot"]="local:999";
                        else nodes.First(n=>n!["kind"]!.GetValue<string>()=="throw" && n["slot"]!.GetValue<string>()!="")!["slot"]="System.InvalidOperationException";
                        bool rejected=false;
                        try{CSharpPracticalLoopLowering.ValidatePatternCandidate(selection,new[]{input},references,Array.Empty<string>(),JsonSerializer.SerializeToUtf8Bytes(changed));}
                        catch(PracticalCaptureFailure e){rejected=e.Code=="loop_lowering_pattern_candidate_mismatch";}
                        if(!rejected)throw new Exception("candidate_mutation");
                    }
                    var compilation=CSharpCompilation.Create("Runtime"+item.Id,new[]{CSharpSyntaxTree.ParseText(source,new CSharpParseOptions((LanguageVersion)1400))},references,new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary,checkOverflow:true));
                    using var stream=new MemoryStream();if(!compilation.Emit(stream).Success)throw new Exception("runtime_compile");
                    var method=Assembly.Load(stream.ToArray()).GetType("Business.Entry")!.GetMethod("Run")!;
                    var runs=new List<object>();
                    foreach(int n in new[]{-1,0,1,2,3,4,6})foreach(int[] a in new[]{Array.Empty<int>(),new[]{-7},new[]{-1,0,2,2,3},new[]{4,1,3}})foreach(string s in new[]{"","A😀日本","other"}) {
                        int? value=null;string error="";
                        try { value=(int)method.Invoke(null,new object[]{n,(int[])a.Clone(),s})!; }
                        catch(TargetInvocationException ex) {error=ex.InnerException!.GetType().Name;}
                        runs.Add(new {n,a,s,value,error});
                    }
                    rows.Add(new{id=item.Id,source,root,accepted=true,lowering=JsonSerializer.Deserialize<JsonElement>(bytes),runs,diagnostic="",code=""});
                } catch(PracticalCaptureFailure e) {
                    if(item.Accept)failures.Add(item.Id+":"+e.Family+":"+e.Code);
                    if(e.ArtifactCount!=0)throw new Exception("artifact");
                    rows.Add(new{id=item.Id,source,root,accepted=false,lowering=(object?)null,runs=Array.Empty<object>(),diagnostic=e.Family.ToString(),code=e.Code});
                }
            }
            if(failures.Count!=0)throw new Exception(string.Join("\n",failures));
            File.WriteAllBytes("loop-source-cases.json",JsonSerializer.SerializeToUtf8Bytes(rows));return 0;
        } catch(Exception error) {Console.Error.WriteLine(stage+":"+error.Message);return 1;}
    }
}
