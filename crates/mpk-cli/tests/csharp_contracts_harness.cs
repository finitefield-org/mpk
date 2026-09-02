using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace Mpk.CSharp2Vir;

internal static class ContractHarness
{
    private const string SourcePath = "src/Case.cs";
    private const string ContractPath = "contracts/case.json";

    public static int Main(string[] args)
    {
        if (args.Length != 1)
        {
            Console.Error.Write("CSHARP_CONTRACT_TEST_USAGE\n");
            return 1;
        }

        try
        {
            string referencePackRoot = args[0];
            FrozenContractVectorNormalizesExactly(referencePackRoot);
            StrictJsonShapeIdentityAndRawClaimsAreClosed(referencePackRoot);
            AttachmentIsExactAndSelectionBound(referencePackRoot);
            SuccessorExpressionTypingIsExact(referencePackRoot);
            ContractLimitsRejectBeforeExcessRetention(referencePackRoot);
            SemanticRowM34IsOwned(referencePackRoot);
            return 0;
        }
        catch (HarnessFailure failure)
        {
            Console.Error.Write("CSHARP_CONTRACT_TEST_" + failure.Code + "\n");
            return 1;
        }
        catch (Exception)
        {
            Console.Error.Write("CSHARP_CONTRACT_TEST_UNEXPECTED\n");
            return 1;
        }
    }

    private static void FrozenContractVectorNormalizesExactly(string referencePackRoot)
    {
        const string source =
            "namespace Example.Payment;\n"
            + "public static class Policy\n"
            + "{\n"
            + "    public static bool Approved(long reserve, long debit)\n"
            + "    {\n"
            + "        return reserve >= debit;\n"
            + "    }\n"
            + "}\n";
        const string method = "Example.Payment.Policy::Approved(i64,i64)->bool";
        const string vectorSourcePath = "src/Policy.cs";
        const string vectorContractPath = "contracts/approved.json";
        const string input =
            "{\n"
            + "  \"schema\": \"mpk.csharp.contract.v0\",\n"
            + "  \"semantic_profile\": \"mpk.csharp.scalar.v0\",\n"
            + "  \"method\": \"Example.Payment.Policy::Approved(i64,i64)->bool\",\n"
            + "  \"requires\": [\n"
            + "    {\"op\":\"signed_ge\",\"args\":[{\"parameter\":\"reserve\"},{\"parameter\":\"debit\"}]},\n"
            + "    {\"op\":\"signed_ge\",\"args\":[{\"parameter\":\"reserve\"},{\"int\":{\"decimal\":\"0\",\"type\":\"i64\"}}]}\n"
            + "  ],\n"
            + "  \"ensures\": [{\"op\":\"eq\",\"args\":[{\"result\":0},{\"bool\":true}]}],\n"
            + "  \"modifies\": [],\n"
            + "  \"abrupt_completion\": \"forbidden\",\n"
            + "  \"termination\": \"total\"\n"
            + "}\n";
        var context = new ContractContext(
            referencePackRoot,
            "payment-policy",
            source,
            new[] { method },
            new[] { vectorContractPath },
            vectorSourcePath);
        ContractSet set = context.Attach((vectorContractPath, Encoding.UTF8.GetBytes(input)));
        Equal(
            "d5033138bd8c53eee3901d0d1852ed4c1b1a85686cf2a68f01effb0b8c70dfcd",
            set.SelectionSha256,
            "VECTOR_SELECTION_HASH");
        Equal(1, set.Contracts.Count, "VECTOR_COUNT");
        AttachedContract attached = set.Contracts[0];
        Equal(
            "6684361a15dc454a8172d7e515dd6a3a49ec1ff8faae00bc12d958eae8982228",
            attached.Sidecar.SidecarSha256,
            "VECTOR_SIDECAR_HASH");
        Equal(440, attached.Sidecar.CanonicalBytes.Length, "VECTOR_SIDECAR_LENGTH");
        Check(
            !string.Equals(
                attached.RawInputSha256,
                attached.Sidecar.SidecarSha256,
                StringComparison.Ordinal),
            "VECTOR_RAW_DISTINCT");
        const string expectedSidecar =
            "{\"abrupt_completion\":\"forbidden\",\"ensures\":[{\"args\":[{\"result\":0},{\"bool\":true}],\"op\":\"eq\"}],"
            + "\"method\":\"Example.Payment.Policy::Approved(i64,i64)->bool\",\"modifies\":[],\"requires\":["
            + "{\"args\":[{\"parameter\":\"reserve\"},{\"parameter\":\"debit\"}],\"op\":\"signed_ge\"},"
            + "{\"args\":[{\"parameter\":\"reserve\"},{\"int\":{\"decimal\":\"0\",\"type\":\"i64\"}}],\"op\":\"signed_ge\"}],"
            + "\"schema\":\"mpk.csharp.contract.v0\",\"semantic_profile\":\"mpk.csharp.scalar.v0\",\"termination\":\"total\"}";
        Equal(
            expectedSidecar,
            Encoding.UTF8.GetString(attached.Sidecar.CanonicalBytes),
            "VECTOR_SIDECAR_CANONICAL");

        NormalizedContract normalized = attached.Normalized;
        Equal(
            "b88b13b2041782b1728563e9ae3d34bf2334771fb05171fa4ba38a8c1ffb0cab",
            normalized.ContractHash,
            "VECTOR_CONTRACT_HASH");
        Equal(1_151, normalized.HashPayloadBytes.Length, "VECTOR_CONTRACT_LENGTH");
        const string expectedNormalized =
            "{\"contract_hash\":\"b88b13b2041782b1728563e9ae3d34bf2334771fb05171fa4ba38a8c1ffb0cab\","
            + "\"ensures\":[{\"lhs\":{\"result\":0},\"op\":\"eq\",\"rhs\":{\"bool\":true}}],"
            + "\"function_id\":\"Example.Payment.Policy::Approved(i64,i64)->bool\",\"loops\":[],\"modifies\":[],\"panic\":\"forbidden\","
            + "\"requires\":[{\"lhs\":{\"var\":\"arg0\"},\"op\":\"signed_ge\",\"rhs\":{\"var\":\"arg1\"}},"
            + "{\"lhs\":{\"var\":\"arg0\"},\"op\":\"signed_ge\",\"rhs\":{\"int\":{\"signed\":true,\"value\":\"0\",\"width\":64}}}],"
            + "\"semantic_context\":{\"profile_entry_sha256\":\"d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac\","
            + "\"profile_registry\":{\"id\":\"mpk.semantic_profile.registry.v1\",\"registry_sha256\":\"fc102411ac266a38db27f904df2ca6f794bca1a216fff12377d88990e653c557\","
            + "\"revision\":3,\"schema\":\"mpk.semantic_profile.registry.v1\"},\"semantic_parameters\":{\"schema\":\"mpk.semantic_parameters.csharp_scalar.v0\","
            + "\"value\":{\"check_overflow_default\":false,\"documentation_mode\":\"none\",\"language_version\":\"14.0\",\"nullable_context\":\"disable\","
            + "\"optimization\":\"release\",\"platform\":\"x64\",\"pointer_width\":64,\"preprocessor_symbols\":[],\"source_kind\":\"regular\","
            + "\"target_framework\":\"net10.0\",\"target_id\":\"linux-x64\",\"unsafe\":false}},\"semantic_profile\":\"mpk.csharp.scalar.v0\","
            + "\"source_language\":\"csharp\"},\"termination\":\"total\",\"unit_id\":\"payment-policy\"}";
        Equal(
            expectedNormalized,
            Encoding.UTF8.GetString(normalized.CanonicalBytes),
            "VECTOR_NORMALIZED_CANONICAL");
        Equal("payment-policy", normalized.UnitId, "VECTOR_UNIT");
        Equal(method, normalized.FunctionId, "VECTOR_FUNCTION");
        Equal("arg0", VariableName(normalized.Requires[0].Arguments[0]), "VECTOR_ARG0");
        Equal("arg1", VariableName(normalized.Requires[0].Arguments[1]), "VECTOR_ARG1");
    }

    private static void StrictJsonShapeIdentityAndRawClaimsAreClosed(string referencePackRoot)
    {
        const string source =
            "namespace Vector;\npublic static class Case\n{\n"
            + "    public static bool F(bool value) { return value; }\n}\n";
        const string method = "Vector.Case::F(bool)->bool";
        var context = new ContractContext(
            referencePackRoot,
            "contract-json",
            source,
            new[] { method },
            new[] { ContractPath });
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes("{\"schema\":"))),
            "CSHARP_CONTRACT_JSON");
        ExpectRejected(
            () => context.Attach((ContractPath, new byte[] { 0xef, 0xbb, 0xbf, (byte)'{' })),
            "CSHARP_CONTRACT_JSON");
        ExpectRejected(
            () => context.Attach((ContractPath, new byte[] { 0xff })),
            "CSHARP_CONTRACT_JSON");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(DefaultContract(method) + "x"))),
            "CSHARP_CONTRACT_JSON");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(
                DefaultContract(method).Replace("{", "{/*comment*/", StringComparison.Ordinal)))),
            "CSHARP_CONTRACT_JSON");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(
                DefaultContract(method).Replace("}", ",}", StringComparison.Ordinal)))),
            "CSHARP_CONTRACT_JSON");

        string duplicate = DefaultContract(method).Replace(
            "{\"schema\":\"mpk.csharp.contract.v0\",",
            "{\"schema\":\"mpk.csharp.contract.v0\",\"schema\":\"mpk.csharp.contract.v0\",");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(duplicate))),
            "CSHARP_CONTRACT_DUPLICATE");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
                method,
                "[]",
                "[{\"bool\":true,\"bool\":false}]")))),
            "CSHARP_CONTRACT_DUPLICATE");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(DefaultContract(method).Replace(
                "\"termination\":\"total\"",
                "\"unknown\":0,\"termination\":\"total\"")))),
            "CSHARP_CONTRACT_SHAPE");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(DefaultContract(method).Replace(
                ",\"termination\":\"total\"",
                string.Empty)))),
            "CSHARP_CONTRACT_SHAPE");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(DefaultContract(method).Replace(
                "mpk.csharp.contract.v0",
                "mpk.csharp.contract.v1")))),
            "CSHARP_CONTRACT_IDENTITY");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(DefaultContract(method).Replace(
                "mpk.csharp.scalar.v0",
                "mpk.go.fixed.v0")))),
            "CSHARP_CONTRACT_IDENTITY");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(DefaultContract(method).Replace(
                "\"modifies\":[]",
                "\"modifies\":[\"state\"]")))),
            "CSHARP_CONTRACT_IDENTITY");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(DefaultContract(method).Replace(
                "\"abrupt_completion\":\"forbidden\"",
                "\"abrupt_completion\":\"allowed\"")))),
            "CSHARP_CONTRACT_IDENTITY");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(DefaultContract(method).Replace(
                "\"termination\":\"total\"",
                "\"termination\":\"partial\"")))),
            "CSHARP_CONTRACT_IDENTITY");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(method, "[]", "[]")))),
            "CSHARP_CONTRACT_IDENTITY");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(DefaultContract(method).Replace(
                "\"termination\":\"total\"",
                "\"contract_hash\":\"claimed\",\"termination\":\"total\"")))),
            "CSHARP_CONTRACT_SHAPE");
    }

    private static void AttachmentIsExactAndSelectionBound(string referencePackRoot)
    {
        const string source =
            "namespace Vector;\npublic static class Case\n{\n"
            + "    public static bool F(bool value) { return G(value); }\n"
            + "    private static bool G(bool value) { return value; }\n}\n";
        const string root = "Vector.Case::F(bool)->bool";
        const string callee = "Vector.Case::G(bool)->bool";
        string[] paths = { "contracts/000.json", "contracts/001.json" };
        var complete = new ContractContext(
            referencePackRoot,
            "contract-attachment",
            source,
            new[] { root },
            paths);
        ContractSet set = complete.Attach(
            (paths[0], Encoding.UTF8.GetBytes(DefaultContract(root))),
            (paths[1], Encoding.UTF8.GetBytes(DefaultContract(callee))));
        Equal(2, set.Contracts.Count, "ATTACH_COUNT");
        Equal(callee, set.Contracts[0].Normalized.FunctionId, "ATTACH_CALLEE_FIRST");
        Equal(root, set.Contracts[1].Normalized.FunctionId, "ATTACH_ROOT_SECOND");

        var missing = new ContractContext(
            referencePackRoot,
            "contract-missing",
            source,
            new[] { root },
            new[] { ContractPath });
        ExpectRejected(
            () => missing.Attach((ContractPath, Encoding.UTF8.GetBytes(DefaultContract(root)))),
            "CSHARP_CONTRACT_MISSING");

        var duplicate = new ContractContext(
            referencePackRoot,
            "contract-duplicate",
            "namespace Vector;\npublic static class Case\n{\n"
                + "    public static bool F(bool value) { return value; }\n}\n",
            new[] { root },
            paths);
        ExpectRejected(
            () => duplicate.Attach(
                (paths[0], Encoding.UTF8.GetBytes(DefaultContract(root))),
                (paths[1], Encoding.UTF8.GetBytes(DefaultContract(root)))),
            "CSHARP_CONTRACT_DUPLICATE");
        ExpectRejected(
            () => duplicate.Attach(
                (paths[0], Encoding.UTF8.GetBytes(DefaultContract(
                    "Vector.Case::H(bool)->bool"))),
                (paths[1], Encoding.UTF8.GetBytes(DefaultContract(root)))),
            "CSHARP_CONTRACT_UNUSED");
        ExpectRejected(
            () => duplicate.Attach(
                (paths[0], Encoding.UTF8.GetBytes(DefaultContract("not a method"))),
                (paths[1], Encoding.UTF8.GetBytes(DefaultContract(root)))),
            "CSHARP_CONTRACT_IDENTITY");

        Selection valid = complete.Selection;
        var parsed = new CanonicalMethodId[valid.ParsedMethods.Count];
        for (int index = 0; index < parsed.Length; index++)
        {
            parsed[index] = valid.ParsedMethods[index];
        }

        var forged = new Selection(
            valid.Raw,
            parsed,
            valid.CanonicalBytes.ToArray(),
            new string('0', 64));
        CapturedSnapshot forgedSnapshot = complete.Snapshot(
            forged,
            (paths[0], Encoding.UTF8.GetBytes(DefaultContract(root))),
            (paths[1], Encoding.UTF8.GetBytes(DefaultContract(callee))));
        ExpectRejected(
            () => CSharpContracts.Attach(forged, forgedSnapshot, complete.Closure),
            "CSHARP_CONTRACT_HASH");
    }

    private static void SuccessorExpressionTypingIsExact(string referencePackRoot)
    {
        const string source =
            "namespace Vector;\npublic static class Case\n{\n"
            + "    public static bool All(bool b, int i, uint u, long l, ulong ul) { return b; }\n}\n";
        const string method = "Vector.Case::All(bool,i32,u32,i64,u64)->bool";
        var context = new ContractContext(
            referencePackRoot,
            "contract-types",
            source,
            new[] { method },
            new[] { ContractPath });
        var accepted = new List<string>
        {
            Op("not", Parameter("b")),
            Op("not_eq", Bool(true), Bool(false)),
            Op("and", Parameter("b"), Bool(true)),
            Op("or", Bool(false), Parameter("b"), Bool(true)),
            Eq(Op("bv_neg", Parameter("i")), Integer("-2147483648", "i32")),
            Eq(Op("bv_neg", Parameter("u")), Integer("4294967295", "u32")),
            Eq(Op("bv_not", Parameter("l")), Integer("-9223372036854775808", "i64")),
            Eq(Op("bv_not", Parameter("ul")), Integer("18446744073709551615", "u64")),
            Op("signed_lt", Parameter("i"), Integer("0", "i32")),
            Op("signed_le", Parameter("l"), Integer("0", "i64")),
            Op("signed_gt", Parameter("i"), Integer("-1", "i32")),
            Op("signed_ge", Parameter("l"), Integer("0", "i64")),
            Op("unsigned_lt", Parameter("u"), Integer("1", "u32")),
            Op("unsigned_le", Parameter("ul"), Integer("1", "u64")),
            Op("unsigned_gt", Parameter("u"), Integer("0", "u32")),
            Op("unsigned_ge", Parameter("ul"), Integer("0", "u64")),
        };
        foreach (string operation in new[]
        {
            "bv_add", "bv_sub", "bv_mul", "bv_and", "bv_or", "bv_xor",
        })
        {
            accepted.Add(Eq(Op(operation, Parameter("i"), Integer("1", "i32")), Parameter("i")));
        }

        accepted.Add(Eq(Op("bv_shl", Parameter("i"), Parameter("ul")), Parameter("i")));
        accepted.Add(Eq(Op("bv_ashr", Parameter("l"), Parameter("u")), Parameter("l")));
        accepted.Add(Eq(Op("bv_lshr", Parameter("ul"), Parameter("i")), Parameter("ul")));
        string broad = Contract(
            method,
            "[" + string.Join(',', accepted) + "]",
            "[" + Eq(Result(), Bool(true)) + "]");
        ContractSet set = context.Attach((ContractPath, Encoding.UTF8.GetBytes(broad)));
        Equal(accepted.Count, set.Contracts[0].Normalized.Requires.Count, "TYPE_ACCEPT_COUNT");

        const string resultSource =
            "namespace Vector;\npublic static class ResultCase\n{\n"
            + "    public static int R(int value) { return value; }\n}\n";
        const string resultMethod = "Vector.ResultCase::R(i32)->i32";
        var resultContext = new ContractContext(
            referencePackRoot,
            "contract-result-type",
            resultSource,
            new[] { resultMethod },
            new[] { ContractPath });
        ContractSet resultSet = resultContext.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
            resultMethod,
            "[]",
            "[" + Eq(Result(), Parameter("value")) + "]"))));
        Check(
            resultSet.Contracts[0].Normalized.Ensures[0].Arguments[0].Type
                == SubsetValueType.I32,
            "TYPE_RESULT_I32");
        ExpectRejected(
            () => resultContext.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
                resultMethod,
                "[]",
                "[" + Eq("{\"result\":1}", Parameter("value")) + "]")))),
            "CSHARP_CONTRACT_TYPE",
            "RESULT_INDEX");

        foreach ((string expression, string caseName) in new[]
        {
            (Parameter("missing"), "UNRESOLVED"),
            (Result(), "RESULT_IN_REQUIRES"),
            (Parameter("i"), "NON_BOOLEAN_CLAUSE"),
            (Eq(Parameter("i"), Parameter("u")), "MIXED_EQ"),
            (Op("signed_lt", Parameter("u"), Parameter("u")), "SIGNEDNESS_SIGNED"),
            (Op("unsigned_lt", Parameter("i"), Parameter("i")), "SIGNEDNESS_UNSIGNED"),
            (Op("not", Parameter("i")), "UNARY_BOOL"),
            (Op("and", Parameter("b"), Parameter("i")), "NARY_BOOL"),
            (Eq(Op("bv_shl", Parameter("i"), Parameter("b")), Parameter("i")), "SHIFT_RHS"),
            (Eq(Op("bv_ashr", Parameter("u"), Parameter("i")), Parameter("u")), "ASHR_SIGN"),
            (Eq(Op("bv_lshr", Parameter("i"), Parameter("i")), Parameter("i")), "LSHR_SIGN"),
        })
        {
            ExpectRejected(
                () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
                    method,
                    "[" + expression + "]",
                    "[" + Bool(true) + "]")))),
                "CSHARP_CONTRACT_TYPE",
                caseName);
        }

        foreach ((string value, string type) in new[]
        {
            ("+1", "i32"), ("01", "i32"), ("-0", "i32"), ("1_0", "i32"),
            ("2147483648", "i32"), ("-2147483649", "i32"), ("4294967296", "u32"),
            ("-1", "u32"), ("9223372036854775808", "i64"),
            ("-9223372036854775809", "i64"), ("18446744073709551616", "u64"),
        })
        {
            ExpectRejected(
                () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
                    method,
                    "[]",
                    "[" + Eq(Integer(value, type), Integer("0", type)) + "]")))),
                "CSHARP_CONTRACT_TYPE",
                "INTEGER_" + type + "_" + value);
        }

        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
                method,
                "[]",
                "[" + Eq(Integer("0", "i16"), Integer("0", "i16")) + "]")))),
            "CSHARP_CONTRACT_TYPE",
            "INTEGER_TYPE");
        foreach (string expression in new[]
        {
            Op("div", Parameter("i"), Parameter("i")),
            Op("+", Parameter("i"), Parameter("i")),
            Op("convert", Parameter("i")),
            Op("eq", Parameter("i")),
            Op("not", Parameter("b"), Parameter("b")),
            Op("and", Parameter("b")),
        })
        {
            ExpectRejected(
                () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
                    method,
                    "[]",
                    "[" + expression + "]")))),
                "CSHARP_CONTRACT_OPERATOR");
        }

        string sixtyFive = Op("and", Enumerable.Repeat(Bool(true), 65).ToArray());
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
                method,
                "[]",
                "[" + sixtyFive + "]")))),
            "CSHARP_CONTRACT_OPERATOR",
            "NARY_65");
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
                method,
                "[]",
                "[{\"call\":\"F\"}]")))),
            "CSHARP_CONTRACT_SHAPE",
            "ARBITRARY_EXPRESSION");
    }

    private static void ContractLimitsRejectBeforeExcessRetention(string referencePackRoot)
    {
        const string source =
            "namespace Vector;\npublic static class Case\n{\n"
            + "    public static bool F(bool value) { return value; }\n}\n";
        const string method = "Vector.Case::F(bool)->bool";
        var context = new ContractContext(
            referencePackRoot,
            "contract-limits",
            source,
            new[] { method },
            new[] { ContractPath });

        string clauses64 = string.Join(',', Enumerable.Repeat(Bool(true), 64));
        context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(method, "[]", "[" + clauses64 + "]"))));
        string requires64 = string.Join(',', Enumerable.Repeat(Bool(true), 64));
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
                method,
                "[" + requires64 + "]",
                "[" + Bool(true) + "]")))),
            "CSHARP_LIMIT_CONTRACT_CLAUSES");

        context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
            method,
            "[]",
            "[" + NodeClauses(1_024) + "]"))));
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
                method,
                "[]",
                "[" + NodeClauses(1_025) + "]")))),
            "CSHARP_LIMIT_CONTRACT_NODES_PER_METHOD");

        context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
            method,
            "[]",
            "[" + NestedNot(32) + "]"))));
        ExpectRejected(
            () => context.Attach((ContractPath, Encoding.UTF8.GetBytes(Contract(
                method,
                "[]",
                "[" + NestedNot(33) + "]")))),
            "CSHARP_LIMIT_CONTRACT_DEPTH");

        ContractContext closure8 = ClosureContext(referencePackRoot, 8, "contract-closure-8");
        (string Path, byte[] Bytes)[] exact = ClosureContracts(8, extraNode: false);
        ContractSet exactSet = closure8.Attach(exact);
        Equal(8, exactSet.Contracts.Count, "CLOSURE_NODE_BOUNDARY");
        ContractContext closure9 = ClosureContext(referencePackRoot, 9, "contract-closure-9");
        ExpectRejected(
            () => closure9.Attach(ClosureContracts(9, extraNode: true)),
            "CSHARP_LIMIT_CONTRACT_NODES_PER_CLOSURE");
    }

    private static void SemanticRowM34IsOwned(string referencePackRoot)
    {
        const string source =
            "namespace Vector;\npublic static class Case\n{\n"
            + "    /// Contract prose is not a contract input.\n"
            + "    public static bool F(bool value) { return value; }\n}\n";
        const string method = "Vector.Case::F(bool)->bool";
        var context = new ContractContext(
            referencePackRoot,
            "semantic-row-m34",
            source,
            new[] { method },
            new[] { ContractPath });
        ContractSet set = context.Attach((ContractPath, Encoding.UTF8.GetBytes(DefaultContract(method))));
        Equal(method, set.Contracts.Single().Normalized.FunctionId, "M34_TYPED_SIDECAR");
    }

    private static ContractContext ClosureContext(
        string referencePackRoot,
        int count,
        string compilation)
    {
        var source = new StringBuilder("namespace Vector;\npublic static class Chain\n{\n");
        for (int index = 0; index < count; index++)
        {
            string visibility = index == 0 ? "public" : "private";
            string body = index + 1 == count
                ? "return value;"
                : $"return F{index + 1:000}(value);";
            source.Append($"    {visibility} static bool F{index:000}(bool value) {{ {body} }}\n");
        }

        source.Append("}\n");
        string[] paths = Enumerable.Range(0, count)
            .Select(index => $"contracts/contract{index:000}.json")
            .ToArray();
        return new ContractContext(
            referencePackRoot,
            compilation,
            source.ToString(),
            new[] { "Vector.Chain::F000(bool)->bool" },
            paths);
    }

    private static (string Path, byte[] Bytes)[] ClosureContracts(int count, bool extraNode)
    {
        var contracts = new (string Path, byte[] Bytes)[count];
        for (int index = 0; index < count; index++)
        {
            int nodes = index < 8 ? 1_024 : 1;
            if (extraNode && index == count - 1)
            {
                nodes = 1;
            }

            string method = $"Vector.Chain::F{index:000}(bool)->bool";
            string clauses = nodes == 1 ? Bool(true) : NodeClauses(nodes);
            contracts[index] = (
                $"contracts/contract{index:000}.json",
                Encoding.UTF8.GetBytes(Contract(method, "[]", "[" + clauses + "]")));
        }

        return contracts;
    }

    private static string NodeClauses(int nodes)
    {
        if (nodes < 1 || nodes > 1_025)
        {
            throw new HarnessFailure("NODE_FIXTURE_RANGE");
        }

        var clauses = new List<string>();
        int remaining = nodes;
        while (remaining >= 65)
        {
            clauses.Add(Op("and", Enumerable.Repeat(Bool(true), 64).ToArray()));
            remaining -= 65;
        }

        if (remaining == 1)
        {
            clauses.Add(Bool(true));
        }
        else if (remaining >= 3)
        {
            clauses.Add(Op("and", Enumerable.Repeat(Bool(true), remaining - 1).ToArray()));
        }
        else if (remaining != 0)
        {
            throw new HarnessFailure("NODE_FIXTURE_SHAPE");
        }

        return string.Join(',', clauses);
    }

    private static string NestedNot(int depth)
    {
        string expression = Bool(true);
        for (int index = 1; index < depth; index++)
        {
            expression = Op("not", expression);
        }

        return expression;
    }

    private static string DefaultContract(string method)
    {
        return Contract(method, "[]", "[" + Bool(true) + "]");
    }

    private static string Contract(string method, string requires, string ensures)
    {
        return "{\"schema\":\"mpk.csharp.contract.v0\","
            + "\"semantic_profile\":\"mpk.csharp.scalar.v0\","
            + "\"method\":\"" + method + "\","
            + "\"requires\":" + requires + ","
            + "\"ensures\":" + ensures + ","
            + "\"modifies\":[],"
            + "\"abrupt_completion\":\"forbidden\","
            + "\"termination\":\"total\"}";
    }

    private static string Parameter(string name) => "{\"parameter\":\"" + name + "\"}";

    private static string Result() => "{\"result\":0}";

    private static string Bool(bool value) => value ? "{\"bool\":true}" : "{\"bool\":false}";

    private static string Integer(string value, string type)
    {
        return "{\"int\":{\"decimal\":\"" + value + "\",\"type\":\"" + type + "\"}}";
    }

    private static string Eq(string left, string right) => Op("eq", left, right);

    private static string Op(string operation, params string[] arguments)
    {
        return "{\"op\":\"" + operation + "\",\"args\":[" + string.Join(',', arguments) + "]}";
    }

    private static string VariableName(NormalizedContractExpression expression)
    {
        if (expression.Kind != NormalizedContractExpressionKind.Variable)
        {
            throw new HarnessFailure("NOT_VARIABLE");
        }

        return "arg" + expression.Index;
    }

    private static void ExpectRejected(Action action, string code, string? detail = null)
    {
        try
        {
            action();
        }
        catch (FrontendFailure failure)
        {
            if (failure.Status != FrontendStatus.Rejected
                || !string.Equals(failure.Phase, "subset", StringComparison.Ordinal)
                || !string.Equals(failure.Code, code, StringComparison.Ordinal))
            {
                throw new HarnessFailure(
                    (detail ?? code)
                    + "_GOT_"
                    + failure.Status
                    + "_"
                    + failure.Phase
                    + "_"
                    + failure.Code);
            }

            return;
        }

        throw new HarnessFailure((detail ?? code) + "_ACCEPTED");
    }

    private static void Check(bool condition, string code)
    {
        if (!condition)
        {
            throw new HarnessFailure(code);
        }
    }

    private static void Equal<T>(T expected, T actual, string code)
        where T : IEquatable<T>
    {
        if (!expected.Equals(actual))
        {
            throw new HarnessFailure(code);
        }
    }

    private sealed class ContractContext
    {
        private readonly string source;
        private readonly string sourcePath;
        private readonly string[] contractPaths;

        internal ContractContext(
            string referencePackRoot,
            string compilation,
            string source,
            string[] selectedMethods,
            string[] contractPaths,
            string sourcePath = SourcePath)
        {
            this.source = source;
            this.sourcePath = sourcePath;
            this.contractPaths = contractPaths;
            Selection = SelectionCodec.Validate(new RawSelection(
                compilation,
                new[] { sourcePath },
                contractPaths,
                selectedMethods));
            CapturedSnapshot snapshot = Snapshot(
                Selection,
                contractPaths.Select(path => (path, Encoding.UTF8.GetBytes("{}"))).ToArray());
            CapturedSourceSet sources = SourceTransport.Validate(snapshot);
            RoslynSourceSession sourceSession = RoslynSessionFactory.Parse(Selection, sources);
            RoslynCompilationSession compilationSession = RoslynSessionFactory.Compile(
                Selection,
                sourceSession,
                referencePackRoot);
            Closure = CSharpSubset.Validate(Selection, compilationSession);
        }

        internal Selection Selection { get; }

        internal SubsetClosure Closure { get; }

        internal ContractSet Attach(params (string Path, byte[] Bytes)[] contracts)
        {
            return CSharpContracts.Attach(Selection, Snapshot(Selection, contracts), Closure);
        }

        internal CapturedSnapshot Snapshot(
            Selection selection,
            params (string Path, byte[] Bytes)[] contracts)
        {
            if (contracts.Length != contractPaths.Length)
            {
                throw new HarnessFailure("SNAPSHOT_CONTRACT_COUNT");
            }

            var files = new CapturedFile[contracts.Length + 1];
            files[0] = new CapturedFile(
                CapturedInputKind.Source,
                sourcePath,
                Encoding.UTF8.GetBytes(source));
            for (int index = 0; index < contracts.Length; index++)
            {
                if (!string.Equals(contracts[index].Path, contractPaths[index], StringComparison.Ordinal))
                {
                    throw new HarnessFailure("SNAPSHOT_CONTRACT_ORDER");
                }

                files[index + 1] = new CapturedFile(
                    CapturedInputKind.Contract,
                    contracts[index].Path,
                    contracts[index].Bytes);
            }

            return new CapturedSnapshot(selection, files);
        }
    }

    private sealed class HarnessFailure : Exception
    {
        internal HarnessFailure(string code)
            : base(code)
        {
            Code = code;
        }

        internal string Code { get; }
    }
}
