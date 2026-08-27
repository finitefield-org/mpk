using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Text;

namespace Mpk.CSharp2Vir;

internal enum FrontendLimitDisposition
{
    Rejected,
    DiagnosticBudget,
    Output,
}

internal sealed class FrontendLimitDefinition
{
    internal FrontendLimitDefinition(
        string id,
        ulong maximum,
        string code,
        FrontendLimitDisposition disposition)
    {
        Id = id;
        Maximum = maximum;
        Code = code;
        Disposition = disposition;
    }

    internal string Id { get; }

    internal ulong Maximum { get; }

    internal string Code { get; }

    internal FrontendLimitDisposition Disposition { get; }
}

internal static class FrontendLimits
{
    internal const int SourceFilesMaximum = 256;
    internal const int SourceFileBytesMaximum = 1_048_576;
    internal const int SourceTotalBytesMaximum = 16_777_216;
    internal const int ContractFilesMaximum = 128;
    internal const int ContractFileBytesMaximum = 1_048_576;
    internal const int ContractTotalBytesMaximum = 8_388_608;
    internal const int SnapshotEntriesMaximum = 512;
    internal const int SnapshotTotalBytesMaximum = 33_554_432;
    internal const int NormalizedPathBytesMaximum = 1_024;
    internal const int CanonicalMethodIdBytesMaximum = 1_024;
    internal const int SelectedMethodsMaximum = 32;
    internal const uint MethodClosureMaximum = 128;
    internal const uint SyntaxNodesMaximum = 250_000;
    internal const uint OperationsPerMethodMaximum = 100_000;
    internal const uint OperationsPerClosureMaximum = 250_000;
    internal const uint CfgBlocksPerMethodMaximum = 1_024;
    internal const uint CfgBlocksPerClosureMaximum = 8_192;
    internal const uint ContractClausesMaximum = 64;
    internal const uint ContractNodesPerMethodMaximum = 1_024;
    internal const uint ContractNodesPerClosureMaximum = 8_192;
    internal const uint ContractDepthMaximum = 32;
    internal const int NormalizedIssuesMaximum = 1_024;
    internal const int DiagnosticMessageBytesEachMaximum = 4_096;
    internal const int DiagnosticMessageBytesTotalMaximum = 2_097_152;
    internal const int FrontendArgumentBytesMaximum = 131_072;
    internal const int PrivateRuntimeStdoutMaximum = 268_435_456;
    internal const int PrivateRuntimeStderrMaximum = 2_097_152;
    internal const int VirCanonicalBytesMaximum = 201_326_592;
    internal const int SourceMapCanonicalBytesMaximum = 33_554_432;
    internal const int SourceManifestCanonicalBytesMaximum = 4_194_304;
    internal const int FrontendStdoutMaximum = 268_435_456;
    internal const int FrontendStderrMaximum = 2_097_152;

    private static readonly UTF8Encoding StrictUtf8 = new UTF8Encoding(false, true);
    private static readonly ReadOnlyCollection<FrontendLimitDefinition> definitions =
        Array.AsReadOnly(new[]
        {
            Rejected("source_files", SourceFilesMaximum, "CSHARP_LIMIT_SOURCE_FILES"),
            Rejected("source_file_bytes", SourceFileBytesMaximum, "CSHARP_LIMIT_SOURCE_FILE_BYTES"),
            Rejected("source_total_bytes", SourceTotalBytesMaximum, "CSHARP_LIMIT_SOURCE_TOTAL_BYTES"),
            Rejected("contract_files", ContractFilesMaximum, "CSHARP_LIMIT_CONTRACT_FILES"),
            Rejected("contract_file_bytes", ContractFileBytesMaximum, "CSHARP_LIMIT_CONTRACT_FILE_BYTES"),
            Rejected("contract_total_bytes", ContractTotalBytesMaximum, "CSHARP_LIMIT_CONTRACT_TOTAL_BYTES"),
            Rejected("snapshot_entries", SnapshotEntriesMaximum, "CSHARP_LIMIT_SNAPSHOT_ENTRIES"),
            Rejected("snapshot_total_bytes", SnapshotTotalBytesMaximum, "CSHARP_LIMIT_SNAPSHOT_TOTAL_BYTES"),
            Rejected("normalized_path_bytes", NormalizedPathBytesMaximum, "CSHARP_LIMIT_NORMALIZED_PATH_BYTES"),
            Rejected("canonical_method_id_bytes", CanonicalMethodIdBytesMaximum, "CSHARP_LIMIT_CANONICAL_METHOD_ID_BYTES"),
            Rejected("selected_methods", SelectedMethodsMaximum, "CSHARP_LIMIT_SELECTED_METHODS"),
            Rejected("method_closure", MethodClosureMaximum, "CSHARP_LIMIT_METHOD_CLOSURE"),
            Rejected("syntax_nodes", SyntaxNodesMaximum, "CSHARP_LIMIT_SYNTAX_NODES"),
            Rejected("operations_per_method", OperationsPerMethodMaximum, "CSHARP_LIMIT_OPERATIONS_PER_METHOD"),
            Rejected("operations_per_closure", OperationsPerClosureMaximum, "CSHARP_LIMIT_OPERATIONS_PER_CLOSURE"),
            Rejected("cfg_blocks_per_method", CfgBlocksPerMethodMaximum, "CSHARP_LIMIT_CFG_BLOCKS_PER_METHOD"),
            Rejected("cfg_blocks_per_closure", CfgBlocksPerClosureMaximum, "CSHARP_LIMIT_CFG_BLOCKS_PER_CLOSURE"),
            Rejected("contract_clauses", ContractClausesMaximum, "CSHARP_LIMIT_CONTRACT_CLAUSES"),
            Rejected("contract_nodes_per_method", ContractNodesPerMethodMaximum, "CSHARP_LIMIT_CONTRACT_NODES_PER_METHOD"),
            Rejected("contract_nodes_per_closure", ContractNodesPerClosureMaximum, "CSHARP_LIMIT_CONTRACT_NODES_PER_CLOSURE"),
            Rejected("contract_depth", ContractDepthMaximum, "CSHARP_LIMIT_CONTRACT_DEPTH"),
            Operational("normalized_issues", NormalizedIssuesMaximum, FrontendLimitDisposition.DiagnosticBudget),
            Operational("diagnostic_message_bytes_each", DiagnosticMessageBytesEachMaximum, FrontendLimitDisposition.DiagnosticBudget),
            Operational("diagnostic_message_bytes_total", DiagnosticMessageBytesTotalMaximum, FrontendLimitDisposition.DiagnosticBudget),
            Rejected("frontend_argument_bytes", FrontendArgumentBytesMaximum, "CSHARP_LIMIT_FRONTEND_ARGUMENT_BYTES"),
            Operational("private_runtime_stdout", PrivateRuntimeStdoutMaximum, FrontendLimitDisposition.Output),
            Operational("private_runtime_stderr", PrivateRuntimeStderrMaximum, FrontendLimitDisposition.Output),
            Rejected("vir_canonical_bytes", VirCanonicalBytesMaximum, "CSHARP_LIMIT_VIR_CANONICAL_BYTES"),
            Rejected("source_map_canonical_bytes", SourceMapCanonicalBytesMaximum, "CSHARP_LIMIT_SOURCE_MAP_CANONICAL_BYTES"),
            Rejected("source_manifest_canonical_bytes", SourceManifestCanonicalBytesMaximum, "CSHARP_LIMIT_SOURCE_MANIFEST_CANONICAL_BYTES"),
            Operational("frontend_stdout", FrontendStdoutMaximum, FrontendLimitDisposition.Output),
            Operational("frontend_stderr", FrontendStderrMaximum, FrontendLimitDisposition.Output),
        });

    internal static IReadOnlyList<FrontendLimitDefinition> Definitions => definitions;

    internal static FrontendLimitDefinition Find(string id)
    {
        foreach (FrontendLimitDefinition definition in definitions)
        {
            if (string.Equals(definition.Id, id, StringComparison.Ordinal))
            {
                return definition;
            }
        }

        throw new InvalidOperationException("unknown C# frontend limit");
    }

    internal static ulong Validate(string id, ulong observed, string phase)
    {
        FrontendLimitDefinition definition = Find(id);
        if (observed > definition.Maximum)
        {
            throw Failure(definition, phase);
        }

        return observed;
    }

    internal static ulong Add(string id, ulong current, ulong increment, string phase)
    {
        ulong observed;
        try
        {
            observed = checked(current + increment);
        }
        catch (OverflowException)
        {
            throw Failure(Find(id), phase);
        }

        return Validate(id, observed, phase);
    }

    internal static void ValidateArguments(IReadOnlyList<string> arguments)
    {
        ulong observed = 0;
        try
        {
            for (int index = 0; index < arguments.Count; index++)
            {
                observed = Add(
                    "frontend_argument_bytes",
                    observed,
                    checked((ulong)StrictUtf8.GetByteCount(arguments[index]) + 1),
                    "capture");
            }
        }
        catch (FrontendFailure)
        {
            throw;
        }
        catch (Exception error) when (error is EncoderFallbackException || error is OverflowException)
        {
            throw FrontendFailure.Internal("capture");
        }
    }

    internal static FrontendFailure Failure(string id, string phase)
    {
        return Failure(Find(id), phase);
    }

    private static FrontendLimitDefinition Rejected(string id, ulong maximum, string code)
    {
        return new FrontendLimitDefinition(id, maximum, code, FrontendLimitDisposition.Rejected);
    }

    private static FrontendLimitDefinition Operational(
        string id,
        ulong maximum,
        FrontendLimitDisposition disposition)
    {
        string code = disposition == FrontendLimitDisposition.DiagnosticBudget
            ? "CSHARP_FRONTEND_DIAGNOSTIC_BUDGET"
            : "CSHARP_FRONTEND_OUTPUT_LIMIT";
        return new FrontendLimitDefinition(id, maximum, code, disposition);
    }

    private static FrontendFailure Failure(FrontendLimitDefinition definition, string phase)
    {
        return definition.Disposition == FrontendLimitDisposition.Rejected
            ? FrontendFailure.Rejected(phase, definition.Code)
            : FrontendFailure.Toolchain(phase, definition.Code);
    }
}

internal sealed class BoundedMemoryStream : Stream
{
    private readonly MemoryStream output = new MemoryStream();
    private readonly ulong maximum;
    private readonly string limitId;
    private readonly string phase;

    internal BoundedMemoryStream(string limitId, string phase)
    {
        FrontendLimitDefinition definition = FrontendLimits.Find(limitId);
        maximum = definition.Maximum;
        this.limitId = limitId;
        this.phase = phase;
    }

    internal byte[] ToArray()
    {
        return output.ToArray();
    }

    public override bool CanRead => false;

    public override bool CanSeek => false;

    public override bool CanWrite => true;

    public override long Length => output.Length;

    public override long Position
    {
        get => output.Position;
        set => throw new NotSupportedException();
    }

    public override void Flush()
    {
        output.Flush();
    }

    public override void Write(byte[] buffer, int offset, int count)
    {
        ValidateAppend(count);
        output.Write(buffer, offset, count);
    }

    public override void Write(ReadOnlySpan<byte> buffer)
    {
        ValidateAppend(buffer.Length);
        output.Write(buffer);
    }

    public override void WriteByte(byte value)
    {
        ValidateAppend(1);
        output.WriteByte(value);
    }

    public override int Read(byte[] buffer, int offset, int count)
    {
        throw new NotSupportedException();
    }

    public override long Seek(long offset, SeekOrigin origin)
    {
        throw new NotSupportedException();
    }

    public override void SetLength(long value)
    {
        throw new NotSupportedException();
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing)
        {
            output.Dispose();
        }

        base.Dispose(disposing);
    }

    private void ValidateAppend(int count)
    {
        if (count < 0 || output.Position != output.Length)
        {
            throw FrontendFailure.Internal(phase);
        }

        ulong projected;
        try
        {
            projected = checked((ulong)output.Length + (ulong)count);
        }
        catch (OverflowException)
        {
            throw FrontendLimits.Failure(limitId, phase);
        }

        if (projected > maximum)
        {
            throw FrontendLimits.Failure(limitId, phase);
        }
    }
}
