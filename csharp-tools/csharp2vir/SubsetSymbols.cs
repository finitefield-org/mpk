using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using System.Threading;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace Mpk.CSharp2Vir;

internal static class SubsetTypeRules
{
    internal static SubsetValueType Validate(
        TypeSyntax syntax,
        SemanticModel semanticModel)
    {
        if (syntax is not PredefinedTypeSyntax predefined
            || !TryMapKeyword(predefined.Keyword, out SubsetValueType type))
        {
            throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_TYPE");
        }

        TypeInfo information;
        try
        {
            information = semanticModel.GetTypeInfo(syntax, CancellationToken.None);
        }
        catch (Exception error) when (SubsetRoslyn.IsAdapterException(error))
        {
            throw FrontendFailure.Toolchain("typecheck", "CSHARP_TOOLCHAIN_ADAPTER");
        }

        ITypeSymbol expected = semanticModel.Compilation.GetSpecialType(SpecialTypeFor(type));
        if (information.Type is null
            || information.ConvertedType is null
            || information.Type.TypeKind == TypeKind.Error
            || information.Type.TypeKind == TypeKind.Dynamic
            || information.Type.NullableAnnotation == NullableAnnotation.Annotated
            || !SymbolEqualityComparer.Default.Equals(information.Type, expected)
            || !SymbolEqualityComparer.Default.Equals(information.ConvertedType, expected)
            || !IsExactPredefinedSymbol(expected, type, semanticModel.Compilation))
        {
            throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_TYPE");
        }

        return type;
    }

    internal static SubsetValueType ValidateSymbol(
        ITypeSymbol? symbol,
        Compilation compilation,
        string phase = "typecheck")
    {
        if (symbol is null || !TryMapSpecialType(symbol.SpecialType, out SubsetValueType type))
        {
            throw FrontendFailure.Rejected(phase, "CSHARP_SUBSET_TYPE");
        }

        ITypeSymbol expected = compilation.GetSpecialType(SpecialTypeFor(type));
        if (symbol.TypeKind == TypeKind.Error
            || symbol.TypeKind == TypeKind.Dynamic
            || symbol.NullableAnnotation == NullableAnnotation.Annotated
            || !SymbolEqualityComparer.Default.Equals(symbol, expected)
            || !IsExactPredefinedSymbol(expected, type, compilation))
        {
            throw FrontendFailure.Rejected(phase, "CSHARP_SUBSET_TYPE");
        }

        return type;
    }

    internal static string CanonicalToken(SubsetValueType type)
    {
        return type switch
        {
            SubsetValueType.Bool => "bool",
            SubsetValueType.I32 => "i32",
            SubsetValueType.U32 => "u32",
            SubsetValueType.I64 => "i64",
            SubsetValueType.U64 => "u64",
            _ => throw FrontendFailure.Internal("typecheck"),
        };
    }

    internal static bool IsInteger(SubsetValueType type)
    {
        return type != SubsetValueType.Bool;
    }

    internal static bool IsSigned(SubsetValueType type)
    {
        return type == SubsetValueType.I32 || type == SubsetValueType.I64;
    }

    private static bool TryMapKeyword(SyntaxToken token, out SubsetValueType type)
    {
        type = token.Kind() switch
        {
            SyntaxKind.BoolKeyword => SubsetValueType.Bool,
            SyntaxKind.IntKeyword => SubsetValueType.I32,
            SyntaxKind.UIntKeyword => SubsetValueType.U32,
            SyntaxKind.LongKeyword => SubsetValueType.I64,
            SyntaxKind.ULongKeyword => SubsetValueType.U64,
            _ => default,
        };

        string expected = token.Kind() switch
        {
            SyntaxKind.BoolKeyword => "bool",
            SyntaxKind.IntKeyword => "int",
            SyntaxKind.UIntKeyword => "uint",
            SyntaxKind.LongKeyword => "long",
            SyntaxKind.ULongKeyword => "ulong",
            _ => string.Empty,
        };
        return expected.Length != 0
            && string.Equals(token.Text, expected, StringComparison.Ordinal)
            && string.Equals(token.ValueText, expected, StringComparison.Ordinal);
    }

    private static bool TryMapSpecialType(SpecialType specialType, out SubsetValueType type)
    {
        switch (specialType)
        {
            case SpecialType.System_Boolean:
                type = SubsetValueType.Bool;
                return true;
            case SpecialType.System_Int32:
                type = SubsetValueType.I32;
                return true;
            case SpecialType.System_UInt32:
                type = SubsetValueType.U32;
                return true;
            case SpecialType.System_Int64:
                type = SubsetValueType.I64;
                return true;
            case SpecialType.System_UInt64:
                type = SubsetValueType.U64;
                return true;
            default:
                type = default;
                return false;
        }
    }

    private static SpecialType SpecialTypeFor(SubsetValueType type)
    {
        return type switch
        {
            SubsetValueType.Bool => SpecialType.System_Boolean,
            SubsetValueType.I32 => SpecialType.System_Int32,
            SubsetValueType.U32 => SpecialType.System_UInt32,
            SubsetValueType.I64 => SpecialType.System_Int64,
            SubsetValueType.U64 => SpecialType.System_UInt64,
            _ => SpecialType.None,
        };
    }

    private static bool IsExactPredefinedSymbol(
        ITypeSymbol symbol,
        SubsetValueType type,
        Compilation compilation)
    {
        return symbol.SpecialType == SpecialTypeFor(type)
            && symbol.ContainingAssembly is not null
            && !SymbolEqualityComparer.Default.Equals(symbol.ContainingAssembly, compilation.Assembly)
            && symbol.DeclaringSyntaxReferences.IsEmpty;
    }
}

internal static class SubsetRoslyn
{
    internal static INamedTypeSymbol GetDeclaredType(
        SemanticModel semanticModel,
        ClassDeclarationSyntax declaration)
    {
        try
        {
            return semanticModel.GetDeclaredSymbol(declaration, CancellationToken.None)
                ?? throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
        }
        catch (FrontendFailure)
        {
            throw;
        }
        catch (Exception error) when (IsAdapterException(error))
        {
            throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
        }
    }

    internal static bool IsAdapterException(Exception error)
    {
        return error is ArgumentException
            || error is InvalidOperationException
            || error is NotSupportedException;
    }
}

internal sealed class SubsetDeclarationSet
{
    internal SubsetDeclarationSet(
        ImmutableArray<DeclaredSubsetMethod> methods,
        ImmutableArray<string> selectedRoots,
        uint syntaxNodeCount)
    {
        Methods = methods;
        SelectedRoots = selectedRoots;
        SyntaxNodeCount = syntaxNodeCount;
    }

    internal ImmutableArray<DeclaredSubsetMethod> Methods { get; }

    internal ImmutableArray<string> SelectedRoots { get; }

    internal uint SyntaxNodeCount { get; }
}

internal static class SubsetDeclarations
{
    internal static SubsetDeclarationSet Validate(
        Selection selection,
        RoslynCompilationSession session)
    {
        uint syntaxNodes = 0;
        var methods = new List<DeclaredSubsetMethod>();
        foreach (SyntaxTree tree in session.Source.SyntaxTrees)
        {
            SemanticModel semanticModel = RoslynPublicApi.GetSemanticModel(session, tree, "typecheck");
            SyntaxNode root;
            try
            {
                root = tree.GetRoot(CancellationToken.None);
            }
            catch (Exception error) when (SubsetRoslyn.IsAdapterException(error))
            {
                throw FrontendFailure.Toolchain("typecheck", "CSHARP_TOOLCHAIN_ADAPTER");
            }

            foreach (SyntaxNode _ in root.DescendantNodesAndSelf(descendIntoTrivia: false))
            {
                syntaxNodes = SubsetLimits.Add(
                    syntaxNodes,
                    1,
                    SubsetLimits.SyntaxNodesMaximum,
                    "CSHARP_LIMIT_SYNTAX_NODES");
            }

            ValidateNoDirectives(root);
            if (root is not CompilationUnitSyntax unit
                || unit.Externs.Count != 0
                || unit.Usings.Count != 0
                || unit.AttributeLists.Count != 0)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
            }

            foreach (MemberDeclarationSyntax member in unit.Members)
            {
                ValidateNamespace(member, semanticModel, methods);
            }
        }

        methods.Sort(static (left, right) => string.CompareOrdinal(left.CanonicalId, right.CanonicalId));
        for (int index = 1; index < methods.Count; index++)
        {
            if (string.Equals(methods[index - 1].CanonicalId, methods[index].CanonicalId, StringComparison.Ordinal))
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
            }
        }

        var byId = new Dictionary<string, DeclaredSubsetMethod>(StringComparer.Ordinal);
        foreach (DeclaredSubsetMethod method in methods)
        {
            byId.Add(method.CanonicalId, method);
        }

        var selectedRoots = ImmutableArray.CreateBuilder<string>(selection.ParsedMethods.Count);
        foreach (CanonicalMethodId selected in selection.ParsedMethods)
        {
            if (!byId.TryGetValue(selected.Canonical, out DeclaredSubsetMethod? method)
                || method.Symbol.DeclaredAccessibility != Accessibility.Public
                || !HasExactMethodModifiers(method.Declaration, SyntaxKind.PublicKeyword))
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
            }

            selectedRoots.Add(selected.Canonical);
        }

        return new SubsetDeclarationSet(
            ImmutableArray.CreateRange(methods),
            selectedRoots.MoveToImmutable(),
            syntaxNodes);
    }

    internal static void ValidateIdentifier(SyntaxToken token, string code = "CSHARP_SUBSET_DECLARATION")
    {
        string text = token.Text;
        if (text.Length == 0
            || !string.Equals(text, token.ValueText, StringComparison.Ordinal)
            || !IsIdentifierStart(text[0]))
        {
            throw FrontendFailure.Rejected("subset", code);
        }

        for (int index = 1; index < text.Length; index++)
        {
            if (!IsIdentifierPart(text[index]))
            {
                throw FrontendFailure.Rejected("subset", code);
            }
        }
    }

    private static void ValidateNoDirectives(SyntaxNode root)
    {
        foreach (SyntaxTrivia trivia in root.DescendantTrivia(descendIntoTrivia: true))
        {
            if (trivia.IsDirective)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
            }
        }
    }

    private static void ValidateNamespace(
        MemberDeclarationSyntax member,
        SemanticModel semanticModel,
        List<DeclaredSubsetMethod> methods)
    {
        SyntaxList<MemberDeclarationSyntax> namespaceMembers;
        switch (member)
        {
            case NamespaceDeclarationSyntax block:
                if (block.Externs.Count != 0
                    || block.Usings.Count != 0)
                {
                    throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
                }

                ValidateNamespaceName(block.Name);
                namespaceMembers = block.Members;
                break;
            case FileScopedNamespaceDeclarationSyntax file:
                if (file.Externs.Count != 0
                    || file.Usings.Count != 0)
                {
                    throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
                }

                ValidateNamespaceName(file.Name);
                namespaceMembers = file.Members;
                break;
            default:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
        }

        foreach (MemberDeclarationSyntax namespaceMember in namespaceMembers)
        {
            if (namespaceMember is not ClassDeclarationSyntax declaration)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
            }

            ValidateClass(declaration, semanticModel, methods);
        }
    }

    private static void ValidateNamespaceName(NameSyntax name)
    {
        switch (name)
        {
            case IdentifierNameSyntax identifier:
                ValidateIdentifier(identifier.Identifier);
                return;
            case QualifiedNameSyntax qualified:
                ValidateNamespaceName(qualified.Left);
                ValidateIdentifier(qualified.Right.Identifier);
                return;
            default:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
        }
    }

    private static void ValidateClass(
        ClassDeclarationSyntax declaration,
        SemanticModel semanticModel,
        List<DeclaredSubsetMethod> methods)
    {
        ValidateIdentifier(declaration.Identifier);
        if (declaration.AttributeLists.Count != 0
            || declaration.TypeParameterList is not null
            || declaration.BaseList is not null
            || declaration.ConstraintClauses.Count != 0
            || !HasExactClassModifiers(declaration))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
        }

        INamedTypeSymbol symbol = SubsetRoslyn.GetDeclaredType(semanticModel, declaration);
        Accessibility expectedAccessibility = declaration.Modifiers.Any(SyntaxKind.PublicKeyword)
            ? Accessibility.Public
            : Accessibility.Internal;
        if (symbol.TypeKind != TypeKind.Class
            || !symbol.IsStatic
            || symbol.IsImplicitlyDeclared
            || symbol.DeclaredAccessibility != expectedAccessibility
            || symbol.Arity != 0
            || symbol.ContainingType is not null
            || symbol.ContainingNamespace is null
            || symbol.ContainingNamespace.IsGlobalNamespace
            || !SymbolEqualityComparer.Default.Equals(symbol.ContainingAssembly, semanticModel.Compilation.Assembly)
            || symbol.DeclaringSyntaxReferences.Length != 1)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
        }

        foreach (MemberDeclarationSyntax member in declaration.Members)
        {
            if (member is FieldDeclarationSyntax
                || member is EventFieldDeclarationSyntax
                || member is ConstructorDeclarationSyntax)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_INITIALIZATION");
            }

            if (member is not MethodDeclarationSyntax method)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
            }

            methods.Add(ValidateMethod(method, symbol, semanticModel));
        }

        if (symbol.StaticConstructors.Length != 0)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_INITIALIZATION");
        }
    }

    private static DeclaredSubsetMethod ValidateMethod(
        MethodDeclarationSyntax declaration,
        INamedTypeSymbol containingType,
        SemanticModel semanticModel)
    {
        ValidateIdentifier(declaration.Identifier);
        if (declaration.Modifiers.Any(SyntaxKind.AsyncKeyword)
            || declaration.DescendantNodes(descendIntoTrivia: false).OfType<YieldStatementSyntax>().Any())
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_ABRUPT");
        }

        SyntaxKind accessibility;
        if (declaration.Modifiers.Any(SyntaxKind.PublicKeyword))
        {
            accessibility = SyntaxKind.PublicKeyword;
        }
        else if (declaration.Modifiers.Any(SyntaxKind.InternalKeyword))
        {
            accessibility = SyntaxKind.InternalKeyword;
        }
        else if (declaration.Modifiers.Any(SyntaxKind.PrivateKeyword))
        {
            accessibility = SyntaxKind.PrivateKeyword;
        }
        else
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
        }

        if (!HasExactMethodModifiers(declaration, accessibility)
            || declaration.AttributeLists.Count != 0
            || declaration.ExplicitInterfaceSpecifier is not null
            || declaration.TypeParameterList is not null
            || declaration.ConstraintClauses.Count != 0
            || declaration.Body is null
            || declaration.ExpressionBody is not null
            || !declaration.SemicolonToken.IsKind(SyntaxKind.None))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
        }

        SubsetValueType resultType = SubsetTypeRules.Validate(declaration.ReturnType, semanticModel);
        IMethodSymbol symbol = RoslynPublicApi.GetDeclaredSymbol(semanticModel, declaration, "typecheck");
        Accessibility expectedAccessibility = accessibility switch
        {
            SyntaxKind.PublicKeyword => Accessibility.Public,
            SyntaxKind.InternalKeyword => Accessibility.Internal,
            SyntaxKind.PrivateKeyword => Accessibility.Private,
            _ => Accessibility.NotApplicable,
        };
        if (symbol.MethodKind != MethodKind.Ordinary
            || !symbol.IsStatic
            || symbol.IsImplicitlyDeclared
            || symbol.DeclaredAccessibility != expectedAccessibility
            || symbol.IsAbstract
            || symbol.IsAsync
            || symbol.IsExtern
            || symbol.IsGenericMethod
            || symbol.IsExtensionMethod
            || symbol.IsOverride
            || symbol.IsVirtual
            || symbol.IsVararg
            || symbol.ReturnsVoid
            || symbol.ReturnsByRef
            || symbol.ReturnsByRefReadonly
            || symbol.PartialDefinitionPart is not null
            || symbol.PartialImplementationPart is not null
            || symbol.DeclaringSyntaxReferences.Length != 1
            || !SymbolEqualityComparer.Default.Equals(symbol.ContainingType, containingType)
            || !SymbolEqualityComparer.Default.Equals(symbol.ContainingAssembly, semanticModel.Compilation.Assembly)
            || SubsetTypeRules.ValidateSymbol(symbol.ReturnType, semanticModel.Compilation) != resultType
            || symbol.Parameters.Length != declaration.ParameterList.Parameters.Count)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
        }

        var parameterTypes = new string[symbol.Parameters.Length];
        for (int index = 0; index < symbol.Parameters.Length; index++)
        {
            ParameterSyntax syntax = declaration.ParameterList.Parameters[index];
            IParameterSymbol parameter = symbol.Parameters[index];
            ValidateIdentifier(syntax.Identifier);
            if (syntax.Modifiers.Any(SyntaxKind.ParamsKeyword)
                || parameter.IsOptional
                || parameter.IsParams
                || parameter.HasExplicitDefaultValue
                || syntax.Default is not null)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
            }

            if (syntax.Type is null
                || syntax.AttributeLists.Count != 0
                || syntax.Modifiers.Count != 0
                || parameter.Ordinal != index
                || parameter.RefKind != RefKind.None
                || !string.Equals(parameter.Name, syntax.Identifier.ValueText, StringComparison.Ordinal))
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
            }

            SubsetValueType syntaxType = SubsetTypeRules.Validate(syntax.Type, semanticModel);
            if (SubsetTypeRules.ValidateSymbol(parameter.Type, semanticModel.Compilation) != syntaxType)
            {
                throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_TYPE");
            }

            parameterTypes[index] = SubsetTypeRules.CanonicalToken(syntaxType);
        }

        string namespaceName = containingType.ContainingNamespace.ToDisplayString(
            SymbolDisplayFormat.FullyQualifiedFormat.WithGlobalNamespaceStyle(
                SymbolDisplayGlobalNamespaceStyle.Omitted));
        if (!IsCanonicalQualifiedName(namespaceName)
            || !string.Equals(containingType.Name, containingType.Name.Normalize(), StringComparison.Ordinal))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
        }

        string canonical = namespaceName
            + "."
            + containingType.Name
            + "::"
            + symbol.Name
            + "("
            + string.Join(',', parameterTypes)
            + ")->"
            + SubsetTypeRules.CanonicalToken(resultType);
        return new DeclaredSubsetMethod(canonical, declaration, symbol, semanticModel);
    }

    private static bool HasExactClassModifiers(ClassDeclarationSyntax declaration)
    {
        if (declaration.Modifiers.Count != 2
            || !declaration.Modifiers.Any(SyntaxKind.StaticKeyword))
        {
            return false;
        }

        bool isPublic = declaration.Modifiers.Any(SyntaxKind.PublicKeyword);
        bool isInternal = declaration.Modifiers.Any(SyntaxKind.InternalKeyword);
        return isPublic != isInternal;
    }

    private static bool HasExactMethodModifiers(
        MethodDeclarationSyntax declaration,
        SyntaxKind accessibility)
    {
        return declaration.Modifiers.Count == 2
            && declaration.Modifiers.Any(SyntaxKind.StaticKeyword)
            && declaration.Modifiers.Any(accessibility);
    }

    private static bool IsCanonicalQualifiedName(string value)
    {
        if (value.Length == 0)
        {
            return false;
        }

        foreach (string component in value.Split('.'))
        {
            if (component.Length == 0 || !IsIdentifierStart(component[0]))
            {
                return false;
            }

            for (int index = 1; index < component.Length; index++)
            {
                if (!IsIdentifierPart(component[index]))
                {
                    return false;
                }
            }
        }

        return true;
    }

    private static bool IsIdentifierStart(char value)
    {
        return (value >= 'A' && value <= 'Z')
            || (value >= 'a' && value <= 'z')
            || value == '_';
    }

    private static bool IsIdentifierPart(char value)
    {
        return IsIdentifierStart(value) || (value >= '0' && value <= '9');
    }
}
