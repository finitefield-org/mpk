using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using Microsoft.CodeAnalysis;

namespace Mpk.CSharp2Vir;
internal static class PracticalOrderedHarness
{
    private static ImmutableArray<MetadataReference> references;
    private static string stage="start";
    private const string Entry="public readonly struct Pair {public readonly int Key; public readonly int Value; public Pair(int key,int value){Key=key;Value=value;}}";
    private const string Wrapper="public sealed class Box {public readonly Pair[] Items;public readonly int Tag;public Box(Pair[] items,int tag){Items=items;Tag=tag;}}";
    private static string Value(string name)=>PracticalIdentity.PrimitiveId(name);
    private static string Source(string name)=>PracticalIdentity.SourceTypeId("Business",name);
    private static PracticalOrderedBinding Map()=>new("ordered_map",PracticalIdentity.ClosedInstanceId("bounded_sequence",Source("Pair")),Source("Pair"),"Key","Value");
    private static PracticalOrderedBinding Set(string element="i32")=>new("ordered_set",PracticalIdentity.ClosedInstanceId("bounded_sequence",Value(element)));
    public static int Main(string[] arguments)
    {
        try {
            references=Directory.EnumerateFiles(Path.Combine(arguments[0],"ref","net10.0"),"*.dll").OrderBy(p=>p,StringComparer.Ordinal).Select(p=>MetadataReference.CreateFromFile(p)).ToImmutableArray<MetadataReference>();
            stage="direct_map";
            var map=Run("var a=new[]{new Pair(1,10),new Pair(3,30)}; return a.Length;",new[]{Map()},Entry);
            var set=Run("var a=new[]{1,3}; return a.Length;",new[]{Set()});
            CheckFixture(new[]{map,set});
            foreach(var result in new[]{map,set}) {
                Check(result.ArtifactCount==0 && result.Projections.Count==1,"PRIVATE_PROJECTION");
                foreach(string kind in new[]{"length_le_4096","total_collection_cells_le_65536","strictly_increasing_keys","unique_keys","element_public_invariants"})
                {Check(result.Obligations.Any(o=>o.Kind==kind)&&result.Obligations.All(o=>!o.Discharged),"PUBLICATION_VCS");}
                Check(result.Operations.Single(o=>o.Name=="add").OrderedOutcomes.First()=="invalid_representation","OUTCOME_ORDER");
                byte[] bytes=result.CopyCanonicalBytes();CSharpPracticalOrderedCollections.ValidateCandidate(result,bytes);bytes[bytes.Length/2]^=1;
                Reject(()=>CSharpPracticalOrderedCollections.ValidateCandidate(result,bytes));
            }
            CSharpPracticalOrderedCollections.ValidateCandidate(map,Run("var a=new[]{new Pair(1,10),new Pair(3,30)}; return a.Length;",new[]{Map()},Entry).CopyCanonicalBytes());
            stage="publication_and_input";
            var published=Run("return new[]{new Pair(1,10),new Pair(3,30)};",new[]{Map()},Entry,"Pair[]",Map().ArrayTypeId);
            var boundaries=published.Sequences.Steps.Where(s=>s.Action is "freeze" or "publish_immutable").ToArray();
            Check(boundaries.Length!=0,"PUBLICATION_REQUIRED");
            foreach(var boundary in boundaries) {Check(published.Obligations.Any(o=>o.Site==boundary.Source.Path+"/"+boundary.Source.Site && o.Kind=="unique_keys"&&!o.Discharged),"EACH_PUBLICATION");}
            var parameter=Run("return input.Length;",new[]{Set()},"","int",Value("i32"),"int[]",Set().ArrayTypeId);
            Check(parameter.Obligations.Any(o=>o.Kind=="strictly_increasing_keys"&&!o.Discharged),"INPUT_OBLIGATIONS");
            stage="wrappers_and_extras";
            var wrapped=Run("var a=new[]{new Pair(1,10)}; var b=new Box(a,2);return b.Items.Length;",new[]{Map() with {WrapperTypeId=Source("Box"),WrapperMember="Items"}},Entry+Wrapper);
            Check(wrapped.Projections.Single().WrapperSourceSha256.Length==64,"WRAPPER_HASH");
            Check(wrapped.Obligations.Any(o=>o.Member=="Tag" && o.Kind=="field_complete_reconstruction"),"WRAPPER_EXTRA");
            var extra=Run("var a=new[]{new Pair(1,2)};return a.Length;",new[]{Map()},Entry.Replace("public readonly int Value;","public readonly int Value;public readonly int Extra;").Replace("Value=value;","Value=value;Extra=0;"));
            Check(extra.Obligations.Any(o=>o.Member=="Extra"&&o.Kind=="field_complete_reconstruction"),"ENTRY_EXTRA");
            stage="nullable_values";
            var nullable=Run("var a=new[]{new Pair(1,null)}; return a.Length;",new[]{Map()},Entry.Replace("int Value","string? Value").Replace("int value","string? value"));
            string option=PracticalIdentity.ClosedInstanceId("option",Value("string"));
            Check(nullable.Projections.Single().ValueTypeId==option && nullable.Projections.Single().LookupTypeId==PracticalIdentity.ClosedInstanceId("lookup",option),"FOUND_NULL");
            stage="conditional_map_order";
            var floats=Run("var a=new[]{new Pair(1,1.0f)};return a.Length;",new[]{Map()},Entry.Replace("int Value","float Value").Replace("int value","float value"));
            Check(floats.Operations.Any(o=>o.Name=="equal")&&!floats.Operations.Any(o=>o.Name=="compare"),"NON_TOTAL_VALUE");
            stage="source_boundaries";
            foreach(string body in new[]{"var a=new int[0];return a.Length;","var a=new int[4096];return a.Length;","var a=new[]{1,1};return a.Length;","var a=new[]{3,1};return a.Length;"}) {
                // Constant duplicate/order failures remain obligations for T06;
                // the concrete projection validator rejects their publication.
                var result=Run(body,new[]{Set()});Check(result.Obligations.Any(o=>o.Kind=="unique_keys"&&!o.Discharged),"DIRECT_OBLIGATION");
            }
            Reject(()=>Run("var a=new int[4097];return a.Length;",new[]{Set()}));
            stage="binding_mutations";
            foreach(var binding in new[]{Map() with {Role="dictionary"},Map() with {KeyMember="Value"},Map() with {ValueMember="Missing"},Map() with {EntryTypeId=Source("Missing")},Map() with {ArrayTypeId=Set().ArrayTypeId}})
            {Reject(()=>Run("var a=new[]{new Pair(1,2)};return a.Length;",new[]{binding},Entry));}
            Reject(()=>Run("var a=new[]{1,2};return a.Length;",new[]{Set(),Set()}));
            Reject(()=>Run("var a=new[]{1.0f};return a.Length;",new[]{Set("f32")}));
            Reject(()=>Run("var a=new[]{new Pair(1.0f,1)};return a.Length;",new[]{Map()},Entry.Replace("int Key","float Key").Replace("int key","float key")));
            Reject(()=>Run("var a=new[]{new Pair(1,2)};var b=new Box(a,2);return b.Items.Length;",new[]{Map() with {WrapperTypeId=Source("Box"),WrapperMember="Tag"}},Entry+Wrapper));
            Reject(()=>Run("var a=new[]{new Pair(1,2)};return a.Length;",new[]{Map()},Entry.Replace("readonly struct","struct").Replace("readonly int","int")));
            Reject(()=>Run("var a=new[]{new Pair<int>(1,2)};return a.Length;",new[]{Map()},"public readonly struct Pair<T>{public readonly T Key;public readonly int Value;public Pair(T key,int value){Key=key;Value=value;}}"));
            Reject(()=>Run("var a=new[]{new Pair(1,2)};return a.Length;",new[]{Map() with {KeyMember="Computed"}},Entry.Replace("public readonly int Key;","public readonly int Key;public int Computed=>Key;")));
            stage="source_loop_and_framework_rejections";
            foreach(string body in new[]{"var a=new[]{1,2};int count=0;foreach(int x in a){count++;}return count;",
                "var a=new[]{1,2};for(int i=0;i<a.Length;i++){if(a[i]==input)return i;}return -1;",
                "var a=new System.Collections.Generic.Dictionary<int,int>();a.Add(1,2);return a.Count;",
                "var a=new System.Collections.Generic.HashSet<int>();a.Add(1);return a.Count;",
                "var a=new[]{1,2};System.Array.Sort(a);return a.Length;",
                "return System.Collections.Generic.Comparer<int>.Default.Compare(1,2);",
                "return input.GetHashCode();", "System.Func<int,int> f=x=>x;return f(input);"}) {Reject(()=>Run(body,new[]{Set()}));}
            Check(CSharpPracticalOrderedCollections.LoopOwner=="CSHARP-03-T04-W01/W02","LOOP_OWNER");
            return 0;
        }catch(Exception error){Console.Error.WriteLine("ORDERED_"+stage+"_"+(error is PracticalCaptureFailure f?f.Family+"_"+f.Code:error.ToString()));return 1;}
    }
    private static PracticalOrderedCollections Run(string body,PracticalOrderedBinding[] bindings,string declarations="",string returnType="int",string returnId="",string parameterType="int",string parameterId="")
    {
        string source="namespace Business; "+declarations+"public static class Entry {public static "+returnType+" Run("+parameterType+" input){"+body+"}}\n";
        string root=PracticalIdentity.CallableId("method","Business",Source("Entry"),"Run",new[]{parameterId.Length==0?Value("i32"):parameterId},returnId.Length==0?Value("i32"):returnId);
        return CSharpPracticalOrderedCollections.Validate(new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"business",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>()),
            new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source))},references,bindings);
    }
    private static void CheckFixture(PracticalOrderedCollections[] results)
    {
        var rows=results.Select(result=>{
            var p=result.Projections.Single();
            return new SortedDictionary<string,object>(StringComparer.Ordinal) {
                ["semantic_type_id"]=p.SemanticTypeId,["key_type_id"]=p.KeyTypeId,["value_type_id"]=p.ValueTypeId,
                ["entry_semantic_type_id"]=p.EntrySemanticTypeId,["lookup_type_id"]=p.LookupTypeId,
                ["operations"]=result.Operations.Select(o=>new SortedDictionary<string,object>(StringComparer.Ordinal){
                    ["name"]=o.Name,["argument_type_ids"]=o.ArgumentTypeIds,["result_type_id"]=o.ResultTypeId,["ordered_outcomes"]=o.OrderedOutcomes,
                }).ToArray(),
            };
        }).OrderBy(row=>(string)row["semantic_type_id"],StringComparer.Ordinal).ToArray();
        Check(JsonSerializer.Serialize(rows)+"\n"==File.ReadAllText("source-ordered.json"),"SOURCE_FOUNDATION_BRIDGE");
    }
    private static void Reject(Action action){bool rejected=false;try{action();}catch(PracticalCaptureFailure f){if(f.Family==PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL)throw;rejected=true;}Check(rejected,"EXPECTED_REJECTION");}
    private static void Check(bool value,string code){if(!value)throw new Exception(code);}
}
