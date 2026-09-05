using System;
using System.Collections.Immutable;
using System.Collections.Generic;
using System.Text.Json;
using Microsoft.CodeAnalysis.Operations;
using System.IO;
using System.Linq;
using System.Text;
using Microsoft.CodeAnalysis;

namespace Mpk.CSharp2Vir;
internal static class PracticalSequencesHarness
{
    private static ImmutableArray<MetadataReference> references;
    private static string stage = "start";
    public static int Main(string[] arguments)
    {
        try {
            references = Directory.EnumerateFiles(Path.Combine(arguments[0],"ref","net10.0"),"*.dll")
                .OrderBy(path=>path,StringComparer.Ordinal).Select(path=>MetadataReference.CreateFromFile(path)).ToImmutableArray<MetadataReference>();
            foreach (string body in new[]{"var a=new int[0]; return a.Length;", "var a=new int[4096]; return a.Length;",
                "int[] a={input,2}; return a[0];", "var a=new[]{input,2}; return a[1];",
                "var a=new string[2]; a[0]=\"a\"; a[1]=\"b\"; return a.Length;",
                "var a=new string?[2]; return a.Length;", "var a=new int[input]; a[0]=input; return a[0];",
                "var a=new int[1]; if(input>0){a[0]=1;}else{a[0]=2;} return a[0];"}) {
                stage=body; var plan=Run(body); Check(plan.ArtifactCount==0,"ARTIFACT");
                Check(plan.Steps.Any(s=>s.Action=="allocate" && s.Types.Count==1),"TYPED_ALLOCATION");
                var types=plan.Steps.First(s=>s.Action=="allocate").Types.Single();
                Check(types.ConstructionTypeId==PracticalIdentity.ClosedInstanceId("sequence_construction",types.ElementTypeId),"INSTANCE");
                byte[] bytes=plan.CopyCanonicalBytes();
                CSharpPracticalSequences.ValidateCandidate(Run(body),bytes);
                bytes[bytes.Length/2]^=1;
                RejectAction(()=>CSharpPracticalSequences.ValidateCandidate(plan,bytes));
            }
            stage="direct_result";
            var result=Run("return Make(input).Length;",extra:"public static int[] Make(int n){return new[]{1,2};}");
            Check(result.Steps.Any(s=>s.Action=="freeze" && s.Source.Operation=="return_transfer" && s.Types.Count==1),"RETURN_FREEZE");
            Check(result.Steps.Where(s=>s.Action=="fill_initializer").Select(s=>s.InitializerIndex).SequenceEqual(new int?[]{0,1}),"FILL_ORDER");
            var allocated=result.Steps.Single(s=>s.Action=="allocate");
            var directType=allocated.Types.Single();
            var direct=new SortedDictionary<string,object>(StringComparer.Ordinal) {
                ["construction_type_id"]=directType.ConstructionTypeId, ["element_type_id"]=directType.ElementTypeId,
                ["sequence_type_id"]=directType.SequenceTypeId,
                ["length"]=((IArrayCreationOperation)allocated.Source.Source).DimensionSizes[0].ConstantValue.Value!,
                ["values"]=result.Steps.Where(s=>s.Action=="fill_initializer").Select(s=>s.Source.Operand!.ConstantValue.Value).ToArray(),
            };
            Check(JsonSerializer.Serialize(direct)+"\n"==File.ReadAllText("source-direct.json"),"SOURCE_TO_VIR_BRIDGE");
            stage="wrapper";
            string box="public sealed class Box {public readonly int[] Items; public readonly int Tag; public Box(int[] items,int tag){Items=items;Tag=tag;}}";
            var binding=new PracticalSequenceBinding(PracticalIdentity.SourceTypeId("Business","Box"),"Items");
            var wrapper=Run("var a=new[]{input}; var b=new Box(a,input); return b.Items[0];",declarations:box,binding:binding);
            Check(wrapper.Projections.Count==1 && wrapper.Steps.Any(s=>s.Action=="freeze" && s.Source.Operation=="wrapper_freeze"),"WRAPPER");
            RejectAction(()=>Run("var b=new Box(new[]{input},input); return b.Items.Length;",declarations:box,binding:binding with {ElementsMember="Tag"}));
            RejectAction(()=>Run("return input;",binding:binding));
            RejectAction(()=>Run("var b=new Box(new[]{input},new[]{input}); return b.Items.Length;",declarations:box.Replace("int Tag","int[] Tag").Replace("int tag","int[] tag"),binding:binding));
            stage="immutable_alias";
            var alias=Run("return Make(input).Length;",extra:"public static int[] Make(int n){var a=new[]{n}; var b=a; return b;}");
            Check(alias.Steps.Count(s=>s.Action=="freeze")==1 && alias.Steps.Any(s=>s.Action=="publish_immutable" && s.Types.All(t=>t.ConstructionTypeId=="")),"SINGLE_FREEZE");
            var parameter=Run("return Use(new[]{input});",extra:"public static int Use(int[] a){return a[0];}");
            Check(parameter.Steps.Any(s=>s.Action=="indexed_read" && s.Types.Count==1 && s.Types[0].ConstructionTypeId==""),"READONLY_PROJECTION");
            stage="source_rejections";
            foreach(string body in new[]{"var a=new string[1]; return a.Length+Use(a);", "var a=new string[1]; return a[0].Length;",
                "var a=new int[1]; var b=a; a[0]=2; return b[0];", "var a=new int[4097]; return a.Length;",
                "var a=new System.Collections.Generic.List<int>(); a.Add(input); return a.Count;",
                "var a=System.Collections.Immutable.ImmutableArray.Create(input); return a.Length;",
                "System.Func<int,int> f=x=>x; return f(input);",
                "int count=0; for(int i=0;i<input;i++){if(i>0)count++;} var a=new int[count]; int j=0; for(int i=0;i<input;i++){if(i>0)a[j++]=i;} return a.Length;"}) {
                RejectAction(()=>Run(body,extra:body.Contains("Use(",StringComparison.Ordinal)?"public static int Use(string[] a){return a.Length;}":""));
            }
            Check(CSharpPracticalSequences.TwoPassOwner=="CSHARP-03-T04-W01/W02","TWO_PASS_OWNER");
            stage="merge_rejection";
            RejectAction(()=>Run("var a=new int[1]; if(input>0){a[0]=1;} return a[0];"));
            RejectAction(()=>Run("var a=new int[1]; if(input>0){var b=a;} return a[0];"));
            stage="live_limits";
            string live=string.Concat(Enumerable.Range(0,8).Select(i=>$"var a{i}=new int[0];"));
            Run(live+"return input;"); RejectAction(()=>Run(live+"var a8=new int[0]; return input;"));
            stage="method_limits";
            string sequential=string.Concat(Enumerable.Range(0,32).Select(i=>$"var a{i}=new int[0]; var b{i}=a{i};"));
            Run(sequential+"return input;"); RejectAction(()=>Run(sequential+"var a32=new int[0]; return input;"));
            stage="symbolic_obligations";
            var symbolic=Run("return Make(input).Length;",extra:"public static string[] Make(int n){var a=new string[n]; a[0]=\"x\"; return a;}");
            Check(symbolic.Steps.Any(s=>s.Source.Operation=="complete_publication_vc") && symbolic.Steps.Any(s=>s.Source.Operation=="profile_bound"),"PENDING_VCS");
            return 0;
        } catch(Exception error) { Console.Error.WriteLine("SEQUENCES_"+stage+"_"+(error is PracticalCaptureFailure f ? f.Family+"_"+f.Code : error.ToString())); return 1; }
    }
    private static PracticalSequences Run(string body,string extra="",string declarations="",PracticalSequenceBinding? binding=null)
    {
        string source="namespace Business; "+declarations+"public static class Entry {public static int Run(int input){"+body+"}"+extra+"}\n";
        string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{PracticalIdentity.PrimitiveId("i32")},PracticalIdentity.PrimitiveId("i32"));
        return CSharpPracticalSequences.Validate(new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"business",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>()),
            new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source))},references,
            binding is null ? null : new[]{binding});
    }
    private static void RejectAction(Action action) { bool rejected=false; try{action();}catch(PracticalCaptureFailure f){if(f.Family==PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL)throw;rejected=true;} Check(rejected,"EXPECTED_REJECTION"); }
    private static void Check(bool value,string code) {if(!value)throw new Exception(code);}
}
