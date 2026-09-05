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
internal static class PracticalBusinessHarness
{
 private static ImmutableArray<MetadataReference> references;
 private static string stage="start";
 public static int Main(string[] args)
 {
  try {
   references=Directory.EnumerateFiles(Path.Combine(args[0],"ref","net10.0"),"*.dll").OrderBy(p=>p,StringComparer.Ordinal).Select(p=>MetadataReference.CreateFromFile(p)).ToImmutableArray<MetadataReference>();
   foreach(string t in new[]{"DateOnly","TimeOnly","TimeSpan","Guid"}) {
    CheckSource(t,"return a.CompareTo(b);","int",Token(t)+".compare");
    foreach(var p in new[]{("==","equal"),("!=","not_equal"),("<","less"),("<=","less_equal"),(">","greater"),(">=","greater_equal")}){if(t=="Guid"&&p.Item1 is not ("==" or "!=")){continue;}CheckSource(t,"return a"+p.Item1+"b;","bool",Token(t)+"."+p.Item2);}
   }
   foreach(string p in new[]{"Year","Month","Day","DayNumber","DayOfWeek"}){CheckSource("DateOnly","return a."+p+";",p=="DayOfWeek"?"DayOfWeek":"int","date."+(p=="DayNumber"?"day_number":p=="DayOfWeek"?"day_of_week":p.ToLowerInvariant()));}
   foreach(string p in new[]{"Ticks","Hour","Minute","Second","Millisecond"}){CheckSource("TimeOnly","return a."+p+";",p=="Ticks"?"long":"int","time."+p.ToLowerInvariant());}
   foreach(string p in new[]{"Ticks","Days","Hours","Minutes","Seconds","Milliseconds"}){CheckSource("TimeSpan","return a."+p+";",p=="Ticks"?"long":"int","duration."+p.ToLowerInvariant());}
   foreach(var p in new[]{("AddDays","add_days"),("AddMonths","add_months"),("AddYears","add_years")}){CheckSource("DateOnly","return a."+p.Item1+"(-2147483648);","DateOnly","date."+p.Item2);}
   CheckSource("DateOnly","return new DateOnly(2000,2,29);","DateOnly","date.construct");
   CheckSource("TimeOnly","return new TimeOnly(9223372036854775807L);","TimeOnly","time.construct");
   CheckSource("TimeSpan","return new TimeSpan(-9223372036854775808L);","TimeSpan","duration.construct");
   CheckSource("TimeOnly","return a.Add(new TimeSpan(-9223372036854775808L));","TimeOnly","time.add_duration");
   CheckSource("TimeOnly","return a-b;","TimeSpan","time.subtract");
   foreach(var p in new[]{("a+b","add"),("a-b","subtract"),("-a","negate")}){CheckSource("TimeSpan","return "+p.Item1+";","TimeSpan","duration."+p.Item2);}
   CheckSource("Guid","return Guid.Empty;","Guid","guid.empty");
   foreach(var p in new[]{("DateOnly","return DateOnly.FromDateTime(DateTime.Now);","DateOnly"),("DateOnly","return a.ToString();","string"),("TimeOnly","return a.AddHours(1);","TimeOnly"),("TimeOnly","int days;return a.Add(new TimeSpan(1),out days);","TimeOnly"),("TimeSpan","return TimeSpan.FromSeconds(1);","TimeSpan"),("TimeSpan","return a.TotalMilliseconds;","double"),("Guid","return Guid.NewGuid();","Guid"),("Guid","return Guid.Parse(\"00000000-0000-0000-0000-000000000000\");","Guid"),("Guid","return a.ToByteArray();","byte[]"),("Guid","return a.GetHashCode();","int"),("DateOnly","return (int)a.DayOfWeek;","int")}){stage=p.Item2;Reject(()=>Run(p.Item1,p.Item2,p.Item3));}
   var integer=Run("long","return a;","long");Check(integer.Projections.Count==0&&integer.Steps.Count==0,"UNCLASSIFIED_I64");
   MoneyDefaults();
   Bindings();
   FallibleBindings();
   // Test-only reflection invokes the unchanged, byte-bound frozen probe.
   MethodInfo evaluate=typeof(global::FoundationDataProbe).GetMethod("Evaluate",BindingFlags.NonPublic|BindingFlags.Static)!;
   using var fixture=JsonDocument.Parse(File.ReadAllBytes("business-runtime.json"));Check(fixture.RootElement.GetArrayLength()==994,"COUNT");
   foreach(string separator in new[]{",",":"}){var culture=(CultureInfo)CultureInfo.InvariantCulture.Clone();culture.NumberFormat.NumberDecimalSeparator=separator;culture.NumberFormat.NegativeSign="~";CultureInfo.CurrentCulture=culture;CultureInfo.CurrentUICulture=culture;
    foreach(var row in fixture.RootElement.EnumerateArray()){stage=row.GetProperty("id").GetString()!;string kind="value";string[] actual;try{actual=(string[])evaluate.Invoke(null,new object[]{row.GetProperty("operation").GetString()!,row.GetProperty("inputs").EnumerateArray().Select(p=>p.GetString()!).ToArray()})!;}catch(TargetInvocationException error){kind="exception";actual=new[]{error.InnerException!.GetType().FullName!};}var expected=row.GetProperty("expected");Check(kind==expected.GetProperty("kind").GetString()&&actual.SequenceEqual(expected.GetProperty("value").EnumerateArray().Select(p=>p.GetString()!)),"DIFFERENTIAL");}
   }
   return 0;
  }catch(Exception e){Console.Error.WriteLine("BUSINESS_"+stage+"_"+(e is PracticalCaptureFailure f?f.Family+"_"+f.Code:e.ToString()));return 1;}
 }
 private static string Token(string t)=>t switch{"DateOnly"=>"date","TimeOnly"=>"time","TimeSpan"=>"duration","Guid"=>"guid","DayOfWeek"=>"day_of_week","int"=>"i32","long"=>"i64","double"=>"f64",_=>t};
 private static string Id(string t)=>t=="byte[]"?PracticalIdentity.ClosedInstanceId("bounded_sequence",PracticalIdentity.PrimitiveId("u8")):PracticalIdentity.PrimitiveId(Token(t));
 private static PracticalBusiness Run(string type,string body,string result){string source="using System;namespace Business;public static class Entry{public static "+result+" Run("+type+" a,"+type+" b){"+body+"}}\n";string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{Id(type),Id(type)},Id(result));return Source(source,root);}
 private static void CheckSource(string type,string body,string result,string op){stage=body;var actual=Run(type,body,result);Check(actual.ArtifactCount==0&&actual.Steps.Any(s=>s.Operation==op),"SOURCE");Check(actual.Steps.All(s=>s.Operands.All(o=>o is not null)),"OPERANDS");CSharpPracticalBusiness.ValidateCandidate(actual,Run(type,body,result).CopyCanonicalBytes());var bytes=actual.CopyCanonicalBytes();bytes[bytes.Length/2]^=1;Reject(()=>CSharpPracticalBusiness.ValidateCandidate(actual,bytes));}
 private static PracticalBusiness Source(string source,string root,PracticalBusinessBinding[]? bindings=null)=>CSharpPracticalBusiness.Validate(new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"business",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>()),new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source))},references,bindings);
 private static void Bindings(){foreach(string role in new[]{"instant","money"}){stage=role;string owner=PracticalIdentity.SourceTypeId("Business","Value");string fields=role=="instant"?"public readonly long Milliseconds;":"public readonly decimal Amount;public readonly string Currency;";string parameters=role=="instant"?"long milliseconds":"decimal amount,string currency";string assigns=role=="instant"?"Milliseconds=milliseconds;":"Amount=amount;Currency=currency;";string args=role=="instant"?"v.Milliseconds":"v.Amount,v.Currency";string returned=role=="instant"?"long":"decimal";string member=role=="instant"?"Milliseconds":"Amount";
 string source="namespace Business;public readonly struct Value{"+fields+"public readonly int Extra;public Value("+parameters+",int extra){"+assigns+"Extra=extra;}}public static class Entry{public static "+returned+" Run(Value v){var copy=new Value("+args+",v.Extra);return copy."+member+";}}\n";
 string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{owner},Id(returned));var members=role=="instant"?new Dictionary<string,string>{{"milliseconds","Milliseconds"}}:new Dictionary<string,string>{{"amount","Amount"},{"currency","Currency"}};var binding=new PracticalBusinessBinding(owner,role,members,new Dictionary<string,string>());var actual=Source(source,root,new[]{binding});Check(actual.Projections.Count==1&&actual.Obligations.All(o=>!o.Discharged)&&actual.Obligations.Any(o=>o.Kind=="field_complete_reconstruction"&&o.Member=="Extra"),"BINDING");Check(actual.Projections[0].SemanticTypeId==(role=="instant"?Id("instant"):PracticalIdentity.ClosedInstanceId("money",Id("string"))),"IDENTITY");Reject(()=>Source(source,root,new[]{binding,binding}));var bad=new Dictionary<string,string>(members);bad[role=="instant"?"milliseconds":"amount"]="Extra";Reject(()=>Source(source,root,new[]{binding with{Members=bad}}));Reject(()=>Source(source,root,new[]{binding with{Operations=new Dictionary<string,string>{{"nonsense",root}}}}));
 }}
 private static void MoneyDefaults(){
  string owner=PracticalIdentity.SourceTypeId("Business","Cash");var binding=new PracticalBusinessBinding(owner,"money",new Dictionary<string,string>{{"amount","Amount"},{"currency","Currency"}},new Dictionary<string,string>());
  foreach(string expression in new[]{"default(Cash)","default","new Cash()","a.GetValueOrDefault()"}){
   stage="money_default:"+expression;bool optional=expression.Contains("GetValue",StringComparison.Ordinal);
   string source="namespace Business;public enum Currency{A=0,B=1}public readonly struct Cash{public readonly decimal Amount;public readonly Currency Currency;public Cash(decimal amount,Currency currency){Amount=amount;Currency=currency;}}public static class Entry{public static Cash Run(Cash"+(optional?"?":"")+" a){new Cash(1,Currency.A);return "+expression+";}}\n";
   string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{optional?PracticalIdentity.ClosedInstanceId("option",owner):owner},owner);
   Reject(()=>Source(source,root,new[]{binding}));
  }
 }
 private static void FallibleBindings() {
  stage="fallible_binding";
  string source="using System;namespace Business;public enum Tag{Ok=0,Error=1}public enum Fault{Precision=4,Range=9}public readonly struct Instant{public readonly long Milliseconds;public Instant(long milliseconds){Milliseconds=milliseconds;}}public readonly struct Outcome{public readonly Tag Tag;public readonly Instant Value;public readonly Fault Error;public Outcome(Tag tag,Instant value,Fault error){Tag=tag;Value=value;Error=error;}}public static class Entry{public static Outcome Run(Instant a,TimeSpan d){return new Outcome(Tag.Ok,new Instant(a.Milliseconds),Fault.Range);}}\n";
  string instant=PracticalIdentity.SourceTypeId("Business","Instant"),outcome=PracticalIdentity.SourceTypeId("Business","Outcome"),fault=PracticalIdentity.SourceTypeId("Business","Fault");
  string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{instant,Id("TimeSpan")},outcome);
  var results=new[]{new PracticalOutcomeBinding(outcome,"result",new Dictionary<string,string>{{"tag","Tag"},{"value","Value"},{"error","Error"}},new Dictionary<string,string>{{"ok","0"},{"error","1"}},new Dictionary<string,string>())};
  var errors=new Dictionary<string,IReadOnlyDictionary<string,string>>{{fault,new Dictionary<string,string>{{"precision","4"},{"range","9"}}}};
  var binding=new PracticalBusinessBinding(instant,"instant",new Dictionary<string,string>{{"milliseconds","Milliseconds"}},new Dictionary<string,string>{{"add_duration",root}},errors);
  PracticalBusiness Build(PracticalBusinessBinding b,PracticalOutcomeBinding[]? r)=>CSharpPracticalBusiness.Validate(new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"business",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>()),new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source))},references,new[]{b},r);
  var actual=Build(binding,results);
  // This helper deliberately ignores the duration. Capture must leave the
  // commutation VCs pending, so passing this test can never prove its body.
  Check(actual.Obligations.All(o=>!o.Discharged)&&actual.Obligations.Count(o=>o.Kind.StartsWith("operation_",StringComparison.Ordinal))==3,"PENDING_COMMUTATION");
  Reject(()=>Build(binding,null));Reject(()=>Build(binding with{EnumArms=null},results));
  var bad=new Dictionary<string,IReadOnlyDictionary<string,string>>{{fault,new Dictionary<string,string>{{"precision","4"},{"range","4"}}}};Reject(()=>Build(binding with{EnumArms=bad},results));
  Reject(()=>Build(binding with{Operations=new Dictionary<string,string>{{"milliseconds",root}}},results));
 }
 private static void Check(bool condition,string code){if(!condition){throw new InvalidOperationException(code);}}
 private static void Reject(Action action){try{action();}catch(PracticalCaptureFailure){return;}throw new InvalidOperationException("ACCEPTED");}
}
