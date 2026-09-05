using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Text;
using System.Reflection;
using System.Runtime.Loader;
using System.Security.Cryptography;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;
internal static class PracticalInitializationHarness
{
    private static ImmutableArray<MetadataReference> references;
    private static string test = "bootstrap";
    public static int Main(string[] arguments)
    {
        try
        {
            references = Directory.EnumerateFiles(Path.Combine(arguments[0], "ref", "net10.0"), "*.dll")
                .OrderBy(path=>path,StringComparer.Ordinal).Select(path=>MetadataReference.CreateFromFile(path)).ToImmutableArray<MetadataReference>();
            test="OrderedTransactions"; OrderedTransactions();
            test="RequiredAndDefaults"; RequiredAndDefaults();
            test="DuplicateAndMutationMatrix"; DuplicateAndMutationMatrix();
            test="InvariantSites"; InvariantSites();
            test="RuntimeOrderAndDiscard"; RuntimeOrderAndDiscard();
            test="ReplayTransaction"; ReplayTransaction();
            test="NestedCreationInConstructor"; NestedCreationInConstructor();
            return 0;
        }
        catch(Exception error)
        {
            Console.Error.WriteLine("INITIALIZATION_"+test+"_"+(error is PracticalCaptureFailure failure ? failure.Family+"_"+failure.Code : error.ToString()));
            return 1;
        }
    }
    private static string Source(string declaration,string body) => "namespace Business; "+declaration+" public static class Entry { public static int Run(int input){"+body+"} }\n";
    private static PracticalConstruction Run(string declaration,string body,Func<string,IReadOnlyList<PracticalTypeInvariantClaim>>? claims=null)
    {
        string source=Source(declaration,body);
        string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",
            new[]{PracticalIdentity.PrimitiveId("i32")},PracticalIdentity.PrimitiveId("i32"));
        return CSharpPracticalInitialization.Validate(new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"business",
            new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>()),
            new[]{new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source))},references,claims?.Invoke(source));
    }
    private static void OrderedTransactions()
    {
        const string declaration="public sealed class Data { public int A {get;init;} public int B {get;init;} public Data(int value){A=value;} }";
        var result=Run(declaration,"return new Data(input){B=input+1}.B;");
        var plan=result.Initializations.Single();
        Check(plan.DefinitelyAssigned==3 && plan.PossiblyAssigned==3,"CTOR_STATE_MERGED");
        Check(string.Join(',',plan.Steps.Select(step=>step.Kind))=="EvaluateArgument,Begin,InvokeConstructor,ConstructionInvariant,EvaluateInitializer,Assign,PublicInvariant,Finalize","EXACT_SEQUENCE");
        var ordered=Run("public sealed class Data { public int A {get;init;} public int B {get;init;} }","return new Data {B=input,A=input+1}.A;").Initializations.Single();
        Check(string.Join(',',ordered.Steps.Where(step=>step.Kind==PracticalInitializationStepKind.Assign).Select(step=>step.Target))=="B,A","SOURCE_ORDER");
        Check(string.Join(',',ordered.MemberOrder)=="A,B","DECLARATION_ORDER");
        Check(ordered.Steps.Where(step=>step.Kind==PracticalInitializationStepKind.EvaluateInitializer).All(step=>step.Expression is not null && step.ExceptionalExit=="discard"),"EXCEPTION_DISCARD_EDGES");
        Check(ordered.Steps.Last().Kind==PracticalInitializationStepKind.Finalize,"PUBLISH_ONLY_AFTER_CHECK");
        var empty=Run("public sealed class Data { public int A {get;init;} }","return new Data().A;").Initializations.Single();
        Check(empty.Steps.Last().Kind==PracticalInitializationStepKind.Finalize,"NO_INITIALIZER_STILL_FINALIZES");
        var equivalent=Run("public sealed class Data { public int A {get;init;} public int B {get;init;} public Data(int a,int b){A=a;B=b;} }","return new Data(input,input+1).B;");
        Check(equivalent.Initializations.Single().DefinitelyAssigned==plan.DefinitelyAssigned,"CTOR_INITIALIZER_EQUIVALENT_FINAL_STATE");
    }
    private static void RequiredAndDefaults()
    {
        const string declaration="public sealed class Data { public required string Name {get;init;} public int Count {get;init;} }";
        var value=Run(declaration,"return new Data {Name=\"ok\"}.Count;");
        Check(value.Initializations.Single().DefinitelyAssigned==1,"REQUIRED_COVERAGE");
        Check(value.Obligations.Any(obligation=>obligation.Kind=="recursive_default" && obligation.Members.Contains("Count")),"OPTIONAL_DEFAULT");
        Reject(declaration,"return new Data().Count;",null);
        Reject(declaration,"var good=new Data{Name=\"ok\"}; var bad=new Data(); return good.Count;",null);
        Reject("public sealed class Data { public required string Name {get;init;} public Data(string name){Name=name;} }","return new Data(\"x\"){Name=\"y\"}.Name.Length;","required_constructor_assignment");
        Run("public readonly struct Data { public required int A {get;init;} public int B {get;init;} }","return new Data{A=input,B=2}.A;");
    }
    private static void DuplicateAndMutationMatrix()
    {
        Reject("public sealed class Data { public int A {get;init;} public Data(int value){A=value;} }","return new Data(input){A=2}.A;","duplicate_member_assignment");
        Reject("public sealed class Data { public int A {get;init;} public Data(int value){if(value>0) A=value;} }","return new Data(input){A=2}.A;","duplicate_member_assignment");
        Reject("public sealed class Data { public int A {get;init;} public Data(int x){A=x;} public Data(int x,int y):this(x){} }","return new Data(input,2){A=3}.A;","duplicate_member_assignment");
        const string declaration="public sealed class Data { public int A {get;init;} }";
        Reject(declaration,"return new Data{A=1,A=2}.A;",null);
        Reject(declaration,"var value=new Data{A=1}; value.A=2; return value.A;",null);
        Reject("public readonly struct Data {public int A {get;init;}}","var value=new Data{A=1}; return (value with {A=input}).A;",null);
        Reject("public sealed class Data { public int A {get;set;} }","return new Data{A=1}.A;",null);
        Reject("public sealed class Data { public int A {get{return 0;} init{}} }","return new Data{A=1}.A;",null);
        Reject("public sealed class Data { public readonly int A; }","return new Data{A=1}.A;",null);
        Reject("public sealed class Inner { public int A {get;init;} } public sealed class Data {public Inner Child {get;} public Data(){Child=new Inner();}}","return new Data{Child={A=1}}.Child.A;",null);
        Reject("public sealed class Data { public required int A; }","return input;",null);
        Reject("public sealed class Data { public int A {get;init;} [System.Diagnostics.CodeAnalysis.SetsRequiredMembers] public Data(){} }","return new Data().A;",null);
        Reject("[System.Runtime.CompilerServices.RequiredMember] public sealed class Data { public int A {get;init;} }","return new Data().A;",null);
        Reject("public sealed class Data { public required int[] A {get;init;} }","var array=new int[]{1}; var value=new Data{A=array}; array[0]=input; return value.A[0];",null);
    }
    private static void InvariantSites()
    {
        const string declaration="public sealed class Data { public required bool A {get;init;} }";
        var result=Run(declaration,"return new Data{A=input>0}.A ? 1:0;",source=>new[]{new PracticalTypeInvariantClaim(
            PracticalIdentity.SourceTypeId("Business","Data"),Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(source))).ToLowerInvariant(),
            new(PracticalInvariantOperator.Boolean,PracticalIdentity.PrimitiveId("bool"),"true"),
            new PracticalInvariantExpression(PracticalInvariantOperator.Member,PracticalIdentity.PrimitiveId("bool"),"Business.Data.A"))});
        Check(result.Obligations.Count(obligation=>obligation.Kind=="declared_construction_invariant")==1,"CTOR_INVARIANT");
        Check(result.Obligations.Count(obligation=>obligation.Kind=="declared_public_invariant")==1,"FINAL_PUBLIC_INVARIANT");
        Check(result.Obligations.All(obligation=>!obligation.Discharged),"NOT_PROOF");
        var publicClaim=result.Obligations.Single(obligation=>obligation.Kind=="declared_public_invariant");
        Check(publicClaim.Site==result.Initializations.Single().Site,"CLAIM_ATTACHMENT");
    }
    private static void NestedCreationInConstructor()
    {
        var result=Run("public sealed class Inner { public int Value {get;init;} } public sealed class Data {public readonly Inner Child; public Data(int value){Child=new Inner{Value=value};}}","return new Data(input).Child.Value;");
        Check(result.Initializations.Count==2,"NESTED_FRESH_TRANSACTIONS");
        Check(result.Initializations.Select(plan=>plan.Site).Distinct().Count()==2,"UNIQUE_SITES");
        Reject("public sealed class Inner { public int Value {get;init;} } public sealed class Data {public readonly Inner Child; public Data(int value){Child=new Inner{Value=Consume(this)};} private static int Consume(Data value){return 1;}}","var value=new Data(input); return input;","unfinished_receiver_escape");
    }
    private static void RuntimeOrderAndDiscard()
    {
        const string declaration="public sealed class Data { public required int A {get;init;} public int B {get;init;} }";
        const string body="int value=input; var result=new Data{A=value++,B=value++}; return result.A*100+result.B;";
        Run(declaration,body); Runtime(Source(declaration,body),3,304,null);
        const string throwing="var result=new Data{A=1/input,B=2147483647+input}; return result.B;";
        var plan=Run(declaration,throwing).Initializations.Single();
        Check(plan.Steps.Count(step=>step.Kind==PracticalInitializationStepKind.EvaluateInitializer)==2,"EVALUATE_ONCE");
        Runtime(Source(declaration,throwing),0,0,typeof(DivideByZeroException));
        Runtime(Source(declaration,throwing),1,0,typeof(OverflowException));
        var nonreturning=Run("public sealed class Data { public int A {get;init;} public Data(int value){while(true){}} }","return new Data(input){A=1/input}.A;").Initializations.Single();
        Check(!nonreturning.HasNormalExit && nonreturning.Steps.All(step=>step.Kind!=PracticalInitializationStepKind.Finalize),"NO_NORMAL_CONSTRUCTOR_NO_PUBLICATION");
    }
    // Independent small interpreter for the scalar initializer plans exercised
    // here. Constructor bodies and general expression lowering retain their
    // separate owners; this test uses the exact inert synthesized constructor.
    private static void ReplayTransaction()
    {
        const string declaration="public sealed class Data { public required int A {get;init;} public int B {get;init;} public int C {get;init;} }";
        const string body="var result=new Data{A=input+1,B=1/input,C=99}; return result.A*100+result.B;";
        var plan=Run(declaration,body).Initializations.Single();
        foreach(int input in new[]{0,2})
        {
            Dictionary<string,int>? transaction=null;
            Dictionary<string,int>? published=null;
            int temporary=0, evaluations=0;
            try
            {
                foreach(var step in plan.Steps)
                {
                    switch(step.Kind)
                    {
                        case PracticalInitializationStepKind.Begin: transaction=new(); break;
                        case PracticalInitializationStepKind.InvokeConstructor:
                            Check(transaction is not null,"FRESH_BEFORE_CONSTRUCTOR"); break;
                        case PracticalInitializationStepKind.EvaluateInitializer:
                            evaluations++; Check(published is null,"UNOBSERVABLE_BEFORE_FINALIZE");
                            temporary=Evaluate(step.Expression!,input); break;
                        case PracticalInitializationStepKind.Assign: transaction![step.Target]=temporary; break;
                        case PracticalInitializationStepKind.Finalize: published=new(transaction!); transaction=null; break;
                    }
                }
                Check(input==2 && published is not null && published["A"]==3 && published["B"]==0
                    && published["C"]==99 && evaluations==3,"PLAN_RESULT");
                Runtime(Source(declaration,body),input,300,null);
            }
            catch(DivideByZeroException)
            {
                Check(plan.Steps.Where(step=>step.Kind==PracticalInitializationStepKind.EvaluateInitializer)
                    .ElementAt(evaluations-1).ExceptionalExit=="discard","DISCARD_BRANCH");
                transaction=null;
                Check(input==0 && published is null && evaluations==2,"EXCEPTION_NO_PUBLICATION_OR_LATER_RHS");
                Runtime(Source(declaration,body),input,0,typeof(DivideByZeroException));
            }
        }
    }
    private static int Evaluate(IOperation expression,int input) => expression switch
    {
        IParameterReferenceOperation => input,
        ILiteralOperation literal => (int)literal.ConstantValue.Value!,
        IBinaryOperation {OperatorKind:BinaryOperatorKind.Add} binary => checked(Evaluate(binary.LeftOperand,input)+Evaluate(binary.RightOperand,input)),
        IBinaryOperation {OperatorKind:BinaryOperatorKind.Divide} binary => Evaluate(binary.LeftOperand,input)/Evaluate(binary.RightOperand,input),
        _ => throw new Exception("UNEXPECTED_REPLAY_EXPRESSION"),
    };
    private static void Runtime(string source,int input,int expected,Type? exception)
    {
        var compilation=CSharpCompilation.Create("initialization_runtime",new[]{CSharpSyntaxTree.ParseText(source)},references,
            new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary,optimizationLevel:OptimizationLevel.Release,checkOverflow:true));
        using var stream=new MemoryStream(); Check(compilation.Emit(stream).Success,"RUNTIME_EMIT"); stream.Position=0;
        var context=new AssemblyLoadContext("initialization_runtime",true);
        try
        {
            var run=context.LoadFromStream(stream).GetType("Business.Entry")!.GetMethod("Run")!;
            try {var value=(int)run.Invoke(null,new object[]{input})!; Check(exception is null && value==expected,"RUNTIME_RESULT");}
            catch(TargetInvocationException error){Check(error.InnerException?.GetType()==exception,"RUNTIME_EXCEPTION_ORDER");}
        }
        finally{context.Unload();}
    }
    private static void Reject(string declaration,string body,string? code)
    {
        try{Run(declaration,body);}
        catch(PracticalCaptureFailure failure){Check(code is null || failure.Code==code,"EXPECTED_"+code+"_GOT_"+failure.Code); Check(failure.ArtifactCount==0,"NO_ARTIFACTS");return;}
        throw new Exception("EXPECTED_REJECTION_"+code);
    }
    private static void Check(bool condition,string code){if(!condition)throw new Exception(code);}
}
