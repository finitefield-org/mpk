using System;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Text;
using Microsoft.CodeAnalysis;

namespace Mpk.CSharp2Vir;
internal static class PracticalArraysHarness
{
    private static ImmutableArray<MetadataReference> references;
    private static string stage = "start";
    private const string Data = "public sealed class Data { public readonly int X; public Data(int x) { X=x; } } ";
    public static int Main(string[] arguments)
    {
        try {
            references = Directory.EnumerateFiles(Path.Combine(arguments[0],"ref","net10.0"),"*.dll")
                .OrderBy(path=>path,StringComparer.Ordinal).Select(path=>MetadataReference.CreateFromFile(path)).ToImmutableArray<MetadataReference>();
            foreach (string body in new[]{
                "var a=new int[0]; return a.Length;", "var a=new int[4095]; return a.Length;", "var a=new int[4096]; a[4095]=input; return a[4095];",
                "var a=new int[input]; a[0]=1; return a.Length;", "int[] a={input,2}; return a[0];",
                "var a=new int[]{input,2}; return a[1];", "var a=new[]{input,2}; return a[1];",
                "var a=new Data[0]; return a.Length;", "var a=new Data[2]; a[0]=new Data(1); int x=a[0].X; a[1]=new Data(x); a[0]=new Data(3); return a[1].X;",
                "var a=new Data[]{new Data(input)}; a[0]=new Data(3); return a[0].X;",
                "var a=new Data[1]; if(input>0){a[0]=new Data(1);}else{a[0]=new Data(2);} return a[0].X;",
                "var a=new int[1]; a[0]=2; var b=a; return b[0];",
                "var a=input>0 ? new int[1] : new int[2]; a[0]=input; return a[0];",
                "var a=new int[1]; a[0]++; a[0]+=input; return a[0];",
                "var a=new int[1]; var b=a; a=new int[2]; a[0]=input; return b[0]+a[0];",
                "var a=new string?[2]; return a.Length;", "var a=new int?[2]; return a.Length;",
                "var a=new string[1]; a[0]=\"x\"; return a[0].Length;",
                "var a=new string[input]; a[0]=\"x\"; a[0]=\"y\"; return a[0].Length;",
                "int n=-1; var a=new int[n]; return a.Length;"
            }) { stage=body; var result=Run(body); Check(result.ArtifactCount==0,"PRIVATE"); Check(result.Steps.Count>0,"PLANS"); }
            foreach (string body in new[]{
                "var a=new int[4097]; return a.Length;", "var a=new int[1]{1}; return a.Length;", "var a=new Data[1]; return a[0].X;",
                "var a=new Data[2]; a[0]=new Data(1); a[0]=new Data(2); return a.Length;",
                "var a=new Data[1]; var b=a; return b.Length;",
                "var a=new Data[1]; if(input>0){a[0]=new Data(1);} return a[0].X;",
                "var a=new int[1]; var b=a; b[0]=2; return a[0];",
                "var a=new int[1]; var b=a; a[0]=2; return b[0];",
                "var a=new int[1]; if(input>0){var b=a;} a[0]=2; return a[0];",
                "var a=new int[1]; Use(a); a[0]=2; return a[0];",
                "var a=new Data[1]; UseData(a); return a.Length;",
                "var a=new int[1,1]; return a[0,0];", "var a=new int[1][]; return a.Length;",
                "var a=new int[][]{new int[1]}; return a.Length;", "System.Array a=new int[1]; return a.Length;",
                "object[] a=new string[]{\"x\"}; return a.Length;",
                "System.Span<int> a=stackalloc int[1]; return a.Length;", "System.Memory<int> a=new int[1]; return a.Length;",
                "var a=new int[1]; return a[^1];", "var a=new int[1]; return a[0..1].Length;",
                "int[] a=[1,2]; return a.Length;", "var a=System.Array.Empty<int>(); return a.Length;",
                "var a=new[]{1,2L}; return a.Length;", "return UseLong(new[]{1,2L});",
                "short n=1; var a=new int[n]; return a.Length;", "var a=new int[1]; short n=0; return a[n];",
                "var a=new int[1]; uint n=0; return a[n];", "var a=new int[1]; foreach(int x in a){a[0]=x;} return a[0];",
                "var a=new Data[1]; for(int i=0;i<1;i++){a[i]=new Data(i);} return a.Length;"
            }) { stage=body; Reject(body); }
            stage="publication";
            Run("return Make(input).Length;", "public static int[] Make(int n){var a=new int[n]; return a;}");
            Reject("var a=Make(input); a[0]=1; return a.Length;", "public static int[] Make(int n){return new int[n];}");
            Reject("return Make(input).Length;", "public static string[] Make(int n){var a=new string[1]; return a;}");
            Reject("var a=new int[1]; return Mutate(a);", "public static int Mutate(int[] a){a[0]=1; return a[0];}");
            Run("var a=new int[1]; return Mutate(a);", "public static int Mutate(int[] source){var a=source; a=new int[1]; a[0]=1; return a[0];}");
            Reject("(new int[1])[0]=1; return input;");
            Reject("var a=new int[1]; return Mutate(a);", "public static int Mutate(int[] source){int[]? a=null; a??=source; a[0]=1; return a[0];}");
            Reject("var a=new int[1]; if(a is {} b){a[0]=1; return b[0];} return 0;");
            stage="storage";
            const string box="public sealed class Box { public readonly int[] Items; public Box(int[] items){Items=items;} public int[] Value => Items; }";
            Run("var a=new int[1]; a[0]=input; var b=new Box(a); return b.Value[0];", declarations:box);
            Reject("var a=new int[1]; var b=new Box(a); a[0]=input; return b.Value[0];", declarations:box);
            const string initBox="public sealed class Box { public required int[] Items {get;init;} }";
            Run("var a=new int[1]; var b=new Box {Items=a}; return b.Items.Length;", declarations:initBox);
            Reject("var a=new int[1]; var b=new Box {Items=a}; a[0]=input; return b.Items.Length;", declarations:initBox);
            Reject("return (new Box()).Items[0].Length;", declarations:"public sealed class Box {public string[] Items => new string[1];}");
            Reject("return (new Box()).Items.Length;", declarations:"public sealed class Box {public readonly int[] Items=new int[1];}");
            Reject("return (new Box()).Items.Length;", declarations:"public sealed class Box {public readonly string[] Items=new string[1];}");
            stage="symbolic_initialization";
            var symbolic=Run("return Make(input).Length;", "public static string[] Make(int n){var a=new string[n]; a[0]=\"x\"; return a;}");
            Check(symbolic.Steps.Any(step=>step.Operation=="complete_publication_vc"),"SYMBOLIC_COMPLETENESS_VC");
            var symbolicIndex=Run("return Make(input).Length;", "public static string[] Make(int n){var a=new string[1]; a[n]=\"x\"; return a;}");
            Check(symbolicIndex.Steps.Any(step=>step.Operation=="complete_publication_vc"),"SYMBOLIC_INDEX_VC");
            stage="source_limits";
            try { Run("var a=new int[4097]; return a.Length;"); throw new Exception("LIMIT_ACCEPTED"); }
            catch(PracticalCaptureFailure failure) { Check(failure.Family==PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT && failure.Code=="array_elements","LIMIT_FAMILY"); }
            try { Run("var a=new int[4097]; return System.Environment.TickCount+a.Length;"); throw new Exception("LIMIT_ACCEPTED"); }
            catch(PracticalCaptureFailure failure) { Check(failure.Family==PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT,"LIMIT_PRECEDENCE"); }
            stage="initializer_limit";
            Reject("var a=new int[]{"+string.Join(",",Enumerable.Repeat("0",4097))+"}; return a.Length;");
            stage="borrow";
            bool conflict=false; try { CSharpPracticalArrays.RequireWritable(true,true); }
            catch(PracticalCaptureFailure failure) { conflict=failure.Code==CSharpPracticalArrays.ForeachWriteConflict; }
            Check(conflict,"BORROW_HANDOFF"); CSharpPracticalArrays.RequireWritable(true,false);
            stage="ordered";
            var ordered=Run("var a=new int[input]; a[0]=input; return a[0];").Steps;
            string[] allocation=ordered.Take(4).Select(step=>step.Operation).ToArray();
            Check(string.Join(",",allocation)=="evaluate_length,csharp_length_check,profile_bound,allocate_unique","ALLOCATION_ORDER");
            Check(ordered.Single(step=>step.Operation=="csharp_length_check").Exception=="OverflowException","LENGTH_EXCEPTION");
            Check(ordered.Any(step=>step.Operation=="profile_bound" && step.Exception==""),"PROFILE_VC");
            foreach(int position in Enumerable.Range(0,ordered.Count).Where(i=>ordered[i].Operation=="read" || ordered[i].Operation=="functional_update")) {
                Check(ordered[position-3].Operation=="null_check" && ordered[position-2].Operation=="index_lower_bound"
                    && ordered[position-1].Operation=="index_upper_bound","ACCESS_ORDER");
            }
            stage="store_evaluation_order";
            var store=Run("var a=new int[1]; a[0]=input; return a.Length;").Steps;
            int rhs=Enumerable.Range(0,store.Count).Single(i=>store[i].Operation=="evaluate_value");
            Check(store[rhs+1].Operation=="null_check" && store[rhs+4].Operation=="functional_update", "RHS_BEFORE_STORE_CHECKS");
            Check(store[rhs+4].Predicate=="element_public_invariant","REWRITE_INVARIANT");
            stage="runtime";
            int touched=0; int[]? missing=null;
            try { missing![0]=(touched=1); } catch(NullReferenceException) {}
            Check(touched==1,"STORE_NULL_TIMING");
            try { (new int[0])[0]=(touched=2); } catch(IndexOutOfRangeException) {}
            Check(touched==2,"STORE_BOUNDS_TIMING");
            int negative=-1; try { _=new int[negative]; throw new Exception("NEGATIVE_ACCEPTED"); } catch(OverflowException) {}
            foreach(int index in new[]{-1,1}) { try { _=(new int[1])[index]; throw new Exception("INDEX_ACCEPTED"); } catch(IndexOutOfRangeException) {} }
            return 0;
        } catch(Exception error) { Console.Error.WriteLine("ARRAYS_"+stage+"_"+(error is PracticalCaptureFailure f ? f.Family+"_"+f.Code : error.ToString())); return 1; }
    }
    private static PracticalArrays Run(string body, string extra = "", string declarations = "")
    {
        if(body.Contains("Data",StringComparison.Ordinal)) { body="input=new Data(input).X; "+body; }
        string helpers = (body.Contains("Use(",StringComparison.Ordinal) ? "public static void Use(int[] a){} " : "")
            + (body.Contains("UseData(",StringComparison.Ordinal) ? "public static void UseData(Data[] a){} " : "")
            + (body.Contains("UseLong(",StringComparison.Ordinal) ? "public static int UseLong(long[] a){return a.Length;} " : "");
        string source="namespace Business; "+declarations+(body.Contains("Data",StringComparison.Ordinal) ? Data : "")
            +"public static class Entry { public static int Run(int input){"+body+"} "+helpers+extra+"}\n";
        string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{PracticalIdentity.PrimitiveId("i32")},PracticalIdentity.PrimitiveId("i32"));
        return CSharpPracticalArrays.Validate(new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"business",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>()),
            new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source))},references);
    }
    private static void Reject(string body, string extra = "", string declarations = "") { bool rejected=false; try{Run(body,extra,declarations);}catch(PracticalCaptureFailure failure){ if(failure.Family==PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL) { throw; } rejected=true; } Check(rejected,"EXPECTED_REJECTION"); }
    private static void Check(bool value,string code) { if(!value) throw new Exception(code); }
}
