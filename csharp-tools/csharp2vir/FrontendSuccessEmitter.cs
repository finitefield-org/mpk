using System;
using System.Text.Json;

namespace Mpk.CSharp2Vir;

internal static class CSharpFrontendSuccessEmitter
{
    internal static EmittedFrontendSuccess Emit(
        LowerRequest request,
        Selection selection,
        CapturedSnapshot snapshot,
        CapturedSourceSet sources,
        RoslynCompilationSession compilation,
        SubsetClosure closure,
        ContractSet contracts,
        LoweredClosure lowered)
    {
        ValidateInputs(request, selection, snapshot, closure, contracts, lowered);
        CanonicalArtifact vir = CSharpVirEmitter.Emit(selection, lowered, contracts);
        CanonicalArtifact sourceMap = CSharpSourceMapEmitter.Emit(
            selection,
            lowered,
            compilation,
            sources,
            vir);
        CanonicalArtifact sourceManifest = CSharpSourceManifestEmitter.Emit(
            request,
            selection,
            snapshot,
            vir,
            sourceMap);
        byte[] envelope = WriteEnvelope(selection, vir, sourceMap, sourceManifest);
        FrontendLimits.Validate(
            "frontend_stdout",
            checked((ulong)envelope.Length + 1),
            "emission");
        var transport = new byte[checked(envelope.Length + 1)];
        Buffer.BlockCopy(envelope, 0, transport, 0, envelope.Length);
        transport[^1] = (byte)'\n';
        return new EmittedFrontendSuccess(vir, sourceMap, sourceManifest, transport);
    }

    private static void ValidateInputs(
        LowerRequest request,
        Selection selection,
        CapturedSnapshot snapshot,
        SubsetClosure closure,
        ContractSet contracts,
        LoweredClosure lowered)
    {
        if (!string.Equals(request.RawSelection.Compilation, selection.Raw.Compilation, StringComparison.Ordinal)
            || !string.Equals(snapshot.Selection.Sha256, selection.Sha256, StringComparison.Ordinal)
            || !string.Equals(contracts.SelectionSha256, selection.Sha256, StringComparison.Ordinal)
            || !string.Equals(lowered.SelectionSha256, selection.Sha256, StringComparison.Ordinal)
            || closure.Methods.Length != lowered.Functions.Count)
        {
            throw EmissionFailure.Internal();
        }

        for (int index = 0; index < closure.Methods.Length; index++)
        {
            if (!string.Equals(
                closure.Methods[index].CanonicalId,
                lowered.Functions[index].Id,
                StringComparison.Ordinal))
            {
                throw EmissionFailure.Internal();
            }
        }
    }

    private static byte[] WriteEnvelope(
        Selection selection,
        CanonicalArtifact vir,
        CanonicalArtifact sourceMap,
        CanonicalArtifact sourceManifest)
    {
        return EmissionCanonical.Write(writer =>
        {
            writer.WriteStartObject();
            writer.WritePropertyName("diagnostics");
            writer.WriteStartArray();
            writer.WriteEndArray();
            writer.WritePropertyName("ir");
            writer.WriteStartObject();
            writer.WriteString("schema", vir.Schema);
            writer.WriteString("sha256", vir.Sha256);
            writer.WritePropertyName("value");
            EmissionCanonical.WriteRaw(writer, vir.CanonicalBytes);
            writer.WriteEndObject();
            writer.WriteString("phase", "emission");
            writer.WritePropertyName("rejected_features");
            writer.WriteStartArray();
            writer.WriteEndArray();
            writer.WriteString("schema", CSharpEmissionProfiles.FrontendSchema);
            writer.WritePropertyName("selection");
            EmissionCanonical.WriteSelection(writer, selection);
            EmissionCanonical.WriteSemanticContext(writer);
            writer.WritePropertyName("source_manifest");
            EmissionCanonical.WriteRaw(writer, sourceManifest.CanonicalBytes);
            writer.WritePropertyName("source_map");
            EmissionCanonical.WriteRaw(writer, sourceMap.CanonicalBytes);
            writer.WriteString("status", "ir-lowered");
            writer.WriteEndObject();
        }, "frontend_stdout", "emission");
    }
}
