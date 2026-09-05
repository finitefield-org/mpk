using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Text.Json;
using Microsoft.CodeAnalysis;
namespace Mpk.CSharp2Vir;
internal static class PracticalDomainHarness
{
    private static ImmutableArray<MetadataReference> references;
    private static string stage="start";
    public static int Main(string[] args)
    {
        try {
            references=Directory.EnumerateFiles(Path.Combine(args[0],"ref","net10.0"),"*.dll").OrderBy(p=>p,StringComparer.Ordinal).Select(p=>MetadataReference.CreateFromFile(p)).ToImmutableArray<MetadataReference>();
            foreach(string type in new[]{"int","long","float","double","decimal"}) {
                foreach(var pair in new[]{("a+b","add"),("a-b","subtract"),("a*b","multiply"),("a/b","divide"),("a%b","remainder"),("+a","plus"),("-a","negate")}){CheckSource(type,"return "+pair.Item1+";","lifted."+Token(type)+"."+pair.Item2);}
                foreach(var pair in new[]{("a==b","equal"),("a!=b","not_equal"),("a<b","less"),("a<=b","less_equal"),("a>b","greater"),("a>=b","greater_equal")}){CheckSource(type,"return "+pair.Item1+";","lifted."+Token(type)+"."+pair.Item2,"bool");}
            }
            foreach(var pair in new[]{("a&b","and"),("a|b","or"),("!a","not")}){CheckSource("bool","return "+pair.Item1+";","lifted.bool."+pair.Item2);}
            CheckSource("int","return a.HasValue;","nullable.has_value","bool");
            CheckSource("int","if(a.HasValue){return a.Value;}return 0;","nullable.value","int");
            CheckSource("int","return a.GetValueOrDefault();","nullable.value_or_default","int");
            CheckSource("int","return a.GetValueOrDefault(7);","nullable.value_or","int");
            CheckSource("int","return a??7;","nullable.coalesce","int");
            CheckSource("int","return default(int?);","nullable.none");
            CheckSource("int","return null;","nullable.none");
            CheckSource("int","return 7;","nullable.some");
            foreach(var body in new[]{"return new int?();","return default;","return (int?)7;","return a^b;","return a<<1;","a??=7;return a;","return (long?)a;"}){stage=body;Reject(()=>Run("int",body,body.Contains("long",StringComparison.Ordinal)?"long?":"int?"));}
            CheckSource("int","if(a.HasValue){return a.Value;}throw new System.InvalidOperationException();","outcome.source_throw","int");
            CheckSource("int","if(a.HasValue){return a.Value;}throw new System.ArgumentException();","outcome.source_throw","int");
            foreach(string body in new[]{"var e=new System.InvalidOperationException();throw e;","throw new System.InvalidOperationException(\"bad\");","throw new System.Exception();"}){stage=body;Reject(()=>Run("int",body,"int"));}
            stage="runtime_remainder_min";
            int overflow=0;try{_ = Remainder(int.MinValue,-1);}catch(OverflowException){overflow++;}try{_ = Remainder(long.MinValue,-1);}catch(OverflowException){overflow++;}Check(overflow==2,"REMAINDER_MIN");
            Defaults();
            References();
            Bindings();
            // Invoke the byte-bound, unchanged frozen runtime probe. Reflection
            // is test infrastructure only; no reflection is admitted as source.
            MethodInfo evaluate=typeof(global::FoundationDataProbe).GetMethod("Evaluate",BindingFlags.NonPublic|BindingFlags.Static)!;
            using var fixture=JsonDocument.Parse(File.ReadAllBytes("domain-runtime.json"));
            foreach(string separator in new[]{",",":"}) {
                var culture=(CultureInfo)CultureInfo.InvariantCulture.Clone();culture.NumberFormat.NumberDecimalSeparator=separator;culture.NumberFormat.NegativeSign="~";CultureInfo.CurrentCulture=culture;CultureInfo.CurrentUICulture=culture;
                foreach(var row in fixture.RootElement.EnumerateArray()) {
                    stage=row.GetProperty("id").GetString()!;string kind="value";string[] actual;
                    try{actual=(string[])evaluate.Invoke(null,new object[]{row.GetProperty("operation").GetString()!,row.GetProperty("inputs").EnumerateArray().Select(p=>p.GetString()!).ToArray()})!;}
                    catch(TargetInvocationException error){kind="exception";actual=new[]{error.InnerException!.GetType().FullName!};}
                    var expected=row.GetProperty("expected");Check(kind==expected.GetProperty("kind").GetString()&&actual.SequenceEqual(expected.GetProperty("value").EnumerateArray().Select(p=>p.GetString()!)),"RUNTIME_DIFFERENTIAL");
                }
            }
            return 0;
        }catch(Exception error){Console.Error.WriteLine("DOMAIN_"+stage+"_"+(error is PracticalCaptureFailure f?f.Family+"_"+f.Code:error.ToString()));return 1;}
    }
    private static int Remainder(int a,int b)=>unchecked(a%b);
    private static long Remainder(long a,long b)=>unchecked(a%b);
    private static string Token(string s)=>s switch{"float"=>"f32","double"=>"f64","int"=>"i32","long"=>"i64",_=>s};
    private static string Id(string s)=>s.EndsWith("?",StringComparison.Ordinal)?PracticalIdentity.ClosedInstanceId("option",Id(s[..^1])):PracticalIdentity.PrimitiveId(Token(s));
    private static PracticalDomain CheckSource(string type,string body,string operation,string? result=null)
    {
        stage=type+":"+body;var actual=Run(type,body,result??type+"?");Check(actual.ArtifactCount==0&&actual.Steps.Any(s=>s.Operation==operation),"SOURCE_OPERATION");
        if(operation is "lifted.i32.remainder" or "lifted.i64.remainder" or "lifted.i32.divide" or "lifted.i64.divide"){Check(actual.Steps.Single(s=>s.Operation==operation).Exceptions.SequenceEqual(new[]{"System.DivideByZeroException","System.OverflowException"}),"INTEGER_EXCEPTION_EDGES");}
        var bytes=actual.CopyCanonicalBytes();CSharpPracticalDomain.ValidateCandidate(actual,Run(type,body,result??type+"?").CopyCanonicalBytes());bytes[bytes.Length/2]^=1;Reject(()=>CSharpPracticalDomain.ValidateCandidate(actual,bytes));return actual;
    }
    private static PracticalDomain Run(string type,string body,string result)
    {
        string source="namespace Business;public static class Entry{public static "+result+" Run("+type+"? a,"+type+"? b){"+body+"}}\n";
        string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{Id(type+"?"),Id(type+"?")},Id(result));
        return Source(source,root);
    }
    private static PracticalDomain Source(string source,string root,PracticalOutcomeBinding[]? bindings=null)=>CSharpPracticalDomain.Validate(new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"business",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>()),new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source))},references,bindings);
    private static void Defaults()
    {
        string payload=PracticalIdentity.SourceTypeId("Business","Payload");
        string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{PracticalIdentity.ClosedInstanceId("option",payload)},payload);
        foreach(bool nullable in new[]{true,false}) {
            stage="struct_default:"+nullable;
            string source="namespace Business;public readonly struct Payload{public readonly string"+(nullable?"?":"")+" Text;public Payload(string"+(nullable?"?":"")+" text){Text=text;}}public static class Entry{public static Payload Run(Payload? p){var sentinel=new Payload("+(nullable?"null":"\"sentinel\"")+");if(p.HasValue){return p.GetValueOrDefault();}return sentinel;}}\n";
            if(nullable){Check(Source(source,root).Steps.Any(s=>s.Operation=="nullable.value_or_default"),"NULLABLE_MEMBER_DEFAULT");}else{Reject(()=>Source(source,root));}
        }
    }
    private static void References()
    {
        string box=PracticalIdentity.SourceTypeId("Business","Box");
        string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{box},Id("string"));
        string SourceText(string body,string getter="")=>"namespace Business;public sealed class Box{public readonly string Value;public Box(string value){Value=value;}"+getter+"}public static class Entry{public static string? Run(Box? b){new Box(\"x\");"+body+"}}\n";
        foreach(var pair in new[]{("return b?.Value;","reference.conditional_access"),("return b?.Text;","reference.conditional_access"),("if(b is not null){return b!.Value;}return null;","reference.suppression_identity"),("if(null==b){return null;}return b.Value;","reference.is_null")}) {
            stage=pair.Item1;var result=Source(SourceText(pair.Item1,pair.Item1.Contains("Text",StringComparison.Ordinal)?"public string Text=>Value;":""),root);Check(result.Steps.Any(s=>s.Operation==pair.Item2),"REFERENCE_OPERATION");Check(result.Obligations.Any(o=>o.Kind=="non_null_stored_field")&&result.Obligations.All(o=>!o.Discharged),"REFERENCE_INVARIANTS");
        }
        foreach(string body in new[]{"return b!.Value;","if(b is not null){b=null;return b!.Value;}return null;","return b?.Text;"}) {
            stage=body;Reject(()=>Source(SourceText(body,body=="return b?.Text;"?"private string Read(){return Value;}public string Text{get{return Read();}}":""),root));
        }
    }
    private static void Bindings()
    {
        foreach(string role in new[]{"option","lookup","result","validation","boundary_field"}) {
            stage="binding:"+role;string[] arms=role switch{"option"=>new[]{"none","some"},"lookup"=>new[]{"missing_key","found"},"result"=>new[]{"ok","error"},"validation"=>new[]{"valid","invalid"},_=>new[]{"missing","null","value"}};
            string extra=role=="result"?"public readonly int Error;":role=="validation"?"public readonly int[] Errors;":"";
            string source="namespace Business;public enum Tag{"+string.Join(",",arms.Select((a,i)=>"Arm"+i+"="+i))+"}public readonly struct Outcome{public readonly Tag Tag;public readonly int Value;public readonly int Extra;"+extra+"public Outcome(Tag tag,int value,int extra"+(role=="result"?",int error":role=="validation"?",int[] errors":"")+"){Tag=tag;Value=value;Extra=extra;"+(role=="result"?"Error=error;":role=="validation"?"Errors=errors;":"")+"}}public static class Entry{public static int Run(Outcome o){var copy=new Outcome(o.Tag,o.Value,o.Extra"+(role=="result"?",o.Error":role=="validation"?",o.Errors":"")+");return copy.Value;}}\n";
            string owner=PracticalIdentity.SourceTypeId("Business","Outcome");string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{owner},Id("int"));
            var members=new Dictionary<string,string>{{"tag","Tag"},{"value","Value"}};if(role=="result"){members.Add("error","Error");}if(role=="validation"){members.Add("errors","Errors");}
            var tags=arms.Select((a,i)=>(a,i)).ToDictionary(p=>p.a,p=>p.i.ToString(CultureInfo.InvariantCulture));
            var binding=new PracticalOutcomeBinding(owner,role,members,tags,new Dictionary<string,string>());
            var actual=Source(source,root,new[]{binding});Check(actual.Projections[0].DefaultEligible==(role is "option" or "lookup"),"BINDING_DEFAULT");
            Check(actual.Projections.Count==1&&actual.Obligations.All(o=>!o.Discharged)&&actual.Obligations.Any(o=>o.Kind=="field_complete_reconstruction"&&o.Member=="Extra"),"BINDING_OBLIGATIONS");
            Reject(()=>Source(source,root,new[]{binding,binding}));
            var bad=new Dictionary<string,string>(tags);bad[arms[1]]=bad[arms[0]];Reject(()=>Source(source,root,new[]{binding with{Tags=bad}}));
            var wrong=new Dictionary<string,string>(members);wrong["value"]="ExtraMissing";Reject(()=>Source(source,root,new[]{binding with{Members=wrong}}));
        }
    }
    private static void Reject(Action action){bool rejected=false;try{action();}catch(PracticalCaptureFailure f){if(f.Family==PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL)throw;rejected=true;}Check(rejected,"EXPECTED_REJECTION");}
    private static void Check(bool condition,string message){if(!condition)throw new Exception(message);}
}
