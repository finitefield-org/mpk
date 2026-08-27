using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;

namespace Mpk.CSharp2Vir;

internal static class CaptureHarness
{
    private const int OneMiB = 1_048_576;
    private static readonly byte[] BaselineSource = Encoding.UTF8.GetBytes(
        "namespace Example.Payment;\npublic static class Policy\n{\n    public static bool Approved(long reserve, long debit)\n    {\n        return reserve >= debit;\n    }\n}\n");

    public static int Main()
    {
        try
        {
            SelectionAndHashAreExact();
            SelectionMutationsFailClosed();
            CliGrammarAndAssertionsAreExact();
            CaptureIsClosedAndImmutable();
            CaptureMutationsHaveExactIssues();
            SourceTransportIsStrict();
            FileAndSnapshotLimitsAreInclusive();
            FailuresAreTypedAndArtifactFree();
            return 0;
        }
        catch (HarnessFailure failure)
        {
            Console.Error.Write("CSHARP_CAPTURE_TEST_" + failure.Code + "\n");
            return 1;
        }
        catch (Exception)
        {
            Console.Error.Write("CSHARP_CAPTURE_TEST_UNEXPECTED\n");
            return 1;
        }
    }

    private static void SelectionAndHashAreExact()
    {
        Selection selection = SelectionCodec.Validate(BaselineSelection());
        const string expected = "{\"schema\":\"mpk.selection.csharp_methods.v0\",\"value\":{\"compilation\":\"payment-policy\",\"contracts\":[\"contracts/approved.json\"],\"methods\":[\"Example.Payment.Policy::Approved(i64,i64)->bool\"],\"sources\":[\"src/Policy.cs\"]}}";
        Equal(expected, Encoding.UTF8.GetString(selection.CanonicalBytes), "SELECTION_BYTES");
        Equal(215, selection.CanonicalBytes.Length, "SELECTION_LENGTH");
        Equal("d5033138bd8c53eee3901d0d1852ed4c1b1a85686cf2a68f01effb0b8c70dfcd", selection.Sha256, "SELECTION_HASH");

        CanonicalMethodId method = selection.ParsedMethods[0];
        Equal("Example.Payment", method.NamespaceName, "METHOD_NAMESPACE");
        Equal("Policy", method.StaticType, "METHOD_TYPE");
        Equal("Approved", method.Method, "METHOD_NAME");
        Equal("i64,i64", string.Join(',', method.ParameterTypes), "METHOD_PARAMETERS");
        Equal("bool", method.ResultType, "METHOD_RESULT");

        CanonicalMethodId noArguments = SelectionCodec.ParseMethodId("N.T::M()->i32");
        Equal(0, noArguments.ParameterTypes.Count, "METHOD_ZERO_PARAMETERS");
    }

    private static void SelectionMutationsFailClosed()
    {
        ExpectFailure(
            () => SelectionCodec.Validate(WithSources("/src/Policy.cs")),
            FrontendStatus.Rejected,
            "capture",
            "CSHARP_CAPTURE_PATH");
        ExpectFailure(
            () => SelectionCodec.Validate(WithSources("src/../Policy.cs")),
            FrontendStatus.Rejected,
            "capture",
            "CSHARP_CAPTURE_PATH");
        ExpectFailure(
            () => SelectionCodec.Validate(WithSources("src/A.cs", "src/a.cs")),
            FrontendStatus.Rejected,
            "capture",
            "CSHARP_CAPTURE_PATH");

        foreach (string invalid in new[]
        {
            "Policy::M()->i32",
            "N.T::M( i32)->i32",
            "N.T::M(int)->i32",
            "N.T::M<T>()->i32",
            "N.T::M()->System.Int32",
            "N.T::Méthod()->i32",
        })
        {
            ExpectSelectionSyntax(() => SelectionCodec.ParseMethodId(invalid));
        }

        string exactMethod = "N.T::" + "M" + new string('a', 1_011) + "()->i32";
        Equal(1_024, exactMethod.Length, "METHOD_LIMIT_CONSTRUCTION");
        _ = SelectionCodec.Validate(WithMethods(exactMethod));
        ExpectFailure(
            () => SelectionCodec.Validate(WithMethods(exactMethod + "a")),
            FrontendStatus.Rejected,
            "capture",
            "CSHARP_LIMIT_CANONICAL_METHOD_ID_BYTES");

        string exactPath = "src/"
            + new string('a', 255) + "/"
            + new string('b', 255) + "/"
            + new string('c', 255) + "/"
            + new string('d', 249) + ".cs";
        Equal(1_024, exactPath.Length, "PATH_LIMIT_CONSTRUCTION");
        _ = SelectionCodec.Validate(WithSources(exactPath));
        ExpectFailure(
            () => SelectionCodec.Validate(WithSources(exactPath.Replace(new string('d', 249), new string('d', 250), StringComparison.Ordinal))),
            FrontendStatus.Rejected,
            "capture",
            "CSHARP_LIMIT_NORMALIZED_PATH_BYTES");

        string[] sources = Enumerable.Range(0, 256).Select(index => $"src/F{index:000}.cs").ToArray();
        _ = SelectionCodec.Validate(WithSources(sources));
        ExpectFailure(
            () => SelectionCodec.Validate(WithSources(Enumerable.Range(0, 257).Select(index => $"src/F{index:000}.cs").ToArray())),
            FrontendStatus.Rejected,
            "capture",
            "CSHARP_LIMIT_SOURCE_FILES");

        string[] contracts = Enumerable.Range(0, 128).Select(index => $"contracts/c{index:000}.json").ToArray();
        _ = SelectionCodec.Validate(WithContracts(contracts));
        ExpectFailure(
            () => SelectionCodec.Validate(WithContracts(Enumerable.Range(0, 129).Select(index => $"contracts/c{index:000}.json").ToArray())),
            FrontendStatus.Rejected,
            "capture",
            "CSHARP_LIMIT_CONTRACT_FILES");

        string[] methods = Enumerable.Range(0, 32).Select(index => $"N.T::M{index:00}()->i32").ToArray();
        _ = SelectionCodec.Validate(WithMethods(methods));
        ExpectFailure(
            () => SelectionCodec.Validate(WithMethods(Enumerable.Range(0, 33).Select(index => $"N.T::M{index:00}()->i32").ToArray())),
            FrontendStatus.Rejected,
            "capture",
            "CSHARP_LIMIT_SELECTED_METHODS");
    }

    private static void CliGrammarAndAssertionsAreExact()
    {
        string[] arguments = BaselineArguments();
        LowerRequest request = CliParser.Parse(arguments);
        Equal(FrontendConstants.SourceRoot, request.SourceRoot, "CLI_ROOT");
        Equal("payment-policy", request.RawSelection.Compilation, "CLI_COMPILATION");
        Equal("frontend.csharp.test.v0", request.Release.FrontendBundleId, "CLI_FRONTEND_BUNDLE");
        Equal("toolchain.csharp.test.v0", request.Release.ToolchainBundleId, "CLI_TOOLCHAIN_BUNDLE");

        ExpectCli(() => CliParser.Parse(arguments.Skip(1).ToArray()));
        ExpectCli(() => CliParser.Parse(Replace(arguments, "--target", "--unknown")));
        ExpectCli(() => CliParser.Parse(ReplaceValue(arguments, "--semantic-profile", "mpk.rust.checked.v0")));
        ExpectCli(() => CliParser.Parse(ReplaceValue(arguments, "--profile-registry-revision", "1")));
        ExpectCli(() => CliParser.Parse(ReplaceValue(arguments, "--profile-registry-sha256", new string('0', 64))));
        ExpectCli(() => CliParser.Parse(ReplaceValue(arguments, "--profile-entry-sha256", new string('0', 64))));
        ExpectCli(() => CliParser.Parse(ReplaceValue(arguments, "--release-registry-id", "mpk.release.registry.v0")));
        ExpectCli(() => CliParser.Parse(ReplaceValue(arguments, "--toolchain-root", "/tmp/toolchain")));
        ExpectFailure(
            () => CliParser.Parse(ArgumentsWithSources(257)),
            FrontendStatus.Rejected,
            "capture",
            "CSHARP_LIMIT_SOURCE_FILES");

        var reordered = arguments.ToArray();
        int source = Array.IndexOf(reordered, "--source");
        int contract = Array.IndexOf(reordered, "--contract");
        (reordered[source], reordered[contract]) = (reordered[contract], reordered[source]);
        ExpectCli(() => CliParser.Parse(reordered));

    }

    private static void CaptureIsClosedAndImmutable()
    {
        WithBaselineRoot(root =>
        {
            Selection selection = SelectionCodec.Validate(BaselineSelection());
            CapturedSnapshot snapshot = SnapshotCapture.Capture(root, selection);
            Equal(2, snapshot.Count, "CAPTURE_COUNT");
            CapturedFile source = snapshot.Find(CapturedInputKind.Source, "src/Policy.cs");
            Equal(BaselineSource.Length, source.SizeBytes, "CAPTURE_SOURCE_SIZE");
            Equal(RawSha256(BaselineSource), source.Sha256, "CAPTURE_SOURCE_HASH");

            File.WriteAllBytes(Path.Combine(root, "src/Policy.cs"), new byte[] { 0xef, 0xbb, 0xbf, (byte)'\n' });
            File.Delete(Path.Combine(root, "contracts/approved.json"));
            CapturedSourceSet sourceSet = SourceTransport.Validate(snapshot);
            Equal(1, sourceSet.Count, "IMMUTABLE_SOURCE_COUNT");
            Equal(Encoding.UTF8.GetString(BaselineSource), sourceSet.SourceAt(0).Text, "IMMUTABLE_SOURCE_TEXT");
            Equal(RawSha256(BaselineSource), source.Sha256, "IMMUTABLE_SOURCE_HASH");
        });
    }

    private static void CaptureMutationsHaveExactIssues()
    {
        WithBaselineRoot(root =>
        {
            File.WriteAllText(Path.Combine(root, "extra.txt"), "extra", new UTF8Encoding(false));
            ExpectCapture(root, BaselineSelection(), "CSHARP_CAPTURE_INVENTORY");
        });
        WithBaselineRoot(root =>
        {
            File.WriteAllText(Path.Combine(root, "project.csproj"), "<Project />", new UTF8Encoding(false));
            ExpectCapture(root, BaselineSelection(), "CSHARP_CAPTURE_INVENTORY");
        });
        WithBaselineRoot(root =>
        {
            File.Delete(Path.Combine(root, "contracts/approved.json"));
            ExpectCapture(root, BaselineSelection(), "CSHARP_CAPTURE_INVENTORY");
        });
        WithBaselineRoot(root =>
        {
            string outside = Path.Combine(Path.GetDirectoryName(root)!, "mpk-csharp-link-target-" + Guid.NewGuid().ToString("N"));
            try
            {
                File.WriteAllBytes(outside, BaselineSource);
                File.Delete(Path.Combine(root, "src/Policy.cs"));
                File.CreateSymbolicLink(Path.Combine(root, "src/Policy.cs"), outside);
                ExpectCapture(root, BaselineSelection(), "CSHARP_CAPTURE_FILE_TYPE");
            }
            finally
            {
                File.Delete(outside);
            }
        });
        WithBaselineRoot(root =>
        {
            string link = Path.Combine(Path.GetDirectoryName(root)!, "mpk-csharp-root-link-" + Guid.NewGuid().ToString("N"));
            try
            {
                Directory.CreateSymbolicLink(link, root);
                ExpectCapture(link, BaselineSelection(), "CSHARP_CAPTURE_FILE_TYPE");
            }
            finally
            {
                Directory.Delete(link);
            }
        });
        WithBaselineRoot(root =>
        {
            string source = Path.Combine(root, "src");
            string outside = Path.Combine(Path.GetDirectoryName(root)!, "mpk-csharp-directory-target-" + Guid.NewGuid().ToString("N"));
            try
            {
                Directory.Move(source, outside);
                Directory.CreateSymbolicLink(source, outside);
                ExpectCapture(root, BaselineSelection(), "CSHARP_CAPTURE_FILE_TYPE");
            }
            finally
            {
                Directory.Delete(outside, recursive: true);
            }
        });
        WithBaselineRoot(root =>
        {
            File.Delete(Path.Combine(root, "src/Policy.cs"));
            Directory.CreateDirectory(Path.Combine(root, "src/Policy.cs"));
            ExpectCapture(root, BaselineSelection(), "CSHARP_CAPTURE_FILE_TYPE");
        });
        WithBaselineRoot(root =>
        {
            string alias = Path.Combine(root, "src/Policy2.cs");
            Check(CreateHardLink(Path.Combine(root, "src/Policy.cs"), alias) == 0, "HARD_LINK_CREATE");
            ExpectCapture(root, WithSources("src/Policy.cs", "src/Policy2.cs"), "CSHARP_CAPTURE_FILE_TYPE");
        });
    }

    private static void SourceTransportIsStrict()
    {
        string accepted = SourceTransport.Decode(Encoding.UTF8.GetBytes("// emoji 😀\n"));
        Equal("// emoji 😀\n", accepted, "SOURCE_UNICODE");

        foreach (byte[] rejected in new[]
        {
            Array.Empty<byte>(),
            new byte[] { 0xef, 0xbb, 0xbf, (byte)'\n' },
            Encoding.UTF8.GetBytes("line\r\n"),
            Encoding.UTF8.GetBytes("line"),
            new byte[] { 0xc3, 0x28, (byte)'\n' },
            new byte[] { (byte)'a', 0, (byte)'\n' },
            new byte[] { 0xed, 0xa0, 0x80, (byte)'\n' },
            new byte[] { 0xef, 0xbf, 0xbe, (byte)'\n' },
            new byte[] { 0xc2, 0x85, (byte)'\n' },
            new byte[] { 0xe2, 0x80, 0xa8, (byte)'\n' },
        })
        {
            ExpectFailure(
                () => SourceTransport.Decode(rejected),
                FrontendStatus.SourceError,
                "source",
                "CSHARP_SOURCE_ENCODING");
        }
    }

    private static void FileAndSnapshotLimitsAreInclusive()
    {
        CaptureSizedSource(OneMiB, expectedCode: null);
        CaptureSizedSource(OneMiB + 1, "CSHARP_LIMIT_SOURCE_FILE_BYTES");
        CaptureSizedContract(OneMiB, expectedCode: null);
        CaptureSizedContract(OneMiB + 1, "CSHARP_LIMIT_CONTRACT_FILE_BYTES");

        CaptureManySizedFiles(CapturedInputKind.Source, 16, OneMiB, expectedCode: null);
        CaptureManySizedFiles(CapturedInputKind.Source, 17, OneMiB, "CSHARP_LIMIT_SOURCE_TOTAL_BYTES");
        CaptureManySizedFiles(CapturedInputKind.Contract, 8, OneMiB, expectedCode: null);
        CaptureManySizedFiles(CapturedInputKind.Contract, 9, OneMiB, "CSHARP_LIMIT_CONTRACT_TOTAL_BYTES");

        CaptureSnapshotEntries(253, expectedCode: null);
        CaptureSnapshotEntries(254, "CSHARP_LIMIT_SNAPSHOT_ENTRIES");
        Equal(
            33_554_432UL,
            InvokeAddWithin(0, 33_554_432, 33_554_432, "CSHARP_LIMIT_SNAPSHOT_TOTAL_BYTES"),
            "SNAPSHOT_TOTAL_BOUNDARY");
        ExpectReflectedFailure(
            () => InvokeAddWithin(0, 33_554_433, 33_554_432, "CSHARP_LIMIT_SNAPSHOT_TOTAL_BYTES"),
            "CSHARP_LIMIT_SNAPSHOT_TOTAL_BYTES");
        ExpectReflectedFailure(
            () => InvokeAddWithin(ulong.MaxValue, 1, int.MaxValue, "CSHARP_LIMIT_SNAPSHOT_TOTAL_BYTES"),
            "CSHARP_LIMIT_SNAPSHOT_TOTAL_BYTES");
    }

    private static void FailuresAreTypedAndArtifactFree()
    {
        FrontendFailure rejected = FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
        Equal(FrontendStatus.Rejected, rejected.Status, "FAILURE_STATUS");
        Equal("capture", rejected.Phase, "FAILURE_PHASE");
        Equal("CSHARP_CAPTURE_INVENTORY", rejected.Code, "FAILURE_CODE");
        Equal(3, rejected.ExitCode, "FAILURE_EXIT");
        string[] properties = typeof(FrontendFailure)
            .GetProperties(System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)
            .Select(property => property.Name)
            .OrderBy(name => name, StringComparer.Ordinal)
            .ToArray();
        Equal("Code,ExitCode,Phase,Status", string.Join(',', properties), "FAILURE_SHAPE");
    }

    private static void CaptureSizedSource(int size, string? expectedCode)
    {
        WithBaselineRoot(root =>
        {
            File.WriteAllBytes(Path.Combine(root, "src/Policy.cs"), RepeatedBytes(size));
            if (expectedCode is null)
            {
                _ = SnapshotCapture.Capture(root, SelectionCodec.Validate(BaselineSelection()));
            }
            else
            {
                ExpectCapture(root, BaselineSelection(), expectedCode);
            }
        });
    }

    private static void CaptureSizedContract(int size, string? expectedCode)
    {
        WithBaselineRoot(root =>
        {
            File.WriteAllBytes(Path.Combine(root, "contracts/approved.json"), RepeatedBytes(size));
            if (expectedCode is null)
            {
                _ = SnapshotCapture.Capture(root, SelectionCodec.Validate(BaselineSelection()));
            }
            else
            {
                ExpectCapture(root, BaselineSelection(), expectedCode);
            }
        });
    }

    private static void CaptureManySizedFiles(CapturedInputKind kind, int count, int size, string? expectedCode)
    {
        WithTemporaryRoot(root =>
        {
            Directory.CreateDirectory(Path.Combine(root, "src"));
            Directory.CreateDirectory(Path.Combine(root, "contracts"));
            string[] sources;
            string[] contracts;
            if (kind == CapturedInputKind.Source)
            {
                sources = Enumerable.Range(0, count).Select(index => $"src/F{index:000}.cs").ToArray();
                contracts = new[] { "contracts/c000.json" };
                foreach (string path in sources)
                {
                    File.WriteAllBytes(ToHostPath(root, path), RepeatedBytes(size));
                }

                File.WriteAllBytes(ToHostPath(root, contracts[0]), new byte[] { (byte)'{' , (byte)'}' });
            }
            else
            {
                sources = new[] { "src/F000.cs" };
                contracts = Enumerable.Range(0, count).Select(index => $"contracts/c{index:000}.json").ToArray();
                File.WriteAllBytes(ToHostPath(root, sources[0]), BaselineSource);
                foreach (string path in contracts)
                {
                    File.WriteAllBytes(ToHostPath(root, path), RepeatedBytes(size));
                }
            }

            RawSelection raw = new RawSelection("limits", sources, contracts, new[] { "N.T::M()->i32" });
            if (expectedCode is null)
            {
                _ = SnapshotCapture.Capture(root, SelectionCodec.Validate(raw));
            }
            else
            {
                ExpectCapture(root, raw, expectedCode);
            }
        });
    }

    private static void CaptureSnapshotEntries(int nestedDirectories, string? expectedCode)
    {
        WithTemporaryRoot(root =>
        {
            var sources = new List<string>();
            for (int index = 0; index < nestedDirectories; index++)
            {
                string path = $"src/d{index:000}/f.cs";
                sources.Add(path);
                Directory.CreateDirectory(Path.GetDirectoryName(ToHostPath(root, path))!);
                File.WriteAllBytes(ToHostPath(root, path), BaselineSource);
            }

            for (int index = nestedDirectories; index < 256; index++)
            {
                string path = $"src/z{index:000}.cs";
                sources.Add(path);
                Directory.CreateDirectory(Path.GetDirectoryName(ToHostPath(root, path))!);
                File.WriteAllBytes(ToHostPath(root, path), BaselineSource);
            }

            Directory.CreateDirectory(Path.Combine(root, "contracts"));
            File.WriteAllBytes(Path.Combine(root, "contracts/c000.json"), new byte[] { (byte)'{', (byte)'}' });
            RawSelection raw = new RawSelection(
                "entry-limit",
                sources,
                new[] { "contracts/c000.json" },
                new[] { "N.T::M()->i32" });
            if (expectedCode is null)
            {
                CapturedSnapshot snapshot = SnapshotCapture.Capture(root, SelectionCodec.Validate(raw));
                Equal(257, snapshot.Count, "SNAPSHOT_FILE_COUNT");
            }
            else
            {
                ExpectCapture(root, raw, expectedCode);
            }
        });
    }

    private static byte[] RepeatedBytes(int size)
    {
        var bytes = new byte[size];
        Array.Fill(bytes, (byte)'x');
        return bytes;
    }

    private static RawSelection BaselineSelection()
    {
        return new RawSelection(
            "payment-policy",
            new[] { "src/Policy.cs" },
            new[] { "contracts/approved.json" },
            new[] { "Example.Payment.Policy::Approved(i64,i64)->bool" });
    }

    private static RawSelection WithSources(params string[] sources)
    {
        RawSelection baseline = BaselineSelection();
        return new RawSelection(baseline.Compilation, sources, baseline.Contracts, baseline.Methods);
    }

    private static RawSelection WithContracts(params string[] contracts)
    {
        RawSelection baseline = BaselineSelection();
        return new RawSelection(baseline.Compilation, baseline.Sources, contracts, baseline.Methods);
    }

    private static RawSelection WithMethods(params string[] methods)
    {
        RawSelection baseline = BaselineSelection();
        return new RawSelection(baseline.Compilation, baseline.Sources, baseline.Contracts, methods);
    }

    private static string[] BaselineArguments()
    {
        const string zero = "0000000000000000000000000000000000000000000000000000000000000000";
        return new[]
        {
            "lower",
            FrontendConstants.SourceRoot,
            "--semantic-profile",
            FrontendConstants.SemanticProfile,
            "--target",
            FrontendConstants.TargetId,
            "--compilation",
            "payment-policy",
            "--source",
            "src/Policy.cs",
            "--contract",
            "contracts/approved.json",
            "--method",
            "Example.Payment.Policy::Approved(i64,i64)->bool",
            "--profile-registry-id",
            FrontendConstants.ProfileRegistryId,
            "--profile-registry-revision",
            "2",
            "--profile-registry-sha256",
            FrontendConstants.ProfileRegistrySha256,
            "--profile-entry-sha256",
            FrontendConstants.ProfileEntrySha256,
            "--frontend-bundle-id",
            "frontend.csharp.test.v0",
            "--frontend-sha256",
            zero,
            "--release-registry-id",
            FrontendConstants.ReleaseRegistryId,
            "--release-registry-sha256",
            zero,
            "--toolchain-bundle-id",
            "toolchain.csharp.test.v0",
            "--toolchain-root",
            FrontendConstants.ToolchainRoot,
            "--toolchain-distribution-sha256",
            zero,
        };
    }

    private static string[] ArgumentsWithSources(int count)
    {
        string[] baseline = BaselineArguments();
        int firstSource = Array.IndexOf(baseline, "--source");
        int contract = Array.IndexOf(baseline, "--contract");
        Check(firstSource >= 0 && contract > firstSource, "CLI_SOURCE_RANGE");
        var result = new List<string>(baseline.Length + checked((count - 1) * 2));
        result.AddRange(baseline.Take(firstSource));
        for (int index = 0; index < count; index++)
        {
            result.Add("--source");
            result.Add($"src/F{index:000}.cs");
        }

        result.AddRange(baseline.Skip(contract));
        return result.ToArray();
    }

    private static string[] Replace(string[] arguments, string oldValue, string newValue)
    {
        string[] copy = arguments.ToArray();
        int index = Array.IndexOf(copy, oldValue);
        Check(index >= 0, "REPLACE_OPTION");
        copy[index] = newValue;
        return copy;
    }

    private static string[] ReplaceValue(string[] arguments, string option, string newValue)
    {
        string[] copy = arguments.ToArray();
        int index = Array.IndexOf(copy, option);
        Check(index >= 0 && index + 1 < copy.Length, "REPLACE_VALUE");
        copy[index + 1] = newValue;
        return copy;
    }

    private static void WithBaselineRoot(Action<string> action)
    {
        WithTemporaryRoot(root =>
        {
            Directory.CreateDirectory(Path.Combine(root, "src"));
            Directory.CreateDirectory(Path.Combine(root, "contracts"));
            File.WriteAllBytes(Path.Combine(root, "src/Policy.cs"), BaselineSource);
            File.WriteAllBytes(Path.Combine(root, "contracts/approved.json"), Encoding.UTF8.GetBytes("{}\n"));
            action(root);
        });
    }

    private static void WithTemporaryRoot(Action<string> action)
    {
        string root = Directory.CreateTempSubdirectory("mpk-csharp-capture-").FullName;
        try
        {
            action(root);
        }
        finally
        {
            Directory.Delete(root, recursive: true);
        }
    }

    private static string ToHostPath(string root, string normalizedPath)
    {
        return Path.Combine(root, normalizedPath.Replace('/', Path.DirectorySeparatorChar));
    }

    private static void ExpectCapture(string root, RawSelection raw, string code)
    {
        ExpectFailure(
            () => SnapshotCapture.Capture(root, SelectionCodec.Validate(raw)),
            FrontendStatus.Rejected,
            "capture",
            code);
    }

    private static void ExpectFailure(Action action, FrontendStatus status, string phase, string code)
    {
        try
        {
            action();
        }
        catch (FrontendFailure failure)
        {
            Equal(status, failure.Status, code + "_STATUS");
            Equal(phase, failure.Phase, code + "_PHASE");
            Equal(code, failure.Code, code + "_CODE");
            return;
        }

        throw new HarnessFailure(code + "_ACCEPTED");
    }

    private static void ExpectSelectionSyntax(Action action)
    {
        try
        {
            action();
        }
        catch (SelectionSyntaxFailure)
        {
            return;
        }

        throw new HarnessFailure("SELECTION_SYNTAX_ACCEPTED");
    }

    private static void ExpectCli(Action action)
    {
        try
        {
            action();
        }
        catch (CliFailure)
        {
            return;
        }

        throw new HarnessFailure("CLI_ACCEPTED");
    }

    private static string RawSha256(byte[] bytes)
    {
        return Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(bytes)).ToLowerInvariant();
    }

    private static ulong InvokeAddWithin(ulong current, ulong increment, int maximum, string code)
    {
        System.Reflection.MethodInfo method = typeof(SnapshotCapture)
            .GetMethods(System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.NonPublic)
            .Single(candidate =>
            {
                if (candidate.Name != "AddWithin")
                {
                    return false;
                }

                System.Reflection.ParameterInfo[] parameters = candidate.GetParameters();
                return parameters.Length == 4 && parameters[1].ParameterType == typeof(ulong);
            });
        return (ulong)method.Invoke(null, new object[] { current, increment, maximum, code })!;
    }

    private static void ExpectReflectedFailure(Action action, string code)
    {
        try
        {
            action();
        }
        catch (System.Reflection.TargetInvocationException error) when (error.InnerException is FrontendFailure failure)
        {
            Equal(code, failure.Code, code + "_REFLECTED_CODE");
            return;
        }

        throw new HarnessFailure(code + "_REFLECTED_ACCEPTED");
    }

    [DllImport("libc", EntryPoint = "link", SetLastError = true)]
    private static extern int CreateHardLink(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string existingPath,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string newPath);

    private static void Equal<T>(T expected, T actual, string code)
    {
        if (!EqualityComparer<T>.Default.Equals(expected, actual))
        {
            throw new HarnessFailure(code);
        }
    }

    private static void Check(bool condition, string code)
    {
        if (!condition)
        {
            throw new HarnessFailure(code);
        }
    }

    private sealed class HarnessFailure : Exception
    {
        internal HarnessFailure(string code)
        {
            Code = code;
        }

        internal string Code { get; }
    }
}
