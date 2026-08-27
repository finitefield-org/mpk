using System;
using System.Text.Json;

namespace Mpk.CSharp2Vir;

internal static class CSharpEmissionProfiles
{
    internal const string FrontendSchema = "mpk.frontend.cli.v1";
    internal const string VirSchema = "mpk.vir.v1";
    internal const string SourceMapSchema = "mpk.source_map.v1";
    internal const string SourceManifestSchema = "mpk.source_manifest.v1";
    internal const string ReleaseRegistrySchema = "mpk.release.bundle_registry.v1";
    internal const string VirHashDomain = "MPK-VIR-1.0";
    internal const string SourceMapHashDomain = "MPK-SOURCE-MAP-1.0";
    internal const string SourceManifestHashDomain = "MPK-SOURCE-MANIFEST-1.0";
    internal const string InputSetHashDomain = "MPK-INPUT-SET-0.1";

    internal const string ManifestContractId = "mpk.profile.manifest.csharp_scalar.v0";
    internal const string SourceMapContractId = "mpk.profile.source_map.csharp_scalar.v0";
    internal const string VirContractId = "mpk.profile.vir.csharp_scalar.v0";
    internal const string VirLimitProfileId = "mpk.vir.limits.v0";
    internal const string VirOperationProfileId = "mpk.csharp.vir_operations.v0";
    internal const string SourceMapProfileId = "mpk.csharp.source_map.v0";

    internal const string ReferenceInventorySha256 =
        "30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad";
    internal const string RuntimeArchiveSha256 =
        "7d847ecaa123efae40b114c5d45641e456b4cd65e5114b4612095d45d7c71a63";
    internal const string RoslynCommonSha256 =
        "42c9ce7891470f430267e2dc02d03571f9d046a7e7e121107754bee58d344613";
    internal const string RoslynCSharpSha256 =
        "1af1de8a162d2312eb2f6b781f5edbe8cec7d5cd268c7e4de24396225e54260f";

    internal static bool IsManifestInput(CapturedInputKind kind)
    {
        return kind == CapturedInputKind.Source || kind == CapturedInputKind.Contract;
    }

    internal static byte[] CanonicalOwnedContracts()
    {
        return EmissionCanonical.Write(writer =>
        {
            writer.WriteStartArray();
            WriteManifestContract(writer);
            WriteSourceMapContract(writer);
            WriteVirContract(writer);
            writer.WriteEndArray();
        });
    }

    private static void WriteManifestContract(Utf8JsonWriter writer)
    {
        WriteContractStart(writer, ManifestContractId);
        writer.WritePropertyName("input_kinds");
        writer.WriteStartArray();
        writer.WriteStringValue("contract");
        writer.WriteStringValue("source");
        writer.WriteEndArray();
        writer.WriteString("source_extension", ".cs");
        writer.WriteString("unit_kind", "compilation");
        WriteContractEnd(writer, "manifest");
    }

    private static void WriteSourceMapContract(Utf8JsonWriter writer)
    {
        WriteContractStart(writer, SourceMapContractId);
        writer.WriteString("encoding", "utf-8");
        writer.WriteString("offset_unit", "utf8-byte");
        writer.WritePropertyName("synthetic_reasons");
        writer.WriteStartArray();
        writer.WriteEndArray();
        WriteContractEnd(writer, "source_map");
    }

    private static void WriteVirContract(Utf8JsonWriter writer)
    {
        WriteContractStart(writer, VirContractId);
        writer.WriteString("operation_profile_id", VirOperationProfileId);
        writer.WriteString("source_map_profile_id", SourceMapProfileId);
        writer.WriteString("vir_limit_profile_id", VirLimitProfileId);
        WriteContractEnd(writer, "vir");
    }

    private static void WriteContractStart(Utf8JsonWriter writer, string contractId)
    {
        writer.WriteStartObject();
        writer.WritePropertyName("envelope");
        writer.WriteStartObject();
        writer.WriteString("contract_id", contractId);
        writer.WriteString("profile_entry_sha256", FrontendConstants.ProfileEntrySha256);
        writer.WritePropertyName("value");
        writer.WriteStartObject();
    }

    private static void WriteContractEnd(Utf8JsonWriter writer, string field)
    {
        writer.WriteEndObject();
        writer.WriteEndObject();
        writer.WriteString("field", field);
        writer.WriteEndObject();
    }
}
