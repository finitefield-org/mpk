using System;
using System.Collections.Generic;

namespace Mpk.CSharp2Vir;

internal sealed class CliFailure : Exception
{
    internal CliFailure()
        : base("invalid C# frontend arguments")
    {
    }
}

internal static class CliParser
{
    internal static LowerRequest Parse(string[] arguments)
    {
        var cursor = new ArgumentCursor(arguments);
        cursor.Expect("lower");
        string sourceRoot = cursor.Take();
        if (sourceRoot != FrontendConstants.SourceRoot)
        {
            throw new CliFailure();
        }

        cursor.Expect("--semantic-profile");
        cursor.Expect(FrontendConstants.SemanticProfile);
        cursor.Expect("--target");
        cursor.Expect(FrontendConstants.TargetId);
        cursor.Expect("--compilation");
        string compilation = cursor.Take();

        string[] sources = cursor.TakeRepeated(
            "--source",
            SelectionCodec.SourceFilesMaximum,
            "CSHARP_LIMIT_SOURCE_FILES");
        string[] contracts = cursor.TakeRepeated(
            "--contract",
            SelectionCodec.ContractFilesMaximum,
            "CSHARP_LIMIT_CONTRACT_FILES");
        string[] methods = cursor.TakeRepeated(
            "--method",
            SelectionCodec.SelectedMethodsMaximum,
            "CSHARP_LIMIT_SELECTED_METHODS");

        cursor.Expect("--profile-registry-id");
        cursor.Expect(FrontendConstants.ProfileRegistryId);
        cursor.Expect("--profile-registry-revision");
        cursor.Expect(FrontendConstants.ProfileRegistryRevision.ToString(System.Globalization.CultureInfo.InvariantCulture));
        cursor.Expect("--profile-registry-sha256");
        cursor.Expect(FrontendConstants.ProfileRegistrySha256);
        cursor.Expect("--profile-entry-sha256");
        cursor.Expect(FrontendConstants.ProfileEntrySha256);
        cursor.Expect("--frontend-bundle-id");
        string frontendBundleId = TakeReleaseIdentifier(cursor);
        cursor.Expect("--frontend-sha256");
        string frontendSha256 = TakeSha256(cursor);
        cursor.Expect("--release-registry-id");
        cursor.Expect(FrontendConstants.ReleaseRegistryId);
        cursor.Expect("--release-registry-sha256");
        string releaseRegistrySha256 = TakeSha256(cursor);
        cursor.Expect("--toolchain-bundle-id");
        string toolchainBundleId = TakeReleaseIdentifier(cursor);
        cursor.Expect("--toolchain-root");
        cursor.Expect(FrontendConstants.ToolchainRoot);
        cursor.Expect("--toolchain-distribution-sha256");
        string toolchainDistributionSha256 = TakeSha256(cursor);
        cursor.Finish();

        return new LowerRequest(
            sourceRoot,
            new RawSelection(compilation, sources, contracts, methods),
            new ReleaseArguments(
                frontendBundleId,
                frontendSha256,
                releaseRegistrySha256,
                toolchainBundleId,
                toolchainDistributionSha256));
    }

    private static string TakeReleaseIdentifier(ArgumentCursor cursor)
    {
        string value = cursor.Take();
        if (value.Length == 0 || value.Length > 128 || !IsAscii(value))
        {
            throw new CliFailure();
        }

        bool separator = false;
        for (int index = 0; index < value.Length; index++)
        {
            char character = value[index];
            if ((character >= 'a' && character <= 'z') || (character >= '0' && character <= '9'))
            {
                separator = false;
            }
            else if ((character == '.' || character == '_' || character == '-') && index > 0 && !separator)
            {
                separator = true;
            }
            else
            {
                throw new CliFailure();
            }
        }

        if (separator)
        {
            throw new CliFailure();
        }

        return value;
    }

    private static string TakeSha256(ArgumentCursor cursor)
    {
        string value = cursor.Take();
        if (value.Length != 64)
        {
            throw new CliFailure();
        }

        foreach (char character in value)
        {
            if (!((character >= '0' && character <= '9') || (character >= 'a' && character <= 'f')))
            {
                throw new CliFailure();
            }
        }

        return value;
    }

    private static bool IsAscii(string value)
    {
        foreach (char character in value)
        {
            if (character > 0x7f)
            {
                return false;
            }
        }

        return true;
    }

    private sealed class ArgumentCursor
    {
        private readonly string[] arguments;
        private int index;

        internal ArgumentCursor(string[] arguments)
        {
            this.arguments = arguments;
        }

        internal string Take()
        {
            if (index >= arguments.Length || string.IsNullOrEmpty(arguments[index]))
            {
                throw new CliFailure();
            }

            return arguments[index++];
        }

        internal void Expect(string expected)
        {
            if (!string.Equals(Take(), expected, StringComparison.Ordinal))
            {
                throw new CliFailure();
            }
        }

        internal string[] TakeRepeated(string option, int maximum, string limitCode)
        {
            var values = new List<string>();
            while (index < arguments.Length && string.Equals(arguments[index], option, StringComparison.Ordinal))
            {
                if (values.Count >= maximum)
                {
                    throw FrontendFailure.Rejected("capture", limitCode);
                }

                index++;
                string value = Take();
                if (value.StartsWith("--", StringComparison.Ordinal))
                {
                    throw new CliFailure();
                }

                values.Add(value);
            }

            if (values.Count == 0)
            {
                throw new CliFailure();
            }

            return values.ToArray();
        }

        internal void Finish()
        {
            if (index != arguments.Length)
            {
                throw new CliFailure();
            }
        }
    }
}
