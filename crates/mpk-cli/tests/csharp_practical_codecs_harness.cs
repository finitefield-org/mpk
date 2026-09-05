using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using Microsoft.CodeAnalysis;
namespace Mpk.CSharp2Vir;
internal static class PracticalCodecsHarness
{
    private static ImmutableArray<MetadataReference> references;
    private static string stage="start";
    public static int Main(string[] args)
    {
        try {
            references=Directory.EnumerateFiles(Path.Combine(args[0],"ref","net10.0"),"*.dll").OrderBy(p=>p,StringComparer.Ordinal).Select(p=>MetadataReference.CreateFromFile(p)).ToImmutableArray<MetadataReference>();
            stage="source_matrix";
            foreach(var expression in new[]{"input.Length","input[0] == unit ? 1 : 0","input.Substring(0,1).Length","string.Compare(input,\"a\",System.StringComparison.Ordinal)","string.Equals(input,\"a\",System.StringComparison.Ordinal)?1:0","input.Equals(\"a\",System.StringComparison.Ordinal)?1:0","input.Contains(\"a\",System.StringComparison.Ordinal)?1:0","input.StartsWith(\"a\",System.StringComparison.Ordinal)?1:0","input.EndsWith(\"a\",System.StringComparison.Ordinal)?1:0","string.IsNullOrEmpty(input)?1:0","(input==\"a\")?1:0","(input!=\"a\")?1:0","(input+unit).Length","(unit+input).Length","(input+input).Length","string.Concat(input,input,input,input).Length","$\"[{input}{unit}]\".Length"}) {
                stage=expression;var result=Run("return "+expression+";");Check(result.ArtifactCount==0&&result.Steps.Count>0,"SOURCE_STEPS");Check(result.Obligations.Any(o=>o.Kind=="input_utf16_length_le_16384")&&result.Obligations.All(o=>!o.Discharged),"PENDING_PROFILE_VCS");
                string expected=expression.StartsWith("input[",StringComparison.Ordinal)?"string.index"
                    :expression.Contains("Substring",StringComparison.Ordinal)?"string.substring.start_length"
                    :expression.Contains("Compare",StringComparison.Ordinal)?"string.compare.ordinal"
                    :expression.Contains("Equals",StringComparison.Ordinal)?"string.equals.ordinal"
                    :expression.Contains("Contains",StringComparison.Ordinal)?"string.contains.ordinal"
                    :expression.Contains("StartsWith",StringComparison.Ordinal)?"string.starts_with.ordinal"
                    :expression.Contains("EndsWith",StringComparison.Ordinal)?"string.ends_with.ordinal"
                    :expression.Contains("IsNullOrEmpty",StringComparison.Ordinal)?"string.is_null_or_empty"
                    :expression.Contains("==",StringComparison.Ordinal)?"string.equality.operator"
                    :expression.Contains("!=",StringComparison.Ordinal)?"string.inequality.operator"
                    :expression.Contains("input+unit",StringComparison.Ordinal)?"string.concat.operator.string_char"
                    :expression.Contains("unit+input",StringComparison.Ordinal)?"string.concat.operator.char_string"
                    :expression.Contains("input+input",StringComparison.Ordinal)?"string.concat.operator.string_string"
                    :expression.Contains("Concat",StringComparison.Ordinal)?"string.concat.string4"
                    :expression.StartsWith("$",StringComparison.Ordinal)?"string.interpolation.restricted":"string.length";
                Check(result.Steps.Any(step=>step.Operation==expected),"EXACT_SOURCE_OPERATION");
                byte[] bytes=result.CopyCanonicalBytes();CSharpPracticalStrings.ValidateCandidate(result,bytes);bytes[bytes.Length/2]^=1;Reject(()=>CSharpPracticalStrings.ValidateCandidate(result,bytes));
            }
            stage="source_checks";
            var substring=Run("return input.Substring(0,1).Length;");var call=substring.Steps.Single(s=>s.Operation=="string.substring.start_length");
            Check(call.Checks.SequenceEqual(new[]{"exception.null_receiver","exception.range","obligation.output_bound"}),"SUBSTRING_ORDER");
            Check(call.Exceptions.SequenceEqual(new[]{"System.NullReferenceException","System.ArgumentOutOfRangeException"}),"SUBSTRING_EXCEPTIONS");
            Check(Run("return input[0];").Steps.Single(s=>s.Operation=="string.index").Exceptions.Last()=="System.IndexOutOfRangeException","INDEX_EXCEPTION");
            Reject(()=>Run("return input.Substring(length:1,startIndex:0).Length;"));
            var interpolation=Run("return $\"{input}{unit}\".Length;").Steps.Single(s=>s.Operation=="string.interpolation.restricted");
            var concat=Run("return (input+unit).Length;").Steps.Single(s=>s.Operation=="string.concat.operator.string_char");
            Check(interpolation.Relation==concat.Relation&&interpolation.Checks.SequenceEqual(concat.Checks),"CONCAT_NORMALIZATION");
            var deterministic=Run("return (input+unit).Length;");CSharpPracticalStrings.ValidateCandidate(deterministic,Run("return (input+unit).Length;").CopyCanonicalBytes());
            foreach(int n in new[]{16383,16384}){Run("return \""+new string('a',n)+"\".Length;");}Reject(()=>Run("return \""+new string('a',16385)+"\".Length;"));
            stage="rejections";
            foreach(string body in new[]{"return $\"{input,10}\".Length;","return $\"{input:x}\".Length;","return $\"{1}\".Length;","return unit+unit;","return string.Concat((object)input,input).Length;","return input.ToString().Length;","return int.Parse(input);","return input.ToUpper().Length;","return input.StartsWith(\"a\")?1:0;","return input.Contains(\"a\",System.StringComparison.OrdinalIgnoreCase)?1:0;","var option=System.StringComparison.Ordinal;return string.Compare(input,input,option);","return string.Format(\"{0}\",input).Length;","int total=0;foreach(char c in input){total++;}return total;"}){Reject(()=>Run(body));}
            stage="runtime_cultures";
            foreach(string separator in new[]{",","\u066b","!"}) {
                var culture=(CultureInfo)CultureInfo.InvariantCulture.Clone();culture.NumberFormat.NumberDecimalSeparator=separator;culture.NumberFormat.NegativeSign="\u2212";CultureInfo.CurrentCulture=culture;CultureInfo.CurrentUICulture=culture;
                using var fixture=JsonDocument.Parse(File.ReadAllBytes("source-strings.json"));
                foreach(var row in fixture.RootElement.EnumerateArray()){Runtime(row);}
            }
            return 0;
        }catch(Exception error){Console.Error.WriteLine("CODECS_"+stage+"_"+(error is PracticalCaptureFailure f?f.Family+"_"+f.Code:error.ToString()));return 1;}
    }
    private static PracticalStrings Run(string body)
    {
        string source="namespace Business;public static class Entry{public static int Run(string input,char unit){"+body+"}}\n";
        string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{PracticalIdentity.PrimitiveId("string"),PracticalIdentity.PrimitiveId("char")},PracticalIdentity.PrimitiveId("i32"));
        return CSharpPracticalStrings.Validate(new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"business",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>()),new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source))},references);
    }
    private static string? Decode(string s)=>s=="null"?null:new string(Enumerable.Range(0,s.Length/4).Select(i=>(char)int.Parse(s.Substring(i*4,4),NumberStyles.HexNumber,CultureInfo.InvariantCulture)).ToArray());
    private static string Hex(string s)=>string.Concat(s.Select(c=>((int)c).ToString("x4",CultureInfo.InvariantCulture)));
    private static void Runtime(JsonElement row)
    {
        stage=row.GetProperty("id").GetString()!;string op=row.GetProperty("operation").GetString()!;
        var inputs=row.GetProperty("inputs").EnumerateArray().Select(v=>v.GetString()!.Split('=',2)).ToArray();
        string? Text(int i)=>Decode(inputs[i][1]);int Number(int i)=>int.Parse(inputs[i][1],CultureInfo.InvariantCulture);char Character(int i)=>(char)int.Parse(inputs[i][1],NumberStyles.HexNumber,CultureInfo.InvariantCulture);
        string? exception=null;string result="";
        try {result=op switch {
            "string.length"=>Text(0)!.Length.ToString(CultureInfo.InvariantCulture),"string.index"=>Hex(new string(Text(0)![Number(1)],1)),
            "string.substring.start_length"=>Hex(Text(0)!.Substring(Number(1),Number(2))),
            "string.compare.ordinal"=>Math.Sign(string.Compare(Text(0),Text(1),StringComparison.Ordinal)).ToString(CultureInfo.InvariantCulture),
            "string.equals.ordinal"=>string.Equals(Text(0),Text(1),StringComparison.Ordinal).ToString().ToLowerInvariant(),
            "string.equality.operator"=>(Text(0)==Text(1)).ToString().ToLowerInvariant(),"string.inequality.operator"=>(Text(0)!=Text(1)).ToString().ToLowerInvariant(),
            "string.contains.ordinal"=>Text(0)!.Contains(Text(1)!,StringComparison.Ordinal).ToString().ToLowerInvariant(),
            "string.starts_with.ordinal"=>Text(0)!.StartsWith(Text(1)!,StringComparison.Ordinal).ToString().ToLowerInvariant(),
            "string.ends_with.ordinal"=>Text(0)!.EndsWith(Text(1)!,StringComparison.Ordinal).ToString().ToLowerInvariant(),
            "string.is_null_or_empty"=>string.IsNullOrEmpty(Text(0)).ToString().ToLowerInvariant(),
            "string.concat.operator.string_string"=>Hex(Text(0)+Text(1)),"string.concat.operator.string_char"=>Hex(Text(0)+Character(1)),"string.concat.operator.char_string"=>Hex(Character(0)+Text(1)),
            "string.concat.string2"=>Hex(string.Concat(Text(0),Text(1))),"string.concat.string3"=>Hex(string.Concat(Text(0),Text(1),Text(2))),"string.concat.string4"=>Hex(string.Concat(Text(0),Text(1),Text(2),Text(3))),
            "string.interpolation.restricted"=>Hex($"{Text(0)}{Text(1)}{Character(2)}{Text(3)}"),
            "string.literal.decode"=>inputs[0][1]=="empty"?"":inputs[0][1].Replace(",","",StringComparison.Ordinal),
            _=>throw new Exception("UNKNOWN_STRING_OPERATION"),
        };}catch(NullReferenceException){exception="exception.null_receiver";}catch(ArgumentNullException){exception="exception.null_argument";}catch(ArgumentOutOfRangeException){exception="exception.range";}catch(IndexOutOfRangeException){exception="exception.range";}
        var expected=row.GetProperty("profile");Check(exception==expected.GetProperty("error_id").GetString()&&result==expected.GetProperty("value").GetString(),"RUNTIME_DIFFERENTIAL");
    }
    private static void Reject(Action action){bool rejected=false;try{action();}catch(PracticalCaptureFailure f){if(f.Family==PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL)throw;rejected=true;}Check(rejected,"EXPECTED_REJECTION");}
    private static void Check(bool condition,string message){if(!condition)throw new Exception(message);}
}
