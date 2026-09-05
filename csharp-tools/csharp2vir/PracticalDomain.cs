using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Globalization;
using System.Linq;
using System.Text.Json;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Operations;
namespace Mpk.CSharp2Vir;

internal sealed record PracticalDomainStep(string Site,string Operation,IReadOnlyList<IOperation> Operands,
    string ResultType,string Evaluation,IReadOnlyList<string> Exceptions,bool Checked=false);
internal sealed record PracticalDomainObligation(string Site,string Kind,string TypeId,string Member="",bool Discharged=false);
// Typed selection over the frozen semantic-binding roles, not a new sidecar schema.
internal sealed record PracticalOutcomeBinding(string SourceTypeId,string Role,IReadOnlyDictionary<string,string> Members,
    IReadOnlyDictionary<string,string> Tags,IReadOnlyDictionary<string,string> Operations);
internal sealed record PracticalOutcomeProjection(string SourceTypeId,string SourceSha256,string SemanticTypeId,string Role,
    IReadOnlyList<string> Arguments,IReadOnlyDictionary<string,string> Members,IReadOnlyDictionary<string,string> Tags,IReadOnlyDictionary<string,string> Operations,bool DefaultEligible);
internal sealed record PracticalDomain(PracticalNumeric Numeric,IReadOnlyList<PracticalDomainStep> Steps,
    IReadOnlyList<PracticalOutcomeProjection> Projections,IReadOnlyList<PracticalDomainObligation> Obligations)
{
    internal int ArtifactCount=>0;
    internal byte[] CopyCanonicalBytes()=>JsonSerializer.SerializeToUtf8Bytes(new {
        source=Convert.ToBase64String(Numeric.CopyCanonicalBytes()),projections=Projections,obligations=Obligations,
        steps=Steps.Select(s=>new{s.Site,s.Operation,s.ResultType,s.Evaluation,s.Exceptions,s.Checked,
            operands=s.Operands.Select(o=>new{site=PracticalStrings.Site(o.Syntax),kind=o.Kind.ToString(),type=o.Type?.ToDisplayString()})})});
}
internal static class CSharpPracticalDomain
{
    private static readonly string[] BasicObligations={"source_invariant_implies_projection","semantic_invariant_implies_reconstruction","source_round_trip","semantic_round_trip","distinct_arms","public_invariant","identity_unobservable"};
    internal static PracticalDomain Validate(PracticalSourceSelection selection,IEnumerable<PracticalCapturedInput> inputs,
        ImmutableArray<MetadataReference> references,IReadOnlyList<PracticalOutcomeBinding>? bindings=null)
    {
        var steps=new List<PracticalDomainStep>();var obligations=new List<PracticalDomainObligation>();
        CSharpCompilation? compilation=null;
        var numeric=CSharpPracticalNumeric.Validate(selection,inputs,references,c=>{compilation=c;Analyze(c,steps,obligations);});
        var projections=Bind(numeric,compilation!,bindings??Array.Empty<PracticalOutcomeBinding>(),obligations);
        foreach(var step in steps.Where(s=>s.Operation=="nullable.value_or_default")) {
            var payload=Payload(step.Operands[0].Type)!;
            var data=numeric.Strings.Arrays.Construction.Data.Types.SingleOrDefault(t=>t.Id==PracticalExactTypeNormalizer.Normalize(payload,compilation!).Id);
            if(data is not null&&(!data.DefaultEligible||projections.Any(p=>p.SourceTypeId==data.Id&&!p.DefaultEligible))){throw PracticalFailures.Type("nullable_default_ineligible");}
        }
        return new(numeric,Array.AsReadOnly(steps.OrderBy(s=>s.Site,StringComparer.Ordinal).ThenBy(s=>s.Operation,StringComparer.Ordinal).ToArray()),projections,
            Array.AsReadOnly(obligations.Distinct().OrderBy(o=>o.Site,StringComparer.Ordinal).ThenBy(o=>o.Kind,StringComparer.Ordinal).ThenBy(o=>o.Member,StringComparer.Ordinal).ToArray()));
    }
    internal static void ValidateCandidate(PracticalDomain regenerated,ReadOnlySpan<byte> candidate)
    {if(!candidate.SequenceEqual(regenerated.CopyCanonicalBytes())){throw PracticalFailures.Type("domain_handoff_mismatch");}}
    private static ITypeSymbol? Payload(ITypeSymbol? t)=>t is INamedTypeSymbol n&&n.OriginalDefinition.SpecialType==SpecialType.System_Nullable_T?n.TypeArguments[0]:null;
    private static string ValueId(ITypeSymbol t,CSharpCompilation c){var n=PracticalExactTypeNormalizer.Normalize(t,c);return t.IsReferenceType?PracticalIdentity.ClosedInstanceId("option",n.Id):n.Id;}
    private static bool Same(ITypeSymbol? a,ITypeSymbol? b)=>SymbolEqualityComparer.Default.Equals(a,b);
    private static IEnumerable<IOperation> Walk(IOperation root){var stack=new Stack<IOperation>();stack.Push(root);while(stack.Count>0){var next=stack.Pop();yield return next;foreach(var child in next.ChildOperations.Reverse()){stack.Push(child);}}}
    private static void Analyze(CSharpCompilation c,List<PracticalDomainStep> steps,List<PracticalDomainObligation> obligations)
    {
        var seen=new HashSet<string>(StringComparer.Ordinal);
        foreach(var tree in c.SyntaxTrees){var model=c.GetSemanticModel(tree);var root=tree.GetRoot();
            foreach(var parameter in root.DescendantNodes().OfType<ParameterSyntax>()){if(model.GetDeclaredSymbol(parameter) is IParameterSymbol p&&p.Type.IsReferenceType&&p.NullableAnnotation==NullableAnnotation.NotAnnotated){obligations.Add(new(PracticalStrings.Site(parameter),"explicit_not_null_precondition",ValueId(p.Type,c)));}}
            foreach(var syntax in root.DescendantNodes().Where(n=>n is ExpressionSyntax or StatementSyntax or ArrowExpressionClauseSyntax or EqualsValueClauseSyntax)) {
                if(syntax is PostfixUnaryExpressionSyntax suppress&&suppress.IsKind(SyntaxKind.SuppressNullableWarningExpression)) {if(!Present(suppress.Operand,model)){throw PracticalFailures.Type("unproved_null_suppression");}steps.Add(new(PracticalStrings.Site(syntax),"reference.suppression_identity",Array.AsReadOnly(new[]{model.GetOperation(suppress.Operand)!}),ValueId(model.GetTypeInfo(suppress.Operand).Type!,c),"receiver_once",Array.Empty<string>()));}
                if(model.GetOperation(syntax) is not IOperation first){continue;}
                foreach(var op in Walk(first)){string site=PracticalStrings.Site(op.Syntax);if(!seen.Add(site+"/"+op.Kind+"/"+op.Type?.ToDisplayString())){continue;}var operands=new List<IOperation>();var exceptions=new List<string>();string id="",evaluation="left_to_right_once";bool isChecked=false;
                    if(op.Type?.IsReferenceType==true&&op.Syntax.Parent is ReturnStatementSyntax or ArrowExpressionClauseSyntax&&model.GetEnclosingSymbol(op.Syntax.SpanStart) is IMethodSymbol method&&method.ReturnNullableAnnotation==NullableAnnotation.NotAnnotated){obligations.Add(new(site,"non_null_normal_result",ValueId(op.Type,c)));}
                    switch(op){
                        case IThrowOperation thrown:
                            IOperation? exception=thrown.Exception;
                            while(exception is IConversionOperation {IsImplicit:true} cast){exception=cast.Operand;}
                            if(exception is not IObjectCreationOperation exceptionCreation||exceptionCreation.Constructor is null){continue;}
                            string exceptionName=exceptionCreation.Constructor.ContainingType.ToDisplayString(SymbolDisplayFormat.CSharpErrorMessageFormat);
                            if(exceptionCreation.Constructor.ContainingType.DeclaringSyntaxReferences.Length!=0||exceptionCreation.Constructor.ContainingAssembly.Name!="System.Runtime"||exceptionName is not ("System.InvalidOperationException" or "System.ArgumentException")){continue;}
                            id="outcome.source_throw";exceptions.Add(exceptionName);break;
                        case IConversionOperation nullReference when nullReference.Type?.IsReferenceType==true
                            &&nullReference.Type.SpecialType!=SpecialType.System_Object
                            &&nullReference.Operand.ConstantValue.HasValue&&nullReference.Operand.ConstantValue.Value is null:
                            id="reference.none";break;
                        case IDefaultValueOperation referenceDefault when referenceDefault.Type?.IsReferenceType==true:
                            id="reference.none";break;
                        case IObjectCreationOperation creation when Payload(creation.Type) is not null:throw PracticalFailures.Type("nullable_construction");
                        case IDefaultValueOperation d when Payload(d.Type) is not null:
                            if(d.Syntax is not DefaultExpressionSyntax {Type:NullableTypeSyntax}){throw PracticalFailures.Type("nullable_construction");}id="nullable.none";break;
                        case IConversionOperation cv when Payload(cv.Type) is ITypeSymbol payload:
                            if(Payload(cv.Operand.Type) is not null){if(!Same(cv.Type,cv.Operand.Type)){throw PracticalFailures.Type("nullable_conversion");}continue;}
                            if(!cv.IsImplicit||cv.OperatorMethod is not null){throw PracticalFailures.Type("nullable_construction");}
                            if(cv.Operand.ConstantValue.HasValue&&cv.Operand.ConstantValue.Value is null){id="nullable.none";}
                            else if(Same(payload,cv.Operand.Type)){id="nullable.some";operands.Add(cv.Operand);obligations.Add(new(site,"payload_public_invariant",ValueId(payload,c)));}
                            else{throw PracticalFailures.Type("nullable_conversion");}break;
                        case IConversionOperation cv when Payload(cv.Operand.Type) is not null:throw PracticalFailures.Type("nullable_conversion");
                        case IPropertyReferenceOperation property when Payload(property.Instance?.Type) is ITypeSymbol payload:
                            id=property.Property.Name switch{"HasValue"=>"nullable.has_value","Value"=>"nullable.value",_=>throw PracticalFailures.Type("nullable_member")};operands.Add(property.Instance!);if(id=="nullable.value"){exceptions.Add("System.InvalidOperationException");}break;
                        case IInvocationOperation call when Payload(call.Instance?.Type) is ITypeSymbol payload:
                            if(call.TargetMethod.Name!="GetValueOrDefault"){throw PracticalFailures.Type("nullable_member");}id=call.Arguments.Length==0?"nullable.value_or_default":"nullable.value_or";operands.Add(call.Instance!);operands.AddRange(call.Arguments.Select(a=>a.Value));
                            if(call.Arguments.Length==0){if(!DefaultEligible(payload,c,new HashSet<ITypeSymbol>(SymbolEqualityComparer.Default))){throw PracticalFailures.Type("nullable_default_ineligible");}}
                            else if(call.Arguments.Length!=1||!Same(payload,call.Arguments[0].Value.Type)||call.Arguments[0].Value is IConversionOperation {Conversion.IsIdentity:false}){throw PracticalFailures.Type("nullable_fallback_type");}
                            else{obligations.Add(new(site,"fallback_public_invariant",ValueId(payload,c)));}break;
                        case ICoalesceOperation coalesce:
                            var target=Payload(coalesce.Value.Type)??(coalesce.Value.Type?.IsReferenceType==true?coalesce.Value.Type:null);
                            if(target is null||!Same(target,coalesce.WhenNull.Type)||!Same(target,coalesce.Type)||!coalesce.ValueConversion.IsIdentity||coalesce.WhenNull is IThrowOperation or IConversionOperation {Conversion.IsIdentity:false}||coalesce.WhenNull.Type!.IsReferenceType&&coalesce.WhenNull.Type.NullableAnnotation!=NullableAnnotation.NotAnnotated){throw PracticalFailures.Type("nullable_coalesce");}
                            id="nullable.coalesce";evaluation="left_once_rhs_only_if_absent";operands.Add(coalesce.Value);operands.Add(coalesce.WhenNull);obligations.Add(new(site,"fallback_public_invariant",ValueId(target,c)));break;
                        case ICoalesceAssignmentOperation:throw PracticalFailures.Type("nullable_coalesce_assignment");
                        case IConditionalAccessOperation access:
                            if(access.Operation.Type is not INamedTypeSymbol receiver||receiver.TypeKind!=TypeKind.Class||receiver.DeclaringSyntaxReferences.IsEmpty||receiver.NullableAnnotation!=NullableAnnotation.Annotated||access.WhenNotNull.Type?.IsReferenceType!=true||access.WhenNotNull.Type.NullableAnnotation!=NullableAnnotation.NotAnnotated||Walk(access.Operation).Any(o=>o is IConditionalAccessOperation)||Walk(access.WhenNotNull).Any(o=>o is IConditionalAccessOperation or IInvocationOperation or IArrayElementReferenceOperation)){throw PracticalFailures.Type("nullable_conditional_access");}
                            if(access.WhenNotNull is IFieldReferenceOperation field&&field.Instance is IConditionalAccessInstanceOperation&&field.Field.DeclaringSyntaxReferences.Length>0){}
                            else if(access.WhenNotNull is IPropertyReferenceOperation prop&&!prop.Property.IsIndexer&&prop.Instance is IConditionalAccessInstanceOperation&&TotalGetter(prop.Property,c)){}
                            else{throw PracticalFailures.Type("nullable_conditional_member");}
                            id="reference.conditional_access";evaluation="receiver_once_member_only_if_present";operands.Add(access.Operation);operands.Add(access.WhenNotNull);break;
                        case IBinaryOperation binary when binary.IsLifted:
                            string bop=binary.OperatorKind switch {BinaryOperatorKind.Add=>"add",BinaryOperatorKind.Subtract=>"subtract",BinaryOperatorKind.Multiply=>"multiply",BinaryOperatorKind.Divide=>"divide",BinaryOperatorKind.Remainder=>"remainder",BinaryOperatorKind.Equals=>"equal",BinaryOperatorKind.NotEquals=>"not_equal",BinaryOperatorKind.LessThan=>"less",BinaryOperatorKind.LessThanOrEqual=>"less_equal",BinaryOperatorKind.GreaterThan=>"greater",BinaryOperatorKind.GreaterThanOrEqual=>"greater_equal",BinaryOperatorKind.And=>"and",BinaryOperatorKind.Or=>"or",_=>throw PracticalFailures.Type("nullable_lift")};
                            if(!Same(binary.LeftOperand.Type,binary.RightOperand.Type)){throw PracticalFailures.Type("nullable_lift");}id=Lift(Payload(binary.LeftOperand.Type),bop,binary.IsChecked,exceptions);operands.Add(binary.LeftOperand);operands.Add(binary.RightOperand);isChecked=binary.IsChecked;evaluation="both_operands_before_presence_test";break;
                        case IUnaryOperation unary when unary.IsLifted:
                            id=Lift(Payload(unary.Operand.Type),unary.OperatorKind switch{UnaryOperatorKind.Plus=>"plus",UnaryOperatorKind.Minus=>"negate",UnaryOperatorKind.Not=>"not",_=>throw PracticalFailures.Type("nullable_lift")},unary.IsChecked,exceptions);operands.Add(unary.Operand);isChecked=unary.IsChecked;break;
                        case IBinaryOperation binary when binary.OperatorKind is BinaryOperatorKind.Equals or BinaryOperatorKind.NotEquals&&binary.LeftOperand.Type?.IsReferenceType==true&&binary.RightOperand.ConstantValue.HasValue&&binary.RightOperand.ConstantValue.Value is null:
                            id=binary.OperatorKind==BinaryOperatorKind.Equals?"reference.is_null":"reference.is_not_null";operands.Add(binary.LeftOperand);break;
                        case IBinaryOperation binary when binary.OperatorKind is BinaryOperatorKind.Equals or BinaryOperatorKind.NotEquals&&binary.RightOperand.Type?.IsReferenceType==true&&binary.LeftOperand.ConstantValue.HasValue&&binary.LeftOperand.ConstantValue.Value is null:
                            id=binary.OperatorKind==BinaryOperatorKind.Equals?"reference.is_null":"reference.is_not_null";operands.Add(binary.RightOperand);break;
                        case IIsPatternOperation pattern when pattern.Value.Type?.IsReferenceType==true:
                            bool negated=pattern.Pattern is INegatedPatternOperation;
                            var nullPattern=negated?((INegatedPatternOperation)pattern.Pattern).Pattern:pattern.Pattern;
                            if(nullPattern is not IConstantPatternOperation {Value.ConstantValue.HasValue:true,Value.ConstantValue.Value:null}){continue;}
                            id=negated?"reference.is_not_null":"reference.is_null";operands.Add(pattern.Value);break;
                        case IFieldReferenceOperation readField when readField.Instance?.Type?.IsReferenceType==true&&readField.Instance is not IConditionalAccessInstanceOperation:
                            id="reference.field_read";operands.Add(readField.Instance);exceptions.Add("System.NullReferenceException");break;
                        default:continue;
                    }
                    steps.Add(new(site,id,Array.AsReadOnly(operands.ToArray()),op.Type is null?"":ValueId(op.Type,c),evaluation,Array.AsReadOnly(exceptions.ToArray()),isChecked));
                }
            }
        }
    }
    private static string Lift(ITypeSymbol? payload,string op,bool check,List<string> exceptions){string token=payload?.SpecialType switch {SpecialType.System_Int32=>"i32",SpecialType.System_Int64=>"i64",SpecialType.System_Single=>"f32",SpecialType.System_Double=>"f64",SpecialType.System_Decimal=>"decimal",SpecialType.System_Boolean=>"bool",_=>throw PracticalFailures.Type("nullable_lift")};if(token=="bool"?!new[]{"and","or","not","equal","not_equal"}.Contains(op):new[]{"and","or","not"}.Contains(op)){throw PracticalFailures.Type("nullable_lift");}if(token is "i32" or "i64" or "decimal"){if(op is "divide" or "remainder"){exceptions.Add("System.DivideByZeroException");}if(((check||token=="decimal")&&op is "add" or "subtract" or "multiply"||check&&token!="decimal"&&op=="negate")||op=="divide"||token!="decimal"&&op=="remainder"){exceptions.Add("System.OverflowException");}}return "lifted."+token+"."+op;}
    private static bool DefaultEligible(ITypeSymbol t,CSharpCompilation c,HashSet<ITypeSymbol> active){if(Payload(t) is not null){return true;}if(t.IsReferenceType){return t.NullableAnnotation==NullableAnnotation.Annotated;}if(t.TypeKind==TypeKind.Enum){return t.GetMembers().OfType<IFieldSymbol>().Any(f=>f.HasConstantValue&&Convert.ToString(f.ConstantValue,CultureInfo.InvariantCulture)=="0");}if(t.SpecialType!=SpecialType.None){return true;}if(!active.Add(t)){return false;}bool result=t.GetMembers().OfType<IFieldSymbol>().Where(f=>!f.IsStatic).All(f=>DefaultEligible(f.Type,c,active));active.Remove(t);return result;}
    private static bool TotalGetter(IPropertySymbol p,CSharpCompilation c)
    {
        if(p.GetMethod is null||p.DeclaringSyntaxReferences.Length==0){return false;}
        if(p.DeclaringSyntaxReferences[0].GetSyntax() is not PropertyDeclarationSyntax syntax){return false;}
        var getter=syntax.AccessorList?.Accessors.SingleOrDefault(a=>a.IsKind(SyntaxKind.GetAccessorDeclaration));
        if(syntax.ExpressionBody is null&&getter?.Body is null&&getter?.ExpressionBody is null){return getter is not null;}
        ExpressionSyntax? expression=syntax.ExpressionBody?.Expression??getter?.ExpressionBody?.Expression;
        if(expression is null&&getter?.Body?.Statements is {Count:1} statements&&statements[0] is ReturnStatementSyntax ret){expression=ret.Expression;}
        return expression is not null&&c.GetSemanticModel(syntax.SyntaxTree).GetOperation(expression) is ILiteralOperation or IFieldReferenceOperation {Instance:IInstanceReferenceOperation};
    }
    // Independent, deliberately local presence proof. Only stable identifiers
    // guarded in the containing branch are accepted; annotations never suffice.
    private static bool Present(ExpressionSyntax value,SemanticModel model){if(value is LiteralExpressionSyntax literal&&!literal.IsKind(SyntaxKind.NullLiteralExpression)){return true;}if(value is not IdentifierNameSyntax name){return false;}var symbol=model.GetSymbolInfo(name).Symbol;if(symbol is not ILocalSymbol and not IParameterSymbol){return false;}foreach(var branch in value.Ancestors().OfType<IfStatementSyntax>()){if(!branch.Statement.Span.Contains(value.Span)){continue;}bool guard=false;if(branch.Condition is BinaryExpressionSyntax binary&&binary.IsKind(SyntaxKind.NotEqualsExpression)&&binary.Right.IsKind(SyntaxKind.NullLiteralExpression)&&SameSymbol(binary.Left)) {guard=true;}if(branch.Condition is IsPatternExpressionSyntax pattern&&pattern.Pattern is UnaryPatternSyntax {Pattern:ConstantPatternSyntax cp}&&cp.Expression.IsKind(SyntaxKind.NullLiteralExpression)&&SameSymbol(pattern.Expression)){guard=true;}if(branch.Condition is MemberAccessExpressionSyntax member&&member.Name.Identifier.ValueText=="HasValue"&&SameSymbol(member.Expression)){guard=true;}if(guard&&!branch.Statement.DescendantNodes().Any(n=>n.SpanStart<value.SpanStart&&(n is AssignmentExpressionSyntax a&&SameSymbol(a.Left)||n is PrefixUnaryExpressionSyntax pre&&(pre.IsKind(SyntaxKind.PreIncrementExpression)||pre.IsKind(SyntaxKind.PreDecrementExpression))&&SameSymbol(pre.Operand)||n is PostfixUnaryExpressionSyntax post&&(post.IsKind(SyntaxKind.PostIncrementExpression)||post.IsKind(SyntaxKind.PostDecrementExpression))&&SameSymbol(post.Operand)))){return true;}}return false;bool SameSymbol(ExpressionSyntax e)=>SymbolEqualityComparer.Default.Equals(model.GetSymbolInfo(e).Symbol,symbol);}

    private static IReadOnlyList<PracticalOutcomeProjection> Bind(PracticalNumeric numeric,CSharpCompilation compilation,
        IReadOnlyList<PracticalOutcomeBinding> bindings,List<PracticalDomainObligation> obligations)
    {
        var data=numeric.Strings.Arrays.Construction.Data;
        var output=new Dictionary<string,PracticalOutcomeProjection>(StringComparer.Ordinal);
        if(bindings.Select(b=>b.SourceTypeId).Distinct(StringComparer.Ordinal).Count()!=bindings.Count){throw PracticalFailures.Type("outcome_duplicate_binding");}
        var pending=bindings.ToDictionary(b=>b.SourceTypeId,StringComparer.Ordinal);
        var active=new HashSet<string>(StringComparer.Ordinal);
        foreach(var id in pending.Keys.OrderBy(s=>s,StringComparer.Ordinal)){Resolve(id);}
        foreach(var type in data.Types)foreach(var field in type.Members.Where(m=>m.Stored&&m.Type.Nullability=="not_annotated"))
            {obligations.Add(new(type.Id,"non_null_stored_field",field.Type.Id,field.Name));}
        return Array.AsReadOnly(output.Values.OrderBy(p=>p.SourceTypeId,StringComparer.Ordinal).ToArray());
        string Value(PracticalNormalizedType type,int depth,string parent)
        {
            if(depth>16){throw PracticalFailures.Type("outcome_nesting");}
            string id=type.Id;
            if(pending.ContainsKey(id)){id=Resolve(id).SemanticTypeId;}
            if(type.Nullability=="annotated"){id=PracticalIdentity.ClosedInstanceId("option",id);}
            // The native closed-instance derivation independently checks every
            // transitive argument, including the single lookup<option<T>> form.
            bool option=type.Arguments.Count>0&&type.Id==PracticalIdentity.ClosedInstanceId("option",type.Arguments[0].Id)
                ||type.Nullability=="annotated"||output.Values.Any(p=>p.SemanticTypeId==id&&p.Role=="option");
            if(parent=="option"&&option){throw PracticalFailures.Type("nested_option");}
            return id;
        }
        PracticalOutcomeProjection Resolve(string id)
        {
            if(output.TryGetValue(id,out var existing)){return existing;}
            if(!active.Add(id)||active.Count>16){throw PracticalFailures.Type("outcome_nesting");}
            var binding=pending[id];var type=data.Types.SingleOrDefault(t=>t.Id==id);
            if(type is null||type.Kind=="enum"){throw PracticalFailures.Type("outcome_source_type");}
            string[] arms=binding.Role switch {"option"=>new[]{"none","some"},"lookup"=>new[]{"missing_key","found"},"result"=>new[]{"ok","error"},"validation"=>new[]{"valid","invalid"},"boundary_field"=>new[]{"missing","null","value"},_=>throw PracticalFailures.Type("outcome_role")};
            string[] roles=binding.Role switch {"result"=>new[]{"tag","value","error"},"validation"=>new[]{"tag","value","errors"},_=>new[]{"tag","value"}};
            if(!binding.Members.Keys.OrderBy(s=>s,StringComparer.Ordinal).SequenceEqual(roles.OrderBy(s=>s,StringComparer.Ordinal))||binding.Members.Values.Distinct(StringComparer.Ordinal).Count()!=roles.Length){throw PracticalFailures.Type("outcome_members");}
            PracticalDataMember Member(string role)=>type.Members.SingleOrDefault(m=>m.Stored&&m.Name==binding.Members[role])??throw PracticalFailures.Type("outcome_member");
            var tag=data.Types.SingleOrDefault(t=>t.Id==Member("tag").Type.Id&&t.Kind=="enum");
            if(tag is null||!binding.Tags.Keys.OrderBy(s=>s,StringComparer.Ordinal).SequenceEqual(arms.OrderBy(s=>s,StringComparer.Ordinal))||binding.Tags.Values.Distinct(StringComparer.Ordinal).Count()!=arms.Length||!binding.Tags.Values.OrderBy(s=>s,StringComparer.Ordinal).SequenceEqual(tag.EnumMembers.Select(m=>m.Value).Distinct(StringComparer.Ordinal).OrderBy(s=>s,StringComparer.Ordinal))){throw PracticalFailures.Type("outcome_tags");}
            var arguments=new List<string>{Value(Member("value").Type,active.Count,binding.Role)};
            if(binding.Role=="result"){arguments.Add(Value(Member("error").Type,active.Count,binding.Role));}
            if(binding.Role=="validation") {
                var errors=Member("errors").Type;
                if(errors.Arguments.Count!=1||errors.Id!=PracticalIdentity.ClosedInstanceId("bounded_sequence",errors.Arguments[0].Id)){throw PracticalFailures.Type("outcome_errors");}
                arguments.Add(Value(errors.Arguments[0],active.Count,binding.Role));
                obligations.Add(new(id,"invalid_errors_1_through_256",arguments[1],binding.Members["errors"]));
                obligations.Add(new(id,"errors_left_before_right_preserve_duplicates",arguments[1],binding.Members["errors"]));
            }
            string semantic=PracticalIdentity.ClosedInstanceId(binding.Role,arguments.ToArray());
            foreach(string kind in BasicObligations){obligations.Add(new(id,kind,semantic));}
            foreach(var member in type.Members.Where(m=>m.Stored)){obligations.Add(new(id,"field_complete_reconstruction",semantic,member.Name));}
            bool eligible=false;
            if(binding.Role is "option" or "lookup") {
                obligations.Add(new(id,"actual_default_public_invariant",semantic));
                eligible=type.Kind=="readonly_struct"&&type.DefaultEligible&&binding.Tags[binding.Role=="option"?"none":"missing_key"]=="0";
            }
            foreach(var operation in binding.Operations.OrderBy(p=>p.Key,StringComparer.Ordinal)) {
                string[] allowed=binding.Role switch {
                    "option"=>new[]{"none","some","has_value","value","value_or","equal","compare"},
                    "lookup"=>new[]{"missing","found","is_found","value","equal","compare"},
                    "result"=>new[]{"ok","error","is_ok","value","error_value","equal","compare"},
                    "validation"=>new[]{"valid","invalid","is_valid","value","errors","append_errors","equal","compare"},
                    _=>new[]{"missing","null","value","tag","payload","equal","compare"}};
                if(!allowed.Contains(operation.Key,StringComparer.Ordinal)){throw PracticalFailures.Type("outcome_operation_name");}
                var callable=data.Syntax.Callables.SingleOrDefault(f=>f.Id==operation.Value);
                if(callable is null){throw PracticalFailures.Type("outcome_operation_identity");}
                // A captured source body is retained for all three commuting
                // relations. A raw stored payload never becomes a throwing getter.
                foreach(string kind in new[]{"operation_normal_commutation","operation_error_commutation","operation_exception_commutation"}){obligations.Add(new(id,kind,semantic,operation.Value));}
                if(operation.Key is "value" or "error_value" or "errors") {
                    obligations.Add(new(id,"inactive_payload_exact_source_exception_or_active_precondition",semantic,operation.Value));
                }
            }
            var closure=data.Syntax.SourceClosure;var declaration=closure.Declarations.Single(d=>d.Kind==PracticalDeclarationKind.Type&&d.Id==id);
            var projection=new PracticalOutcomeProjection(id,closure.Sources[declaration.SourceOrdinal].RawSha256,semantic,binding.Role,Array.AsReadOnly(arguments.ToArray()),
                new System.Collections.ObjectModel.ReadOnlyDictionary<string,string>(new SortedDictionary<string,string>(binding.Members.ToDictionary(p=>p.Key,p=>p.Value),StringComparer.Ordinal)),
                new System.Collections.ObjectModel.ReadOnlyDictionary<string,string>(new SortedDictionary<string,string>(binding.Tags.ToDictionary(p=>p.Key,p=>p.Value),StringComparer.Ordinal)),
                new System.Collections.ObjectModel.ReadOnlyDictionary<string,string>(new SortedDictionary<string,string>(binding.Operations.ToDictionary(p=>p.Key,p=>p.Value),StringComparer.Ordinal)),eligible);
            active.Remove(id);output.Add(id,projection);return projection;
        }
    }
}
