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
internal sealed record PracticalBusinessStep(string Site,string Operation,IReadOnlyList<IOperation> Operands,string ResultType,IReadOnlyList<string> Exceptions);
internal sealed record PracticalBusinessBinding(string SourceTypeId,string Role,IReadOnlyDictionary<string,string> Members,IReadOnlyDictionary<string,string> Operations,IReadOnlyDictionary<string,IReadOnlyDictionary<string,string>>? EnumArms=null);
internal sealed record PracticalBusinessProjection(string SourceTypeId,string SourceSha256,string SemanticTypeId,string Role,IReadOnlyDictionary<string,string> Members,IReadOnlyDictionary<string,string> Operations,IReadOnlyDictionary<string,IReadOnlyDictionary<string,string>> EnumArms);
internal sealed record PracticalBusiness(PracticalDomain Domain,IReadOnlyList<PracticalBusinessStep> Steps,IReadOnlyList<PracticalBusinessProjection> Projections,IReadOnlyList<PracticalDomainObligation> Obligations)
{
 internal int ArtifactCount=>0;
 internal byte[] CopyCanonicalBytes()=>JsonSerializer.SerializeToUtf8Bytes(new{source=Convert.ToBase64String(Domain.CopyCanonicalBytes()),projections=Projections,obligations=Obligations,steps=Steps.Select(s=>new{s.Site,s.Operation,s.ResultType,s.Exceptions,operands=s.Operands.Select(o=>new{site=PracticalStrings.Site(o.Syntax),kind=o.Kind.ToString(),type=o.Type?.ToDisplayString()})})});
}
internal static class CSharpPracticalBusiness
{
 internal static PracticalBusiness Validate(PracticalSourceSelection selection,IEnumerable<PracticalCapturedInput> inputs,ImmutableArray<MetadataReference> references,IReadOnlyList<PracticalBusinessBinding>? bindings=null,IReadOnlyList<PracticalOutcomeBinding>? outcomes=null)
 {
  var steps=new List<PracticalBusinessStep>();CSharpCompilation? compilation=null;
  var domain=CSharpPracticalDomain.Validate(selection,inputs,references,outcomes,c=>{compilation=c;Analyze(c,steps);});
  var obligations=new List<PracticalDomainObligation>();var projections=Bind(domain,compilation!,bindings??Array.Empty<PracticalBusinessBinding>(),obligations);
  foreach(var step in domain.Steps.Where(s=>s.Operation=="nullable.value_or_default")) {
   if(projections.Any(p=>p.SourceTypeId==step.ResultType)){throw PracticalFailures.Type("business_nullable_default_ineligible");}
  }
  var money=projections.Where(p=>p.Role=="money").Select(p=>p.SourceTypeId).ToHashSet(StringComparer.Ordinal);
  if(domain.Numeric.Strings.Arrays.Construction.Obligations.Any(o=>o.Kind=="default_public_invariant"&&money.Contains(o.TypeId))){throw PracticalFailures.Type("money_default_ineligible");}
  foreach(var tree in compilation!.SyntaxTrees){var model=compilation.GetSemanticModel(tree);foreach(var syntax in tree.GetRoot().DescendantNodes().OfType<ExpressionSyntax>()) {
   if(model.GetOperation(syntax) is IOperation op && (op is IDefaultValueOperation || op is IObjectCreationOperation {Constructor.IsImplicitlyDeclared:true,Arguments.Length:0}) && op.Type is not null && money.Contains(PracticalExactTypeNormalizer.Normalize(op.Type,compilation).Id)){throw PracticalFailures.Type("money_default_ineligible");}
  }}
  return new(domain,Array.AsReadOnly(steps.OrderBy(s=>s.Site,StringComparer.Ordinal).ThenBy(s=>s.Operation,StringComparer.Ordinal).ToArray()),projections,Array.AsReadOnly(obligations.OrderBy(o=>o.Site,StringComparer.Ordinal).ThenBy(o=>o.Kind,StringComparer.Ordinal).ThenBy(o=>o.Member,StringComparer.Ordinal).ToArray()));
 }
 internal static void ValidateCandidate(PracticalBusiness regenerated,ReadOnlySpan<byte> candidate){if(!candidate.SequenceEqual(regenerated.CopyCanonicalBytes())){throw PracticalFailures.Type("business_handoff_mismatch");}}
 private static string Token(ITypeSymbol t)=>t.MetadataName switch{"DateOnly"=>"date","TimeOnly"=>"time","TimeSpan"=>"duration","Guid"=>"guid",_=>throw PracticalFailures.Type("business_type")};
 private static IEnumerable<IOperation> Walk(IOperation root){var stack=new Stack<IOperation>();stack.Push(root);while(stack.Count>0){var next=stack.Pop();yield return next;foreach(var child in next.ChildOperations.Reverse()){stack.Push(child);}}}
 private static void Analyze(CSharpCompilation c,List<PracticalBusinessStep> steps)
 {
  var seen=new HashSet<string>(StringComparer.Ordinal);
  foreach(var tree in c.SyntaxTrees){var model=c.GetSemanticModel(tree);foreach(var syntax in tree.GetRoot().DescendantNodes().Where(n=>n is ExpressionSyntax or StatementSyntax or ArrowExpressionClauseSyntax)) {
   if(model.GetOperation(syntax) is not IOperation first){continue;}
   foreach(var op in Walk(first)){string site=PracticalStrings.Site(op.Syntax);if(!seen.Add(site+"/"+op.Kind+"/"+op.Type?.ToDisplayString())){continue;}string id="";var operands=new List<IOperation>();
    switch(op){
     case IObjectCreationOperation creation when creation.Constructor is not null&&CSharpPracticalCapture.IsAllowlistedBusinessMember(creation.Constructor):id=Token(creation.Type!)+".construct";operands.AddRange(creation.Arguments.Select(a=>a.Value));break;
     case IPropertyReferenceOperation property when CSharpPracticalCapture.IsAllowlistedBusinessProperty(property.Property):id=Token(property.Property.ContainingType)+"."+(property.Property.Name switch{"DayNumber"=>"day_number","DayOfWeek"=>"day_of_week",_=>property.Property.Name.ToLowerInvariant()});operands.Add(property.Instance!);break;
     case IFieldReferenceOperation field when field.Field.Name=="Empty"&&field.Field.ContainingType.MetadataName=="Guid"&&field.Field.ContainingType.DeclaringSyntaxReferences.IsEmpty&&field.Field.ContainingAssembly.Name=="System.Runtime":id="guid.empty";break;
     case IInvocationOperation call when CSharpPracticalCapture.IsAllowlistedBusinessMember(call.TargetMethod):id=Token(call.TargetMethod.ContainingType)+"."+(call.TargetMethod.Name switch{"CompareTo"=>"compare","AddDays"=>"add_days","AddMonths"=>"add_months","AddYears"=>"add_years","Add"=>"add_duration",_=>throw PracticalFailures.Type("business_call")});operands.Add(call.Instance!);operands.AddRange(call.Arguments.Select(a=>a.Value));break;
     case IBinaryOperation binary when binary.OperatorMethod is not null&&CSharpPracticalCapture.IsAllowlistedBusinessMember(binary.OperatorMethod):if(binary.IsLifted){throw PracticalFailures.Type("business_lift");}id=Token(binary.OperatorMethod.ContainingType)+"."+(binary.OperatorKind switch{BinaryOperatorKind.Add=>"add",BinaryOperatorKind.Subtract=>"subtract",BinaryOperatorKind.Equals=>"equal",BinaryOperatorKind.NotEquals=>"not_equal",BinaryOperatorKind.LessThan=>"less",BinaryOperatorKind.LessThanOrEqual=>"less_equal",BinaryOperatorKind.GreaterThan=>"greater",BinaryOperatorKind.GreaterThanOrEqual=>"greater_equal",_=>throw PracticalFailures.Type("business_operator")});operands.Add(binary.LeftOperand);operands.Add(binary.RightOperand);break;
     case IUnaryOperation unary when unary.OperatorMethod is not null&&CSharpPracticalCapture.IsAllowlistedBusinessMember(unary.OperatorMethod):if(unary.IsLifted){throw PracticalFailures.Type("business_lift");}id="duration.negate";operands.Add(unary.Operand);break;
     default:continue;
    }
    string[] exceptions=id is "date.construct" or "date.add_days" or "date.add_months" or "date.add_years" or "time.construct"?new[]{"System.ArgumentOutOfRangeException"}:id is "duration.add" or "duration.subtract" or "duration.negate"?new[]{"System.OverflowException"}:Array.Empty<string>();
    steps.Add(new(site,id,Array.AsReadOnly(operands.ToArray()),PracticalExactTypeNormalizer.Normalize(op.Type!,c).Id,Array.AsReadOnly(exceptions)));
   }
  }}
 }
 private static IReadOnlyList<PracticalBusinessProjection> Bind(PracticalDomain domain,CSharpCompilation compilation,IReadOnlyList<PracticalBusinessBinding> bindings,List<PracticalDomainObligation> obligations)
 {
  var data=domain.Numeric.Strings.Arrays.Construction.Data;var result=new List<PracticalBusinessProjection>();var seen=new HashSet<string>(StringComparer.Ordinal);
  foreach(var binding in bindings.OrderBy(b=>b.SourceTypeId,StringComparer.Ordinal)) {
   if(!seen.Add(binding.SourceTypeId)||domain.Projections.Any(p=>p.SourceTypeId==binding.SourceTypeId)){throw PracticalFailures.Type("business_duplicate_binding");}
   var source=data.Types.SingleOrDefault(t=>t.Id==binding.SourceTypeId);
   if(source is null||source.Kind=="enum"||binding.Role is not ("instant" or "money")||binding.Role=="money"&&source.Kind!="readonly_struct"){throw PracticalFailures.Type("business_source_binding");}
   string[] roles=binding.Role=="instant"?new[]{"milliseconds"}:new[]{"amount","currency"};
   if(!binding.Members.Keys.OrderBy(s=>s,StringComparer.Ordinal).SequenceEqual(roles.OrderBy(s=>s,StringComparer.Ordinal))||binding.Members.Values.Distinct(StringComparer.Ordinal).Count()!=roles.Length){throw PracticalFailures.Type("business_member_map");}
   PracticalDataMember Member(string role)=>source.Members.SingleOrDefault(m=>m.Stored&&m.Name==binding.Members[role])??throw PracticalFailures.Type("business_member");
   string semantic;
   if(binding.Role=="instant") {if(Member("milliseconds").Type.Id!=PracticalIdentity.PrimitiveId("i64")){throw PracticalFailures.Type("instant_carrier");}semantic=PracticalIdentity.PrimitiveId("instant");}
   else {var currency=Member("currency").Type;var enumeration=data.Types.SingleOrDefault(t=>t.Id==currency.Id&&t.Kind=="enum");if(Member("amount").Type.Id!=PracticalIdentity.PrimitiveId("decimal")||currency.Nullability=="annotated"||currency.Id!=PracticalIdentity.PrimitiveId("string")&&enumeration is null){throw PracticalFailures.Type("money_members");}semantic=PracticalIdentity.ClosedInstanceId("money",currency.Id);obligations.Add(new(source.Id,"application_currency_predicate",semantic,binding.Members["currency"]));obligations.Add(new(source.Id,"default_ineligible",semantic));}
   foreach(string kind in new[]{"source_invariant_implies_projection","semantic_invariant_implies_reconstruction","source_round_trip","semantic_round_trip","distinct_arms","public_invariant","identity_unobservable"}){obligations.Add(new(source.Id,kind,semantic));}
   foreach(var member in source.Members.Where(m=>m.Stored)){obligations.Add(new(source.Id,"field_complete_reconstruction",semantic,member.Name));}
   string[] allowed=binding.Role=="instant"?new[]{"milliseconds","compare","add_duration","subtract_duration","difference"}:new[]{"create","amount","currency","add","subtract","multiply","divide","amount_compare","equal","compare"};
   foreach(var operation in binding.Operations.OrderBy(p=>p.Key,StringComparer.Ordinal)) {
    if(!allowed.Contains(operation.Key,StringComparer.Ordinal)||!data.Syntax.Callables.Any(c=>c.Id==operation.Value)){throw PracticalFailures.Type("business_operation_identity");}
    var methods=compilation.SyntaxTrees.SelectMany(tree=>tree.GetRoot().DescendantNodes().OfType<TypeDeclarationSyntax>().Select(d=>(INamedTypeSymbol)compilation.GetSemanticModel(tree).GetDeclaredSymbol(d)!)).SelectMany(t=>t.GetMembers().OfType<IMethodSymbol>());
    string TypeId(ITypeSymbol type)=>PracticalExactTypeNormalizer.Normalize(type,compilation,true).Id;
    string MethodId(IMethodSymbol method)=>PracticalIdentity.CallableId("method",method.ContainingNamespace.ToDisplayString(),PracticalIdentity.SourceTypeId(method.ContainingNamespace.ToDisplayString(),method.ContainingType.Name),method.Name,method.Parameters.Select(p=>TypeId(p.Type)),TypeId(method.ReturnType));
    var method=methods.SingleOrDefault(m=>m.MethodKind!=MethodKind.Constructor&&MethodId(m)==operation.Value)??throw PracticalFailures.Type("business_operation_identity");
    string Project(string id)=>id==source.Id?semantic:id;
    var args=method.Parameters.Select(p=>TypeId(p.Type)).ToList();if(!method.IsStatic){args.Insert(0,PracticalIdentity.SourceTypeId(method.ContainingNamespace.ToDisplayString(),method.ContainingType.Name));}
    string own=semantic,dec=PracticalIdentity.PrimitiveId("decimal"),i32=PracticalIdentity.PrimitiveId("i32"),duration=PracticalIdentity.PrimitiveId("duration");
    string[] expected;string output;string[] errors;
    switch(binding.Role+"."+operation.Key) {
     case "instant.milliseconds":expected=new[]{own};output=PracticalIdentity.PrimitiveId("i64");errors=Array.Empty<string>();break;
     case "instant.compare":case "money.compare":expected=new[]{own,own};output=i32;errors=Array.Empty<string>();break;
     case "instant.add_duration":case "instant.subtract_duration":expected=new[]{own,duration};output=own;errors=new[]{"precision","range"};break;
     case "instant.difference":expected=new[]{own,own};output=duration;errors=new[]{"range"};break;
     case "money.amount":expected=new[]{own};output=dec;errors=Array.Empty<string>();break;
     case "money.currency":expected=new[]{own};output=Member("currency").Type.Id;errors=Array.Empty<string>();break;
     case "money.equal":expected=new[]{own,own};output=PracticalIdentity.PrimitiveId("bool");errors=Array.Empty<string>();break;
     case "money.create":expected=new[]{dec,Member("currency").Type.Id,i32};output=own;errors=new[]{"invalid_currency","invalid_scale","invalid_precision"};break;
     case "money.add":case "money.subtract":expected=new[]{own,own};output=own;errors=new[]{"currency_mismatch","decimal_overflow"};break;
     case "money.amount_compare":expected=new[]{own,own};output=i32;errors=new[]{"currency_mismatch"};break;
     case "money.multiply":case "money.divide":expected=new[]{own,dec,i32,PracticalIdentity.PrimitiveId("u32")};output=own;errors=operation.Key=="divide"?new[]{"invalid_scale","invalid_rounding","division_by_zero","decimal_overflow"}:new[]{"invalid_scale","invalid_rounding","decimal_overflow"};
      if(args.Count!=4){throw PracticalFailures.Type("business_operation_signature");}
      ValidateEnum(args[3],new[]{"ToEven","AwayFromZero","ToZero","ToNegativeInfinity","ToPositiveInfinity"},true);args[3]=expected[3];break;
     default:throw PracticalFailures.Type("business_operation_name");
    }
    if(!args.Select(Project).SequenceEqual(expected)){throw PracticalFailures.Type("business_operation_signature");}
    if(errors.Length==0){if(Project(TypeId(method.ReturnType))!=output){throw PracticalFailures.Type("business_operation_result");}}
    else {
     var outcome=domain.Projections.SingleOrDefault(p=>p.SourceTypeId==TypeId(method.ReturnType)&&p.Role=="result")??throw PracticalFailures.Type("business_result_binding");
     if(Project(outcome.Arguments[0])!=output){throw PracticalFailures.Type("business_operation_result");}
     ValidateEnum(outcome.Arguments[1],errors,false);
     obligations.Add(new(source.Id,"separate_closed_result_projection",semantic,outcome.SourceTypeId));
     obligations.Add(new(source.Id,"exhaustive_error_projection",semantic,outcome.Arguments[1]));
    }
    foreach(string kind in new[]{"operation_normal_commutation","operation_error_commutation","operation_exception_commutation"}){obligations.Add(new(source.Id,kind,semantic,operation.Key+":"+operation.Value));}
    void ValidateEnum(string id,string[] required,bool rounding) {
     var en=data.Types.SingleOrDefault(t=>t.Id==id&&t.Kind=="enum");
     if(en is null||binding.EnumArms is null||!binding.EnumArms.TryGetValue(id,out var arms)){throw PracticalFailures.Type("business_enum_projection");}
     string[] allowedErrors=rounding?required:binding.Role=="instant"?new[]{"precision","range"}:new[]{"invalid_currency","invalid_scale","invalid_precision","currency_mismatch","invalid_rounding","division_by_zero","decimal_overflow"};
     if(required.Any(e=>!arms.ContainsKey(e))||arms.Keys.Any(e=>!allowedErrors.Contains(e,StringComparer.Ordinal))||arms.Values.Distinct(StringComparer.Ordinal).Count()!=arms.Count||!arms.Values.OrderBy(v=>v,StringComparer.Ordinal).SequenceEqual(en.EnumMembers.Select(m=>m.Value).Distinct(StringComparer.Ordinal).OrderBy(v=>v,StringComparer.Ordinal))){throw PracticalFailures.Type("business_enum_projection");}
     obligations.Add(new(source.Id,rounding?"exhaustive_rounding_projection":"exhaustive_error_projection",semantic,id));
    }
   }
   var closure=data.Syntax.SourceClosure;var declaration=closure.Declarations.Single(d=>d.Kind==PracticalDeclarationKind.Type&&d.Id==source.Id);
   IReadOnlyDictionary<string,string> Copy(IReadOnlyDictionary<string,string> map)=>new System.Collections.ObjectModel.ReadOnlyDictionary<string,string>(new SortedDictionary<string,string>(map.ToDictionary(p=>p.Key,p=>p.Value),StringComparer.Ordinal));
   result.Add(new(source.Id,closure.Sources[declaration.SourceOrdinal].RawSha256,semantic,binding.Role,Copy(binding.Members),Copy(binding.Operations),new System.Collections.ObjectModel.ReadOnlyDictionary<string,IReadOnlyDictionary<string,string>>(new SortedDictionary<string,IReadOnlyDictionary<string,string>>((binding.EnumArms??new Dictionary<string,IReadOnlyDictionary<string,string>>()).ToDictionary(p=>p.Key,p=>Copy(p.Value)),StringComparer.Ordinal))));
  }
  return Array.AsReadOnly(result.ToArray());
 }
}
