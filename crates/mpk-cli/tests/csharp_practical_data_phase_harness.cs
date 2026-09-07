using System;
using System.Collections.Immutable;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using Microsoft.CodeAnalysis;
namespace Mpk.CSharp2Vir;
internal static class PracticalDataPhaseHarness
{
    private static ImmutableArray<MetadataReference> references;
    private const string Source = "namespace Data;public readonly struct Value{public readonly int Amount;public Value(int amount){Amount=amount;}}public static class Entry{public static int Run(Value value){return new Value(value.Amount).Amount;}}\n";
    private static string Root => PracticalIdentity.CallableId("method", "Data", PracticalIdentity.SourceTypeId("Data", "Entry"), "Run", new[]{PracticalIdentity.SourceTypeId("Data", "Value")}, PracticalIdentity.PrimitiveId("i32"));
    private static PracticalDataSource Run(string source, string path = "src/Entry.cs") => CSharpPracticalDataPhase.Capture(
        new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema, "data", new[]{path}, new[]{Root}, Array.Empty<string>()),
        new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source, path, Encoding.UTF8.GetBytes(source))}, references);
    public static int Main(string[] args)
    {
        try {
            references=Directory.EnumerateFiles(Path.Combine(args[0],"ref","net10.0"),"*.dll").OrderBy(s=>s,StringComparer.Ordinal).Select(p=>(MetadataReference)MetadataReference.CreateFromFile(p)).ToImmutableArray();
            if(File.Exists("data-requests.json")) {RunRequests();return 0;}
            var first=Run(Source); CSharpPracticalDataPhase.ValidateCandidate(first,Run(Source).CopyBytes());
            using var original=JsonDocument.Parse(first.CopyBytes());
            using var relocated=JsonDocument.Parse(Run(Source,"src/Moved.cs").CopyBytes());
            var type=original.RootElement.GetProperty("types")[0];
            var defaults=type.GetProperty("recursive_default");
            var nodes=defaults.GetProperty("nodes");
            if(nodes.GetArrayLength()!=2 || defaults.GetProperty("root").GetInt32()!=1
                || nodes[0].GetProperty("scalar").GetString()!="0"
                || nodes[1].GetProperty("members")[0].GetInt32()!=0)
                throw new Exception("recursive_default_graph");
            if(type.GetProperty("id").GetString()!=relocated.RootElement.GetProperty("types")[0].GetProperty("id").GetString()) throw new Exception("logical_identity");
            if(first.CopyBytes().SequenceEqual(Run(Source,"src/Moved.cs").CopyBytes())) throw new Exception("provenance");
            Reject(()=>CSharpPracticalDataPhase.ValidateCandidate(first,Run(Source.Replace("Amount=amount;","Amount=amount+1;")).CopyBytes()));
            foreach(string body in new[]{"for(int i=0;i<1;i++){}return new Value(value.Amount).Amount;","while(false){}return new Value(value.Amount).Amount;","try{return new Value(value.Amount).Amount;}catch{return 0;}","throw new System.InvalidOperationException();"})
                Reject(()=>Run(Source.Replace("return new Value(value.Amount).Amount;",body)));
            using var surrogate=JsonDocument.Parse(Run(Source.Replace("new Value(value.Amount)","new Value(value.Amount + \"\\uD800\".Length)")).CopyBytes());
            if(!surrogate.RootElement.GetProperty("callables").EnumerateArray().Any(c=>c.GetProperty("body_utf8").GetString()!.Contains("string_utf16:d800",StringComparison.Ordinal)))
                throw new Exception("lossless_string_constant");
            var changed=first.CopyBytes();changed[changed.Length/2]^=1;Reject(()=>CSharpPracticalDataPhase.ValidateCandidate(first,changed));
            File.WriteAllBytes("data-source.json",first.CopyBytes());
            var cases=new System.Collections.Generic.List<object>();
            foreach(var item in new[]{
                ("identity","return new Value(value.Amount).Amount;"),
                ("nested_initializer_same_owner","return new Value{Amount=new Value{Amount=value.Amount}.Amount}.Amount;"),
                ("nullable_source_call","return new Value(Echo(\"x\")==null?0:value.Amount).Amount;"),
                ("nullable_initializer_member","return new Value{Amount=value.Amount,Text=\"ok\"}.Text.Length;"),
                ("enum_equality","Color color=value.Amount>0?Color.Red:Color.Blue;return new Value(color==Color.Red?1:0).Amount;"),
                ("reference_null","return new Value(value==null?0:value.Amount).Amount;"),
                ("reference_nullable_conditional","string? text=value?.Text;return new Value(text==null?0:text.Length).Amount;"),
                ("reference_nullable_field","if(value==null){return 0;}return new Value(value.Amount).Amount;"),
                ("reference_nullable_guard","return new Value(value==null?0:value.Amount).Amount;"),
                ("instant_carrier","return (int)new Value(value.Amount).Amount;"),
                ("literal","return new Value(42).Amount;"),
                ("property_constructor","return new Value(value.Amount).Amount;"),
                ("delegated_constructor","return new Value(value.Amount).Amount;"),
                ("constructor_early_return","return new Value(value.Amount).Amount;"),
                ("money_carrier","return new Value(value.Amount,value.Currency).Currency.Length;"),
                ("string_utf16","return new Value(\"\\uD800\".Length).Amount;"),
                ("string_contains","return new Value(\"abc\".Contains(\"b\",System.StringComparison.Ordinal)?value.Amount:0).Amount;"),
                ("string_substring","return new Value(\"abc\".Substring(0,value.Amount).Length).Amount;"),
                ("string_index","return new Value((int)\"abc\"[value.Amount]).Amount;"),
                ("decimal_round_modes","decimal x=value.Amount;decimal a=decimal.Round(x,1,System.MidpointRounding.ToEven);decimal b=decimal.Round(x,System.MidpointRounding.AwayFromZero);return new Value((int)(a+b)).Amount;"),
                ("string_instance_equals","string text=\"a\";return new Value(text.Equals(\"b\",System.StringComparison.Ordinal)?value.Amount:0).Amount;"),
                ("decimal_arithmetic","decimal x=value.Amount;return new Value((int)(x+1.5m)).Amount;"),
                ("float_intrinsic","double x=(long)value.Amount;return new Value(double.IsFinite(x)?value.Amount:0).Amount;"),
                ("date_construct","return new Value(new System.DateOnly(2024,2,value.Amount).Day).Amount;"),
                ("string_concat","string text=\"a\";return new Value((text+\"b\").Length).Amount;"),
                ("default","return new Value(default(Value).Amount).Amount;"),
                ("array_dynamic_write","int[] values=new int[value.Amount];values[0]=value.Amount;return new Value(values[0]).Amount;"),
                ("unused_assigned_getter","return new Value(value.Amount).Amount;"),
                ("array_dynamic_reinitialize","string[] values=new string[value.Amount];values[0]=\"a\";values[0]=\"b\";return new Value(values[0].Length).Amount;"),
                ("array_symbolic_initialize","string[] values=new string[2];values[value.Amount]=\"a\";values[1]=\"b\";return new Value(values[1].Length).Amount;"),
                ("array_boolean_update","bool[] values=new bool[1];values[0]|=value.Amount>0;return new Value(values[0]?1:0).Amount;"),
                ("reference_nullable_call","if(value!=null){return new Value(value.Plus(12/value.Amount)).Amount;}return 0;"),
                ("array_empty_nondefault","string[] values=new string[0];return new Value(values.Length).Amount;"),
                ("array_dynamic_initialize","string[] values=new string[value.Amount];values[0]=\"a\";return new Value(CountStrings(values)).Amount;"),
                ("array_partial_initialize","string[] values=new string[2];values[0]=\"a\";values[1]=\"b\";values[0]=\"c\";return new Value(values[0].Length).Amount;"),
                ("array_alias_read","int[] values=new int[value.Amount];int[] alias=values;return new Value(alias.Length+values[0]).Amount;"),
                ("array_store_order","int[] values=new int[value.Amount];values[0]=12/value.Amount;return new Value(values[0]).Amount;"),
                ("array_update_order","int[] values=new int[value.Amount];values[0]+=12/value.Amount;return new Value(values[0]).Amount;"),
                ("array_small_update","byte[] values=new byte[1];byte before=values[0]++;values[0]+=2;return new Value(before+values[0]).Amount;"),
                ("array_update","int[] values=new int[value.Amount];int before=values[0]++;values[0]+=value.Amount;int after=++values[0];return new Value(before+after+values[0]).Amount;"),
                ("array_branch_write","int[] values=new int[value.Amount];if(value.Amount>2){values[0]=1;}else{values[0]=2;}return new Value(values[0]).Amount;"),
                ("array_branch_read","int[] values=new int[value.Amount];if(value.Amount>2){return new Value(values[0]).Amount;}return new Value(values.Length).Amount;"),
                ("array_constant_write","int[] values=new int[]{1,2};values[0]=value.Amount;return new Value(values[0]).Amount;"),
                ("array_dynamic_read","int[] values=new int[value.Amount];return new Value(values[0]).Amount;"),
                ("array_dynamic_length","int[] values=new int[value.Amount];return new Value(values.Length).Amount;"),
                ("array_dynamic_call","int[] values=new int[value.Amount];return new Value(Count(values)).Amount;"),
                ("array_initializer","int[] values=new int[]{value.Amount,2};return new Value(values[0]).Amount;"),
                ("array_default","int[] values=new int[2];return new Value(values[0]).Amount;"),
                ("array_length","int[] values=new int[]{value.Amount,2};return new Value(values.Length).Amount;"),
                ("division","return new Value(12/value.Amount).Amount;"),
                ("nullable","int? amount=value.Amount;return new Value(amount.Value).Amount;"),
                ("nullable_none","int? amount=null;return new Value(amount.HasValue?amount.Value:0).Amount;"),
                ("nullable_fallback","int? amount=value.Amount;return new Value(amount.GetValueOrDefault(12/value.Amount)).Amount;"),
                ("nullable_default_fallback","int? amount=null;return new Value(amount.GetValueOrDefault()).Amount;"),
                ("nullable_coalesce","int? amount=value.Amount;return new Value(amount??(12/value.Amount)).Amount;"),
                ("nullable_add","int? a=value.Amount;int? b=2;int? result=a+b;return new Value(result.Value).Amount;"),
                ("nullable_divide","int? a=value.Amount;int? b=null;int? result=a/b;return new Value(result.GetValueOrDefault()).Amount;"),
                ("nullable_boolean","bool? a=true;bool? b=null;bool? result=a&b;return new Value(result.GetValueOrDefault()?1:0).Amount;"),
                ("early_return","if(value.Amount>0){return new Value(value.Amount).Amount;}return new Value(0).Amount;"),
                ("checked_add","return new Value(value.Amount+1).Amount;"),
                ("locals","int amount=value.Amount;amount=amount+1;return new Value(amount).Amount;"),
                ("branch","int amount;if(value.Amount>0){amount=value.Amount;}else{amount=0;}return new Value(amount).Amount;"),
                ("conditional","return new Value(value.Amount>0?value.Amount:0).Amount;"),
                ("short_circuit","return new Value(value.Amount>0&&value.Amount<10?value.Amount:0).Amount;")}) {
                string source=Source.Replace("return new Value(value.Amount).Amount;",item.Item2);
                if(item.Item1=="nested_initializer_same_owner" || item.Item1=="nullable_initializer_member") source=source.Replace("public readonly struct Value{public readonly int Amount;public Value(int amount){Amount=amount;}}","public sealed class Value{public int Amount{get;init;}"+(item.Item1=="nullable_initializer_member"?"public string? Text{get;init;}":"")+"}");
                if(item.Item1=="nullable_source_call") source=source.Replace("public static int Run(","private static string? Echo(string? text){return text;}public static int Run(");
                if(item.Item1=="enum_equality") source=source.Replace("namespace Data;","namespace Data;public enum Color{Red=0,Blue=1}");
                if(item.Item1=="reference_null") source=source.Replace("readonly struct Value","sealed class Value");
                if(item.Item1.StartsWith("reference_nullable_",StringComparison.Ordinal)) source=source.Replace("readonly struct Value","sealed class Value").Replace("Run(Value value)","Run(Value? value)");
                if(item.Item1=="reference_nullable_call") source=source.Replace("public Value(int amount)","public int Plus(int other){return Amount+other;}public Value(int amount)");
                if(item.Item1=="reference_nullable_conditional") source=source.Replace("public readonly int Amount;","public readonly int Amount;public readonly string Text;").Replace("Amount=amount;","Amount=amount;Text=\"x\";");
                if(item.Item1=="array_dynamic_initialize") source=source.Replace("public static int Run(","private static int CountStrings(string[] values){return values.Length;}public static int Run(");
                if(item.Item1=="array_dynamic_call") source=source.Replace("public static int Run(","private static int Count(int[] values){return values.Length;}public static int Run(");
                if(item.Item1=="unused_assigned_getter") source=source.Replace("public readonly int Amount;","public readonly int Amount;public int Extra{get;}").Replace("Amount=amount;","Amount=amount;Extra=0;");
                if(item.Item1=="property_constructor") source=source.Replace("public readonly int Amount;","public int Amount{get;}");
                if(item.Item1=="delegated_constructor") source=source.Replace("public Value(int amount){Amount=amount;}","public Value(int amount):this(amount,true){}public Value(int amount,bool ignored){Amount=amount;}");
                if(item.Item1=="constructor_early_return") source=source.Replace("Amount=amount;","if(amount>0){Amount=amount;return;}Amount=0;");
                if(item.Item1=="instant_carrier") source=source.Replace("readonly int Amount","readonly long Amount").Replace("Value(int amount)","Value(long amount)");
                if(item.Item1=="money_carrier") source=source.Replace("public readonly int Amount;public Value(int amount){Amount=amount;}","public readonly decimal Amount;public readonly string Currency;public Value(decimal amount,string currency){Amount=amount;Currency=currency;}");
                PracticalDataSource captured;try{captured=Run(source);CSharpPracticalDataPhase.ValidateCandidate(captured,Run(source).CopyBytes());}catch(Exception e){throw new Exception(item.Item1+(e is PracticalCaptureFailure f ? ":"+f.Code : ""),e);}
                using var parsed=JsonDocument.Parse(captured.CopyBytes());
                cases.Add(new{name=item.Item1,source_utf8=source,facts=parsed.RootElement.Clone()});
            }
            foreach(var variant in new[]{("a","4","9"),("b","-7","103")}) {
                string source="using System;namespace Business;public enum Tag{Ok=0,Error=1}public enum Fault{Precision="+variant.Item2+",Range="+variant.Item3+"}public readonly struct Instant{public readonly long Milliseconds;public Instant(long milliseconds){Milliseconds=milliseconds;}}public readonly struct Outcome{public readonly Tag Tag;public readonly Instant Value;public readonly Fault Error;public Outcome(Tag tag,Instant value,Fault error){Tag=tag;Value=value;Error=error;}}public static class Entry{public static Outcome Run(Instant a,TimeSpan d){return new Outcome(Tag.Ok,new Instant(a.Milliseconds),Fault.Range);}}\n";
                string instant=PracticalIdentity.SourceTypeId("Business","Instant"),outcome=PracticalIdentity.SourceTypeId("Business","Outcome"),fault=PracticalIdentity.SourceTypeId("Business","Fault");
                string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{instant,PracticalIdentity.PrimitiveId("duration")},outcome);
                var results=new[]{new PracticalOutcomeBinding(outcome,"result",new Dictionary<string,string>{{"tag","Tag"},{"value","Value"},{"error","Error"}},new Dictionary<string,string>{{"ok","0"},{"error","1"}},new Dictionary<string,string>())};
                var errors=new Dictionary<string,IReadOnlyDictionary<string,string>>{{fault,new Dictionary<string,string>{{"precision",variant.Item2},{"range",variant.Item3}}}};
                var binding=new PracticalBusinessBinding(instant,"instant",new Dictionary<string,string>{{"milliseconds","Milliseconds"}},new Dictionary<string,string>{{"add_duration",root}},errors);
                PracticalDataSource Capture()=>CSharpPracticalDataPhase.Capture(new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"data",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>()),new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source))},references,new[]{binding},results);
                var captured=Capture();CSharpPracticalDataPhase.ValidateCandidate(captured,Capture().CopyBytes());
                using var parsed=JsonDocument.Parse(captured.CopyBytes());
                cases.Add(new{name="fallible_instant_"+variant.Item1,source_utf8=source,facts=parsed.RootElement.Clone()});
            }
            {
                string source="using System;namespace Business;public enum Tag{Ok=0,Error=1}public enum Fault{Scale=7,Rounding=11,Overflow=19}public enum Mode{Even=10,Away=20,Zero=30,Down=40,Up=50}public readonly struct Money{public readonly decimal Amount;public readonly string Currency;public Money(decimal amount,string currency){Amount=amount;Currency=currency;}}public readonly struct Outcome{public readonly Tag Tag;public readonly Money Value;public readonly Fault Error;public Outcome(Tag tag,Money value,Fault error){Tag=tag;Value=value;Error=error;}}public static class Entry{public static Outcome Run(Money a,decimal multiplier,int scale,Mode mode){return new Outcome(Tag.Ok,new Money(a.Amount,a.Currency),Fault.Scale);}}\n";
                string money=PracticalIdentity.SourceTypeId("Business","Money"),outcome=PracticalIdentity.SourceTypeId("Business","Outcome"),fault=PracticalIdentity.SourceTypeId("Business","Fault"),mode=PracticalIdentity.SourceTypeId("Business","Mode");
                string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{money,PracticalIdentity.PrimitiveId("decimal"),PracticalIdentity.PrimitiveId("i32"),mode},outcome);
                var results=new[]{new PracticalOutcomeBinding(outcome,"result",new Dictionary<string,string>{{"tag","Tag"},{"value","Value"},{"error","Error"}},new Dictionary<string,string>{{"ok","0"},{"error","1"}},new Dictionary<string,string>())};
                var enums=new Dictionary<string,IReadOnlyDictionary<string,string>> {
                    {fault,new Dictionary<string,string>{{"invalid_scale","7"},{"invalid_rounding","11"},{"decimal_overflow","19"}}},
                    {mode,new Dictionary<string,string>{{"ToEven","10"},{"AwayFromZero","20"},{"ToZero","30"},{"ToNegativeInfinity","40"},{"ToPositiveInfinity","50"}}},
                };
                var binding=new PracticalBusinessBinding(money,"money",new Dictionary<string,string>{{"amount","Amount"},{"currency","Currency"}},new Dictionary<string,string>{{"multiply",root}},enums);
                PracticalDataSource Capture()=>CSharpPracticalDataPhase.Capture(new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"data",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>()),new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source))},references,new[]{binding},results);
                var captured=Capture();CSharpPracticalDataPhase.ValidateCandidate(captured,Capture().CopyBytes());
                using var parsed=JsonDocument.Parse(captured.CopyBytes());cases.Add(new{name="fallible_money",source_utf8=source,facts=parsed.RootElement.Clone()});
            }
            // Fixed source fuzz seeds stress legal operation depth and checked
            // exceptional chains. Every seed goes through the real compiler.
            foreach(int seed in new[]{0,1,2,3,5,8,13,21,34,55,128,500}) {
                string expression="value.Amount";
                for(int depth=0;depth<seed;depth++) expression="-("+expression+")";
                string source=Source.Replace("new Value(value.Amount)","new Value("+expression+")");
                var captured=Run(source);CSharpPracticalDataPhase.ValidateCandidate(captured,Run(source).CopyBytes());
                using var facts=JsonDocument.Parse(captured.CopyBytes());
                cases.Add(new{name="fuzz_unary_"+seed.ToString("D3",System.Globalization.CultureInfo.InvariantCulture),source_utf8=source,facts=facts.RootElement.Clone()});
            }
            File.WriteAllBytes("data-source-cases.json",JsonSerializer.SerializeToUtf8Bytes(cases));
            return 0;
        } catch(Exception e) { Console.Error.WriteLine(e is PracticalCaptureFailure f ? f.Family+"/"+f.Code+"/"+e : e);return 1; }
    }
    private static void RunRequests()
    {
        using var document=JsonDocument.Parse(File.ReadAllBytes("data-requests.json"));
        var results=new List<object>();
        foreach(var row in document.RootElement.EnumerateArray()) {
            string id=row.GetProperty("id").GetString()!;
            try {
                var inputs=row.GetProperty("inputs").EnumerateArray().Select(i=>new PracticalCapturedInput(i.GetProperty("kind").GetString()=="source"?PracticalCapturedInputKind.Source:PracticalCapturedInputKind.Sidecar,i.GetProperty("path").GetString()!,Encoding.UTF8.GetBytes(i.GetProperty("utf8").GetString()!))).ToArray();
                var selection=new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,row.GetProperty("compilation_id").GetString()!,inputs.Where(i=>i.Kind==PracticalCapturedInputKind.Source).Select(i=>i.NormalizedPath),row.GetProperty("roots").EnumerateArray().Select(i=>i.GetString()!),inputs.Where(i=>i.Kind==PracticalCapturedInputKind.Sidecar).Select(i=>i.NormalizedPath));
                var captured=CSharpPracticalDataPhase.CaptureSelected(selection,inputs,references);
                CSharpPracticalDataPhase.ValidateCandidate(captured,CSharpPracticalDataPhase.CaptureSelected(selection,inputs,references).CopyBytes());
                using var facts=JsonDocument.Parse(captured.CopyBytes());results.Add(new{id,facts=facts.RootElement.Clone()});
            }catch(Exception e) {results.Add(new{id,reject=e is PracticalCaptureFailure f?f.Family+"/"+f.Code:e.GetType().Name});}
        }
        File.WriteAllBytes("data-source-responses.json",JsonSerializer.SerializeToUtf8Bytes(results));
    }
    private static void Reject(Action action) { try {action();} catch(PracticalCaptureFailure) {return;} throw new Exception("expected rejection"); }
}
