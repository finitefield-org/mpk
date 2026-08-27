using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using System.Text;

namespace Mpk.CSharp2Vir;

internal static class SubsetHarness
{
    private const string SourcePath = "src/Case.cs";
    private const string ContractPath = "contracts/case.json";

    public static int Main(string[] args)
    {
        if (args.Length != 1)
        {
            Console.Error.Write("CSHARP_SUBSET_TEST_USAGE\n");
            return 1;
        }

        try
        {
            string referencePackRoot = args[0];
            AcceptedClosureIsDeterministic(referencePackRoot);
            ExactTypesLiteralsAndControlAreAccepted(referencePackRoot);
            DeclarationTypeAndLiteralAdmissionIsClosed(referencePackRoot);
            ControlOperationAndConversionAdmissionIsClosed(referencePackRoot);
            ClosurePurityAndInitializationAreClosed(referencePackRoot);
            DefiniteAssignmentAndCfgAccountingAreOwned(referencePackRoot);
            SemanticRowRejectionsAreOwned(referencePackRoot);
            LimitsAreInclusiveAndChecked(referencePackRoot);
            return 0;
        }
        catch (HarnessFailure failure)
        {
            Console.Error.Write("CSHARP_SUBSET_TEST_" + failure.Code + "\n");
            return 1;
        }
        catch (Exception)
        {
            Console.Error.Write("CSHARP_SUBSET_TEST_UNEXPECTED\n");
            return 1;
        }
    }

    private static void AcceptedClosureIsDeterministic(string referencePackRoot)
    {
        const string source =
            "namespace Vector;\n"
            + "public static class Case\n"
            + "{\n"
            + "    public static int F(bool choose, int x)\n"
            + "    {\n"
            + "        int result = x;\n"
            + "        if (choose)\n"
            + "        {\n"
            + "            result = G(result);\n"
            + "        }\n"
            + "        else\n"
            + "        {\n"
            + "            result = H(result);\n"
            + "        }\n"
            + "        return result;\n"
            + "    }\n"
            + "    private static int G(int x) { return H(x); }\n"
            + "    private static int H(int x) { return x; }\n"
            + "}\n";
        SubsetClosure first = Validate(
            referencePackRoot,
            source,
            "Vector.Case::F(bool,i32)->i32");
        SubsetClosure second = Validate(
            referencePackRoot,
            source,
            "Vector.Case::F(bool,i32)->i32");
        string expected = string.Join(',', new[]
        {
            "Vector.Case::H(i32)->i32",
            "Vector.Case::G(i32)->i32",
            "Vector.Case::F(bool,i32)->i32",
        });
        Equal(expected, string.Join(',', first.Methods.Select(method => method.CanonicalId)), "CLOSURE_ORDER");
        Equal(expected, string.Join(',', second.Methods.Select(method => method.CanonicalId)), "CLOSURE_REPEAT");
        Equal(3, first.Methods.Length, "CLOSURE_COUNT");
        Equal("Vector.Case::F(bool,i32)->i32", first.SelectedRoots.Single(), "CLOSURE_ROOT");
        Equal("Vector.Case::H(i32)->i32", first.Methods[1].Callees.Single(), "CLOSURE_EDGE_G");
        Equal(
            "Vector.Case::G(i32)->i32,Vector.Case::H(i32)->i32",
            string.Join(',', first.Methods[2].Callees),
            "CLOSURE_EDGE_F");
        Check(first.SyntaxNodeCount > 0, "CLOSURE_SYNTAX_COUNT");
        Check(first.OperationCount > 0, "CLOSURE_OPERATION_COUNT");
        Check(first.CfgBlockCount >= 9, "CLOSURE_CFG_COUNT");
        Equal(first.SyntaxNodeCount, second.SyntaxNodeCount, "CLOSURE_SYNTAX_REPEAT");
        Equal(first.OperationCount, second.OperationCount, "CLOSURE_OPERATION_REPEAT");
        Equal(first.CfgBlockCount, second.CfgBlockCount, "CLOSURE_CFG_REPEAT");

        const string helpers =
            "namespace Vector;\ninternal static class Helpers\n{\n"
            + "    internal static int G(int x) { return x; }\n"
            + "}\n";
        const string policy =
            "namespace Vector;\npublic static class Policy\n{\n"
            + "    public static int F(int x) { return Helpers.G(x); }\n"
            + "}\n";
        SubsetClosure multiFile = ValidateFiles(
            referencePackRoot,
            new[]
            {
                ("src/Helpers.cs", helpers),
                ("src/Policy.cs", policy),
            },
            "Vector.Policy::F(i32)->i32");
        Equal(
            "Vector.Helpers::G(i32)->i32,Vector.Policy::F(i32)->i32",
            string.Join(',', multiFile.Methods.Select(method => method.CanonicalId)),
            "MULTI_FILE_CLOSURE");

        const string multiRootSource =
            "namespace Vector;\npublic static class Case\n{\n"
            + "    public static int A(int x) { return Z(x); }\n"
            + "    public static int B(int x) { return Y(x); }\n"
            + "    private static int Y(int x) { return x; }\n"
            + "    private static int Z(int x) { return x; }\n"
            + "}\n";
        SubsetClosure multiRoot = Validate(
            referencePackRoot,
            multiRootSource,
            "Vector.Case::A(i32)->i32",
            "Vector.Case::B(i32)->i32");
        Equal(
            "Vector.Case::Y(i32)->i32,Vector.Case::B(i32)->i32,"
                + "Vector.Case::Z(i32)->i32,Vector.Case::A(i32)->i32",
            string.Join(',', multiRoot.Methods.Select(method => method.CanonicalId)),
            "MULTI_ROOT_CANONICAL_ORDER");
    }

    private static void ExactTypesLiteralsAndControlAreAccepted(string referencePackRoot)
    {
        foreach ((string sourceType, string methodType) in new[]
        {
            ("bool", "bool"),
            ("int", "i32"),
            ("uint", "u32"),
            ("long", "i64"),
            ("ulong", "u64"),
        })
        {
            string source =
                "namespace Vector;\npublic static class Case\n{\n"
                + $"    public static {sourceType} F({sourceType} x) {{ {sourceType} y = x; return y; }}\n"
                + "}\n";
            _ = Validate(referencePackRoot, source, $"Vector.Case::F({methodType})->{methodType}");
        }

        const string literals =
            "namespace Vector;\n"
            + "public static class Case\n"
            + "{\n"
            + "    public static ulong F(bool choose, uint value, long signed, ulong fallback)\n"
            + "    {\n"
            + "        uint one = 1U;\n"
            + "        long two = 2L;\n"
            + "        ulong three = 3UL;\n"
            + "        if (choose && value == one) { return unchecked((ulong)signed); }\n"
            + "        if (two == 2L) { return fallback; }\n"
            + "        return three;\n"
            + "    }\n"
            + "}\n";
        _ = Validate(referencePackRoot, literals, "Vector.Case::F(bool,u32,i64,u64)->u64");

        const string negativeMinimum =
            "namespace Vector;\n"
            + "public static class Case\n"
            + "{\n"
            + "    public static long F()\n"
            + "    {\n"
            + "        int x = -2147483648;\n"
            + "        return x == -2147483648 ? -9223372036854775808L : 0L;\n"
            + "    }\n"
            + "}\n";
        _ = Validate(referencePackRoot, negativeMinimum, "Vector.Case::F()->i64");

        const string scalarOperations =
            "namespace Vector;\n"
            + "public static class Case\n"
            + "{\n"
            + "    public static int F(bool flag, int x, int y)\n"
            + "    {\n"
            + "        int a = checked(x + y);\n"
            + "        int c = unchecked(a - y);\n"
            + "        int d = checked(c * y);\n"
            + "        int e = checked(d / y);\n"
            + "        int f = unchecked(e % y);\n"
            + "        int g = checked(-f);\n"
            + "        int h = ~g;\n"
            + "        int i = (h & x) | (y ^ h);\n"
            + "        int j = i << y;\n"
            + "        int k = j >> y;\n"
            + "        bool predicate = !(flag && x == y) || x < y;\n"
            + "        return predicate ? k : x;\n"
            + "    }\n"
            + "}\n";
        _ = Validate(
            referencePackRoot,
            scalarOperations,
            "Vector.Case::F(bool,i32,i32)->i32");

        _ = Validate(
            referencePackRoot,
            Wrap("public static long F(long value, int count) { return value << count; }"),
            "Vector.Case::F(i64,i32)->i64");
        _ = Validate(
            referencePackRoot,
            Wrap("public static long F(uint value) { long result = value; return result; }"),
            "Vector.Case::F(u32)->i64");
    }

    private static void DeclarationTypeAndLiteralAdmissionIsClosed(string referencePackRoot)
    {
        ExpectRejected(
            referencePackRoot,
            "using System;\nnamespace Vector;\npublic static class Case { public static int F(int x) { return x; } }\n",
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_DECLARATION");
        ExpectRejected(
            referencePackRoot,
            "namespace Vector;\npublic class Case { public static int F(int x) { return x; } }\n",
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_DECLARATION");
        ExpectRejected(
            referencePackRoot,
            "namespace Vector;\npublic static class Case { private static int Value = 1; public static int F(int x) { return Value; } }\n",
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_INITIALIZATION");
        ExpectRejected(
            referencePackRoot,
            "namespace Vector;\npublic static class Case { static Case() { } public static int F(int x) { return x; } }\n",
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_INITIALIZATION");
        ExpectRejected(
            referencePackRoot,
            "namespace Vector;\npublic static class Case { private static int Value => 1; public static int F(int x) { return Value; } }\n",
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_DECLARATION");
        ExpectRejected(
            referencePackRoot,
            "namespace Vector;\npublic static class Case { public static int F<T>(int x) { return x; } }\n",
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_DECLARATION");

        foreach ((string type, string canonical) in new[]
        {
            ("byte", "i32"),
            ("nint", "i64"),
            ("double", "i64"),
            ("decimal", "i64"),
            ("string", "i32"),
            ("System.Int32", "i32"),
        })
        {
            string source =
                "namespace Vector;\npublic static class Case\n{\n"
                + $"    public static {type} F({type} x) {{ return x; }}\n"
                + "}\n";
            ExpectRejected(
                referencePackRoot,
                source,
                $"Vector.Case::F({canonical})->{canonical}",
                "typecheck",
                "CSHARP_SUBSET_TYPE");
        }

        foreach ((string returnType, string literal, string canonical) in new[]
        {
            ("uint", "1u", "u32"),
            ("int", "0x10", "i32"),
            ("int", "1_0", "i32"),
            ("int", "+1", "i32"),
            ("int", "~1", "i32"),
            ("int", "1 + 2", "i32"),
        })
        {
            string source =
                "namespace Vector;\npublic static class Case\n{\n"
                + $"    public static {returnType} F() {{ return {literal}; }}\n"
                + "}\n";
            ExpectRejected(
                referencePackRoot,
                source,
                $"Vector.Case::F()->{canonical}",
                "typecheck",
                "CSHARP_SUBSET_LITERAL");
        }
    }

    private static void ControlOperationAndConversionAdmissionIsClosed(string referencePackRoot)
    {
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { while (x > 0) { return x; } return x; }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_CONTROL_FLOW");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { var y = x; return y; }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_CONTROL_FLOW");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { x = 1; return x; }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_CONTROL_FLOW");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(bool choose, int x) { int result = x; if (choose) { int y = x; result = y; } else { int y = x; result = y; } return result; }"),
            "Vector.Case::F(bool,i32)->i32",
            "subset",
            "CSHARP_SUBSET_CONTROL_FLOW");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { return x + 1; }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_OVERFLOW_CONTEXT");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static long F(long x, int y) { return checked(x + y); }"),
            "Vector.Case::F(i64,i32)->i64",
            "subset",
            "CSHARP_SUBSET_OPERATION");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static bool F(bool x, bool y) { return x & y; }"),
            "Vector.Case::F(bool,bool)->bool",
            "subset",
            "CSHARP_SUBSET_OPERATION");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x, int y) { return x >>> y; }"),
            "Vector.Case::F(i32,i32)->i32",
            "subset",
            "CSHARP_SUBSET_OPERATION");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x, int y) { x += y; return x; }"),
            "Vector.Case::F(i32,i32)->i32",
            "subset",
            "CSHARP_SUBSET_OPERATION");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(long x) { return checked((int)x); }"),
            "Vector.Case::F(i64)->i32",
            "subset",
            "CSHARP_SUBSET_CHECKED_CONVERSION");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(long x) { return (int)x; }"),
            "Vector.Case::F(i64)->i32",
            "subset",
            "CSHARP_SUBSET_CONVERSION");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { return (int)(object)x; }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_CONVERSION");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { return (System.Int32)x; }"),
            "Vector.Case::F(i32)->i32",
            "typecheck",
            "CSHARP_SUBSET_TYPE");
    }

    private static void ClosurePurityAndInitializationAreClosed(string referencePackRoot)
    {
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { return G(x); } private static int G(int x) { return F(x); }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_CALL");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { return x; } private static int G(int x) { return x; }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_CALL");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { return System.Math.Abs(x); }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_CALL");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { return G(value: x); } private static int G(int value) { return value; }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_CALL");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { return System.Int32.MaxValue; }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_PURITY");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static bool F() { return new int[1].Length == 1; }"),
            "Vector.Case::F()->bool",
            "subset",
            "CSHARP_SUBSET_PURITY");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { System.Console.WriteLine(x); return x; }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_PURITY");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { throw new System.Exception(); }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_ABRUPT");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { try { return x; } finally { } }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_ABRUPT");
        ExpectRejected(
            referencePackRoot,
            Wrap("public static int F(int x) { lock (new object()) { return x; } }"),
            "Vector.Case::F(i32)->i32",
            "subset",
            "CSHARP_SUBSET_ABRUPT");

        const string sourceDeadCall =
            "namespace Vector;\npublic static class Case\n{\n"
            + "    public static int F(int x) { if (x == 0 && x != 0) { return G(x); } return x; }\n"
            + "    private static int G(int x) { return x; }\n"
            + "}\n";
        SubsetClosure closure = Validate(referencePackRoot, sourceDeadCall, "Vector.Case::F(i32)->i32");
        Equal(2, closure.Methods.Length, "SOURCE_DEAD_CLOSURE");
    }

    private static void DefiniteAssignmentAndCfgAccountingAreOwned(string referencePackRoot)
    {
        const string source =
            "namespace Vector;\npublic static class Case\n{\n"
            + "    public static int F(bool choose, int left, int right)\n"
            + "    {\n"
            + "        int result = left;\n"
            + "        if (choose) { result = left; } else { result = right; }\n"
            + "        return result;\n"
            + "    }\n"
            + "}\n";
        SubsetClosure closure = Validate(
            referencePackRoot,
            source,
            "Vector.Case::F(bool,i32,i32)->i32");
        SubsetMethod method = closure.Methods.Single();
        Check(method.OperationCount > 0, "CFG_METHOD_OPERATIONS");
        Check(method.CfgBlockCount >= 5, "CFG_METHOD_BLOCKS");
        Equal(method.OperationCount, closure.OperationCount, "CFG_CLOSURE_OPERATIONS");
        Equal(method.CfgBlockCount, closure.CfgBlockCount, "CFG_CLOSURE_BLOCKS");
    }

    private static void SemanticRowRejectionsAreOwned(string referencePackRoot)
    {
        var rows = new HashSet<string>(StringComparer.Ordinal);
        foreach ((string row, string source, string method, string phase, string code) in new[]
        {
            ("M03", Wrap("public static nint F(nint x) { return x; }"), "Vector.Case::F(i64)->i64", "typecheck", "CSHARP_SUBSET_TYPE"),
            ("M04", Wrap("public static double F(double x) { return x; }"), "Vector.Case::F(i64)->i64", "typecheck", "CSHARP_SUBSET_TYPE"),
            ("M05", Wrap("public static decimal F(decimal x) { return x; }"), "Vector.Case::F(i64)->i64", "typecheck", "CSHARP_SUBSET_TYPE"),
            ("M06", Wrap("public static int[] F(int[] x) { return x; }"), "Vector.Case::F(i32)->i32", "typecheck", "CSHARP_SUBSET_TYPE"),
            ("M15", Wrap("public static System.Numerics.BigInteger F(System.Numerics.BigInteger x) { return x; }"), "Vector.Case::F(i64)->i64", "typecheck", "CSHARP_SUBSET_TYPE"),
            ("M17", Wrap("public static int F(int x, int y) { return System.Math.DivRem(x, y).Quotient; }"), "Vector.Case::F(i32,i32)->i32", "subset", "CSHARP_SUBSET_PURITY"),
            ("M20", Wrap("public static System.Numerics.BigInteger F(System.Numerics.BigInteger x, int count) { return x << count; }"), "Vector.Case::F(i64,i32)->i64", "typecheck", "CSHARP_SUBSET_TYPE"),
            ("M22", Wrap("public static int F(long x) { return checked((int)x); }"), "Vector.Case::F(i64)->i32", "subset", "CSHARP_SUBSET_CHECKED_CONVERSION"),
            ("M23", Wrap("public static int F(int x) { return (int)(object)x; }"), "Vector.Case::F(i32)->i32", "subset", "CSHARP_SUBSET_CONVERSION"),
            ("M24", Wrap("public static int? F(int? x) { return x; }"), "Vector.Case::F(i32)->i32", "typecheck", "CSHARP_SUBSET_TYPE"),
            ("M25", Wrap("public static bool F() { return new int[1].Length == 1; }"), "Vector.Case::F()->bool", "subset", "CSHARP_SUBSET_PURITY"),
            ("M26", Wrap("public static int F(int x) { throw new System.Exception(); }"), "Vector.Case::F(i32)->i32", "subset", "CSHARP_SUBSET_ABRUPT"),
            ("M28", Wrap("public static int F(int x, int y) { return x.CompareTo(y); }"), "Vector.Case::F(i32,i32)->i32", "subset", "CSHARP_SUBSET_CALL"),
            ("M30", "namespace Vector;\npublic static class Case { private static int Value = 1; public static int F(int x) { return Value; } }\n", "Vector.Case::F(i32)->i32", "subset", "CSHARP_SUBSET_INITIALIZATION"),
            ("M31", Wrap("public static int F(int x) { lock (new object()) { return x; } }"), "Vector.Case::F(i32)->i32", "subset", "CSHARP_SUBSET_ABRUPT"),
            ("M32", Wrap("public static async System.Threading.Tasks.Task<int> F(int x) { await System.Threading.Tasks.Task.Yield(); return x; }"), "Vector.Case::F(i32)->i32", "subset", "CSHARP_SUBSET_ABRUPT"),
        })
        {
            Check(rows.Add(row), "SEMANTIC_ROW_DUPLICATE");
            ExpectRejected(referencePackRoot, source, method, phase, code);
        }

        Equal(
            "M03,M04,M05,M06,M15,M17,M20,M22,M23,M24,M25,M26,M28,M30,M31,M32",
            string.Join(',', rows.OrderBy(row => row, StringComparer.Ordinal)),
            "SEMANTIC_ROW_SET");
    }

    private static void LimitsAreInclusiveAndChecked(string referencePackRoot)
    {
        foreach ((uint maximum, string code) in new[]
        {
            (SubsetLimits.MethodClosureMaximum, "CSHARP_LIMIT_METHOD_CLOSURE"),
            (SubsetLimits.SyntaxNodesMaximum, "CSHARP_LIMIT_SYNTAX_NODES"),
            (SubsetLimits.OperationsPerMethodMaximum, "CSHARP_LIMIT_OPERATIONS_PER_METHOD"),
            (SubsetLimits.OperationsPerClosureMaximum, "CSHARP_LIMIT_OPERATIONS_PER_CLOSURE"),
            (SubsetLimits.CfgBlocksPerMethodMaximum, "CSHARP_LIMIT_CFG_BLOCKS_PER_METHOD"),
            (SubsetLimits.CfgBlocksPerClosureMaximum, "CSHARP_LIMIT_CFG_BLOCKS_PER_CLOSURE"),
        })
        {
            Equal(maximum, SubsetLimits.Add(maximum - 1, 1, maximum, code), code + "_BOUNDARY");
            ExpectFailure(
                () => SubsetLimits.Add(maximum, 1, maximum, code),
                FrontendStatus.Rejected,
                "subset",
                code);
        }

        SubsetClosure boundary = Validate(
            referencePackRoot,
            ClosureSource(128),
            "Vector.Case::F000(i32)->i32");
        Equal(128, boundary.Methods.Length, "METHOD_LIMIT_BOUNDARY");
        ExpectRejected(
            referencePackRoot,
            ClosureSource(129),
            "Vector.Case::F000(i32)->i32",
            "subset",
            "CSHARP_LIMIT_METHOD_CLOSURE");
    }

    private static SubsetClosure Validate(
        string referencePackRoot,
        string source,
        params string[] methods)
    {
        return ValidateFiles(
            referencePackRoot,
            new[] { (SourcePath, source) },
            methods);
    }

    private static SubsetClosure ValidateFiles(
        string referencePackRoot,
        (string Path, string Source)[] files,
        params string[] methods)
    {
        Array.Sort(files, static (left, right) => string.CompareOrdinal(left.Path, right.Path));
        Array.Sort(methods, StringComparer.Ordinal);
        Selection selection = SelectionCodec.Validate(new RawSelection(
            "subset-case",
            files.Select(file => file.Path).ToArray(),
            new[] { ContractPath },
            methods));
        var sources = new CapturedSourceSet(files
            .Select(file => new CapturedSourceText(file.Path, file.Source))
            .ToArray());
        RoslynSourceSession sourceSession = RoslynSessionFactory.Parse(selection, sources);
        RoslynCompilationSession compilation = RoslynSessionFactory.Compile(
            selection,
            sourceSession,
            referencePackRoot);
        return CSharpSubset.Validate(selection, compilation);
    }

    private static void ExpectRejected(
        string referencePackRoot,
        string source,
        string method,
        string phase,
        string code)
    {
        ExpectFailure(
            () => Validate(referencePackRoot, source, method),
            FrontendStatus.Rejected,
            phase,
            code);
    }

    private static string Wrap(string members)
    {
        return "namespace Vector;\npublic static class Case\n{\n    "
            + members
            + "\n}\n";
    }

    private static string ClosureSource(int count)
    {
        var source = new StringBuilder();
        source.Append("namespace Vector;\npublic static class Case\n{\n");
        for (int index = 0; index < count; index++)
        {
            string accessibility = index == 0 ? "public" : "private";
            string name = $"F{index:000}";
            string body = index + 1 == count ? "return x;" : $"return F{index + 1:000}(x);";
            source.Append($"    {accessibility} static int {name}(int x) {{ {body} }}\n");
        }

        source.Append("}\n");
        return source.ToString();
    }

    private static void ExpectFailure(
        Action action,
        FrontendStatus status,
        string phase,
        string code)
    {
        try
        {
            action();
        }
        catch (FrontendFailure failure)
        {
            if (failure.Status != status)
            {
                throw new HarnessFailure(code + "_STATUS_" + failure.Status);
            }

            if (!string.Equals(failure.Phase, phase, StringComparison.Ordinal))
            {
                throw new HarnessFailure(code + "_PHASE_" + failure.Phase);
            }

            if (!string.Equals(failure.Code, code, StringComparison.Ordinal))
            {
                throw new HarnessFailure(code + "_CODE_" + failure.Code);
            }

            return;
        }

        throw new HarnessFailure(code + "_ACCEPTED");
    }

    private static void Check(bool condition, string code)
    {
        if (!condition)
        {
            throw new HarnessFailure(code);
        }
    }

    private static void Equal<T>(T expected, T actual, string code)
    {
        if (!EqualityComparer<T>.Default.Equals(expected, actual))
        {
            throw new HarnessFailure(code);
        }
    }
}

internal sealed class HarnessFailure : Exception
{
    internal HarnessFailure(string code)
        : base(code)
    {
        Code = code;
    }

    internal string Code { get; }
}
