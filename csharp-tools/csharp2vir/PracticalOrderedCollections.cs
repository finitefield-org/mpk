using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using System.Text.Json;
using Microsoft.CodeAnalysis;

namespace Mpk.CSharp2Vir;

// Typed consumers of the frozen semantic-binding roles, not another sidecar
// format. Binding-expression parsing/proof discharge remain T05/T06 work.
internal sealed record PracticalOrderedBinding(string Role, string ArrayTypeId,
    string EntryTypeId = "", string KeyMember = "", string ValueMember = "",
    string WrapperTypeId = "", string WrapperMember = "");
internal sealed record PracticalOrderedProjection(PracticalOrderedBinding Binding,
    string KeyTypeId, string ValueTypeId, string SemanticTypeId, string EntrySemanticTypeId,
    string LookupTypeId, string EntrySourceSha256, string WrapperSourceSha256, bool Comparable);
internal sealed record PracticalOrderedObligation(string SemanticTypeId, string Site,
    string Kind, string Member = "", bool Discharged = false);
internal sealed record PracticalOrderedOperation(string SemanticTypeId, string Name,
    IReadOnlyList<string> ArgumentTypeIds, string ResultTypeId, IReadOnlyList<string> OrderedOutcomes);
internal sealed record PracticalOrderedCollections(PracticalSequences Sequences,
    IReadOnlyList<PracticalOrderedProjection> Projections, IReadOnlyList<PracticalOrderedObligation> Obligations,
    IReadOnlyList<PracticalOrderedOperation> Operations)
{
    internal int ArtifactCount => 0;
    internal byte[] CopyCanonicalBytes() => JsonSerializer.SerializeToUtf8Bytes(new {
        source = Convert.ToBase64String(Sequences.CopyCanonicalBytes()), projections = Projections,
        obligations = Obligations, operations = Operations,
    });
}

internal static class CSharpPracticalOrderedCollections
{
    internal const string LoopOwner = "CSHARP-03-T04-W01/W02";
    internal const int MaximumEntries = 4096;
    internal const int MaximumCells = 65536;
    private static readonly string[] ProjectionObligations = {
        "source_invariant_implies_projection", "semantic_invariant_implies_reconstruction",
        "source_round_trip", "semantic_round_trip", "distinct_arms", "public_invariant", "identity_unobservable",
    };
    internal static PracticalOrderedCollections Validate(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs, ImmutableArray<MetadataReference> references,
        IReadOnlyList<PracticalOrderedBinding> bindings,
        IReadOnlyList<PracticalTypeInvariantClaim>? invariantClaims = null)
    {
        // W08 validates wrapper immutability/array shape and retains all source
        // bodies. Several collection roles may not silently classify one array.
        var wrappers = bindings.Where(b=>b.WrapperTypeId.Length!=0)
            .Select(b=>new PracticalSequenceBinding(b.WrapperTypeId,b.WrapperMember)).ToArray();
        PracticalSequences sequences = CSharpPracticalSequences.Validate(selection,inputs,references,wrappers,invariantClaims);
        var projections = new List<PracticalOrderedProjection>();
        var obligations = new List<PracticalOrderedObligation>();
        var operations = new List<PracticalOrderedOperation>();
        var classified = new HashSet<string>(StringComparer.Ordinal);
        foreach (PracticalOrderedBinding binding in bindings.OrderBy(b=>b.ArrayTypeId,StringComparer.Ordinal)) {
            if (binding.Role is not ("ordered_map" or "ordered_set") || !classified.Add(binding.ArrayTypeId))
            { throw PracticalFailures.Type("ordered_binding_role"); }
            var observed = sequences.Steps.SelectMany(s=>s.Types).Where(t=>t.SequenceTypeId==binding.ArrayTypeId).ToArray();
            if (observed.Length==0 || observed.Any(t=>t.ElementTypeId!=observed[0].ElementTypeId))
            { throw PracticalFailures.Type("ordered_array_binding"); }
            if (binding.WrapperTypeId.Length==0 ? binding.WrapperMember.Length!=0
                : !sequences.Projections.Any(p=>p.SourceTypeId==binding.WrapperTypeId && p.SequenceTypeId==binding.ArrayTypeId))
            { throw PracticalFailures.Type("ordered_wrapper_binding"); }
            string element = observed[0].ElementTypeId, key, value = "", entrySemantic = "", lookup = "";
            bool keyTotal, valueTotal = true;
            PracticalDataType? entry = null;
            if (binding.Role=="ordered_map") {
                entry = sequences.Arrays.Construction.Data.Types.SingleOrDefault(t=>t.Id==binding.EntryTypeId);
                if (entry is null || entry.Kind=="enum" || element!=entry.Id || binding.KeyMember==binding.ValueMember)
                { throw PracticalFailures.Type("ordered_entry_binding"); }
                PracticalDataMember? k=entry.Members.SingleOrDefault(m=>m.Stored && m.Name==binding.KeyMember);
                PracticalDataMember? v=entry.Members.SingleOrDefault(m=>m.Stored && m.Name==binding.ValueMember);
                if(k is null || v is null) { throw PracticalFailures.Type("ordered_entry_members"); }
                key=ValueId(k.Type); value=ValueId(v.Type);
                keyTotal=Total(k.Type,sequences); valueTotal=Total(v.Type,sequences);
                entrySemantic=PracticalIdentity.ClosedInstanceId("ordered_entry",key,value);
                lookup=PracticalIdentity.ClosedInstanceId("lookup",value);
            } else {
                if (binding.EntryTypeId.Length!=0 || binding.KeyMember.Length!=0 || binding.ValueMember.Length!=0)
                { throw PracticalFailures.Type("ordered_set_binding"); }
                key=element;
                PracticalNormalizedType? sourceType=sequences.Arrays.Steps.FirstOrDefault(s=>s.ElementType is not null
                    && ValueId(s.ElementType)==key)?.ElementType;
                if(sourceType is null) {throw PracticalFailures.Type("ordered_key_type");}
                keyTotal=Total(sourceType,sequences);
            }
            if(!keyTotal) {throw PracticalFailures.Type("ordered_key_type");}
            string semantic=binding.Role=="ordered_map" ? PracticalIdentity.ClosedInstanceId("ordered_map",key,value)
                : PracticalIdentity.ClosedInstanceId("ordered_set",key);
            var projection=new PracticalOrderedProjection(binding,key,value,semantic,entrySemantic,lookup,
                entry is null ? "" : SourceHash(entry.Id,sequences),
                binding.WrapperTypeId.Length==0 ? "" : SourceHash(binding.WrapperTypeId,sequences),valueTotal);
            projections.Add(projection);
            foreach (string source in new[]{binding.EntryTypeId,binding.WrapperTypeId}.Where(id=>id.Length!=0).Distinct(StringComparer.Ordinal)) {
                foreach (string kind in ProjectionObligations) {obligations.Add(new(semantic,source,kind));}
                foreach (var member in sequences.Arrays.Construction.Data.Types.Single(t=>t.Id==source).Members.Where(m=>m.Stored))
                {obligations.Add(new(semantic,source,"field_complete_reconstruction",member.Name));}
            }
            // Input and each publication boundary carry representation VCs.
            // No sortedness/uniqueness failure is turned into a C# exception.
            foreach(var step in sequences.Steps.Where(s=>s.Types.Any(t=>t.SequenceTypeId==binding.ArrayTypeId)
                && (s.Action is "freeze" or "publish_immutable" || s.Source.Operation is "profile_bound" or "parameter_profile_bound"))) {
                string site=step.Source.Path+"/"+step.Source.Site;
                foreach(string kind in new[]{"length_le_4096","total_collection_cells_le_65536","strictly_increasing_keys","unique_keys","element_public_invariants"})
                {obligations.Add(new(semantic,site,kind));}
            }
            AddOperations(projection,operations);
        }
        return new(sequences,Array.AsReadOnly(projections.ToArray()),
            Array.AsReadOnly(obligations.Distinct().OrderBy(o=>o.SemanticTypeId,StringComparer.Ordinal)
                .ThenBy(o=>o.Site,StringComparer.Ordinal).ThenBy(o=>o.Kind,StringComparer.Ordinal).ThenBy(o=>o.Member,StringComparer.Ordinal).ToArray()),
            Array.AsReadOnly(operations.OrderBy(o=>o.SemanticTypeId,StringComparer.Ordinal).ThenBy(o=>o.Name,StringComparer.Ordinal).ToArray()));
    }
    internal static void ValidateCandidate(PracticalOrderedCollections regenerated,ReadOnlySpan<byte> candidate)
    {
        if(!candidate.SequenceEqual(regenerated.CopyCanonicalBytes())) {throw PracticalFailures.Type("ordered_handoff_mismatch");}
    }
    private static void AddOperations(PracticalOrderedProjection p,List<PracticalOrderedOperation> operations)
    {
        string id=p.SemanticTypeId;
        void Add(string name,string result,string[] args,params string[] outcomes) =>
            operations.Add(new(id,name,Array.AsReadOnly(args),result,Array.AsReadOnly(outcomes)));
        Add("validate",PracticalIdentity.PrimitiveId("bool"),new[]{id});
        Add("count",PracticalIdentity.PrimitiveId("u32"),new[]{id});
        Add("contains",PracticalIdentity.PrimitiveId("bool"),new[]{id,p.KeyTypeId});
        Add("equal",PracticalIdentity.PrimitiveId("bool"),new[]{id,id});
        if(p.Comparable) {Add("compare",PracticalIdentity.PrimitiveId("i32"),new[]{id,id});}
        if(p.ValueTypeId.Length!=0) {
            Add("lookup",p.LookupTypeId,new[]{id,p.KeyTypeId});
            Add("add",id,new[]{id,p.KeyTypeId,p.ValueTypeId},"invalid_representation","duplicate_key","capacity");
            Add("replace",id,new[]{id,p.KeyTypeId,p.ValueTypeId},"invalid_representation","missing_key");
        } else {Add("add",id,new[]{id,p.KeyTypeId},"invalid_representation","duplicate_element","capacity");}
    }
    private static string SourceHash(string id,PracticalSequences sequences)
    {
        var closure=sequences.Arrays.Construction.Data.Syntax.SourceClosure;
        var declaration=closure.Declarations.Single(d=>d.Kind==PracticalDeclarationKind.Type && d.Id==id);
        return closure.Sources[declaration.SourceOrdinal].RawSha256;
    }
    private static string ValueId(PracticalNormalizedType type) => type.Nullability=="annotated"
        ? PracticalIdentity.ClosedInstanceId("option",type.Id) : type.Id;
    // Only an early source rejection. The shared Rust structural generator
    // independently decides key admissibility and conditional map comparison.
    private static bool Total(PracticalNormalizedType type,PracticalSequences sequences)
    {
        if(type.Arguments.Count!=0) {return type.Arguments.All(t=>Total(t,sequences));}
        if(type.Id.StartsWith("mpk.csharp.value.",StringComparison.Ordinal))
        {return type.Id is not ("mpk.csharp.value.f32.v1" or "mpk.csharp.value.f64.v1" or "mpk.csharp.value.exception.v1");}
        var source=sequences.Arrays.Construction.Data.Types.SingleOrDefault(t=>t.Id==type.Id);
        return source is not null && source.Members.Where(m=>m.Stored).All(m=>Total(m.Type,sequences));
    }
}
