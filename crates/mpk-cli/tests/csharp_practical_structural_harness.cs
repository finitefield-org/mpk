using System;
using System.Collections.Immutable;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;

namespace Mpk.CSharp2Vir;
internal static class PracticalStructuralHarness
{
    private static ImmutableArray<MetadataReference> references;
    private static string stage="routing";
    public static int Main(string[] arguments)
    {
        try {
            references = Directory.EnumerateFiles(Path.Combine(arguments[0], "ref", "net10.0"), "*.dll")
                .OrderBy(path=>path,StringComparer.Ordinal).Select(path=>MetadataReference.CreateFromFile(path)).ToImmutableArray<MetadataReference>();
            string source = File.ReadAllText(arguments[1]);
            var result = Run(source, PracticalIdentity.SourceTypeId("Business", "Containers"));
            string projection = Encoding.UTF8.GetString(CSharpPracticalStructural.CopyTypeProjection(result));
            Check(projection == File.ReadAllText(arguments[2]).TrimEnd('\n'), "SOURCE_ROUTING");
            Check(result.Calls.Any(call=>call.Target == PracticalIdentity.CallableId("method","Business",
                PracticalIdentity.SourceTypeId("Business","Product"),"Same",new[]{PracticalIdentity.SourceTypeId("Business","Product")},PracticalIdentity.PrimitiveId("bool"))), "HELPER_CAPTURE");
            Check(result.Functions.Any(function=>function.Body.CopyBodyBytes().Length>0), "HELPER_BODY");
            Check(result.ArtifactCount == 0, "PRIVATE_ROUTE");
            Check(result.SourceEqualities.Count >= 3 && result.SourceEqualities.All(plan=>plan.Operation=="structural_equal"), "SOURCE_LOWERING");
            Check(result.SourceEqualities.Any(plan=>plan.OperandType.Id==PracticalIdentity.SourceTypeId("Business","Product")
                && plan.OperandType.Nullability=="annotated"), "NULL_COMPARISON_TYPE");
            SourceEquality(); stage="runtime_corners"; RuntimeCorners();
            return 0;
        } catch(Exception error) {
            Console.Error.WriteLine("STRUCTURAL_"+stage+"_"+(error is PracticalCaptureFailure f ? f.Family+"_"+f.Code : error.ToString()));
            return 1;
        }
    }
    private static PracticalConstruction Run(string source, string parameter) {
        if (!source.EndsWith("\n", StringComparison.Ordinal)) { source += "\n"; }
        string root = PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{parameter},PracticalIdentity.PrimitiveId("i32"));
        return CSharpPracticalStructural.Validate(new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"business",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>()),
            new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source))},references);
    }
    private static void SourceEquality() {
        const string start="namespace Business; public static class Entry { public static int Run(int input) { ";
        foreach(string expression in new[]{"input == 1", "input != 1", "\"x\" == (input == 1 ? \"x\" : \"y\")", "\"x\" != null", "(float)input == 1f", "(decimal)input == 1m", "string.Equals(\"x\", input == 1 ? \"x\" : \"y\")",
            "string.Equals(\"x\", \"y\", System.StringComparison.Ordinal)",
            "\"x\".Equals(\"y\", System.StringComparison.Ordinal)"})
            {
                stage=expression;
                var result=Run(start+"return ("+expression+") ? 1 : 0; } }", PracticalIdentity.PrimitiveId("i32"));
                Check(result.SourceEqualities.Count>0,"OPERATOR_LOWERING");
                Check(result.SourceEqualities.Any(plan=>plan.Negated)==expression.Contains("!=",StringComparison.Ordinal),"NEGATION");
                Check(result.SourceEqualities.Any(plan=>plan.NullCheckAfterArguments)==expression.StartsWith("\"x\".Equals",StringComparison.Ordinal),"INSTANCE_NULL_EDGE");
                foreach(var plan in result.SourceEqualities) { Check(plan.Left is not null && plan.Right is not null,"ORDERED_OPERANDS"); }
            }
        foreach(string body in new[]{"var a=new Data(); var b=new Data(); return a==b ? 1:0;", "return new Data().GetHashCode();", "return object.ReferenceEquals(new Data(),new Data()) ? 1:0;", "return new Data().Equals(new Data()) ? 1:0;", "return string.Equals(\"x\",\"X\",System.StringComparison.OrdinalIgnoreCase) ? 1:0;"}) {
            bool rejected=false;
            try { Run("namespace Business; public sealed class Data {} public static class Entry {public static int Run(int input){"+body+"}}", PracticalIdentity.PrimitiveId("i32")); }
            catch(PracticalCaptureFailure){rejected=true;}
            Check(rejected,"IDENTITY_ESCAPE");
        }
    }
    private static void RuntimeCorners() {
        foreach(string separator in new[]{".",",","\u066b"}) {
            var culture=(CultureInfo)CultureInfo.InvariantCulture.Clone();
            culture.NumberFormat.NumberDecimalSeparator=separator;
            culture.NumberFormat.NegativeSign="!";
            CultureInfo.CurrentCulture=culture;
            Check(1.0m==1.00m && new decimal(0,0,0,true,28)==0m,"DECIMAL_VALUE");
            Check(decimal.Compare(decimal.MaxValue,decimal.MaxValue/10m)>0,"DECIMAL_EXTREME");
            float nan=BitConverter.Int32BitsToSingle(unchecked((int)0x7fc00001));
            float other=nan;
            Check(!(nan==other) && 0f==BitConverter.Int32BitsToSingle(unchecked((int)0x80000000)),"IEEE_EQUAL");
            Check(string.CompareOrdinal("\ud800","\ue000")<0 && string.CompareOrdinal(null,"")<0,"UTF16_NULL");
            var low=Guid.ParseExact("00000001000000000000000000000000","N");
            var high=Guid.ParseExact("00000100000000000000000000000000","N");
            Check(low.CompareTo(high)<0,"GUID_UNSIGNED_FIELDS");
        }
    }
    private static void Check(bool value,string code){if(!value)throw new Exception(code);}
}
