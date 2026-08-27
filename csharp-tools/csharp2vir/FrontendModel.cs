using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;

namespace Mpk.CSharp2Vir;

internal static class FrontendConstants
{
    internal const string SemanticProfile = "mpk.csharp.scalar.v0";
    internal const string SemanticParametersSchema = "mpk.semantic_parameters.csharp_scalar.v0";
    internal const string SelectionSchema = "mpk.selection.csharp_methods.v0";
    internal const string ContractSchema = "mpk.csharp.contract.v0";
    internal const string ProfileRegistryId = "mpk.semantic_profile.registry.v1";
    internal const int ProfileRegistryRevision = 2;
    internal const string ProfileRegistrySha256 = "6928e49ab2d0af03bdc1b92c189f99308f815e77edb3850a5f5a8fd9a3d48b75";
    internal const string ProfileEntrySha256 = "d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac";
    internal const string ReleaseRegistryId = "mpk.release.registry.v1";
    internal const string SourceRoot = "/mpk/source";
    internal const string ToolchainRoot = "/mpk/toolchain";
    internal const string TargetId = "linux-x64";
}

internal enum FrontendStatus
{
    Rejected,
    SourceError,
    FrontendError,
}

internal sealed class FrontendFailure : Exception
{
    private readonly ReadOnlyCollection<FrontendIssue> issues;

    private FrontendFailure(
        FrontendStatus status,
        string phase,
        string code,
        IReadOnlyList<FrontendIssue>? issues = null)
        : base(code)
    {
        FrontendDiagnosticRegistry.Validate(code, status, phase);
        Status = status;
        Phase = phase;
        Code = code;
        FrontendIssue[] retained = issues is null
            ? new[]
            {
                new FrontendIssue(
                    code,
                    FrontendDiagnosticRegistry.PublicMessage(status, code)),
            }
            : CopyIssues(issues);
        if (retained.Length == 0)
        {
            throw new InvalidOperationException("C# frontend failure requires an issue");
        }

        foreach (FrontendIssue issue in retained)
        {
            if (!string.Equals(issue.Code, code, StringComparison.Ordinal))
            {
                throw new InvalidOperationException("C# frontend failure issue code mismatch");
            }

            FrontendDiagnosticRegistry.ValidateIssue(issue, status, phase);
        }

        this.issues = Array.AsReadOnly(retained);
    }

    internal FrontendStatus Status { get; }

    internal string Phase { get; }

    internal string Code { get; }

    internal IReadOnlyList<FrontendIssue> Issues => issues;

    internal int ExitCode => Status switch
    {
        FrontendStatus.FrontendError => 1,
        FrontendStatus.Rejected => 3,
        FrontendStatus.SourceError => 4,
        _ => 1,
    };

    internal static FrontendFailure Rejected(string phase, string code)
    {
        return new FrontendFailure(FrontendStatus.Rejected, phase, code);
    }

    internal static FrontendFailure SourceError(string code)
    {
        return new FrontendFailure(FrontendStatus.SourceError, "source", code);
    }

    internal static FrontendFailure SourceError(string phase, string code)
    {
        return new FrontendFailure(FrontendStatus.SourceError, phase, code);
    }

    internal static FrontendFailure Toolchain(string phase, string code)
    {
        return new FrontendFailure(FrontendStatus.FrontendError, phase, code);
    }

    internal static FrontendFailure Internal(string phase)
    {
        return new FrontendFailure(FrontendStatus.FrontendError, phase, "CSHARP_FRONTEND_INTERNAL");
    }

    internal static FrontendFailure WithIssues(
        FrontendStatus status,
        string phase,
        string code,
        IReadOnlyList<FrontendIssue> issues)
    {
        return new FrontendFailure(status, phase, code, issues);
    }

    private static FrontendIssue[] CopyIssues(IReadOnlyList<FrontendIssue> values)
    {
        var result = new FrontendIssue[values.Count];
        for (int index = 0; index < values.Count; index++)
        {
            result[index] = values[index]
                ?? throw new InvalidOperationException("null C# frontend issue");
        }

        return result;
    }
}

internal sealed class ReleaseArguments
{
    internal ReleaseArguments(
        string frontendBundleId,
        string frontendSha256,
        string releaseRegistrySha256,
        string toolchainBundleId,
        string toolchainDistributionSha256)
    {
        FrontendBundleId = frontendBundleId;
        FrontendSha256 = frontendSha256;
        ReleaseRegistrySha256 = releaseRegistrySha256;
        ToolchainBundleId = toolchainBundleId;
        ToolchainDistributionSha256 = toolchainDistributionSha256;
    }

    internal string FrontendBundleId { get; }

    internal string FrontendSha256 { get; }

    internal string ReleaseRegistrySha256 { get; }

    internal string ToolchainBundleId { get; }

    internal string ToolchainDistributionSha256 { get; }
}

internal sealed class RawSelection
{
    private readonly ReadOnlyCollection<string> sources;
    private readonly ReadOnlyCollection<string> contracts;
    private readonly ReadOnlyCollection<string> methods;

    internal RawSelection(
        string compilation,
        IReadOnlyList<string> sources,
        IReadOnlyList<string> contracts,
        IReadOnlyList<string> methods)
    {
        Compilation = compilation;
        this.sources = Array.AsReadOnly(CopyWithinLimit(
            sources,
            SelectionCodec.SourceFilesMaximum,
            "CSHARP_LIMIT_SOURCE_FILES"));
        this.contracts = Array.AsReadOnly(CopyWithinLimit(
            contracts,
            SelectionCodec.ContractFilesMaximum,
            "CSHARP_LIMIT_CONTRACT_FILES"));
        this.methods = Array.AsReadOnly(CopyWithinLimit(
            methods,
            SelectionCodec.SelectedMethodsMaximum,
            "CSHARP_LIMIT_SELECTED_METHODS"));
    }

    internal string Compilation { get; }

    internal IReadOnlyList<string> Sources => sources;

    internal IReadOnlyList<string> Contracts => contracts;

    internal IReadOnlyList<string> Methods => methods;

    private static string[] CopyWithinLimit(IReadOnlyList<string> values, int maximum, string code)
    {
        if (values.Count > maximum)
        {
            throw FrontendFailure.Rejected("capture", code);
        }

        var result = new string[values.Count];
        for (int index = 0; index < values.Count; index++)
        {
            result[index] = values[index];
        }

        return result;
    }
}

internal sealed class LowerRequest
{
    internal LowerRequest(string sourceRoot, RawSelection rawSelection, ReleaseArguments release)
    {
        SourceRoot = sourceRoot;
        RawSelection = rawSelection;
        Release = release;
    }

    internal string SourceRoot { get; }

    internal RawSelection RawSelection { get; }

    internal ReleaseArguments Release { get; }
}
