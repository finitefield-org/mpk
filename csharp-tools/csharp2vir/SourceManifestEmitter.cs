using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;

namespace Mpk.CSharp2Vir;

internal static class CSharpSourceManifestEmitter
{
    internal static CanonicalArtifact Emit(
        LowerRequest request,
        Selection selection,
        CapturedSnapshot snapshot,
        CanonicalArtifact vir,
        CanonicalArtifact sourceMap)
    {
        if (!string.Equals(vir.Schema, CSharpEmissionProfiles.VirSchema, StringComparison.Ordinal)
            || !string.Equals(
                sourceMap.Schema,
                CSharpEmissionProfiles.SourceMapSchema,
                StringComparison.Ordinal)
            || !string.Equals(selection.Sha256, snapshot.Selection.Sha256, StringComparison.Ordinal))
        {
            throw EmissionFailure.Internal();
        }

        ManifestInput[] inputs = Inputs(snapshot);
        byte[] inputPayload = WriteInputs(inputs);
        string inputSetHash = EmissionCanonical.Hash(
            CSharpEmissionProfiles.InputSetHashDomain,
            inputPayload);
        byte[] payload = WriteManifest(
            request,
            selection,
            inputs,
            inputSetHash,
            vir,
            sourceMap,
            null);
        string hash = EmissionCanonical.Hash(
            CSharpEmissionProfiles.SourceManifestHashDomain,
            payload);
        byte[] canonical = WriteManifest(
            request,
            selection,
            inputs,
            inputSetHash,
            vir,
            sourceMap,
            hash);
        return new CanonicalArtifact(CSharpEmissionProfiles.SourceManifestSchema, hash, canonical);
    }

    private static ManifestInput[] Inputs(CapturedSnapshot snapshot)
    {
        var inputs = new ManifestInput[snapshot.Count];
        for (int index = 0; index < snapshot.Count; index++)
        {
            CapturedFile file = snapshot.FileAt(index);
            if (!CSharpEmissionProfiles.IsManifestInput(file.Kind))
            {
                throw EmissionFailure.Internal();
            }

            inputs[index] = new ManifestInput(
                file.Kind,
                file.NormalizedPath,
                file.SizeBytes,
                file.Sha256);
        }

        Array.Sort(inputs, (left, right) =>
        {
            int path = string.CompareOrdinal(left.NormalizedPath, right.NormalizedPath);
            return path != 0
                ? path
                : string.CompareOrdinal(InputKind(left.Kind), InputKind(right.Kind));
        });
        if (inputs.Length == 0
            || inputs.All(input => input.Kind != CapturedInputKind.Source)
            || inputs.Zip(inputs.Skip(1), (left, right) =>
                string.Equals(left.NormalizedPath, right.NormalizedPath, StringComparison.Ordinal))
                .Any(equal => equal))
        {
            throw EmissionFailure.Internal();
        }

        return inputs;
    }

    private static byte[] WriteInputs(IReadOnlyList<ManifestInput> inputs)
    {
        return EmissionCanonical.Write(writer =>
        {
            writer.WriteStartArray();
            foreach (ManifestInput input in inputs)
            {
                WriteInput(writer, input);
            }

            writer.WriteEndArray();
        });
    }

    private static byte[] WriteManifest(
        LowerRequest request,
        Selection selection,
        IReadOnlyList<ManifestInput> inputs,
        string inputSetHash,
        CanonicalArtifact vir,
        CanonicalArtifact sourceMap,
        string? hash)
    {
        return EmissionCanonical.Write(writer =>
        {
            writer.WriteStartObject();
            writer.WritePropertyName("frontend");
            WriteFrontend(writer, request.Release);
            writer.WriteString("input_set_hash", inputSetHash);
            writer.WritePropertyName("inputs");
            writer.WriteStartArray();
            foreach (ManifestInput input in inputs)
            {
                WriteInput(writer, input);
            }

            writer.WriteEndArray();
            writer.WriteString("limit_profile", CSharpEmissionProfiles.VirLimitProfileId);
            writer.WritePropertyName("release_registry");
            writer.WriteStartObject();
            writer.WriteString("id", FrontendConstants.ReleaseRegistryId);
            writer.WriteString("registry_sha256", request.Release.ReleaseRegistrySha256);
            writer.WriteString("schema", CSharpEmissionProfiles.ReleaseRegistrySchema);
            writer.WriteEndObject();
            writer.WriteString("schema", CSharpEmissionProfiles.SourceManifestSchema);
            writer.WritePropertyName("selection");
            EmissionCanonical.WriteSelection(writer, selection);
            EmissionCanonical.WriteSemanticContext(writer);
            if (hash is not null)
            {
                writer.WriteString("source_manifest_hash", hash);
            }

            writer.WriteString("source_map_hash", sourceMap.Sha256);
            writer.WritePropertyName("target");
            writer.WriteStartObject();
            writer.WriteString("id", FrontendConstants.TargetId);
            writer.WriteNumber("pointer_width", 64);
            writer.WriteEndObject();
            writer.WritePropertyName("toolchain");
            WriteToolchain(writer, request.Release);
            writer.WritePropertyName("units");
            writer.WriteStartArray();
            writer.WriteStartObject();
            writer.WriteString("identity", selection.Raw.Compilation);
            writer.WriteString("kind", "compilation");
            writer.WriteString("name", selection.Raw.Compilation);
            writer.WriteEndObject();
            writer.WriteEndArray();
            writer.WriteString("vir_hash", vir.Sha256);
            writer.WriteEndObject();
        }, "source_manifest_canonical_bytes", "emission");
    }

    private static void WriteFrontend(Utf8JsonWriter writer, ReleaseArguments release)
    {
        writer.WriteStartObject();
        writer.WriteString("binary_sha256", release.FrontendSha256);
        writer.WriteString("bundle_id", release.FrontendBundleId);
        writer.WriteString("name", "csharp2vir");
        writer.WritePropertyName("subordinate_binaries");
        writer.WriteStartArray();
        WriteSubordinate(
            writer,
            "Microsoft.CodeAnalysis.CSharp.dll",
            CSharpEmissionProfiles.RoslynCSharpSha256);
        WriteSubordinate(
            writer,
            "Microsoft.CodeAnalysis.dll",
            CSharpEmissionProfiles.RoslynCommonSha256);
        writer.WriteEndArray();
        writer.WriteString("version", "0.1.0");
        writer.WriteEndObject();
    }

    private static void WriteSubordinate(
        Utf8JsonWriter writer,
        string name,
        string sha256)
    {
        writer.WriteStartObject();
        writer.WriteString("binary_sha256", sha256);
        writer.WriteString("name", name);
        writer.WriteString("version", "5.6.0");
        writer.WriteEndObject();
    }

    private static void WriteToolchain(Utf8JsonWriter writer, ReleaseArguments release)
    {
        writer.WriteStartObject();
        writer.WriteString("bundle_id", release.ToolchainBundleId);
        writer.WritePropertyName("components");
        writer.WriteStartArray();
        WriteContentComponent(
            writer,
            "dotnet-runtime",
            "10.0.11",
            CSharpEmissionProfiles.RuntimeArchiveSha256);
        WriteContentComponent(
            writer,
            "microsoft-codeanalysis-common",
            "5.6.0",
            CSharpEmissionProfiles.RoslynCommonSha256);
        WriteContentComponent(
            writer,
            "microsoft-codeanalysis-csharp",
            "5.6.0",
            CSharpEmissionProfiles.RoslynCSharpSha256);
        WriteContentComponent(
            writer,
            "reference-pack",
            "10.0.11",
            CSharpEmissionProfiles.ReferenceInventorySha256);
        writer.WriteEndArray();
        writer.WriteString("distribution_sha256", release.ToolchainDistributionSha256);
        writer.WriteEndObject();
    }

    private static void WriteContentComponent(
        Utf8JsonWriter writer,
        string name,
        string release,
        string sha256)
    {
        writer.WriteStartObject();
        writer.WriteString("content_sha256", sha256);
        writer.WriteString("kind", "content");
        writer.WriteString("name", name);
        writer.WriteString("release", release);
        writer.WriteEndObject();
    }

    private static void WriteInput(Utf8JsonWriter writer, ManifestInput input)
    {
        writer.WriteStartObject();
        writer.WriteString("kind", InputKind(input.Kind));
        writer.WriteString("normalized_path", input.NormalizedPath);
        writer.WriteString("sha256", input.Sha256);
        writer.WriteNumber("size_bytes", input.SizeBytes);
        writer.WriteEndObject();
    }

    private static string InputKind(CapturedInputKind kind)
    {
        return kind switch
        {
            CapturedInputKind.Source => "source",
            CapturedInputKind.Contract => "contract",
            _ => throw EmissionFailure.Internal(),
        };
    }

    private sealed class ManifestInput
    {
        internal ManifestInput(
            CapturedInputKind kind,
            string normalizedPath,
            int sizeBytes,
            string sha256)
        {
            Kind = kind;
            NormalizedPath = normalizedPath;
            SizeBytes = sizeBytes;
            Sha256 = sha256;
        }

        internal CapturedInputKind Kind { get; }

        internal string NormalizedPath { get; }

        internal int SizeBytes { get; }

        internal string Sha256 { get; }
    }
}
