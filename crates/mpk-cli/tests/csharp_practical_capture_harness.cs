using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Text;
using Microsoft.CodeAnalysis;

namespace Mpk.CSharp2Vir;

internal static class PracticalCaptureHarness
{
    private static ImmutableArray<MetadataReference> references;
    private static string currentTest = "bootstrap";

    public static int Main(string[] arguments)
    {
        try
        {
            if (arguments.Length != 1)
            {
                throw new HarnessFailure("ARGUMENTS");
            }

            references = LoadReferences(arguments[0]);
            RunTest(nameof(MultiFileClosureIsCompleteAndDeterministic), MultiFileClosureIsCompleteAndDeterministic);
            RunTest(nameof(DeadAndUnsupportedDeclarationsReject), DeadAndUnsupportedDeclarationsReject);
            RunTest(nameof(CompilerSeverityGateIsClosed), CompilerSeverityGateIsClosed);
            RunTest(nameof(SelectionCapturePathAndEncodingAreClosed), SelectionCapturePathAndEncodingAreClosed);
            RunTest(nameof(MpkAndAmbientDependenciesReject), MpkAndAmbientDependenciesReject);
            RunTest(nameof(DelegatesDynamicLinqReflectionAndEffectsReject), DelegatesDynamicLinqReflectionAndEffectsReject);
            RunTest(nameof(GenericsNullableAndIncidentalMetadataAreClosed), GenericsNullableAndIncidentalMetadataAreClosed);
            RunTest(nameof(ClosedFrameworkTypesAreExact), ClosedFrameworkTypesAreExact);
            RunTest(nameof(ConstructorInitializersEnterTheCallClosure), ConstructorInitializersEnterTheCallClosure);
            RunTest(nameof(CallAndTypeCyclesReject), CallAndTypeCyclesReject);
            RunTest(nameof(MethodClosureLimitIsInclusive), MethodClosureLimitIsInclusive);
            RunTest(nameof(SourceTypeLimitIsInclusive), SourceTypeLimitIsInclusive);
            RunTest(nameof(CompilerSynthesizedMarkersStayOpaque), CompilerSynthesizedMarkersStayOpaque);
            RunTest(nameof(FailuresAreArtifactFree), FailuresAreArtifactFree);
            return 0;
        }
        catch (HarnessFailure failure)
        {
            Console.Error.Write(
                "CSHARP_PRACTICAL_CAPTURE_TEST_"
                + currentTest + "_" + failure.Code + "\n");
            return 1;
        }
        catch (Exception error)
        {
            string detail = error is PracticalCaptureFailure practical
                ? practical.Family + "_" + practical.Code
                : error.GetType().Name;
            Console.Error.Write(
                "CSHARP_PRACTICAL_CAPTURE_TEST_UNEXPECTED_"
                + currentTest + "_" + detail + "\n");
            return 1;
        }
    }

    private static void RunTest(string name, Action test)
    {
        currentTest = name;
        test();
    }

    private static void MultiFileClosureIsCompleteAndDeterministic()
    {
        Equal(
            "mpk.csharp.source.66026d6206a6760e0afdc05dd225167b5346a52361be7cf3b35b9bf904a1b657",
            PracticalIdentity.SourceTypeId("Business", "Entry"),
            "SOURCE_TYPE_ID");
        Equal(
            "mpk.csharp.source.78e51d3041e153c1b3760806931b8d1ec19c38c3ef981ce2a50473edc0ad829c",
            DefaultRootId(),
            "SOURCE_METHOD_ID");
        byte[] entryBytes = Utf8(
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(int value) { return Worker.Step(value); }\n"
            + "}\n");
        byte[] workerBytes = Utf8(
            "namespace Business;\n"
            + "internal static class Worker\n"
            + "{\n"
            + "    internal static int Step(int value) { return value + 1; }\n"
            + "}\n");
        var entry = new PracticalCapturedInput(PracticalCapturedInputKind.Source, "src/Entry.cs", entryBytes);
        var worker = new PracticalCapturedInput(PracticalCapturedInputKind.Source, "src/Worker.cs", workerBytes);
        entryBytes[0] = (byte)'X';
        workerBytes[0] = (byte)'X';

        string root = MethodId("Business", "Entry", "Run", new[] { Value("i32") }, Value("i32"));
        PracticalSourceSelection selection = Selection(
            new[] { "src/Entry.cs", "src/Worker.cs" },
            new[] { root });
        PracticalSourceClosure first = CSharpPracticalCapture.Validate(
            selection,
            new[] { worker, entry },
            references);
        PracticalSourceClosure second = CSharpPracticalCapture.Validate(
            selection,
            new[] { entry, worker },
            references.Reverse().ToImmutableArray());

        Equal(2, first.Sources.Count, "MULTI_SOURCE_COUNT");
        Equal(4, first.Declarations.Count, "MULTI_DECLARATION_COUNT");
        Equal(first.Declarations.Count, first.ReachableDeclarations.Count, "MULTI_REACHABLE_COUNT");
        Equal(1, first.CallEdges.Count, "MULTI_CALL_EDGE_COUNT");
        Equal(0, first.SourceDataExceptionTypeCount, "MULTI_DATA_TYPE_COUNT");
        Equal("namespace Business;", first.Sources[0].Text.Split('\n')[0], "IMMUTABLE_CAPTURE");
        Equal(
            string.Join('|', first.Declarations.Select(declaration => declaration.Id)),
            string.Join('|', second.Declarations.Select(declaration => declaration.Id)),
            "DETERMINISTIC_DECLARATIONS");
        Equal(
            string.Join('|', first.CallEdges.Select(edge => edge.SourceId + ">" + edge.TargetId)),
            string.Join('|', second.CallEdges.Select(edge => edge.SourceId + ">" + edge.TargetId)),
            "DETERMINISTIC_CALL_EDGES");

        string constructorOwner = PracticalIdentity.SourceTypeId("Business", "Constructed");
        string constructorRoot = PracticalIdentity.CallableId(
            "constructor",
            "Business",
            constructorOwner,
            "Constructed",
            new[] { Value("i32") },
            constructorOwner);
        PracticalSourceClosure constructor = Run(
            new Dictionary<string, string>(StringComparer.Ordinal)
            {
                ["src/Constructed.cs"] =
                    "namespace Business;\n"
                    + "public sealed class Constructed { public Constructed(int value) { } }\n",
            },
            new[] { constructorRoot });
        Equal(2, constructor.Declarations.Count, "CONSTRUCTOR_ROOT");
    }

    private static void DeadAndUnsupportedDeclarationsReject()
    {
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "dead_declaration",
            () => RunSingle(
                "namespace Business;\n"
                + "public static class Entry\n"
                + "{\n"
                + "    public static int Run(int value) { return value; }\n"
                + "    private static int Dead(int value) { return value; }\n"
                + "}\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "dead_declaration",
            () => RunSingle(
                "namespace Business;\n"
                + "public static class Entry\n"
                + "{\n"
                + "    public static int Run(int value) { _ = nameof(Dead); return value; }\n"
                + "    private static int Dead(int value) { return value; }\n"
                + "}\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "delegate_dynamic_or_runtime_codegen",
            () => RunSingle(
                "namespace Business;\n"
                + "public delegate int Hidden();\n"
                + "public static class Entry { public static int Run(int value) { return value; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "dead_declaration",
            () => RunSingle(
                "namespace Business;\n"
                + "public sealed class Dead { }\n"
                + "public static class Entry\n"
                + "{\n"
                + "    public static int Run(int value) { _ = nameof(Dead); return value; }\n"
                + "}\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "unsupported_declaration",
            () => RunSingle(
                "namespace Business;\n"
                + "public interface Hidden { int Value { get; } }\n"
                + "public static class Entry { public static int Run(int value) { return value; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "unsupported_declaration",
            () => RunSingle(
                "namespace Business;\n"
                + "public sealed class Item(int value) { public int Read() { return value; } }\n"
                + "public static class Entry { public static int Run(int value) { return new Item(value).Read(); } }\n"));
        foreach (string source in new[]
        {
            "namespace Business;\npublic static class Entry { public static int @Run(int value) { return value; } }\n",
            "namespace Business;\npublic static class Entry { public static int Run(int value) { int café = value; return café; } }\n",
            "namespace Bus\\u0069ness;\npublic static class Entry { public static int Run(int value) { return value; } }\n",
        })
        {
            Expect(
                PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
                "source_identifier",
                () => RunSingle(source));
        }
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "selected_root_missing",
            () => Run(
                new Dictionary<string, string>(StringComparer.Ordinal)
                {
                    ["src/Entry.cs"] = DefaultSource(),
                },
                new[] { PracticalIdentity.SourceTypeId("Business", "Missing") }));
        string getterRoot = MethodId(
            "Business",
            "Entry",
            "get_Value",
            Array.Empty<string>(),
            Value("i32"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "selected_root_kind",
            () => Run(
                new Dictionary<string, string>(StringComparer.Ordinal)
                {
                    ["src/Entry.cs"] =
                        "namespace Business;\n"
                        + "public static class Entry { public static int Value { get { return 1; } } }\n",
                },
                new[] { getterRoot }));
        string staticOwner = PracticalIdentity.SourceTypeId("Business", "StaticOnly");
        string staticConstructorRoot = PracticalIdentity.CallableId(
            "constructor",
            "Business",
            staticOwner,
            "StaticOnly",
            Array.Empty<string>(),
            staticOwner);
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "selected_root_kind",
            () => Run(
                new Dictionary<string, string>(StringComparer.Ordinal)
                {
                    ["src/StaticOnly.cs"] =
                        "namespace Business;\n"
                        + "public sealed class StaticOnly { static StaticOnly() { } }\n",
                },
                new[] { staticConstructorRoot }));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "dead_declaration",
            () => RunSingle(
                "namespace Business;\n"
                + "public sealed class Dead { public Dead[][]? Items { get; } }\n"
                + "public static class Entry { public static int Run(int value) { return value; } }\n"));

        var deadTypes = new StringBuilder("namespace Business;\n");
        for (int index = 0; index < 129; index++)
        {
            deadTypes.Append("internal enum Dead").Append(index.ToString("D3"))
                .Append(" { Value }\n");
        }
        deadTypes.Append(
            "public static class Entry { public static int Run(int value) { return value; } }\n");
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "dead_declaration",
            () => RunSingle(deadTypes.ToString()));
    }

    private static void CompilerSeverityGateIsClosed()
    {
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "compiler_diagnostic",
            () => RunSingle(
                "namespace Business;\n"
                + "public static class Entry\n"
                + "{\n"
                + "    public static int Run(int value) { int unused; return value; }\n"
                + "}\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "compiler_diagnostic",
            () => RunSingle(
                "namespace Business;\n"
                + "public static class Entry { public static int Run(int value) { return missing; } }\n"));

        PracticalSourceClosure hiddenIgnored = RunSingle(
            "using System;\n"
            + "namespace Business;\n"
            + "public static class Entry { public static int Run(int value) { return value; } }\n");
        Equal(2, hiddenIgnored.Declarations.Count, "HIDDEN_DIAGNOSTIC_TABLE");
    }

    private static void SelectionCapturePathAndEncodingAreClosed()
    {
        string root = DefaultRootId();
        byte[] valid = Utf8(DefaultSource());
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL,
            "selection_path",
            () => CSharpPracticalCapture.Validate(
                Selection(new[] { "src/Z.cs", "src/A.cs" }, new[] { root }),
                new[]
                {
                    Source("src/Z.cs", DefaultSource()),
                    Source("src/A.cs", DefaultSource().Replace("Entry", "Other", StringComparison.Ordinal)),
                },
                references));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DEPENDENCY,
            "input_inventory",
            () => CSharpPracticalCapture.Validate(
                Selection(new[] { "src/Entry.cs" }, new[] { root }),
                new[] { Source("src/Entry.cs", DefaultSource()), Source("src/Extra.cs", DefaultSource()) },
                references));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DEPENDENCY,
            "input_inventory",
            () => CSharpPracticalCapture.Validate(
                Selection(new[] { "src/Entry.cs" }, new[] { root }),
                OverlongCaptureSequence(),
                references));
        foreach (string path in new[] { "src/CON.cs", "src/com1.data.cs", "src/Lpt9.cs" })
        {
            Expect(
                PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL,
                "selection_path",
                () => CSharpPracticalCapture.Validate(
                    Selection(new[] { path }, new[] { root }),
                    new[] { Source(path, DefaultSource()) },
                    references));
        }
        string[] tooManyRoots = Enumerable.Range(0, 33)
            .Select(index => "mpk.csharp.source." + index.ToString("x64"))
            .ToArray();
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT,
            "selected_methods",
            () => CSharpPracticalCapture.Validate(
                Selection(new[] { "src/Entry.cs" }, tooManyRoots),
                new[] { Source("src/Entry.cs", DefaultSource()) },
                references));
        string[] tooManySources = Enumerable.Range(0, 257)
            .Select(index => "src/" + index.ToString("D3") + ".cs")
            .ToArray();
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT,
            "source_files",
            () => CSharpPracticalCapture.Validate(
                Selection(tooManySources, new[] { root }),
                Array.Empty<PracticalCapturedInput>(),
                references));
        string[] tooManySidecars = Enumerable.Range(0, 129)
            .Select(index => "contracts/" + index.ToString("D3") + ".json")
            .ToArray();
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT,
            "contract_files",
            () => CSharpPracticalCapture.Validate(
                new PracticalSourceSelection(
                    CSharpPracticalCapture.SelectionSchema,
                    "business",
                    new[] { "src/Entry.cs" },
                    new[] { root },
                    tooManySidecars),
                Array.Empty<PracticalCapturedInput>(),
                references));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT,
            "source_file_bytes",
            () => CSharpPracticalCapture.Validate(
                Selection(new[] { "src/Entry.cs" }, new[] { root }),
                new[]
                {
                    new PracticalCapturedInput(
                        PracticalCapturedInputKind.Source,
                        "src/Entry.cs",
                        new byte[1_048_577]),
                },
                references));
        byte[] oneMibSource = Enumerable.Repeat((byte)' ', 1_048_576).ToArray();
        oneMibSource[^1] = (byte)'\n';
        string[] totalSourcePaths = Enumerable.Range(0, 17)
            .Select(index => "src/" + index.ToString("D2") + ".cs")
            .ToArray();
        PracticalCapturedInput[] totalSources = totalSourcePaths
            .Select(path => new PracticalCapturedInput(
                PracticalCapturedInputKind.Source,
                path,
                oneMibSource))
            .ToArray();
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT,
            "source_total_bytes",
            () => CSharpPracticalCapture.Validate(
                Selection(totalSourcePaths, new[] { root }),
                totalSources,
                references));
        var syntaxLimit = new StringBuilder(
            "namespace Business;\npublic static class Entry { public static int Run(int input) { ");
        syntaxLimit.Append(';', 250_001);
        syntaxLimit.Append(" return input; } }\n");
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT,
            "syntax_nodes",
            () => RunSingle(syntaxLimit.ToString()));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL,
            "selected_root_id",
            () => CSharpPracticalCapture.Validate(
                Selection(new[] { "src/Entry.cs" }, new string[] { null! }),
                new[] { Source("src/Entry.cs", DefaultSource()) },
                references));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL,
            "selection_shape",
            () => CSharpPracticalCapture.Validate(
                null!,
                new[] { Source("src/Entry.cs", DefaultSource()) },
                references));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DEPENDENCY,
            "input_inventory",
            () => CSharpPracticalCapture.Validate(
                Selection(new[] { "src/Entry.cs" }, new[] { root }),
                new PracticalCapturedInput[] { null! },
                references));
        foreach (byte[] rejected in new[]
        {
            Array.Empty<byte>(),
            new byte[] { 0xef, 0xbb, 0xbf, (byte)'\n' },
            Utf8(DefaultSource().Replace("\n", "\r\n", StringComparison.Ordinal)),
            Utf8(DefaultSource().TrimEnd('\n')),
            new byte[] { 0xc3, 0x28, (byte)'\n' },
            new byte[] { (byte)'a', 0, (byte)'\n' },
        })
        {
            Expect(
                PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL,
                "source_encoding",
                () => CSharpPracticalCapture.Validate(
                    Selection(new[] { "src/Entry.cs" }, new[] { root }),
                    new[] { new PracticalCapturedInput(PracticalCapturedInputKind.Source, "src/Entry.cs", rejected) },
                    references));
        }

        var sidecarSelection = new PracticalSourceSelection(
            CSharpPracticalCapture.SelectionSchema,
            "business",
            new[] { "src/Entry.cs" },
            new[] { root },
            new[] { "contracts/entry.json" });
        byte[] sidecarBytes = Utf8("{}\n");
        var sidecarInput = new PracticalCapturedInput(
            PracticalCapturedInputKind.Sidecar,
            "contracts/entry.json",
            sidecarBytes);
        sidecarBytes[0] = (byte)'X';
        PracticalSourceClosure closure = CSharpPracticalCapture.Validate(
            sidecarSelection,
            new[]
            {
                sidecarInput,
                new PracticalCapturedInput(PracticalCapturedInputKind.Source, "src/Entry.cs", valid),
            },
            references);
        Equal(1, closure.Sources.Count, "SIDECAR_INVENTORY");
        Equal(1, closure.Sidecars.Count, "SIDECAR_CAPTURE_COUNT");
        Equal("contracts/entry.json", closure.Sidecars[0].Path, "SIDECAR_CAPTURE_PATH");
        Equal((byte)'{', closure.Sidecars[0].CopyBytes()[0], "SIDECAR_IMMUTABLE_CAPTURE");

        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT,
            "contract_file_bytes",
            () => CSharpPracticalCapture.Validate(
                sidecarSelection,
                new[]
                {
                    new PracticalCapturedInput(
                        PracticalCapturedInputKind.Sidecar,
                        "contracts/entry.json",
                        new byte[1_048_577]),
                    Source("src/Entry.cs", DefaultSource()),
                },
                references));

        string[] sidecars = Enumerable.Range(0, 9)
            .Select(index => "contracts/" + index.ToString("D2") + ".json")
            .ToArray();
        var totalInputs = new List<PracticalCapturedInput>
        {
            Source("src/Entry.cs", DefaultSource()),
        };
        byte[] oneMib = new byte[1_048_576];
        totalInputs.AddRange(sidecars.Select(path =>
            new PracticalCapturedInput(PracticalCapturedInputKind.Sidecar, path, oneMib)));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT,
            "contract_total_bytes",
            () => CSharpPracticalCapture.Validate(
                new PracticalSourceSelection(
                    CSharpPracticalCapture.SelectionSchema,
                    "business",
                    new[] { "src/Entry.cs" },
                    new[] { root },
                    sidecars),
                totalInputs,
                references));
    }

    private static void MpkAndAmbientDependenciesReject()
    {
        string applicationSource =
            "namespace Business;\n"
            + "// customer-owned 🧪 name\n"
            + "public static class Entry { public static int Run(int Mpk) { return Mpk; } }\n";
        PracticalSourceClosure applicationName = RunSingle(applicationSource);
        Equal(2, applicationName.Declarations.Count, "APPLICATION_MPK_NAME_IS_NOT_DEPENDENCY");
        PracticalDeclaration applicationType = applicationName.Declarations.Single(declaration =>
            declaration.Kind == PracticalDeclarationKind.Type);
        int expectedTypeStart = new UTF8Encoding(false, true).GetByteCount(
            applicationSource[..applicationSource.IndexOf("public static class", StringComparison.Ordinal)]);
        Equal(expectedTypeStart, applicationType.StartByte, "UTF8_DECLARATION_START");
        Equal(
            new UTF8Encoding(false, true).GetByteCount(applicationSource.TrimEnd('\n')),
            applicationType.EndByte,
            "UTF8_DECLARATION_END");

        PracticalSourceClosure applicationAssemblyName = CSharpPracticalCapture.Validate(
            new PracticalSourceSelection(
                CSharpPracticalCapture.SelectionSchema,
                "mpk",
                new[] { "src/Entry.cs" },
                new[] { DefaultRootId() },
                Array.Empty<string>()),
            new[] { Source("src/Entry.cs", DefaultSource()) },
            references);
        Equal(
            2,
            applicationAssemblyName.Declarations.Count,
            "APPLICATION_MPK_COMPILATION_ID_IS_NOT_DEPENDENCY");

        foreach (string source in new[]
        {
            "using Mpk.Internal;\nnamespace Business;\npublic static class Entry { public static int Run(int value) { return value; } }\n",
            "namespace Mpk.Customer;\npublic static class Entry { public static int Run(int value) { return value; } }\n",
            "namespace Business;\n[Mpk.Internal.Marker]\npublic static class Entry { public static int Run(int value) { return value; } }\n",
            "namespace Business;\npublic sealed class Item : Mpk.Internal.Base { }\npublic static class Entry { public static int Run(int value) { return value; } }\n",
        })
        {
            ExpectFamily(PracticalDiagnosticFamily.CSHARP_PRACTICAL_DEPENDENCY, () => RunSingle(source));
        }

        foreach (PracticalCapturedInputKind kind in new[]
        {
            PracticalCapturedInputKind.Project,
            PracticalCapturedInputKind.Package,
            PracticalCapturedInputKind.Binary,
            PracticalCapturedInputKind.GeneratedSource,
            PracticalCapturedInputKind.AnalyzerConfig,
            PracticalCapturedInputKind.EditorConfig,
        })
        {
            ExpectFamily(
                PracticalDiagnosticFamily.CSHARP_PRACTICAL_DEPENDENCY,
                () => CSharpPracticalCapture.Validate(
                    Selection(new[] { "src/Entry.cs" }, new[] { DefaultRootId() }),
                    new[] { new PracticalCapturedInput(kind, "src/Entry.cs", Utf8(DefaultSource())) },
                    references));
        }

        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DEPENDENCY,
            "ambient_reference",
            () => CSharpPracticalCapture.Validate(
                Selection(new[] { "src/Entry.cs" }, new[] { DefaultRootId() }),
                new[] { Source("src/Entry.cs", DefaultSource()) },
                references.Add(references[0])));
    }

    private static void DelegatesDynamicLinqReflectionAndEffectsReject()
    {
        foreach (string body in new[]
        {
            "System.Func<int> value = Other; return value();",
            "System.Func<int> value = () => 1; return value();",
            "dynamic value = 1; return value;",
            "int[] values = new int[] { 1 }; return (from value in values select value).First();",
            "return typeof(int).Name.Length;",
            "return input.GetType().Name.Length;",
            "return System.Activator.CreateInstance(typeof(int)) is null ? 0 : 1;",
            "System.Linq.Expressions.Expression<System.Func<int>> value = () => 1; return 1;",
        })
        {
            string other = body.Contains("Other", StringComparison.Ordinal)
                ? " private static int Other() { return 1; }"
                : string.Empty;
            ExpectFamily(
                PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
                () => RunSingle(
                    "namespace Business;\n"
                    + "public static class Entry { public static int Run(int input) { " + body + " }"
                    + other + " }\n"));
        }

        string[] effectBodies =
        {
            "System.Console.WriteLine(input); return input;",
            "return System.IO.File.Exists(\"x\") ? 1 : 0;",
            "return System.Environment.TickCount;",
            "return new System.Random().Next();",
            "return System.Diagnostics.Stopwatch.GetTimestamp() > 0 ? 1 : 0;",
            "_ = System.DateTime.Today; return input;",
            "_ = System.TimeProvider.System; return input;",
            "return System.Guid.NewGuid() == default ? 0 : 1;",
            "System.Threading.Tasks.Task pending = System.Threading.Tasks.Task.CompletedTask; return pending.IsCompleted ? input : 0;",
        };
        for (int index = 0; index < effectBodies.Length; index++)
        {
            ExpectFamily(
                PracticalDiagnosticFamily.CSHARP_PRACTICAL_EFFECT,
                "EFFECT_" + index,
                () => RunSingle(
                    "namespace Business;\n"
                    + "public static class Entry { public static int Run(int input) { "
                    + effectBodies[index] + " } }\n"));
        }

        ExpectFamily(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_EFFECT,
            "FOREACH_EFFECT",
            () => RunSingle(
                "namespace Business;\n"
                + "public static class Entry\n"
                + "{\n"
                + "    public static int Run(int input)\n"
                + "    {\n"
                + "        int result = input;\n"
                + "        foreach (int value in new int[] { input }) { result = value; }\n"
                + "        return result;\n"
                + "    }\n"
                + "}\n"));
        string asyncRoot = MethodId(
            "Business",
            "Entry",
            "Run",
            new[] { Value("i32") },
            Value("unit"));
        ExpectFamily(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_EFFECT,
            "ASYNC_EFFECT",
            () => Run(
                new Dictionary<string, string>(StringComparer.Ordinal)
                {
                    ["src/Entry.cs"] =
                        "namespace Business;\n"
                        + "public static class Entry\n"
                        + "{\n"
                        + "    public static async void Run(int input)\n"
                        + "    {\n"
                        + "        await System.Threading.Tasks.Task.Yield();\n"
                        + "    }\n"
                        + "}\n",
                },
                new[] { asyncRoot }));
        string state = PracticalIdentity.SourceTypeId("Business", "State");
        string volatileRoot = MethodId(
            "Business",
            "Entry",
            "Run",
            new[] { state },
            Value("i32"));
        ExpectFamily(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_EFFECT,
            "VOLATILE_EFFECT",
            () => Run(
                new Dictionary<string, string>(StringComparer.Ordinal)
                {
                    ["src/Entry.cs"] =
                        "namespace Business;\n"
                        + "public sealed class State { public volatile int Value; }\n"
                        + "public static class Entry { public static int Run(State state) { return state.Value; } }\n",
                },
                new[] { volatileRoot }));

        ExpectFamily(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE,
            () => RunSingle(
                "namespace Business;\n"
                + "public static class Entry { public static int Run(int input) { return System.BitConverter.GetBytes(input).Length; } }\n"));
    }

    private static void GenericsNullableAndIncidentalMetadataAreClosed()
    {
        foreach (string source in new[]
        {
            "namespace Business;\npublic static class Entry { public static T Run<T>(T value) { return value; } }\n",
            "namespace Business;\npublic sealed class Box<T> { }\npublic static class Entry { public static int Run(int value) { return value; } }\n",
            "namespace Business;\npublic static class Entry { public static int Run(int input) { System.Collections.Generic.List<int>? value = null; return value is null ? input : input + 1; } }\n",
            "namespace Business;\npublic static class Entry { public static int Run(int input) { _ = new System.Collections.Generic.List<int>(); return input; } }\n",
            "namespace Business;\npublic static class Entry { public static int Run(int input) { System.Collections.Generic.List<int>[]? values = null; return values is null ? input : 0; } }\n",
            "namespace Business;\npublic static class Entry { public static int Run(int input) { System.Nullable<int> value = input; return value.Value; } }\n",
            "namespace Business;\npublic static class Entry { public static System.Threading.Tasks.Task<int> Run(int input) { return System.Threading.Tasks.Task.FromResult(input); } }\n",
            "namespace Business;\npublic static class Entry { public static System.Collections.Generic.IEnumerable<int> Run(int input) { yield return input; } }\n",
        })
        {
            ExpectFamily(PracticalDiagnosticFamily.CSHARP_PRACTICAL_GENERIC, () => RunSingle(source));
        }

        PracticalSourceClosure nullable = RunSingle(
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(int input) { int? value = input; return value.GetValueOrDefault(); }\n"
            + "}\n");
        Equal(2, nullable.Declarations.Count, "NULLABLE_EXACT_FORM");
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE,
            "framework_api",
            () => RunSingle(
                "namespace Business;\n"
                + "public static class Entry\n"
                + "{\n"
                + "    public static int Run(int input) { int? value = input; return value.GetHashCode(); }\n"
                + "}\n"));

        PracticalSourceClosure incidental = RunSingle(
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(string? value) { return value is null ? 0 : value.Length; }\n"
            + "}\n",
            new[] { Value("string") });
        Equal(2, incidental.Declarations.Count, "INCIDENTAL_GENERIC_METADATA");

        PracticalSourceClosure semanticIdentifiers = RunSingle(
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(int input) { int dynamic = input; int mpk_value = dynamic; return mpk_value; }\n"
            + "}\n");
        Equal(2, semanticIdentifiers.Declarations.Count, "SEMANTIC_IDENTIFIER_CLASSIFICATION");

        PracticalSourceClosure nullableName = RunSingle(
            "namespace Business;\n"
            + "public readonly struct NullableAmount { public int Value { get; } }\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(int input)\n"
            + "    {\n"
            + "        NullableAmount? amount = default(NullableAmount?);\n"
            + "        return amount.HasValue ? amount.Value.Value : input;\n"
            + "    }\n"
            + "}\n");
        Equal(5, nullableName.Declarations.Count, "NULLABLE_SOURCE_NAME_IS_SEMANTIC");
    }

    private static void ClosedFrameworkTypesAreExact()
    {
        string i32 = Value("i32");
        string sequence = PracticalIdentity.ClosedInstanceId("bounded_sequence", i32);
        string option = PracticalIdentity.ClosedInstanceId("option", i32);
        Equal(
            "mpk.csharp.instance.ed1833d7de995f851050cda920877e4f114c8ceedd7ad25010205e5238bc76c3",
            sequence,
            "BOUND_SEQUENCE_ID");
        Equal(
            "mpk.csharp.instance.9f7acdcf062807a2fd9542fe16184fa682ff33a6e64bd43974b14953447f7338",
            option,
            "OPTION_ID");
        PracticalSourceClosure array = RunSingle(
            "namespace Business;\n"
            + "public static class Entry { public static int Run(int[] values) { return values.Length; } }\n",
            new[] { sequence });
        Equal(2, array.Declarations.Count, "BOUND_SEQUENCE_ROOT_ID");
        PracticalSourceClosure nullable = RunSingle(
            "namespace Business;\n"
            + "public static class Entry { public static int Run(int? value) { return value.GetValueOrDefault(); } }\n",
            new[] { option });
        Equal(2, nullable.Declarations.Count, "OPTION_ROOT_ID");
        string nullableSequence = PracticalIdentity.ClosedInstanceId("bounded_sequence", option);
        PracticalSourceClosure nullableArray = RunSingle(
            "namespace Business;\n"
            + "public static class Entry { public static int Run(int?[] values) { return values.Length; } }\n",
            new[] { nullableSequence });
        Equal(2, nullableArray.Declarations.Count, "OPTION_SEQUENCE_ROOT_ID");

        PracticalSourceClosure admitted = RunSingle(
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(\n"
            + "        System.DateOnly date, System.TimeOnly time, System.TimeSpan duration,\n"
            + "        System.Guid guid, System.DayOfWeek day) { return 0; }\n"
            + "}\n",
            new[]
            {
                Value("date"),
                Value("time"),
                Value("duration"),
                Value("guid"),
                Value("day_of_week"),
            });
        Equal(2, admitted.Declarations.Count, "CLOSED_FRAMEWORK_VALUES");

        PracticalSourceClosure intrinsicConstants = RunSingle(
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(string value)\n"
            + "    {\n"
            + "        decimal rounded = decimal.Round(1m, 0, System.MidpointRounding.ToEven);\n"
            + "        return string.Equals(value, \"ok\", System.StringComparison.Ordinal) && rounded == 1m ? 1 : 0;\n"
            + "    }\n"
            + "}\n",
            new[] { Value("string") });
        Equal(2, intrinsicConstants.Declarations.Count, "INTRINSIC_CONSTANT_ARGUMENTS");

        PracticalSourceClosure closedConstants = RunSingle(
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(System.Guid guid, System.DayOfWeek day)\n"
            + "    {\n"
            + "        return guid == System.Guid.Empty && day == System.DayOfWeek.Monday ? 1 : 0;\n"
            + "    }\n"
            + "}\n",
            new[] { Value("guid"), Value("day_of_week") });
        Equal(2, closedConstants.Declarations.Count, "CLOSED_CONSTANT_MEMBERS");

        foreach (string source in new[]
        {
            "namespace Business;\npublic static class Entry { public static int Run(int input) { _ = System.StringComparison.Ordinal; return input; } }\n",
            "namespace Business;\npublic static class Entry { public static int Run(int input) { _ = System.MidpointRounding.ToEven; return input; } }\n",
            "namespace Business;\npublic static class Entry { public static int Run(string value) { return string.Equals(value, \"ok\", default) ? 1 : 0; } }\n",
            "namespace Business;\npublic static class Entry { public static int Run(int input) { return decimal.Round(1m, 0, default) == 1m ? input : 0; } }\n",
            "namespace Business;\npublic static class Entry { public static int Run(string value, bool choose) { return string.Equals(value, \"ok\", choose ? System.StringComparison.Ordinal : default) ? 1 : 0; } }\n",
        })
        {
            ExpectFamily(PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE, () => RunSingle(source));
        }

        foreach (string source in new[]
        {
            "namespace Business;\npublic static class Entry { public static int Run(System.Uri value) { return 0; } }\n",
            "namespace Business;\npublic static class Entry { public static int Run(System.DateTime value) { return 0; } }\n",
            "namespace Business;\npublic static class Entry { public static int Run(int[,] value) { return 0; } }\n",
            "namespace Business;\npublic static class Entry { public static int Run(int input) { System.Uri? value = null; return value is null ? input : 0; } }\n",
        })
        {
            Expect(
                PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE,
                "closed_type",
                () => RunSingle(source));
        }
    }

    private static void ConstructorInitializersEnterTheCallClosure()
    {
        string root = MethodId("Business", "Entry", "Run", new[] { Value("i32") }, Value("i32"));
        PracticalSourceClosure closure = Run(
            new Dictionary<string, string>(StringComparer.Ordinal)
            {
                ["src/Entry.cs"] =
                    "namespace Business;\n"
                    + "public sealed class Item\n"
                    + "{\n"
                    + "    private readonly int value;\n"
                    + "    public Item() { value = 0; }\n"
                    + "    public Item(int value) : this() { this.value = value; }\n"
                    + "    public int Read() { return value; }\n"
                    + "}\n"
                    + "public static class Entry\n"
                    + "{\n"
                    + "    public static int Run(int input) { return new Item(input).Read(); }\n"
                    + "}\n",
            },
            new[] { root });
        Equal(3, closure.CallEdges.Count, "CONSTRUCTOR_INITIALIZER_EDGE");
        Equal(7, closure.Declarations.Count, "CONSTRUCTOR_INITIALIZER_DECLARATIONS");
    }

    private static void CallAndTypeCyclesReject()
    {
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "call_cycle",
            () => RunSingle(
                "namespace Business;\n"
                + "public static class Entry\n"
                + "{\n"
                + "    public static int Run(int input) { return Again(input); }\n"
                + "    private static int Again(int input) { return Run(input); }\n"
                + "}\n"));

        string a = PracticalIdentity.SourceTypeId("Business", "A");
        string root = MethodId("Business", "Entry", "Run", new[] { a }, Value("i32"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE,
            "type_cycle",
            () => Run(
                new Dictionary<string, string>(StringComparer.Ordinal)
                {
                    ["src/Cycle.cs"] =
                        "namespace Business;\n"
                        + "public sealed class A { public B? Next { get; } }\n"
                        + "public sealed class B { public A? Next { get; } }\n"
                        + "public static class Entry { public static int Run(A value) { return value.Next is null ? 0 : 1; } }\n",
                },
                new[] { root }));

        string self = PracticalIdentity.SourceTypeId("Business", "Self");
        string selfRoot = MethodId("Business", "Entry", "Run", new[] { self }, Value("i32"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE,
            "type_cycle",
            () => Run(
                new Dictionary<string, string>(StringComparer.Ordinal)
                {
                    ["src/SelfCycle.cs"] =
                        "namespace Business;\n"
                        + "public sealed class Self { public Self? Next { get; } }\n"
                        + "public static class Entry { public static int Run(Self value) { return value.Next is null ? 0 : 1; } }\n",
                },
                new[] { selfRoot }));

        string nested = PracticalIdentity.SourceTypeId("Business", "Nested");
        string nestedRoot = MethodId("Business", "Entry", "Run", new[] { nested }, Value("i32"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE,
            "type_cycle",
            () => Run(
                new Dictionary<string, string>(StringComparer.Ordinal)
                {
                    ["src/NestedCycle.cs"] =
                        "namespace Business;\n"
                        + "public sealed class Nested { public Nested[][]? Items { get; } }\n"
                        + "public static class Entry { public static int Run(Nested value) { return value.Items is null ? 0 : value.Items.Length; } }\n",
                },
                new[] { nestedRoot }));
    }

    private static void MethodClosureLimitIsInclusive()
    {
        PracticalSourceClosure at = RunMethodClosureLimit(128);
        Equal(129, at.Declarations.Count, "METHOD_CLOSURE_LIMIT_AT");
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT,
            "method_closure",
            () => RunMethodClosureLimit(129));
    }

    private static void SourceTypeLimitIsInclusive()
    {
        PracticalSourceClosure below = RunTypeLimit(127);
        Equal(127, below.SourceDataExceptionTypeCount, "TYPE_LIMIT_BELOW");
        PracticalSourceClosure at = RunTypeLimit(128);
        Equal(128, at.SourceDataExceptionTypeCount, "TYPE_LIMIT_AT");
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT,
            "source_data_exception_types",
            () => RunTypeLimit(129));
    }

    private static void CompilerSynthesizedMarkersStayOpaque()
    {
        string request = PracticalIdentity.SourceTypeId("Business", "Request");
        string root = MethodId("Business", "Entry", "Run", new[] { request }, Value("i32"));
        PracticalSourceClosure closure = Run(
            new Dictionary<string, string>(StringComparer.Ordinal)
            {
                ["src/Required.cs"] =
                    "namespace Business;\n"
                    + "public sealed class Request { public required int Value { get; init; } }\n"
                    + "public static class Entry { public static int Run(Request value) { return value.Value; } }\n",
            },
            new[] { root });
        Equal(1, closure.SourceDataExceptionTypeCount, "SYNTHESIZED_MARKER_DATA_TYPE");
        Check(
            closure.Declarations.All(declaration =>
                !declaration.Id.Contains("RequiredMemberAttribute", StringComparison.Ordinal)
                && !declaration.Id.Contains("IsExternalInit", StringComparison.Ordinal)),
            "SYNTHESIZED_MARKER_OPAQUE");

        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "source_attribute",
            () => RunSingle(
                "namespace Business;\n"
                + "[System.Obsolete]\n"
                + "public static class Entry { public static int Run(int input) { return input; } }\n"));

        string fieldRequest = PracticalIdentity.SourceTypeId("Business", "FieldRequest");
        string fieldRoot = MethodId("Business", "Entry", "Run", new[] { fieldRequest }, Value("i32"));
        PracticalSourceClosure fieldClosure = Run(
            new Dictionary<string, string>(StringComparer.Ordinal)
            {
                ["src/RequiredField.cs"] =
                    "namespace Business;\n"
                    + "public sealed class FieldRequest { public required int Value; }\n"
                    + "public static class Entry { public static int Run(FieldRequest value) { return value.Value; } }\n",
            },
            new[] { fieldRoot });
        Equal(4, fieldClosure.Declarations.Count, "SYNTHESIZED_REQUIRED_FIELD");
    }

    private static void FailuresAreArtifactFree()
    {
        PracticalCaptureFailure failure = CaptureFailure(() => RunSingle(
            "namespace Customer.Secret;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(int input) { return MissingCustomerMember; }\n"
            + "}\n"));
        Equal(0, failure.ArtifactCount, "FAILURE_ARTIFACT_COUNT");
        Equal(
            PracticalCaptureFailure.PublicMessage,
            failure.Message,
            "FAILURE_PUBLIC_MESSAGE");
        Check(!failure.Message.Contains("Customer", StringComparison.Ordinal), "FAILURE_NO_NAMESPACE");
        Check(!failure.Message.Contains("MissingCustomerMember", StringComparison.Ordinal), "FAILURE_NO_MEMBER");
        Check(!failure.Message.Contains("src/", StringComparison.Ordinal), "FAILURE_NO_PATH");
    }

    private static PracticalSourceClosure RunTypeLimit(int count)
    {
        var source = new StringBuilder("namespace Business;\n");
        for (int index = 0; index < count; index++)
        {
            source.Append("internal enum T").Append(index.ToString("D3"))
                .Append(" { Value = ").Append(index).Append(" }\n");
        }

        source.Append("public static class Entry\n{\n    public static int Run(int input)\n    {\n        return input");
        for (int index = 0; index < count; index++)
        {
            source.Append(" + (int)T").Append(index.ToString("D3")).Append(".Value");
        }
        source.Append(";\n    }\n}\n");
        return RunSingle(source.ToString());
    }

    private static PracticalSourceClosure RunMethodClosureLimit(int count)
    {
        var source = new StringBuilder("namespace Business;\npublic static class Entry\n{\n");
        for (int index = 0; index < count; index++)
        {
            source.Append("    public static int M").Append(index.ToString("D3"))
                .Append("(int input) { return ");
            if (index + 1 == count)
            {
                source.Append("input");
            }
            else
            {
                source.Append("M").Append((index + 1).ToString("D3")).Append("(input)");
            }
            source.Append("; }\n");
        }
        source.Append("}\n");
        string root = MethodId("Business", "Entry", "M000", new[] { Value("i32") }, Value("i32"));
        return Run(
            new Dictionary<string, string>(StringComparer.Ordinal) { ["src/Entry.cs"] = source.ToString() },
            new[] { root });
    }

    private static IEnumerable<PracticalCapturedInput> OverlongCaptureSequence()
    {
        yield return Source("src/Entry.cs", DefaultSource());
        yield return Source("src/Extra.cs", DefaultSource());
        throw new InvalidOperationException("capture enumerated beyond its frozen bound");
    }

    private static PracticalSourceClosure RunSingle(string source, string[]? parameterTypes = null)
    {
        string root = MethodId(
            "Business",
            "Entry",
            "Run",
            parameterTypes ?? new[] { Value("i32") },
            Value("i32"));
        return Run(
            new Dictionary<string, string>(StringComparer.Ordinal) { ["src/Entry.cs"] = source },
            new[] { root });
    }

    private static PracticalSourceClosure Run(
        IReadOnlyDictionary<string, string> sourceByPath,
        string[] roots)
    {
        string[] paths = sourceByPath.Keys.OrderBy(value => value, StringComparer.Ordinal).ToArray();
        PracticalCapturedInput[] inputs = paths
            .Select(path => Source(path, sourceByPath[path]))
            .Reverse()
            .ToArray();
        return CSharpPracticalCapture.Validate(Selection(paths, roots), inputs, references);
    }

    private static PracticalSourceSelection Selection(string[] paths, string[] roots) =>
        new PracticalSourceSelection(
            CSharpPracticalCapture.SelectionSchema,
            "business",
            paths,
            roots.OrderBy(value => value, StringComparer.Ordinal),
            Array.Empty<string>());

    private static PracticalCapturedInput Source(string path, string source) =>
        new PracticalCapturedInput(PracticalCapturedInputKind.Source, path, Utf8(source));

    private static string DefaultSource() =>
        "namespace Business;\n"
        + "public static class Entry { public static int Run(int value) { return value; } }\n";

    private static string DefaultRootId() =>
        MethodId("Business", "Entry", "Run", new[] { Value("i32") }, Value("i32"));

    private static string MethodId(
        string namespaceName,
        string type,
        string method,
        IEnumerable<string> parameters,
        string result)
    {
        string owner = PracticalIdentity.SourceTypeId(namespaceName, type);
        return PracticalIdentity.CallableId("method", namespaceName, owner, method, parameters, result);
    }

    private static string Value(string token) => PracticalIdentity.PrimitiveId(token);

    private static byte[] Utf8(string value) => new UTF8Encoding(false, true).GetBytes(value);

    private static ImmutableArray<MetadataReference> LoadReferences(string referenceRoot)
    {
        string directory = Path.Combine(referenceRoot, "ref", "net10.0");
        string[] paths = Directory.EnumerateFiles(directory, "*.dll", SearchOption.TopDirectoryOnly)
            .OrderBy(path => Path.GetFileName(path), StringComparer.Ordinal)
            .ToArray();
        Equal(167, paths.Length, "REFERENCE_COUNT");
        return paths.Select(path => MetadataReference.CreateFromFile(path)).ToImmutableArray<MetadataReference>();
    }

    private static void Expect(
        PracticalDiagnosticFamily family,
        string code,
        Action action)
    {
        PracticalCaptureFailure failure = CaptureFailure(action);
        Equal(family, failure.Family, "FAILURE_FAMILY_" + code);
        Equal(code, failure.Code, "FAILURE_CODE_" + code + "_GOT_" + failure.Code);
        Equal(ExpectedPhase(family), failure.Phase, "FAILURE_PHASE_" + code);
        Equal(0, failure.ArtifactCount, "FAILURE_ARTIFACTS_" + code);
    }

    private static void ExpectFamily(PracticalDiagnosticFamily family, Action action)
    {
        ExpectFamily(family, family.ToString(), action);
    }

    private static void ExpectFamily(PracticalDiagnosticFamily family, string context, Action action)
    {
        PracticalCaptureFailure failure = CaptureFailure(action);
        Equal(
            family,
            failure.Family,
            "FAILURE_FAMILY_" + context + "_GOT_" + failure.Family + "_" + failure.Code);
        Equal(ExpectedPhase(family), failure.Phase, "FAILURE_PHASE");
        Equal(0, failure.ArtifactCount, "FAILURE_ARTIFACTS");
    }

    private static PracticalCaptureFailure CaptureFailure(Action action)
    {
        try
        {
            action();
        }
        catch (PracticalCaptureFailure failure)
        {
            return failure;
        }

        throw new HarnessFailure("EXPECTED_FAILURE");
    }

    private static int ExpectedPhase(PracticalDiagnosticFamily family) => family switch
    {
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL => 0,
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT => 0,
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_DEPENDENCY => 1,
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION => 2,
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE => 2,
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_GENERIC => 3,
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_EFFECT => 7,
        _ => throw new HarnessFailure("UNKNOWN_PHASE"),
    };

    private static void Check(bool condition, string code)
    {
        if (!condition)
        {
            throw new HarnessFailure(code);
        }
    }

    private static void Equal<T>(T expected, T actual, string code)
        where T : notnull
    {
        if (!EqualityComparer<T>.Default.Equals(expected, actual))
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
