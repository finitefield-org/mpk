using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using Microsoft.CodeAnalysis;
namespace Mpk.CSharp2Vir;
// Test-only observer: the unchanged stage assertions run before complete data
// replay. Failed pre-capture transport/reference vectors remain tested there.
internal static class PracticalDataReplayHarness
{
    private static readonly SortedDictionary<string,(PracticalSourceSelection selection,PracticalCapturedInput[] inputs,SortedSet<string> stages)> cases = new(StringComparer.Ordinal);
    private static string? stage;
    internal static void Record(PracticalSourceSelection selection,PracticalCapturedInput[] inputs)
    {
        if(stage is null)return;
        var key=Convert.ToHexString(SHA256.HashData(JsonSerializer.SerializeToUtf8Bytes(new {
            schema=selection.Schema,compilation_id=selection.CompilationId,roots=selection.SelectedRootIds,
            source_paths=selection.SourcePaths,sidecar_paths=selection.SidecarPaths,
            inputs=inputs.Select(i=>new{kind=i.Kind,path=i.NormalizedPath,hash=i.RawSha256})}))).ToLowerInvariant();
        if(!cases.TryGetValue(key,out var value))cases.Add(key,value=(selection,inputs,new(StringComparer.Ordinal)));
        value.stages.Add(stage);
    }
    public static int Main(string[] args)
    {
        try {
            var stages=new (string id,Func<string[],int> run)[]{
                ("W01",PracticalCaptureHarness.Main),("W02",PracticalSyntaxHarness.Main),
                ("W03",PracticalTypesHarness.Main),("W04",PracticalConstructionHarness.Main),
                ("W05",PracticalInitializationHarness.Main),("W06",PracticalStructuralHarness.Main),
                ("W07",PracticalArraysHarness.Main),("W08",PracticalSequencesHarness.Main),
                ("W09",PracticalOrderedHarness.Main),("W10",PracticalCodecsHarness.Main),
                ("W11",PracticalNumericHarness.Main),("W12",PracticalDomainHarness.Main),
                ("W13",PracticalBusinessHarness.Main)};
            foreach(var item in stages){stage=item.id;var arguments=item.id=="W06"?new[]{args[0],"source.cs","source-routes.json"}:args;
                if(item.run(arguments)!=0)throw new Exception("stage_"+item.id);}
            stage=null;
            var references=Directory.EnumerateFiles(Path.Combine(args[0],"ref","net10.0"),"*.dll").OrderBy(p=>p,StringComparer.Ordinal).Select(p=>(MetadataReference)MetadataReference.CreateFromFile(p)).ToImmutableArray();
            var rows=new List<object>();
            foreach(var pair in cases){var c=pair.Value;object outcome;
                try {using var facts=JsonDocument.Parse(CSharpPracticalDataPhase.CaptureSelected(c.selection,c.inputs,references).CopyBytes());outcome=new{facts=facts.RootElement.Clone()};}
                catch(PracticalCaptureFailure error){outcome=new{reject=error.Family+"/"+error.Code,phase=error.Phase,artifact_count=error.ArtifactCount};}
                catch(JsonException){outcome=new{reject="sidecar/json",phase=4,artifact_count=0};}
                catch(KeyNotFoundException){outcome=new{reject="sidecar/shape",phase=4,artifact_count=0};}
                rows.Add(new{id=pair.Key,stages=c.stages,compilation_id=c.selection.CompilationId,roots=c.selection.SelectedRootIds,
                    inputs=c.inputs.Select(i=>new{kind=i.Kind==PracticalCapturedInputKind.Source?"source":"sidecar",path=i.NormalizedPath,utf8=Encoding.UTF8.GetString(i.CopyBytes())}),outcome});
            }
            File.WriteAllBytes("data-replay.json",JsonSerializer.SerializeToUtf8Bytes(rows));return 0;
        }catch(Exception error){Console.Error.WriteLine(error);return 1;}
    }
}
