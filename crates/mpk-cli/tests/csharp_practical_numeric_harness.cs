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
internal static class PracticalNumericHarness
{
    private static ImmutableArray<MetadataReference> references;
    private static string stage="start";
    public static int Main(string[] args)
    {
        try {
            references=Directory.EnumerateFiles(Path.Combine(args[0],"ref","net10.0"),"*.dll").OrderBy(p=>p,StringComparer.Ordinal).Select(p=>MetadataReference.CreateFromFile(p)).ToImmutableArray<MetadataReference>();
            foreach(string type in new[]{"float","double","decimal"}) {
                string prefix=type=="decimal"?"decimal.":"floating."+(type=="float"?"single.":"double.");
                foreach(var pair in new[]{("a+b","add"),("a-b","subtract"),("a*b","multiply"),("a/b","divide"),("a%b","remainder"),("+a","plus"),("-a","negate")}) {CheckSource(type,"return "+pair.Item1+";",prefix+pair.Item2);}
                foreach(var pair in new[]{("a==b","equal"),("a!=b","not_equal"),("a<b","less"),("a<=b","less_equal"),("a>b","greater"),("a>=b","greater_equal")}) {CheckSource(type,"return "+pair.Item1+";",prefix+pair.Item2,"bool");}
                if(type!="decimal") {
                    string math=type=="float"?"System.MathF":"System.Math";
                    foreach(string name in new[]{"Abs","Min","Max"}){CheckSource(type,"return "+math+"."+name+"(a"+(name=="Abs"?"":",b")+");",prefix+name.ToLowerInvariant());}
                    foreach(var pair in new[]{("IsNaN","is_nan"),("IsInfinity","is_infinity"),("IsFinite","is_finite")}){CheckSource(type,"return "+type+"."+pair.Item1+"(a);",prefix+pair.Item2,"bool");}
                } else {
                    foreach(string name in new[]{"Truncate","Floor","Ceiling"}){CheckSource(type,"return decimal."+name+"(a);",prefix+name.ToLowerInvariant());}
                    foreach(string mode in new[]{"ToEven","AwayFromZero","ToZero","ToNegativeInfinity","ToPositiveInfinity"}){var step=CheckSource(type,"return decimal.Round(a,n,System.MidpointRounding."+mode+");","decimal.round").Steps.Single(s=>s.Operation=="decimal.round");Check(step.Rounding==mode&&step.Exceptions.SequenceEqual(new[]{"System.ArgumentOutOfRangeException"}),"ROUND_CONFIG");}
                    foreach(string call in new[]{"decimal.Round(a)","decimal.Round(a,n)","decimal.Round(a,System.MidpointRounding.ToEven)"}){var step=CheckSource(type,"return "+call+";","decimal.round").Steps.Single(s=>s.Operation=="decimal.round");Check(step.Exceptions.Count==(call.Contains(",n",StringComparison.Ordinal)?1:0),"ROUND_OVERLOAD_EXCEPTIONS");}
                }
            }
            CheckSource("float","return (float)n;","numeric.conversion.int32_to_single");
            CheckSource("double","return (double)l;","numeric.conversion.int64_to_double");
            CheckSource("float","return (double)a;","numeric.conversion.single_to_double","double");
            CheckSource("double","return (float)a;","numeric.conversion.double_to_single","float");
            CheckSource("float","return checked((int)a);","numeric.conversion.single_to_int32.checked","int");
            CheckSource("double","return checked((long)a);","numeric.conversion.double_to_int64.checked","long");
            CheckSource("decimal","return (decimal)l;","decimal.conversion.int64_to_decimal");
            CheckSource("decimal","return (int)a;","decimal.conversion.decimal_to_int32","int");
            var divide=CheckSource("decimal","return a/b;","decimal.divide").Steps.Single(s=>s.Operation=="decimal.divide");Check(divide.Exceptions.SequenceEqual(new[]{"System.DivideByZeroException","System.OverflowException"}),"DECIMAL_EXCEPTIONS");
            CheckSource("float","return 1.25f;","floating.single.literal");CheckSource("double","return 1.25d;","floating.double.literal");CheckSource("decimal","return 1.25m;","decimal.literal");
            foreach(var test in new[]{("float","return unchecked((int)a);","int"),("double","return checked((int)a);","int"),("float","return (decimal)a;","decimal"),("decimal","return (double)a;","double"),("double","return System.Math.Sin(a);","double"),("float","return System.MathF.Sqrt(a);","float"),("float","return float.Parse(\"1\");","float"),("decimal","return decimal.GetBits(a)[0];","int"),("decimal","var mode=System.MidpointRounding.ToEven;return decimal.Round(a,mode);","decimal"),("float","a+=b;return a;","float"),("double","return ++a;","double")}) {stage=test.Item2;Reject(()=>Run(test.Item1,test.Item2,test.Item3));}
            using var fixture=JsonDocument.Parse(File.ReadAllBytes("numeric-runtime.json"));
            foreach(string separator in new[]{",","\u066b","!"}) {var culture=(CultureInfo)CultureInfo.InvariantCulture.Clone();culture.NumberFormat.NumberDecimalSeparator=separator;culture.NumberFormat.NegativeSign="\u2212";CultureInfo.CurrentCulture=culture;CultureInfo.CurrentUICulture=culture;foreach(var row in fixture.RootElement.EnumerateArray()){Runtime(row);}}
            return 0;
        }catch(Exception error){Console.Error.WriteLine("NUMERIC_"+stage+"_"+(error is PracticalCaptureFailure f?f.Family+"_"+f.Code:error.ToString()));return 1;}
    }
    private static PracticalNumeric CheckSource(string type,string body,string op,string? result=null)
    {stage=type+":"+body;var actual=Run(type,body,result??type);Check(actual.ArtifactCount==0&&actual.Steps.Any(s=>s.Operation==op),"SOURCE_OPERATION");byte[] bytes=actual.CopyCanonicalBytes();CSharpPracticalNumeric.ValidateCandidate(actual,Run(type,body,result??type).CopyCanonicalBytes());bytes[bytes.Length/2]^=1;Reject(()=>CSharpPracticalNumeric.ValidateCandidate(actual,bytes));return actual;}
    private static PracticalNumeric Run(string type,string body,string result)
    {
        string Token(string s)=>s switch {"float"=>"f32","double"=>"f64","int"=>"i32","long"=>"i64",_=>s};
        string source="namespace Business;public static class Entry{public static "+result+" Run("+type+" a,"+type+" b,int n,long l){"+body+"}}\n";
        string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{Token(type),Token(type),"i32","i64"}.Select(PracticalIdentity.PrimitiveId).ToArray(),PracticalIdentity.PrimitiveId(Token(result)));
        return CSharpPracticalNumeric.Validate(new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"business",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>()),new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source))},references);
    }
    private static decimal Dec(string text) {var p=text.Split(';').Select(s=>s.Split('=')[1]).ToArray();string h=p[2];return new decimal(unchecked((int)uint.Parse(h[16..24],NumberStyles.HexNumber,CultureInfo.InvariantCulture)),unchecked((int)uint.Parse(h[8..16],NumberStyles.HexNumber,CultureInfo.InvariantCulture)),unchecked((int)uint.Parse(h[..8],NumberStyles.HexNumber,CultureInfo.InvariantCulture)),p[0]=="1",byte.Parse(p[1],CultureInfo.InvariantCulture));}
    private static string F(float n)=>BitConverter.SingleToUInt32Bits(n).ToString("x8",CultureInfo.InvariantCulture);
    private static string D(double n)=>BitConverter.DoubleToUInt64Bits(n).ToString("x16",CultureInfo.InvariantCulture);
    private static string B(bool b)=>b?"true":"false";
    private static void Runtime(JsonElement row)
    {
        stage=row.GetProperty("id").GetString()!;string op=row.GetProperty("operation").GetString()!;
        var fields=row.GetProperty("inputs").EnumerateArray().Select(p=>p.GetString()!.Split('=',2)).ToDictionary(p=>p[0],p=>p[1],StringComparer.Ordinal);
        var values=fields.Where(p=>p.Key is not "rounding" and not "digits").Select(p=>p.Value).ToArray();
        var expected=row.GetProperty("profile");string? error=null;string result="";decimal? decResult=null;
        try {
            if(op.StartsWith("floating.single.",StringComparison.Ordinal)) {float a=BitConverter.UInt32BitsToSingle(uint.Parse(values[0],NumberStyles.HexNumber,CultureInfo.InvariantCulture));float b=values.Length>1?BitConverter.UInt32BitsToSingle(uint.Parse(values[1],NumberStyles.HexNumber,CultureInfo.InvariantCulture)):0;result=op[16..] switch {"plus"=>F(+a),"negate"=>F(-a),"abs"=>F(MathF.Abs(a)),"add"=>F(a+b),"subtract"=>F(a-b),"multiply"=>F(a*b),"divide"=>F(a/b),"remainder"=>F(a%b),"min"=>F(MathF.Min(a,b)),"max"=>F(MathF.Max(a,b)),"is_nan"=>B(float.IsNaN(a)),"is_infinity"=>B(float.IsInfinity(a)),"is_finite"=>B(float.IsFinite(a)),"equal"=>B(a==b),"not_equal"=>B(a!=b),"less"=>B(a<b),"less_equal"=>B(a<=b),"greater"=>B(a>b),"greater_equal"=>B(a>=b),_=>throw new Exception("FLOAT_OP")};}
            else if(op.StartsWith("floating.double.",StringComparison.Ordinal)) {double a=BitConverter.UInt64BitsToDouble(ulong.Parse(values[0],NumberStyles.HexNumber,CultureInfo.InvariantCulture));double b=values.Length>1?BitConverter.UInt64BitsToDouble(ulong.Parse(values[1],NumberStyles.HexNumber,CultureInfo.InvariantCulture)):0;result=op[16..] switch {"plus"=>D(+a),"negate"=>D(-a),"abs"=>D(Math.Abs(a)),"add"=>D(a+b),"subtract"=>D(a-b),"multiply"=>D(a*b),"divide"=>D(a/b),"remainder"=>D(a%b),"min"=>D(Math.Min(a,b)),"max"=>D(Math.Max(a,b)),"is_nan"=>B(double.IsNaN(a)),"is_infinity"=>B(double.IsInfinity(a)),"is_finite"=>B(double.IsFinite(a)),"equal"=>B(a==b),"not_equal"=>B(a!=b),"less"=>B(a<b),"less_equal"=>B(a<=b),"greater"=>B(a>b),"greater_equal"=>B(a>=b),_=>throw new Exception("DOUBLE_OP")};}
            else if(op.StartsWith("numeric.conversion.",StringComparison.Ordinal)) {result=op[19..] switch {"int32_to_single"=>F((float)int.Parse(values[0],CultureInfo.InvariantCulture)),"int64_to_double"=>D((double)long.Parse(values[0],CultureInfo.InvariantCulture)),"single_to_double"=>D((double)BitConverter.UInt32BitsToSingle(uint.Parse(values[0],NumberStyles.HexNumber,CultureInfo.InvariantCulture))),"double_to_single"=>F((float)BitConverter.UInt64BitsToDouble(ulong.Parse(values[0],NumberStyles.HexNumber,CultureInfo.InvariantCulture))),"single_to_int32.checked"=>checked((int)BitConverter.UInt32BitsToSingle(uint.Parse(values[0],NumberStyles.HexNumber,CultureInfo.InvariantCulture))).ToString(CultureInfo.InvariantCulture),"double_to_int64.checked"=>checked((long)BitConverter.UInt64BitsToDouble(ulong.Parse(values[0],NumberStyles.HexNumber,CultureInfo.InvariantCulture))).ToString(CultureInfo.InvariantCulture),_=>throw new Exception("CONVERSION_OP")};}
            else if(op.StartsWith("decimal.conversion.",StringComparison.Ordinal)) {if(op.EndsWith("decimal_to_int32",StringComparison.Ordinal)){result=((int)Dec(values[0])).ToString(CultureInfo.InvariantCulture);}else{decResult=op.EndsWith("uint64_to_decimal",StringComparison.Ordinal)?(decimal)ulong.Parse(values[0],CultureInfo.InvariantCulture):(decimal)long.Parse(values[0],CultureInfo.InvariantCulture);}}
            else {
                decimal a,b;if(fields.TryGetValue("case",out string? edge)){a=edge is "min_minus_one" or "negate_min"?decimal.MinValue:decimal.MaxValue;b=edge switch {"max_divide_fraction"=>0.1m,"max_times_two"=>2m,"max_divide_zero" or "max_remainder_zero"=>0m,_=>1m};}else{a=Dec(values[0]);b=values.Length>1?Dec(values[1]):0;}
                string name=op[8..];if(name is "equal" or "value_equality" or "not_equal" or "less" or "less_equal" or "greater" or "greater_equal"){result=name switch {"equal" or "value_equality"=>B(a==b),"not_equal"=>B(a!=b),"less"=>B(a<b),"less_equal"=>B(a<=b),"greater"=>B(a>b),_=>B(a>=b)};}else{decResult=name switch {"plus"=>+a,"negate"=>-a,"add"=>a+b,"subtract"=>a-b,"multiply"=>a*b,"divide"=>a/b,"remainder"=>a%b,"truncate"=>decimal.Truncate(a),"floor"=>decimal.Floor(a),"ceiling"=>decimal.Ceiling(a),"round"=>decimal.Round(a,int.Parse(fields["digits"],CultureInfo.InvariantCulture),Enum.Parse<MidpointRounding>(fields["rounding"])),_=>throw new Exception("DECIMAL_OP")};}
            }
        }catch(OverflowException){error="exception.overflow";}catch(DivideByZeroException){error="exception.division_by_zero";}catch(ArgumentOutOfRangeException){error="exception.range";}
        Check(error==expected.GetProperty("error_id").GetString(),"ERROR_DIFFERENTIAL");
        if(error is null){Check(decResult.HasValue?decResult.Value==Dec(expected.GetProperty("value").GetString()!):result==expected.GetProperty("value").GetString(),"VALUE_DIFFERENTIAL");}
    }
    private static void Reject(Action action){bool rejected=false;try{action();}catch(PracticalCaptureFailure f){if(f.Family==PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL)throw;rejected=true;}Check(rejected,"EXPECTED_REJECTION");}
    private static void Check(bool condition,string message){if(!condition)throw new Exception(message);}
}
