using System;
using System.Collections.Generic;
using System.Linq;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using System.Text.Json;

namespace Mpk.CSharp2Vir;

// W06's source-to-T02 type projection. This contains no equality/order rules:
// all recursive semantic specialization belongs to generate_structural_program.
// Source helpers retain their captured bodies; this never replaces a helper
// with structural equality or claims field-completeness without a proof.
internal static class CSharpPracticalStructural
{
    internal static PracticalConstruction Validate(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs, ImmutableArray<MetadataReference> references,
        IReadOnlyList<PracticalTypeInvariantClaim>? invariantClaims = null) =>
        CSharpPracticalConstruction.Validate(selection, inputs, references, invariantClaims,
            allowInitializers: true, allowStructuralEquality: true);

    internal static object TypeDescriptor(PracticalNormalizedType type)
    {
        object descriptor;
        if (type.Arguments.Count != 0)
        {
            if (type.Arguments.Count != 1) { throw PracticalFailures.Type("structural_type"); }
            string? template = new[] { "option", "bounded_sequence" }.SingleOrDefault(name =>
                PracticalIdentity.ClosedInstanceId(name, type.Arguments[0].Id) == type.Id);
            if (template is null) { throw PracticalFailures.Type("structural_type"); }
            descriptor = new SortedDictionary<string, object>(StringComparer.Ordinal) {
                ["arguments"] = type.Arguments.Select(TypeDescriptor).ToArray(),
                ["kind"] = "instance", ["template"] = template,
            };
        }
        else
        {
            bool source = type.Id.StartsWith("mpk.csharp.source.", StringComparison.Ordinal);
            string id = source ? type.Id : type.Id["mpk.csharp.value.".Length..^3];
            descriptor = new SortedDictionary<string, object>(StringComparer.Ordinal) {
                ["id"] = id, ["kind"] = source ? "source" : "primitive",
            };
        }
        // Annotated references carry presence separately from their value.
        return type.Nullability == "annotated"
            ? new SortedDictionary<string, object>(StringComparer.Ordinal) {
                ["arguments"] = new[] { descriptor }, ["kind"] = "instance", ["template"] = "option",
              }
            : descriptor;
    }

    internal static byte[] CopyTypeProjection(PracticalConstruction construction) =>
        JsonSerializer.SerializeToUtf8Bytes(construction.Data.Types.OrderBy(type => type.Id, StringComparer.Ordinal)
            .Select(type => new SortedDictionary<string, object>(StringComparer.Ordinal) {
                ["carrier"] = type.Carrier.Length == 0 ? "" : type.Carrier["mpk.csharp.value.".Length..^3],
                ["enum_values"] = type.EnumMembers.Select(member => member.Value).Distinct(StringComparer.Ordinal).ToArray(),
                ["id"] = type.Id, ["kind"] = type.Kind,
                ["members"] = type.Members.Where(member => member.Stored).Select(member =>
                    new SortedDictionary<string, object>(StringComparer.Ordinal) {
                        ["name"] = member.Name, ["type"] = TypeDescriptor(member.Type),
                    }).ToArray(),
            }).ToArray());
}
