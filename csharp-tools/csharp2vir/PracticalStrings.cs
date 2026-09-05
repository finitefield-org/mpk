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

// Per-expression inventory; captured bodies determine control/execution order.
internal sealed record PracticalStringStep(string Site, string Operation, string Relation,
    IReadOnlyList<IOperation> Operands, IReadOnlyList<int> ArgumentOrdinals,
    IReadOnlyList<string> Checks, IReadOnlyList<string> Exceptions, string ResultTypeId);
internal sealed record PracticalStringObligation(string Site,string Kind,bool Discharged=false);
internal sealed record PracticalStrings(PracticalArrays Arrays, IReadOnlyList<PracticalStringStep> Steps, IReadOnlyList<PracticalStringObligation> Obligations)
{
    internal int ArtifactCount=>0;
    internal byte[] CopyCanonicalBytes()=>JsonSerializer.SerializeToUtf8Bytes(new {
        source=Arrays.Construction.Data.Syntax.SemanticSha256, obligations=Obligations,
        steps=Steps.Select(s=>new {s.Site,s.Operation,s.Relation,s.ArgumentOrdinals,s.Checks,s.Exceptions,s.ResultTypeId,
            operands=s.Operands.Select(o=>new {site=Site(o.Syntax),kind=o.Kind.ToString(),
                type=o.Type?.SpecialType.ToString(),constant=o.ConstantValue.HasValue&&o.ConstantValue.Value is string text
                    ? string.Concat(text.Select(c=>((int)c).ToString("x4",System.Globalization.CultureInfo.InvariantCulture))) : ""})}),
    });
    internal static string Site(SyntaxNode s)=>s.SyntaxTree.FilePath+":"+s.SpanStart.ToString(System.Globalization.CultureInfo.InvariantCulture)+":"+s.Span.Length.ToString(System.Globalization.CultureInfo.InvariantCulture);
}
internal static class CSharpPracticalStrings
{
    internal const int MaximumUtf16Units=16384;
    internal static PracticalStrings Validate(PracticalSourceSelection selection,IEnumerable<PracticalCapturedInput> inputs,
        ImmutableArray<MetadataReference> references,IReadOnlyList<PracticalTypeInvariantClaim>? invariantClaims=null, Action<CSharpCompilation>? validateNumeric=null)
    {
        var steps=new List<PracticalStringStep>();var obligations=new List<PracticalStringObligation>();
        var arrays=CSharpPracticalArrays.Validate(selection,inputs,references,invariantClaims,true,current=>{Analyze(current,steps,obligations);validateNumeric?.Invoke(current);});
        return new(arrays,Array.AsReadOnly(steps.OrderBy(s=>s.Site,StringComparer.Ordinal).ThenBy(s=>s.Operation,StringComparer.Ordinal).ToArray()),Array.AsReadOnly(obligations.Distinct().OrderBy(o=>o.Site,StringComparer.Ordinal).ThenBy(o=>o.Kind,StringComparer.Ordinal).ToArray()));
    }
    internal static void ValidateCandidate(PracticalStrings regenerated,ReadOnlySpan<byte> candidate)
    {if(!candidate.SequenceEqual(regenerated.CopyCanonicalBytes())){throw PracticalFailures.Type("string_handoff_mismatch");}}
    private static IOperation Exact(IOperation value)
    {while(value is IConversionOperation {IsImplicit:true} c&&c.Type?.SpecialType==SpecialType.System_Object){value=c.Operand;}return value;}
    private static bool String(IOperation o)=>Exact(o).Type?.SpecialType==SpecialType.System_String;
    private static bool Char(IOperation o)=>Exact(o).Type?.SpecialType==SpecialType.System_Char;
    private static void Analyze(CSharpCompilation compilation,List<PracticalStringStep> steps,List<PracticalStringObligation> obligations)
    {
        var seen=new HashSet<string>(StringComparer.Ordinal);
        foreach(var tree in compilation.SyntaxTrees) {
            var model=compilation.GetSemanticModel(tree);
            foreach(var parameter in tree.GetRoot().DescendantNodes().OfType<ParameterSyntax>()) {
                if(model.GetDeclaredSymbol(parameter) is IParameterSymbol p&&p.Type.SpecialType==SpecialType.System_String)
                {obligations.Add(new(PracticalStrings.Site(parameter),"input_utf16_length_le_16384"));}
            }
            foreach(var syntax in tree.GetRoot().DescendantNodes().OfType<ExpressionSyntax>()) {
                var operation=model.GetOperation(syntax);
                if(operation is null||operation.Syntax!=syntax){continue;}
                string site=PracticalStrings.Site(syntax);
                if(operation.Type?.SpecialType==SpecialType.System_String && syntax.Parent is ReturnStatementSyntax or ArrowExpressionClauseSyntax)
                {obligations.Add(new(site,"output_utf16_length_le_16384"));}
                var operands=new List<IOperation>();var ordinals=new List<int>();var checks=new List<string>();var exceptions=new List<string>();
                string id="",relation="";
                void Receiver(){checks.Add("exception.null_receiver");exceptions.Add("System.NullReferenceException");}
                void Bound(){checks.Add("obligation.output_bound");}
                if(operation.ConstantValue.HasValue&&operation.ConstantValue.Value is string constant&&constant.Length>MaximumUtf16Units)
                {throw PracticalFailures.Limit("string_utf16_units");}
                switch(operation) {
                    case IInterpolatedStringOperation interpolation:
                        foreach(var part in interpolation.Parts) {
                            if(part is IInterpolatedStringTextOperation text){operands.Add(text.Text);}
                            else if(part is IInterpolationOperation hole&&hole.Alignment is null&&hole.FormatString is null&&(String(hole.Expression)||Char(hole.Expression))) {operands.Add(Exact(hole.Expression));}
                            else {throw PracticalFailures.Type("string_interpolation");}
                        }
                        id="string.interpolation.restricted";relation="bounded_utf16_concat";Bound();break;
                    case IBinaryOperation binary when binary.OperatorKind==BinaryOperatorKind.Add:
                        if(model.GetTypeInfo(((BinaryExpressionSyntax)syntax).Left).Type?.SpecialType==SpecialType.System_Char&&model.GetTypeInfo(((BinaryExpressionSyntax)syntax).Right).Type?.SpecialType==SpecialType.System_Char){throw PracticalFailures.Type("char_char_concat");}
                        if(binary.Type?.SpecialType!=SpecialType.System_String){continue;}
                        if(binary.OperatorMethod is not null||!(String(binary.LeftOperand)&&String(binary.RightOperand)||String(binary.LeftOperand)&&Char(binary.RightOperand)||Char(binary.LeftOperand)&&String(binary.RightOperand)))
                        {throw PracticalFailures.Type("string_concat_operands");}
                        id="string.concat.operator."+(Char(binary.LeftOperand)?"char":"string")+"_"+(Char(binary.RightOperand)?"char":"string");relation="bounded_utf16_concat";
                        operands.Add(Exact(binary.LeftOperand));operands.Add(Exact(binary.RightOperand));Bound();break;
                    case IBinaryOperation binary when String(binary.LeftOperand)&&String(binary.RightOperand)&&binary.OperatorKind is BinaryOperatorKind.Equals or BinaryOperatorKind.NotEquals:
                        id=binary.OperatorKind==BinaryOperatorKind.Equals?"string.equality.operator":"string.inequality.operator";operands.Add(binary.LeftOperand);operands.Add(binary.RightOperand);break;
                    case IPropertyReferenceOperation property when property.Property.ContainingType.SpecialType==SpecialType.System_String:
                        operands.Add(property.Instance!);operands.AddRange(property.Arguments.Select(a=>a.Value));Receiver();
                        if(property.Property.Name=="Length"){id="string.length";}
                        else if(property.Property.IsIndexer){id="string.index";checks.Add("exception.range");exceptions.Add("System.IndexOutOfRangeException");}
                        else {throw PracticalFailures.Type("string_property");}break;
                    case IInvocationOperation call when call.TargetMethod.ContainingType.SpecialType==SpecialType.System_String:
                        if(call.Instance is not null){operands.Add(call.Instance);ordinals.Add(-1);}
                        foreach(var arg in call.Arguments) {
                            if(arg.Parameter!.Type.Name=="StringComparison") {
                                if(arg.Value is not IFieldReferenceOperation field||field.Field.Name!="Ordinal"||field.Field.ContainingType.Name!="StringComparison") {throw PracticalFailures.Type("string_ordinal_constant");}
                                continue;
                            }
                            operands.Add(arg.Value);ordinals.Add(arg.Parameter.Ordinal);
                        }
                        if(!call.TargetMethod.IsStatic){Receiver();}
                        id=call.TargetMethod.Name switch {"Equals"=>"string.equals.ordinal","Compare"=>"string.compare.ordinal","IsNullOrEmpty"=>"string.is_null_or_empty","Contains"=>"string.contains.ordinal","StartsWith"=>"string.starts_with.ordinal","EndsWith"=>"string.ends_with.ordinal","Substring"=>"string.substring.start_length","Concat"=>"string.concat.string"+call.Arguments.Length,_=>throw PracticalFailures.Type("string_method")};
                        if(call.TargetMethod.Name is "Contains" or "StartsWith" or "EndsWith") {checks.Add("exception.null_argument");exceptions.Add("System.ArgumentNullException");}
                        if(call.TargetMethod.Name=="Substring"){checks.Add("exception.range");exceptions.Add("System.ArgumentOutOfRangeException");Bound();}
                        if(call.TargetMethod.Name=="Concat"){relation="bounded_utf16_concat";Bound();}break;
                    case ILiteralOperation literal when String(literal)&&literal.ConstantValue.Value is string:id="string.literal.decode";operands.Add(literal);Bound();break;
                    default:continue;
                }
                // Operands remain in source evaluation order. Ordinals retain
                // formal positions for named arguments; receiver checks occur
                // only after all argument evaluations, as in captured C#.
                if(ordinals.Count==0){ordinals.AddRange(Enumerable.Range(0,operands.Count));}
                if(!seen.Add(site+"/"+id)){continue;}
                if(checks.Contains("obligation.output_bound")){obligations.Add(new(site,"output_utf16_length_le_16384"));}
                steps.Add(new(site,id,relation.Length==0?id:relation,Array.AsReadOnly(operands.ToArray()),Array.AsReadOnly(ordinals.ToArray()),
                    Array.AsReadOnly(checks.ToArray()),Array.AsReadOnly(exceptions.ToArray()),PracticalExactTypeNormalizer.Normalize(operation.Type!,compilation).Id));
            }
        }
    }
}
