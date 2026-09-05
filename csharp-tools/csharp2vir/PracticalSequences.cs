using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using System.Text.Json;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

// Private typed handoff to T04/T06. A plan is an unproved candidate, never a
// source API or a certificate. Roslyn operands remain attached to ordered steps.
internal sealed record PracticalSequenceBinding(string SourceTypeId, string ElementsMember);
internal sealed record PracticalSequenceProjection(string SourceTypeId, string ElementsMember,
    string ElementTypeId, string SequenceTypeId);
internal sealed record PracticalSequenceType(string ArrayId, string ElementTypeId,
    string ConstructionTypeId, string SequenceTypeId);
internal sealed record PracticalSequenceStep(PracticalArrayStep Source, string Action,
    IReadOnlyList<PracticalSequenceType> Types, int? InitializerIndex);
internal sealed record PracticalSequences(PracticalArrays Arrays,
    IReadOnlyList<PracticalSequenceProjection> Projections, IReadOnlyList<PracticalSequenceStep> Steps)
{
    internal int ArtifactCount => 0;
    internal byte[] CopyCanonicalBytes() => JsonSerializer.SerializeToUtf8Bytes(new {
        source = Arrays.Construction.Data.Syntax.SemanticSha256,
        projections = Projections,
        steps = Steps.Select(step => new {
            site = step.Source.Site, path = step.Source.Path, method = step.Source.Method, operation = step.Source.Operation,
            action = step.Action, types = step.Types, initializer_index = step.InitializerIndex,
            arrays = step.Source.Arrays, predicate = step.Source.Predicate, exception = step.Source.Exception,
            operand = step.Source.Operand is null ? "" : step.Source.Operand.Syntax.SyntaxTree.FilePath + ":"
                + step.Source.Operand.Syntax.SpanStart.ToString(System.Globalization.CultureInfo.InvariantCulture)
                + ":" + step.Source.Operand.Kind,
        }),
    });
}

internal static class CSharpPracticalSequences
{
    internal const string TwoPassOwner = "CSHARP-03-T04-W01/W02";
    internal const int StatesPerMethodMaximum = 32;
    internal const int LiveStatesMaximum = 8;
    internal const int CapacityMaximum = 16384;

    internal static PracticalSequences Validate(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs, ImmutableArray<MetadataReference> references,
        IReadOnlyList<PracticalSequenceBinding>? bindings = null,
        IReadOnlyList<PracticalTypeInvariantClaim>? invariantClaims = null)
    {
        PracticalArrays arrays = CSharpPracticalArrays.Validate(selection, inputs, references, invariantClaims, sequenceConstruction:true);
        var projections = new List<PracticalSequenceProjection>();
        foreach (PracticalSequenceBinding binding in bindings ?? Array.Empty<PracticalSequenceBinding>()) {
            PracticalDataType? type = arrays.Construction.Data.Types.SingleOrDefault(t => t.Id == binding.SourceTypeId);
            if (type is null || type.Kind == "enum" || projections.Any(p => p.SourceTypeId == type.Id))
            { throw PracticalFailures.Type("sequence_wrapper_binding"); }
            PracticalDataMember[] stored = type.Members.Where(m => m.Stored).ToArray();
            PracticalDataMember[] sequences = stored.Where(m => IsSequence(m.Type)).ToArray();
            if (sequences.Length != 1 || sequences[0].Name != binding.ElementsMember
                || sequences[0].Type.Nullability == "annotated"
                || stored.Where(m => m != sequences[0]).Any(m => !IsScalar(m.Type, arrays)))
            { throw PracticalFailures.Type("sequence_wrapper_shape"); }
            string element = ValueId(sequences[0].Type.Arguments[0]);
            projections.Add(new(type.Id, binding.ElementsMember, element,
                PracticalIdentity.ClosedInstanceId("bounded_sequence", element)));
        }
        var types = new Dictionary<string, PracticalSequenceType>(StringComparer.Ordinal);
        var initializers = new Dictionary<string, int>(StringComparer.Ordinal);
        var steps = new List<PracticalSequenceStep>();
        var totals = new Dictionary<string, int>(StringComparer.Ordinal);
        var live = new Dictionary<string, HashSet<string>>(StringComparer.Ordinal);
        foreach (PracticalArrayStep step in arrays.Steps) {
            string method = step.Method;
            if (!live.TryGetValue(step.Path, out var owners)) {
                string? parent = live.Keys.Where(key => step.Path.StartsWith(key + "/", StringComparison.Ordinal))
                    .OrderByDescending(key => key.Length).FirstOrDefault();
                owners = parent is null ? new(StringComparer.Ordinal) : new(live[parent], StringComparer.Ordinal);
                live.Add(step.Path, owners);
            }
            if (step.Operation == "merge") {
                var branches = live.Where(pair => pair.Key.StartsWith(step.Path + "/" + step.Site + ":", StringComparison.Ordinal)).ToArray();
                if (step.Source is IConditionalOperation { WhenFalse: not null }
                    && branches.Any(b => b.Key.EndsWith(":true", StringComparison.Ordinal))
                    && branches.Any(b => b.Key.EndsWith(":false", StringComparison.Ordinal))) { owners.Clear(); }
                foreach (var branch in branches) {
                    // Keep every possibly live owner at a join. CFG compatibility,
                    // versions and initialized-set joins are revalidated by VIR.
                    owners.UnionWith(branch.Value);
                    live.Remove(branch.Key);
                }
            }
            if (step.ElementType is not null) {
                foreach (string id in step.Arrays.Where(id => !types.ContainsKey(id))) {
                    string element = ValueId(step.ElementType);
                    types.Add(id, new(id, element, "", PracticalIdentity.ClosedInstanceId("bounded_sequence", element)));
                }
            }
            if (step.Operation == "allocate_unique") {
                string id = step.Arrays.Single();
                string element = ValueId(step.ElementType ?? throw PracticalFailures.Protocol("sequence_element_type"));
                types[id] = new(id, element, PracticalIdentity.ClosedInstanceId("sequence_construction", element),
                    PracticalIdentity.ClosedInstanceId("bounded_sequence", element));
                initializers.Add(id, 0);
                totals.TryGetValue(method, out int count); totals[method] = checked(count + 1);
                if (totals[method] > StatesPerMethodMaximum) { throw PracticalFailures.Limit("construction_states_per_method"); }
                owners.Add(id);
                if (owners.Count > LiveStatesMaximum) { throw PracticalFailures.Limit("simultaneously_live_construction_states"); }
            }
            string action = step.Operation switch {
                "allocate_unique" => "allocate",
                "initialize_element" => "fill_initializer",
                "functional_update" => "fill_or_rewrite",
                "read" => "indexed_read", "length" => "length",
                "alias_freeze" or "storage_freeze" or "call_freeze" or "wrapper_freeze" or "return_transfer" => "freeze",
                "discard_on_exit" or "discard_exception" => "discard",
                "merge" => "merge", _ => "ordered_check_or_evaluation",
            };
            int? index = step.Operation == "initialize_element" ? initializers[step.Arrays.Single()]++ : null;
            var operandTypes = step.Arrays.Where(types.ContainsKey).Select(id => owners.Contains(id)
                ? types[id] : types[id] with { ConstructionTypeId = "" }).ToArray();
            if (action == "freeze" && !step.Arrays.Any(owners.Contains)) { action = "publish_immutable"; }
            if (action == "indexed_read" && step.Arrays.Any(owners.Contains)) { action = "construction_read"; }
            if (action == "length" && step.Arrays.Any(owners.Contains)) { action = "construction_length"; }
            if (action is "freeze" or "discard") { owners.ExceptWith(step.Arrays); }
            if (owners.Count > LiveStatesMaximum) { throw PracticalFailures.Limit("simultaneously_live_construction_states"); }
            steps.Add(new(step, action, Array.AsReadOnly(operandTypes), index));
        }
        return new(arrays, Array.AsReadOnly(projections.OrderBy(p => p.SourceTypeId,StringComparer.Ordinal).ToArray()),
            Array.AsReadOnly(steps.ToArray()));
    }

    // Importers must regenerate from captured source and sidecar selection.
    // Matching only a caller-supplied hash would trust modified ownership/checks.
    internal static void ValidateCandidate(PracticalSequences regenerated, ReadOnlySpan<byte> candidate)
    {
        if (!candidate.SequenceEqual(regenerated.CopyCanonicalBytes()))
        { throw PracticalFailures.Type("sequence_handoff_mismatch"); }
    }
    private static bool IsScalar(PracticalNormalizedType type, PracticalArrays arrays) => type.Arguments.Count == 0
        ? type.Id.StartsWith("mpk.csharp.value.", StringComparison.Ordinal)
            || arrays.Construction.Data.Types.Any(t => t.Id == type.Id && t.Kind == "enum")
        : type.Arguments.Count == 1 && type.Id == PracticalIdentity.ClosedInstanceId("option", type.Arguments[0].Id)
            && IsScalar(type.Arguments[0], arrays);
    private static bool IsSequence(PracticalNormalizedType type) => type.Arguments.Count == 1
        && type.Id == PracticalIdentity.ClosedInstanceId("bounded_sequence", type.Arguments[0].Id);
    private static string ValueId(PracticalNormalizedType type) => type.Nullability == "annotated"
        ? PracticalIdentity.ClosedInstanceId("option", type.Id) : type.Id;
}
