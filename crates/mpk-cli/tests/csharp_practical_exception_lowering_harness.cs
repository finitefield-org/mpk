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
internal static class PracticalExceptionLoweringHarness
{
    public static int Main(string[] args)
    {
        try {
            var refs=Directory.EnumerateFiles(Path.Combine(args[0],"ref","net10.0"),"*.dll").OrderBy(p=>p,StringComparer.Ordinal).Select(p=>MetadataReference.CreateFromFile(p)).ToImmutableArray<MetadataReference>();
            string integer=PracticalIdentity.PrimitiveId("i32");
            string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{integer},integer);
            string payload="public sealed class Fault:System.Exception{public readonly int Code;public Fault(int code){Code=code;}}";
            var cases=new List<(string Id,string Declaration,string Body,bool Accept)>();
            foreach(string type in new[]{"DivideByZeroException","OverflowException","IndexOutOfRangeException","ArgumentException","ArgumentOutOfRangeException","ArgumentNullException","InvalidOperationException","NullReferenceException","Runtime.CompilerServices.SwitchExpressionException"})
                cases.Add((type,"","if(n<0)throw new System."+type+"();return n;",true));
            cases.AddRange(new (string,string,string,bool)[]{
                ("payload",payload,"if(n<0)throw new Fault(n);return n;",true),
                ("explicit_base",payload.Replace("Fault(int code)","Fault(int code):base()"),"throw new Fault(n);",true),
                ("get_only",payload.Replace("public readonly int Code;","public int Code{get;}"),"throw new Fault(n);",true),
                ("empty","public sealed class Fault:System.Exception{}","throw new Fault();",true),
                ("argument_order",payload.Replace("public Fault(int code){Code=code;}","public Fault(int first,int second){Code=first*10+second;}"),"throw new Fault(n++,n++);",true),
                ("constructor_failure",payload.Replace("Code=code;","if(code<0)throw new System.ArgumentException();Code=code;"),"throw new Fault(n);",true),
                ("argument_failure",payload,"throw new Fault(10/n);",true),
                ("loop",payload,"while(n<0){throw new Fault(n);}return n;",true),
                ("pattern",payload,"if(n is <0)throw new Fault(n);return n;",true),
                ("message","","throw new System.ArgumentException(\"message\");",false),
                ("inner","","throw new System.ArgumentException(\"message\",new System.Exception());",false),
                ("null","","throw null;",false),
                ("expression","","return n>0?n:throw new System.ArgumentException();",false),
                ("stored","","System.ArgumentException e=new System.ArgumentException();throw e;",false),
                ("rethrow","","throw;",false),
                ("oom","","throw new System.OutOfMemoryException();",false),
                ("stack","","throw new System.StackOverflowException();",false),
                ("unknown","","throw new System.NotSupportedException();",false),
                ("unsealed",payload.Replace("sealed ",""),"throw new Fault(n);",false),
                ("wrong_base",payload.Replace("System.Exception","System.ArgumentException"),"throw new Fault(n);",false),
                ("message_base",payload.Replace("Fault(int code)","Fault(int code):base(\"message\")"),"throw new Fault(n);",false),
                ("mutable",payload.Replace("readonly ",""),"throw new Fault(n);",false),
                ("throw_initializer",payload,"throw new Fault(n){};",false),
                ("observe",payload,"Fault e=new Fault(n);return e.Message.Length;",false),
                ("catch","","try{throw new System.ArgumentException();}catch(System.ArgumentException){return n;}",false),
                ("runtime_observation",payload.Replace("Code=code;","Code=Message.Length;"),"throw new Fault(n);",false),
                ("resource_base",payload.Replace("System.Exception","System.OutOfMemoryException"),"throw new Fault(n);",false),
                ("named_arguments",payload,"throw new Fault(code:n);",false),
                ("dynamic","","dynamic e=n;throw e;",false),
                ("finally","","try{return n;}finally{n++;}",false),
            });
            var rows=new List<object>();var failures=new List<string>();
            foreach(var item in cases) {
                string source="namespace Business;"+item.Declaration+"public static class Entry{public static int Run(int n){"+item.Body+"}}\n";
                var input=new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source));
                var selection=new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"data",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>());
                try {
                    byte[] bytes=CSharpPracticalLoopLowering.Capture(selection,new[]{input},refs,true,Array.Empty<string>(),true);
                    if(!item.Accept)throw new Exception(item.Id+":unexpected_accept");
                    CSharpPracticalLoopLowering.ValidateExceptionCandidate(selection,new[]{input},refs,Array.Empty<string>(),bytes);
                    if(item.Id=="payload") {
                        var changed=JsonNode.Parse(bytes)!;var nodes=changed["functions"]![0]!["nodes"]!.AsArray();
                        nodes.First(n=>n!["operation"]!.GetValue<string>()=="closed_exception")!["slot"]="System.ArgumentException";
                        bool rejected=false;try{CSharpPracticalLoopLowering.ValidateExceptionCandidate(selection,new[]{input},refs,Array.Empty<string>(),JsonSerializer.SerializeToUtf8Bytes(changed));}catch(PracticalCaptureFailure e){rejected=e.Code=="loop_lowering_exception_candidate_mismatch";}
                        if(!rejected)throw new Exception("candidate_mutation");
                    }
                    var compilation=CSharpCompilation.Create("Runtime"+rows.Count,new[]{CSharpSyntaxTree.ParseText(source,new CSharpParseOptions((LanguageVersion)1400))},refs,new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary,checkOverflow:true));
                    using var stream=new MemoryStream();if(!compilation.Emit(stream).Success)throw new Exception(item.Id+":runtime_compile");
                    var method=Assembly.Load(stream.ToArray()).GetType("Business.Entry")!.GetMethod("Run")!;
                    var runs=new List<object>();
                    foreach(int n in new[]{-2,-1,0,1,2}) {
                        int? value=null,code=null;string error="";
                        try{value=(int)method.Invoke(null,new object[]{n})!;}catch(TargetInvocationException ex){var e=ex.InnerException!;var t=e.GetType();error=t.FullName!;code=(int?)(t.GetField("Code")?.GetValue(e)??t.GetProperty("Code")?.GetValue(e));}
                        runs.Add(new{n,value,error,code});
                    }
                    rows.Add(new{id=item.Id,source,root,accepted=true,lowering=JsonSerializer.Deserialize<JsonElement>(bytes),runs,diagnostic="",code=""});
                }catch(PracticalCaptureFailure e){if(e.ArtifactCount!=0)throw new Exception("artifact");if(item.Accept)failures.Add(item.Id+":"+e.Family+":"+e.Code);rows.Add(new{id=item.Id,source,root,accepted=false,lowering=(object?)null,runs=Array.Empty<object>(),diagnostic=e.Family.ToString(),code=e.Code});}
            }
            if(failures.Count!=0)throw new Exception(string.Join("\n",failures));
            File.WriteAllBytes("loop-source-cases.json",JsonSerializer.SerializeToUtf8Bytes(rows));return 0;
        }catch(Exception e){Console.Error.WriteLine(e.Message);return 1;}
    }
}
