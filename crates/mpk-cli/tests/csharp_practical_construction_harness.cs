using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Text;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using System.Reflection;
using System.Runtime.Loader;
using System.Security.Cryptography;

namespace Mpk.CSharp2Vir;
internal static class PracticalConstructionHarness
{
    private static ImmutableArray<MetadataReference> references;
    private static string test = "bootstrap";
    public static int Main(string[] arguments)
    {
        try
        {
            references = Directory.EnumerateFiles(Path.Combine(arguments[0], "ref", "net10.0"), "*.dll")
                .OrderBy(path => path, StringComparer.Ordinal).Select(path => MetadataReference.CreateFromFile(path))
                .ToImmutableArray<MetadataReference>();
            test = "delegation";
            var delegated = Run("public readonly struct Data { public readonly int X; public readonly int Y; public Data(int x) { X=x; } public Data(int x,int y):this(x) { Y=y; } }", "return new Data(input,2).Y;");
            Check(delegated.Constructors.Any(plan => plan.DelegatesTo is not null && plan.DefinitelyAssigned == 3), "DELEGATION_STATE");
            var nonreturning = Run("public sealed class Data { public readonly int X; public Data(int x) { while(true){} } public Data(int x,int y):this(x){} }", "return new Data(input,2).X;");
            Check(nonreturning.Constructors.All(plan=>!plan.HasNormalExit) && nonreturning.Obligations.Count==0, "NONRETURNING_DELEGATION");
            test = "branches";
            Run("public sealed class Data { public readonly int X; public Data(int x) { if(x>0) X=x; else X=0; } }", "return new Data(input).X;");
            Reject("public sealed class Data { public readonly int X; public Data(int x) { if(x>0) X=x; X=0; } }", "return new Data(input).X;", "duplicate_member_assignment");
            Reject("public readonly struct Data { public readonly int X; public readonly int Y; public Data(int x) { if(x>0) X=x; Y=X; } }", "return new Data(input).Y;", "member_not_definitely_assigned");
            test = "delegation_duplicates";
            Reject("public sealed class Data { public readonly int X; public Data(int x) { X=x; } public Data(int x,int y):this(x) { X=y; } }", "return new Data(input,2).X;", "duplicate_member_assignment");
            Check(Run("public sealed class Data { public readonly int X; public Data(int x) { X=x>0 ? x:0; } }", "return new Data(input).X;")
                .Constructors.Single().DefinitelyAssigned==1, "CONDITIONAL_RHS_ASSIGNMENT");
            Reject("public sealed class Data { public readonly int X; public Data(int x) { X=x>0 ? x:0; X=2; } }", "return new Data(input).X;", "duplicate_member_assignment");
            Run("public readonly struct Data { public readonly string X; public Data(string x){X=x;} }", "return new Data(\"value\").X.Length;");
            Reject("public sealed class Data { public readonly int X; public Data(int x) { X=x; X++; } }", "return new Data(input).X;", "duplicate_member_assignment");
            Reject("public sealed class Data { public readonly int X; public Data(int x) { X=x; X+=1; } }", "return new Data(input).X;", "duplicate_member_assignment");
            Reject("public sealed class Data { public readonly int X; public Data(int x) { X=x; X+=x>0 ? 1:2; } }", "return new Data(input).X;", "duplicate_member_assignment");
            test = "loops";
            Reject("public sealed class Data { public readonly int X; public Data(int x) { while(x>0) { X=x; x--; } } }", "return new Data(input).X;", "duplicate_member_assignment");
            test = "unfinished_this";
            Reject("public sealed class Data { public readonly int X; public Data(int x) { X=Read(); } public int Read() => X; }", "return new Data(input).X;", "unfinished_receiver_escape");
            Reject("public sealed class Data { public int X {get;} public readonly int Y; public Data(int x) { X=x; Y=X; } }", "return new Data(input).Y;", "unfinished_receiver_escape");
            test = "synthesis";
            var synthesis = Run("public sealed class Data { public int X {get;} }", "return new Data().X;");
            Check(synthesis.Constructors.Single().Synthesized, "SYNTHESIZED_CTOR");
            Check(synthesis.Obligations.All(obligation => !obligation.Discharged), "NOT_PROOF");
            Reject("public sealed class Data { public string X {get;} }", "return new Data().X.Length;", null);
            test = "calls";
            var calls = Run("public sealed class Data { public int Add(int x) => x+1; }", "return new Data().Add(input);");
            Check(string.Join(',',calls.Calls.Single().EvaluationOrder)=="receiver,argument:0,call", "RECEIVER_FIRST");
            Check(calls.Calls.Single().NullCheckAfterArguments, "NULL_CHECK_POINT");
            var overloads = Run("public sealed class Data { public int Pick(int x)=>x; public long Pick(long x)=>x; }",
                "var d=new Data(); return d.Pick(input)+(int)d.Pick((long)input);");
            Check(overloads.Calls.Select(call=>call.Target).Distinct().Count()==2, "EXACT_OVERLOAD_SELECTION");
            test = "signatures";
            Reject("public sealed class Data { public Data(int x=0) {} }", "return new Data(input)==null ? 0:1;", null);
            Reject("public sealed class Data { public Data(int x) {} }", "Data value = new(input); return input;", "explicit_construction_required");
            Reject("public sealed class Data { public Data(int x) {} }", "var value = new Data(x:input); return input;", "argument_form");
            test = "invariants";
            InvariantAttachment();
            test = "synthesized_mutations";
            SynthesizedMutations();
            test = "runtime_equivalence";
            RuntimeEquivalence();
            test = "rejection_matrix";
            RejectionMatrix();
            test = "limits";
            foreach(int count in new[] {7,8,9})
            {
                string constructors = string.Join(" ",Enumerable.Range(1,count).Select(arity => "public Data("+string.Join(',',Enumerable.Range(0,arity).Select(i=>"int p"+i))+") {}"));
                string body = string.Join(" ",Enumerable.Range(1,count).Select(arity => "var d"+arity+" = new Data("+string.Join(',',Enumerable.Repeat("input",arity))+");"))+" return input;";
                if(count==9) Reject("public sealed class Data {"+constructors+"}",body,"constructors_per_type");
                else Check(Run("public sealed class Data {"+constructors+"}",body).Constructors.Count==count,"CTOR_LIMIT");
            }
            return 0;
        }
        catch(Exception error)
        {
            Console.Error.WriteLine("CONSTRUCTION_"+test+"_"+(error is PracticalCaptureFailure failure ? failure.Family+"_"+failure.Code : error.ToString()));
            return 1;
        }
    }
    private static PracticalConstruction Run(string declarations,string body,
        Func<string,IReadOnlyList<PracticalTypeInvariantClaim>>? claims = null)
    {
        string source="namespace Business; "+declarations+" public static class Entry { public static int Run(int input) { "+body+" } }\n";
        string type=PracticalIdentity.SourceTypeId("Business","Entry");
        string root=PracticalIdentity.CallableId("method","Business",type,"Run",new[]{PracticalIdentity.PrimitiveId("i32")},PracticalIdentity.PrimitiveId("i32"));
        return CSharpPracticalConstruction.Validate(new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"business",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>()),
            new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source))},references, claims?.Invoke(source));
    }
    private static PracticalInvariantExpression Bool(bool value) => new(PracticalInvariantOperator.Boolean,
        PracticalIdentity.PrimitiveId("bool"), value ? "true" : "false");
    private static IReadOnlyList<PracticalTypeInvariantClaim> Claim(string source, PracticalInvariantExpression expression,
        string? type = null, string? hash = null, PracticalInvariantExpression? construction = null) =>
        new[] { new PracticalTypeInvariantClaim(type ?? PracticalIdentity.SourceTypeId("Business", "Data"),
            hash ?? Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(source))).ToLowerInvariant(), construction, expression) };
    private static void InvariantAttachment()
    {
        const string declaration = "public readonly struct Data { public readonly bool Valid; public Data(int x) { Valid=x>0; } }";
        const string body = "return new Data(input).Valid ? 1:0;";
        var result = Run(declaration, body, source => Claim(source, Bool(false)));
        Check(result.Obligations.Count(obligation => obligation.Expression is not null)==2, "COMPLETE_CLAIM_EMISSION");
        Check(result.Obligations.All(obligation => !obligation.Discharged), "FALSE_IS_AN_OBLIGATION_NOT_PROOF");
        var member = new PracticalInvariantExpression(PracticalInvariantOperator.Member, PracticalIdentity.PrimitiveId("bool"), "Business.Data.Valid");
        Run(declaration, body, source => Claim(source, member));
        RejectAction(() => Run(declaration, body, source => Claim(source, member, hash:new string('0',64))), "invariant_attachment");
        RejectAction(() => Run(declaration, body, source => Claim(source, member, type:PracticalIdentity.SourceTypeId("Business","Other"))), "invariant_attachment");
        RejectAction(() => Run(declaration, body, source => Array.Empty<PracticalTypeInvariantClaim>()), "missing_type_invariant");
        RejectAction(() => Run(declaration, body, source => Claim(source,
            new(PracticalInvariantOperator.Int32, PracticalIdentity.PrimitiveId("i32"), "1"))), "ill_typed_invariant");
        RejectAction(() => Run(declaration, body, source => Claim(source,
            new(PracticalInvariantOperator.Member, PracticalIdentity.PrimitiveId("bool"), "Business.Other.Valid"))), "ill_typed_invariant");
        RejectAction(() => Run(declaration, body, source => Claim(source, member, construction:Bool(true))), "construction_invariant_attachment");
        const string initDeclaration = "public sealed class Data { public int X {get;init;} }";
        var init = Run(initDeclaration, "return new Data().X;", source => Claim(source, Bool(true), construction:Bool(false)));
        Check(init.Obligations.Any(obligation=>obligation.Kind=="declared_construction_invariant")
            && init.Obligations.Any(obligation=>obligation.Kind=="declared_public_invariant"), "CONSTRUCTION_AND_PUBLIC_CLAIMS");
        RejectAction(() => Run(initDeclaration,"return new Data().X;",source=>Claim(source,Bool(true))), "construction_invariant_attachment");
        var defaults = Run("public readonly struct Data { public readonly bool Valid; }", "return default(Data).Valid ? 1:0;", source => Claim(source, Bool(false)));
        Check(defaults.Obligations.Any(obligation => obligation.Expression is not null), "DEFAULT_CLAIM_PENDING");
        Check(!defaults.Data.Types.Single().DefaultEligible && defaults.Data.Types.Single().DefaultInvariantPending,
            "CLAIM_CANNOT_PROMOTE_DEFAULT_ELIGIBILITY");
        var nested = Run("public readonly struct Inner { public readonly bool Valid; } public readonly struct Data { public readonly Inner Nested; }",
            "return default(Data).Nested.Valid ? 1:0;", source => Claim(source, Bool(true)).Concat(
                Claim(source, Bool(false), type:PracticalIdentity.SourceTypeId("Business","Inner"))).ToArray());
        Check(nested.Obligations.Any(obligation => obligation.TypeId==PracticalIdentity.SourceTypeId("Business","Inner")
            && obligation.Kind=="declared_default_public_invariant" && obligation.Expression?.Value=="false"), "NESTED_DEFAULT_OBLIGATION");
    }
    private static void SynthesizedMutations()
    {
        byte[] body = {2,0x28,1,0,0,0x0a,0x2a};
        CSharpPracticalConstruction.RequireSynthesizedBody(body,new byte[]{2,0x28},0x0a000001);
        for(int index=0;index<body.Length;index++)
        {
            byte[] changed=(byte[])body.Clone(); changed[index]^=1;
            RejectAction(() => CSharpPracticalConstruction.RequireSynthesizedBody(changed,new byte[]{2,0x28},0x0a000001),"synthesized_body_shape");
        }
        RejectAction(() => CSharpPracticalConstruction.RequireSynthesizedBody(body.Concat(new byte[]{0}).ToArray(),new byte[]{2,0x28},0x0a000001),"synthesized_body_shape");
        Run("public sealed class Data { public int X {get; init;} }", "return new Data().X;");
    }
    private static void RuntimeEquivalence()
    {
        const string declaration="public sealed class Data { public readonly int X; public Data(int x){X=x;} public int Add(int y)=>X+y; }";
        const string body="return new Data(input).Add(2);";
        var result=Run(declaration,body);
        PracticalReceiverFunction function=result.Functions.Single(function=>function.Parameters.Count==2);
        Check(function.Parameters[0].Id==PracticalIdentity.SourceTypeId("Business","Data"),"RECEIVER_PARAMETER_ZERO");
        Check(result.Calls.Single().Operands.Count==2,"OPERANDS_ONCE");
        var block=Run(declaration.Replace("public int Add(int y)=>X+y;","public int Add(int y){return X+y;}"),body);
        Check(function.Body.CopyBodyBytes().SequenceEqual(block.Functions.Single(function=>function.Parameters.Count==2).Body.CopyBodyBytes()),"GETTER_METHOD_NORMALIZATION");
        string source="namespace Business; "+declaration+" public static class Entry { public static int Run(int input) {"+body+"} }";
        var compilation=CSharpCompilation.Create("construction_runtime",new[]{CSharpSyntaxTree.ParseText(source)},references,
            new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary,optimizationLevel:OptimizationLevel.Release));
        using var stream=new MemoryStream(); Check(compilation.Emit(stream).Success,"RUNTIME_EMIT"); stream.Position=0;
        var context=new AssemblyLoadContext("construction_runtime",true);
        try
        {
            Assembly assembly=context.LoadFromStream(stream); MethodInfo run=assembly.GetType("Business.Entry")!.GetMethod("Run")!;
            foreach(int value in new[]{-10,0,23}) Check((int)run.Invoke(null,new object[]{value})! == value+2,"RECEIVER_FIRST_RUNTIME");
        }
        finally {context.Unload();}
        const string nullableDeclaration = "public sealed class Data { public int Add(int x)=>x; }";
        const string nullableBody = "Data value=null!; return value.Add(1/input);";
        var nullable = Run(nullableDeclaration, nullableBody);
        Check(nullable.Calls.Single().NullCheckAfterArguments, "NULL_CHECK_AFTER_ARGUMENT_EVALUATION");
        string nullableSource = "namespace Business; " + nullableDeclaration + " public static class Entry { public static int Run(int input){" + nullableBody + "} }";
        var nullCompilation=CSharpCompilation.Create("null_call_runtime",new[]{CSharpSyntaxTree.ParseText(nullableSource)},references,
            new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));
        using var nullStream=new MemoryStream(); Check(nullCompilation.Emit(nullStream).Success,"NULL_RUNTIME_EMIT"); nullStream.Position=0;
        var nullContext=new AssemblyLoadContext("null_call_runtime",true);
        try
        {
            MethodInfo run=nullContext.LoadFromStream(nullStream).GetType("Business.Entry")!.GetMethod("Run")!;
            foreach(var item in new[]{(Input:0, Exception:typeof(DivideByZeroException)),(Input:1, Exception:typeof(NullReferenceException))})
            {
                try {run.Invoke(null,new object[]{item.Input}); throw new Exception("EXPECTED_RUNTIME_EXCEPTION");}
                catch(TargetInvocationException error) {Check(error.InnerException?.GetType()==item.Exception,"NULL_CALL_EVALUATION_ORDER");}
            }
        }
        finally {nullContext.Unload();}
    }
    private static void RejectionMatrix()
    {
        foreach(string parameter in new[]{"int x=0","params int[] x","in int x","ref int x","out int x","this int x"})
            Reject("public sealed class Data { public Data("+parameter+") {} }","return input;",null);
        Reject("public sealed class Data { public Data(int x){} }","var x=new { Value=input }; return input;",null);
        Reject("public sealed class Data { public int X {get;set;} }","return new Data().X;",null);
        Reject("public sealed class Data { public readonly int X=1; }","return new Data().X;",null);
        Reject("public sealed class Data { public Data(int x):this(x,0){} public Data(int x,int y):this(x){} }","var x=new Data(input); return input;",null);
        Reject("public sealed class Data { public int Read()=>1; public Data(int x){ Consume(this); } private static int Consume(Data d)=>d.Read(); }","var x=new Data(input); return input;","unfinished_receiver_escape");
        Reject("public readonly struct Data { public readonly int X; public Data(int x){this=default(Data);} }","return new Data(input).X;",null);
        Reject("public sealed class Data { public int X {get;init;} }","return new Data {X=input}.X;","initializer_requires_w05");
        Reject("public sealed class Data { public readonly int X; public Data(int x) { X=x; X=2; System.Console.WriteLine(x); } }","return new Data(input).X;","duplicate_member_assignment");
        Reject("public sealed class Data { public readonly int X; public Data(int x) => X=Read(); public int Read()=>X; }", "return new Data(input).X;", "expression_body_kind");
    }
    private static void RejectAction(Action action,string code)
    {
        try { action(); }
        catch(PracticalCaptureFailure failure) {Check(failure.Code==code,"EXPECTED_"+code+"_GOT_"+failure.Code); return;}
        throw new Exception("EXPECTED_REJECTION_"+code);
    }
    private static void Reject(string declaration,string body,string? code)
    {
        try { Run(declaration,body); }
        catch(PracticalCaptureFailure failure)
        { Check(code is null || failure.Code==code,"EXPECTED_"+code+"_GOT_"+failure.Code); Check(failure.ArtifactCount==0,"NO_ARTIFACT"); return; }
        throw new Exception("EXPECTED_REJECTION_"+code);
    }
    private static void Check(bool condition,string code) { if(!condition) throw new Exception(code); }
}
