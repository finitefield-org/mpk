using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using System.Reflection;
namespace Mpk.CSharp2Vir;
// Reuse the actual-source/CLR corpora; collect the independently typed data
// handoff for each accepted control selection under the same pinned compiler.
internal static class PracticalControlEmissionHarness
{
    public static int Main(string[] args)
    {
        try {
            var references=Directory.EnumerateFiles(Path.Combine(args[0],"ref","net10.0"),"*.dll")
                .OrderBy(p=>p,StringComparer.Ordinal).Select(p=>MetadataReference.CreateFromFile(p)).ToImmutableArray<MetadataReference>();
            if(File.Exists("control-requests.json")){RunRequests(references);return 0;}
            var rows=new List<object>();
            foreach(var stage in new (string Name,Func<string[],int> Run)[]{
                ("loops",PracticalLoopLoweringHarness.Main),
                ("patterns",PracticalPatternLoweringHarness.Main),
                ("exceptions",PracticalExceptionLoweringHarness.Main),
                ("handlers",PracticalHandlerLoweringHarness.Main)}) {
                if(stage.Run(args)!=0)throw new Exception("predecessor_capture");
                using var cases=JsonDocument.Parse(File.ReadAllBytes("loop-source-cases.json"));
                foreach(var item in cases.RootElement.EnumerateArray()) {
                    var source=item.GetProperty("source").GetString()!;
                    var root=item.GetProperty("root").GetString()!;
                    var selection=new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"data",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>());
                    var input=new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source));
                    JsonElement? data=null;string diagnostic="";
                    try {
                        var prior=item.GetProperty("lowering");
                        var getters=prior.ValueKind==JsonValueKind.Object && prior.TryGetProperty("total_getters",out var claims)?claims.EnumerateArray().Select(c=>c.GetString()!).ToArray():Array.Empty<string>();
                        var first=CSharpPracticalControlPhase.Capture(selection,new[]{input},references,totalGetters:getters).CopyBytes();
                        if(!first.SequenceEqual(CSharpPracticalControlPhase.Capture(selection,new[]{input},references,totalGetters:getters).CopyBytes()))throw new Exception("control_determinism");
                        data=JsonSerializer.Deserialize<JsonElement>(first);
                    } catch(PracticalCaptureFailure e) {diagnostic=e.Family+"/"+e.Code;if(e.ArtifactCount!=0)throw new Exception("artifact");}
                    rows.Add(new{stage=stage.Name,source_case=item.Clone(),accepted=data is not null,data,diagnostic,artifact_count=0});
                }
            }
            File.WriteAllBytes("loop-source-cases.json",JsonSerializer.SerializeToUtf8Bytes(rows));return 0;
        }catch(Exception e){Console.Error.WriteLine(e);return 1;}
    }
    private static void RunRequests(ImmutableArray<MetadataReference> references)
    {
        using var requests=JsonDocument.Parse(File.ReadAllBytes("control-requests.json"));
        var results=new List<object>();
        foreach(var row in requests.RootElement.EnumerateArray()) {
            string id=row.GetProperty("id").GetString()!;
            try {
                var inputs=row.GetProperty("inputs").EnumerateArray().Select(i=>new PracticalCapturedInput(
                    i.GetProperty("kind").GetString()=="source"?PracticalCapturedInputKind.Source:PracticalCapturedInputKind.Sidecar,
                    i.GetProperty("path").GetString()!,Encoding.UTF8.GetBytes(i.GetProperty("utf8").GetString()!))).ToArray();
                var selection=new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,row.GetProperty("compilation_id").GetString()!,
                    inputs.Where(i=>i.Kind==PracticalCapturedInputKind.Source).Select(i=>i.NormalizedPath),
                    row.GetProperty("roots").EnumerateArray().Select(i=>i.GetString()!),
                    inputs.Where(i=>i.Kind==PracticalCapturedInputKind.Sidecar).Select(i=>i.NormalizedPath));
                var first=CSharpPracticalControlPhase.Capture(selection,inputs,references).CopyBytes();
                if(!first.SequenceEqual(CSharpPracticalControlPhase.Capture(selection,inputs,references).CopyBytes()))throw new Exception("determinism");
                if(row.TryGetProperty("runs",out var runInputs)) {
                    var syntax=inputs.Where(i=>i.Kind==PracticalCapturedInputKind.Source).Select(i=>CSharpSyntaxTree.ParseText(Encoding.UTF8.GetString(i.CopyBytes()),new CSharpParseOptions((LanguageVersion)1400)));
                    var compilation=CSharpCompilation.Create("ControlRuntime"+results.Count,syntax,references,new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary,checkOverflow:true,nullableContextOptions:NullableContextOptions.Enable));
                    using var stream=new MemoryStream();if(!compilation.Emit(stream).Success)throw new Exception("runtime_compile");
                    var method=Assembly.Load(stream.ToArray()).GetType("Business.Entry")!.GetMethod("Run")!;
                    var runs=new List<object>();
                    foreach(var run in runInputs.EnumerateArray()) {
                        int n=run.GetProperty("n").GetInt32();int[] a=run.GetProperty("a").EnumerateArray().Select(v=>v.GetInt32()).ToArray();string text=run.GetProperty("s").GetString()!;
                        int? value=null;string error="";
                        try{value=(int)method.Invoke(null,new object[]{n,(int[])a.Clone(),text})!;}catch(TargetInvocationException e){error=e.InnerException!.GetType().Name;}
                        runs.Add(new{n,a,s=text,value,error});
                    }
                    results.Add(new{id,facts=JsonSerializer.Deserialize<JsonElement>(first),runs});
                } else results.Add(new{id,facts=JsonSerializer.Deserialize<JsonElement>(first)});
            } catch(PracticalCaptureFailure e) {
                if(e.ArtifactCount!=0)throw new Exception("artifact");
                results.Add(new{id,reject=e.Family+"/"+e.Code,artifact_count=0});
            }
        }
        File.WriteAllBytes("loop-source-cases.json",JsonSerializer.SerializeToUtf8Bytes(results));
    }
}
