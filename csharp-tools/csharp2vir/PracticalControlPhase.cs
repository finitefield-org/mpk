using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Nodes;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
namespace Mpk.CSharp2Vir;
// The data and control owners consume the same captured bytes. This private
// handoff is input to native validation, never a frontend success artifact.
internal static class CSharpPracticalControlPhase
{
    internal static PracticalDataSource Capture(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> supplied, ImmutableArray<MetadataReference> references,
        IReadOnlyList<string>? totalGetters = null)
    {
        var inputs=supplied.ToArray();
        PracticalBusiness? business=null;CSharpCompilation? compilation=null;
        var data=CSharpPracticalDataPhase.CaptureSelected(selection,inputs,references,allowControl:true,
            validatedSource:(b,c)=>{business=b;compilation=c;});
        var arrays=business!.Domain.Numeric.Strings.Arrays;
        var sequence=CSharpPracticalSequences.FromValidatedArrays(arrays);
        var facts=JsonSerializer.Deserialize<JsonElement>(CSharpPracticalLoopContracts.CaptureValidated(
            selection,arrays.Construction.Data.Syntax.SourceClosure,compilation!,true,true));
        var control=CSharpPracticalLoopLowering.CaptureValidated(sequence,facts,compilation!,true,
            totalGetters??Array.Empty<string>(),true,true,ssaConditions:true);
        var document=JsonNode.Parse(data.CopyBytes())!;
        document["control_lowering"]=JsonNode.Parse(control);
        return new PracticalDataSource(JsonSerializer.SerializeToUtf8Bytes(document));
    }
}
