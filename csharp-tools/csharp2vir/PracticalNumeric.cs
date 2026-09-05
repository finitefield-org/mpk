using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using System.Text.Json;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Operations;
namespace Mpk.CSharp2Vir;

// Per-expression typed recipes; captured bodies own evaluation/control order.
// No recipe discharges its commutation VC or creates a checker primitive.
internal sealed record PracticalNumericStep(string Site,string Operation,
    IReadOnlyList<IOperation> Operands,IReadOnlyList<string> ArgumentTypes,string ResultType,
    string Rounding,IReadOnlyList<string> Exceptions,string Relation="bounded_integer_numeric");
internal sealed record PracticalNumeric(PracticalStrings Strings,IReadOnlyList<PracticalNumericStep> Steps)
{
    internal int ArtifactCount=>0;
    internal byte[] CopyCanonicalBytes()=>JsonSerializer.SerializeToUtf8Bytes(new {
        source=Strings.Arrays.Construction.Data.Syntax.SemanticSha256,
        steps=Steps.Select(s=>new{s.Site,s.Operation,s.ArgumentTypes,s.ResultType,s.Rounding,s.Exceptions,s.Relation,
            operands=s.Operands.Select(o=>new{site=PracticalStrings.Site(o.Syntax),kind=o.Kind.ToString()})})});
}
internal static class CSharpPracticalNumeric
{
    internal static PracticalNumeric Validate(PracticalSourceSelection selection,IEnumerable<PracticalCapturedInput> inputs,
        ImmutableArray<MetadataReference> references, Action<CSharpCompilation>? validateDomain=null)
    {
        var steps=new List<PracticalNumericStep>();
        var strings=CSharpPracticalStrings.Validate(selection,inputs,references,validateNumeric:c=>{Analyze(c,steps,validateDomain is not null);validateDomain?.Invoke(c);},domainOperations:validateDomain is not null);
        return new(strings,Array.AsReadOnly(steps.OrderBy(s=>s.Site,StringComparer.Ordinal).ThenBy(s=>s.Operation,StringComparer.Ordinal).ToArray()));
    }
    internal static void ValidateCandidate(PracticalNumeric regenerated,ReadOnlySpan<byte> candidate)
    {if(!candidate.SequenceEqual(regenerated.CopyCanonicalBytes())){throw PracticalFailures.Type("numeric_handoff_mismatch");}}
    private static bool Numeric(ITypeSymbol? t)=>t?.SpecialType is SpecialType.System_Single or SpecialType.System_Double or SpecialType.System_Decimal;
    private static bool Integral(ITypeSymbol? t)=>t?.SpecialType is SpecialType.System_SByte or SpecialType.System_Byte or SpecialType.System_Int16 or SpecialType.System_UInt16 or SpecialType.System_Int32 or SpecialType.System_UInt32 or SpecialType.System_Int64 or SpecialType.System_UInt64 or SpecialType.System_Char;
    private static string Name(ITypeSymbol? t)=>t?.SpecialType switch {SpecialType.System_Single=>"single",SpecialType.System_Double=>"double",SpecialType.System_Decimal=>"decimal",SpecialType.System_SByte=>"sbyte",SpecialType.System_Byte=>"byte",SpecialType.System_Int16=>"int16",SpecialType.System_UInt16=>"uint16",SpecialType.System_Int32=>"int32",SpecialType.System_UInt32=>"uint32",SpecialType.System_Int64=>"int64",SpecialType.System_UInt64=>"uint64",SpecialType.System_Char=>"char",_=>""};
    private static string Prefix(ITypeSymbol? t)=>t?.SpecialType==SpecialType.System_Decimal?"decimal.":"floating."+Name(t)+".";
    private static void Analyze(CSharpCompilation compilation,List<PracticalNumericStep> steps,bool nullableOperations)
    {
        var seen=new HashSet<string>(StringComparer.Ordinal);
        foreach(var tree in compilation.SyntaxTrees) {
            var model=compilation.GetSemanticModel(tree);
            foreach(var syntax in tree.GetRoot().DescendantNodes().OfType<ExpressionSyntax>()) {
                var first=model.GetOperation(syntax);if(first is null){continue;}
                var pending=new Stack<IOperation>();pending.Push(first);
                while(pending.Count!=0) {
                    var operation=pending.Pop();foreach(var child in operation.ChildOperations){pending.Push(child);}
                    string site=PracticalStrings.Site(operation.Syntax);
                    string key=site+"/"+operation.Kind+"/"+operation.Type?.ToDisplayString();
                    if(!seen.Add(key)){continue;}
                    var operands=new List<IOperation>();var exceptions=new List<string>();string id="",rounding="";
                    if(nullableOperations && (operation is IBinaryOperation {IsLifted:true} || operation is IUnaryOperation {IsLifted:true}
                        || operation is IConversionOperation cv && (cv.Type is INamedTypeSymbol nt && nt.OriginalDefinition.SpecialType==SpecialType.System_Nullable_T || cv.Operand.Type is INamedTypeSymbol nf && nf.OriginalDefinition.SpecialType==SpecialType.System_Nullable_T))) {continue;}
                    switch(operation) {
                        case IConversionOperation convert when Numeric(convert.Type)||Numeric(convert.Operand.Type):
                            if(SymbolEqualityComparer.Default.Equals(convert.Type,convert.Operand.Type)){continue;}
                            if(convert.OperatorMethod is not null&&convert.OperatorMethod.ContainingType.SpecialType!=SpecialType.System_Decimal){throw PracticalFailures.Type("numeric_conversion");}
                            string from=Name(convert.Operand.Type),to=Name(convert.Type);
                            if((from=="decimal"&&Integral(convert.Type))||(to=="decimal"&&Integral(convert.Operand.Type))) {id="decimal.conversion."+from+"_to_"+to;if(from=="decimal"){exceptions.Add("System.OverflowException");}}
                            else if((from,to) is ("int32","single") or ("int64","double") or ("single","double") or ("double","single")){id="numeric.conversion."+from+"_to_"+to;}
                            else if(convert.IsChecked&&(from,to) is ("single","int32") or ("double","int64")){id="numeric.conversion."+from+"_to_"+to+".checked";exceptions.Add("System.OverflowException");}
                            else if(convert.IsImplicit&&convert.ConstantValue.HasValue&&Integral(convert.Operand.Type)&&Numeric(convert.Type)){id=Prefix(convert.Type)+"literal";}
                            else {throw PracticalFailures.Type("numeric_conversion");}
                            operands.Add(id.EndsWith(".literal",StringComparison.Ordinal)?convert:convert.Operand);break;
                        case IBinaryOperation binary when Numeric(binary.LeftOperand.Type)||Numeric(binary.RightOperand.Type):
                            if(!SymbolEqualityComparer.Default.Equals(binary.LeftOperand.Type,binary.RightOperand.Type)){throw PracticalFailures.Type("numeric_operands");}
                            string op=binary.OperatorKind switch {BinaryOperatorKind.Add=>"add",BinaryOperatorKind.Subtract=>"subtract",BinaryOperatorKind.Multiply=>"multiply",BinaryOperatorKind.Divide=>"divide",BinaryOperatorKind.Remainder=>"remainder",BinaryOperatorKind.Equals=>"equal",BinaryOperatorKind.NotEquals=>"not_equal",BinaryOperatorKind.LessThan=>"less",BinaryOperatorKind.LessThanOrEqual=>"less_equal",BinaryOperatorKind.GreaterThan=>"greater",BinaryOperatorKind.GreaterThanOrEqual=>"greater_equal",_=>throw PracticalFailures.Type("numeric_binary")};
                            id=Prefix(binary.LeftOperand.Type)+op;operands.Add(binary.LeftOperand);operands.Add(binary.RightOperand);
                            if(binary.LeftOperand.Type?.SpecialType==SpecialType.System_Decimal) {if(op is "divide" or "remainder"){exceptions.Add("System.DivideByZeroException");}if(op is "add" or "subtract" or "multiply" or "divide"){exceptions.Add("System.OverflowException");}}break;
                        case IUnaryOperation unary when Numeric(unary.Operand.Type):
                            id=Prefix(unary.Operand.Type)+(unary.OperatorKind switch {UnaryOperatorKind.Plus=>"plus",UnaryOperatorKind.Minus=>"negate",_=>throw PracticalFailures.Type("numeric_unary")});operands.Add(unary.Operand);break;
                        case ICompoundAssignmentOperation compound when Numeric(compound.Type):
                            // Compound storage/evaluation is a T04 control normalization;
                            // do not silently omit a numeric operation in this handoff.
                            throw PracticalFailures.Type("numeric_compound_assignment");
                        case IIncrementOrDecrementOperation increment when Numeric(increment.Type):
                            throw PracticalFailures.Type("numeric_increment");
                        case IInvocationOperation call when Numeric(call.TargetMethod.ContainingType)||call.TargetMethod.ContainingType.ToDisplayString() is "System.Math" or "System.MathF":
                            string name=call.TargetMethod.Name;
                            ITypeSymbol scalar=call.Arguments[0].Value.Type!;
                            id=Prefix(scalar)+(name switch {"IsNaN"=>"is_nan","IsInfinity"=>"is_infinity","IsFinite"=>"is_finite","Abs"=>"abs","Min"=>"min","Max"=>"max","Round"=>"round","Truncate"=>"truncate","Floor"=>"floor","Ceiling"=>"ceiling",_=>throw PracticalFailures.Type("numeric_intrinsic")});
                            foreach(var arg in call.Arguments) {if(arg.Parameter!.Type.Name=="MidpointRounding") {if(arg.Value is not IFieldReferenceOperation field){throw PracticalFailures.Type("numeric_rounding");}rounding=field.Field.Name;}else{operands.Add(arg.Value);}}
                            if(name=="Round"){if(rounding.Length==0){rounding="ToEven";}if(operands.Count==2){exceptions.Add("System.ArgumentOutOfRangeException");}}break;
                        case ILiteralOperation literal when Numeric(literal.Type):id=Prefix(literal.Type)+"literal";operands.Add(literal);break;
                        default:continue;
                    }
                    steps.Add(new(site,id,Array.AsReadOnly(operands.ToArray()),Array.AsReadOnly(operands.Select(o=>PracticalExactTypeNormalizer.Normalize(o.Type!,compilation).Id).ToArray()),PracticalExactTypeNormalizer.Normalize(operation.Type!,compilation).Id,rounding,Array.AsReadOnly(exceptions.ToArray())));
                }
            }
        }
    }
}
