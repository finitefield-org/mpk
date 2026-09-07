using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using System.Text;
using System.Text.Json;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

// Captured source facts for the sole native specialization/import path. This
// transport is not a frontend success envelope and makes no proof claim.
internal sealed class PracticalDataSource
{
    private readonly byte[] bytes;
    internal PracticalDataSource(byte[] bytes) { this.bytes = (byte[])bytes.Clone(); }
    internal byte[] CopyBytes() => (byte[])bytes.Clone();
}

internal static class CSharpPracticalDataPhase
{
    // Native preflight validates the strict sidecar transport. This bridge uses
    // those actual selected bytes and resolves stored-member IDs against this
    // compilation before invoking the existing W12/W13 source validators.
    internal static PracticalDataSource CaptureSelected(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> supplied, ImmutableArray<MetadataReference> references)
    {
        var inputs=supplied.ToArray();
        var initial=Capture(selection,inputs,references);
        using var facts=JsonDocument.Parse(initial.CopyBytes());
        var business=new List<PracticalBusinessBinding>();var outcomes=new List<PracticalOutcomeBinding>();
        foreach(var input in inputs.Where(i=>i.Kind==PracticalCapturedInputKind.Sidecar)) {
            using var document=JsonDocument.Parse(input.CopyBytes());
            var root=document.RootElement;
            if(root.GetProperty("schema").GetString()!="mpk.csharp.semantic_bindings.v1") continue;
            foreach(var row in root.GetProperty("bindings").EnumerateArray()) {
                string id=row.GetProperty("source_type_id").GetString()!;
                string role=row.GetProperty("role").GetString()!;
                var type=facts.RootElement.GetProperty("types").EnumerateArray().SingleOrDefault(t=>t.GetProperty("id").GetString()==id);
                if(type.ValueKind==JsonValueKind.Undefined) throw PracticalFailures.Type("business_source_binding");
                if(row.GetProperty("source_content_sha256").GetString()!=type.GetProperty("source_sha256").GetString()) throw PracticalFailures.Type("data_binding_source_hash");
                var members=new Dictionary<string,string>(StringComparer.Ordinal);
                foreach(var mapping in row.GetProperty("member_map").EnumerateObject()) {
                    var member=type.GetProperty("members").EnumerateArray().SingleOrDefault(m=>StoredMemberId(id,m)==mapping.Value.GetString());
                    if(member.ValueKind==JsonValueKind.Undefined) throw PracticalFailures.Type("business_member");
                    members.Add(mapping.Name,member.GetProperty("name").GetString()!);
                }
                Dictionary<string,string> Map(JsonElement value)=>value.EnumerateObject().ToDictionary(p=>p.Name,p=>p.Value.GetString()!,StringComparer.Ordinal);
                var operations=Map(row.GetProperty("operation_map"));
                if(role is "instant" or "money") {
                    var enums=row.GetProperty("enum_arms").EnumerateObject().ToDictionary(p=>p.Name,p=>(IReadOnlyDictionary<string,string>)Map(p.Value),StringComparer.Ordinal);
                    business.Add(new(id,role,members,operations,enums));
                } else if(role is "option" or "lookup" or "result" or "validation" or "boundary_field") {
                    outcomes.Add(new(id,role,members,Map(row.GetProperty("tag_arms")),operations));
                }
            }
        }
        return Capture(selection,inputs,references,business,outcomes);
    }
    private static string StoredMemberId(string owner,JsonElement member)
    {
        var value=new SortedDictionary<string,object>(StringComparer.Ordinal) {
            ["name"]=member.GetProperty("name").GetString()!,["owner"]=owner,
            ["storage"]=member.GetProperty("storage").GetString()!,["type"]=CanonicalObject(member.GetProperty("type")),
        };
        byte[] body=JsonSerializer.SerializeToUtf8Bytes(value);
        byte[] domain=Encoding.ASCII.GetBytes("MPK-CSHARP-FOUNDATION-MEMBER-1.0\0");
        return "mpk.csharp.member."+Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(domain.Concat(body).ToArray())).ToLowerInvariant();
    }
    private static object CanonicalObject(JsonElement value)=>value.ValueKind switch {
        JsonValueKind.Object=>new SortedDictionary<string,object>(value.EnumerateObject().ToDictionary(p=>p.Name,p=>CanonicalObject(p.Value)),StringComparer.Ordinal),
        JsonValueKind.Array=>value.EnumerateArray().Select(CanonicalObject).ToArray(),
        JsonValueKind.String=>value.GetString()!,
        _=>throw PracticalFailures.Type("data_member_descriptor"),
    };

    internal static PracticalDataSource Capture(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs, ImmutableArray<MetadataReference> references,
        IReadOnlyList<PracticalBusinessBinding>? businessBindings = null,
        IReadOnlyList<PracticalOutcomeBinding>? outcomeBindings = null)
    {
        inputs=inputs.ToArray();
        CSharpCompilation? compilation = null;
        var business = CSharpPracticalBusiness.Validate(selection, inputs, references,
            businessBindings, outcomeBindings, c => compilation = c, deferSidecarAttachment: selection.SidecarPaths.Count != 0);
        var data = business.Domain.Numeric.Strings.Arrays.Construction.Data;
        var closure = data.Syntax.SourceClosure;
        foreach (var tree in compilation!.SyntaxTrees)
        {
            foreach (var node in tree.GetRoot().DescendantNodes())
            {
                if (node is ForStatementSyntax or ForEachStatementSyntax or ForEachVariableStatementSyntax
                    or WhileStatementSyntax or DoStatementSyntax)
                    throw PracticalFailures.Type("data_later_owner_T04_W01_W02");
                if (node is TryStatementSyntax or ThrowStatementSyntax or ThrowExpressionSyntax)
                    throw PracticalFailures.Type("data_later_owner_T04_W05");
            }
        }

        var symbols = new Dictionary<string, INamedTypeSymbol>(StringComparer.Ordinal);
        foreach (var tree in compilation.SyntaxTrees)
        {
            var model = compilation.GetSemanticModel(tree);
            foreach (var declaration in tree.GetRoot().DescendantNodes().OfType<BaseTypeDeclarationSyntax>())
            {
                var symbol = (INamedTypeSymbol)model.GetDeclaredSymbol(declaration)!;
                if (CSharpPracticalDataTypes.ClassifySourceException(symbol) is not null)
                    throw PracticalFailures.Type("data_later_owner_T04_W04");
                symbols.Add(PracticalIdentity.SourceTypeId(symbol.ContainingNamespace.ToDisplayString(), symbol.Name), symbol);
            }
        }
        var types = data.Types.Where(type => type.Id.StartsWith("mpk.csharp.source.", StringComparison.Ordinal)).Select(type => {
            var symbol = symbols[type.Id];
            var location = closure.Declarations.Single(d => d.Id == type.Id);
            return new {
                id = type.Id, kind = type.Kind, name = symbol.Name,
                @namespace = symbol.ContainingNamespace.ToDisplayString(),
                source_path = closure.Sources[location.SourceOrdinal].Path,
                source_sha256 = closure.Sources[location.SourceOrdinal].RawSha256,
                start_byte = location.StartByte, end_byte = location.EndByte,
                enum_underlying = type.Carrier, enum_values = type.EnumMembers.Select(m => m.Value).ToArray(),
                members = type.Members.Where(m => m.Stored).Select(m => new {
                    name = m.Name, type = CSharpPracticalStructural.TypeDescriptor(m.Type), required = m.Required,
                    storage = m.Kind == "field" ? "readonly_field" :
                        ((IPropertySymbol)symbol.GetMembers(m.Name).Single()).SetMethod is null ? "get_auto" : "init_auto",
                    start_byte = closure.Sources[location.SourceOrdinal].ByteOffset(symbol.GetMembers(m.Name).Single().DeclaringSyntaxReferences.Single().GetSyntax().SpanStart),
                    end_byte = closure.Sources[location.SourceOrdinal].ByteOffset(symbol.GetMembers(m.Name).Single().DeclaringSyntaxReferences.Single().GetSyntax().Span.End),
                }).ToArray(),
                recursive_default = DefaultGraph(type.DefaultValue),
                public_default = type.DefaultEligible,
            };
        }).ToArray();
        var callables = data.Syntax.Callables.Select(callable => {
            var declaration = closure.Declarations.Single(d => d.Id == callable.Id);
            var source = closure.Sources[declaration.SourceOrdinal];
            var tree = compilation.SyntaxTrees.Single(t => t.FilePath == source.Path);
            var syntax = tree.GetRoot().DescendantNodes().Single(n =>
                n is BaseMethodDeclarationSyntax or PropertyDeclarationSyntax &&
                source.ByteOffset(n.SpanStart) == declaration.StartByte && source.ByteOffset(n.Span.End) == declaration.EndByte);
            var declared = compilation.GetSemanticModel(tree).GetDeclaredSymbol(syntax);
            var method = declared is IPropertySymbol property ? property.GetMethod! : (IMethodSymbol)declared!;
            var owner = PracticalIdentity.SourceTypeId(method.ContainingNamespace.ToDisplayString(),method.ContainingType.Name);
            bool constructor = method.MethodKind == MethodKind.Constructor;
            var result = constructor ? PracticalExactTypeNormalizer.Normalize(method.ContainingType.IsReferenceType?method.ContainingType.WithNullableAnnotation(NullableAnnotation.NotAnnotated):method.ContainingType,compilation) : PracticalExactTypeNormalizer.Normalize(method.ReturnType,compilation,true);
            return new {
                identity = new { kind = constructor ? "constructor" : "method", name = constructor ? method.ContainingType.Name : method.Name,
                    @namespace = method.ContainingNamespace.ToDisplayString(), owner,
                    parameter_type_ids = method.Parameters.Select(p => PracticalExactTypeNormalizer.Normalize(p.Type,compilation).Id).ToArray(), result_type_id = result.Id },
                is_static = method.IsStatic,
                is_property_getter = method.MethodKind == MethodKind.PropertyGet,
                is_synthesized_constructor = false,
                parameters = method.Parameters.Select(p => CSharpPracticalStructural.TypeDescriptor(PracticalExactTypeNormalizer.Normalize(p.Type,compilation))).ToArray(),
                result = CSharpPracticalStructural.TypeDescriptor(result),
                id = callable.Id, source_path = closure.Sources[declaration.SourceOrdinal].Path,
                source_sha256 = closure.Sources[declaration.SourceOrdinal].RawSha256,
                start_byte = declaration.StartByte, end_byte = declaration.EndByte,
                body_sha256 = callable.BodySha256,
                body_utf8 = Encoding.UTF8.GetString(callable.CopyBodyBytes()),
                data_steps = Steps(callable,business),
                initialization_plans = Initializations(callable,business.Domain.Numeric.Strings.Arrays.Construction),
            };
        }).Cast<object>().ToList();
        var synthesized=business.Domain.Numeric.Strings.Arrays.Construction.Constructors.Where(p=>p.Synthesized).ToArray();
        foreach(var plan in synthesized)
        {
            var symbol=symbols[plan.TypeId];
            var method=symbol.InstanceConstructors.Single(m=>m.IsImplicitlyDeclared && m.Parameters.Length==0);
            if(PracticalIdentity.CallableId("constructor",symbol.ContainingNamespace.ToDisplayString(),plan.TypeId,symbol.Name,
                Array.Empty<string>(),plan.TypeId)!=plan.Id) throw PracticalFailures.Object("data_synthesized_constructor_identity");
            var declaration=closure.Declarations.Single(d=>d.Id==plan.TypeId);
            var source=closure.Sources[declaration.SourceOrdinal];
            var result=PracticalExactTypeNormalizer.Normalize(symbol.IsReferenceType?symbol.WithNullableAnnotation(NullableAnnotation.NotAnnotated):symbol,compilation);
            callables.Add(new {
                identity=new {kind="constructor",name=symbol.Name,@namespace=symbol.ContainingNamespace.ToDisplayString(),owner=plan.TypeId,
                    parameter_type_ids=Array.Empty<string>(),result_type_id=plan.TypeId},
                is_static=false,is_property_getter=false,is_synthesized_constructor=true,
                parameters=Array.Empty<object>(),result=CSharpPracticalStructural.TypeDescriptor(result),id=plan.Id,
                source_path=source.Path,source_sha256=source.RawSha256,start_byte=declaration.StartByte,end_byte=declaration.EndByte,
                body_sha256=Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(Encoding.UTF8.GetBytes("[]"))).ToLowerInvariant(),
                body_utf8="[]",data_steps=Array.Empty<object>(),initialization_plans=Array.Empty<object>(),
            });
        }
        return new PracticalDataSource(JsonSerializer.SerializeToUtf8Bytes(new {
            compilation_id=selection.CompilationId,
            input_files=inputs.OrderBy(i=>i.NormalizedPath,StringComparer.Ordinal).Select(i=>new {kind=i.Kind.ToString().ToLowerInvariant(),path=i.NormalizedPath,raw_sha256=i.RawSha256,size_bytes=i.SizeBytes}).ToArray(),
            types, callables=callables.OrderBy(c=>JsonSerializer.SerializeToElement(c).GetProperty("id").GetString(),StringComparer.Ordinal).ToArray(),
            source_obligations = Obligations(business),
            source_containers = symbols.Where(p=>p.Value.IsStatic && closure.ReachableDeclarations.Any(d=>d.Id==p.Key))
                .OrderBy(p=>p.Key,StringComparer.Ordinal).Select(p=> {
                    var declaration=closure.Declarations.Single(d=>d.Id==p.Key);
                    var source=closure.Sources[declaration.SourceOrdinal];
                    return new {id=p.Key,name=p.Value.Name,@namespace=p.Value.ContainingNamespace.ToDisplayString(),
                        source_path=source.Path,source_sha256=source.RawSha256,start_byte=declaration.StartByte,end_byte=declaration.EndByte};
                }).ToArray(),
            sources = closure.Sources.Select(s => new { path = s.Path, raw_sha256 = s.RawSha256, size_bytes = s.SizeBytes }).ToArray(),
            selected_root_ids = selection.SelectedRootIds,
            reachable_declarations = closure.ReachableDeclarations.Select(d => d.Id).Concat(synthesized.Select(p=>p.Id)).Distinct(StringComparer.Ordinal).OrderBy(id=>id,StringComparer.Ordinal).ToArray(),
            exact_types = data.Syntax.ExactTypes.Select(t => new { callable_id = t.CallableId, local_ordinal = t.LocalOrdinal, type = CSharpPracticalStructural.TypeDescriptor(t.Type) }).ToArray(),
        }));
    }

    private static object[] Obligations(PracticalBusiness business)
    {
        var construction=business.Domain.Numeric.Strings.Arrays.Construction;
        var closure=construction.Data.Syntax.SourceClosure;
        var rows=new SortedDictionary<string,object>(StringComparer.Ordinal);
        void Add(string family,string site,string kind,string typeId,IReadOnlyList<string> members,string predicate,bool discharged=false,IOperation? operation=null)
        {
            if(discharged) throw PracticalFailures.Type("data_discharged_obligation");
            // Binding obligations are about this exact source projection. Its
            // final semantic target (including nested business projections)
            // is supplied by the sole native specialization engine.
            if(family=="business" && business.Projections.Any(p=>p.SourceTypeId==site&&p.SemanticTypeId==typeId)
                || family=="domain" && business.Domain.Projections.Any(p=>p.SourceTypeId==site&&p.SemanticTypeId==typeId)) typeId=site;
            var declaration=closure.Declarations.Where(d=>site==d.Id || site.StartsWith(d.Id+":",StringComparison.Ordinal))
                .OrderByDescending(d=>d.Id.Length).FirstOrDefault();
            var synthesized=construction.Constructors.SingleOrDefault(p=>p.Synthesized && (site==p.Id || site.StartsWith(p.Id+":",StringComparison.Ordinal)));
            if(declaration is null && synthesized is not null) declaration=closure.Declarations.Single(d=>d.Id==synthesized.TypeId);
            string path;int start,end;
            if(declaration is not null) {
                path=closure.Sources[declaration.SourceOrdinal].Path;start=declaration.StartByte;end=declaration.EndByte;
            } else {
                var syntax=operation?.Syntax;
                if(syntax is not null) {
                    path=syntax.SyntaxTree.FilePath;
                    var source=closure.Sources.Single(s=>s.Path==path);
                    start=source.ByteOffset(syntax.SpanStart);end=source.ByteOffset(syntax.Span.End);
                } else {
                    var match=System.Text.RegularExpressions.Regex.Match(site,@"^(.*):([0-9]+):([0-9]+)(?::.*)?$");
                    if(!match.Success) throw PracticalFailures.Type("data_obligation_site");
                    path=match.Groups[1].Value;var source=closure.Sources.Single(s=>s.Path==path);
                    int offset=int.Parse(match.Groups[2].Value,System.Globalization.CultureInfo.InvariantCulture);
                    start=source.ByteOffset(offset);end=source.ByteOffset(offset+int.Parse(match.Groups[3].Value,System.Globalization.CultureInfo.InvariantCulture));
                }
                declaration=closure.Declarations.Where(d=>closure.Sources[d.SourceOrdinal].Path==path&&d.StartByte<=start&&end<=d.EndByte)
                    .OrderBy(d=>d.EndByte-d.StartByte).FirstOrDefault();
            }
            // Property display declarations and getter declarations share a
            // source span. Proof subjects must use the retained logical getter.
            if(declaration is not null && !declaration.Id.StartsWith("mpk.csharp.source.",StringComparison.Ordinal))
                declaration=closure.Declarations.Where(d=>d.Id.StartsWith("mpk.csharp.source.",StringComparison.Ordinal)
                    && closure.Sources[d.SourceOrdinal].Path==path && d.StartByte<=start && end<=d.EndByte)
                    .OrderBy(d=>d.EndByte-d.StartByte).FirstOrDefault();
            if(declaration is null || !closure.ReachableDeclarations.Any(d=>d.Id==declaration.Id))
                throw PracticalFailures.Type("data_obligation_declaration");
            var row=new {family,site,kind,type_id=typeId,members,predicate,discharged=false,
                declaration_id=synthesized?.Id??declaration.Id,source_path=path,source_sha256=closure.Sources.Single(s=>s.Path==path).RawSha256,start_byte=start,end_byte=end};
            rows.TryAdd(JsonSerializer.Serialize(row),row);
        }
        foreach(var o in construction.Obligations) {
            if(o.Expression is not null) throw PracticalFailures.Type("data_source_invariant_expression_handoff");
            Add("construction",o.Site,o.Kind,o.TypeId,o.Members,"",o.Discharged);
        }
        foreach(var o in business.Domain.Numeric.Strings.Obligations)
            Add("string",o.Site,o.Kind,"",Array.Empty<string>(),"",o.Discharged);
        foreach(var o in business.Domain.Obligations)
            Add("domain",o.Site,o.Kind,o.TypeId,o.Member.Length==0?Array.Empty<string>():new[]{o.Member},"",o.Discharged);
        foreach(var o in business.Obligations)
            Add("business",o.Site,o.Kind,o.TypeId,o.Member.Length==0?Array.Empty<string>():new[]{o.Member},"",o.Discharged);
        foreach(var o in business.Domain.Numeric.Strings.Arrays.Steps.Where(s=>s.Predicate.Length!=0))
            Add("array",o.Site,o.Operation,o.ElementType?.Id??"",o.Arrays,o.Predicate,operation:o.Source);
        return rows.Values.ToArray();
    }

    // Recipes retain the existing owning validators. Their operand ordinals
    // reference the original preorder body; source evaluation order is not lost
    // when named arguments are later put into semantic argument positions.
    private static object[] Steps(PracticalNormalizedCallable callable,PracticalBusiness business)
    {
        var nodes=callable.OperationNodes;
        bool Same(IOperation? a,IOperation b)=>a is not null && a.Kind==b.Kind
            && a.IsImplicit==b.IsImplicit && a.Syntax.SyntaxTree.FilePath==b.Syntax.SyntaxTree.FilePath
            && a.Syntax.Span==b.Syntax.Span
            && a.Type?.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat)==b.Type?.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat);
        int Find(IOperation operation) {
            var exact=Enumerable.Range(0,nodes.Count).Where(i=>ReferenceEquals(nodes[i],operation)).ToArray();
            var matches=exact.Length==0?Enumerable.Range(0,nodes.Count).Where(i=>Same(nodes[i],operation)).ToArray():exact;
            if(matches.Length!=1) throw PracticalFailures.Type("data_recipe_operation_identity");
            return matches[0];
        }
        var result=new SortedDictionary<int,object>();
        void Add(string family,IOperation source,string id,IReadOnlyList<IOperation> operands,IReadOnlyList<int> ordinals,string rounding) {
            if(!nodes.Any(n=>Same(n,source))) return;
            int ordinal=Find(source);
            if(result.ContainsKey(ordinal)) throw PracticalFailures.Type("data_recipe_overlap");
            result.Add(ordinal,new { node_ordinal=ordinal,family,operation=id,
                operand_ordinals=operands.Select(Find).ToArray(),argument_ordinals=ordinals,rounding });
        }
        foreach(var step in business.Domain.Numeric.Strings.Steps)
            Add("string",step.Source,step.Operation=="string.equals.ordinal" && step.Source is IInvocationOperation {Instance:not null}?"string.equals.instance.ordinal":step.Operation,step.Operands,step.ArgumentOrdinals,"");
        foreach(var step in business.Domain.Numeric.Steps)
            Add("numeric",step.Source,step.Operation,step.Operands,Ordinals(step.Source,step.Operands),step.Rounding);
        foreach(var step in business.Steps)
            Add("business",step.Source,step.Operation,step.Operands,Ordinals(step.Source,step.Operands),"");
        foreach(var step in business.Domain.Steps.Where(s=>s.Source is not null &&
            (s.Operation.StartsWith("nullable.",StringComparison.Ordinal) || s.Operation.StartsWith("lifted.",StringComparison.Ordinal) || s.Operation=="reference.conditional_access")))
            Add("domain",step.Source!,step.Operation.StartsWith("lifted.",StringComparison.Ordinal)?step.Operation+(step.Checked?".checked":".unchecked"):step.Operation,
                step.Operands,Ordinals(step.Source!,step.Operands),"");
        foreach(var step in business.Domain.Numeric.Strings.Arrays.Steps.Where(s=>s.WriteMode.Length!=0)) {
            if(step.Operand is not ISimpleAssignmentOperation) continue;
            Add("array",step.Operand,step.WriteMode,Array.Empty<IOperation>(),Array.Empty<int>(),"");
        }
        foreach(var equality in business.Domain.Numeric.Strings.Arrays.Construction.SourceEqualities) {
            var matches=nodes.Where(n=>n is not null && PracticalStrings.Site(n.Syntax)==equality.Site
                && n is IBinaryOperation or IInvocationOperation).ToArray();
            if(matches.Length==0) continue;
            if(matches.Length!=1) throw PracticalFailures.Type("data_equality_operation_identity");
            int ordinal=Find(matches[0]!);
            if(result.ContainsKey(ordinal)) continue;
            Add("structural",matches[0]!,equality.Negated?"structural_not_equal":"structural_equal",
                new[]{equality.Left,equality.Right},new[]{0,1},"");
        }
        return result.Values.ToArray();
    }
    private static object[] Initializations(PracticalNormalizedCallable callable,PracticalConstruction construction)
    {
        var nodes=callable.OperationNodes;
        int Find(IOperation operation)
        {
            var exact=Enumerable.Range(0,nodes.Count).Where(i=>ReferenceEquals(nodes[i],operation)).ToArray();
            var matches=exact.Length==0?Enumerable.Range(0,nodes.Count).Where(i=>nodes[i] is IOperation n
                && n.Syntax.SyntaxTree.FilePath==operation.Syntax.SyntaxTree.FilePath && n.Syntax.Span==operation.Syntax.Span
                && n.Kind==operation.Kind && n.IsImplicit==operation.IsImplicit
                && n.Type?.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat)==operation.Type?.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat)).ToArray():exact;
            if(matches.Length!=1) throw PracticalFailures.Object("data_initialization_operation_identity");
            return matches[0];
        }
        var rows=new SortedDictionary<int,object>();
        foreach(var plan in construction.Initializations)
        {
            var creations=nodes.OfType<IObjectCreationOperation>().Where(n=>PracticalStrings.Site(n.Syntax)==plan.Site).ToArray();
            if(creations.Length==0) continue;
            if(creations.Length!=1) throw PracticalFailures.Object("data_initialization_site_identity");
            int ordinal=Find(creations[0]);
            rows.Add(ordinal,new {node_ordinal=ordinal,site=plan.Site,type_id=plan.TypeId,constructor_id=plan.ConstructorId,
                member_order=plan.MemberOrder,definitely_assigned=plan.DefinitelyAssigned,possibly_assigned=plan.PossiblyAssigned,
                has_normal_exit=plan.HasNormalExit,
                steps=plan.Steps.Select(step=>new {kind=step.Kind.ToString(),target=step.Target,
                    expression_ordinal=step.Expression is null?(int?)null:Find(step.Expression),exceptional_exit=step.ExceptionalExit}).ToArray()});
        }
        return rows.Values.ToArray();
    }
    private static int[] Ordinals(IOperation source,IReadOnlyList<IOperation> operands)
    {
        var arguments=source switch { IInvocationOperation call=>call.Arguments,
            IObjectCreationOperation creation=>creation.Arguments,_=>ImmutableArray<IArgumentOperation>.Empty };
        return operands.Select((operand,index)=>arguments.FirstOrDefault(a=>a.Value.Syntax.Span==operand.Syntax.Span
            && a.Value.Kind==operand.Kind)?.Parameter?.Ordinal ?? (source is IInvocationOperation {Instance:not null} && index==0?-1:index)).ToArray();
    }

    // Serialize each shared node once. System.Text.Json does not serialize
    // internal properties, and expanding this DAG as a tree can be exponential.
    private static object? DefaultGraph(PracticalDefaultValue? root)
    {
        if (root is null) return null;
        var ordinals = new Dictionary<PracticalDefaultValue, int>(ReferenceEqualityComparer.Instance);
        var nodes = new List<object>();
        int Visit(PracticalDefaultValue value)
        {
            if (ordinals.TryGetValue(value, out int existing)) return existing;
            int[] children = value.Members.Select(Visit).ToArray();
            int ordinal = nodes.Count;
            nodes.Add(new { type_id = value.TypeId, kind = value.Kind,
                scalar = value.Scalar, members = children });
            ordinals.Add(value, ordinal);
            return ordinal;
        }
        int rootOrdinal = Visit(root);
        return new { root = rootOrdinal, nodes = nodes.ToArray() };
    }

    internal static void ValidateCandidate(PracticalDataSource regenerated, ReadOnlySpan<byte> candidate)
    {
        if (!candidate.SequenceEqual(regenerated.CopyBytes())) throw PracticalFailures.Type("data_source_mismatch");
    }
}
