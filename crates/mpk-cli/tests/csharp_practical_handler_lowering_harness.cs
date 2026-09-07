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
using Microsoft.CodeAnalysis.CSharp.Syntax;
namespace Mpk.CSharp2Vir;
internal static class PracticalHandlerLoweringHarness
{
    public static int Main(string[] args)
    {
        try {
            var refs=Directory.EnumerateFiles(Path.Combine(args[0],"ref","net10.0"),"*.dll").OrderBy(p=>p,StringComparer.Ordinal).Select(p=>MetadataReference.CreateFromFile(p)).ToImmutableArray<MetadataReference>();
            string integer=PracticalIdentity.PrimitiveId("i32");
            string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{integer},integer);
            string payload="public sealed class Fault:System.Exception{public readonly int Code;public Fault(int code){Code=code;}}";
            var cases=new List<(string Id,string Declaration,string Body,bool Accept)>();
            cases.AddRange(new (string,string,string,bool)[]{
                ("ordered","","try{if(n<0)throw new System.ArgumentOutOfRangeException();throw new System.ArgumentException();}catch(System.ArgumentOutOfRangeException){return 1;}catch(System.ArgumentException){return 2;}",true),
                ("payload",payload,"try{throw new Fault(n);}catch(Fault e){return e.Code;}",true),
                ("get_only",payload.Replace("public readonly int Code;","public int Code{get;}"),"try{throw new Fault(n);}catch(Fault e){return e.Code;}",true),
                ("init_only",payload.Replace("public readonly int Code;","public int Code{get;init;}"),"try{throw new Fault(n);}catch(Fault e){return e.Code;}",true),
                ("payload_filter",payload,"try{throw new Fault(n);}catch(Fault e)when(e.Code<0){return e.Code;}catch(Fault e){return e.Code+10;}",true),
                ("filter","","try{throw new System.ArgumentException();}catch(System.ArgumentException)when(n>0){return 1;}catch(System.ArgumentException){return 2;}",true),
                ("filter_call","public static class Helper{public static bool Accept(int n){return n>0;}}","try{throw new System.ArgumentException();}catch(System.ArgumentException)when(Helper.Accept(n)){return 1;}catch(System.ArgumentException){return 2;}",true),
                ("filter_failure","public static class Helper{public static bool Accept(int n){throw new System.InvalidOperationException();}}","try{throw new System.ArgumentException();}catch(System.ArgumentException)when(Helper.Accept(n)){return 1;}catch(System.ArgumentException){return 2;}",true),
                ("filter_divide","","try{throw new System.ArgumentException();}catch(System.ArgumentException)when(10/n>0){return 1;}catch(System.ArgumentException){return 2;}",true),
                ("filter_before_finally","","int state=n;try{try{throw new System.ArgumentException();}finally{state=10;}}catch(System.ArgumentException)when(state<0){return state;}catch(System.ArgumentException){return state+1;}",true),
                ("selected_finally","","int state=0;try{try{throw new System.ArgumentException();}catch(System.ArgumentException){state=1;}finally{state=state*10+2;}}finally{state=state*10+3;}return state;",true),
                ("finally_normal","","try{n++;}finally{n++;}return n;",true),
                ("finally_return","","try{return n;}finally{n++;}",true),
                ("finally_throw_return","","try{return n;}finally{throw new System.InvalidOperationException();}",true),
                ("finally_throw_exception","","try{throw new System.ArgumentException();}finally{throw new System.InvalidOperationException();}",true),
                ("finally_restarts_search","","try{try{throw new System.ArgumentException();}finally{throw new System.InvalidOperationException();}}catch(System.ArgumentException){return 1;}catch(System.InvalidOperationException){return 2;}",true),
                ("rethrow",payload,"try{try{throw new Fault(n);}catch(Fault e){n=e.Code+10;throw;}}catch(Fault e){return e.Code;}",true),
                ("catch_throw","","try{try{throw new System.ArgumentException();}catch(System.ArgumentException){throw new System.InvalidOperationException();}}catch(System.InvalidOperationException){return n;}",true),
                ("rethrow_finally","","try{try{throw new System.ArgumentException();}catch(System.ArgumentException){throw;}finally{n++;}}catch(System.ArgumentException){return n;}",true),
                ("loop_finally","","int value=0;for(int i=0;i<3;i++){try{if(i==0)continue;if(i==2)break;value++;}finally{value+=10;}}return value;",true),
                ("finally_inner_loop","","try{return n;}finally{for(int i=0;i<2;i++){if(i==0)continue;break;}}",true),
                ("propagation","public static class Helper{public static int Fail(int n){throw new System.ArgumentException();}}","try{return Helper.Fail(n);}catch(System.ArgumentException){return n+1;}",true),
                ("uncaught","","try{throw new System.ArgumentException();}catch(System.InvalidOperationException){return 1;}finally{n++;}",true),
                ("nested_filter_false","","try{try{throw new System.ArgumentException();}catch(System.ArgumentException)when(n<0){return 1;}finally{n++;}}catch(System.ArgumentException)when(n>=0){return n;}",true),
                ("call_filter_before_finally","public static class Helper{public static int Fail(int n){try{throw new System.ArgumentException();}finally{n++;}}}","try{return Helper.Fail(n);}catch(System.ArgumentException)when(n<0){return 1;}catch(System.ArgumentException){return 2;}",true),
                ("filter_call_finally","public static class Helper{public static bool Accept(int n){try{throw new System.ArgumentException();}finally{n++;}}}","try{throw new System.ArgumentException();}catch(System.ArgumentException)when(Helper.Accept(n)){return 1;}catch(System.ArgumentException){return 2;}",true),
                ("finally_catches_inner_throw","","try{return n;}finally{try{throw new System.ArgumentException();}catch(System.ArgumentException){n++;}}",true),
                ("nested_filter_failures","public static class Helper{public static bool Accept(int n){try{throw new System.ArgumentException();}catch(System.ArgumentException)when(10/n>0){return true;}}}","try{throw new System.ArgumentException();}catch(System.ArgumentException)when(Helper.Accept(n)){return 1;}catch(System.ArgumentException){return 2;}",true),
                ("pattern_fallback","","try{return n switch{0=>1};}catch(System.InvalidOperationException){return 2;}",true),
                ("filter_write","","try{throw new System.ArgumentException();}catch(System.ArgumentException)when(n++>0){return n;}",false),
                ("filter_array_write","public static class Helper{public static bool Accept(int[] a){a[0]=2;return true;}}","int[] a=new int[]{1};try{throw new System.ArgumentException();}catch(System.ArgumentException)when(Helper.Accept(a)){return n;}",false),
                ("filter_nonboolean","","try{throw new System.ArgumentException();}catch(System.ArgumentException)when(n){return 1;}",false),
                ("catch_resource","","try{throw new System.ArgumentException();}catch(System.OutOfMemoryException){return 1;}",false),
                ("catch_all","","try{throw new System.ArgumentException();}catch{return 1;}",false),
                ("payload_identity",payload,"try{throw new Fault(n);}catch(Fault e){return e==null?0:1;}",false),
                ("payload_message",payload,"try{throw new Fault(n);}catch(Fault e){return e.Message.Length;}",false),
                ("payload_escape",payload,"try{throw new Fault(n);}catch(Fault e){throw e;}",false),
                ("rethrow_outside","","throw;",false),
                ("finally_return_reject","","try{return n;}finally{return 0;}",false),
                ("finally_break_reject","","while(n>0){try{n--;}finally{break;}}return n;",false),
                ("finally_continue_reject","","while(n>0){try{n--;}finally{continue;}}return n;",false),
                ("finally_goto_reject","","try{n++;}finally{goto Done;}Done:return n;",false),
            });
            var rows=new List<object>();var failures=new List<string>();
            foreach(var item in cases) {
                string source="namespace Business;"+item.Declaration+"public static class Entry{public static int Run(int n){"+item.Body+"}}\n";
                var input=new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source));
                var selection=new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"data",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>());
                try {
                    byte[] bytes=CSharpPracticalLoopLowering.Capture(selection,new[]{input},refs,true,Array.Empty<string>(),true,true);
                    if(!item.Accept)throw new Exception(item.Id+":unexpected_accept");
                    CSharpPracticalLoopLowering.ValidateHandlerCandidate(selection,new[]{input},refs,Array.Empty<string>(),bytes);
                    if(item.Id=="payload") {
                        var changed=JsonNode.Parse(bytes)!;var nodes=changed["functions"]!.AsArray().SelectMany(f=>f!["nodes"]!.AsArray());
                        nodes.First(n=>n!["operation"]!.GetValue<string>()=="closed_exception")!["slot"]="System.ArgumentException";
                        bool rejected=false;try{CSharpPracticalLoopLowering.ValidateHandlerCandidate(selection,new[]{input},refs,Array.Empty<string>(),JsonSerializer.SerializeToUtf8Bytes(changed));}catch(PracticalCaptureFailure e){rejected=e.Code=="loop_lowering_handler_candidate_mismatch";}
                        if(!rejected)throw new Exception("candidate_mutation");
                    }
                    var compilation=CSharpCompilation.Create("Runtime"+rows.Count,new[]{CSharpSyntaxTree.ParseText(source,new CSharpParseOptions((LanguageVersion)1400))},refs,new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary,checkOverflow:true));
                    using var stream=new MemoryStream();if(!compilation.Emit(stream).Success)throw new Exception(item.Id+":runtime_compile");
                    var method=Assembly.Load(stream.ToArray()).GetType("Business.Entry")!.GetMethod("Run")!;
                    var rewritten=new TraceRewriter().Visit(CSharpSyntaxTree.ParseText(source).GetRoot())!;
                    var tracedCompilation=compilation.WithAssemblyName("Traced"+rows.Count).RemoveAllSyntaxTrees().AddSyntaxTrees(CSharpSyntaxTree.Create((CSharpSyntaxNode)rewritten),CSharpSyntaxTree.ParseText(TraceSupport));
                    using var traceStream=new MemoryStream();var emitted=tracedCompilation.Emit(traceStream);
                    if(!emitted.Success)throw new Exception(item.Id+":trace_compile:"+string.Join("|",emitted.Diagnostics));
                    var traced=Assembly.Load(traceStream.ToArray());var traceType=traced.GetType("Business.W05Trace")!;
                    var traceMethod=traced.GetType("Business.Entry")!.GetMethod("Run")!;
                    var runs=new List<object>();
                    foreach(int n in new[]{-2,-1,0,1,2}) {
                        int? value=null,code=null;string error="";
                        try{value=(int)method.Invoke(null,new object[]{n})!;}catch(TargetInvocationException ex){var e=ex.InnerException!;var t=e.GetType();error=t.FullName!;code=(int?)(t.GetField("Code")?.GetValue(e)??t.GetProperty("Code")?.GetValue(e));}
                        var trace=(List<string>)traceType.GetField("Events")!.GetValue(null)!;trace.Clear();
                        int? tracedValue=null,tracedCode=null;string tracedError="";
                        try{tracedValue=(int)traceMethod.Invoke(null,new object[]{n})!;}catch(TargetInvocationException ex){var e=ex.InnerException!;var t=e.GetType();tracedError=t.FullName!;tracedCode=(int?)(t.GetField("Code")?.GetValue(e)??t.GetProperty("Code")?.GetValue(e));}
                        if((value,error,code)!=(tracedValue,tracedError,tracedCode))throw new Exception("trace_changed_outcome");
                        runs.Add(new{n,value,error,code,trace=trace.ToArray()});
                    }
                    rows.Add(new{id=item.Id,source,root,accepted=true,lowering=JsonSerializer.Deserialize<JsonElement>(bytes),runs,diagnostic="",code=""});
                }catch(PracticalCaptureFailure e){if(e.ArtifactCount!=0)throw new Exception("artifact");if(item.Accept)failures.Add(item.Id+":"+e.Family+":"+e.Code);rows.Add(new{id=item.Id,source,root,accepted=false,lowering=(object?)null,runs=Array.Empty<object>(),diagnostic=e.Family.ToString(),code=e.Code});}
            }
            if(failures.Count!=0)throw new Exception(string.Join("\n",failures));
            File.WriteAllBytes("loop-source-cases.json",JsonSerializer.SerializeToUtf8Bytes(rows));return 0;
        }catch(Exception e){Console.Error.WriteLine(e.Message);return 1;}
    }
    private const string TraceSupport="""
        namespace Business;
        public static class W05Trace {
            public static readonly System.Collections.Generic.List<string> Events=new();
            public static void Log(string value){Events.Add(value);}
            public static bool Filter(string id,System.Func<bool> evaluate){
                Log(id+":enter");try{bool result=evaluate();Log(id+":"+(result?"true":"false"));return result;}
                catch(System.Exception e){Log(id+":throw:"+e.GetType().FullName);throw;}
            }
        }
        """;
    private sealed class TraceRewriter:CSharpSyntaxRewriter
    {
        public override SyntaxNode? VisitTryStatement(TryStatementSyntax node) {
            var method=node.Ancestors().First(n=>n is BaseMethodDeclarationSyntax);
            int id=Array.IndexOf(method.DescendantNodes().OfType<TryStatementSyntax>().ToArray(),node);
            var rewritten=(TryStatementSyntax)base.VisitTryStatement(node)!;
            BlockSyntax Log(BlockSyntax block,string value)=>block.WithStatements(block.Statements.Insert(0,SyntaxFactory.ParseStatement("W05Trace.Log(\""+value+"\");")));
            rewritten=rewritten.WithBlock(Log(rewritten.Block,"try:"+id));
            for(int i=0;i<rewritten.Catches.Count;i++) {
                var caught=rewritten.Catches[i];var updated=caught.WithBlock(Log(caught.Block,"catch:"+id+":"+i));
                if(caught.Filter is not null)updated=updated.WithFilter(caught.Filter.WithFilterExpression(SyntaxFactory.ParseExpression("W05Trace.Filter(\"filter:"+id+":"+i+"\",()=> ("+caught.Filter.FilterExpression.ToFullString()+"))")));
                rewritten=rewritten.ReplaceNode(caught,updated);
            }
            if(rewritten.Finally is not null)rewritten=rewritten.WithFinally(rewritten.Finally.WithBlock(Log(rewritten.Finally.Block,"finally:"+id)));
            return rewritten;
        }
    }

}
